// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::ToDbKey;
use serde::{de::DeserializeOwned, Serialize};

pub(crate) trait Key: ToDbKey + PartialEq + Eq + DeserializeOwned {}

/// A value in a key-value store.
///
/// The KV-store is paramaterised by the value type, from which a key can be derived.
pub(crate) trait Value: Serialize + DeserializeOwned {
    type Key: Key;
    fn key(&self) -> &Self::Key;
}
