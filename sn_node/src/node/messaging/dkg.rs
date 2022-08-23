// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    dkg::{check_key, DkgPubKeys},
    flow_ctrl::cmds::Cmd,
    messaging::{OutgoingMsg, Peers},
    Error, Node, Proposal, Result,
};

use bytes::Bytes;
#[cfg(feature = "traceroute")]
use sn_interface::messaging::Traceroute;
use sn_interface::{
    messaging::{
        system::{DkgSessionId, SigShare, SystemMsg},
        AuthorityProof, BlsShareAuth, NodeMsgAuthority, WireMsg,
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
        .filter_map(|(index, peer)| (index != our_index).then(|| peer))
        .collect()
}

fn acknowledge_dkg_oucome(
    session_id: &DkgSessionId,
    participant_index: usize,
    pub_key_set: PublicKeySet,
    sec_key_share: SecretKeyShare,
) -> Cmd {
    trace!(
        "{} s{}: {:?}",
        LogMarker::DkgSessionComplete,
        session_id.sum(),
        pub_key_set.public_key(),
    );

    let section_auth =
        SectionAuthorityProvider::from_dkg_session(session_id.clone(), pub_key_set.clone());

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
            session_id.sum(),
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

    fn broadcast_dkg_vote(
        &self,
        session_id: &DkgSessionId,
        pub_keys: DkgPubKeys,
        participant_index: usize,
        vote: DkgSignedVote,
    ) -> Cmd {
        let recipients = dkg_peers(participant_index, session_id);
        trace!(
            "{} s{}: {:?}",
            LogMarker::DkgBroadcastVote,
            session_id.sum(),
            vote
        );
        let node_msg = SystemMsg::DkgVotes {
            session_id: session_id.clone(),
            pub_keys,
            votes: vec![vote],
        };
        self.send_system_msg(node_msg, Peers::Multiple(recipients))
    }

    fn request_dkg_ae(&self, session_id: &DkgSessionId, sender: Peer) -> Cmd {
        let node_msg = SystemMsg::DkgAE(session_id.clone());
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
                session_id.sum()
            );
            return Ok(vec![]);
        };

        // ignore DkgStart from old chains
        let current_chain_len = self.network_knowledge.chain_len();
        if session_id.section_chain_len < current_chain_len {
            trace!("Skipping DkgStart for older chain: s{:?}", session_id.sum());
            return Ok(vec![]);
        }

        // gen key
        let (ephemeral_pub_key, sig) =
            self.dkg_voter
                .gen_ephemeral_key(session_id.hash(), our_name, &self.keypair)?;

        // assert people can check key
        assert!(check_key(&session_id, our_name, ephemeral_pub_key, sig).is_ok());

        // broadcast signed pub key
        trace!(
            "{} s{} from {our_id:?}",
            LogMarker::DkgBroadcastEphemeralPubKey,
            session_id.sum(),
        );
        let peers = dkg_peers(our_id, &session_id);
        let node_msg = SystemMsg::DkgEphemeralPubKey {
            session_id,
            pub_key: ephemeral_pub_key,
            sig,
        };

        let cmd = self.send_system_msg(node_msg, Peers::Multiple(peers));
        Ok(vec![cmd])
    }

    pub(crate) fn handle_dkg_ephemeral_pubkey(
        &mut self,
        session_id: &DkgSessionId,
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
                session_id.sum()
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
                Err(e) => {
                    error!(
                        "Failed to init DKG s{} id:{our_id:?}: {:?}",
                        session_id.sum(),
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
            session_id.sum()
        );
        let peers = dkg_peers(our_id, session_id);
        let node_msg = SystemMsg::DkgVotes {
            session_id: session_id.clone(),
            pub_keys,
            votes: vec![vote],
        };
        let cmd = self.send_system_msg(node_msg, Peers::Multiple(peers));
        Ok(vec![cmd])
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
                session_id.sum()
            );
            return Ok(vec![]);
        };

        // make sure the keys are valid
        let (pub_keys, just_completed) = self.dkg_voter.check_keys(session_id, msg_keys)?;

        // if we just completed our keyset thanks to the incoming keys, bcast 1st vote
        let mut cmds = Vec::new();
        if just_completed {
            let (first_vote, _) = self.dkg_voter.initialize_dkg_state(session_id, our_id)?;
            cmds.push(self.broadcast_dkg_vote(session_id, pub_keys.clone(), our_id, first_vote));
        }

        // handle vote
        let mut cmds: Vec<Cmd> = Vec::new();
        let mut ae_cmds: Vec<Cmd> = Vec::new();
        for v in votes {
            match self.dkg_voter.handle_dkg_vote(session_id, v.clone()) {
                Ok(VoteResponse::WaitingForMoreVotes) => {
                    debug!(
                        "Dkg s{}: WaitingForMoreVotes after: {v:?}",
                        session_id.sum()
                    );
                }
                Ok(VoteResponse::RequestAntiEntropy) => {
                    cmds.push(self.request_dkg_ae(session_id, sender))
                }
                Ok(VoteResponse::BroadcastVote(vote)) => {
                    cmds.push(self.broadcast_dkg_vote(session_id, pub_keys.clone(), our_id, *vote))
                }
                Ok(VoteResponse::DkgComplete(new_pubs, new_sec)) => {
                    debug!("DkgComplete s{:?}", session_id.sum());
                    cmds.push(acknowledge_dkg_oucome(
                        session_id, our_id, new_pubs, new_sec,
                    ));
                }
                Err(err) => {
                    error!(
                        "Error processing DKG vote s{} id:{our_id:?} {v:?} from {sender:?}: {err:?}",
                        session_id.sum()
                    );
                }
            }
        }

        // ae is not necessary if we have votes or termination cmds
        if cmds.is_empty() {
            cmds.append(&mut ae_cmds);
        }
        Ok(cmds)
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
            session_id.sum()
        );
        let node_msg = SystemMsg::DkgVotes {
            session_id,
            pub_keys,
            votes,
        };
        let cmd = self.send_system_msg(node_msg, Peers::Single(sender));
        Ok(vec![cmd])
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
