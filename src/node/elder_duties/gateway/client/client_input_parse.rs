// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use bytes::Bytes;
use log::info;
use safe_nd::{HandshakeRequest, Message, MessageId, MsgEnvelope, MsgSender, PublicId};
use std::{
    fmt::{self, Display, Formatter},
    net::SocketAddr,
};

pub enum ClientInput {
    Msg(ClientMsg),
    Handshake(HandshakeRequest),
}

#[derive(Clone, Debug)]
pub struct ClientMsg {
    pub msg: MsgEnvelope,
    pub public_id: PublicId,
}

impl ClientMsg {
    pub fn id(&self) -> MessageId {
        self.msg.id()
    }
}

impl Display for ClientMsg {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}, {}", self.public_id.name(), &self.msg.id().0)
    }
}

pub fn try_deserialize_msg(bytes: &Bytes) -> Option<ClientInput> {
    let msg = match bincode::deserialize(&bytes) {
        Ok((
            public_id @ PublicId,
            msg
            @
            MsgEnvelope {
                message: Message::Cmd { .. },
                origin: MsgSender::Client { .. },
                ..
            },
        ))
        | Ok((
            public_id @ PublicId,
            msg
            @
            MsgEnvelope {
                message: Message::Query { .. },
                origin: MsgSender::Client { .. },
                ..
            },
        )) => ClientMsg { msg, public_id },
        _ => return None, // Only cmds and queries from client are allowed through here.
    };

    Some(ClientInput::Msg(msg))
}

pub fn try_deserialize_handshake(bytes: &Bytes, peer_addr: SocketAddr) -> Option<ClientInput> {
    let hs = match bincode::deserialize(&bytes) {
        Ok(hs @ HandshakeRequest::Bootstrap(_))
        | Ok(hs @ HandshakeRequest::Join(_))
        | Ok(hs @ HandshakeRequest::ChallengeResult(_)) => hs,
        Err(err) => {
            info!(
                "Failed to deserialize client input from {} as a handshake: {}",
                peer_addr, err
            );
            return None;
        }
    };
    Some(ClientInput::Handshake(hs))
}
