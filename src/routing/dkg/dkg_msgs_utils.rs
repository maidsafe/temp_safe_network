// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::node::{DkgFailureSig, DkgFailureSigSet, DkgKey, ElderCandidates};
use crate::routing::{
    ed25519::{self, Digest256, Keypair, Verifier},
    peer::PeerUtils,
    section::ElderCandidatesUtils,
    supermajority,
};
use crate::types::CFSet;
use dashmap::DashSet;
use std::collections::BTreeSet;
use tiny_keccak::{Hasher, Sha3};
use xor_name::XorName;

pub(crate) trait DkgKeyUtils {
    fn new(elder_candidates: &ElderCandidates, generation: u64) -> Self;
}

impl DkgKeyUtils for DkgKey {
    fn new(elder_candidates: &ElderCandidates, generation: u64) -> Self {
        // Calculate the hash without involving serialization to avoid having to return `Result`.
        let mut hasher = Sha3::v256();
        let mut hash = Digest256::default();

        for peer in elder_candidates.peers() {
            hasher.update(&peer.name().0);
        }

        hasher.update(&elder_candidates.prefix.name().0);
        hasher.update(&elder_candidates.prefix.bit_count().to_le_bytes());
        hasher.finalize(&mut hash);

        Self { hash, generation }
    }
}

pub(crate) trait DkgFailureSigUtils {
    fn new(keypair: &Keypair, failed_participants: &BTreeSet<XorName>, dkg_key: &DkgKey) -> Self;

    fn verify(&self, dkg_key: &DkgKey, failed_participants: &BTreeSet<XorName>) -> bool;
}

impl DkgFailureSigUtils for DkgFailureSig {
    fn new(keypair: &Keypair, failed_participants: &BTreeSet<XorName>, dkg_key: &DkgKey) -> Self {
        DkgFailureSig {
            public_key: keypair.public,
            signature: ed25519::sign(&hashed_failure(dkg_key, failed_participants), keypair),
        }
    }

    fn verify(&self, dkg_key: &DkgKey, failed_participants: &BTreeSet<XorName>) -> bool {
        let hash = hashed_failure(dkg_key, failed_participants);
        self.public_key.verify(&hash, &self.signature).is_ok()
    }
}

pub(crate) trait DkgFailureSigSetUtils {
    fn insert(&self, sig: DkgFailureSig, failed_participants: &BTreeSet<XorName>) -> bool;

    fn has_agreement(&self, elder_candidates: &ElderCandidates) -> bool;

    fn verify(&self, elder_candidates: &ElderCandidates, generation: u64) -> bool;
}
/// Dkg failure info for a round
#[derive(Debug)]
pub(crate) struct DkgFailChecker {
    //
    pub(crate) sigs: CFSet<DkgFailureSig>,
    //
    pub(crate) failed_participants: DashSet<XorName>,
}

impl From<&DkgFailureSigSet> for DkgFailChecker {
    fn from(set: &DkgFailureSigSet) -> Self {
        let mapped = DkgFailChecker::new();
        for sig in &set.sigs {
            let _ = mapped.insert(*sig, &set.failed_participants);
        }
        mapped
    }
}

impl DkgFailChecker {
    ///
    pub(crate) fn new() -> Self {
        Self {
            sigs: CFSet::new(),
            failed_participants: DashSet::new(),
        }
    }

    /// data transfer object
    pub(crate) fn dto(&self) -> DkgFailureSigSet {
        let mapped = DkgFailureSigSet {
            sigs: self
                .sigs
                .values()
                .into_iter()
                .map(|s| s.as_ref().clone())
                .collect(),
            failed_participants: self
                .failed_participants
                .iter()
                .map(|r| (*r))
                .collect::<BTreeSet<_>>(),
        };
        mapped
    }
}

impl DkgFailureSigSetUtils for DkgFailChecker {
    // Insert a signature into this set. The signature is assumed valid. Returns `true` if the signature was
    // not already present in the set and `false` otherwise.
    fn insert(&self, sig: DkgFailureSig, failed_participants: &BTreeSet<XorName>) -> bool {
        if self.failed_participants.is_empty() {
            failed_participants.iter().for_each(|key| {
                let _ = self.failed_participants.insert(*key);
            });
        }
        if self
            .sigs
            .all(|existing_sig| existing_sig.public_key != sig.public_key)
        {
            let _ = self.sigs.push(sig);
            true
        } else {
            false
        }
    }

    // Check whether we have enough signatures to reach agreement on the failure. The contained signatures
    // are assumed valid.
    fn has_agreement(&self, elder_candidates: &ElderCandidates) -> bool {
        has_failure_agreement(elder_candidates.elders.len(), self.sigs.len())
    }

    fn verify(&self, elder_candidates: &ElderCandidates, generation: u64) -> bool {
        let hash = hashed_failure(
            &DkgKey::new(elder_candidates, generation),
            &self
                .failed_participants
                .iter()
                .map(|r| *r)
                .collect::<BTreeSet<_>>(),
        );
        let votes = self.sigs.count(|sig| {
            elder_candidates
                .elders
                .contains_key(&ed25519::name(&sig.public_key))
                && sig.public_key.verify(&hash, &sig.signature).is_ok()
        });

        has_failure_agreement(elder_candidates.elders.len(), votes)
    }
}

// Check whether we have enough signeds to reach agreement on the failure. We only need
// `N - supermajority(N) + 1` signeds, because that already makes a supermajority agreement on a
// successful outcome impossible.
fn has_failure_agreement(num_participants: usize, num_votes: usize) -> bool {
    num_votes > num_participants - supermajority(num_participants)
}

// Create a value whose signature serves as the signed that a failure of a DKG session with the given
// `dkg_key` was observed.
fn hashed_failure(dkg_key: &DkgKey, failed_participants: &BTreeSet<XorName>) -> Digest256 {
    let mut hasher = Sha3::v256();
    let mut hash = Digest256::default();
    hasher.update(&dkg_key.hash);
    hasher.update(&dkg_key.generation.to_le_bytes());
    for name in failed_participants.iter() {
        hasher.update(&name.0);
    }
    hasher.update(b"failure");
    hasher.finalize(&mut hash);
    hash
}
