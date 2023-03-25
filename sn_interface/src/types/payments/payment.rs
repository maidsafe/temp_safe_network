// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, Result};

use sn_dbc::{BlindedAmount, Dbc, Hash, PublicKey};

use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Sha3};

/// A payment is a list of Dbcs.
#[derive(Clone, Deserialize, Serialize)]
pub struct Payment {
    pub dbcs: Vec<Dbc>,
}

impl Payment {
    /// The hash of a payment is the hash of all constituent dbcs.
    pub fn hash(&self) -> Hash {
        let mut sha3 = Sha3::v256();
        for dp in self.dbcs.iter() {
            sha3.update(dp.hash().as_ref());
        }
        let mut hash = [0u8; 32];
        sha3.finalize(&mut hash);
        Hash::from(hash)
    }

    /// Retrieve sum of blinded amounts for Dbcs derived from buyer_public_key.
    pub fn sum_by_owner(&self, buyer_public_key: &PublicKey) -> Result<BlindedAmount> {
        self.dbcs
            .iter()
            .filter(|d| &d.content.owner_base.public_key() == buyer_public_key)
            .map(|d| {
                d.blinded_amount()
                    .map_err(|_| Error::AmountCommitmentInvalid)
            })
            .sum::<Result<BlindedAmount, _>>()
    }
}
