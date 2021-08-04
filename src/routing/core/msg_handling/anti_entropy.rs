// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::{
    node::{KeyedSig, NodeMsg, SectionAuth},
    DstLocation, SectionAuthorityProvider, SrcLocation, WireMsg,
};
use crate::routing::{
    dkg::SectionAuthUtils, error::Result, messages::WireMsgUtils, network::NetworkUtils,
    routing_api::command::Command, section::SectionUtils, SectionAuthorityProviderUtils,
};
use crate::types::PublicKey;
use bls::PublicKey as BlsPublicKey;
use secured_linked_list::SecuredLinkedList;
use std::{cmp::Ordering, net::SocketAddr};
use xor_name::XorName;

impl Core {
    pub(crate) async fn handle_anti_entropy_retry_msg(
        &mut self,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        proof_chain: SecuredLinkedList,
        bounced_msg: Box<NodeMsg>,
        sender: SocketAddr,
        src_name: XorName,
    ) -> Result<Vec<Command>> {
        info!("Anti-Entropy: retry message received from peer: {}", sender);

        let dst_section_pk = section_auth.public_key_set.public_key();
        let section_signed = SectionAuth {
            value: section_auth,
            sig: section_signed,
        };

        match self.network.update_remote_section_sap(
            section_signed,
            &proof_chain,
            self.section.chain(),
        ) {
            Ok(_) => {
                // Regardless if the SAP already existed, it may have been just updated by
                // a concurrent handler of another bounced msg, so we still resend this message.
                // TODO: we may need to check if 'bounced_msg' dest section pk is different
                // from the received new SAP pk, to prevent from endlessly resending a msg
                // if a sybil/corrupt peer keeps sending us the same AE msg.
                let cmd =
                    self.send_direct_message((src_name, sender), *bounced_msg, dst_section_pk)?;
                Ok(vec![cmd])
            }
            Err(err) => {
                warn!("Anti-Entropy: failed to update remote section SAP upon receiving Anti-Entropy bounced msg: {:?}, {}", bounced_msg, err);
                Ok(vec![])
            }
        }
    }

    pub(crate) async fn handle_anti_entropy_redirect_msg(
        &mut self,
        section_auth: SectionAuthorityProvider,
        section_signed: KeyedSig,
        bounced_msg: Box<NodeMsg>,
        sender: SocketAddr,
    ) -> Result<Vec<Command>> {
        info!(
            "Anti-Entropy: redirect message received from peer: {}",
            sender
        );

        let section_signed = SectionAuth {
            value: section_auth,
            sig: section_signed,
        };

        // TODO: is there a value in verifying SAP signature since we cannot trust it anyways??
        if section_signed.self_verify() {
            // We retrieve a SAP we know and trust from our records to send the redirect msg,
            // if we don't have one then we'll send the msg with the genesis key so we get
            // a AE-Retry with a proof chain we can verify the SAP with.
            let (dst_elders, dst_section_pk) = match self
                .network
                .section_by_name(&section_signed.value.prefix.name())
            {
                Ok(section_auth) => (
                    section_auth.elders,
                    section_auth.public_key_set.public_key(),
                ),
                Err(_) => (section_signed.value.elders, *self.section.genesis_key()),
            };

            // FIXME: pick the closest Elder instead??
            if let Some((name, addr)) = dst_elders.iter().next() {
                let cmd = self.send_direct_message((*name, *addr), *bounced_msg, dst_section_pk)?;
                return Ok(vec![cmd]);
            }
        } else {
            warn!(
                "Anti-Entropy: failed to verify SAP signature upon receiving a redirect msg for: {:?}",
                bounced_msg
            );
        }

        Ok(vec![])
    }

    pub(crate) async fn check_for_entropy(
        &self,
        node_msg: &NodeMsg,
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
        match node_msg {
            NodeMsg::AntiEntropyRetry { .. }
            | NodeMsg::AntiEntropyRedirect { .. }
            | NodeMsg::JoinRequest(_)
            | NodeMsg::JoinAsRelocatedRequest(_) => return Ok(None),
            _ => {}
        }

        match dst_location.section_pk() {
            None => Ok(None),
            Some(dst_section_pk) => {
                self.check_dest_section_pk(node_msg, src_location, &dst_section_pk, sender)
                    .await
            }
        }
    }

    // If entropy is found, determine the msg to send in order to
    // bring the sender's knowledge about us up to date.
    pub(crate) async fn check_dest_section_pk(
        &self,
        node_msg: &NodeMsg,
        src_location: &SrcLocation,
        dst_section_pk: &BlsPublicKey,
        sender: SocketAddr,
    ) -> Result<Option<Command>> {
        if self
            .section
            .chain()
            .cmp_by_position(dst_section_pk, self.section.chain().last_key())
            != Ordering::Less
        {
            // Destination section key matches our current section key
            return Ok(None);
        }

        info!("Anti-Entropy: sender's ({}) knowledge of our SAP is outdated, bounce msg with up to date SAP info.", sender);
        let ae_node_msg = match self
            .section
            .chain()
            .get_proof_chain_to_current(dst_section_pk)
        {
            Ok(proof_chain) => {
                let section_signed_auth = self.section.section_signed_authority_provider().clone();
                let section_auth = section_signed_auth.value;
                let section_signed = section_signed_auth.sig;

                NodeMsg::AntiEntropyRetry {
                    section_auth,
                    section_signed,
                    proof_chain,
                    bounced_msg: Box::new(node_msg.clone()),
                }
            }
            Err(_) => {
                trace!(
                    "Anti-Entropy: cannot find dst_section_pk {:?} sent by {} in our chain",
                    dst_section_pk,
                    sender
                );

                // Let's try to find a section closer to the destination section key,
                // otherwise we just drop the message.
                let name = XorName::from(PublicKey::Bls(*dst_section_pk));
                match self.network.closest(&name) {
                    Some(section_auth)
                        if &section_auth.value != self.section.authority_provider() =>
                    {
                        // Redirect to the closest section
                        NodeMsg::AntiEntropyRedirect {
                            section_auth: section_auth.value.clone(),
                            section_signed: section_auth.sig.clone(),
                            bounced_msg: Box::new(node_msg.clone()),
                        }
                    }
                    Some(_) | None => {
                        // TODO: instead of just dropping the message, don't we actually need
                        // to get up to date info from other Elders in our section as it may be
                        // a section key we are not aware of yet?
                        // ...and once we acquired new key/s we attempt AE check again?
                        error!(
                                "Anti-Entropy: cannot reply with redirect msg for dest key {:?} to a closest section",
                                dst_section_pk
                            );
                        // FIXME: drop the message
                        return Ok(None);
                    }
                }
            }
        };

        let wire_msg = WireMsg::single_src(
            &self.node,
            src_location.to_dst(),
            ae_node_msg,
            self.section.authority_provider().section_key(),
        )?;

        Ok(Some(Command::SendMessage {
            recipients: vec![(src_location.name(), sender)],
            wire_msg,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::{node::Section, MessageType};
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
    use eyre::{Context, Result};
    use secured_linked_list::SecuredLinkedList;
    use tokio::sync::mpsc;
    use xor_name::Prefix;

    #[tokio::test(flavor = "multi_thread")]
    async fn ae_everything_up_to_date() -> Result<()> {
        let env = Env::new(1).await?;

        let (node_msg, src_location) = env.create_message(
            env.core.section().prefix(),
            *env.core.section_chain().last_key(),
        )?;
        let sender = env.core.node().addr;
        let dst_section_pk = *env.core.section_chain().last_key();

        let command = env
            .core
            .check_dest_section_pk(&node_msg, &src_location, &dst_section_pk, sender)
            .await?;

        assert!(command.is_none());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ae_newer_dst_key_of_our_section() -> Result<()> {
        let env = Env::new(1).await?;

        let our_new_sk = bls::SecretKey::random();
        let our_new_pk = our_new_sk.public_key();

        let (node_msg, src_location) =
            env.create_message(env.core.section().prefix(), our_new_pk)?;
        let sender = env.core.node().addr;

        let command = env
            .core
            .check_dest_section_pk(&node_msg, &src_location, &our_new_pk, sender)
            .await?;

        let msg_type = assert_matches!(command, Some(Command::SendMessage { wire_msg, .. }) => {
            wire_msg
                .into_message()
                .context("failed to deserialised anti-entropy message")?
        });

        assert_matches!(msg_type, MessageType::Node{ msg, .. } => {
            assert_matches!(msg, NodeMsg::AntiEntropyRetry { ref section_auth, ref proof_chain, .. } => {
                assert_eq!(section_auth, env.core.section().authority_provider());
                assert_eq!(proof_chain, env.core.section_chain());
            });
        });

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ae_redirect_to_other_section() -> Result<()> {
        let env = Env::new(2).await?;

        let (node_msg, src_location) =
            env.create_message(&env.their_prefix, *env.core.section_chain().last_key())?;
        let sender = env.core.node().addr;

        let dst_section_pk = *env.core.section_chain().root_key();
        let command = env
            .core
            .check_dest_section_pk(&node_msg, &src_location, &dst_section_pk, sender)
            .await?;

        let msg_type = assert_matches!(command, Some(Command::SendMessage { wire_msg, .. }) => {
            wire_msg
                .into_message()
                .context("failed to deserialised anti-entropy message")?
        });

        assert_matches!(msg_type, MessageType::Node{ msg, .. } => {
            assert_matches!(msg, NodeMsg::AntiEntropyRedirect { ref section_auth, .. } => {
                assert_eq!(section_auth, env.core.section().authority_provider());
            });
        });

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ae_outdated_dst_key_of_our_section() -> Result<()> {
        let env = Env::new(2).await?;

        let (node_msg, src_location) = env.create_message(
            env.core.section().prefix(),
            *env.core.section_chain().last_key(),
        )?;
        let sender = env.core.node().addr;
        let dst_section_pk = *env.core.section_chain().root_key();

        let command = env
            .core
            .check_dest_section_pk(&node_msg, &src_location, &dst_section_pk, sender)
            .await?;

        let msg_type = assert_matches!(command, Some(Command::SendMessage { wire_msg, .. }) => {
            wire_msg
                .into_message()
                .context("failed to deserialised anti-entropy message")?
        });

        assert_matches!(msg_type, MessageType::Node{ msg, .. } => {
            assert_matches!(msg, NodeMsg::AntiEntropyRetry { ref section_auth, ref proof_chain, .. } => {
                assert_eq!(section_auth, env.core.section().authority_provider());
                assert_eq!(proof_chain, env.core.section_chain());
            });
        });

        Ok(())
    }

    struct Env {
        core: Core,
        their_prefix: Prefix,
    }

    impl Env {
        async fn new(chain_len: usize) -> Result<Self> {
            let prefix0 = Prefix::default().pushed(false);
            let prefix1 = Prefix::default().pushed(true);

            let (chain, our_sk) =
                create_chain(chain_len).context("failed to create section chain")?;

            let (section_auth, mut nodes, _) = gen_section_authority_provider(prefix0, ELDER_SIZE);
            let node = nodes.remove(0);

            let signed_section_auth = section_signed(&our_sk, section_auth)?;
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
                their_prefix: prefix1,
            })
        }

        fn create_message(
            &self,
            src_section_prefix: &Prefix,
            src_section_pk: BlsPublicKey,
        ) -> Result<(NodeMsg, SrcLocation)> {
            let sender = Node::new(
                ed25519::gen_keypair(&src_section_prefix.range_inclusive(), MIN_ADULT_AGE),
                gen_addr(),
            );

            let node_msg = NodeMsg::StartConnectivityTest(XorName::random());

            let src_location = SrcLocation::Node {
                name: sender.name(),
                section_pk: src_section_pk,
            };

            Ok((node_msg, src_location))
        }
    }

    fn create_chain(len: usize) -> Result<(SecuredLinkedList, bls::SecretKey)> {
        let mut sk = bls::SecretKey::random();
        let mut chain = SecuredLinkedList::new(sk.public_key());

        for _ in 1..len {
            let old_pk = *chain.last_key();

            let new_sk = bls::SecretKey::random();
            let new_pk = new_sk.public_key();
            let new_signature = sk.sign(&bincode::serialize(&new_pk)?);

            chain.insert(&old_pk, new_pk, new_signature)?;
            sk = new_sk
        }

        Ok((chain, sk))
    }
}
