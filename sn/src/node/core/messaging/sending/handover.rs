// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_consensus::SignedVote;
use std::collections::BTreeSet;
use tracing::warn;

use crate::messaging::system::SectionAuth;
use crate::messaging::system::SystemMsg;
use crate::node::SectionAuthorityProvider;
use crate::node::{api::cmds::Cmd, core::Node, core::Proposal, handover::SapCandidate, Result};
use crate::types::log_markers::LogMarker;

impl Node {
    /// Make a handover consensus proposal vote for a sap candidate
    pub(crate) async fn propose_handover_consensus(
        &self,
        sap_candidates: SapCandidate,
    ) -> Result<Vec<Cmd>> {
        let mut wlock = self.handover_voting.write().await;
        match &*wlock {
            Some(handover_voting_state) => {
                let mut vs = handover_voting_state.clone();
                let vote = vs.propose(sap_candidates)?;
                *wlock = Some(vs);
                debug!(">>> {}: {:?}", LogMarker::HandoverConsensusTrigger, &vote);
                Ok(self.broadcast_handover_vote_msg(vote).await)
            }
            None => {
                warn!("Failed to make handover consensus proposal because node is not an Elder");
                Ok(vec![])
            }
        }
    }

    /// Broadcast handover Vote message to Elders
    pub(crate) async fn broadcast_handover_vote_msg(
        &self,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Vec<Cmd> {
        // Deliver each SignedVote to all current Elders
        trace!(">>> Broadcasting Vote msg: {:?}", signed_vote);
        let node_msg = SystemMsg::HandoverVote(signed_vote);
        match self.send_msg_to_our_elders(node_msg).await {
            Ok(cmd) => vec![cmd],
            Err(err) => {
                error!(">>> Failed to send SystemMsg::Handover message: {:?}", err);
                vec![]
            }
        }
    }

    /// Broadcast the decision of the terminated handover consensus by proposing the NewElders list
    /// for last confirmation and signature by the current elders
    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn broadcast_handover_decision(
        &self,
        candidates_sap: SapCandidate,
    ) -> Vec<Cmd> {
        match candidates_sap {
            SapCandidate::ElderHandover(sap) => {
                // NB TODO make sure this error has to be swallowed
                self.propose_new_elders(sap).await.unwrap_or_else(|e| {
                    error!("Failed to propose new elders {}", e);
                    vec![]
                })
            }
            SapCandidate::SectionSplit(sap1, sap2) => {
                let mut prop1 = self.propose_new_elders(sap1).await.unwrap_or_else(|e| {
                    error!("Failed to propose new elders {}", e);
                    vec![]
                });
                let mut prop2 = self.propose_new_elders(sap2).await.unwrap_or_else(|e| {
                    error!("Failed to propose new elders {}", e);
                    vec![]
                });
                prop1.append(&mut prop2);
                prop1
            }
        }
    }

    /// Helper function to propose a NewElders list to sign from a SAP
    async fn propose_new_elders(
        &self,
        sap: SectionAuth<SectionAuthorityProvider>,
    ) -> Result<Vec<Cmd>> {
        let saps_candidates = self
            .network_knowledge
            .promote_and_demote_elders(&self.info.read().await.name(), &BTreeSet::new())
            .await;

        if !saps_candidates.contains(&sap.elder_candidates()) {
            // candidates_sap out of date, ignore.
            return Ok(vec![]);
        }

        // Send the `NewElders` proposal to all of the to-be-Elders so it's aggregated by them.
        let proposal = Proposal::NewElders(sap);
        let proposal_recipients = saps_candidates
            .iter()
            .flat_map(|info| info.elders())
            .cloned()
            .collect();

        self.send_proposal(proposal_recipients, proposal).await
    }
}
