// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::{
    client::ClientMsg,
    node::{DstInfo, KeyedSig},
    ClientSigned, DstLocation, EndUser, MessageType, SrcLocation,
};
use crate::routing::{
    error::{Error, Result},
    routing_api::command::Command,
    section::SectionUtils,
    Event,
};
use bls::PublicKey;
use bytes::Bytes;

impl Core {
    pub(crate) async fn handle_forwarded_message(
        &mut self,
        msg: ClientMsg,
        user: EndUser,
        client_signed: ClientSigned,
    ) -> Result<Vec<Command>> {
        self.send_event(Event::ClientMsgReceived {
            msg: Box::new(msg),
            user,
            client_signed,
        })
        .await;

        Ok(vec![])
    }

    pub(crate) async fn handle_user_message(
        &mut self,
        content: Bytes,
        src: SrcLocation,
        dst: DstLocation,
        section_pk: PublicKey,
        sig: Option<KeyedSig>,
    ) -> Result<Vec<Command>> {
        if let DstLocation::EndUser(EndUser {
            xorname: xor_name,
            socket_id,
        }) = dst
        {
            if let Some(socket_addr) = self.get_socket_addr(socket_id).copied() {
                trace!("sending user message to client {:?}", socket_addr);
                unimplemented!();
                /*
                return Ok(vec![Command::SendMessage {
                    recipients: vec![(xor_name, socket_addr)],
                    delivery_group_size: 1,
                    message: MessageType::Client {
                        msg: ClientMsg::from(content)?,
                        dst_info: DstInfo {
                            dst: xor_name,
                            dst_section_pk: *self.section.chain().last_key(),
                        },
                    },
                }]);*/
            } else {
                trace!(
                    "Cannot route user message, socket id not found {:?}",
                    socket_id
                );
                return Err(Error::EmptyRecipientList);
            }
        }

        self.send_event(Event::MessageReceived {
            content,
            src,
            dst,
            sig,
            section_pk,
        })
        .await;

        Ok(vec![])
    }
}
