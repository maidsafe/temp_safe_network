// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::super::Core;
use crate::messaging::{
    system::{
        JoinAsRelocatedRequest, JoinAsRelocatedResponse, JoinRejectionReason, JoinRequest,
        JoinResponse, Peer, SystemMsg,
    },
    WireMsg,
};
use crate::routing::{
    error::{Error, Result},
    log_markers::LogMarker,
    peer::PeerUtils,
    relocation::RelocatePayloadUtils,
    routing_api::command::Command,
    FIRST_SECTION_MAX_AGE, FIRST_SECTION_MIN_AGE, RECOMMENDED_SECTION_SIZE,
};
use bls::PublicKey as BlsPublicKey;

// Message handling
impl Core {
    /// Check if we already have this peer in our section
    fn peer_is_already_a_member(&self, peer: Peer) -> bool {
        if self.section.members().is_joined(peer.name()) {
            debug!(
                "Ignoring JoinRequest from {} - already member of our section.",
                peer
            );
            return true;
        }

        false
    }

    pub(crate) async fn handle_join_request(
        &self,
        peer: Peer,
        join_request: JoinRequest,
    ) -> Result<Vec<Command>> {
        debug!("Received {:?} from {}", join_request, peer);

        let _permit = self
            .current_joins_semaphore
            .acquire()
            .await
            .map_err(|_| Error::PermitAcquisitionFailed)?;

        let section_key_matches = join_request.section_key == self.section.section_key().await;

        // Ignore `JoinRequest` if we are not elder unless the join request
        // is outdated in which case we reply with `BootstrapResponse::Join`
        // with the up-to-date info (see `handle_join_request`).
        if self.is_not_elder().await && section_key_matches {
            // Note: We don't bounce this message because the current bounce-resend
            // mechanism wouldn't preserve the original SocketAddr which is needed for
            // properly handling this message.
            // This is OK because in the worst case the join request just timeouts and the
            // joining node sends it again.
            return Ok(vec![]);
        }

        if self.peer_is_already_a_member(peer) {
            return Ok(vec![]);
        }

        if !*self.joins_allowed.read().await {
            debug!(
                "Rejecting JoinRequest from {} - joins currently not allowed.",
                peer,
            );
            let node_msg = SystemMsg::JoinResponse(Box::new(JoinResponse::Rejected(
                JoinRejectionReason::JoinsDisallowed,
            )));

            trace!("{}", LogMarker::SendJoinsDisallowed);

            trace!("Sending {:?} to {}", node_msg, peer);
            return Ok(vec![
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.section_key().await,
                )
                .await?,
            ]);
        }

        // Safe to use number of own section's members.
        // As it is for the stepped fixed age during genesis section only.
        // During the first section, node shall use ranged age to avoid too many nodes got
        // relocated at the same time. After the first section got split, nodes shall only
        // start with age of MIN_ADULT_AGE
        let section_members = self.section.active_members().await.len() as u8;
        let expected_age: u8 = if self.section.prefix().await.is_empty()
            && section_members < RECOMMENDED_SECTION_SIZE as u8 * 3
        {
            FIRST_SECTION_MAX_AGE - section_members * 2
        } else {
            FIRST_SECTION_MIN_AGE
        };

        if !section_key_matches || !self.section.prefix().await.matches(peer.name()) {
            if section_key_matches {
                debug!(
                    "JoinRequest from {} - name doesn't match our prefix {:?}.",
                    peer,
                    self.section.prefix().await
                );
            } else {
                debug!(
                    "JoinRequest from {} - doesn't have our latest section_key {:?}, presented {:?}.",
                    peer,
                    self.section.section_key().await,
                    join_request.section_key
                );
            }

            let retry_sap = self.matching_section(peer.name()).await?;

            let node_msg = SystemMsg::JoinResponse(Box::new(JoinResponse::Retry {
                section_auth: retry_sap,
                expected_age,
            }));

            trace!("Sending {:?} to {}", node_msg, peer);
            trace!("{}", LogMarker::SendJoinRetryNotCorrectKey);
            return Ok(vec![
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.section_key().await,
                )
                .await?,
            ]);
        }

        if peer.age() != expected_age {
            let node_msg = SystemMsg::JoinResponse(Box::new(JoinResponse::Retry {
                section_auth: self.section.authority_provider().await,
                expected_age,
            }));
            trace!("{}", LogMarker::SendJoinRetryFirstSectionAgeIssue);
            trace!(
                "New node joining first section with incorrect age {:?}. Sending {:?} to {}",
                peer.age(),
                node_msg,
                peer
            );

            return Ok(vec![
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.section_key().await,
                )
                .await?,
            ]);
        }

        // Require resource signed if joining as a new node.
        if let Some(response) = join_request.resource_proof_response {
            if !self
                .validate_resource_proof_response(peer.name(), response)
                .await
            {
                debug!(
                    "Ignoring JoinRequest from {} - invalid resource signed response",
                    peer
                );
                return Ok(vec![]);
            }
        } else {
            // Do reachability check only for the initial join request
            let cmd = if self.comm.is_reachable(peer.addr()).await.is_err() {
                let node_msg = SystemMsg::JoinResponse(Box::new(JoinResponse::Rejected(
                    JoinRejectionReason::NodeNotReachable(*peer.addr()),
                )));

                trace!("{}", LogMarker::SendJoinRejected);

                trace!("Sending {:?} to {}", node_msg, peer);
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.section_key().await,
                )
                .await?
            } else {
                // It's reachable, let's then send the proof challenge
                self.send_resource_proof_challenge(&peer).await?
            };

            return Ok(vec![cmd]);
        }

        Ok(vec![Command::ProposeOnline {
            peer,
            previous_name: None,
            dst_key: None,
        }])
    }

    pub(crate) async fn handle_join_as_relocated_request(
        &self,
        peer: Peer,
        join_request: JoinAsRelocatedRequest,
        known_keys: Vec<BlsPublicKey>,
    ) -> Result<Vec<Command>> {
        debug!("Received {:?} from {}", join_request, peer);
        let relocate_payload = if let Some(relocate_payload) = join_request.relocate_payload {
            relocate_payload
        } else {
            // Do reachability check
            let node_msg = if self.comm.is_reachable(peer.addr()).await.is_err() {
                SystemMsg::JoinAsRelocatedResponse(Box::new(
                    JoinAsRelocatedResponse::NodeNotReachable(*peer.addr()),
                ))
            } else {
                SystemMsg::JoinAsRelocatedResponse(Box::new(JoinAsRelocatedResponse::Retry(
                    self.section.authority_provider().await,
                )))
            };
            trace!("{}", LogMarker::SendJoinAsRelocatedResponse);

            trace!("Sending {:?} to {}", node_msg, peer);
            return Ok(vec![
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.section_key().await,
                )
                .await?,
            ]);
        };

        if !self.section.prefix().await.matches(peer.name())
            || join_request.section_key != self.section.section_key().await
        {
            debug!(
                "JoinAsRelocatedRequest from {} - name doesn't match our prefix {:?}.",
                peer,
                self.section.prefix().await
            );

            let node_msg = SystemMsg::JoinAsRelocatedResponse(Box::new(
                JoinAsRelocatedResponse::Retry(self.section.authority_provider().await),
            ));

            trace!("{} b", LogMarker::SendJoinAsRelocatedResponse);

            trace!("Sending {:?} to {}", node_msg, peer);
            return Ok(vec![
                self.send_direct_message(
                    (*peer.name(), *peer.addr()),
                    node_msg,
                    self.section.section_key().await,
                )
                .await?,
            ]);
        }

        if self.section.members().is_joined(peer.name()) {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - already member of our section.",
                peer
            );
            return Ok(vec![]);
        }

        if !relocate_payload.verify_identity(peer.name()) {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - invalid signature.",
                peer
            );
            return Ok(vec![]);
        }

        let details = relocate_payload.relocate_details()?;

        if !self.section.prefix().await.matches(&details.dst) {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - destination {} doesn't match \
                         our prefix {:?}.",
                peer,
                details.dst,
                self.section.prefix().await
            );
            return Ok(vec![]);
        }

        // Requires the node name matches the age.
        let age = details.age;
        if age != peer.age() {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - relocation age ({}) doesn't match peer's age ({}).",
                peer, age, peer.age(),
            );
            return Ok(vec![]);
        }

        // Check for signatures and trust of the relocate_payload msg
        let serialised_relocate_details =
            WireMsg::serialize_msg_payload(&SystemMsg::Relocate(details.clone()))?;

        let payload_section_signed = &relocate_payload.section_signed;
        let is_valid_sig = payload_section_signed.sig.public_key.verify(
            &payload_section_signed.sig.signature,
            serialised_relocate_details,
        );
        let is_key_unknown = !known_keys
            .iter()
            .any(|key| *key == payload_section_signed.sig.public_key);

        if !is_valid_sig || is_key_unknown {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - invalid signature or untrusted src.",
                peer
            );
            return Ok(vec![]);
        }

        let previous_name = Some(details.pub_id);
        let dst_key = Some(details.dst_key);

        if self
            .section
            .members()
            .is_relocated_to_our_section(&details.pub_id)
        {
            debug!(
                "Ignoring JoinAsRelocatedRequest from {} - original node {:?} already relocated to us.",
                peer, previous_name
            );
            return Ok(vec![]);
        }

        Ok(vec![Command::ProposeOnline {
            peer,
            previous_name,
            dst_key,
        }])
    }
}
