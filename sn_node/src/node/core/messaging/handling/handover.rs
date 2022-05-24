// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_consensus::{Generation, SignedVote, VoteResponse};

use crate::node::core::SystemMsg;
use crate::node::handover::Error as HandoverError;
use crate::node::handover::Handover;
use crate::node::Peer;
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
        peer: Peer,
    ) -> Result<Vec<Cmd>> {
        match handover_state.handle_signed_vote(signed_vote.clone()) {
            Ok(VoteResponse::Broadcast(signed_vote)) => {
                trace!(
                    ">>> Handover Vote msg successfully handled, broadcasting our vote: {:?}",
                    signed_vote
                );
                Ok(self.broadcast_handover_vote_msg(signed_vote).await)
            }
            Ok(VoteResponse::WaitingForMoreVotes) => {
                trace!(
                    ">>> Handover Vote msg successfully handled, awaiting for more votes: {:?}",
                    signed_vote
                );
                Ok(vec![])
            }
            Err(HandoverError::RequestAntiEntropy) => {
                trace!("Handover - We are behind the voter, requesting AE");
                Err(Error::RequestHandoverAntiEntropy(
                    handover_state.generation(),
                ))
            }
            Err(err) => {
                error!(">>> Failed to handle handover Vote msg: {:?}", err);
                Ok(vec![])
            }
        }
    }

    /// Verifies the SAP signature and checks that the signature's public key matches the
    /// signature of the SAP, because SAP candidates are signed by the candidate section key
    fn check_sap(
        &self,
        sap: &SectionAuth<SectionAuthorityProvider>,
        gen: Generation,
    ) -> Result<()> {
        let sap_bytes = Proposal::SectionInfo {
            sap: sap.value.clone(),
            generation: gen,
        }
        .as_signable_bytes()?;
        if !sap.sig.verify(&sap_bytes) {
            return Err(Error::InvalidSignature);
        }
        if sap.value.section_key() != sap.sig.public_key {
            return Err(Error::UntrustedSectionAuthProvider(format!(
                "the auth around the SAP does not match the SAP's public key: {:?} != {:?}",
                sap.sig.public_key,
                sap.value.section_key(),
            )));
        }
        Ok(())
    }

    fn check_sap_candidate(&self, sap_candidate: &SapCandidate, gen: Generation) -> Result<()> {
        match sap_candidate {
            SapCandidate::ElderHandover(authed_sap) => self.check_sap(authed_sap, gen),
            SapCandidate::SectionSplit(authed_sap1, authed_sap2) => {
                self.check_sap(authed_sap1, gen)?;
                self.check_sap(authed_sap2, gen)
            }
        }
    }

    fn check_signed_vote_saps(&self, signed_vote: &SignedVote<SapCandidate>) -> Result<()> {
        let sap_candidates = signed_vote.proposals();
        let gen = signed_vote.vote.gen;
        let checks: Result<Vec<_>> = sap_candidates
            .iter()
            .map(|sap_can| self.check_sap_candidate(sap_can, gen))
            .collect();
        let _ = checks?;
        Ok(())
    }

    async fn handle_handover_vote(
        &self,
        peer: Peer,
        signed_vote: SignedVote<SapCandidate>,
    ) -> Result<Vec<Cmd>> {
        self.check_signed_vote_saps(&signed_vote)?;

        let mut wlock = self.handover_voting.write().await;
        match &*wlock {
            Some(handover_state) => {
                let mut state = handover_state.clone();
                let mut cmds = self.handle_vote(&mut state, signed_vote, peer).await?;

                // check for unsuccessful termination
                state.handle_empty_set_decision();

                // check for successful termination
                if let Some(candidates_sap) = state.consensus_value() {
                    debug!(
                        "{}: {:?}",
                        LogMarker::HandoverConsensusTermination,
                        candidates_sap
                    );

                    let bcast_cmds = self.broadcast_handover_decision(candidates_sap).await;
                    cmds.extend(bcast_cmds);
                }
                *wlock = Some(state);
                Ok(cmds)
            }
            None => {
                trace!("Non-elder node unexpectedly received handover Vote msg, ignoring...");
                Ok(vec![])
            }
        }
    }

    pub(crate) async fn handle_handover_msg(
        &self,
        peer: Peer,
        signed_votes: Vec<SignedVote<SapCandidate>>,
    ) -> Result<Vec<Cmd>> {
        debug!("{}", LogMarker::HandoverMsgBeingHandled);

        let mut cmds = vec![];

        for vote in signed_votes {
            match self.handle_handover_vote(peer, vote).await {
                Ok(commands) => {
                    cmds.extend(commands);
                }
                Err(Error::RequestHandoverAntiEntropy(gen)) => {
                    // We hit an error while processing this vote, perhaps we are missing information.
                    // We'll send a handover AE request to see if they can help us catch up.
                    let sap = self.network_knowledge.authority_provider().await;
                    let dst_section_pk = sap.section_key();
                    let section_name = self.network_knowledge.prefix().await.name();
                    let msg = SystemMsg::HandoverAE(gen);
                    let cmd = self
                        .send_direct_msg_to_nodes(vec![peer], msg, section_name, dst_section_pk)
                        .await?;

                    debug!("{:?}", LogMarker::HandoverSendingAeUpdateRequest);
                    cmds.push(cmd);
                    // return the vec w/ the AE cmd there so as not to loop and generate AE for
                    // any subsequent commands
                    return Ok(cmds);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(cmds)
    }

    pub(crate) async fn handle_handover_anti_entropy(
        &self,
        peer: Peer,
        gen: Generation,
    ) -> Result<Vec<Cmd>> {
        debug!(
            "{:?} handover anti-entropy request for gen {:?} from {}",
            LogMarker::HandoverAeRequestReceived,
            gen,
            peer,
        );

        let cmds = if let Some(handover) = self.handover_voting.read().await.as_ref() {
            match handover.anti_entropy(gen) {
                Ok(catchup_votes) => {
                    vec![
                        self.send_direct_msg(
                            peer,
                            SystemMsg::HandoverVotes(catchup_votes),
                            self.network_knowledge.section_key().await,
                        )
                        .await?,
                    ]
                }
                Err(e) => {
                    error!("Handover - Error while processing anti-entropy {:?}", e);
                    vec![]
                }
            }
        } else {
            error!("Unexpected attempt to handle handover anti-entropy when we don't have a handover instance (handover is for elders only)");
            vec![]
        };

        Ok(cmds)
    }
}
