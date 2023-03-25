// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_dbc::{
    rng::thread_rng, BlindedAmount, Ciphertext, Hash, PedersenGens, PublicKey, RevealedAmount,
    Token,
};

use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Sha3};

/// Represents data fields of an Invoice.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct InvoiceContent {
    pub blinded_amount: BlindedAmount,
    pub revealed_amount_cipher: Ciphertext,
    pub seller_public_key: PublicKey, // Owner's well-known key.  must match key Dbc.owner_base().public_key
}

impl InvoiceContent {
    /// Create InvoiceContent from the amount of the invoice, the buyer and seller public keys.
    /// The buyer public key is used to encrypt the amount.
    pub fn new(amount: Token, buyer_public_key: &PublicKey, seller_public_key: PublicKey) -> Self {
        let revealed_amount = RevealedAmount::from_amount(amount.as_nano(), thread_rng());
        let blinded_amount = revealed_amount.blinded_amount(&PedersenGens::default());
        let revealed_amount_cipher = revealed_amount.encrypt(buyer_public_key);
        Self {
            blinded_amount,
            revealed_amount_cipher,
            seller_public_key,
        }
    }

    /// Represent as byte array.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Default::default();
        v.extend(&self.blinded_amount.compress().to_bytes());
        v.extend(&self.revealed_amount_cipher.to_bytes());
        v.extend(&self.seller_public_key.to_bytes());
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

    /// Checks if the blinded amount from the provided revealed amount
    /// equals the blinded amount of the invoice.
    pub fn amount_equals(&self, amount: &RevealedAmount) -> bool {
        self.blinded_amount == amount.blinded_amount(&PedersenGens::default())
    }
}
