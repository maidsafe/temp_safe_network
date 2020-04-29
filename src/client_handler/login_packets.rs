// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::COST_OF_PUT;
use crate::{
    action::{Action, ConsensusAction},
    chunk_store::{error::Error as ChunkStoreError, LoginPacketChunkStore},
    rpc::Rpc,
    utils,
};
use safe_nd::{
    Coins, Error as NdError, LoginPacket, MessageId, NodePublicId, PublicId, PublicKey, Request,
    Response, Result as NdResult, Transaction, XorName,
};

pub(super) struct LoginPackets {
    id: NodePublicId,
    login_packets: LoginPacketChunkStore,
}

impl LoginPackets {
    pub fn new(id: NodePublicId, login_packets: LoginPacketChunkStore) -> Self {
        Self { id, login_packets }
    }

    // client query
    pub fn get_login_packet(
        &mut self,
        client_id: &PublicId,
        address: &XorName,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = self
            .login_packet(utils::own_key(client_id)?, address)
            .map(LoginPacket::into_data_and_signature);
        Some(Action::RespondToClient {
            message_id,
            response: Response::GetLoginPacket(result),
        })
    }

    // on client request
    pub fn initiate_login_packet_creation(
        &mut self,
        client_id: &PublicId,
        login_packet: LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        if !login_packet.size_is_valid() {
            return Some(Action::RespondToClient {
                message_id,
                response: Response::Mutation(Err(NdError::ExceededSize)),
            });
        }

        let request = Request::CreateLoginPacket(login_packet);

        Some(Action::VoteFor(ConsensusAction::Forward {
            request,
            client_public_id: client_id.clone(),
            message_id,
            cost: COST_OF_PUT,
        }))
    }

    // on consensus
    pub fn finalize_login_packet_creation(
        &mut self,
        requester: PublicId,
        login_packet: &LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = if self.login_packets.has(login_packet.destination()) {
            Err(NdError::LoginPacketExists)
        } else {
            self.login_packets
                .put(login_packet)
                .map_err(|error| error.to_string().into())
        };
        Some(Action::RespondToClientHandlers {
            sender: *login_packet.destination(),
            rpc: Rpc::Response {
                response: Response::Mutation(result),
                requester,
                message_id,
            },
        })
    }

    /// Step one of the process - the payer is effectively doing a `CreateAccount` request to
    /// new_owner, and bundling the new_owner's `CreateLoginPacket` along with it.
    pub fn initiate_proxied_login_packet_creation(
        &mut self,
        payer: &PublicId,
        new_owner: PublicKey,
        amount: Coins,
        login_packet: LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        if !login_packet.size_is_valid() {
            return Some(Action::RespondToClient {
                message_id,
                response: Response::Transaction(Err(NdError::ExceededSize)),
            });
        }
        // The requester bears the cost of storing the login packet
        let new_amount = amount.checked_add(COST_OF_PUT)?;
        Some(Action::VoteFor(ConsensusAction::Proxy {
            request: Request::CreateLoginPacketFor {
                new_owner,
                amount,
                new_login_packet: login_packet,
            },
            client_public_id: payer.clone(),
            message_id,
            cost: new_amount,
        }))
    }

    /// Step two or three of the process - the payer is effectively doing a `CreateAccount` request
    /// to new_owner, and bundling the new_owner's `CreateLoginPacket` along with it.
    #[allow(clippy::too_many_arguments)]
    pub fn finalize_proxied_login_packet_creation(
        &mut self,
        src: XorName,
        payer: PublicId,
        new_owner: PublicKey,
        amount: Coins,
        login_packet: LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        if &src == payer.name() {
            // Step two - create balance and forward login_packet.
            //if let Err(error) = self.create_balance(&payer, new_owner, amount) {
            if false {
                unimplemented!(); // NB: Temporarily disabled creating of balance.
                                  // Some(Action::RespondToClientHandlers {
                                  //     sender: XorName::from(new_owner),
                                  //     rpc: Rpc::Response {
                                  //         response: Response::Transaction(Err(error)),
                                  //         requester: payer,
                                  //         message_id,
                                  //     },
                                  // })
            } else {
                Some(Action::ForwardClientRequest(Rpc::Request {
                    request: Request::CreateLoginPacketFor {
                        new_owner,
                        amount,
                        new_login_packet: login_packet,
                    },
                    requester: payer,
                    message_id,
                }))
            }
        } else {
            // Step three - store login_packet.
            let result = if self.login_packets.has(login_packet.destination()) {
                Err(NdError::LoginPacketExists)
            } else {
                self.login_packets
                    .put(&login_packet)
                    .map(|_| Transaction {
                        id: message_id,
                        amount,
                    })
                    .map_err(|error| error.to_string().into())
            };
            Some(Action::RespondToClientHandlers {
                sender: *login_packet.destination(),
                rpc: Rpc::Response {
                    response: Response::Transaction(result),
                    requester: payer,
                    message_id,
                },
            })
        }
    }

    // on client request
    pub fn initiate_login_packet_update(
        &mut self,
        client_id: PublicId,
        updated_login_packet: LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::VoteFor(ConsensusAction::Forward {
            request: Request::UpdateLoginPacket(updated_login_packet),
            client_public_id: client_id,
            message_id,
            cost: Coins::from_nano(0),
        }))
    }

    // on consensus
    pub fn finalize_login_packet_update(
        &mut self,
        requester: PublicId,
        updated_login_packet: &LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = self
            .login_packet(
                utils::own_key(&requester)?,
                updated_login_packet.destination(),
            )
            .and_then(|_existing_login_packet| {
                if !updated_login_packet.size_is_valid() {
                    return Err(NdError::ExceededSize);
                }
                self.login_packets
                    .put(&updated_login_packet)
                    .map_err(|err| err.to_string().into())
            });
        Some(Action::RespondToClientHandlers {
            sender: *self.id.name(),
            rpc: Rpc::Response {
                response: Response::Mutation(result),
                requester,
                message_id,
            },
        })
    }

    fn login_packet(
        &self,
        requester_pub_key: &PublicKey,
        packet_name: &XorName,
    ) -> NdResult<LoginPacket> {
        self.login_packets
            .get(packet_name)
            .map_err(|e| match e {
                ChunkStoreError::NoSuchChunk => NdError::NoSuchLoginPacket,
                error => error.to_string().into(),
            })
            .and_then(|login_packet| {
                if login_packet.authorised_getter() == requester_pub_key {
                    Ok(login_packet)
                } else {
                    Err(NdError::AccessDenied)
                }
            })
    }
}
