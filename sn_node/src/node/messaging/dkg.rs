// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    flow_ctrl::cmds::Cmd,
    messaging::{OutgoingMsg, Peers},
    Error, Node, Proposal, Result,
};

use bytes::Bytes;
#[cfg(feature = "traceroute")]
use sn_interface::messaging::Traceroute;
use sn_interface::{
    messaging::{
        system::{DkgFailureSig, DkgFailureSigSet, DkgSessionId, SigShare, SystemMsg},
        AuthorityProof, BlsShareAuth, NodeMsgAuthority, WireMsg,
    },
    network_knowledge::{SectionAuthorityProvider, SectionKeyShare},
    types::{log_markers::LogMarker, Peer},
};

use bls_dkg::key_gen::message::Message as DkgMessage;
use std::collections::BTreeSet;
use xor_name::XorName;

impl Node {
    /// Send a `DkgStart` message to the provided set of candidates
    pub(crate) fn send_dkg_start(&self, session_id: DkgSessionId) -> Result<Vec<Cmd>> {
        // Send DKG start to all candidates
        let recipients = Vec::from_iter(session_id.elder_peers());

        trace!(
            "{} for {:?} with {:?} to {:?}",
            LogMarker::SendDkgStart,
            session_id.elders,
            session_id,
            recipients
        );

        let mut handle = true;
        let mut cmds = vec![];
        let mut others = BTreeSet::new();

        // remove ourself from recipients
        let our_name = self.info().name();
        for recipient in recipients {
            if recipient.name() == our_name {
                handle = true;
            } else {
                let _ = others.insert(recipient);
            }
        }

        let src_name = session_id.prefix.name();
        let msg = SystemMsg::DkgStart(session_id);
        let (auth, payload) = self.get_auth(&msg, src_name)?;

        if !others.is_empty() {
            cmds.push(Cmd::send_msg(
                OutgoingMsg::DstAggregated((auth.clone(), payload.clone())),
                Peers::Multiple(others),
            ));
        }

        if handle {
            cmds.push(Cmd::HandleValidSystemMsg {
                origin: Peer::new(our_name, self.addr),
                msg_id: sn_interface::messaging::MsgId::new(),
                msg,
                msg_authority: NodeMsgAuthority::BlsShare(AuthorityProof(auth)),
                wire_msg_payload: payload,
                #[cfg(feature = "traceroute")]
                traceroute: Traceroute(vec![]),
            });
        }

        Ok(cmds)
    }

    fn get_auth(&self, msg: &SystemMsg, src_name: XorName) -> Result<(BlsShareAuth, Bytes)> {
        let section_key = self.network_knowledge.section_key();
        let key_share = self
            .section_keys_provider
            .key_share(&section_key)
            .map_err(|err| {
                trace!("Can't create message {:?} for accumulation: {:?}", msg, err);
                err
            })?;

        let payload = WireMsg::serialize_msg_payload(&msg).map_err(|_| Error::InvalidMessage)?;

        let auth = BlsShareAuth {
            section_pk: section_key,
            src_name,
            sig_share: SigShare {
                public_key_set: key_share.public_key_set.clone(),
                index: key_share.index,
                signature_share: key_share.secret_key_share.sign(&payload),
            },
        };

        Ok((auth, payload))
    }

    pub(crate) fn handle_dkg_start(&mut self, session_id: DkgSessionId) -> Result<Vec<Cmd>> {
        let current_generation = self.network_knowledge.section_chain_len();
        if session_id.section_chain_len < current_generation {
            trace!("Skipping DkgStart for older generation: {:?}", &session_id);
            return Ok(vec![]);
        }
        let section_auth = self.network_knowledge().section_auth();

        let mut peers = vec![];
        for session_peer in session_id.elder_peers() {
            // Reuse known peers from network_knowledge, in order to preserve connections
            let peer = if let Some(elder) = section_auth
                .get_elder(&session_peer.name())
                .filter(|elder| elder.addr() == session_peer.addr())
            {
                *elder
            } else if let Some(peer) = self
                .network_knowledge()
                .find_member_by_addr(&session_peer.addr())
            {
                peer
            } else {
                session_peer
            };

            peers.push(peer);
        }

        trace!("Received DkgStart for {:?}", session_id);
        self.dkg_sessions.retain(|_, existing_session_info| {
            existing_session_info.session_id.section_chain_len >= session_id.section_chain_len
        });
        let cmds = self.dkg_voter.start(
            &self.info(),
            session_id,
            self.network_knowledge().section_key(),
        )?;
        Ok(cmds)
    }

    pub(crate) fn handle_dkg_msg(
        &self,
        session_id: DkgSessionId,
        message: DkgMessage,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        trace!(
            "{} {:?} from {}",
            LogMarker::DkgMessageHandling,
            message,
            sender
        );

        if session_id.prefix.bit_count() < self.network_knowledge.prefix().bit_count() {
            return Err(Error::InvalidDkgPrefix);
        }

        self.dkg_voter.process_msg(
            sender,
            &self.info(),
            &session_id,
            message,
            self.network_knowledge().section_key(),
        )
    }

    pub(crate) fn handle_dkg_not_ready(
        &self,
        sender: Peer,
        message: DkgMessage,
        session_id: DkgSessionId,
    ) -> Cmd {
        let msg = SystemMsg::DkgRetry {
            message_history: self.dkg_voter.get_cached_msgs(&session_id),
            message,
            session_id,
        };
        self.send_system_msg(msg, Peers::Single(sender))
    }

    pub(crate) fn handle_dkg_retry(
        &self,
        session_id: &DkgSessionId,
        message_history: Vec<DkgMessage>,
        message: DkgMessage,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        let section_key = self.network_knowledge().section_key();
        let current_generation = self.network_knowledge.section_chain_len();
        if session_id.section_chain_len < current_generation {
            trace!(
                "Ignoring DkgRetry for expired DKG session: {:?}",
                &session_id
            );
            return Ok(vec![]);
        }
        let mut cmds = self.dkg_voter.handle_dkg_history(
            &self.info(),
            session_id,
            message_history,
            sender.name(),
            section_key,
        )?;

        cmds.extend(self.dkg_voter.process_msg(
            sender,
            &self.info(),
            session_id,
            message,
            section_key,
        )?);
        Ok(cmds)
    }

    pub(crate) fn handle_dkg_failure_observation(
        &self,
        session_id: DkgSessionId,
        failed_participants: &BTreeSet<XorName>,
        signed: DkgFailureSig,
    ) -> Result<Vec<Cmd>> {
        match self
            .dkg_voter
            .process_failure(&session_id, failed_participants, signed)
        {
            None => Ok(vec![]),
            Some(cmd) => Ok(vec![cmd]),
        }
    }

    pub(crate) fn handle_dkg_failure_agreement(
        &mut self,
        sender: &XorName,
        failure_set: &DkgFailureSigSet,
    ) -> Result<Vec<Cmd>> {
        if !self.network_knowledge.is_section_member(sender) {
            return Err(Error::InvalidDkgParticipant);
        }

        let generation = self.network_knowledge.section_chain_len();

        let dkg_session = if let Some(dkg_session) = self
            .promote_and_demote_elders(&BTreeSet::new())
            .into_iter()
            .find(|session_id| failure_set.verify(session_id))
        {
            dkg_session
        } else {
            trace!("Ignore DKG failure agreement with invalid signeds or outdated participants",);
            return Ok(vec![]);
        };

        let mut cmds = vec![];

        if !failure_set.failed_participants.is_empty() {
            // The DKG failure is regarding failed_participants, i.e. potential unresponsive node.
            trace!(
                "Received DKG failure agreement, propose offline for failed participants: {:?} , DKG generation({}), candidates: {:?}",
                failure_set.failed_participants,
                generation, dkg_session
            );
            cmds.extend(self.cast_offline_proposals(&failure_set.failed_participants)?);
        }

        trace!(
            "Received DKG failure agreement, we will restart with candidates: {:?} except failed participants: {:?}",
            dkg_session, failure_set.failed_participants
        );

        cmds.extend(self.promote_and_demote_elders_except(&failure_set.failed_participants)?);
        Ok(cmds)
    }

    pub(crate) async fn handle_dkg_outcome(
        &mut self,
        sap: SectionAuthorityProvider,
        key_share: SectionKeyShare,
    ) -> Result<Vec<Cmd>> {
        let key_share_pk = key_share.public_key_set.public_key();
        trace!(
            "{} public_key={:?}",
            LogMarker::HandlingDkgSuccessfulOutcome,
            key_share_pk
        );

        // Add our new keyshare to our cache, we will then use
        // it to sign any msg that needs section agreement.
        self.section_keys_provider.insert(key_share.clone());

        let snapshot = self.state_snapshot();

        // If we are lagging, we may have been already approved as new Elder, and
        // an AE update provided us with this same SAP but already signed by previous Elders,
        // if so we can skip the SectionInfo agreement proposal phase.
        if self
            .network_knowledge
            .try_update_current_sap(key_share_pk, &sap.prefix())
        {
            self.update_on_elder_change(&snapshot).await
        } else {
            // This proposal is sent to the current set of elders to be aggregated
            // and section signed.
            let proposal = Proposal::SectionInfo(sap);
            let recipients: Vec<_> = self.network_knowledge.section_auth().elders_vec();
            self.send_proposal_with(recipients, proposal, &key_share)
        }
    }

    pub(crate) fn handle_dkg_failure(&mut self, failure_set: DkgFailureSigSet) -> Cmd {
        // track those failed participants
        for name in &failure_set.failed_participants {
            trace!("Logging {name} as having Dkg issue in dysfunction");
            self.log_dkg_issue(*name);
        }
        self.send_msg_to_our_elders(SystemMsg::DkgFailureAgreement(failure_set))
    }
}
