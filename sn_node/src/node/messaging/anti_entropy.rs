// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    flow_ctrl::cmds::Cmd, messages::WireMsgUtils, Error, Event, MembershipEvent, Node, Result,
    StateSnapshot,
};
use backoff::{backoff::Backoff, ExponentialBackoff};
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use itertools::Itertools;
use secured_linked_list::SecuredLinkedList;
use sn_interface::{
    messaging::{
        system::{KeyedSig, NodeCmd, SectionAuth, SectionPeers, SystemMsg},
        MsgId, MsgType, SrcLocation, WireMsg,
    },
    network_knowledge::SectionAuthorityProvider,
    types::{log_markers::LogMarker, Peer, PublicKey},
};
use std::time::Duration;
use xor_name::{Prefix, XorName};

impl Node {
    /// Send `AntiEntropyUpdate` message to all nodes in our own section.
    pub(crate) fn send_ae_update_to_our_section(&self) -> Vec<Cmd> {
        let our_name = self.info().name();
        let nodes: Vec<_> = self
            .network_knowledge
            .section_members()
            .into_iter()
            .filter(|info| info.name() != our_name)
            .map(|info| *info.peer())
            .collect();

        if nodes.is_empty() {
            warn!("No peers of our section found in our network knowledge to send AE-Update");
            return vec![];
        }

        // The previous PK which is likely what adults know
        let previous_pk = *self.section_chain().prev_key();

        let our_prefix = self.network_knowledge.prefix();

        self.send_ae_update_to_nodes(nodes, &our_prefix, previous_pk)
    }

    /// Send `AntiEntropyUpdate` message to the specified nodes.
    pub(crate) fn send_ae_update_to_nodes(
        &self,
        recipients: Vec<Peer>,
        prefix: &Prefix,
        section_pk: BlsPublicKey,
    ) -> Vec<Cmd> {
        let node_msg = match self.generate_ae_update_msg(section_pk) {
            Ok(node_msg) => node_msg,
            Err(err) => {
                warn!("Failed to generate AE-Update msg to send: {:?}", err);
                return vec![];
            }
        };

        let our_section_key = self.network_knowledge.section_key();
        match self.send_direct_msg_to_nodes(
            recipients.clone(),
            node_msg,
            prefix.name(),
            our_section_key,
        ) {
            Ok(cmd) => vec![cmd],
            Err(err) => {
                error!(
                    "Failed to send AE update to ({:?}) {:?}: {:?}",
                    prefix, recipients, err
                );
                vec![]
            }
        }
    }

    /// Send `MetadataExchange` packet to the specified nodes
    pub(crate) fn send_metadata_updates(
        &self,
        recipients: Vec<Peer>,
        prefix: &Prefix,
        section_pk: BlsPublicKey,
    ) -> Result<Vec<Cmd>> {
        let metadata = self.get_metadata_of(prefix);
        let data_update_msg = SystemMsg::NodeCmd(NodeCmd::ReceiveMetadata { metadata });

        match self.send_direct_msg_to_nodes(
            recipients.clone(),
            data_update_msg,
            prefix.name(),
            section_pk,
        ) {
            Ok(cmd) => Ok(vec![cmd]),
            Err(err) => {
                error!(
                    "Failed to send data updates to: {:?} with {:?}",
                    recipients, err
                );
                Ok(vec![])
            }
        }
    }

    #[instrument(skip_all)]
    /// Send AntiEntropyUpdate message to the nodes in our sibling section.
    pub(crate) fn send_updates_to_sibling_section(
        &self,
        our_prev_state: &StateSnapshot,
    ) -> Result<Vec<Cmd>> {
        debug!("{}", LogMarker::AeSendUpdateToSiblings);
        let sibling_prefix = self.network_knowledge.prefix().sibling();
        if let Some(sibling_sap) = self
            .network_knowledge
            .prefix_map()
            .get_signed(&sibling_prefix)
        {
            let promoted_sibling_elders: Vec<_> = sibling_sap
                .elders()
                .filter(|peer| !our_prev_state.elders.contains(&peer.name()))
                .cloned()
                .collect();

            if promoted_sibling_elders.is_empty() {
                debug!("No promoted siblings found in our network knowledge to send AE-Update");
                return Ok(vec![]);
            }

            // Using previous_key as dst_section_key as newly promoted
            // sibling Elders shall still in the state of pre-split.
            let previous_section_key = our_prev_state.section_key;
            let sibling_prefix = sibling_sap.prefix();

            let mut cmds = self.send_metadata_updates(
                promoted_sibling_elders.clone(),
                &sibling_prefix,
                previous_section_key,
            )?;

            // Also send AE update to sibling section's new Elders
            cmds.extend(self.send_ae_update_to_nodes(
                promoted_sibling_elders,
                &sibling_prefix,
                previous_section_key,
            ));

            Ok(cmds)
        } else {
            error!("Failed to get sibling SAP during split.");
            Ok(vec![])
        }
    }

    // Private helper to generate AntiEntropyUpdate message to update
    // a peer abot our SAP, with proof_chain and members list.
    fn generate_ae_update_msg(&self, dst_section_key: BlsPublicKey) -> Result<SystemMsg> {
        let signed_sap = self.network_knowledge.section_signed_authority_provider();

        let proof_chain = if let Ok(chain) = self
            .network_knowledge
            .get_proof_chain_to_current(&dst_section_key)
        {
            chain
        } else {
            // error getting chain from key, so let's send the whole chain from genesis
            self.network_knowledge.section_chain()
        };

        let members = self
            .network_knowledge
            .section_signed_members()
            .iter()
            .map(|state| state.clone().into_authed_msg())
            .collect();

        Ok(SystemMsg::AntiEntropyUpdate {
            section_auth: signed_sap.value.to_msg(),
            section_signed: signed_sap.sig,
            proof_chain,
            members,
        })
    }

    #[instrument(skip_all)]
    pub(crate) async fn handle_anti_entropy_update_msg(
        &mut self,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        proof_chain: SecuredLinkedList,
        members: SectionPeers,
    ) -> Result<Vec<Cmd>> {
        let snapshot = self.state_snapshot();

        let our_name = self.info().name();
        let signed_sap = SectionAuth {
            value: section_auth.clone(),
            sig: section_signed.clone(),
        };

        let updated = self.network_knowledge.update_knowledge_if_valid(
            signed_sap.clone(),
            &proof_chain,
            Some(members),
            &our_name,
            &self.section_keys_provider,
        )?;

        // always run this, only changes will trigger events
        let mut cmds = self.update_on_elder_change(snapshot).await?;

        // Only trigger reorganize data when there is a membership change happens.
        if updated {
            cmds.extend(self.try_reorganize_data()?);
        }

        Ok(cmds)
    }

    pub(crate) async fn handle_anti_entropy_retry_msg(
        &mut self,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        proof_chain: SecuredLinkedList,
        bounced_msg: Bytes,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        let dst_section_key = section_auth.section_key();
        let snapshot = self.state_snapshot();

        let to_resend = self
            .update_network_knowledge(
                section_auth,
                section_signed,
                proof_chain,
                bounced_msg,
                sender,
            )
            .await?;

        match to_resend {
            None => Ok(vec![]),
            Some((msg_to_resend, _)) => {
                // TODO: we may need to check if 'bounced_msg' dst section pk is different
                // from the received new SAP key, to prevent from endlessly resending a msg
                // if a sybil/corrupt peer keeps sending us the same AE msg.
                trace!(
                    "{} resending {:?}",
                    LogMarker::AeResendAfterRetry,
                    msg_to_resend
                );

                self.create_or_wait_for_backoff(&sender).await;

                let mut result = Vec::new();
                if let Ok(cmds) = self.update_on_elder_change(snapshot).await {
                    result.extend(cmds);
                }

                result.push(self.send_direct_msg(sender, msg_to_resend, dst_section_key)?);

                Ok(result)
            }
        }
    }

    pub(crate) async fn handle_anti_entropy_redirect_msg(
        &mut self,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        section_chain: SecuredLinkedList,
        bounced_msg: Bytes,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        let dst_section_key = section_auth.section_key();

        // We choose the Elder closest to the dst section key,
        // just to pick one of them in an arbitrary but deterministic fashion.
        let target_name = XorName::from(PublicKey::Bls(dst_section_key));
        let chosen_dst_elder = section_auth
            .elders()
            .sorted_by(|lhs, rhs| target_name.cmp_distance(&lhs.name(), &rhs.name()))
            .peekable()
            .peek()
            .copied()
            .copied();

        let to_resend = self
            .update_network_knowledge(
                section_auth,
                section_signed,
                section_chain,
                bounced_msg,
                sender,
            )
            .await?;

        match to_resend {
            None => Ok(vec![]),
            Some((msg_to_redirect, msg_id)) => match chosen_dst_elder {
                None => {
                    error!(
                            "Failed to find closest Elder to resend msg ({:?}) upon AE-Redirect response.",
                            msg_id
                        );
                    Ok(vec![])
                }
                Some(elder) if elder.addr() == sender.addr() => {
                    error!(
                            "Failed to find an alternative Elder to resend msg ({:?}) upon AE-Redirect response.",
                            msg_id
                        );
                    Ok(vec![])
                }
                Some(elder) => {
                    trace!("{}", LogMarker::AeResendAfterAeRedirect);

                    self.create_or_wait_for_backoff(&elder).await;

                    let cmd = self.send_direct_msg(elder, msg_to_redirect, dst_section_key)?;

                    Ok(vec![cmd])
                }
            },
        }
    }

    // Try to update network knowledge and return the 'SystemMsg' that needs to be resent.
    async fn update_network_knowledge(
        &mut self,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        proof_chain: SecuredLinkedList,
        bounced_msg: Bytes,
        sender: Peer,
    ) -> Result<Option<(SystemMsg, MsgId)>> {
        let (bounced_msg, msg_id, dst_location) = match WireMsg::deserialize(bounced_msg)? {
            MsgType::System {
                msg,
                msg_id,
                dst_location,
                ..
            } => (msg, msg_id, dst_location),
            _ => {
                warn!("Non System MsgType received in AE response. We do not handle any other type in AE msgs yet.");
                return Ok(None);
            }
        };

        info!(
            "Anti-Entropy: message received from peer: {}",
            sender.addr()
        );

        let prefix = section_auth.prefix();
        let dst_section_key = section_auth.section_key();
        let signed_sap = SectionAuth {
            value: section_auth.clone(),
            sig: section_signed.clone(),
        };
        let our_name = self.info().name();
        let our_section_prefix = self.network_knowledge.prefix();
        let equal_prefix = section_auth.prefix() == our_section_prefix;
        let is_extension_prefix = section_auth.prefix().is_extension_of(&our_section_prefix);
        let our_peer_info = self.info().peer();

        // Update our network knowledge
        let there_was_an_update = self.network_knowledge.update_knowledge_if_valid(
            signed_sap.clone(),
            &proof_chain,
            None,
            &our_name,
            &self.section_keys_provider,
        )?;

        if there_was_an_update {
            self.write_prefix_map().await;
            info!(
                "PrefixMap written to disk with update for prefix {:?}",
                prefix
            );

            // check for churn join miss
            let is_in_current_section = section_auth
                .members()
                .any(|node_state| node_state.peer() == &our_peer_info);
            let prefix_matches_our_name = prefix.matches(&our_name);
            let was_in_ancestor_section = equal_prefix || is_extension_prefix;

            if was_in_ancestor_section && prefix_matches_our_name && !is_in_current_section {
                error!("Detected churn join miss while processing msg ({:?}), was in section {:?}, updated to {:?}, wasn't in members anymore even if name matches: {:?}", msg_id, our_section_prefix, prefix, our_name);
                self.send_event(Event::Membership(MembershipEvent::ChurnJoinMissError))
                    .await;
                return Err(Error::ChurnJoinMiss);
            }
        }

        // If the new SAP's section key is the same as the section key set when the
        // bounced message was originally sent, we just drop it.
        if dst_location.section_pk() == Some(dst_section_key) {
            error!("Dropping bounced msg ({:?}) received in AE-Retry from {} as suggested new dst section key is the same as previously sent: {:?}", msg_id, sender,dst_section_key);
            Ok(None)
        } else {
            Ok(Some((bounced_msg, msg_id)))
        }
    }

    /// Checks AE-BackoffCache for backoff, or creates a new instance
    /// waits for any required backoff duration
    async fn create_or_wait_for_backoff(&mut self, peer: &Peer) {
        let our_backoff = self
            .ae_backoff_cache
            .find(|(node, _)| node == peer)
            .map(|(_, backoff)| backoff);

        if let Some(backoff) = our_backoff {
            let next_backoff = backoff.next_backoff();
            let sleep_time = if let Some(mut next_wait) = next_backoff {
                // The default setup start with around 400ms
                // then increases to around 50s after 25 calls.
                next_wait /= 100;
                if next_wait > Duration::from_secs(1) {
                    backoff.reset();
                }
                Some(next_wait)
            } else {
                // TODO: we've done all backoffs and are _still_ getting messages?
                // we should probably penalise the node here.
                None
            };

            if let Some(sleep_time) = sleep_time {
                tokio::time::sleep(sleep_time).await;
            }
        } else {
            let _res = self
                .ae_backoff_cache
                .insert((*peer, ExponentialBackoff::default()));
        }
    }

    // If entropy is found, determine the msg to send in order to
    // bring the sender's knowledge about us up to date.
    pub(crate) fn check_for_entropy(
        &self,
        original_bytes: Bytes,
        src_location: &SrcLocation,
        dst_section_key: &BlsPublicKey,
        dst_name: XorName,
        sender: &Peer,
    ) -> Result<Option<Cmd>> {
        // Check if the message has reached the correct section,
        // if not, we'll need to respond with AE
        let our_prefix = self.network_knowledge.prefix();

        // Let's try to find a section closer to the destination, if it's not for us.
        if !self.network_knowledge.prefix().matches(&dst_name) {
            debug!(
                "AE: prefix not matching. We are: {:?}, they sent to: {:?}",
                our_prefix, dst_name
            );
            match self
                .network_knowledge
                .get_closest_or_opposite_signed_sap(&dst_name)
            {
                Some((signed_sap, section_chain)) => {
                    info!("Found a better matching prefix {:?}", signed_sap.prefix());
                    let bounced_msg = original_bytes;
                    // Redirect to the closest section
                    let ae_msg = SystemMsg::AntiEntropyRedirect {
                        section_auth: signed_sap.value.to_msg(),
                        section_signed: signed_sap.sig,
                        section_chain,
                        bounced_msg,
                    };
                    let wire_msg = WireMsg::single_src(
                        &self.info(),
                        src_location.to_dst(),
                        ae_msg,
                        self.network_knowledge.section_key(),
                    )?;
                    trace!("{}", LogMarker::AeSendRedirect);

                    return Ok(Some(Cmd::SendMsg {
                        recipients: vec![*sender],
                        wire_msg,
                    }));
                }
                None => {
                    warn!("Our PrefixMap is empty");
                    // TODO: instead of just dropping the message, don't we actually need
                    // to get up to date info from other Elders in our section as it may be
                    // a section key we are not aware of yet?
                    // ...and once we acquired new key/s we attempt AE check again?
                    warn!(
                        "Anti-Entropy: cannot reply with redirect msg for dst_name {:?} and key {:?} to a closest section.",
                        dst_name, dst_section_key
                    );

                    return Err(Error::NoMatchingSection);
                }
            }
        }

        let section_key = self.network_knowledge.section_key();
        trace!(
            "Performing AE checks, provided pk was: {:?} ours is: {:?}",
            dst_section_key,
            section_key
        );

        if dst_section_key == &section_key {
            // Destination section key matches our current section key
            return Ok(None);
        }

        let ae_msg = match self
            .network_knowledge
            .get_proof_chain_to_current(dst_section_key)
        {
            Ok(proof_chain) => {
                info!("Anti-Entropy: sender's ({}) knowledge of our SAP is outdated, bounce msg for AE-Retry with up to date SAP info.", sender);

                let signed_sap = self.network_knowledge.section_signed_authority_provider();

                trace!(
                    "Sending AE-Retry with: proofchain last key: {:?} and  section key: {:?}",
                    proof_chain.last_key(),
                    &signed_sap.value.section_key()
                );
                trace!("{}", LogMarker::AeSendRetryAsOutdated);

                SystemMsg::AntiEntropyRetry {
                    section_auth: signed_sap.value.to_msg(),
                    section_signed: signed_sap.sig,
                    proof_chain,
                    bounced_msg: original_bytes,
                }
            }
            Err(_) => {
                trace!(
                    "Anti-Entropy: cannot find dst_section_key {:?} sent by {} in our chain",
                    dst_section_key,
                    sender
                );

                let proof_chain = self.network_knowledge.section_chain();

                let signed_sap = self.network_knowledge.section_signed_authority_provider();

                trace!("{}", LogMarker::AeSendRetryDstPkFail);

                SystemMsg::AntiEntropyRetry {
                    section_auth: signed_sap.value.to_msg(),
                    section_signed: signed_sap.sig,
                    proof_chain,
                    bounced_msg: original_bytes,
                }
            }
        };

        let wire_msg = WireMsg::single_src(
            &self.info(),
            src_location.to_dst(),
            ae_msg,
            self.network_knowledge.section_key(),
        )?;

        Ok(Some(Cmd::SendMsg {
            recipients: vec![*sender],
            wire_msg,
        }))
    }

    // Generate an AE redirect cmd for the given message
    pub(crate) fn ae_redirect_to_our_elders(
        &self,
        sender: Peer,
        src_location: &SrcLocation,
        original_wire_msg: &Bytes,
    ) -> Result<Cmd> {
        let signed_sap = self.network_knowledge.section_signed_authority_provider();

        let ae_msg = SystemMsg::AntiEntropyRedirect {
            section_auth: signed_sap.value.to_msg(),
            section_signed: signed_sap.sig,
            section_chain: self.network_knowledge.section_chain(),
            bounced_msg: original_wire_msg.clone(),
        };

        let wire_msg = WireMsg::single_src(
            &self.info(),
            src_location.to_dst(),
            ae_msg,
            self.network_knowledge.section_key(),
        )?;

        trace!("{} in ae_redirect", LogMarker::AeSendRedirect);

        Ok(Cmd::SendMsg {
            recipients: vec![sender],
            wire_msg,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::node::{
        cfg::create_test_max_capacity_and_root_storage,
        flow_ctrl::{event_channel, tests::create_comm},
        MIN_ADULT_AGE,
    };
    use crate::UsedSpace;

    use sn_interface::{
        elder_count,
        messaging::{
            AuthKind, AuthorityProof, DstLocation, MsgId, MsgType, NodeAuth, NodeMsgAuthority,
            SectionAuth as SectionAuthMsg,
        },
        network_knowledge::{
            test_utils::{gen_addr, random_sap, section_signed},
            NetworkKnowledge, NodeInfo, SectionKeyShare, SectionKeysProvider,
        },
        types::keys::ed25519,
    };

    use assert_matches::assert_matches;
    use bls::SecretKey;
    use eyre::{Context, Result};
    use secured_linked_list::SecuredLinkedList;
    use std::collections::BTreeSet;
    use xor_name::Prefix;

    #[tokio::test]
    async fn ae_everything_up_to_date() -> Result<()> {
        // Construct a local task set that can run `!Send` futures.
        let local = tokio::task::LocalSet::new();

        // Run the local task set.
        local
            .run_until(async move {
                let env = Env::new().await?;
                let our_prefix = env.node.network_knowledge().prefix();
                let (msg, src_location) =
                    env.create_msg(&our_prefix, env.node.network_knowledge().section_key())?;
                let sender = env.node.info().peer();
                let dst_name = our_prefix.substituted_in(xor_name::rand::random());
                let dst_section_key = env.node.network_knowledge().section_key();

                let cmd = env.node.check_for_entropy(
                    msg.serialize()?,
                    &src_location,
                    &dst_section_key,
                    dst_name,
                    &sender,
                )?;

                assert!(cmd.is_none());
                Result::<()>::Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn ae_update_msg_to_be_trusted() -> Result<()> {
        // Construct a local task set that can run `!Send` futures.
        let local = tokio::task::LocalSet::new();

        // Run the local task set.
        local
            .run_until(async move {
                let env = Env::new().await?;

                let known_keys = vec![*env.node.network_knowledge().genesis_key()];

                // This proof_chain already contains other_pk
                let proof_chain = env.proof_chain.clone();
                let (msg, msg_authority) = env.create_update_msg(proof_chain)?;

                // AeUpdate message shall get pass through.
                assert!(NetworkKnowledge::verify_node_msg_can_be_trusted(
                    msg_authority.clone(),
                    msg,
                    &known_keys
                ));

                // AeUpdate message contains corrupted proof_chain shall get rejected.
                let other_env = Env::new().await?;
                let (corrupted_msg, _msg_authority) =
                    env.create_update_msg(other_env.proof_chain)?;
                assert!(!NetworkKnowledge::verify_node_msg_can_be_trusted(
                    msg_authority.clone(),
                    corrupted_msg,
                    &known_keys
                ));

                // Other messages shall get rejected.
                let other_msg = SystemMsg::StartConnectivityTest(xor_name::rand::random());
                assert!(!NetworkKnowledge::verify_node_msg_can_be_trusted(
                    msg_authority,
                    other_msg,
                    &known_keys
                ));
                Result::<()>::Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn ae_redirect_to_other_section() -> Result<()> {
        // Construct a local task set that can run `!Send` futures.
        let local = tokio::task::LocalSet::new();

        // Run the local task set.
        local.run_until(async move {
            let mut env = Env::new().await?;

            let other_sk = bls::SecretKey::random();
            let other_pk = other_sk.public_key();

            let (msg, src_location) = env.create_msg(&env.other_sap.prefix(), other_pk)?;
            let sender = env.node.info().peer();

            // since it's not aware of the other prefix, it will redirect to self
            let dst_section_key = other_pk;
            let dst_name = env.other_sap.prefix().name();
            let cmd = env
                .node
                .check_for_entropy(
                    msg.serialize()?,
                    &src_location,
                    &dst_section_key,
                    dst_name,
                    &sender,
                );

            let msg_type = assert_matches!(cmd, Ok(Some(Cmd::SendMsg { wire_msg, .. })) => {
                wire_msg
                    .into_msg()
                    .context("failed to deserialised anti-entropy message")?
            });

            assert_matches!(msg_type, MsgType::System{ msg, .. } => {
                assert_matches!(msg, SystemMsg::AntiEntropyRedirect { section_auth, .. } => {
                    assert_eq!(section_auth, env.node.network_knowledge().authority_provider().to_msg());
                });
            });

            // now let's insert the other SAP to make it aware of the other prefix
            assert!(
                env.node
                    .network_knowledge
                    .update_knowledge_if_valid(
                        env.other_sap.clone(),
                        &env.proof_chain,
                        None,
                        &env.node.info().name(),
                        &env.node.section_keys_provider
                    )?
            );

            // and it now shall give us an AE redirect msg
            // with the SAP we inserted for other prefix
            let cmd = env
                .node
                .check_for_entropy(
                    msg.serialize()?,
                    &src_location,
                    &dst_section_key,
                    dst_name,
                    &sender,
                );

            let msg_type = assert_matches!(cmd, Ok(Some(Cmd::SendMsg { wire_msg, .. })) => {
                wire_msg
                    .into_msg()
                    .context("failed to deserialised anti-entropy message")?
            });

            assert_matches!(msg_type, MsgType::System{ msg, .. } => {
                assert_matches!(msg, SystemMsg::AntiEntropyRedirect { section_auth, .. } => {
                    assert_eq!(section_auth, env.other_sap.value.to_msg());
                });
            });
            Result::<()>::Ok(())
        }).await
    }

    #[tokio::test]
    async fn ae_outdated_dst_key_of_our_section() -> Result<()> {
        // Construct a local task set that can run `!Send` futures.
        let local = tokio::task::LocalSet::new();

        // Run the local task set.
        local.run_until(async move {


            let env = Env::new().await?;
            let our_prefix = env.node.network_knowledge().prefix();

            let (msg, src_location) = env.create_msg(
                &our_prefix,
                env.node.network_knowledge().section_key(),
            )?;
            let sender = env.node.info().peer();
            let dst_name = our_prefix.substituted_in(xor_name::rand::random());
            let dst_section_key = env.node.network_knowledge().genesis_key();

            let cmd = env
                .node
                .check_for_entropy(
                    msg.serialize()?,
                    &src_location,
                    dst_section_key,
                    dst_name,
                    &sender,
                )?;

            let msg_type = assert_matches!(cmd, Some(Cmd::SendMsg { wire_msg, .. }) => {
                wire_msg
                    .into_msg()
                    .context("failed to deserialised anti-entropy message")?
            });

            assert_matches!(msg_type, MsgType::System{ msg, .. } => {
                assert_matches!(msg, SystemMsg::AntiEntropyRetry { ref section_auth, ref proof_chain, .. } => {
                    assert_eq!(section_auth, &env.node.network_knowledge().authority_provider().to_msg());
                    assert_eq!(proof_chain, &env.node.section_chain());
                });
            });
            Result::<()>::Ok(())
        }).await
    }

    #[tokio::test]
    async fn ae_wrong_dst_key_of_our_section_returns_retry() -> Result<()> {
        // Construct a local task set that can run `!Send` futures.
        let local = tokio::task::LocalSet::new();

        // Run the local task set.
        local.run_until(async move {

            let env = Env::new().await?;
            let our_prefix = env.node.network_knowledge().prefix();

            let (msg, src_location) = env.create_msg(
                &our_prefix,
                env.node.network_knowledge().section_key(),
            )?;
            let sender = env.node.info().peer();
            let dst_name = our_prefix.substituted_in(xor_name::rand::random());

            let bogus_env = Env::new().await?;
            let dst_section_key = bogus_env.node.network_knowledge().genesis_key();

            let cmd = env
                .node
                .check_for_entropy(
                    msg.serialize()?,
                    &src_location,
                    dst_section_key,
                    dst_name,
                    &sender,
                )?;

            let msg_type = assert_matches!(cmd, Some(Cmd::SendMsg { wire_msg, .. }) => {
                wire_msg
                    .into_msg()
                    .context("failed to deserialised anti-entropy message")?
            });

            assert_matches!(msg_type, MsgType::System{ msg, .. } => {
                assert_matches!(msg, SystemMsg::AntiEntropyRetry { ref section_auth, ref proof_chain, .. } => {
                    assert_eq!(*section_auth, env.node.network_knowledge().authority_provider().to_msg());
                    assert_eq!(*proof_chain, env.node.section_chain());
                });
            });
            Result::<()>::Ok(())
        }).await
    }

    struct Env {
        node: Node,
        other_sap: SectionAuth<SectionAuthorityProvider>,
        proof_chain: SecuredLinkedList,
    }

    impl Env {
        async fn new() -> Result<Self> {
            let prefix0 = Prefix::default().pushed(false);
            let prefix1 = Prefix::default().pushed(true);

            // generate a SAP for prefix0
            let (section_auth, mut nodes, secret_key_set) = random_sap(prefix0, elder_count());
            let info = nodes.remove(0);
            let sap_sk = secret_key_set.secret_key();
            let signed_sap = section_signed(sap_sk, section_auth)?;

            let (chain, genesis_sk_set) = create_chain(sap_sk, signed_sap.section_key())
                .context("failed to create section chain")?;
            let genesis_pk = genesis_sk_set.public_keys().public_key();
            assert_eq!(genesis_pk, *chain.root_key());

            let (max_capacity, root_storage_dir) = create_test_max_capacity_and_root_storage()?;
            let (mut node, _) = Node::first_node(
                create_comm().await?.socket_addr(),
                info.keypair.clone(),
                event_channel::new(1).0,
                UsedSpace::new(max_capacity),
                root_storage_dir,
                genesis_sk_set.clone(),
            )
            .await?;

            let section_key_share = SectionKeyShare {
                public_key_set: secret_key_set.public_keys(),
                index: 0,
                secret_key_share: secret_key_set.secret_key_share(0),
            };

            node.section_keys_provider = SectionKeysProvider::new(Some(section_key_share));

            // get our Node to now be in prefix(0)
            let _ = node.network_knowledge.update_knowledge_if_valid(
                signed_sap.clone(),
                &chain,
                None,
                &info.name(),
                &node.section_keys_provider,
            );

            // generate other SAP for prefix1
            let (other_sap, _, secret_key_set) = random_sap(prefix1, elder_count());
            let other_sap_sk = secret_key_set.secret_key();
            let other_sap = section_signed(other_sap_sk, other_sap)?;
            // generate a proof chain for this other SAP
            let mut proof_chain = SecuredLinkedList::new(genesis_pk);
            let signature = bincode::serialize(&other_sap_sk.public_key())
                .map(|bytes| genesis_sk_set.secret_key().sign(&bytes))?;
            proof_chain.insert(&genesis_pk, other_sap_sk.public_key(), signature)?;

            Ok(Self {
                node,
                other_sap,
                proof_chain,
            })
        }

        fn create_msg(
            &self,
            src_section_prefix: &Prefix,
            src_section_pk: BlsPublicKey,
        ) -> Result<(WireMsg, SrcLocation)> {
            let sender = NodeInfo::new(
                ed25519::gen_keypair(&src_section_prefix.range_inclusive(), MIN_ADULT_AGE),
                gen_addr(),
            );

            let sender_name = sender.name();
            let src_node_keypair = sender.keypair;

            let payload_msg = SystemMsg::StartConnectivityTest(xor_name::rand::random());
            let payload = WireMsg::serialize_msg_payload(&payload_msg)?;

            let dst_name = xor_name::rand::random();
            let dst_section_key = SecretKey::random().public_key();
            let dst_location = DstLocation::Node {
                name: dst_name,
                section_pk: dst_section_key,
            };

            let msg_id = MsgId::new();

            let node_auth = NodeAuth::authorize(src_section_pk, &src_node_keypair, &payload);

            let auth = AuthKind::Node(node_auth.into_inner());

            let wire_msg = WireMsg::new_msg(msg_id, payload, auth, dst_location)?;

            let src_location = SrcLocation::Node {
                name: sender_name,
                section_pk: src_section_pk,
            };

            Ok((wire_msg, src_location))
        }

        fn create_update_msg(
            &self,
            proof_chain: SecuredLinkedList,
        ) -> Result<(SystemMsg, NodeMsgAuthority)> {
            let payload_msg = SystemMsg::AntiEntropyUpdate {
                section_auth: self.other_sap.value.to_msg(),
                section_signed: self.other_sap.sig.clone(),
                proof_chain,
                members: BTreeSet::new(),
            };

            let auth_proof = AuthorityProof(SectionAuthMsg {
                src_name: self.other_sap.value.prefix().name(),
                sig: self.other_sap.sig.clone(),
            });
            let node_auth = NodeMsgAuthority::Section(auth_proof);

            Ok((payload_msg, node_auth))
        }
    }

    // Creates a section chain with three blocks
    fn create_chain(
        sap_sk: &SecretKey,
        last_key: BlsPublicKey,
    ) -> Result<(SecuredLinkedList, bls::SecretKeySet)> {
        // create chain with random genesis key
        let genesis_sk_set = bls::SecretKeySet::random(0, &mut rand::thread_rng());
        let genesis_pk = genesis_sk_set.public_keys().public_key();
        let mut chain = SecuredLinkedList::new(genesis_pk);

        // insert second key which is the PK derived from SAP's SK
        let sap_pk = sap_sk.public_key();
        let sig = genesis_sk_set
            .secret_key()
            .sign(&bincode::serialize(&sap_pk)?);
        chain.insert(&genesis_pk, sap_pk, sig)?;

        // insert third key which is provided `last_key`
        let last_sig = sap_sk.sign(&bincode::serialize(&last_key)?);
        chain.insert(&sap_pk, last_key, last_sig)?;

        Ok((chain, genesis_sk_set))
    }
}
