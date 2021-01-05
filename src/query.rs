// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{data::DataQuery, transfer::TransferQuery, AuthorisationKind, QueryResponse};
use crate::Error;
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// TODO: docs
#[allow(clippy::large_enum_variant)]
#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Query {
    ///
    Data(DataQuery),
    ///
    Transfer(TransferQuery),
}

impl Query {
    /// Returns the type of authorisation needed for the request.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        use Query::*;
        match self {
            Data(q) => q.authorisation_kind(),
            Transfer(q) => q.authorisation_kind(),
        }
    }

    /// Creates a Response containing an error, with the Response variant corresponding to the
    /// Request variant.
    pub fn error(&self, error: Error) -> QueryResponse {
        use Query::*;
        match self {
            Data(q) => q.error(error),
            Transfer(q) => q.error(error),
        }
    }

    /// Returns the address of the destination for `request`.
    pub fn dst_address(&self) -> XorName {
        use Query::*;
        match self {
            Data(q) => q.dst_address(),
            Transfer(q) => q.dst_address(),
        }
    }
}
