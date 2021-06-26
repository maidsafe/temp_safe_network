// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::NodeMsg;
use crate::messaging::DstLocation;
use bls::PublicKey as BlsPublicKey;
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Section-source message without signature and proof.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct PlainMessage {
    /// Name in the source section.
    pub src: XorName,
    /// Destination location.
    pub dst: DstLocation,
    /// The latest key of the destination section according to the sender's knowledge.
    pub dst_key: BlsPublicKey,
    /// Message body.
    pub variant: NodeMsg,
}
