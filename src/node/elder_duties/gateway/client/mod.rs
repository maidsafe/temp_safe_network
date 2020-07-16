// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use self::onboarding::Onboarding;
use crate::{cmd::MessagingDuty, utils};
use bytes::Bytes;
use log::{debug, error, info, trace, warn};
use rand::{CryptoRng, Rng};
use routing::Node as Routing;
use safe_nd::{
    Address, Error, HandshakeRequest, HandshakeResponse, Message, MessageId, MsgEnvelope,
    MsgSender, NodePublicId, PublicId, Result, Signature, XorName,
};
use serde::Serialize;
use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct ClientMsg {
    pub client: ClientInfo,
    pub msg: MsgEnvelope,
}

#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub public_id: PublicId,
}

impl Display for ClientInfo {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.public_id.name())
    }
}

pub struct ClientMsgTracking {
    id: NodePublicId,
    onboarding: Onboarding,
    tracked_incoming: HashMap<MessageId, SocketAddr>,
    tracked_outgoing: HashMap<MessageId, MsgEnvelope>,
}

impl ClientMsgTracking {
    pub fn new(id: NodePublicId, onboarding: Onboarding) -> Self {
        Self {
            id,
            tracked_incoming: Default::default(),
            tracked_outgoing: Default::default(),
        }
    }

    /// If 
    pub fn track_incoming(&mut self, msg_id: MessageId, client_address: SocketAddr) -> Option<MessagingDuty> {
        // We could have received a group decision containing a client msg,
        // before receiving the msg from that client directly.
        if let Some(msg) = self.tracked_outgoing.remove(&msg_id) {
            return Some(MessagingDuty::SendToClient { address: client_address, msg });
        }

        if let Entry::Vacant(ve) = self.tracked_incoming.entry(msg_id) {
            let _ = ve.insert(peer_addr);
            None
        } else {
            info!(
                "Pending MessageId {:?} reused - ignoring client message.",
                msg_id
            );
            None
        }
    }

    pub fn match_outgoing(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        match msg.destination() {
            Address::Client { .. } => (),
            _ => {
                error!(
                    "{} for message-id {:?}, Invalid destination.",
                    self,
                    msg.id()
                );
                return None;
                //return Err(Error::InvalidOperation);
            }
        };
        let correlation_id = match msg.message {
            Message::Event { correlation_id, .. }
            | Message::CmdError { correlation_id, .. }
            | Message::QueryResponse { correlation_id, .. } => correlation_id,
            _ => {
                error!(
                    "{} for message-id {:?}, Invalid message for client.",
                    self,
                    msg.id()
                );
                return None;
                //return Err(Error::InvalidOperation);
            }
        };
        let client_address = match self.tracked_incoming.remove(&correlation_id) {
            Some(address) => address,
            None => {
                info!(
                    "{} for message-id {:?}, Unable to find the client to respond to.",
                    self, correlation_id
                );
                let _ = self.pending_actions.insert(correlation_id, msg.clone());
                return None;
                //return Err(Error::NoSuchKey);
            }
        };

        Some(MessagingDuty::SendToClient { address: client_address, msg })
    }

    
    // #[allow(unused)]
    // pub fn notify_client(&mut self, client: &XorName, receipt: &DebitAgreementProof) {
    //     for client_id in self.lookup_client_and_its_apps(client) {
    //         self.send_notification_to_client(&client_id, &TransferNotification(receipt.clone()));
    //     }
    // }

    // pub(crate) fn send_notification_to_client(
    //     &mut self,
    //     client_id: &PublicId,
    //     notification: &TransferNotification,
    // ) {
    //     let peer_addrs = self.lookup_client_peer_addrs(&client_id);

    //     if peer_addrs.is_empty() {
    //         warn!(
    //             "{}: can't notify {} as none of the instances of the client is connected.",
    //             self, client_id
    //         );
    //         return;
    //     };

    //     for peer_addr in peer_addrs {
    //         self.send(
    //             peer_addr,
    //             &Message::TransferNotification {
    //                 payload: notification.clone(),
    //             },
    //         )
    //     }
    // }

    fn lookup_client_peer_addrs(&self, id: &PublicId) -> Vec<SocketAddr> {
        self.clients
            .iter()
            .filter_map(|(peer_addr, client)| {
                if &client.public_id == id {
                    Some(*peer_addr)
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn lookup_client_and_its_apps(&self, name: &XorName) -> Vec<PublicId> {
        self.clients
            .values()
            .filter_map(|client| {
                if client.public_id.name() == name {
                    Some(client.public_id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

}

impl Display for ClientMsgTracking {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
