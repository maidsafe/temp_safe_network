// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::super::Core;
use crate::routing::{
    dkg::{DkgFailChecker, DkgFailureSigSetUtils},
    error::{Error, Result},
    routing_api::command::Command,
    section::{SectionLogic, SectionPeersLogic},
    SectionAuthorityProviderUtils,
};
use crate::{
    messaging::{
        node::{DkgFailureSig, DkgFailureSigSet, DkgKey, ElderCandidates, NodeMsg, Proposal},
        SectionAuthorityProvider,
    },
    routing::dkg::SectionDkgOutcome,
};
use bls_dkg::key_gen::message::Message as DkgMessage;
use std::{collections::BTreeSet, slice};
use xor_name::XorName;

impl Core {
    pub(crate) async fn handle_dkg_start(
        &self,
        dkg_key: DkgKey,
        elder_candidates: ElderCandidates,
    ) -> Result<Vec<Command>> {
        trace!("Received DkgStart for {:?}", elder_candidates);
        self.dkg_voter
            .start(
                &self.node,
                dkg_key,
                elder_candidates,
                *self.section_chain().await.last_key(),
            )
            .await
    }

    pub(crate) async fn handle_dkg_message(
        &self,
        dkg_key: DkgKey,
        message: DkgMessage,
        sender: XorName,
    ) -> Result<Vec<Command>> {
        trace!("handle DKG message {:?} from {}", message, sender);

        self.dkg_voter
            .process_message(
                &self.node,
                &dkg_key,
                message,
                *self.section_chain().await.last_key(),
            )
            .await
    }

    pub(crate) async fn handle_dkg_failure_observation(
        &self,
        dkg_key: DkgKey,
        failed_participants: &BTreeSet<XorName>,
        signed: DkgFailureSig,
    ) -> Result<Vec<Command>> {
        match self
            .dkg_voter
            .process_failure(&dkg_key, failed_participants, signed)
            .await
        {
            None => Ok(vec![]),
            Some(cmd) => Ok(vec![cmd]),
        }
    }

    pub(crate) async fn handle_dkg_failure_agreement(
        &self,
        sender: &XorName,
        failure_set: &DkgFailureSigSet,
    ) -> Result<Vec<Command>> {
        let sender = &self
            .section
            .members()
            .get(sender)
            .await
            .ok_or(Error::InvalidSrcLocation)?
            .peer;

        let set_checker: DkgFailChecker = failure_set.into();

        let generation = self.section.main_branch_len().await as u64;
        let elder_candidates = self
            .section
            .promote_and_demote_elders(&self.node.name())
            .await
            .into_iter()
            .find(|elder_candidates| set_checker.verify(elder_candidates, generation));
        let elder_candidates = if let Some(elder_candidates) = elder_candidates {
            elder_candidates
        } else {
            trace!("Ignore DKG failure agreement with invalid signeds or outdated participants",);
            return Ok(vec![]);
        };

        if failure_set.failed_participants.is_empty() {
            // The DKG failure is a corrupted one due to lagging.
            trace!(
                "Received DKG failure agreement - restarting: {:?}",
                elder_candidates
            );

            self.send_dkg_start_to(elder_candidates, slice::from_ref(sender))
                .await
        } else {
            // The DKG failure is regarding failed_participants, i.e. potential unresponsive node.
            trace!(
                "Received DKG failure agreement of failed_participants {:?} , DKG generation({}) {:?}",
                failure_set.failed_participants,
                generation,
                elder_candidates
            );
            self.cast_offline_proposals(&failure_set.failed_participants)
                .await
        }
    }

    pub(crate) async fn handle_dkg_outcome(
        &self,
        section_auth: SectionAuthorityProvider,
        key_share: SectionDkgOutcome,
    ) -> Result<Vec<Command>> {
        let proposal = Proposal::SectionInfo(section_auth);
        let recipients: Vec<_> = self.section.authority_provider().await.peers().collect();

        let public_key = key_share.public_key();

        self.section_keys.get().await.insert_dkg_outcome(key_share);

        if self.section.has_key(&public_key).await {
            let section_keys = self.section_keys.get().await;
            section_keys.finalise_dkg(&public_key);
            let key_share = section_keys.key_share()?;
            self.send_proposal_with(&recipients, proposal, key_share)
                .await
        } else {
            // sign using pending key internally in section_keys
            unimplemented!()
        }
    }

    pub(crate) async fn handle_dkg_failure(
        &self,
        failure_set: DkgFailureSigSet,
    ) -> Result<Command> {
        let node_msg = NodeMsg::DkgFailureAgreement(failure_set);
        self.send_message_to_our_elders(node_msg).await
    }
}
