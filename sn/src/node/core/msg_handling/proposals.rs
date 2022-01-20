// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{signature_aggregator::Error as AggregatorError, MessageId};
use crate::node::{
    api::command::Command,
    core::{Core, Proposal},
    dkg::SigShare,
    Result,
};
use crate::peer::Peer;

// Decisions
impl Core {
    // Insert the proposal into the proposal aggregator and handle it if aggregated.
    pub(crate) async fn handle_proposal(
        &self,
        msg_id: MessageId,
        proposal: Proposal,
        sig_share: SigShare,
        sender: Peer,
    ) -> Result<Vec<Command>> {
        let sig_share_pk = &sig_share.public_key_set.public_key();

        // Any other proposal than SectionInfo needs to be signed by a known section key.
        if let Proposal::SectionInfo(section_auth) = &proposal {
            if section_auth.prefix() == self.network_knowledge.prefix().await
                || section_auth
                    .prefix()
                    .is_extension_of(&self.network_knowledge.prefix().await)
            {
                // This `SectionInfo` is proposed by the DKG participants and
                // it's signed by the new key created by the DKG so we don't
                // know it yet. We only require the src_name of the
                // proposal to be one of the DKG participants.
                if !section_auth.contains_elder(&sender.name()) {
                    trace!(
                        "Ignoring proposal from src not being a DKG participant: {:?}",
                        proposal
                    );
                    return Ok(vec![]);
                }
            }
        } else {
            // Proposal from other section shall be ignored.
            // TODO: check this is for our prefix , or a child prefix, otherwise just drop it
            if !self
                .network_knowledge
                .prefix()
                .await
                .matches(&sender.name())
            {
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
            if !self.network_knowledge.has_chain_key(sig_share_pk).await {
                warn!(
                    "Dropped Propose msg ({:?}) with untrusted sig share from {}: {:?}",
                    msg_id, sender, proposal
                );
                return Ok(vec![]);
            }
        }

        let mut commands = vec![];

        match proposal.as_signable_bytes() {
            Err(error) => error!(
                "Failed to serialise proposal from {}, {:?}: {:?}",
                sender, msg_id, error
            ),
            Ok(serialised_proposal) => {
                match self
                    .proposal_aggregator
                    .add(&serialised_proposal, sig_share)
                    .await
                {
                    Ok(sig) => match proposal {
                        Proposal::NewElders(_) => {
                            commands.push(Command::HandleNewEldersAgreement { proposal, sig })
                        }
                        _ => commands.push(Command::HandleAgreement { proposal, sig }),
                    },
                    Err(AggregatorError::NotEnoughShares) => {
                        trace!(
                            "Proposal from {} inserted in aggregator, not enough sig shares yet: {:?}",
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

        Ok(commands)
    }
}
