// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::super::Core;
use crate::messaging::{
    node::{DkgFailureSig, DkgFailureSigSet, DkgKey, ElderCandidates, NodeMsg, Proposal, Variant},
    DstLocation, SectionAuthorityProvider,
};
use crate::routing::{
    dkg::{commands::DkgCommands, DkgFailureSigSetUtils},
    error::{Error, Result},
    messages::WireMsgUtils,
    routing_api::command::Command,
    section::{SectionAuthorityProviderUtils, SectionKeyShare, SectionPeersUtils, SectionUtils},
};
use bls_dkg::key_gen::message::Message as DkgMessage;
use std::{collections::BTreeSet, slice};
use xor_name::XorName;

impl Core {
    pub(crate) fn handle_dkg_start(
        &mut self,
        dkg_key: DkgKey,
        elder_candidates: ElderCandidates,
    ) -> Result<Vec<Command>> {
        trace!("Received DkgStart for {:?}", elder_candidates);
        self.dkg_voter
            .start(&self.node.keypair, dkg_key, elder_candidates)
            .into_commands(&self.node, *self.section_chain().last_key())
    }

    pub(crate) fn handle_dkg_message(
        &mut self,
        dkg_key: DkgKey,
        message: DkgMessage,
        sender: XorName,
    ) -> Result<Vec<Command>> {
        trace!("handle DKG message {:?} from {}", message, sender);

        self.dkg_voter
            .process_message(&self.node.keypair, &dkg_key, message)
            .into_commands(&self.node, *self.section_chain().last_key())
    }

    pub(crate) fn handle_dkg_failure_observation(
        &mut self,
        dkg_key: DkgKey,
        failed_participants: &BTreeSet<XorName>,
        signed: DkgFailureSig,
    ) -> Result<Vec<Command>> {
        self.dkg_voter
            .process_failure(&dkg_key, failed_participants, signed)
            .into_commands(&self.node, *self.section_chain().last_key())
    }

    pub(crate) fn handle_dkg_failure_agreement(
        &self,
        sender: &XorName,
        failure_set: &DkgFailureSigSet,
    ) -> Result<Vec<Command>> {
        let sender = &self
            .section
            .members()
            .get(sender)
            .ok_or(Error::InvalidSrcLocation)?
            .peer;

        let generation = self.section.chain().main_branch_len() as u64;
        let elder_candidates = self
            .section
            .promote_and_demote_elders(&self.node.name())
            .into_iter()
            .find(|elder_candidates| failure_set.verify(elder_candidates, generation));
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
        } else {
            // The DKG failure is regarding failed_participants, i.e. potential unresponsive node.
            trace!(
                "Received DKG failure agreement of failed_participants {:?} , DKG generation({}) {:?}",
                failure_set.failed_participants,
                generation,
                elder_candidates
            );
            self.cast_offline_proposals(&failure_set.failed_participants)
        }
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
        unimplemented!();
        /*
                let variant = Variant::DkgFailureAgreement(failure_set);
        let message = NodeMsg::single_src(
            &self.node,
            DstLocation::DirectAndUnrouted,
            variant,
            self.section.authority_provider().section_key(),
        )?;
        Ok(self.send_message_to_our_elders(message))*/
    }
}
