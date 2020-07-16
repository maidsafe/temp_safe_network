// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::utils;
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

pub enum ClientInput {
    Handshake(HandshakeRequest),
    Msg(MsgEnvelope),
}

struct InputParsing {

}

impl InputParsing {
    
    pub fn new() -> Self {
        Self {
        
        }
    }

    pub fn try_parse_client_input<R: CryptoRng + Rng>(
        &mut self,
        peer_addr: SocketAddr,
        bytes: &Bytes,
        rng: &mut R,
    ) -> Option<ClientInput> {
        match self.clients.get(&peer_addr).cloned() {
            None => self.try_deserialize_handshake(&bytes, peer_addr, rng),
            Some(client) => self.try_deserialize_msg(bytes),
                // if self.shall_handle_request(msg.message.id(), peer_addr) {
                //     trace!(
                //         "{}: Received ({:?} {:?}) from {}",
                //         self,
                //         "msg.get_type()",
                //         msg.message.id(),
                //         client,
                //     );
                //     return Some(ClientMsg { client, msg });
                // }
        }
    }

    fn try_deserialize_msg(&mut self, bytes: &Bytes) -> Option<ClientInput> {
        let msg = match bincode::deserialize(&bytes) {
            Ok(
                msg
                @
                MsgEnvelope {
                    message: Message::Cmd { .. },
                    origin: MsgSender::Client { .. },
                    ..
                },
            )
            | Ok(
                msg
                @
                MsgEnvelope {
                    message: Message::Query { .. },
                    origin: MsgSender::Client { .. },
                    ..
                },
            ) => msg,
            _ => return None, // Only cmds and queries from client are allowed through here.
        };

        Some(ClientInput::Msg(msg))
    }

    fn try_deserialize_handshake(&mut self, bytes: &Bytes, peer_addr: SocketAddr) -> Option<ClientInput> {
        let hs = match bincode::deserialize(&bytes) {
            Ok(hs @ HandshakeRequest::Bootstrap(_))
            | Ok(hs @ HandshakeRequest::Join(_))
            | Ok( hs @ HandshakeRequest::ChallengeResult(_)) => hs,
            Err(err) => {
                info!(
                    "{}: Failed to deserialize client input from {} as a handshake: {}",
                    self, peer_addr, err
                );
                return None;
            }
        };
        Some(ClientInput::Handshake(hs))
    }
}
