// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{MessageType, WireMsg};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::SocketAddr};
use threshold_crypto::PublicKey;
use xor_name::{Prefix, XorName};

/// Message to query the network infrastructure.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Query {
    /// Message to request information about the section that matches the given name.
    GetSectionRequest(XorName),
    /// Response to `GetSectionRequest`.
    GetSectionResponse(GetSectionResponse),
}

/// Information about a section.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum GetSectionResponse {
    /// Successful response to `GetSectionRequest`. Contains information about the requested
    /// section.
    Success {
        /// Prefix of the section.
        prefix: Prefix,
        /// Public key of the section.
        key: PublicKey,
        /// Section elders.
        elders: BTreeMap<XorName, SocketAddr>,
    },
    /// Response to `GetSectionRequest` containing addresses of nodes that are closer to the
    /// requested name than the recipient. The request should be repeated to these addresses.
    Redirect(Vec<SocketAddr>),
}

impl Query {
    /// Convinience function to deserialise a 'Query' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to an infrastructure query.
    pub fn from(bytes: Bytes) -> crate::Result<Self> {
        let deserialised = WireMsg::deserialise(bytes)?;
        if let MessageType::InfrastructureQuery(query) = deserialised {
            Ok(query)
        } else {
            Err(crate::Error::FailedToParse(
                "bytes as an infrastructure query message".to_string(),
            ))
        }
    }

    /// Serialise this Query into bytes ready to be sent over the wire.
    pub fn serialise(&self) -> crate::Result<Bytes> {
        WireMsg::serialise_infrastructure_query(self)
    }
}
