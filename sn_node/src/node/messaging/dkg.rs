// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    core::DkgSessionInfo,
    dkg::{check_ephemeral_dkg_key, DkgPubKeys},
    flow_ctrl::cmds::Cmd,
    messaging::Peers,
    Error, MyNode, Proposal, Result,
};

use sn_interface::{
    messaging::{
        system::{DkgSessionId, NodeMsg, SectionSigShare},
        AuthorityProof, SectionSig,
    },
    network_knowledge::{SectionAuthorityProvider, SectionKeyShare},
    types::{self, log_markers::LogMarker, Peer},
};

use bls::{PublicKey as BlsPublicKey, PublicKeySet, SecretKeyShare};
use ed25519::Signature;
use sn_sdkg::{DkgSignedVote, VoteResponse};
use std::collections::BTreeSet;
use xor_name::XorName;

/// Helper to get our DKG peers (excluding us)
fn dkg_peers(our_index: usize, session_id: &DkgSessionId) -> BTreeSet<Peer> {
    session_id
        .elder_peers()
        .enumerate()
        .filter_map(|(index, peer)| (index != our_index).then_some(peer))
        .collect()
}

fn acknowledge_dkg_oucome(
    session_id: &DkgSessionId,
    participant_index: usize,
    pub_key_set: PublicKeySet,
    sec_key_share: SecretKeyShare,
) -> Cmd {
    let section_auth = SectionAuthorityProvider::from_dkg_session(session_id, pub_key_set.clone());
    let outcome = SectionKeyShare {
        public_key_set: pub_key_set,
        index: participant_index,
        secret_key_share: sec_key_share,
    };

    Cmd::HandleDkgOutcome {
        section_auth,
        outcome,
    }
}

impl MyNode {
    /// Send a `DkgStart` message to the provided set of candidates
    /// Before a DKG session kicks off, the `DkgStart { ... }` message is individually signed
    /// by the current _set of elders_ and sent to the _new elder candidates_ to be accumulated.
    /// This is to prevent nodes from spamming `DkgStart` messages which might lead to unnecessary
    /// DKG sessions.
    /// Whenever there is a change in Elders in the network Distributed Key Generation
    /// is used to generate a new set of BLS key shares for individual Elders along with the
    /// SectionKey which will represent the section.
    /// DKG is triggered by the following events:
    /// - A change in the Elders
    /// - Section Split
    pub(crate) fn send_dkg_start(&mut self, session_id: DkgSessionId) -> Result<Vec<Cmd>> {
        // Send DKG start to all candidates
        let recipients = Vec::from_iter(session_id.elder_peers());

        trace!(
            "{} s{} {:?} with {:?} to {:?}",
            LogMarker::SendDkgStart,
            session_id.sh(),
            session_id.prefix,
            session_id.elders,
            recipients
        );

        let mut we_are_a_participant = false;
        let mut cmds = vec![];
        let mut others = BTreeSet::new();

        // remove ourself from recipients
        let our_name = self.info().name();
        for recipient in recipients {
            if recipient.name() == our_name {
                we_are_a_participant = true;
            } else {
                let _ = others.insert(recipient);
            }
        }

        // sign the DkgSessionId
        let section_sig_share = self.sign_session_id(&session_id)?;
        let node_msg = NodeMsg::DkgStart(session_id.clone(), section_sig_share.clone());

        // send it to the other participants
        if !others.is_empty() {
            cmds.push(self.send_system_msg(node_msg, Peers::Multiple(others)))
        }

        // handle our own
        if we_are_a_participant {
            cmds.extend(self.handle_dkg_start(session_id, section_sig_share)?);
        }

        Ok(cmds)
    }

    fn sign_session_id(&self, session_id: &DkgSessionId) -> Result<SectionSigShare> {
        // get section key
        let section_key = self.network_knowledge.section_key();
        let key_share = self
            .section_keys_provider
            .key_share(&section_key)
            .map_err(|err| {
                warn!(
                    "Can't obtain key share to sign DkgSessionId s{} {:?}",
                    session_id.sh(),
                    err
                );
                err
            })?;

        // sign
        let serialized_session_id = bincode::serialize(session_id)?;
        Ok(SectionSigShare {
            public_key_set: key_share.public_key_set.clone(),
            index: key_share.index,
            signature_share: key_share.secret_key_share.sign(&serialized_session_id),
        })
    }

    fn broadcast_dkg_votes(
        &self,
        session_id: &DkgSessionId,
        pub_keys: DkgPubKeys,
        participant_index: usize,
        votes: Vec<DkgSignedVote>,
    ) -> Cmd {
        let recipients = dkg_peers(participant_index, session_id);
        trace!(
            "{} s{}: {:?}",
            LogMarker::DkgBroadcastVote,
            session_id.sh(),
            votes
        );
        let node_msg = NodeMsg::DkgVotes {
            session_id: session_id.clone(),
            pub_keys,
            votes,
        };
        self.send_system_msg(node_msg, Peers::Multiple(recipients))
    }

    fn request_dkg_ae(&self, session_id: &DkgSessionId, sender: Peer) -> Cmd {
        let node_msg = NodeMsg::DkgAE(session_id.clone());
        self.send_system_msg(node_msg, Peers::Single(sender))
    }

    fn aggregate_dkg_start(
        &mut self,
        session_id: &DkgSessionId,
        elder_sig: SectionSigShare,
    ) -> Result<Option<SectionSig>> {
        // check sig share
        let public_key = elder_sig.public_key_set.public_key();
        if self.network_knowledge.section_key() != public_key {
            return Err(Error::InvalidKeyShareSectionKey);
        }
        let serialized_session_id = bincode::serialize(session_id)?;

        // try aggregate
        self.dkg_start_aggregator
            .try_aggregate(&serialized_session_id, elder_sig)
            .map_err(|err| {
                warn!(
                    "Error aggregating signature in DkgStart s{}: {err:?}",
                    session_id.sh()
                );
                Error::InvalidSignatureShare
            })
    }

    pub(crate) fn handle_dkg_start(
        &mut self,
        session_id: DkgSessionId,
        elder_sig: SectionSigShare,
    ) -> Result<Vec<Cmd>> {
        // try to create a section sig by aggregating the elder_sig
        match self.aggregate_dkg_start(&session_id, elder_sig) {
            Ok(Some(section_sig)) => {
                trace!(
                    "DkgStart: section key aggregated, starting session s{}",
                    session_id.sh()
                );
                self.dkg_start(session_id, section_sig)
            }
            Ok(None) => {
                trace!(
                    "DkgStart: waiting for more shares for session s{}",
                    session_id.sh()
                );
                Ok(vec![])
            }
            Err(e) => {
                warn!(
                    "DkgStart: failed to aggregate received elder sig in s{}: {e:?}",
                    session_id.sh()
                );
                Ok(vec![])
            }
        }
    }

    pub(crate) fn dkg_start(
        &mut self,
        session_id: DkgSessionId,
        section_sig: SectionSig,
    ) -> Result<Vec<Cmd>> {
        // make sure we are in this dkg session
        let our_name = types::keys::ed25519::name(&self.keypair.public);
        let our_id = if let Some(index) = session_id.elder_index(our_name) {
            index
        } else {
            error!(
                "DKG failed to start s{}: {our_name} is not a participant",
                session_id.sh()
            );
            return Ok(vec![]);
        };

        // ignore DkgStart from old chains
        let current_chain_len = self.network_knowledge.section_chain_len();
        if session_id.section_chain_len < current_chain_len {
            trace!("Skipping DkgStart for older chain: s{}", session_id.sh());
            return Ok(vec![]);
        }

        // acknowledge Dkg session
        let session_info = DkgSessionInfo {
            session_id: session_id.clone(),
            authority: AuthorityProof(section_sig),
        };
        let section_auth = session_info.authority.clone();
        let _existing = self
            .dkg_sessions_info
            .insert(session_id.hash(), session_info);

        // gen key
        let (ephemeral_pub_key, sig) =
            match self
                .dkg_voter
                .gen_ephemeral_key(session_id.hash(), our_name, &self.keypair)
            {
                Ok(k) => k,
                Err(Error::DkgEphemeralKeyAlreadyGenerated) => {
                    trace!(
                        "Skipping already acknowledged DkgStart s{}",
                        session_id.sh()
                    );
                    return Ok(vec![]);
                }
                Err(e) => return Err(e),
            };

        // assert people can check key
        assert!(check_ephemeral_dkg_key(&session_id, our_name, ephemeral_pub_key, sig).is_ok());

        // broadcast signed pub key
        trace!(
            "{} s{} from {our_id:?}",
            LogMarker::DkgBroadcastEphemeralPubKey,
            session_id.sh(),
        );
        let peers = dkg_peers(our_id, &session_id);
        let node_msg = NodeMsg::DkgEphemeralPubKey {
            session_id,
            section_auth,
            pub_key: ephemeral_pub_key,
            sig,
        };

        let cmd = self.send_system_msg(node_msg, Peers::Multiple(peers));
        Ok(vec![cmd])
    }

    fn handle_missed_dkg_start(
        &mut self,
        session_id: &DkgSessionId,
        section_auth: AuthorityProof<SectionSig>,
        pub_key: BlsPublicKey,
        sig: Signature,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        trace!(
            "Detected missed dkg start for s{:?} after msg from {sender:?}",
            session_id.sh()
        );

        // check the signature
        let serialized_session_id = bincode::serialize(session_id)?;
        let section_sig = section_auth.clone().into_inner();
        if self.network_knowledge.section_key() != section_sig.public_key {
            warn!(
                "Invalid section key in dkg auth proof in s{:?}: {sender:?}",
                session_id.sh()
            );
            return Ok(vec![]);
        }
        if let Err(err) = AuthorityProof::verify(section_sig.clone(), serialized_session_id) {
            error!(
                "Invalid signature in dkg auth proof in s{:?}: {err:?}",
                session_id.sh()
            );
            return Ok(vec![]);
        }

        // catch back up
        info!(
            "Handling missed dkg start for s{:?} after msg from {sender:?}",
            session_id.sh()
        );
        let mut cmds = vec![];
        cmds.extend(self.dkg_start(session_id.clone(), section_sig)?);
        cmds.extend(self.handle_dkg_ephemeral_pubkey(
            session_id,
            section_auth,
            pub_key,
            sig,
            sender,
        )?);
        Ok(cmds)
    }

    pub(crate) fn handle_dkg_ephemeral_pubkey(
        &mut self,
        session_id: &DkgSessionId,
        section_auth: AuthorityProof<SectionSig>,
        pub_key: BlsPublicKey,
        sig: Signature,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        // make sure we are in this dkg session
        let name = types::keys::ed25519::name(&self.keypair.public);
        let our_id = if let Some(index) = session_id.elder_index(name) {
            index
        } else {
            error!(
                "DKG ephemeral key ignored for s{}: {name} is not a participant",
                session_id.sh()
            );
            return Ok(vec![]);
        };

        // try to start DKG if we've got all the keys
        let outcome =
            match self
                .dkg_voter
                .try_init_dkg(session_id, our_id, pub_key, sig, sender.name())
            {
                Ok(o) => o,
                Err(Error::NoDkgKeysForSession(_)) => {
                    return self.handle_missed_dkg_start(
                        session_id,
                        section_auth,
                        pub_key,
                        sig,
                        sender,
                    );
                }
                Err(e) => {
                    error!(
                        "Failed to init DKG s{} id:{our_id:?}: {:?}",
                        session_id.sh(),
                        e
                    );
                    return Ok(vec![]);
                }
            };
        let (vote, pub_keys) = if let Some(start) = outcome {
            start
        } else {
            // we don't have all the keys yet
            return Ok(vec![]);
        };

        // send first vote
        trace!(
            "{} s{} from id:{our_id:?}",
            LogMarker::DkgBroadcastFirstVote,
            session_id.sh()
        );
        let cmd = self.broadcast_dkg_votes(session_id, pub_keys, our_id, vec![vote]);
        Ok(vec![cmd])
    }

    fn handle_vote_response(
        &mut self,
        session_id: &DkgSessionId,
        pub_keys: DkgPubKeys,
        sender: Peer,
        our_id: usize,
        vote_response: VoteResponse,
    ) -> (Vec<Cmd>, Vec<Cmd>) {
        let mut cmds = vec![];
        let mut ae_cmds = vec![];
        match vote_response {
            VoteResponse::WaitingForMoreVotes => {}
            VoteResponse::RequestAntiEntropy => {
                ae_cmds.push(self.request_dkg_ae(session_id, sender))
            }
            VoteResponse::BroadcastVote(vote) => {
                cmds.push(self.broadcast_dkg_votes(session_id, pub_keys, our_id, vec![*vote]))
            }
            VoteResponse::DkgComplete(new_pubs, new_sec) => {
                trace!(
                    "{} s{:?} {:?}: {} elders: {:?}",
                    LogMarker::DkgComplete,
                    session_id.sh(),
                    session_id.prefix,
                    session_id.elders.len(),
                    new_pubs.public_key(),
                );
                cmds.push(acknowledge_dkg_oucome(
                    session_id, our_id, new_pubs, new_sec,
                ))
            }
        }
        (cmds, ae_cmds)
    }

    pub(crate) fn handle_dkg_votes(
        &mut self,
        session_id: &DkgSessionId,
        msg_keys: DkgPubKeys,
        votes: Vec<DkgSignedVote>,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        // make sure we are in this dkg session
        let name = types::keys::ed25519::name(&self.keypair.public);
        let our_id = if let Some(index) = session_id.elder_index(name) {
            index
        } else {
            error!(
                "DKG failed to handle vote in s{}: {name} is not a participant",
                session_id.sh()
            );
            return Ok(vec![]);
        };

        // make sure the keys are valid
        let (pub_keys, just_completed) = self.dkg_voter.check_keys(session_id, msg_keys)?;

        // if we just completed our keyset thanks to the incoming keys, bcast 1st vote
        let mut cmds = Vec::new();
        if just_completed {
            let (first_vote, _) = self.dkg_voter.initialize_dkg_state(session_id, our_id)?;
            cmds.push(self.broadcast_dkg_votes(
                session_id,
                pub_keys.clone(),
                our_id,
                vec![first_vote],
            ));
        }

        // handle vote
        let mut cmds: Vec<Cmd> = Vec::new();
        let mut ae_cmds: Vec<Cmd> = Vec::new();
        let mut is_old_gossip = true;
        let their_votes_len = votes.len();
        for v in votes {
            match self.dkg_voter.handle_dkg_vote(session_id, v.clone()) {
                Ok(vote_responses) => {
                    debug!(
                        "Dkg s{}: {:?} after: {v:?}",
                        session_id.sh(),
                        vote_responses,
                    );
                    if !vote_responses.is_empty() {
                        self.dkg_voter.learned_something_from_message();
                        is_old_gossip = false;
                    }
                    for r in vote_responses {
                        let (cmd, ae_cmd) = self.handle_vote_response(
                            session_id,
                            pub_keys.clone(),
                            sender,
                            our_id,
                            r,
                        );
                        cmds.extend(cmd);
                        ae_cmds.extend(ae_cmd);
                    }
                }
                Err(err) => {
                    error!(
                        "Error processing DKG vote s{} id:{our_id:?} {v:?} from {sender:?}: {err:?}",
                        session_id.sh()
                    );
                }
            }
        }

        // ae is not necessary if we have votes or termination cmds
        if cmds.is_empty() {
            cmds.append(&mut ae_cmds);
        }

        // if their un-interesting gossip is missing votes, send them ours
        if is_old_gossip && their_votes_len != 1 {
            let mut manual_ae = match self.gossip_missing_votes(session_id, sender, their_votes_len)
            {
                Ok(g) => g,
                Err(err) => {
                    error!(
                        "Error gossiping s{} id:{our_id:?} from {sender:?}: {err:?}",
                        session_id.sh()
                    );
                    vec![]
                }
            };
            cmds.append(&mut manual_ae);
        }

        Ok(cmds)
    }

    /// Gossips all our votes if they have less votes than us
    /// Assumes we know all their votes so the length difference is enough to know that they
    /// are missing votes
    fn gossip_missing_votes(
        &self,
        session_id: &DkgSessionId,
        sender: Peer,
        their_votes_len: usize,
    ) -> Result<Vec<Cmd>> {
        let our_votes = self.dkg_voter.get_all_votes(session_id)?;
        if their_votes_len < our_votes.len() {
            let pub_keys = self.dkg_voter.get_dkg_keys(session_id)?;
            trace!(
                "{} s{}: gossip including missing votes to {sender:?}",
                LogMarker::DkgBroadcastVote,
                session_id.sh()
            );
            let node_msg = NodeMsg::DkgVotes {
                session_id: session_id.clone(),
                pub_keys,
                votes: our_votes,
            };
            let cmd = self.send_system_msg(node_msg, Peers::Single(sender));
            Ok(vec![cmd])
        } else {
            Ok(vec![])
        }
    }

    pub(crate) fn handle_dkg_anti_entropy(
        &self,
        session_id: DkgSessionId,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        let pub_keys = self.dkg_voter.get_dkg_keys(&session_id)?;
        let votes = self.dkg_voter.get_all_votes(&session_id)?;
        trace!(
            "{} s{}: AE to {sender:?}",
            LogMarker::DkgBroadcastVote,
            session_id.sh()
        );
        let node_msg = NodeMsg::DkgVotes {
            session_id,
            pub_keys,
            votes,
        };
        let cmd = self.send_system_msg(node_msg, Peers::Single(sender));
        Ok(vec![cmd])
    }

    // broadcasts our current known votes
    fn gossip_votes(
        &self,
        session_id: DkgSessionId,
        votes: Vec<DkgSignedVote>,
        our_id: usize,
    ) -> Vec<Cmd> {
        let pub_keys = match self.dkg_voter.get_dkg_keys(&session_id) {
            Ok(k) => k,
            Err(_) => {
                warn!(
                    "Unexpectedly missing dkg keys when gossiping s{}",
                    session_id.sh()
                );
                return vec![];
            }
        };
        trace!(
            "{} s{}: gossiping votes",
            LogMarker::DkgBroadcastVote,
            session_id.sh()
        );
        let cmd = self.broadcast_dkg_votes(&session_id, pub_keys, our_id, votes);
        vec![cmd]
    }

    // broadcasts our ephemeral key
    fn gossip_our_key(
        &self,
        session_id: DkgSessionId,
        our_name: XorName,
        our_id: usize,
    ) -> Vec<Cmd> {
        // get the keys
        let dkg_keys = match self.dkg_voter.get_dkg_keys(&session_id) {
            Ok(k) => k,
            Err(_) => {
                warn!(
                    "Unexpectedly missing dkg pub keys when gossiping s{}",
                    session_id.sh()
                );
                return vec![];
            }
        };
        let (pub_key, sig) = match dkg_keys.get(&our_name) {
            Some(res) => res,
            None => {
                warn!(
                    "Unexpectedly missing our dkg key when gossiping s{}",
                    session_id.sh()
                );
                return vec![];
            }
        };

        // get original auth (as proof for those who missed the original DkgStart msg)
        let section_info = match self.dkg_sessions_info.get(&session_id.hash()) {
            Some(auth) => auth,
            None => {
                warn!(
                    "Unexpectedly missing dkg section info when gossiping s{}",
                    session_id.sh()
                );
                return vec![];
            }
        };
        let section_auth = section_info.authority.clone();

        trace!(
            "{} s{}: gossiping ephemeral key",
            LogMarker::DkgBroadcastVote,
            session_id.sh()
        );

        // broadcast our key
        let peers = dkg_peers(our_id, &session_id);
        let node_msg = NodeMsg::DkgEphemeralPubKey {
            session_id,
            section_auth,
            pub_key: *pub_key,
            sig: *sig,
        };
        let cmd = self.send_system_msg(node_msg, Peers::Multiple(peers));
        vec![cmd]
    }

    pub(crate) fn had_sap_change_since(&self, session_id: &DkgSessionId) -> bool {
        self.network_knowledge.section_chain_len() != session_id.section_chain_len
    }

    pub(crate) fn gossip_handover_trigger(&self, session_id: &DkgSessionId) -> Vec<Cmd> {
        match self.dkg_voter.outcome(session_id) {
            Ok(Some((our_id, new_pubs, new_sec))) => {
                trace!(
                    "Gossiping DKG outcome for s{} as we didn't notice SAP change",
                    session_id.sh()
                );
                let cmd = acknowledge_dkg_oucome(session_id, our_id.into(), new_pubs, new_sec);
                vec![cmd]
            }
            Ok(None) => {
                error!(
                    "Missing DKG outcome for s{}, when trying to gossip outcome",
                    session_id.sh()
                );
                vec![]
            }
            Err(e) => {
                error!(
                    "Failed to get DKG outcome for s{}, when trying to gossip outcome: {}",
                    session_id.sh(),
                    e
                );
                vec![]
            }
        }
    }

    /// For all the ongoing DKG sessions, sends out all the current known votes to all DKG
    /// participants if we don't have any votes yet, sends out our ephemeral key
    pub(crate) fn dkg_gossip_msgs(&self) -> Vec<Cmd> {
        let mut cmds = vec![];
        for (_hash, session_info) in self.dkg_sessions_info.iter() {
            // get our id
            let name = types::keys::ed25519::name(&self.keypair.public);
            let our_id = if let Some(index) = session_info.session_id.elder_index(name) {
                index
            } else {
                error!(
                    "DKG failed gossip in s{}: {name} is not a participant",
                    session_info.session_id.sh()
                );
                continue;
            };

            // skip if we already reached termination
            match self.dkg_voter.reached_termination(&session_info.session_id) {
                Ok(true) => {
                    trace!(
                        "Skipping DKG gossip for s{} as we have reached termination",
                        session_info.session_id.sh()
                    );

                    if !self.had_sap_change_since(&session_info.session_id) {
                        cmds.extend(self.gossip_handover_trigger(&session_info.session_id));
                    }

                    continue;
                }
                Ok(false) => {}
                Err(err) => {
                    error!(
                        "DKG failed gossip in s{}: {:?}",
                        session_info.session_id.sh(),
                        err
                    );
                }
            }

            // gossip votes else gossip our key
            if let Ok(votes) = self.dkg_voter.get_all_votes(&session_info.session_id) {
                cmds.extend(self.gossip_votes(session_info.session_id.clone(), votes, our_id));
            } else {
                cmds.extend(self.gossip_our_key(session_info.session_id.clone(), name, our_id));
            }
        }
        cmds
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
}

#[cfg(test)]
mod tests {
    use super::MyNode;
    use crate::{
        comm::Comm,
        node::{
            cfg::create_test_max_capacity_and_root_storage,
            flow_ctrl::{cmds::Cmd, event_channel, tests::network_utils::create_comm},
            messaging::{OutgoingMsg, Peers},
        },
        UsedSpace,
    };
    use assert_matches::assert_matches;
    use bls::SecretKeySet;
    use eyre::{eyre, Result};
    use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
    use sn_interface::{
        init_logger,
        messaging::{
            signature_aggregator::SignatureAggregator,
            system::{DkgSessionId, NodeMsg, Proposal, SectionSigned},
            MsgId, SectionSigShare, Traceroute,
        },
        network_knowledge::{
            supermajority, NodeState, SectionAuthUtils, SectionKeyShare, SectionsDAG,
        },
        test_utils::{TestKeys, TestNetworkKnowledge, TestSectionTree},
        types::Peer,
        SectionAuthorityProvider,
    };
    use std::{
        collections::{BTreeMap, BTreeSet},
        sync::Arc,
    };
    use tokio::sync::RwLock;
    use xor_name::{Prefix, XorName};

    /// Simulate an entire round of dkg till termination; The dkg round just creates a new keyshare
    /// but without any elder change (i.e., the dkg is between the same set of elders). The test
    /// collect the `NodeMsg`s and passes them to the recipient nodes directly instead of using the
    /// comm module.
    /// TODO: implement elder change, i.e., a new node with higher age joins, now all 8 nodes should
    /// handle the messages.
    #[tokio::test]
    async fn simulate_dkg_round() -> Result<()> {
        init_logger();
        // Construct a local task set that can run `!Send` futures.
        let loc = tokio::task::LocalSet::new();

        loc.run_until(async {
            let mut rng_for_seed = rand::thread_rng();
            let seed = rng_for_seed.gen();
            let node_count = 7;
            let mut rng = StdRng::seed_from_u64(seed);
            let traceroute = Traceroute(vec![]);
            let (mut node_instances, initial_sk_set) =
                MyNodeInstance::new_instances(node_count, &mut rng).await?;

            // let current set of elders start the dkg round and capture the msgs that are outbound to the other nodes
            MyNodeInstance::start_dkg(&mut node_instances).await?;

            let mut new_secret_key_shares = BTreeMap::new();
            let mut done = false;
            while !done {
                // For each of the node instances and for each of the msg in their `msg_queue`, 1) handle the msg and
                // the cmds 2) handle the cmds 3) if the cmds produce more msgs, add them to the `msg_queue` of the
                // respective nodes
                let mut msgs_to_other_nodes = Vec::new();
                for mock_node in node_instances.values() {
                    let node = mock_node.node.clone();
                    debug!("\n\n NODE: {}", node.read().await.name());
                    while let Some((msg_id, msg, sender)) = mock_node.msg_queue.write().await.pop()
                    {
                        let cmds = MyNode::handle_valid_system_msg(
                            node.clone(),
                            msg_id,
                            msg,
                            sender,
                            &mock_node.comm,
                            traceroute.clone(),
                        )
                        .await?;

                        for cmd in cmds {
                            debug!("Got cmd {}", cmd);
                            match cmd {
                                Cmd::SendMsg {
                                    msg,
                                    msg_id,
                                    recipients,
                                    ..
                                } => {
                                    let new_msgs =
                                        node.read().await.mock_send_msg(msg, msg_id, recipients)?;
                                    msgs_to_other_nodes.push(new_msgs);
                                }
                                Cmd::HandleDkgOutcome {
                                    section_auth,
                                    outcome,
                                } => {
                                    // capture the sk_share here as we don't proceed with the SAP update
                                    let _ = new_secret_key_shares
                                        .insert(node.read().await.name(), outcome.clone());
                                    let ((_, msg, _), _) = node
                                        .write()
                                        .await
                                        .mock_dkg_outcome_proposal(section_auth, outcome)
                                        .await?;
                                    assert_matches!(msg, NodeMsg::Propose { proposal, .. } => {
                                        assert_matches!(proposal, Proposal::SectionInfo(_))
                                    });
                                }
                                _ => panic!("got a different cmd {:?}", cmd),
                            }
                        }
                    }
                }

                // add the msgs to the msg_queue of each node
                MyNodeInstance::add_msgs_to_queue(&mut node_instances, msgs_to_other_nodes).await?;

                // done if the queues are empty
                done = MyNodeInstance::is_msg_queue_empty(&node_instances).await;
            }

            // dkg done, make sure the new key share is valid
            MyNodeInstance::verify_new_key(
                &node_instances,
                &initial_sk_set,
                &new_secret_key_shares,
                node_count,
            )
            .await?;

            Ok(())
        })
        .await
    }

    /// If a node already has the new SAP in its `SectionTree`, then it should not propose `SectionInfo`
    #[tokio::test]
    async fn lagging_node_should_not_propose_new_section_info() -> Result<()> {
        init_logger();
        // Construct a local task set that can run `!Send` futures.
        let loc = tokio::task::LocalSet::new();

        loc.run_until(async {
            let mut rng = rand::thread_rng();
            let node_count = 7;
            let traceroute = Traceroute(vec![]);
            let (mut node_instances, initial_sk_set) =
                MyNodeInstance::new_instances(node_count, &mut rng).await?;

            // let current set of elders start the dkg round and capture the msgs that are outbound to the other nodes
            MyNodeInstance::start_dkg(&mut node_instances).await?;

            let mut new_secret_key_shares: BTreeMap<XorName, SectionKeyShare> = BTreeMap::new();
            let mut new_sap: BTreeSet<SectionAuthorityProvider> = BTreeSet::new();
            let mut lagging = false;
            let mut done = false;
            while !done {
                // For each of the node instances and for each of the msg in their `msg_queue`, 1) handle the msg and
                // the cmds 2) handle the cmds 3) if the cmds produce more msgs, add them to the `msg_queue` of the
                // respective nodes
                let mut msgs_to_other_nodes = Vec::new();
                for mock_node in node_instances.values() {
                    let node = mock_node.node.clone();
                    let name = node.read().await.name();
                    debug!("\n\n NODE: {}", name);
                    while let Some((msg_id, msg, sender)) = mock_node.msg_queue.write().await.pop()
                    {
                        let cmds = MyNode::handle_valid_system_msg(
                            node.clone(),
                            msg_id,
                            msg,
                            sender,
                            &mock_node.comm,
                            traceroute.clone(),
                        )
                        .await?;

                        // if we have the supermajority of the sk_shares, sign the sap and insert it into the remaining node's
                        // section tree. Now these nodes should not trigger the `Proposal::SectionInfo`
                        if !lagging && new_secret_key_shares.len() >= supermajority(node_count) {
                            assert_eq!(new_sap.len(), 1);
                            let new_sap = new_sap
                                .clone()
                                .into_iter()
                                .next()
                                .ok_or_else(|| eyre!("should contain 1"))?;
                            let serialized_new_sap = bincode::serialize(&new_sap.clone())?;
                            // sign the sap using the new_key to get the signed sap
                            let mut sig_shares = Vec::new();
                            for share in new_secret_key_shares.values() {
                                let sig_share = SectionSigShare::new(
                                    share.public_key_set.clone(),
                                    share.index,
                                    &share.secret_key_share,
                                    &serialized_new_sap,
                                );
                                sig_shares.push(sig_share);
                            }
                            let mut agg = SignatureAggregator::default();
                            for sig_share in &sig_shares {
                                // try_aggregate will return Some<_> only once;
                                if let Some(section_sig) =
                                    agg.try_aggregate(&serialized_new_sap, sig_share.clone())?
                                {
                                    let signed_sap =
                                        SectionSigned::new(new_sap.clone(), section_sig);
                                    let proof_chain = {
                                        let parent = initial_sk_set.public_keys().public_key();
                                        let mut dag = SectionsDAG::new(parent);
                                        let sig = TestKeys::sign(
                                            &initial_sk_set.secret_key(),
                                            &new_sap.section_key(),
                                        )?;
                                        dag.insert(&parent, new_sap.section_key(), sig)?;
                                        dag
                                    };
                                    let update = TestSectionTree::get_section_tree_update(
                                        &signed_sap,
                                        &proof_chain,
                                        &initial_sk_set.secret_key(),
                                    )?;

                                    // find all the lagging nodes; i.e., ones that are yet to handle the dkg_outcome
                                    let lagging_nodes = node_instances
                                        .keys()
                                        .filter(|node| !new_secret_key_shares.contains_key(node))
                                        .cloned()
                                        .collect::<Vec<_>>();
                                    debug!("Lagging node {lagging_nodes:?}");
                                    // update them
                                    for lag in lagging_nodes {
                                        let _updated = node_instances
                                            .get(&lag)
                                            .ok_or_else(|| eyre!("node will be present"))?
                                            .node
                                            .write()
                                            .await
                                            .network_knowledge
                                            .update_knowledge_if_valid(
                                                update.clone(),
                                                None,
                                                &name,
                                            )?;
                                        debug!("nw update: {_updated} for {lag} ");
                                    }
                                    // successfully simulated lagging nodes
                                    lagging = true;
                                }
                            }

                            // if we have passed the supermajority check, then we should enter the aggregate block; else
                            // something went wrong
                            if !lagging {
                                panic!("sig aggregation did not complete")
                            }
                        }

                        for cmd in cmds {
                            debug!("Got cmd {}", cmd);
                            match cmd {
                                Cmd::SendMsg {
                                    msg,
                                    msg_id,
                                    recipients,
                                    ..
                                } => {
                                    let new_msgs =
                                        node.read().await.mock_send_msg(msg, msg_id, recipients)?;
                                    msgs_to_other_nodes.push(new_msgs);
                                }
                                Cmd::HandleDkgOutcome {
                                    section_auth,
                                    outcome,
                                } => {
                                    let _ = new_secret_key_shares
                                        .insert(node.read().await.name(), outcome.clone());
                                    let _ = new_sap.insert(section_auth.clone());
                                    if !lagging {
                                        let ((_, msg, _), _) = node
                                            .write()
                                            .await
                                            .mock_dkg_outcome_proposal(section_auth, outcome)
                                            .await?;
                                        assert_matches!(msg, NodeMsg::Propose { proposal, .. } => {
                                            assert_matches!(proposal, Proposal::SectionInfo(_))
                                        });
                                    } else {
                                        // Since the dkg session is for the same prefix, the
                                        // lagging node just returns a empty cmd list. There are
                                        // multiple paths here and testing them here is not a wise
                                        // choice, instead we can test them where the logic is
                                        // defined.
                                        let cmds = node
                                            .write()
                                            .await
                                            .handle_dkg_outcome(section_auth, outcome)
                                            .await?;
                                        assert_eq!(cmds.len(), 0);
                                    }
                                }
                                _ => panic!("got a different cmd {:?}", cmd),
                            }
                        }
                    }
                }

                // add the msgs to the msg_queue of each node
                MyNodeInstance::add_msgs_to_queue(&mut node_instances, msgs_to_other_nodes).await?;

                // done if the queues are empty
                done = MyNodeInstance::is_msg_queue_empty(&node_instances).await;
            }

            // dkg done, make sure the new key share is valid
            MyNodeInstance::verify_new_key(
                &node_instances,
                &initial_sk_set,
                &new_secret_key_shares,
                node_count,
            )
            .await?;

            Ok(())
        })
        .await
    }

    #[tokio::test]
    async fn total_participation_is_required_for_dkg_votes() -> Result<()> {
        init_logger();
        // Construct a local task set that can run `!Send` futures.
        let loc = tokio::task::LocalSet::new();

        loc.run_until(async {
            let mut rng = rand::thread_rng();
            let node_count = 7;
            let traceroute = Traceroute(vec![]);
            let (mut node_instances, _initial_sk_set) =
                MyNodeInstance::new_instances(node_count, &mut rng).await?;

            MyNodeInstance::start_dkg(&mut node_instances).await?;

            let dead_node = node_instances
                .keys()
                .next()
                .cloned()
                .ok_or_else(|| eyre!("node_instances is not empty"))?;
            let mut done = false;
            while !done {
                let mut msgs_to_other_nodes = Vec::new();
                for mock_node in node_instances.values() {
                    let node = mock_node.node.clone();
                    debug!("\n\n NODE: {}", node.read().await.name());
                    while let Some((msg_id, msg, sender)) = mock_node.msg_queue.write().await.pop()
                    {
                        let cmds = MyNode::handle_valid_system_msg(
                            node.clone(),
                            msg_id,
                            msg,
                            sender,
                            &mock_node.comm,
                            traceroute.clone(),
                        )
                        .await?;

                        for cmd in cmds {
                            debug!("Got cmd {}", cmd);
                            match cmd {
                                Cmd::SendMsg {
                                    msg,
                                    msg_id,
                                    recipients,
                                    ..
                                } => {
                                    let mut new_msgs =
                                        node.read().await.mock_send_msg(msg, msg_id, recipients)?;
                                    // dead_node will not recieve the msg
                                    new_msgs.1.retain(|peer| peer.name() != dead_node);
                                    msgs_to_other_nodes.push(new_msgs);
                                }
                                _ => panic!("got a different cmd {:?}", cmd),
                            }
                        }
                    }
                }

                // add the msgs to the msg_queue of each node
                MyNodeInstance::add_msgs_to_queue(&mut node_instances, msgs_to_other_nodes).await?;

                // done if the queues are empty
                done = MyNodeInstance::is_msg_queue_empty(&node_instances).await;
            }

            // all the msgs are processed and we counldn't reach dkg termination
            Ok(())
        })
        .await
    }

    // Test helpers

    /// TODO: integrate this into `NodeTestBuilder` which only creates a single working node for now
    /// Generate a set of working node instances (no dispatcher)
    async fn generate_working_nodes<R: RngCore>(
        elders: usize,
        rng: &mut R,
    ) -> Result<(Vec<MyNode>, Vec<Comm>, SecretKeySet)> {
        let mut nodes = Vec::new();
        let mut comms = Vec::new();
        let (max_capacity, root_storage_dir) = create_test_max_capacity_and_root_storage()?;

        let gen_section_key_set = {
            // sk_share for each elder
            let poly = bls::poly::Poly::random(supermajority(elders), rng);
            SecretKeySet::from(poly)
        };

        let (network_knowledge, node_infos) = TestNetworkKnowledge::random_section_with_key(
            Prefix::default(),
            elders,
            0,
            &gen_section_key_set,
        )?;

        for (idx, info) in node_infos.into_iter().enumerate() {
            let comm = create_comm().await?;
            let section_key_share = TestKeys::get_section_key_share(&gen_section_key_set, idx);
            let node = MyNode::new(
                comm.socket_addr(),
                info.keypair.clone(),
                network_knowledge.clone(),
                Some(section_key_share),
                event_channel::new(1).0,
                UsedSpace::new(max_capacity),
                root_storage_dir.clone(),
            )
            .await?;
            nodes.push(node);
            comms.push(comm);
        }
        Ok((nodes, comms, gen_section_key_set))
    }

    type MockSystemMsg = (MsgId, NodeMsg, Peer);

    struct MyNodeInstance {
        node: Arc<RwLock<MyNode>>,
        comm: Comm,
        // might need to modify the queue when we are iterating through `Vec<MyNodeInstance>`, hence the rwlock
        msg_queue: RwLock<Vec<MockSystemMsg>>,
    }

    impl MyNodeInstance {
        // Creates a set of MyNodeInstances. The network contains a genesis section with all the
        // node_count present in it. The gen_sk_set is also returned
        async fn new_instances<R: RngCore>(
            node_count: usize,
            rng: &mut R,
        ) -> Result<(BTreeMap<XorName, MyNodeInstance>, SecretKeySet)> {
            let (nodes, comm, sk_set) = generate_working_nodes(node_count, rng).await?;

            let node_instances = nodes
                .into_iter()
                .zip(comm.into_iter())
                .map(|(node, comm)| {
                    let name = node.name();
                    let mock = MyNodeInstance {
                        node: Arc::new(RwLock::new(node)),
                        comm,
                        msg_queue: RwLock::new(Vec::new()),
                    };
                    (name, mock)
                })
                .collect::<BTreeMap<_, _>>();
            Ok((node_instances, sk_set))
        }

        // Each node sends out DKG start msg and they are added to the msg queue for the other nodes
        async fn start_dkg(nodes: &mut BTreeMap<XorName, MyNodeInstance>) -> Result<()> {
            let mut elders = BTreeMap::new();
            for (name, node) in nodes.iter() {
                let _ = elders.insert(*name, node.node.read().await.addr);
            }
            let bootstrap_members = elders
                .iter()
                .map(|(name, addr)| {
                    let peer = Peer::new(*name, *addr);
                    NodeState::joined(peer, None)
                })
                .collect::<BTreeSet<_>>();
            // A DKG session which just creates a new key for the same set of eleders
            let session_id = DkgSessionId {
                prefix: Prefix::default(),
                elders,
                section_chain_len: 1,
                bootstrap_members,
                membership_gen: 0,
            };
            let mut msgs_to_other_nodes = Vec::new();
            for node in nodes.values() {
                let mut node = node.node.write().await;
                let mut cmd = node.send_dkg_start(session_id.clone())?;
                assert_eq!(cmd.len(), 1);
                let msg = assert_matches!(cmd.remove(0), Cmd::SendMsg { msg, msg_id, recipients, .. } => (msg, msg_id, recipients));
                let msg = node.mock_send_msg(msg.0, msg.1, msg.2)?;
                msgs_to_other_nodes.push(msg);
            }
            // add the msgs to the msg_queue of each node
            Self::add_msgs_to_queue(nodes, msgs_to_other_nodes).await
        }

        // Given a list of node instances and a lit of NodeMsgs, add the msgs to the message queue of the recipients
        async fn add_msgs_to_queue(
            nodes: &mut BTreeMap<XorName, MyNodeInstance>,
            msgs: Vec<(MockSystemMsg, Vec<Peer>)>,
        ) -> Result<()> {
            for (system_msg, recipients) in msgs {
                for recp in recipients {
                    nodes
                        .get(&recp.name())
                        .ok_or_else(|| eyre!("recp is present in node_instances"))?
                        .msg_queue
                        .write()
                        .await
                        .push(system_msg.clone());
                }
            }
            Ok(())
        }

        async fn is_msg_queue_empty(nodes: &BTreeMap<XorName, MyNodeInstance>) -> bool {
            let mut not_empty = false;
            for node in nodes.values() {
                if !node.msg_queue.read().await.is_empty() {
                    not_empty = true;
                }
            }
            !not_empty
        }

        async fn verify_new_key(
            nodes: &BTreeMap<XorName, MyNodeInstance>,
            previous_sk_set: &SecretKeySet,
            new_secret_key_shares: &BTreeMap<XorName, SectionKeyShare>,
            node_count: usize,
        ) -> Result<()> {
            let mut pub_key_set = BTreeSet::new();
            let mut sig_shares = Vec::new();
            for node in nodes.values() {
                let node = node.node.read().await;
                let old_pk = node
                    .section_keys_provider
                    .key_share(&previous_sk_set.secret_key().public_key())?
                    .public_key_set
                    .public_key();
                // new key
                // a node can be absent if it did not terminate
                let key_share = if let Some(share) = new_secret_key_shares.get(&node.name()) {
                    share
                } else {
                    continue;
                };
                let pk = key_share.public_key_set.public_key();
                assert_ne!(old_pk, pk);
                let _ = pub_key_set.insert(pk);

                let sig_share = SectionSigShare::new(
                    key_share.public_key_set.clone(),
                    key_share.index,
                    &key_share.secret_key_share,
                    "msg".as_bytes(),
                );
                sig_shares.push(sig_share);
            }
            assert_eq!(pub_key_set.len(), 1);
            let mut agg = SignatureAggregator::default();
            let mut sig_count = 1;
            for sig_share in sig_shares {
                // sup(7) = 5 i.e, we need 5 shares to gen the complete sig; Thus the first 4 return None, and 5th one
                // gives us the complete sig;
                if sig_count < supermajority(node_count) || sig_count > supermajority(node_count) {
                    assert!(agg.try_aggregate("msg".as_bytes(), sig_share)?.is_none());
                } else if sig_count == supermajority(node_count) {
                    assert!(agg.try_aggregate("msg".as_bytes(), sig_share)?.is_some());
                }
                sig_count += 1;
            }
            Ok(())
        }
    }

    impl MyNode {
        fn mock_send_msg(
            &self,
            msg: OutgoingMsg,
            msg_id: MsgId,
            recipients: Peers,
        ) -> Result<(MockSystemMsg, Vec<Peer>)> {
            trace!("msg: {msg:?} msg_id {msg_id:?}, recipients {recipients:?}");
            if let OutgoingMsg::Node(msg) = msg {
                let current_node = Peer::new(self.name(), self.addr);

                let recipients = match recipients {
                    Peers::Single(peer) => vec![peer],
                    Peers::Multiple(peers) => peers.into_iter().collect(),
                };
                let mock_system_msg: MockSystemMsg = (msg_id, msg, current_node);
                debug!("SendMsg output {}", mock_system_msg.2);
                Ok((mock_system_msg, recipients))
            } else {
                Err(eyre!("Should be OutgoingMsg::Node"))
            }
        }

        // if SectionInfo proposal is triggered, it will send out msgs to other nodes
        async fn mock_dkg_outcome_proposal(
            &mut self,
            sap: SectionAuthorityProvider,
            key_share: SectionKeyShare,
        ) -> Result<(MockSystemMsg, Vec<Peer>)> {
            let mut cmds = self.handle_dkg_outcome(sap, key_share).await?;
            // contains only the SendMsg for SectionInfo proposal
            assert_eq!(cmds.len(), 1);
            if let Cmd::SendMsg {
                msg,
                msg_id,
                recipients,
                ..
            } = cmds.remove(0)
            {
                self.mock_send_msg(msg, msg_id, recipients)
            } else {
                Err(eyre!("Should be Cmd::SendMsg"))
            }
        }
    }
}
