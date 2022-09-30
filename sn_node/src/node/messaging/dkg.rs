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
    messaging::{OutgoingMsg, Peers},
    Error, Node, Proposal, Result,
};

use bytes::Bytes;
#[cfg(feature = "traceroute")]
use sn_interface::messaging::Traceroute;
use sn_interface::{
    messaging::{
        system::{DkgSessionId, NodeMsg, SigShare},
        AuthorityProof, NodeMsgAuthority, SectionAuth, SectionAuthShare, WireMsg,
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

impl Node {
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
    pub(crate) fn send_dkg_start(&self, session_id: DkgSessionId) -> Result<Vec<Cmd>> {
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
        let msg = NodeMsg::DkgStart(session_id);
        let (auth, payload) = self.get_auth(&msg, src_name)?;

        if !others.is_empty() {
            cmds.push(Cmd::send_msg(
                OutgoingMsg::SectionAuth((auth.clone(), payload.clone())),
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

    fn get_auth(&self, msg: &NodeMsg, src_name: XorName) -> Result<(SectionAuthShare, Bytes)> {
        let section_key = self.network_knowledge.section_key();
        let key_share = self
            .section_keys_provider
            .key_share(&section_key)
            .map_err(|err| {
                trace!("Can't create message {:?} for accumulation: {:?}", msg, err);
                err
            })?;

        let payload = WireMsg::serialize_msg_payload(&msg).map_err(|_| Error::InvalidMessage)?;

        let auth = SectionAuthShare {
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

    pub(crate) fn handle_dkg_start(&mut self, session_id: DkgSessionId) -> Result<Vec<Cmd>> {
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

        // get original auth (as proof for those who missed the original DkgStart msg)
        let section_auth = self
            .dkg_sessions_info
            .get(&session_id.hash())
            .ok_or(Error::InvalidState)?
            .authority
            .clone();

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
        section_auth: AuthorityProof<SectionAuth>,
        pub_key: BlsPublicKey,
        sig: Signature,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        trace!(
            "Detected missed dkg start for s{:?} after msg from {sender:?}",
            session_id.sh()
        );

        // reconstruct the original DKG start message and verify the section signature
        let payload = WireMsg::serialize_msg_payload(&NodeMsg::DkgStart(session_id.clone()))?;
        let auth = section_auth.clone().into_inner();
        if self.network_knowledge.section_key() != auth.sig.public_key {
            warn!(
                "Invalid section key in dkg auth proof in s{:?}: {sender:?}",
                session_id.sh()
            );
            return Ok(vec![]);
        }
        if let Err(err) = AuthorityProof::verify(auth, payload) {
            error!(
                "Invalid signature in dkg auth proof in s{:?}: {err:?}",
                session_id.sh()
            );
            return Ok(vec![]);
        }

        // acknowledge Dkg Session
        info!(
            "Handling missed dkg start for s{:?} after msg from {sender:?}",
            session_id.sh()
        );
        self.log_dkg_session(&sender.name());
        let session_info = DkgSessionInfo {
            session_id: session_id.clone(),
            authority: section_auth.clone(),
        };
        let _existing = self
            .dkg_sessions_info
            .insert(session_id.hash(), session_info);

        // catch back up
        let mut cmds = vec![];
        cmds.extend(self.handle_dkg_start(session_id.clone())?);
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
        section_auth: AuthorityProof<SectionAuth>,
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
            VoteResponse::WaitingForMoreVotes | VoteResponse::IgnoringKnownVote => {}
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
                Ok(vote_response) => {
                    debug!("Dkg s{}: {:?} after: {v:?}", session_id.sh(), vote_response,);
                    if !matches!(vote_response, VoteResponse::IgnoringKnownVote) {
                        self.dkg_voter.learned_something_from_message();
                        is_old_gossip = false;
                    }
                    let (cmd, ae_cmd) = self.handle_vote_response(
                        session_id,
                        pub_keys.clone(),
                        sender,
                        our_id,
                        vote_response,
                    );
                    cmds.extend(cmd);
                    ae_cmds.extend(ae_cmd);
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
