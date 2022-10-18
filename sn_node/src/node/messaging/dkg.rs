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
    use bls::SecretKeySet;
    use eyre::{eyre, Result};
    use rand::{rngs::StdRng, RngCore, SeedableRng};
    use sn_interface::{
        messaging::{
            signature_aggregator::SignatureAggregator,
            system::{DkgSessionId, NodeMsg},
            AuthKind, AuthorityProof, MsgId, NodeMsgAuthority, Traceroute,
        },
        network_knowledge::{
            test_utils::{gen_network_knowledge_with_key, gen_sorted_nodes},
            SectionKeyShare, SectionKeysProvider,
        },
        types::{keys::ed25519::gen_keypair, Peer},
    };
    use std::{
        collections::{BTreeMap, BTreeSet},
        sync::Arc,
    };
    use xor_name::Prefix;

    #[tokio::test]
    async fn gg() {
        let loc = tokio::task::LocalSet::new();
        loc.run_until(async {
            let mut rng = rand::thread_rng();
            generate_working_nodes(5, &mut rng).await;
        })
        .await;
    }

    #[tokio::test]
    async fn simulate_dkg_round() -> Result<()> {
        // Construct a local task set that can run `!Send` futures.
        let loc = tokio::task::LocalSet::new();

        loc.run_until(async {
            // let mut rng_for_seed = rand::thread_rng();
            // let seed = rng_for_seed.gen();
            let seed = 123;
            let mut rng = StdRng::seed_from_u64(seed);
            let traceroute = Traceroute(vec![]);
            let (nodes, comm) = generate_working_nodes(7, &mut rng).await?;

            let elders = nodes
                .iter()
                .map(|node| (node.name(), node.addr))
                .collect::<BTreeMap<_, _>>();
            let session_id = DkgSessionId {
                prefix: Prefix::default(),
                elders,
                section_chain_len: 1,
                bootstrap_members: BTreeSet::new(),
                membership_gen: 0,
            };

            let mut node_instances = nodes
                .into_iter()
                .zip(comm.into_iter())
                .map(|(node, comm)| {
                    let peer = Peer::new(node.name(), node.addr);
                    let mock = MyNodeInstance {
                        node,
                        comm,
                        msg_queue: Vec::new(),
                    };
                    (peer, mock)
                })
                .collect::<BTreeMap<_, _>>();

            // let a random node start dkg
            let random_node = &mut node_instances
                .values_mut()
                .next()
                .ok_or_else(|| eyre!("Have atleast 1 node"))?
                .node;
            let original_tree = random_node.network_knowledge().section_tree().clone();
            let cmds = random_node.send_dkg_start(session_id)?;

            // random node should process cmds => assuming they are SendMsgs for now
            let mut msgs_to_other_nodes = Vec::new();
            for cmd in cmds {
                let msg = random_node
                    .mock_process_send_msg(cmd)?
                    .ok_or_else(|| eyre!("send_dkg_start will send msgs to other nodes"))?;
                println!("random_node cmd output: {}", msg.0 .2);
                msgs_to_other_nodes.push(msg);
            }
            // add the msgs to the recipients queue
            msgs_to_other_nodes
                .into_iter()
                .try_for_each(|(system_msg, recipients)| {
                    recipients.iter().try_for_each(|recp| -> Result<()> {
                        node_instances
                            .get_mut(recp)
                            .ok_or_else(|| eyre!("recp is present in node_instances"))?
                            .msg_queue
                            .push(system_msg.clone());
                        Ok(())
                    })
                })?;

            let mut done = false;
            while !done {
                // let the nodes process the 1. SystemMsg -> 2. Process Cmd from prev step -> 3. add the system msg to queue
                let mut msgs_to_other_nodes = Vec::new();
                for mock_node in node_instances.values_mut() {
                    let node = &mut mock_node.node;
                    let comm = &mock_node.comm;
                    println!("\n\n NODE: {}", node.name());
                    while let Some((msg_id, msg_authority, msg, sender)) = mock_node.msg_queue.pop()
                    {
                        let cmds = node
                            .handle_valid_system_msg(
                                msg_id,
                                msg_authority,
                                msg,
                                sender,
                                comm,
                                traceroute.clone(),
                            )
                            .await?;

                        for cmd in cmds {
                            println!("Got cmd {}", cmd);
                            if let Some(new_msgs) = node.mock_process_send_msg(cmd.clone())? {
                                println!("Cmd output {}", new_msgs.0 .2);
                                msgs_to_other_nodes.push(new_msgs);
                            } else {
                                let cmds = node.mock_process_dkg_outcome(cmd).await?;
                                for cmd in cmds {
                                    let mut new_msgs =
                                        node.mock_process_send_msg(cmd.clone())?.ok_or_else(
                                            || eyre!("dkg_outcome will send msgs to other nodes"),
                                        )?;

                                    // if no recepients, lets handle it (because we get a proposal here, not sure what to do wwith it)
                                    if new_msgs.1.is_empty() {
                                        new_msgs.1 = vec![Peer::new(node.name(), node.addr)];
                                    }
                                    println!("Cmd output after dkg outcome {}", new_msgs.0 .2);
                                    msgs_to_other_nodes.push(new_msgs);
                                }
                            }
                        }
                    }
                }

                msgs_to_other_nodes
                    .into_iter()
                    .try_for_each(|(system_msg, recipients)| {
                        recipients.iter().try_for_each(|recp| -> Result<()> {
                            node_instances
                                .get_mut(recp)
                                .ok_or_else(|| eyre!("recp is present in node_instances"))?
                                .msg_queue
                                .push(system_msg.clone());
                            Ok(())
                        })
                    })?;

                // done if the queues are empty
                done = node_instances
                    .values()
                    .all(|node| node.msg_queue.is_empty());
            }

            // dkg done, make sure the key is valid
            let mut pub_key_set = BTreeSet::new();
            // let mut sig_shares = Vec::new();
            let _agg = SignatureAggregator::default();
            println!("\n\n{original_tree:?}");
            for node in node_instances.values() {
                let key_share = node.node.key_share()?;
                let _ = pub_key_set.insert(key_share.public_key_set);

                // agg.try_aggregate("msg".as_bytes(), key_share.secret_key_share.sign("msg"))?;
            }

            assert_eq!(pub_key_set.len(), 1);

            Ok(())
        })
        .await
    }

    /// Generate a random `SectionAuthorityProvider` for testing.
    ///
    /// The total number of members in the section will be `elder_count` + `adult_count`. A lot of
    /// tests don't require adults in the section, so zero is an acceptable value for
    /// `adult_count`.
    ///
    /// An optional `sk_threshold_size` can be passed to specify the threshold when the secret key
    /// set is generated for the section key. Some tests require a low threshold.
    async fn generate_working_nodes<R: RngCore>(
        elders: usize,
        rng: &mut R,
    ) -> Result<(Vec<MyNode>, Vec<Comm>)> {
        let mut nodes = Vec::new();
        let mut comms = Vec::new();
        let (max_capacity, root_storage_dir) = create_test_max_capacity_and_root_storage()?;

        // node infos for all the elders
        // the key used by the node to sign NodeMsgs
        let gen_keypair = gen_keypair(&Prefix::default().range_inclusive(), 255);
        //  Used to derive genesis_key i.e., the first section's key
        let gen_section_key_set = {
            // sk_share for each elder
            let poly = bls::poly::Poly::random(elders, rng);
            SecretKeySet::from(poly)
        };

        let (network_knowledge, node_infos) =
            gen_network_knowledge_with_key(Prefix::default(), elders, 4, &gen_section_key_set)?;

        for node in node_infos {
            println!("{:?}", node.age());
        }

        let node_infos = gen_sorted_nodes(&Prefix::default(), elders, true);
        for node in &node_infos {
            println!("ggg: {:?}", node.age());
        }

        let gen_comm = create_comm().await?;

        let (mut gen_node, _) = MyNode::first_node(
            gen_comm.socket_addr(),
            Arc::new(gen_keypair),
            event_channel::new(1).0,
            UsedSpace::new(max_capacity),
            root_storage_dir.clone(),
            // 0 threshold initially, swapping out with a
            SecretKeySet::random(0, rng),
        )
        .await?;
        // genesis gets a sk_share
        let gen_section_key_share = SectionKeyShare {
            public_key_set: gen_section_key_set.public_keys(),
            index: 0,
            secret_key_share: gen_section_key_set.secret_key_share(0),
        };
        gen_node.section_keys_provider = SectionKeysProvider::new(Some(gen_section_key_share));

        let network_knowledge = gen_node.network_knowledge.clone();
        comms.push(gen_comm);
        nodes.push(gen_node);

        for (idx, info) in node_infos.into_iter().enumerate() {
            let comm = create_comm().await?;
            let section_key_share = SectionKeyShare {
                public_key_set: gen_section_key_set.public_keys(),
                index: 0,
                // +1 since we gave one to genesis node
                secret_key_share: gen_section_key_set.secret_key_share(idx + 1),
            };
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
        Ok((nodes, comms))
    }

    type MockSystemMsg = (MsgId, NodeMsgAuthority, NodeMsg, Peer);

    struct MyNodeInstance {
        node: MyNode,
        comm: Comm,
        msg_queue: Vec<MockSystemMsg>,
    }

    impl MyNode {
        fn mock_process_send_msg(&self, cmd: Cmd) -> Result<Option<(MockSystemMsg, Vec<Peer>)>> {
            match cmd {
                Cmd::SendMsg {
                    msg,
                    msg_id,
                    recipients,
                    ..
                } => {
                    let (auth, payload) = self.sign_msg(msg.clone())?;
                    if let OutgoingMsg::Node(msg) = msg {
                        if let AuthKind::Node(auth) = auth {
                            let auth = AuthorityProof::verify(auth, payload)?;
                            let node_auth = NodeMsgAuthority::Node(auth);
                            let current_node = Peer::new(self.name(), self.addr);

                            let recipients = match recipients {
                                Peers::Single(peer) => vec![peer],
                                Peers::Multiple(peers) => peers.into_iter().collect(),
                            };
                            let mock_system_msg: MockSystemMsg =
                                (msg_id, node_auth, msg, current_node);
                            Ok(Some((mock_system_msg, recipients)))
                        } else {
                            Err(eyre!("Should be Authkind::Node"))
                        }
                    } else {
                        Err(eyre!("Should be OutgoingMsg::Node"))
                    }
                }
                Cmd::HandleDkgOutcome { .. } => Ok(None),
                _ => Err(eyre!("Should be Cmd::SendMsg")),
            }
        }

        // can lead to more cmds for self unlike the above.. (these cmds give SendMsgs again)
        async fn mock_process_dkg_outcome(&mut self, cmd: Cmd) -> Result<Vec<Cmd>> {
            match cmd {
                Cmd::HandleDkgOutcome {
                    section_auth,
                    outcome,
                } => {
                    println!("proposed sap {section_auth:?}");
                    Ok(self.handle_dkg_outcome(section_auth, outcome).await?)
                }
                _ => Err(eyre!("Should be Cmd::HandleDkgOutcome")),
            }
        }
    }
}
