// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use hex_fmt::HexFmt;
use serde::{Deserialize, Serialize};
use std::fmt;
use xor_name::XorName;

/// Constant byte length of `MessageId`.
pub const MESSAGE_ID_LEN: usize = 32;

/// Unique ID for messages.
#[derive(
    Ord, PartialOrd, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Hash, custom_debug::Debug,
)]
pub struct MessageId(#[debug(with = "Self::fmt_bytes")] [u8; MESSAGE_ID_LEN]);

impl MessageId {
    /// Generates a new `MessageId` with random content.
    pub fn new() -> Self {
        // Here we use XorName just as helper to generate a random id
        Self(XorName::random().0)
    }

    /// Convert an XorName into a MessageId
    pub fn from_xor_name(xor_name: XorName) -> Self {
        Self(xor_name.0)
    }

    fn fmt_bytes(bytes: &[u8; MESSAGE_ID_LEN], f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:0.10}", HexFmt(bytes))
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<[u8; MESSAGE_ID_LEN]> for MessageId {
    fn as_ref(&self) -> &[u8; MESSAGE_ID_LEN] {
        &self.0
    }
}
