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

    /// Decrypts the derivation index cipher using the reward secret, then derives the secret key
    /// that was used to decrypt the amount cipher, giving the RevealedAmount containing amount and blinding factor.
    /// Returns the public key of the derived secret, and the revealed amount.
    pub fn decrypt(
        &self,
        elder_reward_secret: &bls::SecretKey,
    ) -> Result<(bls::PublicKey, RevealedAmount)> {
        let derivation_index = self.decrypt_derivation_index(elder_reward_secret)?;
        let owner_base = Owner::from(elder_reward_secret.clone());

        let derived_sk = owner_base.derive(&derivation_index).secret_key()?;
        let derived_pk = derived_sk.public_key();
        let amount = RevealedAmount::try_from((&derived_sk, &self.amount))?;

        Ok((derived_pk, amount))
    }

    /// The derivation index is encrypted to the well-known Elder reward key.
    /// The key which can be derived from the Elder reward key using that index, is then used to decrypt the amount cihper.
    fn decrypt_derivation_index(
        &self,
        elder_reward_secret: &bls::SecretKey,
    ) -> Result<DerivationIndex> {
        let bytes = elder_reward_secret
            .decrypt(&self.derivation_index)
            .ok_or(sn_dbc::Error::DecryptionBySecretKeyFailed)?;

        let mut index = [0u8; 32];
        index.copy_from_slice(&bytes[0..32]);

        Ok(index)
    }
}
