// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{MessageType, SectionAuthorityProvider, WireMsg};
use crate::types::PublicKey;
use bls::PublicKey as BlsPublicKey;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use xor_name::XorName;

/// Messages for exchanging network info, specifically on a target section for a msg.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SectionInfoMsg {
    /// SectionInfoMsg to request information about the section that matches the given name.
    GetSectionQuery(PublicKey),
    /// Response to `GetSectionQuery`.
    GetSectionResponse(GetSectionResponse),
}

/// Information about a section.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum GetSectionResponse {
    /// Successful response to `GetSectionQuery`. Contains information about the requested
    /// section.
    Success(SectionAuthorityProvider),
    /// Response to `GetSectionQuery` containing addresses of nodes that are closer to the
    /// requested name than the recipient. The request should be repeated to these addresses.
    Redirect(SectionAuthorityProvider),
}

impl SectionInfoMsg {
    /// Convenience function to deserialize a 'Query' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to a network info query.
    pub fn from(bytes: Bytes) -> crate::messaging::Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        if let MessageType::SectionInfo { msg, .. } = deserialized {
            Ok(msg)
        } else {
            Err(crate::messaging::Error::FailedToParse(
                "bytes as a network info message".to_string(),
            ))
        }
    }

    /// serialize this Query into bytes ready to be sent over the wire.
    pub fn serialize(
        &self,
        dst: XorName,
        dst_section_pk: BlsPublicKey,
    ) -> crate::messaging::Result<Bytes> {
        unimplemented!();
        //WireMsg::serialize_section_info_msg(self, dst, dst_section_pk)
    }
}
