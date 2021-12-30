// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{encoding::serialise, Result};
use crate::types::{ChunkAddress, Keypair, PublicKey, RegisterAddress};
use serde::Serialize;
use xor_name::XorName;

pub(crate) trait ToDbKey: Serialize {
    /// The encoded string representation of an identifier, used as a key in the context of a
    /// Db <key,value> store.
    fn to_db_key(&self) -> Result<String> {
        let serialised = serialise(&self)?;
        Ok(hex::encode(&serialised))
    }
}

impl ToDbKey for RegisterAddress {}
impl ToDbKey for Keypair {}
impl ToDbKey for ChunkAddress {}
impl ToDbKey for PublicKey {}
impl ToDbKey for XorName {}
