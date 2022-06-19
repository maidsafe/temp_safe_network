// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_consensus::Generation;
use sn_interface::{
    messaging::system::{DkgFailureSig, DkgFailureSigSet, DkgSessionId, NodeState},
    network_knowledge::supermajority,
    types::keys::ed25519::{self, Digest256, Keypair, Verifier},
};

use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
};
use tiny_keccak::{Hasher, Sha3};
use xor_name::{Prefix, XorName};

// TODO: remove all of these traits
pub(crate) trait DkgSessionIdUtils {
    fn new(
        prefix: Prefix,
        elder: BTreeMap<XorName, SocketAddr>,
        generation: u64,
        bootstrap_members: BTreeSet<NodeState>,
        membership_gen: Generation,
    ) -> Self;
}

impl DkgSessionIdUtils for DkgSessionId {
    fn new(
        prefix: Prefix,
        elders: BTreeMap<XorName, SocketAddr>,
        section_chain_len: u64,
        bootstrap_members: BTreeSet<NodeState>,
        membership_gen: Generation,
    ) -> Self {
        assert!(elders
            .keys()
            .all(|e| bootstrap_members.iter().any(|m| &m.name == e)));

        // Calculate the hash without involving serialization to avoid having to return `Result`.
        Self {
            prefix,
            elders,
            section_chain_len,
            bootstrap_members,
            membership_gen,
        }
    }
}

pub(crate) trait DkgFailureSigUtils {
    fn new(
        keypair: &Keypair,
        failed_participants: &BTreeSet<XorName>,
        dkg_key: DkgSessionId,
    ) -> Self;

    fn verify(&self, dkg_key: &DkgSessionId, failed_participants: &BTreeSet<XorName>) -> bool;
}

impl DkgFailureSigUtils for DkgFailureSig {
    fn new(
        keypair: &Keypair,
        failed_participants: &BTreeSet<XorName>,
        session_id: DkgSessionId,
    ) -> Self {
        DkgFailureSig {
            public_key: keypair.public,
            signature: ed25519::sign(&hashed_failure(&session_id, failed_participants), keypair),
            session_id,
        }
    }

    fn verify(&self, dkg_key: &DkgSessionId, failed_participants: &BTreeSet<XorName>) -> bool {
        let hash = hashed_failure(dkg_key, failed_participants);
        self.public_key.verify(&hash, &self.signature).is_ok()
    }
}

pub(crate) trait DkgFailureSigSetUtils {
    fn insert(&mut self, sig: DkgFailureSig, failed_participants: &BTreeSet<XorName>) -> bool;

    fn has_agreement(&self, session_id: &DkgSessionId) -> bool;

    fn verify(&self, reference_session_id: &DkgSessionId) -> bool;
}

impl DkgFailureSigSetUtils for DkgFailureSigSet {
    // Insert a signature into this set. The signature is assumed valid. Returns `true` if the signature was
    // not already present in the set and `false` otherwise.
    fn insert(&mut self, sig: DkgFailureSig, failed_participants: &BTreeSet<XorName>) -> bool {
        if self.failed_participants.is_empty() {
            self.failed_participants = failed_participants.clone();
        }
        if self
            .sigs
            .iter()
            .all(|existing_sig| existing_sig.public_key != sig.public_key)
        {
            self.sigs.push(sig);
            true
        } else {
            false
        }
    }

    // Check whether we have enough signatures to reach agreement on the failure. The contained signatures
    // are assumed valid.
    fn has_agreement(&self, session_id: &DkgSessionId) -> bool {
        has_failure_agreement(session_id.elders.len(), self.sigs.len())
    }

    fn verify(&self, reference_session_id: &DkgSessionId) -> bool {
        let hash = hashed_failure(reference_session_id, &self.failed_participants);
        let votes = self
            .sigs
            .iter()
            .filter(|sig| {
                let sig_name = ed25519::name(&sig.public_key);
                reference_session_id.contains_elder(sig_name)
                    && sig.public_key.verify(&hash, &sig.signature).is_ok()
            })
            .count();

        has_failure_agreement(reference_session_id.elders.len(), votes)
    }
}

// Check whether we have enough signeds to reach agreement on the failure. We only need
// `N - supermajority(N) + 1` signeds, because that already makes a supermajority agreement on a
// successful outcome impossible.
fn has_failure_agreement(num_participants: usize, num_votes: usize) -> bool {
    num_votes > num_participants - supermajority(num_participants)
}

// Create a value whose signature serves as proof that a failure of a DKG session with the given
// `dkg_key` was observed.
fn hashed_failure(dkg_key: &DkgSessionId, failed_participants: &BTreeSet<XorName>) -> Digest256 {
    let mut hasher = Sha3::v256();
    let mut hash = Digest256::default();

    dkg_key.hash_update(&mut hasher);

    for name in failed_participants.iter() {
        hasher.update(&name.0);
    }

    hasher.update(b"failure");

    hasher.finalize(&mut hash);
    hash
}
