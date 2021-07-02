// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::{client::ClientMsg, ClientSigned, EndUser};
use crate::routing::{
    core::enduser_registry::SocketId,
    error::{Error, Result},
    routing_api::{command::Command, Event},
};
use xor_name::XorName;

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

    pub(crate) async fn handle_end_user_message(
        &mut self,
        //client_msg: ClientMsg,
        xorname: XorName,
        socket_id: SocketId,
    ) -> Result<Vec<Command>> {
        if let Some(socket_addr) = self.get_socket_addr(socket_id).copied() {
            trace!("sending user message to client {:?}", socket_addr);
            unimplemented!();
            /*Ok(vec![Command::SendMessage {
                recipients: vec![(xorname, socket_addr)],
                delivery_group_size: 1,
                message: MessageType::Client {
                    msg: ClientMsg::from(content)?,
                    dst_info: DstInfo {
                        dst: xor_name,
                        dst_section_pk: *self.section.chain().last_key(),
                    },
                },
            }])*/
        } else {
            trace!(
                "Cannot route user message, socket id not found {:?}",
                socket_id
            );
            Err(Error::EmptyRecipientList)
        }
    }
}
