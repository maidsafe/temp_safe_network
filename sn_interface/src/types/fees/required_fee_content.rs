// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, Result};

use sn_dbc::{Ciphertext, Hash, PublicKey, Token};

use bls::SecretKey;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Sha3};

const AMOUNT_SIZE: usize = std::mem::size_of::<u64>(); // Amount size: 8 bytes (u64)

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RequiredFeeContent {
    /// A simple u64, encrypted to the dbc id of the dbc being spent,
    /// for which a fee is being required.
    pub amount_cipher: Ciphertext,
    // Elder's well-known reward key. Used to derive
    // a dbc id for a new dbc with the fee payment in it.
    // Deriving a dbc id from a public key, means that the holder
    // of the secret key corresponding to this public key, can access
    // the tokens in that dbc.
    pub elder_reward_key: PublicKey,
}

impl RequiredFeeContent {
    /// Create RequiredFeeContent from the fee amount, the id of the dbc to spend, and elder reward public key.
    /// The dbc id is used to encrypt the amount, so that only the holder of the dbc to spend can see the fee amount.
    pub fn new(amount: Token, dbc_id: &PublicKey, elder_reward_key: PublicKey) -> Self {
        let amount_cipher = dbc_id.encrypt(amount.as_nano().to_le_bytes());
        Self {
            amount_cipher,
            elder_reward_key,
        }
    }

    /// Represent as byte array.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Default::default();
        v.extend(&self.amount_cipher.to_bytes());
        v.extend(&self.elder_reward_key.to_bytes());
        v
    }

    /// Generate hash.
    pub fn hash(&self) -> Hash {
        let mut sha3 = Sha3::v256();
        sha3.update(&self.to_bytes());
        let mut hash = [0; 32];
        sha3.finalize(&mut hash);
        Hash::from(hash)
    }

    /// Decrypts the amount using the secret key of the dbc to spend.
    pub fn decrypt_amount(&self, dbc_secret_key: &SecretKey) -> Result<Token> {
        let bytes = dbc_secret_key
            .decrypt(&self.amount_cipher)
            .ok_or(Error::AmountDecryptionFailed)?;
        let amount = u64::from_le_bytes({
            let mut b = [0u8; AMOUNT_SIZE];
            b.copy_from_slice(&bytes[0..AMOUNT_SIZE]);
            b
        });
        Ok(Token::from_nano(amount))
    }
}
