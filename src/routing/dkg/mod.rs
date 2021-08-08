// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod dkg_msgs_utils;
pub(super) mod proposal;
mod section_signed;
mod session;
#[cfg(test)]
pub(crate) mod test_utils;
mod voter;

pub(crate) use self::{
    dkg_msgs_utils::{DkgFailChecker, DkgFailureSigSetUtils, DkgKeyUtils},
    proposal::{ProposalAggregator, ProposalError, ProposalUtils},
    voter::DkgVoter,
};
pub(crate) use crate::messaging::node::{KeyedSig, SigShare};
pub(super) use section_signed::SectionAuthUtils;
use serde::Serialize;

/// All the key material needed to sign or combine signature for our section key.
#[derive(custom_debug::Debug)]
pub(crate) struct SectionDkgOutcome {
    /// Public key set to verify threshold signatures and combine shares.
    public_key_set: bls::PublicKeySet,
    /// Index of the owner of this key share within the set of all section elders.
    index: usize,
    /// Secret Key share.
    #[debug(skip)]
    secret_key_share: bls::SecretKeyShare,
}

impl SectionDkgOutcome {
    ///
    pub(crate) fn consume(self) -> (usize, bls::PublicKeySet, bls::SecretKeyShare) {
        let SectionDkgOutcome {
            index,
            public_key_set,
            secret_key_share,
        } = self;

        (index, public_key_set, secret_key_share)
    }

    ///
    pub(crate) fn new(
        public_key_set: bls::PublicKeySet,
        index: usize,
        secret_key_share: bls::SecretKeyShare,
    ) -> Self {
        Self {
            public_key_set,
            index,
            secret_key_share: secret_key_share,
        }
    }

    // ///
    // pub(crate) fn index(&self) -> &usize {
    //     &self.index
    // }

    ///
    pub(crate) fn public_key(&self) -> bls::PublicKey {
        self.public_key_set.public_key()
    }
}

// Verify the integrity of `message` against `sig`.
pub(crate) fn verify_sig<T: Serialize>(sig: &KeyedSig, message: &T) -> bool {
    bincode::serialize(message)
        .map(|bytes| sig.verify(&bytes))
        .unwrap_or(false)
}
