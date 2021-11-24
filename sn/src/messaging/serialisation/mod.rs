// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod wire_msg;
mod wire_msg_header;

use xor_name::XorName;

// highest prio as we can't do anything until we've joined
pub(crate) const JOIN_RESPONSE_PRIORITY: i32 = 4;
pub(crate) const DKG_MSG_PRIORITY: i32 = 3;
pub(crate) const AE_MSG_PRIORITY: i32 = 2;
pub(crate) const INFRASTRUCTURE_MSG_PRIORITY: i32 = 1;
pub(crate) const NODE_DATA_MSG_PRIORITY: i32 = 0;
pub(crate) const SERVICE_MSG_PRIORITY: i32 = -2;

use crate::types::PublicKey;

pub use self::wire_msg::WireMsg;
use super::{
    data::ServiceMsg, system::SystemMsg, AuthorityProof, BlsShareAuth, DstLocation, MessageId,
    NodeAuth, SectionAuth, ServiceAuth,
};

/// Type of message.
/// Note this is part of this crate's public API but this enum is
/// never serialised or even part of the message that is sent over the wire.
#[derive(PartialEq, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum MessageType {
    /// Service message
    Service {
        /// Message ID
        msg_id: MessageId,
        /// Requester's authority over this message
        auth: AuthorityProof<ServiceAuth>,
        /// Message destination location
        dst_location: DstLocation,
        /// the message
        msg: ServiceMsg,
    },
    /// System message
    System {
        /// Message ID
        msg_id: MessageId,
        /// Node authority over this message
        msg_authority: NodeMsgAuthority,
        /// Message destination location
        dst_location: DstLocation,
        /// the message
        msg: SystemMsg,
    },
}

impl MessageType {
    /// The priority of the message, when handled by lower level comms.
    pub fn priority(&self) -> i32 {
        match self {
            MessageType::System {
                msg: SystemMsg::JoinResponse(_),
                ..
            }
            | MessageType::System {
                msg: SystemMsg::JoinAsRelocatedResponse(_),
                ..
            } => JOIN_RESPONSE_PRIORITY,
            // DKG messages
            MessageType::System {
                msg: SystemMsg::DkgStart { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::DkgSessionUnknown { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::DkgSessionInfo { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::DkgNotReady { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::DkgRetry { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::DkgMessage { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::DkgFailureObservation { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::DkgFailureAgreement(_),
                ..
            } => DKG_MSG_PRIORITY,

            // Node messages for AE updates
            MessageType::System {
                msg: SystemMsg::AntiEntropyRetry { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::AntiEntropyRedirect { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::AntiEntropyUpdate { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::AntiEntropyProbe(_),
                ..
            } => AE_MSG_PRIORITY,

            MessageType::System {
                msg: SystemMsg::BackPressure(_),
                ..
            }
            | MessageType::System {
                msg: SystemMsg::Relocate(_),
                ..
            }
            | MessageType::System {
                msg: SystemMsg::RelocatePromise(_),
                ..
            }
            | MessageType::System {
                msg: SystemMsg::JoinRequest(_),
                ..
            }
            | MessageType::System {
                msg: SystemMsg::JoinAsRelocatedRequest(_),
                ..
            }
            | MessageType::System {
                msg: SystemMsg::Propose { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::StartConnectivityTest(_),
                ..
            } => INFRASTRUCTURE_MSG_PRIORITY,

            // Inter-node comms related to processing client requests
            MessageType::System {
                msg: SystemMsg::NodeCmd(_),
                ..
            }
            | MessageType::System {
                msg: SystemMsg::NodeQuery(_),
                ..
            }
            | MessageType::System {
                msg: SystemMsg::NodeQueryResponse { .. },
                ..
            }
            | MessageType::System {
                msg: SystemMsg::NodeMsgError { .. },
                ..
            } => NODE_DATA_MSG_PRIORITY,

            // Client<->node service comms
            MessageType::Service { .. } => SERVICE_MSG_PRIORITY,
        }
    }
}

/// Authority of a NodeMsg.
/// Src of message and authority to send it. Authority is validated by the signature.
#[derive(PartialEq, Debug, Clone)]
pub enum NodeMsgAuthority {
    /// Authority of a single peer.
    Node(AuthorityProof<NodeAuth>),
    /// Authority of a single peer that uses it's BLS Keyshare to sign the message.
    BlsShare(AuthorityProof<BlsShareAuth>),
    /// Authority of a whole section.
    Section(AuthorityProof<SectionAuth>),
}

impl NodeMsgAuthority {
    /// Returns the XorName of the authority used for the auth signing
    pub(crate) fn get_auth_xorname(&self) -> XorName {
        match self.clone() {
            NodeMsgAuthority::BlsShare(auth_proof) => {
                let auth = auth_proof.into_inner();
                auth.src_name
            }
            NodeMsgAuthority::Node(auth_proof) => {
                let auth = auth_proof.into_inner();
                let pk = auth.public_key;

                XorName::from(PublicKey::from(pk))
            }
            NodeMsgAuthority::Section(auth_proof) => {
                let auth = auth_proof.into_inner();
                auth.src_name
            }
        }
    }
}
