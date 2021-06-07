// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::signed::{Signed, SignedShare};
use std::{
    collections::HashMap,
    fmt::Debug,
    time::{Duration, Instant},
};
use thiserror::Error;
use threshold_crypto as bls;
use tiny_keccak::{Hasher, Sha3};

/// Default duration since their last modification after which all unaggregated entries expire.
pub const DEFAULT_EXPIRATION: Duration = Duration::from_secs(120);

type Digest256 = [u8; 32];

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
///
pub struct SignatureAggregator {
    map: HashMap<Digest256, State>,
    expiration: Duration,
}

impl SignatureAggregator {
    /// Create new aggregator with default expiration.
    pub fn new() -> Self {
        Self::with_expiration(DEFAULT_EXPIRATION)
    }

    /// Create new aggregator with the given expiration.
    pub fn with_expiration(expiration: Duration) -> Self {
        Self {
            map: Default::default(),
            expiration,
        }
    }

    /// Add new share into the aggregator. If enough valid signature shares were collected, returns
    /// its `Signed` (signature + public key). Otherwise returns error which details why the
    /// aggregation did not succeed yet.
    ///
    /// Note: returned `Error::NotEnoughShares` does not indicate a failure. It simply means more
    /// shares still need to be added for that particular payload. This error could be safely
    /// ignored (it might still be useful perhaps for debugging). The other error variants, however,
    /// indicate failures and should be treated a such. See [Error] for more info.
    pub fn add(&mut self, payload: &[u8], signed_share: SignedShare) -> Result<Signed, Error> {
        self.remove_expired();

        if !signed_share.verify(payload) {
            return Err(Error::InvalidShare);
        }

        // Use the hash of the payload + the public key as the key in the map to avoid mixing
        // entries that have the same payload but are signed using different keys.
        let public_key = signed_share.public_key_set.public_key();

        let mut hasher = Sha3::v256();
        let mut hash = Digest256::default();
        hasher.update(payload);
        hasher.update(&public_key.to_bytes());
        hasher.finalize(&mut hash);

        self.map
            .entry(hash)
            .or_insert_with(State::new)
            .add(signed_share)
            .map(|signature| Signed {
                public_key,
                signature,
            })
    }

    fn remove_expired(&mut self) {
        let expiration = self.expiration;
        self.map
            .retain(|_, state| state.modified.elapsed() < expiration)
    }
}

impl Default for SignatureAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned from SignatureAggregator::add.
#[derive(Debug, Error)]
pub enum Error {
    /// There are not enough signature shares yet, more need to be added. This is not a failure.
    #[error("not enough signature shares")]
    NotEnoughShares,
    /// The signature share being added is invalid. Such share is rejected but the already collected
    /// shares are kept intact. If enough new valid shares are collected afterwards, the
    /// aggregation might still succeed.
    #[error("signature share is invalid")]
    InvalidShare,
    /// The signature combination failed even though there are enough valid signature shares. This
    /// should probably never happen.
    #[error("failed to combine signature shares: {0}")]
    // TODO: add '#[from]` when `threshold_crytpo::Error` implements `std::error::Error`
    Combine(bls::error::Error),
}

struct State {
    shares: HashMap<usize, bls::SignatureShare>,
    modified: Instant,
}

impl State {
    fn new() -> Self {
        Self {
            shares: Default::default(),
            modified: Instant::now(),
        }
    }

    fn add(&mut self, signed_share: SignedShare) -> Result<bls::Signature, Error> {
        if self
            .shares
            .insert(signed_share.index, signed_share.signature_share)
            .is_none()
        {
            self.modified = Instant::now();
        } else {
            // Duplicate share
            return Err(Error::NotEnoughShares);
        }

        if self.shares.len() > signed_share.public_key_set.threshold() {
            let signature = signed_share
                .public_key_set
                .combine_signatures(self.shares.iter().map(|(&index, share)| (index, share)))
                .map_err(Error::Combine)?;
            self.shares.clear();

            Ok(signature)
        } else {
            Err(Error::NotEnoughShares)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use std::thread::sleep;

    #[test]
    fn smoke() {
        let mut rng = thread_rng();
        let threshold = 3;
        let sk_set = bls::SecretKeySet::random(threshold, &mut rng);

        let mut aggregator = SignatureAggregator::default();
        let payload = b"hello";

        // Not enough shares yet
        for index in 0..threshold {
            let signed_share = create_signed_share(&sk_set, index, payload);
            println!("{:?}", signed_share);
            let result = aggregator.add(payload, signed_share);

            match result {
                Err(Error::NotEnoughShares) => (),
                _ => panic!("unexpected result: {:?}", result),
            }
        }

        // Enough shares now
        let signed_share = create_signed_share(&sk_set, threshold, payload);
        let signed = aggregator.add(payload, signed_share).unwrap();

        assert!(signed.verify(payload));

        // Extra shares start another round
        let signed_share = create_signed_share(&sk_set, threshold + 1, payload);
        let result = aggregator.add(payload, signed_share);

        match result {
            Err(Error::NotEnoughShares) => (),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    #[test]
    fn invalid_share() {
        let mut rng = thread_rng();
        let threshold = 3;
        let sk_set = bls::SecretKeySet::random(threshold, &mut rng);

        let mut aggregator = SignatureAggregator::new();
        let payload = b"good";

        // First insert less than threshold + 1 valid shares.
        for index in 0..threshold {
            let signed_share = create_signed_share(&sk_set, index, payload);
            let _ = aggregator.add(payload, signed_share);
        }

        // Then try to insert invalid share.
        let invalid_signed_share = create_signed_share(&sk_set, threshold, b"bad");
        let result = aggregator.add(payload, invalid_signed_share);

        match result {
            Err(Error::InvalidShare) => (),
            _ => panic!("unexpected result: {:?}", result),
        }

        // The invalid share doesn't spoil the aggregation - we can still aggregate once enough
        // valid shares are inserted.
        let signed_share = create_signed_share(&sk_set, threshold + 1, payload);
        let signed = aggregator.add(payload, signed_share).unwrap();
        assert!(signed.verify(payload))
    }

    #[test]
    fn expiration() {
        let mut rng = thread_rng();
        let threshold = 3;
        let sk_set = bls::SecretKeySet::random(threshold, &mut rng);

        let mut aggregator = SignatureAggregator::with_expiration(Duration::from_millis(500));
        let payload = b"hello";

        for index in 0..threshold {
            let signed_share = create_signed_share(&sk_set, index, payload);
            let _ = aggregator.add(payload, signed_share);
        }

        sleep(Duration::from_secs(1));

        // Adding another share does nothing now, because the previous shares expired.
        let signed_share = create_signed_share(&sk_set, threshold, payload);
        let result = aggregator.add(payload, signed_share);

        match result {
            Err(Error::NotEnoughShares) => (),
            _ => panic!("unexpected result: {:?}", result),
        }
    }

    #[test]
    fn repeated_voting() {
        let mut rng = thread_rng();
        let threshold = 3;
        let sk_set = bls::SecretKeySet::random(threshold, &mut rng);

        let mut aggregator = SignatureAggregator::new();

        let payload = b"hello";

        // round 1

        for index in 0..threshold {
            let signed_share = create_signed_share(&sk_set, index, payload);
            assert!(aggregator.add(payload, signed_share).is_err());
        }

        let signed_share = create_signed_share(&sk_set, threshold, payload);
        assert!(aggregator.add(payload, signed_share).is_ok());

        // round 2

        let offset = 2;

        for index in offset..(threshold + offset) {
            let signed_share = create_signed_share(&sk_set, index, payload);
            assert!(aggregator.add(payload, signed_share).is_err());
        }

        let signed_share = create_signed_share(&sk_set, threshold + offset + 1, payload);
        assert!(aggregator.add(payload, signed_share).is_ok());
    }

    fn create_signed_share(
        sk_set: &bls::SecretKeySet,
        index: usize,
        payload: &[u8],
    ) -> SignedShare {
        let sk_share = sk_set.secret_key_share(index);
        SignedShare::new(sk_set.public_keys(), index, &sk_share, &payload)
    }
}
