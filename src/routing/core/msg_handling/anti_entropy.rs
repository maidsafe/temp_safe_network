// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::{
    node::{JoinResponse, NodeMsg, Section},
    DstLocation, NodeMsgAuthority, WireMsg,
};
use crate::routing::{
    error::Result,
    messages::{NodeMsgAuthorityUtils, WireMsgUtils},
    network::NetworkUtils,
    node::Node,
    routing_api::command::Command,
    section::{SectionAuthorityProviderUtils, SectionUtils},
};
use bls::PublicKey as BlsPublicKey;
use std::{cmp::Ordering, net::SocketAddr};

impl Core {
    pub async fn check_for_entropy(
        &self,
        node_msg: &NodeMsg,
        msg_authority: &NodeMsgAuthority,
        dst_location: &DstLocation,
        sender: SocketAddr,
    ) -> Result<(Option<Command>, bool)> {
        if self.is_not_elder() {
            // Adult nodes do need to carry out entropy checking, however the message shall always
            // be handled.
            return Ok((None, true));
        }

        let (command, shall_be_handled) = match dst_location.section_pk() {
            None => return Ok((None, true)),
            Some(section_pk) => {
                match process(
                    &self.node,
                    &self.section,
                    node_msg,
                    msg_authority,
                    section_pk,
                )? {
                    (Some(msg_to_send), shall_be_handled) => {
                        let command = self.relay_message(msg_to_send).await?;
                        (command, shall_be_handled)
                    }
                    (None, shall_be_handled) => (None, shall_be_handled),
                }
            }
        };

        if shall_be_handled || command.is_some() {
            Ok((command, shall_be_handled))
        } else {
            // For the case of receiving a JoinRequest not matching our prefix.
            let sender_name = msg_authority.name();
            let section_auth = self
                .network
                .closest(&sender_name)
                .unwrap_or_else(|| self.section.authority_provider());

            let node_msg =
                NodeMsg::JoinResponse(Box::new(JoinResponse::Redirect(section_auth.clone())));

            trace!("Sending {:?} to {}", node_msg, sender_name);
            let cmd = self.send_direct_message(
                (sender_name, sender),
                node_msg,
                section_auth.section_key(),
            )?;

            Ok((Some(cmd), false))
        }
    }
}

// On reception of an incoming message, determine the msg to send in order to
// bring our's and the sender's knowledge about each other up to date. Returns a tuple of
// `NodeMsg` and `bool`. The boolean signals if the incoming message shall still be processed.
// If entropy is found, we do not process the incoming message by returning `false`.
fn process(
    node: &Node,
    section: &Section,
    node_msg: &NodeMsg,
    msg_authority: &NodeMsgAuthority,
    dst_section_pk: BlsPublicKey,
) -> Result<(Option<WireMsg>, bool)> {
    let src_name = msg_authority.name();

    if section.prefix().matches(&src_name) {
        // This message is from our section. We update our members via the `Sync` message which is
        // done elsewhere.
        return Ok((None, true));
    }

    let dst = msg_authority.src_location().to_dst();

    if let Ordering::Less = section
        .chain()
        .cmp_by_position(&dst_section_pk, section.chain().last_key())
    {
        info!("Anti-Entropy: Source's knowledge of our key is outdated, send them an update.");
        let chain = if let Ok(chain) = section.chain().get_proof_chain_to_current(&dst_section_pk) {
            chain
        } else {
            trace!(
                "Cannot find section_key {:?} within the chain",
                dst_section_pk
            );
            // In case a new node is trying to bootstrap from us, not being its matching section.
            // Reply with no msg and with false flag to send back a JoinResponse::Redirect.
            if let NodeMsg::JoinRequest(_) = node_msg {
                return Ok((None, false));
            }
            section.chain().clone()
        };

        let section_auth = section.section_signed_authority_provider();
        let node_msg = NodeMsg::SectionKnowledge {
            src_info: (section_auth.clone(), chain),
            msg: Some(Box::new(node_msg.clone())),
        };
        let msg = WireMsg::single_src(
            node,
            dst,
            node_msg,
            section.authority_provider().section_key(),
        )?;

        return Ok((Some(msg), false));
    }

    Ok((None, true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::{node::DstInfo, DstLocation};
    use crate::routing::{
        dkg::test_utils::section_signed,
        ed25519,
        section::test_utils::{gen_addr, gen_section_authority_provider},
        XorName, ELDER_SIZE, MIN_ADULT_AGE,
    };
    use anyhow::{Context, Result};
    use assert_matches::assert_matches;
    use secured_linked_list::SecuredLinkedList;
    use xor_name::Prefix;

    #[test]
    fn everything_up_to_date() -> Result<()> {
        let env = Env::new(1)?;

        let msg = env.create_message(
            &env.their_prefix,
            env.section.authority_provider().section_key(),
        )?;
        let dst_info = DstInfo {
            dst: XorName::random(),
            dst_section_pk: *env.section.chain().last_key(),
        };

        let (msg_to_send, _) = process(&env.node, &env.section, &msg, dst_info)?;
        assert_eq!(msg_to_send, None);

        Ok(())
    }

    #[test]
    fn new_src_key_from_our_section() -> Result<()> {
        let env = Env::new(1)?;

        let our_new_sk = bls::SecretKey::random();
        let our_new_pk = our_new_sk.public_key();

        let msg = env.create_message(
            env.section.prefix(),
            env.section.authority_provider().section_key(),
        )?;
        let dst_info = DstInfo {
            dst: env.node.name(),
            dst_section_pk: our_new_pk,
        };

        let (msg_to_send, _) = process(&env.node, &env.section, &msg, dst_info)?;

        assert_eq!(msg_to_send, None);

        Ok(())
    }

    #[test]
    fn outdated_dst_key_from_other_section() -> Result<()> {
        let env = Env::new(2)?;

        let msg = env.create_message(
            &env.their_prefix,
            env.section.authority_provider().section_key(),
        )?;
        let dst_info = DstInfo {
            dst: XorName::random(),
            dst_section_pk: *env.section.chain().root_key(),
        };

        let (msg_to_send, _) = process(&env.node, &env.section, &msg, dst_info)?;

        assert_matches!(msg_to_send, Some(message) => {
            assert_matches!(message.variant, Variant::SectionKnowledge { ref src_info, .. } => {
                assert_eq!(src_info.0.value, *env.section.authority_provider());
                assert_eq!(src_info.1, *env.section.chain());
            });
        });

        Ok(())
    }

    #[test]
    fn outdated_dst_key_from_our_section() -> Result<()> {
        let env = Env::new(2)?;

        let msg = env.create_message(
            env.section.prefix(),
            env.section.authority_provider().section_key(),
        )?;
        let dst_info = DstInfo {
            dst: XorName::random(),
            dst_section_pk: *env.section.chain().root_key(),
        };

        let (msg_to_send, _) = process(&env.node, &env.section, &msg, dst_info)?;

        assert_eq!(msg_to_send, None);

        Ok(())
    }

    struct Env {
        node: Node,
        section: Section,
        their_prefix: Prefix,
    }

    impl Env {
        fn new(chain_len: usize) -> Result<Self> {
            let prefix0 = Prefix::default().pushed(false);
            let prefix1 = Prefix::default().pushed(true);

            let (chain, our_sk) =
                create_chain(chain_len).context("failed to create section chain")?;

            let (section_auth0, mut nodes, _) = gen_section_authority_provider(prefix0, ELDER_SIZE);
            let node = nodes.remove(0);

            let section_auth0 = section_signed(&our_sk, section_auth0)?;
            let section = Section::new(*chain.root_key(), chain, section_auth0)
                .context("failed to create section")?;

            Ok(Self {
                node,
                section,
                their_prefix: prefix1,
            })
        }

        fn create_message(
            &self,
            src_section: &Prefix,
            section_pk: bls::PublicKey,
        ) -> Result<NodeMsg> {
            let sender = Node::new(
                ed25519::gen_keypair(&src_section.range_inclusive(), MIN_ADULT_AGE),
                gen_addr(),
            );

            Ok(NodeMsg::single_src(
                &sender,
                DstLocation::Section(self.node.name()),
                Variant::UserMessage(b"hello".to_vec()),
                section_pk,
            )?)
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
