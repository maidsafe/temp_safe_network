// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{core::NodeContext, Cmd, Error, MyNode, Result};

use sn_interface::{
    messaging::{
        data::ClientDataResponse,
        system::{NodeDataResponse, NodeMsg},
        Dst, MsgId, MsgKind, WireMsg,
    },
    types::Peer,
};

use bytes::Bytes;
use qp2p::SendStream;
use std::{collections::BTreeSet, sync::Arc};
use tokio::sync::RwLock;
use xor_name::XorName;

// Message handling over streams
impl MyNode {
    pub(crate) async fn send_node_msg_response(
        msg: NodeMsg,
        msg_id: MsgId,
        recipient: Peer,
        send_stream: SendStream,
        context: NodeContext,
    ) -> Result<Option<Cmd>> {
        let stream_id = send_stream.id();
        trace!("Sending response msg {msg_id:?} over {stream_id}");
        let (kind, payload) = MyNode::serialize_node_msg(context.name, &msg)?;
        send_msg_on_stream(
            context.network_knowledge.section_key(),
            payload,
            kind,
            Arc::new(RwLock::new(send_stream)),
            recipient.name(),
            msg_id,
        )
        .await
    }

    pub(crate) async fn send_client_response(
        msg: ClientDataResponse,
        correlation_id: MsgId,
        client_name: XorName,
        send_stream: Arc<RwLock<SendStream>>,
        context: NodeContext,
    ) -> Result<Option<Cmd>> {
        trace!("Sending client response msg for {correlation_id:?}");
        let (kind, payload) = MyNode::serialize_client_msg_response(context.name, &msg)?;
        send_msg_on_stream(
            context.network_knowledge.section_key(),
            payload,
            kind,
            send_stream,
            client_name,
            correlation_id,
        )
        .await
    }

    pub(crate) async fn send_node_data_response(
        msg: NodeDataResponse,
        send_stream: SendStream,
        context: NodeContext,
        requesting_peer: Peer,
    ) -> Result<Option<Cmd>> {
        trace!("Sending data response to msg {:?}..", msg.correlation_id());
        let (kind, payload) = MyNode::serialize_node_data_response(context.name, &msg)?;
        send_msg_on_stream(
            context.network_knowledge.section_key(),
            payload,
            kind,
            Arc::new(RwLock::new(send_stream)),
            requesting_peer.name(),
            *msg.correlation_id(),
        )
        .await
    }

    /// Sends a msg, and listens for any response
    /// The response is returned to be handled via the dispatcher (though a response is not necessarily expected)
    pub(crate) fn send_and_enqueue_any_response(
        msg_id: MsgId,
        msg: NodeMsg,
        context: NodeContext,
        recipients: BTreeSet<Peer>,
    ) -> Result<()> {
        let targets_len = recipients.len();
        debug!("Sending out + awaiting response of {msg_id:?} to {targets_len} holder node/s {recipients:?}");

        let (kind, payload) = MyNode::serialize_node_msg(context.name, &msg)?;

        // We create a Dst with random dst name, but we'll update it accordingly for each target
        let mut dst = Dst {
            name: XorName::default(),
            section_key: context.network_knowledge.section_key(),
        };
        let mut wire_msg = WireMsg::new_msg(msg_id, payload, kind, dst);
        let _bytes = wire_msg.serialize_and_cache_bytes()?;

        for target in recipients {
            dst.name = target.name();
            let bytes_to_node = wire_msg.serialize_with_new_dst(&dst)?;
            let comm = context.comm.clone();
            info!("About to send {msg_id:?} to holder node: {target:?}");
            comm.send_and_return_response(target, msg_id, bytes_to_node);
        }

        Ok(())
    }
}

// Send a msg on a given stream
async fn send_msg_on_stream(
    section_key: bls::PublicKey,
    payload: Bytes,
    kind: MsgKind,
    send_stream: Arc<RwLock<SendStream>>,
    target_peer: XorName,
    correlation_id: MsgId,
) -> Result<Option<Cmd>> {
    let dst = Dst {
        name: target_peer,
        section_key,
    };
    let msg_id = MsgId::new();
    let wire_msg = WireMsg::new_msg(msg_id, payload, kind, dst);
    let bytes = wire_msg.serialize().map_err(|_| Error::InvalidMessage)?;

    let stream_id = {
        let mut stream = send_stream.write().await;
        let stream_id = stream.id();
        trace!("Sending response {msg_id:?} of msg {correlation_id:?}, to {target_peer:?} over {stream_id}");
        if let Err(error) = stream.send_user_msg(bytes).await {
            error!(
                "Could not send response {msg_id:?} of msg {correlation_id:?}, to {target_peer:?} \
                over response {stream_id}: {error:?}"
            );
            return Err(Error::Comms(sn_comms::Error::from(error)));
        }
        stream_id
    };

    trace!(
        "Sent: Response {msg_id:?} of msg {correlation_id:?} to {target_peer:?}, over {stream_id}."
    );

    // unblock + move finish off thread as it's not strictly related to the sending of the msg.
    let stream_id_clone = stream_id.clone();
    let _handle = tokio::spawn(async move {
        let mut stream = send_stream.write().await;
        // Attempt to gracefully terminate the stream.
        // If this errors it does _not_ mean our message has not been sent
        let result = stream.finish().await;
        trace!("Response {msg_id:?} of msg {correlation_id:?} sent to {target_peer:?} over {stream_id_clone}. Stream finished with result: {result:?}");
    });

    debug!("Sent the response {msg_id:?} of msg {correlation_id:?} to {target_peer:?} over {stream_id}");

    Ok(None)
}
