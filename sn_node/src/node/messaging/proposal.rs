// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{flow_ctrl::cmds::Cmd, messaging::Peers, MyNode, Proposal, Result};
use sn_interface::messaging::system::SectionSigShare;

use sn_interface::{
    messaging::{system::NodeMsg, MsgId},
    network_knowledge::SectionKeyShare,
    types::Peer,
};

impl MyNode {
    /// Send proposal to all our elders.
    pub(crate) fn propose(&mut self, proposal: Proposal) -> Result<Vec<Cmd>> {
        let elders = self.network_knowledge.section_auth().elders_vec();
        self.send_proposal(elders, proposal)
    }

    /// Send `proposal` to `recipients`.
    pub(crate) fn send_proposal(
        &mut self,
        recipients: Vec<Peer>,
        proposal: Proposal,
    ) -> Result<Vec<Cmd>> {
        let section_key = self.network_knowledge.section_key();

        let key_share = self
            .section_keys_provider
            .key_share(&section_key)
            .map_err(|err| {
                trace!("Can't propose {:?}: {:?}", proposal, err);
                err
            })?;

        self.send_proposal_with(recipients, proposal, &key_share)
    }

    /// Send `proposal` to `recipients` signing it with the provided key share.
    pub(crate) fn send_proposal_with(
        &mut self,
        recipients: Vec<Peer>,
        proposal: Proposal,
        key_share: &SectionKeyShare,
    ) -> Result<Vec<Cmd>> {
        trace!("Propose {proposal:?}, key_share: {key_share:?}, aggregators: {recipients:?}");

        let sig_share = proposal.sign_with_key_share(
            key_share.public_key_set.clone(),
            key_share.index,
            &key_share.secret_key_share,
        )?;

        // Broadcast the proposal to the rest of the section elders.
        let msg = NodeMsg::Propose {
            proposal: proposal.clone().into_msg(),
            sig_share: sig_share.clone(),
        };

        let msg_id = MsgId::new();

        let mut cmds = vec![];
        let our_name = self.info().name();
        // handle ourselves if we should
        for peer in recipients.clone() {
            if peer.name() == our_name {
                cmds.extend(self.handle_proposal(
                    msg_id,
                    proposal.clone(),
                    sig_share.clone(),
                    peer,
                )?)
            }
        }

        // remove ourself from recipients
        let recipients = recipients
            .into_iter()
            .filter(|peer| peer.name() != our_name)
            .collect();

        cmds.push(self.send_system_msg(msg, Peers::Multiple(recipients)));

        Ok(cmds)
    }

    pub(crate) fn handle_proposal(
        &mut self,
        msg_id: MsgId,
        proposal: Proposal,
        sig_share: SectionSigShare,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        let sig_share_pk = &sig_share.public_key_set.public_key();
        let our_prefix = self.network_knowledge.prefix();
        // Any other proposal than SectionInfo needs to be signed by a known section key.
        if let Proposal::SectionInfo(sap) = &proposal {
            if sap.prefix() == our_prefix || sap.prefix().is_extension_of(&our_prefix) {
                // This `SectionInfo` is proposed by the DKG participants and
                // it's signed by the new key created by the DKG so we don't
                // know it yet. We only require the src_name of the
                // proposal to be one of the DKG participants.
                if !sap.contains_elder(&sender.name()) {
                    trace!(
                        "Ignoring proposal from src not being a DKG participant: {:?}",
                        proposal
                    );
                    return Ok(vec![]);
                }
            }
        } else {
            // Proposal from other sections shall be ignored.
            if !our_prefix.matches(&sender.name()) {
                trace!(
                    "Ignore proposal {:?} from other section, src {}: {:?}",
                    proposal,
                    sender,
                    msg_id
                );
                return Ok(vec![]);
            }

            // Let's now verify the section key in the msg authority is trusted
            // based on our current knowledge of the network and sections chains.
            if !self.network_knowledge.has_chain_key(sig_share_pk) {
                warn!(
                    "Dropped Propose msg ({:?}) with untrusted sig share from {}: {:?}",
                    msg_id, sender, proposal
                );
                return Ok(vec![]);
            }
        }

        let mut cmds = vec![];

        match proposal.as_signable_bytes() {
            Err(error) => error!(
                "Failed to serialise proposal from {}, {:?}: {:?}",
                sender, msg_id, error
            ),
            Ok(serialised_proposal) => {
                match self
                    .proposal_aggregator
                    .try_aggregate(&serialised_proposal, sig_share)
                {
                    Ok(Some(sig)) => match proposal {
                        Proposal::NewElders(new_elders) => {
                            cmds.push(Cmd::HandleNewEldersAgreement { new_elders, sig })
                        }
                        _ => cmds.push(Cmd::HandleAgreement { proposal, sig }),
                    },
                    Ok(None) => {
                        trace!(
                        "Proposal from {} inserted in aggregator, not enough sig shares yet: {proposal:?} {:?}",
                        sender,
                        msg_id
                    );
                    }
                    Err(error) => {
                        error!(
                            "Failed to add proposal from {}, {:?}: {:?}",
                            sender, msg_id, error
                        );
                    }
                }
            }
        }

        Ok(cmds)
    }
}
