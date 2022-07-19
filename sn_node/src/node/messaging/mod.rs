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

use crate::node::{flow_ctrl::cmds::Cmd, Node, Result, DATA_QUERY_LIMIT};

use sn_interface::{
    messaging::{
        data::ServiceMsg,
        system::{NodeMsgAuthorityUtils, SystemMsg},
        DstLocation, MsgType, NodeMsgAuthority, WireMsg,
    },
    network_knowledge::NetworkKnowledge,
    types::Peer,
};

use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use itertools::Itertools;

// Message handling
impl Node {
    #[instrument(skip(self, original_bytes))]
    pub(crate) async fn validate_msg(
        &mut self,
        origin: Peer,
        wire_msg: WireMsg,
        original_bytes: Bytes,
    ) -> Result<Vec<Cmd>> {
        // Deserialize the payload of the incoming message
        let msg_id = wire_msg.msg_id();
        // payload needed for aggregation
        let wire_msg_payload = wire_msg.payload.clone();

        let msg_type = match wire_msg.into_msg() {
            Ok(msg_type) => msg_type,
            Err(error) => {
                error!(
                    "Failed to deserialize message payload ({:?}): {:?}",
                    msg_id, error
                );
                return Ok(vec![]);
            }
        };

        match msg_type {
            MsgType::System {
                msg_id,
                msg_authority,
                dst_location,
                msg,
            } => {
                // Verify that the section key in the msg authority is trusted
                let known_keys =
                    if let Some(known_keys) = self.verify_section_key(&msg_authority, &msg).await {
                        known_keys
                    } else {
                        warn!(
                            "Untrusted message ({:?}) dropped from {:?}: {:?} ",
                            msg_id, origin, msg
                        );
                        return Ok(vec![]);
                    };

                // Check for entropy before we proceed further
                if let Some(ae_cmd) = self
                    .apply_ae(
                        &origin,
                        &msg,
                        &wire_msg,
                        &dst_location,
                        &msg_authority,
                        original_bytes.clone(), // a cheap clone w/ Bytes
                    )
                    .await?
                {
                    // short circuit and send those AE responses
                    return Ok(vec![ae_cmd]);
                }

                #[cfg(feature = "traceroute")]
                let traceroute = wire_msg.show_trace();

                Ok(vec![Cmd::HandleValidSystemMsg {
                    origin,
                    msg_id,
                    msg,
                    msg_authority,
                    known_keys,
                    wire_msg_payload,
                    #[cfg(feature = "traceroute")]
                    traceroute,
                }])
            }
            MsgType::Service {
                msg_id,
                msg,
                dst_location,
                auth,
            } => {
                if let DstLocation::EndUser(_) = dst_location {
                    warn!(
                        "Service msg has been dropped as its destination location ({:?}) is invalid: {:?}",
                        dst_location, msg
                    );
                    return Ok(vec![]);
                }

                let dst_name = match msg.dst_address() {
                    Some(name) => name,
                    None => {
                        error!(
                            "Service msg {:?} from {} has been dropped since there is no dst address.",
                            msg_id, origin.addr(),
                        );
                        return Ok(vec![]);
                    }
                };

                let src_location = wire_msg.auth_kind().src();

                if self.is_not_elder() {
                    trace!("Redirecting from adult to section elders");

                    return Ok(vec![self.ae_redirect_to_our_elders(
                        origin,
                        &src_location,
                        &original_bytes,
                    )?]);
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
                        return Ok(vec![]);
                    }
                };

                if let Some(cmd) = self.check_for_entropy(
                    original_bytes,
                    &src_location,
                    &received_section_pk,
                    dst_name,
                    &origin,
                )? {
                    // short circuit and send those AE responses
                    return Ok(vec![cmd]);
                }

                Ok(vec![Cmd::HandleValidServiceMsg {
                    msg_id,
                    msg,
                    origin,
                    auth,
                    #[cfg(feature = "traceroute")]
                    traceroute: wire_msg.show_trace(),
                }])
            }
        }
    }

    /// Verifies that the section key in the msg authority is trusted
    /// based on our current knowledge of the network and sections chains.
    #[instrument(skip_all)]
    async fn verify_section_key(
        &mut self,
        msg_authority: &NodeMsgAuthority,
        msg: &SystemMsg,
    ) -> Option<Vec<BlsPublicKey>> {
        let mut known_keys: Vec<BlsPublicKey> = self
            .network_knowledge
            .section_chain()
            .keys()
            .copied()
            .collect();
        known_keys.extend(self.network_knowledge.prefix_map().section_keys());
        known_keys.push(*self.network_knowledge.genesis_key());

        if NetworkKnowledge::verify_node_msg_can_be_trusted(
            msg_authority.clone(),
            msg.clone(),
            &known_keys,
        ) {
            Some(known_keys)
        } else {
            None
        }
    }

    /// Check if the origin needs to be updated on network structure/members.
    /// Returns an ae cmd if we need to halt msg validation and update the origin instead.
    #[instrument(skip_all)]
    async fn apply_ae(
        &mut self,
        origin: &Peer,
        msg: &SystemMsg,
        wire_msg: &WireMsg,
        dst_location: &DstLocation,
        msg_authority: &NodeMsgAuthority,
        msg_bytes: Bytes,
    ) -> Result<Option<Cmd>> {
        // Adult nodes don't need to carry out entropy checking,
        // however the message shall always be handled.
        if !self.is_elder() {
            return Ok(None);
        }
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
                    wire_msg.msg_id()
                );
                Ok(None)
            }
            _ => match dst_location.section_pk() {
                None => Ok(None),
                Some(dst_section_pk) => {
                    if let Some(ae_cmd) = self.check_for_entropy(
                        msg_bytes,
                        &msg_authority.src_location(),
                        &dst_section_pk,
                        dst_location.name(),
                        origin,
                    )? {
                        // we want to log issues with an elder who is out of sync here...
                        let knowledge = self.network_knowledge.elders();
                        let mut known_elders = knowledge.iter().map(|peer| peer.name());

                        if known_elders.contains(&origin.name()) {
                            // we track a dysfunction against our elder here
                            self.log_knowledge_issue(origin.name())?;
                        }

                        return Ok(Some(ae_cmd));
                    }

                    trace!(
                        "Entropy check passed. Handling verified msg {:?}",
                        wire_msg.msg_id()
                    );

                    Ok(None)
                }
            },
        }
    }
}
