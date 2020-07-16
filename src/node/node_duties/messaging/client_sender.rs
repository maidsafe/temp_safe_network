// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{cmd::MessagingDuty, utils};
pub use client::{ClientInfo, ClientMessaging, ClientMsg};
use log::{error, info};
use routing::{DstLocation, Node as Routing, SrcLocation};
use safe_nd::{Address, MsgEnvelope, XorName};
use std::{cell::RefCell, collections::BTreeSet, rc::Rc, net::SocketAddr};

pub(super) struct ClientSender {
    routing: Rc<RefCell<Routing>>,
}

impl ClientSender {
    pub fn new(routing: Rc<RefCell<Routing>>) -> Self {
        Self { routing }
    }

    pub fn send(&self, recipient: SocketAddress, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        match msg.destination() {
            Address::Node(_) => Some(MessagingDuty::SendToNode(msg)),
            Address::Section(_) => Some(MessagingDuty::SendToSection(msg)),
            Address::Client(_) => self.send_to_client(recipient, msg),
        }
    }

    fn send_to_client(&self, recipient: SocketAddress, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        self.send_any_to_client(recipient, msg)
    }

    fn send_any_to_client<T: Serialize>(&mut self, recipient: SocketAddr, msg: &T) -> Option<MessagingDuty> {
        let msg = utils::serialise(msg);
        let bytes = Bytes::from(msg);

        if let Err(e) = self
            .routing
            .borrow_mut()
            .send_message_to_client(recipient, bytes, 0)
        {
            warn!(
                "{}: Could not send message to client {}: {:?}",
                self, recipient, e
            );
        }
        None
    }
}
