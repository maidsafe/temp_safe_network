// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod errors;

use crate::{client::Error as ClientError, client::MessageId, MessageType, WireMsg};
use bytes::Bytes;
pub use errors::Error;
use serde::{Deserialize, Serialize};
use sn_data_types::ReplicaPublicKeySet;
use std::{collections::BTreeMap, net::SocketAddr};
use xor_name::{Prefix, XorName};

/// Message to query the network infrastructure.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Message {
    /// Message to request information about the section that matches the given name.
    GetSectionRequest(XorName),
    /// Response to `GetSectionRequest`.
    GetSectionResponse(GetSectionResponse),
    /// Error related to section infrastructure
    InfrastructureError(ErrorResponse),
}

/// All the info a client needs about their section
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct InfrastructureInformation {
    /// Prefix of the section.
    pub prefix: Prefix,
    /// Public key set of the section.
    pub pk_set: ReplicaPublicKeySet,
    /// Section elders.
    pub elders: BTreeMap<XorName, SocketAddr>,
}
// Infrastructure error wrapper to add correltion info for triggering message
#[derive(Debug, Serialize, Deserialize, Hash, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct ErrorResponse {
    /// Optional correlation id if this messge is in response to some non infra query/cmd
    pub correlation_id: MessageId,
    /// Section data error message
    pub error: Error,
}

/// Information about a section.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum GetSectionResponse {
    /// Successful response to `GetSectionRequest`. Contains information about the requested
    /// section.
    Success(InfrastructureInformation),
    /// Response to `GetSectionRequest` containing addresses of nodes that are closer to the
    /// requested name than the recipient. The request should be repeated to these addresses.
    Redirect(Vec<SocketAddr>),
    /// Request could not be fulfilled due to error related to section infrastructure
    SectionInfrastructureError(Error),
}

impl Message {
    /// Convinience function to deserialize a 'Query' from bytes received over the wire.
    /// It returns an error if the bytes don't correspond to an infrastructure query.
    pub fn from(bytes: Bytes) -> crate::Result<Self> {
        let deserialized = WireMsg::deserialize(bytes)?;
        if let MessageType::InfrastructureMessage(query) = deserialized {
            Ok(query)
        } else {
            Err(crate::Error::FailedToParse(
                "bytes as an infrastructure query message".to_string(),
            ))
        }
    }

    /// serialize this Query into bytes ready to be sent over the wire.
    pub fn serialize(&self) -> crate::Result<Bytes> {
        WireMsg::serialize_infrastructure_msg(self)
    }
}
