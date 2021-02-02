// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use serde::{Deserialize, Serialize};
use std::fmt;
use tiny_keccak::{Hasher, Sha3};
use xor_name::XorName;

/// Unique ID for messages.
///
/// This is used for deduplication: Since the network sends messages redundantly along different
/// routes, the same message will usually arrive more than once at any given node. A message with
/// an ID that is already in the cache will be ignored.
#[derive(Ord, PartialOrd, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct MessageId(pub XorName);
impl MessageId {
    /// Generates a new `MessageId` with random content.
    pub fn new() -> Self {
        Self(XorName::random())
    }

    /// Generates a new based on provided id.
    pub fn in_response_to(src: &MessageId) -> MessageId {
        let mut hash_bytes = Vec::new();
        let src = src.0;
        hash_bytes.extend_from_slice(&src.0);

        let mut hasher = Sha3::v256();
        let mut output = [0; 32];
        hasher.update(&hash_bytes);
        hasher.finalize(&mut output);

        MessageId(XorName(output))
    }

    /// Generates a new based on provided sources.
    pub fn combine(srcs: Vec<XorName>) -> MessageId {
        let mut hash_bytes = Vec::new();
        for src in srcs.into_iter() {
            hash_bytes.extend_from_slice(&src.0);
        }

        let mut hasher = Sha3::v256();
        let mut output = [0; 32];
        hasher.update(&hash_bytes);
        hasher.finalize(&mut output);

        MessageId(XorName(output))
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
