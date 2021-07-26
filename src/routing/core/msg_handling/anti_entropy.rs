// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::{
    node::{KeyedSig, NodeMsg},
    DstLocation, SectionAuthorityProvider, SrcLocation, WireMsg,
};
use crate::routing::{
    error::Result, messages::WireMsgUtils, routing_api::command::Command, section::SectionUtils,
    SectionAuthorityProviderUtils,
};
use bls::PublicKey as BlsPublicKey;
use secured_linked_list::SecuredLinkedList;
use std::{cmp::Ordering, net::SocketAddr};

impl Core {
    pub(crate) async fn handle_anti_entropy_bounced_msg(
        &self,
        _section_auth: SectionAuthorityProvider,
        _section_signed: KeyedSig,
        _proof_chain: SecuredLinkedList,
        _bounced_msg: Box<NodeMsg>,
        _sender: SocketAddr,
    ) -> Result<Vec<Command>> {
        unimplemented!();
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

        // For the case of receiving a JoinRequest not matching our prefix,
        // we just let the JoinRequest handler to deal with it later on.
        if let NodeMsg::JoinRequest(_) = node_msg {
            return Ok(None);
        }

        match dst_location.section_pk() {
            None => Ok(None),
            Some(dst_section_pk) => {
                self.check_dest_information(node_msg, src_location, &dst_section_pk, sender)
                    .await
            }
        }
    }

    // If entropy is found, determine the msg to send in order to
    // bring the sender's knowledge about us up to date.
    pub(crate) async fn check_dest_information(
        &self,
        node_msg: &NodeMsg,
        src_location: &SrcLocation,
        dst_section_pk: &BlsPublicKey,
        sender: SocketAddr,
    ) -> Result<Option<Command>> {
        if let Ordering::Less = self
            .section
            .chain()
            .cmp_by_position(dst_section_pk, self.section.chain().last_key())
        {
            info!("Anti-Entropy: sender's ({}) knowledge of our SAP is outdated, bounce msg with up to date SAP info.", sender);
            let proof_chain = if let Ok(chain) = self
                .section
                .chain()
                .get_proof_chain_to_current(dst_section_pk)
            {
                chain
            } else {
                trace!(
                    "Cannot find section_key {:?} within the chain",
                    dst_section_pk
                );

                self.section.chain().clone()
            };

            let section_signed_auth = self.section.section_signed_authority_provider().clone();
            let section_auth = section_signed_auth.value;
            let section_signed = section_signed_auth.sig;

            let knowledge_node_msg = NodeMsg::AntiEntropyRetry {
                section_auth,
                section_signed,
                proof_chain,
                bounced_msg: Box::new(node_msg.clone()),
            };
            let wire_msg = WireMsg::single_src(
                &self.node,
                src_location.to_dst(),
                knowledge_node_msg,
                self.section.authority_provider().section_key(),
            )?;

            Ok(Some(Command::SendMessage {
                recipients: vec![(src_location.name(), sender)],
                wire_msg,
            }))
        } else {
            Ok(None)
        }
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
            .check_dest_information(&node_msg, &src_location, &dst_section_pk, sender)
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

        let msg_to_send = env
            .core
            .check_dest_information(&node_msg, &src_location, &our_new_pk, sender)
            .await?;

        assert!(msg_to_send.is_none());

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
            .check_dest_information(&node_msg, &src_location, &dst_section_pk, sender)
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
            .check_dest_information(&node_msg, &src_location, &dst_section_pk, sender)
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
