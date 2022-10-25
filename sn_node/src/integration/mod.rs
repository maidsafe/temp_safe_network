// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Implementation of the module integration for the SAFE Network.

pub(crate) mod cmds;
pub(super) mod event;
pub(super) mod event_channel;

pub(crate) use self::cmds::Cmd;
pub use self::{
    event::{Event, MembershipEvent, MessagingEvent, NodeElderChange},
    event_channel::EventReceiver,
};

use sn_interface::{
    messaging::{data::ClientMsg, system::NodeMsg},
    types::Peer,
};

use std::collections::BTreeSet;

/// Msg to be sent from a node.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum OutgoingMsg {
    /// Msg to a node.
    Node(NodeMsg),
    /// Msg to a client.
    Client(ClientMsg),
}

/// Peers in the network.
#[derive(Debug, Clone)]
pub enum Peers {
    /// A single peer.
    Single(Peer),
    /// Multiple peers.
    Multiple(BTreeSet<Peer>),
}
