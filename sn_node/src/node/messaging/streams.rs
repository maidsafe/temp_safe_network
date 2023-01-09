// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    core::NodeContext, flow_ctrl::fault_detection::FaultsCmd, messaging::Peers, Cmd, Error, MyNode,
    Result,
};

use sn_fault_detection::IssueType;
use sn_interface::{
    messaging::{
        data::ClientDataResponse,
        system::{NodeDataResponse, NodeMsg},
        Dst, MsgId, MsgKind, WireMsg,
    },
    network_knowledge::NetworkKnowledge,
    types::Peer,
};

use qp2p::{SendStream, UsrMsgBytes};

use bytes::Bytes;
use lazy_static::lazy_static;
use std::{env::var, str::FromStr};
use tokio::time::{timeout, Duration};
use xor_name::XorName;

/// Environment variable to set timeout value (in seconds) for data queries
/// forwarded to Adults. Default value (`NODE_RESPONSE_DEFAULT_TIMEOUT`) is otherwise used.
const ENV_NODE_RESPONSE_TIMEOUT: &str = "SN_NODE_RESPONSE_TIMEOUT";

// Default timeout period set for data queries forwarded to Adult.
// TODO: how to determine this time properly?
const NODE_RESPONSE_DEFAULT_TIMEOUT: Duration = Duration::from_secs(70);

lazy_static! {
    static ref NODE_RESPONSE_TIMEOUT: Duration = match var(ENV_NODE_RESPONSE_TIMEOUT)
        .map(|v| u64::from_str(&v))
    {
        Ok(Ok(secs)) => {
            let timeout = Duration::from_secs(secs);
            info!("{ENV_NODE_RESPONSE_TIMEOUT} env var set, Node data query response timeout set to {timeout:?}");
            timeout
        }
        Ok(Err(err)) => {
            warn!(
                "Failed to parse {ENV_NODE_RESPONSE_TIMEOUT} value, using \
                default value ({NODE_RESPONSE_DEFAULT_TIMEOUT:?}): {err:?}"
            );
            NODE_RESPONSE_DEFAULT_TIMEOUT
        }
        Err(_) => NODE_RESPONSE_DEFAULT_TIMEOUT,
    };
}

// Message handling over streams
impl MyNode {
    pub(crate) async fn send_node_msg_response(
        msg: NodeMsg,
        msg_id: MsgId,
        recipient: Peer,
        context: NodeContext,
        send_stream: SendStream,
    ) -> Result<Vec<Cmd>> {
        let stream_id = send_stream.id();
        trace!("Sending response msg {msg_id:?} over {stream_id}");
        let (kind, payload) = MyNode::serialize_node_msg(context.name, msg)?;

        match send_msg_on_stream(
            context.network_knowledge.section_key(),
            payload,
            kind,
            send_stream,
            recipient,
            msg_id,
        )
        .await
        {
            Ok(()) => Ok(vec![]),
            Err(err) => {
                error!(
                    "Could not send response msg {msg_id:?} \
                    to {recipient:?} over {stream_id}: {err:?}"
                );
                if let Error::Comms(_) = err {
                    Ok(vec![Cmd::HandleFailedSendToNode {
                        peer: recipient,
                        msg_id,
                    }])
                } else {
                    Ok(vec![])
                }
            }
        }
    }

    pub(crate) async fn send_client_response(
        msg: ClientDataResponse,
        correlation_id: MsgId,
        send_stream: SendStream,
        context: NodeContext,
        source_client: Peer,
    ) -> Result<()> {
        trace!("Sending client response msg for {correlation_id:?}");
        let (kind, payload) = MyNode::serialize_client_msg_response(context.name, msg)?;
        send_msg_on_stream(
            context.network_knowledge.section_key(),
            payload,
            kind,
            send_stream,
            source_client,
            correlation_id,
        )
        .await
    }

    pub(crate) async fn send_node_data_response(
        msg: NodeDataResponse,
        correlation_id: MsgId,
        send_stream: SendStream,
        context: NodeContext,
        requesting_peer: Peer,
    ) -> Result<()> {
        trace!("Sending node response msg for {correlation_id:?}");
        let (kind, payload) = MyNode::serialize_node_data_msg_response(context.name, msg)?;
        send_msg_on_stream(
            context.network_knowledge.section_key(),
            payload,
            kind,
            send_stream,
            requesting_peer,
            correlation_id,
        )
        .await
    }

    pub(crate) async fn send_msg_and_await_response(
        msg_id: MsgId,
        msg: NodeMsg,
        context: NodeContext,
        recipient: Peer,
        client_stream: SendStream,
        source_client: Peer,
    ) -> Result<Vec<Cmd>> {
        let (kind, payload) = MyNode::serialize_node_msg(context.name, msg)?;
        let bytes = form_usr_msg_bytes(
            context.network_knowledge.section_key(),
            payload,
            kind,
            recipient.name(),
            msg_id,
        )?;

        debug!("Sending out {msg_id:?} to node {recipient:?}");
        let comm = context.comm.clone();
        let response = match timeout(*NODE_RESPONSE_TIMEOUT, async {
            comm.send_out_bytes_to_peer_and_return_response(recipient, msg_id, bytes)
                .await
        })
        .await
        {
            Ok(resp) => {
                if resp.is_err() {
                    if let Err(_error) = context
                        .fault_cmds_sender
                        .send(FaultsCmd::TrackIssue(
                            recipient.name(),
                            IssueType::RequestOperation,
                        ))
                        .await
                    {
                        error!("Could not track node fault against {:?}", recipient.name());
                    }
                }

                resp
            }
            Err(_elapsed) => {
                error!(
                    "{msg_id:?}: No response from {recipient:?} after {:?} timeout. \
                    Marking node as faulty",
                    *NODE_RESPONSE_TIMEOUT
                );
                return Ok(vec![Cmd::TrackNodeIssue {
                    name: recipient.name(),
                    issue: IssueType::Communication,
                }]);
            }
        }?;

        debug!("Response in from peer for query {msg_id:?} {response:?}");
        let cmds = MyNode::send_query_response_to_client(
            msg_id,
            context,
            response.into_msg()?,
            client_stream,
            source_client,
        );
        Ok(cmds)
    }
}

// Serializes the msg, producing one [`WireMsg`] instance
// per recipient - the last step before passing it over to comms module.
pub(crate) fn into_msg_bytes(
    network_knowledge: &NetworkKnowledge,
    our_node_name: XorName,
    msg: NodeMsg,
    msg_id: MsgId,
    recipients: Peers,
) -> Result<Vec<(Peer, UsrMsgBytes)>> {
    let (kind, payload) = MyNode::serialize_node_msg(our_node_name, msg)?;
    let recipients = match recipients {
        Peers::Single(peer) => vec![peer],
        Peers::Multiple(peers) => peers.into_iter().collect(),
    };

    // we first generate the XorName
    let dst = Dst {
        name: xor_name::rand::random(),
        section_key: bls::SecretKey::random().public_key(),
    };

    let mut initial_wire_msg = WireMsg::new_msg(msg_id, payload, kind, dst);

    let _bytes = initial_wire_msg.serialize_and_cache_bytes()?;

    let mut msgs = vec![];
    for peer in recipients {
        match network_knowledge.generate_dst(&peer.name()) {
            Ok(dst) => {
                // TODO log error here isntead of throwing
                let all_the_bytes = initial_wire_msg.serialize_with_new_dst(&dst)?;
                msgs.push((peer, all_the_bytes));
            }
            Err(error) => {
                error!("Could not get route for {peer:?}: {error}");
            }
        }
    }

    Ok(msgs)
}

fn form_usr_msg_bytes(
    section_key: bls::PublicKey,
    payload: Bytes,
    kind: MsgKind,
    target_name: XorName,
    msg_id: MsgId,
) -> Result<UsrMsgBytes> {
    let dst = Dst {
        name: target_name,
        section_key,
    };
    let wire_msg = WireMsg::new_msg(msg_id, payload, kind, dst);
    wire_msg.serialize().map_err(|_| Error::InvalidMessage)
}

// Send a msg on a given stream
async fn send_msg_on_stream(
    section_key: bls::PublicKey,
    payload: Bytes,
    kind: MsgKind,
    mut send_stream: SendStream,
    target_peer: Peer,
    msg_id: MsgId,
) -> Result<()> {
    let bytes = form_usr_msg_bytes(section_key, payload, kind, target_peer.name(), msg_id)?;

    let stream_id = send_stream.id();
    trace!("Sending response {msg_id:?} to {target_peer:?} over {stream_id}");

    let stream_prio = 10;
    send_stream.set_priority(stream_prio);
    trace!("Prio set for {msg_id:?} to {target_peer:?}, over {stream_id}");

    if let Err(error) = send_stream.send_user_msg(bytes).await {
        error!(
            "Could not send response {msg_id:?} to peer {target_peer:?} \
            over response {stream_id}: {error:?}"
        );
        return Err(error.into());
    }

    trace!("Msg away for {msg_id:?} to {target_peer:?}, over {stream_id}");

    // unblock + move finish off thread as it's not strictly related to the sending of the msg.
    let stream_id_clone = stream_id.clone();
    let _handle = tokio::spawn(async move {
        // Attempt to gracefully terminate the stream.
        // If this errors it does _not_ mean our message has not been sent
        let result = send_stream.finish().await;
        trace!("Response {msg_id:?} sent to {target_peer:?} over {stream_id_clone}. Stream finished with result: {result:?}");
    });

    debug!("Sent the msg {msg_id:?} to {target_peer:?} over {stream_id}");

    Ok(())
}
