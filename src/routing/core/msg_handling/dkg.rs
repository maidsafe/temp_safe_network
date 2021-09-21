// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::super::Core;
use crate::messaging::{
    system::{DkgFailureSig, DkgFailureSigSet, DkgKey, ElderCandidates, Proposal, SystemMsg},
    SectionAuthorityProvider,
};
use crate::routing::{
    dkg::DkgFailureSigSetUtils,
    error::{Error, Result},
    routing_api::command::Command,
    section::{SectionKeyShare, SectionPeersUtils},
    SectionAuthorityProviderUtils,
};
use bls_dkg::key_gen::message::Message as DkgMessage;
use std::collections::BTreeSet;
use xor_name::XorName;

impl Core {
    pub(crate) fn handle_dkg_start(
        &mut self,
        dkg_key: DkgKey,
        elder_candidates: ElderCandidates,
    ) -> Result<Vec<Command>> {
        trace!("Received DkgStart for {:?}", elder_candidates);
        self.dkg_voter.start(
            &self.node,
            dkg_key,
            elder_candidates,
            *self.section_chain().last_key(),
        )
    }

    pub(crate) fn handle_dkg_message(
        &mut self,
        dkg_key: DkgKey,
        message: DkgMessage,
        sender: XorName,
    ) -> Result<Vec<Command>> {
        trace!("handle DKG message {:?} from {}", message, sender);

        self.dkg_voter.process_message(
            &self.node,
            &dkg_key,
            message,
            *self.section_chain().last_key(),
        )
    }

    pub(crate) fn handle_dkg_failure_observation(
        &mut self,
        dkg_key: DkgKey,
        failed_participants: &BTreeSet<XorName>,
        signed: DkgFailureSig,
    ) -> Result<Vec<Command>> {
        match self
            .dkg_voter
            .process_failure(&dkg_key, failed_participants, signed)
        {
            None => Ok(vec![]),
            Some(cmd) => Ok(vec![cmd]),
        }
    }

    pub(crate) fn handle_dkg_failure_agreement(
        &mut self,
        sender: &XorName,
        failure_set: &DkgFailureSigSet,
    ) -> Result<Vec<Command>> {
        if self.section.members().get(sender).is_none() {
            return Err(Error::InvalidSrcLocation);
        }

        let generation = self.section.chain().main_branch_len() as u64;

        let elder_candidates = if let Some(elder_candidates) = self
            .section
            .promote_and_demote_elders(&self.node.name())
            .into_iter()
            .find(|elder_candidates| failure_set.verify(elder_candidates, generation))
        {
            elder_candidates
        } else {
            trace!("Ignore DKG failure agreement with invalid signeds or outdated participants",);
            return Ok(vec![]);
        };

        let mut commands = vec![];

        if !failure_set.failed_participants.is_empty() {
            // The DKG failure is regarding failed_participants, i.e. potential unresponsive node.
            trace!(
                "Received DKG failure agreement, propose offline for failed participants: {:?} , DKG generation({}), candidates: {:?}",
                failure_set.failed_participants,
                generation, elder_candidates
            );
            commands.extend(self.cast_offline_proposals(&failure_set.failed_participants)?);
        }

        trace!(
            "==========Received DKG failure agreement, we will restart with candidates: {:?} except failed participants: {:?}",
            elder_candidates, failure_set.failed_participants
        );

        commands.extend(self.promote_and_demote_elders_except(&failure_set.failed_participants)?);
        Ok(commands)
    }

    pub(crate) fn handle_dkg_outcome(
        &mut self,
        section_auth: SectionAuthorityProvider,
        key_share: SectionKeyShare,
    ) -> Result<Vec<Command>> {
        let proposal = Proposal::SectionInfo(section_auth);
        let recipients: Vec<_> = self.section.authority_provider().peers().collect();
        let result = self.send_proposal_with(&recipients, proposal, &key_share);

        let public_key = key_share.public_key_set.public_key();

        self.section_keys_provider.insert_dkg_outcome(key_share);

        if self.section.chain().has_key(&public_key) {
            self.section_keys_provider.finalise_dkg(&public_key)
        }

        result
    }

    pub(crate) fn handle_dkg_failure(&mut self, failure_set: DkgFailureSigSet) -> Result<Command> {
        let node_msg = SystemMsg::DkgFailureAgreement(failure_set);
        self.send_message_to_our_elders(node_msg)
    }
}
