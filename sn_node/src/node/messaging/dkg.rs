// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    dkg::DkgPubKeys,
    flow_ctrl::cmds::Cmd,
    messaging::{OutgoingMsg, Peers},
    Error, Node, Proposal, Result,
};

use bytes::Bytes;
#[cfg(feature = "traceroute")]
use sn_interface::messaging::Traceroute;
use sn_interface::{
    messaging::{
        system::{DkgSessionId, SystemMsg, SigShare},
        AuthorityProof, BlsShareAuth, NodeMsgAuthority, WireMsg,
    },
    network_knowledge::{SectionAuthorityProvider, SectionKeyShare},
    types::{self, log_markers::LogMarker, Peer},
};

use bls::{PublicKeySet, SecretKeyShare, PublicKey as BlsPublicKey};
use xor_name::{XorName, Prefix};
use ed25519::Signature;
use sn_sdkg::{DkgSignedVote, VoteResponse};

/// Helper to our DKG peers (excluding us)
fn dkg_peers(our_index: usize, session_id: &DkgSessionId) -> Vec<Peer> {
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
        "{} {:?}: {:?}",
        LogMarker::DkgSessionComplete,
        session_id,
        pub_key_set.clone().public_key(),
    );

    let section_auth = SectionAuthorityProvider::from_dkg_session(
        session_id.clone(),
        pub_key_set.clone(),
    );

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
        for recipient in recipients.into_iter() {
            if recipient.name() == our_name {
                handle = true;
            } else {
                let _ = others.insert(recipient);
            }
        }

        let src_name = session_id.prefix.name();
        let msg = SystemMsg::DkgStart(session_id);
        let (auth, payload) = self.get_auth(msg.clone(), src_name)?;

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

    fn get_auth(&self, msg: SystemMsg, src_name: XorName) -> Result<(BlsShareAuth, Bytes)> {
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
    ) -> Result<Vec<Cmd>> {
        let recipients = dkg_peers(participant_index, session_id);
        let node_msg = SystemMsg::DkgVotes {
            session_id: session_id.clone(),
            pub_keys,
            votes: vec![vote],
        };
        self.send_system_msg(node_msg, Peers::Multiple(recipients))
    }

    fn request_dkg_ae(
        &self,
        session_id: &DkgSessionId,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        let node_msg = SystemMsg::DkgAE(session_id.clone());
        self.send_system_msg(node_msg, Peers::Single(sender))
    }

    pub(crate) fn handle_dkg_start(&mut self, session_id: DkgSessionId) -> Result<Vec<Cmd>> {
        // ignore DkgStart from old chains
        let current_chain_len = self.network_knowledge.chain_len();
        if session_id.section_chain_len < current_chain_len {
            trace!("Skipping DkgStart for older chain: {:?}", &session_id);
            return Ok(vec![]);
        }

        // gen key
        let ephemeral_pub_key = self.dkg_voter.gen_ephemeral_key(session_id.hash());
        let serialized = bincode::serialize(&ephemeral_pub_key)?;

        // broadcast signed pub key
        let peers = Vec::from_iter(session_id.elder_peers());
        let node_msg = SystemMsg::DkgEphemeralPubKey{
            session_id: session_id.clone(),
            pub_key: ephemeral_pub_key,
            sig: types::keys::ed25519::sign(&serialized, &self.keypair),
        };
        self.send_system_msg(node_msg, Peers::Multiple(peers))
    }

    pub(crate) fn handle_dkg_ephemeral_pubkey(
        &mut self,
        session_id: &DkgSessionId,
        pub_key: BlsPublicKey,
        sig: Signature,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        // get our index
        let name = types::keys::ed25519::name(&self.keypair.public);
        let participant_index = if let Some(index) = session_id.elder_index(name) {
            index
        } else {
            error!("DKG failed to start for {session_id:?}: {name} is not a participant");
            return Ok(vec![]);
        };

        // try to start DKG if we've got all the keys
        let (vote, pub_keys) = if let Some(start) = self.dkg_voter.try_init_dkg(
            session_id,
            participant_index,
            pub_key,
            sig,
            sender.name(),
        )? {
            start
        } else {
            // we don't have all the keys yet
            return Ok(vec![]);
        };

        // send first vote
        let peers = Vec::from_iter(session_id.elder_peers());
        let node_msg = SystemMsg::DkgVotes{
            session_id: session_id.clone(),
            pub_keys,
            votes: vec![vote],
        };
        self.send_system_msg(node_msg, Peers::Multiple(peers))
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
            error!("DKG failed to handle vote for {session_id:?}: {name} is not a participant");
            return Ok(vec![]);
        };

        // make sure the keys are valid
        let (pub_keys, just_completed) = self.dkg_voter.check_keys(&session_id, msg_keys)?;

        // if we just completed our keyset thanks to the incoming keys, bcast 1st vote
        let mut cmds = Vec::new();
        if just_completed {
            let (first_vote, _) = self.dkg_voter.initialize_dkg_state(session_id, our_id)?;
            cmds.extend(self.broadcast_dkg_vote(session_id, pub_keys, our_id, first_vote));
        }

        // handle vote
        let mut cmds: Vec<Cmd> = Vec::new();
        let mut ae_cmds: Vec<Cmd> = Vec::new();
        for v in votes {
            match self.dkg_voter.handle_dkg_vote(session_id, v) {
                Ok(VoteResponse::WaitingForMoreVotes) => {}
                Ok(VoteResponse::RequestAntiEntropy) => {
                    cmds.append(&mut self.request_dkg_ae(session_id, sender)?)
                    // TODO deal with errs above dont break
                }
                Ok(VoteResponse::BroadcastVote(vote)) => {
                    cmds.append(&mut self.broadcast_dkg_vote(session_id, pub_keys, our_id, *vote)?)
                    // TODO deal with errs above dont break
                }
                Ok(VoteResponse::DkgComplete(new_pubs, new_sec)) => {
                    cmds.push(acknowledge_dkg_oucome(session_id, our_id, new_pubs, new_sec));
                }
                Err(error) => {
                    error!("Error processing DKG vote {:?} from {:?}: {:?}", v, sender, error);
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
        let node_msg = SystemMsg::DkgVotes{
            session_id: session_id,
            pub_keys: self.dkg_voter.get_dkg_keys(&session_id)?,
            votes: self.dkg_voter.get_all_votes(&session_id)?,
        };
        self.send_system_msg(node_msg, Peers::Single(sender))
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
            let recipients: Vec<_> = self.network_knowledge.authority_provider().elders_vec();
            self.send_proposal_with(recipients, proposal, &key_share)
        }
    }
}
