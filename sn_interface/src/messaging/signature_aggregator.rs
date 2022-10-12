// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{SectionSig, SectionSigShare};
use crate::types::keys::ed25519::Digest256;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use tiny_keccak::{Hasher, Sha3};

/// Aggregator for signature shares for arbitrary payloads.
///
/// This aggregator allows to collect BLS signature shares for some payload one by one until enough
/// of them are collected. At that point it combines them into a full BLS signature of the given
/// payload. It also automatically rejects invalid signature shares and expires entries that did not
/// collect enough signature shares within a given time.
///
/// This aggregator also handles the case when the same payload is signed with a signature share
/// corresponding to a different BLS public key. In that case, the payloads will be aggregated
/// separately. This avoids mixing signature shares created from different curves which would
/// otherwise lead to invalid signature to be produced even though all the shares are valid.
#[derive(Debug, Default)]
pub struct SignatureAggregator {
    /// a map of the hash(payload + bls pubkey) to the signature shares
    map: BTreeMap<Digest256, BTreeSet<SectionSigShare>>,
}

/// AggregatorErrors returned from `SignatureAggregator::add`.
#[derive(Debug, Error)]
pub enum AggregatorError {
    /// The signature share being added is invalid. Such share is rejected but the already collected
    /// shares are kept intact. If enough new valid shares are collected afterwards, the
    /// aggregation might still succeed.
    #[error("signature share is invalid")]
    InvalidSigShare,
    /// The signature combination failed even though there are enough valid signature shares. This
    /// should probably never happen.
    #[error("failed to combine signature shares: {0}")]
    FailedToCombineSigShares(#[from] bls::error::Error),
}

impl SignatureAggregator {
    /// Add new share into the aggregator. If enough valid signature shares were collected, returns
    /// the aggregated signature: `Some(SectionSig)` else returns None.
    /// Checks if the signature are valid
    pub fn try_aggregate(
        &mut self,
        payload: &[u8],
        sig_share: SectionSigShare,
    ) -> Result<Option<SectionSig>, AggregatorError> {
        if !sig_share.verify(payload) {
            return Err(AggregatorError::InvalidSigShare);
        }

        // Use the hash of the payload + the public key as the key in the map to avoid mixing
        // entries that have the same payload but are signed using different keys.
        let public_key = sig_share.public_key_set.public_key();

        let mut hasher = Sha3::v256();
        let mut hash = Digest256::default();
        hasher.update(payload);
        hasher.update(&public_key.to_bytes());
        hasher.finalize(&mut hash);

        // save the sig share
        let current_shares = self.map.entry(hash).or_insert(BTreeSet::new());
        let _ = current_shares.insert(sig_share.clone());

        // try aggregate
        if current_shares.len() > sig_share.public_key_set.threshold() {
            let signature = sig_share
                .public_key_set
                .combine_signatures(
                    current_shares
                        .iter()
                        .map(|s| (s.index, s.signature_share.clone())),
                )
                .map_err(AggregatorError::FailedToCombineSigShares)?;
            let section_sig = SectionSig {
                public_key,
                signature,
            };
            Ok(Some(section_sig))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn smoke() -> Result<(), AggregatorError> {
        let mut rng = thread_rng();
        let threshold = 3;
        let sk_set = bls::SecretKeySet::random(threshold, &mut rng);

        let mut aggregator = SignatureAggregator::default();
        let payload = b"hello";

        // Not enough shares yet
        for index in 0..threshold {
            let sig_share = create_sig_share(&sk_set, index, payload);
            let result = aggregator.try_aggregate(payload, sig_share);

            match result {
                Ok(None) => (),
                _ => panic!("unexpected result: {:?}", result),
            }
        }

        // Enough shares now
        let sig_share = create_sig_share(&sk_set, threshold, payload);
        let sig = aggregator.try_aggregate(payload, sig_share)?;

        assert!(sig.expect("some key").verify(payload));

        // Extra shares start another round
        let sig_share = create_sig_share(&sk_set, threshold + 1, payload);
        let result = aggregator.try_aggregate(payload, sig_share);

        match result {
            Ok(None) => Ok(()),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    #[test]
    fn invalid_share() -> Result<(), AggregatorError> {
        let mut rng = thread_rng();
        let threshold = 3;
        let sk_set = bls::SecretKeySet::random(threshold, &mut rng);

        let mut aggregator = SignatureAggregator::default();
        let payload = b"good";

        // First insert less than threshold + 1 valid shares.
        for index in 0..threshold {
            let sig_share = create_sig_share(&sk_set, index, payload);
            let _keyed_sig = aggregator.try_aggregate(payload, sig_share);
        }

        // Then try to insert invalid share.
        let invalid_sig_share = create_sig_share(&sk_set, threshold, b"bad");
        let result = aggregator.try_aggregate(payload, invalid_sig_share);

        match result {
            Err(AggregatorError::InvalidSigShare) => (),
            _ => panic!("unexpected result: {:?}", result),
        }

        // The invalid share doesn't spoil the aggregation - we can still aggregate once enough
        // valid shares are inserted.
        let sig_share = create_sig_share(&sk_set, threshold + 1, payload);
        let sig = aggregator.try_aggregate(payload, sig_share)?;
        assert!(sig.expect("some key").verify(payload));

        Ok(())
    }

    #[test]
    fn repeated_voting() {
        let mut rng = thread_rng();
        let threshold = 3;
        let sk_set = bls::SecretKeySet::random(threshold, &mut rng);

        let mut aggregator = SignatureAggregator::default();

        let payload = b"hello";

        // round 1

        for index in 0..threshold {
            let sig_share = create_sig_share(&sk_set, index, payload);
            assert!(aggregator.try_aggregate(payload, sig_share).is_err());
        }

        let sig_share = create_sig_share(&sk_set, threshold, payload);
        assert!(aggregator.try_aggregate(payload, sig_share).is_ok());

        // round 2

        let offset = 2;

        for index in offset..(threshold + offset) {
            let sig_share = create_sig_share(&sk_set, index, payload);
            assert!(aggregator.try_aggregate(payload, sig_share).is_err());
        }

        let sig_share = create_sig_share(&sk_set, threshold + offset + 1, payload);
        assert!(aggregator.try_aggregate(payload, sig_share).is_ok());
    }

    fn create_sig_share(
        sk_set: &bls::SecretKeySet,
        index: usize,
        payload: &[u8],
    ) -> SectionSigShare {
        let sk_share = sk_set.secret_key_share(index);
        SectionSigShare::new(sk_set.public_keys(), index, &sk_share, payload)
    }
}
