// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use thiserror::Error;

/// Specialisation of `std::Result` for dbs.
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
#[non_exhaustive]
/// Node error variants.
pub enum Error {
    /// System time error
    #[error("Could not calculate system time")]
    CouldNotCalculateSystemTime,
    /// Error when an operation ID is not supplied when adding a pending request operation.
    #[error("An operation ID must be supplied for a pending request operation.")]
    OpIdNotSupplied(String),
    /// Error when an operation ID is supplied that doesn't apply to this type of issue.
    #[error("An operation ID only applies to pending unfulfilled requests.")]
    UnusedOpIdSupplied(String),
    /// SystemTime error
    #[error(transparent)]
    SysTime(#[from] std::time::SystemTimeError),
}
