// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, Result};
use serde::{Deserialize, Serialize};

/// Wrapper for raw `bincode::serialise`.
pub(crate) fn serialise<T: Serialize>(data: &T) -> Result<Vec<u8>> {
    bincode::serialize(data).map_err(|err| Error::Serialize(err.as_ref().to_string()))
}

/// Wrapper for `bincode::deserialize`.
pub(crate) fn deserialise<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    bincode::deserialize(bytes).map_err(|err| Error::Deserialize(err.as_ref().to_string()))
}
