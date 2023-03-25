// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, RequiredFeeContent, Result};

use sn_dbc::{Hash, PublicKey, Signature, Token};

use bls::SecretKey;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Sha3};

/// An Elder responds to a Client who wishes to spend a dbc,
/// informing the Client of the required fee for the spend.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RequiredFee {
    pub content: RequiredFeeContent,
    pub elder_reward_key_sig: Signature,
}

impl RequiredFee {
    /// Instantiate RequiredFee by encrypting the amount to the id of the dbc to spend, and signing
    /// it all with the Elder reward secret key.
    pub fn new(amount: Token, dbc_id: &PublicKey, elder_reward_key_secret: &SecretKey) -> Self {
        let content = RequiredFeeContent::new(amount, dbc_id, elder_reward_key_secret.public_key());
        let elder_reward_key_sig = elder_reward_key_secret.sign(content.to_bytes());
        Self {
            content,
            elder_reward_key_sig,
        }
    }

    /// Verifies that elder_reward_key_sig is correct.
    pub fn verify(&self) -> Result<()> {
        let valid = self
            .content
            .elder_reward_key
            .verify(&self.elder_reward_key_sig, self.content.to_bytes());

        match valid {
            true => Ok(()),
            false => Err(Error::RequiredFeeSignatureInvalid),
        }
    }

    /// Represent RequiredFee as bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Default::default();
        v.extend(&self.content.to_bytes());
        v.extend(&self.elder_reward_key_sig.to_bytes());
        v
    }

    /// Generate hash of RequiredFee.
    pub fn hash(&self) -> Hash {
        let mut sha3 = Sha3::v256();
        sha3.update(&self.to_bytes());
        let mut hash = [0; 32];
        sha3.finalize(&mut hash);
        Hash::from(hash)
    }
}
