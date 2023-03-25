// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Result;

use sn_dbc::{DerivationIndex, Owner, RevealedAmount};

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// These are sent with a spend, so that an Elder
/// can verify that the transfer fee is being paid.
///
/// A client asks for the fee for a spend, and an Elder returns
/// a cipher of the amount and a blinding factor, i.e. a `RevealedAmount`.
/// The Client decrypts it and uses the amount and blinding factor to build
/// the payment dbc to the Elder. The amount + blinding factor is then
/// encrypted to a _derived_ key of the Elder reward key.
/// The client also encrypts the derivation index used, to the Elder _reward key_,
/// and sends both the amount + blinding factor cipher and the derivation index cipher
/// to the Elder by including this `FeeCiphers` struct in the spend cmd.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FeeCiphers {
    amount: bls::Ciphertext,
    derivation_index: bls::Ciphertext,
}

impl FeeCiphers {
    pub fn new(amount: bls::Ciphertext, derivation_index: bls::Ciphertext) -> Self {
        Self {
            amount,
            derivation_index,
        }
    }

    /// Decrypts the derivation index cipher, and derives the public key from it.
    pub fn decrypt_derived_key(&self, base_sk: &bls::SecretKey) -> Result<bls::PublicKey> {
        let derivation_index = self.decrypt_derivation_index(base_sk)?;
        let owner_base = Owner::from(base_sk.clone());
        let public_key = owner_base
            .derive(&derivation_index)
            .secret_key()?
            .public_key();
        Ok(public_key)
    }

    /// Decrypts the derivation index cipher, and uses that to decrypt
    /// the amount cipher, and turn that to a blinded amount that is returned.
    pub fn decrypt_revealed_amount(&self, base_sk: &bls::SecretKey) -> Result<RevealedAmount> {
        let derivation_index = self.decrypt_derivation_index(base_sk)?;
        let owner_base = Owner::from(base_sk.clone());
        let derived_sk = owner_base.derive(&derivation_index).secret_key()?;
        Ok(RevealedAmount::try_from((&derived_sk, &self.amount))?)
    }

    fn decrypt_derivation_index(&self, base_sk: &bls::SecretKey) -> Result<DerivationIndex> {
        let bytes = base_sk
            .decrypt(&self.derivation_index)
            .ok_or(sn_dbc::Error::DecryptionBySecretKeyFailed)?;

        let mut index = [0u8; 32];
        index.copy_from_slice(&bytes[0..32]);

        Ok(index)
    }
}
