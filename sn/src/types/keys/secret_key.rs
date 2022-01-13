// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Module providing keys, keypairs, and signatures.
//!
//! The easiest way to get a `PublicKey` is to create a random `Keypair` first through one of the
//! `new` functions. A `PublicKey` can't be generated by itself; it must always be derived from a
//! secret key.

use super::super::{Error, Result};
use bls::{self, serde_impl::SerdeSecret};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};
// TODO: remove clones. We need to restructure to hold keypair ones and only require references for this.
/// Wrapper for different secret key types.
#[derive(Debug, Serialize, Deserialize)]
pub enum SecretKey {
    /// Ed25519 secretkey.
    Ed25519(ed25519_dalek::SecretKey),
    /// BLS secretkey share.
    BlsShare(SerdeSecret<bls::SecretKeyShare>),
}

impl SecretKey {
    /// Construct a secret key from a hex string
    ///
    /// Similar to public key, it is often useful in user
    /// facing apps to be able to set your own secret
    /// key without depending on both the ed25519_dalek
    /// and hex crates just to reimplement this function
    pub fn ed25519_from_hex(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex).map_err(|err| {
            Error::FailedToParse(format!(
                "Couldn't parse edd25519 secret key bytes from hex: {}",
                err
            ))
        })?;
        let ed25519_sk = ed25519_dalek::SecretKey::from_bytes(bytes.as_ref()).map_err(|err| {
            Error::FailedToParse(format!(
                "Couldn't parse ed25519 secret key from bytes: {}",
                err
            ))
        })?;
        Ok(Self::Ed25519(ed25519_sk))
    }
}

impl Display for SecretKey {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}
