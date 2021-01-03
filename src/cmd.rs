// Copyright 2021MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{data::DataCmd, transfer::TransferCmd, AuthorisationKind};
use serde::{Deserialize, Serialize};
use sn_data_types::TransferAgreementProof;
use xor_name::XorName;

/// Command messages for data or transfer operations
#[allow(clippy::large_enum_variant)]
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum Cmd {
    /// Commands for manipulating data
    Data {
        /// The data command struct itself
        cmd: DataCmd,
        /// Proof of payment for the data command
        payment: TransferAgreementProof,
    },
    /// Command for transfering safe network tokens
    Transfer(TransferCmd),
}

impl Cmd {
    /// Returns the type of authorisation needed for the cuest.
    pub fn authorisation_kind(&self) -> AuthorisationKind {
        use Cmd::*;
        match self {
            Data { cmd, .. } => cmd.authorisation_kind(),
            Transfer(c) => c.authorisation_kind(),
        }
    }

    /// Returns the address of the destination for `cuest`.
    pub fn dst_address(&self) -> XorName {
        use Cmd::*;
        match self {
            Data { cmd, .. } => cmd.dst_address(),
            Transfer(c) => c.dst_address(),
        }
    }
}
