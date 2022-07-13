// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod agreement;
mod anti_entropy;
mod dkg;
mod handover;
mod join;
mod left;
mod membership;
mod proposal;
mod relocation;
mod resource_proof;
mod service_msgs;
mod system_msgs;
mod update_section;

use crate::comm::Comm;
use crate::node::{node_api::cmds::Cmd, Node, Result, DATA_QUERY_LIMIT};
use sn_interface::{
    messaging::{
        data::ServiceMsg,
        system::{NodeMsgAuthorityUtils, SystemMsg},
        MsgType, WireMsg,
    },
    network_knowledge::NetworkKnowledge,
    types::Peer,
};

use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use itertools::Itertools;

// Message handling
impl Node {
    #[instrument(skip(self, original_bytes, comm))]
    pub(crate) async fn handle_msg(
        &mut self,
        sender: Peer,
        wire_msg: WireMsg,
        original_bytes: Option<Bytes>,
        comm: &Comm,
    ) -> Result<Vec<Cmd>> {
        let mut cmds = vec![];

        // Deserialize the payload of the incoming message
        let msg_id = wire_msg.msg_id();
        // payload needed for aggregation
        let payload = wire_msg.payload.clone();

        let message_type = match wire_msg.into_msg() {
            Ok(message_type) => message_type,
            Err(error) => {
                error!(
                    "Failed to deserialize message payload ({:?}): {:?}",
                    msg_id, error
                );
                return Ok(cmds);
            }
        };

        match message_type {
            MsgType::System {
                msg_id,
                msg_authority,
                dst_location,
                msg,
            } => {
                // Let's now verify the section key in the msg authority is trusted
                // based on our current knowledge of the network and sections chains.
                let mut known_keys: Vec<BlsPublicKey> = self
                    .network_knowledge
                    .section_chain()
                    .await
                    .keys()
                    .copied()
                    .collect();
                known_keys.extend(self.network_knowledge.prefix_map().section_keys());
                known_keys.push(*self.network_knowledge.genesis_key());

                if !NetworkKnowledge::verify_node_msg_can_be_trusted(
                    msg_authority.clone(),
                    msg.clone(),
                    &known_keys,
                ) {
                    warn!(
                        "Untrusted message ({:?}) dropped from {:?}: {:?} ",
                        msg_id, sender, msg
                    );
                    return Ok(cmds);
                }

                // Let's check for entropy before we proceed further
                // Adult nodes don't need to carry out entropy checking,
                // however the message shall always be handled.
                if self.is_elder() {
                    // For the case of receiving a join request not matching our prefix,
                    // we just let the join request handler to deal with it later on.
                    // We also skip AE check on Anti-Entropy messages
                    //
                    // TODO: consider changing the join and "join as relocated" flows to
                    // make use of AntiEntropy retry/redirect responses.
                    match msg {
                        SystemMsg::AntiEntropyRetry { .. }
                        | SystemMsg::AntiEntropyUpdate { .. }
                        | SystemMsg::AntiEntropyRedirect { .. }
                        | SystemMsg::JoinRequest(_)
                        | SystemMsg::JoinAsRelocatedRequest(_) => {
                            trace!(
                                "Entropy check skipped for {:?}, handling message directly",
                                msg_id
                            );
                        }
                        _ => match dst_location.section_pk() {
                            None => {}
                            Some(dst_section_pk) => {
                                let msg_bytes = original_bytes.unwrap_or(wire_msg.serialize()?);

                                if let Some(ae_cmd) = self
                                    .check_for_entropy(
                                        // a cheap clone w/ Bytes
                                        msg_bytes,
                                        &msg_authority.src_location(),
                                        &dst_section_pk,
                                        dst_location.name(),
                                        &sender,
                                    )
                                    .await?
                                {
                                    // we want to log issues with an elder who is out of sync here...
                                    let knowledge = self.network_knowledge.elders();
                                    let mut known_elders = knowledge.iter().map(|peer| peer.name());

                                    if known_elders.contains(&sender.name()) {
                                        // we track a dysfunction against our elder here
                                        self.log_knowledge_issue(sender.name()).await?;
                                    }

                                    // short circuit and send those AE responses
                                    cmds.push(ae_cmd);
                                    return Ok(cmds);
                                }

                                trace!("Entropy check passed. Handling verified msg {:?}", msg_id);
                            }
                        },
                    }
                }

                let handling_msg_cmds = self
                    .handle_system_msg(
                        sender,
                        msg_id,
                        msg_authority,
                        msg,
                        payload,
                        known_keys,
                        comm,
                    )
                    .await?;

                cmds.extend(handling_msg_cmds);

                Ok(cmds)
            }
            MsgType::Service {
                msg_id,
                msg,
                dst_location,
                auth,
            } => {
                let dst_name = match msg.dst_address() {
                    Some(name) => name,
                    None => {
                        error!(
                            "Service msg has been dropped since {:?} is not a valid msg to send from a client {}.",
                            msg, sender.addr()
                        );
                        return Ok(vec![]);
                    }
                };

                let src_location = wire_msg.auth_kind().src();

                if self.is_not_elder() {
                    trace!("Redirecting from adult to section elders");
                    cmds.push(
                        self.ae_redirect_to_our_elders(sender, &src_location, &wire_msg)
                            .await?,
                    );
                    return Ok(cmds);
                }

                // First we check if it's query and we have too many on the go at the moment...
                if let ServiceMsg::Query(_) = msg {
                    // we have a query, check if we have too many on the go....
                    let pending_query_length = self.pending_data_queries.len().await;

                    if pending_query_length > DATA_QUERY_LIMIT {
                        // TODO: check if query is pending for this already.. add to that if that makes sense.
                        warn!("Pending queries length exceeded, dropping query {msg:?}");
                        return Ok(vec![]);
                    }
                }

                // Then we perform AE checks
                let received_section_pk = match dst_location.section_pk() {
                    Some(section_pk) => section_pk,
                    None => {
                        warn!("Dropping service message as there is no valid dst section_pk.");
                        return Ok(cmds);
                    }
                };

                let msg_bytes = original_bytes.unwrap_or(wire_msg.serialize()?);
                if let Some(cmd) = self
                    .check_for_entropy(
                        // a cheap clone w/ Bytes
                        msg_bytes,
                        &src_location,
                        &received_section_pk,
                        dst_name,
                        &sender,
                    )
                    .await?
                {
                    // short circuit and send those AE responses
                    cmds.push(cmd);
                    return Ok(cmds);
                }

                cmds.extend(
                    self.handle_service_msg(msg_id, msg, dst_location, auth, sender)
                        .await?,
                );

                Ok(cmds)
            }
        }
    }
}
