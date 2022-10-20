// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Module providing keys, keypairs, and signatures.
//!
//! The easiest way to get a `PublicKey` is to create a random `Keypair` first through one of the
//! `new` functions. A `PublicKey` can't be generated by itself; it must always be derived from a
//! secret key.

pub mod ed25519;
pub(super) mod keypair;
pub(super) mod node_keypairs;
pub(crate) mod public_key;
pub(super) mod secret_key;
pub(super) mod signature;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils {
    use crate::{
        messaging::system::{SectionSig, SectionSigned},
        network_knowledge::{SectionAuthUtils, SectionKeyShare},
    };
    use serde::Serialize;

    /// bls key related test utilities
    pub struct TestKeys {}

    impl TestKeys {
        /// Create `bls::Signature` for the given payload using the provided `bls::SecretKey`
        pub fn sign<T: Serialize>(secret_key: &bls::SecretKey, payload: &T) -> bls::Signature {
            let bytes = bincode::serialize(payload).expect("Failed to serialize payload");
            Self::sign_bytes(secret_key, &bytes)
        }
        /// Create `bls::Signature` for the given bytes using the provided `bls::SecretKey`
        pub fn sign_bytes(secret_key: &bls::SecretKey, bytes: &[u8]) -> bls::Signature {
            secret_key.sign(bytes)
        }

        /// Create `SectionSig` for the given bytes using the provided `bls::SecretKey`
        pub fn get_section_sig_bytes(secret_key: &bls::SecretKey, bytes: &[u8]) -> SectionSig {
            SectionSig {
                public_key: secret_key.public_key(),
                signature: Self::sign_bytes(secret_key, bytes),
            }
        }

        /// Create `SectionSig` for the given payload using the provided `bls::SecretKey`
        pub fn get_section_sig<T: Serialize>(
            secret_key: &bls::SecretKey,
            payload: &T,
        ) -> SectionSig {
            let bytes = bincode::serialize(payload).expect("Failed to serialize payload");
            Self::get_section_sig_bytes(secret_key, &bytes)
        }

        /// Create signature for the given payload using the provided `bls::SecretKey` and
        /// wrap them using `SectionSigned`
        pub fn get_section_signed<T: Serialize>(
            secret_key: &bls::SecretKey,
            payload: T,
        ) -> SectionSigned<T> {
            let sig = Self::get_section_sig(secret_key, &payload);
            SectionSigned::new(payload, sig)
        }

        /// Generate a `SectionKeyShare` from the `bls::SecretKeySet` and given index
        pub fn get_section_key_share(sk_set: &bls::SecretKeySet, index: usize) -> SectionKeyShare {
            SectionKeyShare {
                public_key_set: sk_set.public_keys(),
                index,
                secret_key_share: sk_set.secret_key_share(index),
            }
        }
    }
}
