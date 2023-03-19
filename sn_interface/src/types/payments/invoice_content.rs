// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_dbc::{rng::thread_rng, AmountSecrets, Ciphertext, Commitment, Hash, PublicKey, Token};

use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Sha3};

/// Represents data fields of an Invoice.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct InvoiceContent {
    pub amount_commitment: Commitment,
    pub amount_secrets_cipher: Ciphertext,
    pub seller_public_key: PublicKey, // Owner's well-known key.  must match key Dbc.owner_base().public_key
}

impl InvoiceContent {
    /// Create InvoiceContent from the amount of the invoice, the buyer and seller public keys.
    /// The buyer public key is used to encrypt the amount.
    pub fn new(amount: Token, buyer_public_key: &PublicKey, seller_public_key: PublicKey) -> Self {
        let mut rng = thread_rng();

        let amount_secrets = AmountSecrets::from_amount(amount.as_nano(), &mut rng);
        let amount_commitment = amount_secrets.commitment();
        let amount_secrets_cipher = amount_secrets.encrypt(buyer_public_key);

        Self {
            amount_commitment,
            amount_secrets_cipher,
            seller_public_key,
        }
    }

    /// Represent as byte array.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Default::default();
        v.extend(&self.amount_commitment.compress().to_bytes());
        v.extend(&self.amount_secrets_cipher.to_bytes());
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

    /// Checks if the provided AmountSecrets matches the amount commitment.
    /// Note that both the amount and blinding_factor must be correct.
    pub fn matches_commitment(&self, amount: &AmountSecrets) -> bool {
        self.amount_commitment == amount.commitment()
    }
}
