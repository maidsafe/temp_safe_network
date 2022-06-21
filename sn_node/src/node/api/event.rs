// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_interface::messaging::{
    data::ServiceMsg, system::SystemMsg, AuthorityProof, DstLocation, EndUser, MsgId, ServiceAuth,
    SrcLocation,
};

use bls::PublicKey as BlsPublicKey;
use ed25519_dalek::Keypair;
use std::{collections::BTreeSet, sync::Arc};
use xor_name::{Prefix, XorName};

/// Node-internal events raised by a [`Node`] via its event sender.
#[allow(clippy::large_enum_variant)]
#[derive(custom_debug::Debug)]
pub enum Event {
    ///
    Data(DataEvent),
    ///
    Messaging(MessagingEvent),
    ///
    Membership(MembershipEvent),
}

///
//#[derive(custom_debug::Debug)]
#[derive(Debug)]
pub enum DataEvent {}

///
//#[derive(custom_debug::Debug)]
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum MessagingEvent {
    /// Received a msg from another Node.
    SystemMsgReceived {
        /// The msg ID
        msg_id: MsgId,
        /// Source location
        src: SrcLocation,
        /// Destination location
        dst: DstLocation,
        /// The msg.
        msg: Box<SystemMsg>,
    },
    /// Received a msg from a client.
    ServiceMsgReceived {
        /// The msg ID
        msg_id: MsgId,
        /// The content of the msg.
        msg: Box<ServiceMsg>,
        /// Data authority
        auth: AuthorityProof<ServiceAuth>,
        /// The end user that sent the msg.
        /// Its xorname is derived from the client public key,
        /// and the socket_id maps against the actual socketaddr
        user: EndUser,
        /// DstLocation for the msg
        dst_location: DstLocation,
    },
}

///
#[derive(custom_debug::Debug)]
pub enum MembershipEvent {
    /// Join occured during section churn and new elders missed it,
    /// therefore the node is not a section member anymore, it needs to rejoin the network.
    ChurnJoinMissError,
    /// A new peer joined our section.
    MemberJoined {
        /// Name of the node
        name: XorName,
        /// Previous name before relocation or `None` if it is a new node.
        previous_name: Option<XorName>,
        /// Age of the node
        age: u8,
    },
    /// A node left our section.
    MemberLeft {
        /// Name of the node
        name: XorName,
        /// Age of the node
        age: u8,
    },
    /// The set of elders in our section has changed.
    EldersChanged {
        /// The Elders of our section.
        elders: Elders,
        /// Promoted, demoted or no change?
        self_status_change: NodeElderChange,
    },
    /// Notify the current list of adult nodes, in case of churning.
    AdultsChanged {
        /// Remaining Adults in our section.
        remaining: BTreeSet<XorName>,
        /// New Adults in our section.
        added: BTreeSet<XorName>,
        /// Removed Adults in our section.
        removed: BTreeSet<XorName>,
    },
    /// Our section has split.
    SectionSplit {
        /// The Elders of our section.
        elders: Elders,
        /// Promoted, demoted or no change?
        self_status_change: NodeElderChange,
    },
    /// This node has started relocating to other section. Will be followed by
    /// `Relocated` when the node finishes joining the destination section.
    RelocationStarted {
        /// Previous name before relocation
        previous_name: XorName,
    },
    /// This node has completed relocation to other section.
    Relocated {
        /// Old name before the relocation.
        previous_name: XorName,
        /// New keypair to be used after relocation.
        #[debug(skip)]
        new_keypair: Arc<Keypair>,
    },
}

/// A flag in EldersChanged event, indicating
/// whether the node got promoted, demoted or did not change.
#[derive(Debug)]
pub enum NodeElderChange {
    /// The node was promoted to Elder.
    Promoted,
    /// The node was demoted to Adult.
    Demoted,
    /// There was no change to the node.
    None,
}

/// Bound name of elders and section_key, section_prefix info together.
#[derive(Debug, Clone, PartialEq)]
pub struct Elders {
    /// The prefix of the section.
    pub prefix: Prefix,
    /// The BLS public key of a section.
    pub key: BlsPublicKey,
    /// Remaining Elders in our section.
    pub remaining: BTreeSet<XorName>,
    /// New Elders in our section.
    pub added: BTreeSet<XorName>,
    /// Removed Elders in our section.
    pub removed: BTreeSet<XorName>,
}
