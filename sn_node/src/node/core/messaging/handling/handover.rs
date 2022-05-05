// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use futures::future;
use sn_consensus::{SignedVote, VoteResponse};

use crate::node::handover::Handover;
use crate::node::{api::cmds::Cmd, core::Node, Error, Result};

use crate::node::core::Proposal;
use sn_interface::messaging::system::SectionAuth;
use sn_interface::network_knowledge::SapCandidate;
use sn_interface::types::log_markers::LogMarker;
use sn_interface::SectionAuthorityProvider;

impl Node {
    /// helper to handle a handover vote
    #[instrument(skip(self), level = "trace")]
    async fn handle_vote(
        &self,
        handover_state: &mut Handover,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Vec<Cmd> {
        match handover_state.handle_signed_vote(signed_vote.clone()) {
            Ok(VoteResponse::Broadcast(signed_vote)) => {
                trace!(
                    ">>> Handover Vote msg successfully handled, broadcasting our vote: {:?}",
                    signed_vote
                );
                self.broadcast_handover_vote_msg(signed_vote).await
            }
            Ok(VoteResponse::WaitingForMoreVotes) => {
                trace!(
                    ">>> Handover Vote msg successfully handled, awaiting for more votes: {:?}",
                    signed_vote
                );
                vec![]
            }
            Err(err) => {
                error!(">>> Failed to handle handover Vote msg: {:?}", err);
                vec![]
            }
        }
    }

    async fn check_sap(&self, sap: &SectionAuth<SectionAuthorityProvider>) -> Result<()> {
        let sap_bytes = Proposal::SectionInfo(sap.value.clone()).as_signable_bytes()?;
        if !sap.sig.verify(&sap_bytes) {
            return Err(Error::InvalidSignature);
        }
        let our_section_key = self.network_knowledge.section_key().await;
        if our_section_key != sap.sig.public_key {
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "the signer of this SAP is not our section: {:?} != {:?}",
                sap.sig.public_key, our_section_key,
            )));
        }
        Ok(())
    }

    async fn check_sap_candidate(&self, sap_candidate: &SapCandidate) -> Result<()> {
        match sap_candidate {
            SapCandidate::ElderHandover(authed_sap) => self.check_sap(authed_sap).await,
            SapCandidate::SectionSplit(authed_sap1, authed_sap2) => {
                self.check_sap(authed_sap1).await?;
                self.check_sap(authed_sap2).await
            }
        }
    }

    async fn check_signed_vote_saps(&self, signed_vote: &SignedVote<SapCandidate>) -> Result<()> {
        let sap_candidates = signed_vote.proposals();
        let checks = sap_candidates
            .iter()
            .map(|sap_can| self.check_sap_candidate(sap_can));
        let _ = future::try_join_all(checks).await?;
        Ok(())
    }

    #[instrument(skip(self), level = "trace")]
    pub(crate) async fn handle_handover_msg(
        &self,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Vec<Cmd> {
        debug!(
            ">>> {}: {:?}",
            LogMarker::HandoverMsgToBeHandled,
            signed_vote
        );

        if let Err(err) = self.check_signed_vote_saps(&signed_vote).await {
            // NB TODO tracking who sent this invalid vote?
            error!("Ignoring invalid handover vote: {}", err);
            return vec![];
        }

        let mut wlock = self.handover_voting.write().await;
        match &*wlock {
            Some(handover_state) => {
                let mut state = handover_state.clone();
                let mut cmds = self.handle_vote(&mut state, signed_vote).await;
                if let Some(candidates_sap) = state.consensus_value() {
                    debug!(
                        ">>> {}: {:?}",
                        LogMarker::HandoverConsensusTermination,
                        candidates_sap
                    );
                    // NB TOTO make sure error has to be swallowed
                    let bcast_cmds = self.broadcast_handover_decision(candidates_sap).await;
                    cmds.extend(bcast_cmds);
                }
                *wlock = Some(state);
                cmds
            }
            None => {
                trace!(">>> Non-elder node unexpectedly received handover Vote msg, ignoring...");
                vec![]
            }
        }
    }
}
