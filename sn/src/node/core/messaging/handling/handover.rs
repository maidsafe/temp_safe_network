// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_consensus::{SignedVote, VoteResponse};

use crate::node::{
    api::cmds::Cmd,
    core::Node,
};
use crate::messaging::SectionAuthorityProvider;
use crate::types::log_markers::LogMarker;
use crate::node::Handover;

impl Node {
    #[instrument(skip(self), level = "trace")]
    async fn handle_vote(
        &self,
        handover_state: Handover,
        signed_vote: SignedVote<Vec<SectionAuthorityProvider>>,
    ) -> Vec<Cmd> {
        match handover_state.handle_signed_vote(signed_vote.clone()) {
            Ok(VoteResponse::Broadcast(signed_vote)) => {
                trace!(
                    ">>> Handover Vote msg successfully handled, broadcasting our vote: {:?}",
                    signed_vote
                );
                self.broadcast_handover_vote_msg(signed_vote).await
            },
            Ok(VoteResponse::WaitingForMoreVotes) => {
                trace!(
                    ">>> Handover Vote msg successfully handled, awaiting for more votes: {:?}",
                    signed_vote
                );
                vec![]
            },
            Err(err) => {
                error!(">>> Failed to handle handover Vote msg: {:?}", err);
                vec![]
            },
        }
    }

    async fn propose_new_elders(&self, sap: SectionAuthorityProvider) -> Result<Vec<Cmd>> {
        let signed_section_auth = SectionAuth::new(sap, sig.clone());
        let saps_candidates = self
            .network_knowledge
            .promote_and_demote_elders(&self.info.read().await.name(), &BTreeSet::new())
            .await;

        if !saps_candidates.contains(&signed_section_auth.elder_candidates()) {
            // candidates_sap out of date, ignore.
            return Ok(vec![]);
        }

        // Send the `NewElders` proposal to all of the to-be-Elders so it's aggregated by them.
        let proposal = Proposal::NewElders(signed_section_auth);
        let proposal_recipients = saps_candidates
            .iter()
            .flat_map(|info| info.elders())
            .cloned()
            .collect();

        let section_key = self.network_knowledge.section_key().await;
        let key_share = self
            .section_keys_provider
            .key_share(&section_key)
            .await
            .map_err(|err| {
                trace!("Can't propose {:?}: {:?}", proposal, err);
                err
            })?;

        self.send_proposal(proposal_recipients, proposal, &key_share)
        .await
    }

    #[instrument(skip(self), level = "trace")]
    async fn broadcast_consensus_termination(&self, candidates_sap: SapCandidates) -> Vec<Cmd> {
        match candidates_sap {
            ElderHandover(sap) => {
                propose_new_elders(sap).await?
            },
            SectionSplit(sap1, sap2) => {
                let mut prop1 = propose_new_elders(sap1).await?;
                let mut prop2 = propose_new_elders(sap2).await?;
                prop1.append(&mut prop2);
                prop1
            },
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_handover_msg(
        &self,
        signed_vote: SignedVote<Vec<SectionAuthorityProvider>>,
    ) -> Vec<Cmd> {
        debug!(">>> {}: {:?}", LogMarker::HandoverMsg, signed_vote);

        // is that a copy???
        match *self.elder_handover.write().await {
            Some(handover_state) => {
                let mut cmds = self.handle_vote(handover_state, signed_vote).await;
                if let Some(candidates_sap) = handover_state.consensus_value() {
                    // should we do that every time???
                    self.broadcast_consensus_termination(candidates_sap)
                }
            },
            None => {
                trace!(">>> Non-elder node unexpectedly received handover Vote msg, ignoring...");
                vec![]
            },
        }
    }
}
