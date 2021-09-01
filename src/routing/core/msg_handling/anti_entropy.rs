// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::{
    system::{KeyedSig, SectionAuth, SystemMsg},
    DstLocation, MessageType, SectionAuthorityProvider, SrcLocation, WireMsg,
};
use crate::routing::{
    dkg::SectionAuthUtils,
    error::{Error, Result},
    messages::WireMsgUtils,
    routing_api::command::Command,
    section::SectionUtils,
    SectionAuthorityProviderUtils,
};
use crate::types::PublicKey;
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use itertools::Itertools;
use secured_linked_list::SecuredLinkedList;
use std::net::SocketAddr;
use xor_name::XorName;

impl Core {
    pub(crate) async fn handle_anti_entropy_retry_msg(
        &mut self,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        proof_chain: SecuredLinkedList,
        bounced_msg: Bytes,
        sender: SocketAddr,
        src_name: XorName,
    ) -> Result<Vec<Command>> {
        let bounced_msg = match WireMsg::deserialize(bounced_msg)? {
            MessageType::System { msg, .. } => msg,
            _ => {
                warn!("Non Infrastructure MessageType received at Node in AE response. We do not handle any other type yet");
                return Ok(vec![]);
            }
        };

        info!(
            "Anti-Entropy: retry message received from peer: {} ({})",
            src_name, sender
        );

        match self.network.update_remote_section_sap(
            SectionAuth {
                value: section_auth.clone(),
                sig: section_signed,
            },
            &proof_chain,
            self.section.chain(),
        ) {
            Ok(updated) => {
                if updated {
                    info!(
                        "Anti-Entropy: updated remote section SAP updated for {:?}",
                        section_auth.prefix
                    );
                } else {
                    debug!(
                        "Anti-Entropy: discarded SAP for {:?} since it's the same as the one in our records: {:?}",
                        section_auth.prefix, section_auth
                    );
                }

                // Regardless if the SAP already existed, it may have been just updated by
                // a concurrent handler of another bounced msg, so we still resend this message.
                //
                // TODO: we may need to check if 'bounced_msg' dest section pk is different
                // from the received new SAP key, to prevent from endlessly resending a msg
                // if a sybil/corrupt peer keeps sending us the same AE msg.
                let dst_section_pk = section_auth.public_key_set.public_key();
                let cmd =
                    self.send_direct_message((src_name, sender), bounced_msg, dst_section_pk)?;
                Ok(vec![cmd])
            }
            Err(err) => {
                warn!("Anti-Entropy: failed to update remote section SAP, bounced msg dropped: {:?}, {}", bounced_msg, err);
                Ok(vec![])
            }
        }
    }

    pub(crate) async fn handle_anti_entropy_redirect_msg(
        &mut self,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        bounced_msg: Bytes,
        sender: SocketAddr,
    ) -> Result<Vec<Command>> {
        debug!(
            "Anti-Entropy: redirect message received from peer: {}",
            sender
        );

        let bounced_msg = match WireMsg::deserialize(bounced_msg)? {
            MessageType::System { msg, .. } => msg,
            _ => {
                warn!("Non Infrastructure MessageType received at Node in AE response. We do not handle any other type yet");
                return Ok(vec![]);
            }
        };

        // We verify SAP signature although we cannot trust it without a proof chain anyways.
        let section_signed = SectionAuth {
            value: section_auth.clone(),
            sig: section_signed,
        };
        if !section_signed.self_verify() {
            warn!(
                "Anti-Entropy: failed to verify signature of SAP received in a redirect msg, bounced msg dropped: {:?}",
                bounced_msg
            );
            return Ok(vec![]);
        }

        // When there are elders exist in both the local and incoming SAPs, send msg to the elder
        // closest to the dest section key.
        // The dst_section_pk is set to be local knowledge or genesis_key when no local knowledge.
        let (local_dst_elders, dst_section_pk) =
            match self.network.section_by_prefix(&section_auth.prefix) {
                Ok(trusted_sap) => (trusted_sap.elders, trusted_sap.public_key_set.public_key()),
                Err(_) => {
                    // In case we don't have the knowledge of that neighbour locally,
                    // have to trust the incoming SAP when it's self verifable.
                    (section_signed.value.elders, *self.section.genesis_key())
                }
            };

        // We send the msg to the Elder closest to the dest section key,
        // just to pick one of them in a random but deterministic fashion.
        let name = XorName::from(PublicKey::Bls(dst_section_pk));
        let chosen_dst_elder = local_dst_elders
            .iter()
            .filter(|(local_elder, _)| section_auth.elders.contains_key(local_elder))
            .sorted_by(|lhs, rhs| name.cmp_distance(lhs.0, rhs.0))
            .next();

        if let Some((name, addr)) = chosen_dst_elder {
            let cmd = self.send_direct_message((*name, *addr), bounced_msg, dst_section_pk)?;
            Ok(vec![cmd])
        } else {
            warn!(
                "Anti-Entropy: no trust-worthy elder among incoming SAP {:?} and local elders {:?}",
                section_auth, local_dst_elders
            );

            // For the situation non-elder exists in both incoming and local SAP, send to one of
            // the incoming elder with the geneis key to trigger AE.
            if let Some((name, addr)) = section_auth.elders.iter().next() {
                let cmd = self.send_direct_message(
                    (*name, *addr),
                    bounced_msg,
                    *self.section.genesis_key(),
                )?;
                Ok(vec![cmd])
            } else {
                error!(
                    "Anti-Entropy: incoming SAP doesn't contain any elder! {:?}",
                    section_auth
                );
                Ok(vec![])
            }
        }
    }

    pub(crate) async fn check_for_entropy_if_needed(
        &self,
        wire_msg: &WireMsg,
        msg: &SystemMsg,
        src_location: &SrcLocation,
        dst_location: &DstLocation,
        sender: SocketAddr,
    ) -> Result<Option<Command>> {
        // Adult nodes don't need to carry out entropy checking,
        // however the message shall always be handled.
        if self.is_not_elder() {
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
            | SystemMsg::AntiEntropyRedirect { .. }
            | SystemMsg::JoinRequest(_)
            | SystemMsg::JoinAsRelocatedRequest(_) => Ok(None),
            _ => match dst_location.section_pk() {
                None => Ok(None),
                Some(dst_section_pk) => {
                    self.check_for_entropy(
                        wire_msg,
                        src_location,
                        &dst_section_pk,
                        dst_location.name(),
                        sender,
                    )
                    .await
                }
            },
        }
    }

    /// Tells us if the message has reached the correct section
    /// If not, we'll need to respond with AE
    pub(crate) fn dst_is_for_our_section(&self, dst_section_pk: &BlsPublicKey) -> bool {
        // Destination section key matches our current section key
        dst_section_pk == self.section.chain().last_key()
    }

    // If entropy is found, determine the msg to send in order to
    // bring the sender's knowledge about us up to date.
    pub(crate) async fn check_for_entropy(
        &self,
        original_wire_msg: &WireMsg,
        src_location: &SrcLocation,
        dst_section_pk: &BlsPublicKey,
        dst_name: Option<XorName>,
        sender: SocketAddr,
    ) -> Result<Option<Command>> {
        if self.dst_is_for_our_section(dst_section_pk) {
            // Destination section key matches our current section key
            return Ok(None);
        }

        let bounced_msg = original_wire_msg.serialize()?;

        let ae_msg = match self
            .section
            .chain()
            .get_proof_chain_to_current(dst_section_pk)
        {
            Ok(proof_chain) => {
                info!("Anti-Entropy: sender's ({}) knowledge of our SAP is outdated, bounce msg with up to date SAP info.", sender);
                let section_signed_auth = self.section.section_signed_authority_provider().clone();
                let section_auth = section_signed_auth.value;
                let section_signed = section_signed_auth.sig;

                SystemMsg::AntiEntropyRetry {
                    section_auth,
                    section_signed,
                    proof_chain,
                    bounced_msg,
                }
            }
            Err(_) => {
                trace!(
                    "Anti-Entropy: cannot find dst_section_pk {:?} sent by {} in our chain",
                    dst_section_pk,
                    sender
                );

                // Let's try to find a section closer to the destination,
                // otherwise we just drop the message.
                let name = dst_name.ok_or_else(|| Error::InvalidDstLocation(format!("DirectAndUnrouted destination with section key ({:?}) not found in our section chain", dst_section_pk)))?;
                match self.network.closest(&name) {
                    Some(section_auth) => {
                        // Redirect to the closest section
                        SystemMsg::AntiEntropyRedirect {
                            section_auth: section_auth.value.clone(),
                            section_signed: section_auth.sig,
                            bounced_msg,
                        }
                    }
                    None => {
                        // Last ditch effort to find a better SAP the ideal section for this data
                        if let Some(section_auth) =
                            self.check_for_better_section_sap_for_data(dst_name)
                        {
                            SystemMsg::AntiEntropyRedirect {
                                section_auth: section_auth.value.clone(),
                                section_signed: section_auth.sig,
                                bounced_msg,
                            }
                            // let ae_commands = self.check_for_entropy().await
                        } else {
                            // TODO: instead of just dropping the message, don't we actually need
                            // to get up to date info from other Elders in our section as it may be
                            // a section key we are not aware of yet?
                            // ...and once we acquired new key/s we attempt AE check again?
                            error!(
                                    "Anti-Entropy: cannot reply with redirect msg for dest key {:?} to a closest section",
                                    dst_section_pk
                                );

                            return Err(Error::NoMatchingSection);
                        }
                    }
                }
            }
        };

        let wire_msg = WireMsg::single_src(
            &self.node,
            src_location.to_dst(),
            ae_msg,
            self.section.authority_provider().section_key(),
        )?;

        Ok(Some(Command::SendMessage {
            recipients: vec![(src_location.name(), sender)],
            wire_msg,
        }))
    }

    // checks to see if we're actually in the ideal section for this data
    pub(crate) fn check_for_better_section_sap_for_data(
        &self,
        data_name: Option<XorName>,
    ) -> Option<SectionAuth<SectionAuthorityProvider>> {
        if let Some(data_name) = data_name {
            let our_section = self.section.section_auth.clone();
            let better_sap = self
                .network
                .get_matching_or_opposite(&data_name)
                .unwrap_or_else(|_| self.section.section_auth.clone());

            trace!("Our SAP: {:?}", our_section);
            trace!("Better SAP for data {:?}: {:?}", data_name, better_sap);

            if better_sap != our_section {
                // Update the client of the actual destination section
                trace!(
                    "We have a better matched section for the data name {:?}",
                    data_name
                );
                Some(better_sap)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::{system::Section, MessageId, MessageType, MsgKind, NodeAuth};
    use crate::routing::{
        create_test_used_space_and_root_storage,
        dkg::test_utils::section_signed,
        ed25519,
        node::Node,
        routing_api::tests::create_comm,
        section::test_utils::{gen_addr, gen_section_authority_provider},
        XorName, ELDER_SIZE, MIN_ADULT_AGE,
    };
    use assert_matches::assert_matches;
    use bls::SecretKey;
    use eyre::{eyre, Context, Result};
    use secured_linked_list::SecuredLinkedList;
    use tokio::sync::mpsc;
    use xor_name::Prefix;

    #[tokio::test(flavor = "multi_thread")]
    async fn ae_everything_up_to_date() -> Result<()> {
        let env = Env::new().await?;

        let (_msg, src_location) = env.create_message(
            env.core.section().prefix(),
            *env.core.section_chain().last_key(),
        )?;
        let sender = env.core.node().addr;
        let dst_section_pk = *env.core.section_chain().last_key();

        let command = env
            .core
            .check_for_entropy(&_msg, &src_location, &dst_section_pk, None, sender)
            .await?;

        assert!(command.is_none());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ae_redirect_to_other_section() -> Result<()> {
        let env = Env::new().await?;

        let other_sk = bls::SecretKey::random();
        let other_pk = other_sk.public_key();

        let (_msg, src_location) = env.create_message(&env.other_prefix, other_pk)?;
        let sender = env.core.node().addr;

        // since it's not aware of the other prefix, it shall fail with NoMatchingSection
        let dst_section_pk = other_pk;
        let dst_name = Some(env.other_prefix.name());
        match env
            .core
            .check_for_entropy(&_msg, &src_location, &dst_section_pk, dst_name, sender)
            .await
        {
            Err(Error::NoMatchingSection) => {}
            _ => return Err(eyre!("expected Error::NoMatchingSection")),
        }

        // now let's insert a SAP to make it aware of the other prefix
        let (some_other_auth, _, _) = gen_section_authority_provider(env.other_prefix, ELDER_SIZE);
        let some_other_sap = section_signed(&other_sk, some_other_auth)?;
        let _ = env
            .core
            .network
            .insert(some_other_sap.value.prefix, some_other_sap.clone());

        // and it now shall give us an AE redirect msg
        // with the SAP we inserted for other prefix
        let command = env
            .core
            .check_for_entropy(&_msg, &src_location, &dst_section_pk, dst_name, sender)
            .await?;

        let msg_type = assert_matches!(command, Some(Command::SendMessage { wire_msg, .. }) => {
            wire_msg
                .into_message()
                .context("failed to deserialised anti-entropy message")?
        });

        assert_matches!(msg_type, MessageType::System{ msg, .. } => {
            assert_matches!(msg, SystemMsg::AntiEntropyRedirect { section_auth, .. } => {
                assert_eq!(section_auth, some_other_sap.value);
            });
        });

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ae_outdated_dst_key_of_our_section() -> Result<()> {
        let env = Env::new().await?;

        let (_msg, src_location) = env.create_message(
            env.core.section().prefix(),
            *env.core.section_chain().last_key(),
        )?;
        let sender = env.core.node().addr;
        let dst_section_pk = *env.core.section_chain().root_key();

        let command = env
            .core
            .check_for_entropy(&_msg, &src_location, &dst_section_pk, None, sender)
            .await?;

        let msg_type = assert_matches!(command, Some(Command::SendMessage { wire_msg, .. }) => {
            wire_msg
                .into_message()
                .context("failed to deserialised anti-entropy message")?
        });

        assert_matches!(msg_type, MessageType::System{ msg, .. } => {
            assert_matches!(msg, SystemMsg::AntiEntropyRetry { ref section_auth, ref proof_chain, .. } => {
                assert_eq!(section_auth, env.core.section().authority_provider());
                assert_eq!(proof_chain, env.core.section_chain());
            });
        });

        Ok(())
    }

    struct Env {
        core: Core,
        other_prefix: Prefix,
    }

    impl Env {
        async fn new() -> Result<Self> {
            let prefix0 = Prefix::default().pushed(false);
            let prefix1 = Prefix::default().pushed(true);

            let (section_auth, mut nodes, secret_key_set) =
                gen_section_authority_provider(prefix0, ELDER_SIZE);
            let node = nodes.remove(0);
            let sap_secret_key = secret_key_set.secret_key();
            let signed_section_auth = section_signed(sap_secret_key, section_auth)?;

            let chain = create_chain(
                sap_secret_key,
                signed_section_auth.value.public_key_set.public_key(),
            )
            .context("failed to create section chain")?;
            let section = Section::new(*chain.root_key(), chain, signed_section_auth)
                .context("failed to create section")?;

            let (used_space, root_storage_dir) = create_test_used_space_and_root_storage()?;
            let tmp_core = Core::first_node(
                create_comm().await?,
                node.clone(),
                mpsc::channel(1).0,
                used_space,
                root_storage_dir,
            )?;
            let core = tmp_core.relocated(node, section).await?;

            Ok(Self {
                core,
                other_prefix: prefix1,
            })
        }

        fn create_message(
            &self,
            src_section_prefix: &Prefix,
            src_section_pk: BlsPublicKey,
        ) -> Result<(WireMsg, SrcLocation)> {
            let sender = Node::new(
                ed25519::gen_keypair(&src_section_prefix.range_inclusive(), MIN_ADULT_AGE),
                gen_addr(),
            );

            let sender_name = sender.name();
            let src_node_keypair = sender.keypair;

            let payload_msg = SystemMsg::StartConnectivityTest(XorName::random());
            let payload = WireMsg::serialize_msg_payload(&payload_msg)?;

            let dst_name = XorName::random();
            let dst_section_pk = SecretKey::random().public_key();
            let dst_location = DstLocation::Node {
                name: dst_name,
                section_pk: dst_section_pk,
            };

            let msg_id = MessageId::new();

            let node_auth = NodeAuth::authorize(src_section_pk, &src_node_keypair, &payload);

            let msg_kind = MsgKind::NodeAuthMsg(node_auth.into_inner());

            let wire_msg = WireMsg::new_msg(msg_id, payload, msg_kind, dst_location)?;

            let src_location = SrcLocation::Node {
                name: sender_name,
                section_pk: src_section_pk,
            };

            Ok((wire_msg, src_location))
        }
    }

    // Creates a section chain with three blocks
    fn create_chain(sap_sk: &SecretKey, last_key: BlsPublicKey) -> Result<SecuredLinkedList> {
        // create chain with random genesis key
        let genesis_sk = SecretKey::random();
        let genesis_pk = genesis_sk.public_key();
        let mut chain = SecuredLinkedList::new(genesis_pk);

        // insert second key which is the PK derived from SAP's SK
        let sap_pk = sap_sk.public_key();
        let sig = genesis_sk.sign(&bincode::serialize(&sap_pk)?);
        chain.insert(&genesis_pk, sap_pk, sig)?;

        // insert third key which is provided `last_key`
        let last_sig = sap_sk.sign(&bincode::serialize(&last_key)?);
        chain.insert(&sap_pk, last_key, last_sig)?;

        Ok(chain)
    }
}
