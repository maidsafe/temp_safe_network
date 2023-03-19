// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    errors::{Error, Result},
    InvoiceContent,
};

use bls::SecretKey;
use sn_dbc::{Hash, PublicKey, Signature, Token};

use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Sha3};

/// A seller issues an Invoice thereby commiting to a price.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Invoice {
    pub content: InvoiceContent,
    pub seller_signature: Signature,
}

impl Invoice {
    /// Create an invoice by encrypting the amount to the buyers key, and signing
    /// it all with the seller secret key.
    pub fn new(amount: Token, buyer_public_key: &PublicKey, seller_secret_key: &SecretKey) -> Self {
        let content = InvoiceContent::new(amount, buyer_public_key, seller_secret_key.public_key());
        let seller_signature = seller_secret_key.sign(content.to_bytes());
        Self {
            content,
            seller_signature,
        }
    }

    /// Verifies that seller_signature is correct.
    pub fn verify(&self) -> Result<()> {
        let valid = self
            .content
            .seller_public_key
            .verify(&self.seller_signature, self.content.to_bytes());

        match valid {
            true => Ok(()),
            false => Err(Error::InvoiceSignatureInvalid),
        }
    }

    /// Represent Invoice as bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Default::default();
        v.extend(&self.content.to_bytes());
        v.extend(&self.seller_signature.to_bytes());
        v
    }

    /// Generate hash of Invoice.
    pub fn hash(&self) -> Hash {
        let mut sha3 = Sha3::v256();
        sha3.update(&self.to_bytes());
        let mut hash = [0; 32];
        sha3.finalize(&mut hash);
        Hash::from(hash)
    }
}
