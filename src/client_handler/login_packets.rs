// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{auth::ClientInfo, Transfers};
use crate::client_handler::messaging::Messaging;
use crate::{
    action::{Action, ConsensusAction},
    chunk_store::{error::Error as ChunkStoreError, LoginPacketChunkStore},
    rpc::Rpc,
    utils,
};
use log::error;
use safe_nd::{
    DebitAgreementProof, Error as NdError, LoginPacket, LoginPacketRequest, MessageId,
    NodePublicId, PublicId, PublicKey, Request, Response, Result as NdResult, TransferRegistered,
    XorName,
};
use std::fmt::{self, Display, Formatter};

pub(super) struct LoginPackets {
    id: NodePublicId,
    login_packets: LoginPacketChunkStore,
}

impl LoginPackets {
    pub fn new(id: NodePublicId, login_packets: LoginPacketChunkStore) -> Self {
        Self { id, login_packets }
    }

    // on client request
    pub fn process_client_request(
        &mut self,
        client: &ClientInfo,
        request: LoginPacketRequest,
        message_id: MessageId,
    ) -> Option<Action> {
        use LoginPacketRequest::*;
        match request {
            Create {
                login_packet,
                debit_proof,
            } => self.initiate_creation(&client.public_id, login_packet, message_id, debit_proof),
            CreateFor {
                new_owner,
                debit_proof,
                optional_debit_proof,
                new_login_packet,
            } => self.initiate_proxied_creation(
                &client.public_id,
                new_owner,
                debit_proof,
                optional_debit_proof,
                new_login_packet,
                message_id,
            ),
            Update(updated_login_packet) => {
                self.initiate_update(client.public_id.clone(), updated_login_packet, message_id)
            }
            Get(ref address) => self.get(&client.public_id, address, message_id),
        }
    }

    // client query
    fn get(
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
    fn initiate_creation(
        &mut self,
        client_id: &PublicId,
        login_packet: LoginPacket,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<Action> {
        if !login_packet.size_is_valid() {
            return Some(Action::RespondToClient {
                message_id,
                response: Response::Write(Err(NdError::ExceededSize)),
            });
        }

        let request = Request::LoginPacket(LoginPacketRequest::Create {
            login_packet,
            debit_proof: debit_proof.clone(),
        });

        Some(Action::VoteFor(ConsensusAction::PayAndForward {
            request,
            client_public_id: client_id.clone(),
            message_id,
            debit_proof,
        }))
    }

    // on consensus
    pub fn finalise_client_request(
        &mut self,
        src: XorName,
        requester: PublicId,
        request: LoginPacketRequest,
        message_id: MessageId,
        transfers: &mut Transfers,
        messaging: &mut Messaging,
    ) -> Option<Action> {
        use LoginPacketRequest::*;
        match request {
            Create {
                ref login_packet,
                ref debit_proof,
            } => self.finalise_creation(requester, login_packet, message_id, debit_proof.clone()),
            CreateFor {
                new_owner,
                debit_proof,
                optional_debit_proof,
                new_login_packet,
            } => {
                if &src == requester.name() {
                    // Create balance and forward login_packet.
                    if let Some(optional_amount_proof) = optional_debit_proof.clone() {
                        if let Some(action) = transfers.receive_propagated(
                            &optional_amount_proof,
                            &requester,
                            message_id,
                            messaging,
                        ) {
                            // Failure occured - Refund here
                            return Some(action);
                        }
                    }
                    // Forward for creating LoginPacket
                    return Some(Action::ForwardClientRequest(Rpc::Request {
                        request: Request::LoginPacket(CreateFor {
                            new_owner,
                            debit_proof,
                            optional_debit_proof,
                            new_login_packet,
                        }),
                        requester,
                        message_id,
                        signature: None,
                    }));
                } else {
                    self.finalise_proxied_creation(
                        requester,
                        debit_proof,
                        new_login_packet,
                        message_id,
                    )
                }
            }
            Update(updated_login_packet) => {
                self.finalise_update(requester, &updated_login_packet, message_id)
            }
            Get(..) => {
                error!(
                    "{}: Should not receive {:?} as a client handler.",
                    self, request
                );
                None
            }
        }
    }

    // on consensus
    fn finalise_creation(
        &mut self,
        requester: PublicId,
        login_packet: &LoginPacket,
        message_id: MessageId,
        debit_proof: DebitAgreementProof,
    ) -> Option<Action> {
        let result = if self.login_packets.has(login_packet.destination()) {
            Err(NdError::LoginPacketExists)
        } else {
            self.login_packets
                .put(login_packet)
                .map_err(|error| error.to_string().into())
        };

        let refund = utils::get_refund_for_put(&result, debit_proof);
        Some(Action::RespondToClientHandlers {
            sender: *login_packet.destination(),
            rpc: Rpc::Response {
                response: Response::Write(result),
                requester,
                message_id,
                refund,
                proof: None,
            },
        })
    }

    /// Step one of the process - the payer is effectively doing a `CreateAccount` request to
    /// new_owner, and bundling the new_owner's `CreateLoginPacket` along with it.
    fn initiate_proxied_creation(
        &mut self,
        payer: &PublicId,
        new_owner: PublicKey,
        put_debit_proof: DebitAgreementProof,
        optional_amount_debit_proof: Option<DebitAgreementProof>,
        login_packet: LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        if !login_packet.size_is_valid() {
            return Some(Action::RespondToClient {
                message_id,
                response: Response::TransferRegistration(Err(NdError::ExceededSize)),
            });
        }
        // The requester bears the cost of storing the login packet
        Some(Action::VoteFor(ConsensusAction::PayAndProxy {
            request: Request::LoginPacket(LoginPacketRequest::CreateFor {
                new_owner,
                debit_proof: put_debit_proof.clone(),
                optional_debit_proof: optional_amount_debit_proof.clone(),
                new_login_packet: login_packet,
            }),
            client_public_id: payer.clone(),
            message_id,
            put_debit_proof,
            optional_amount_debit_proof,
        }))
    }

    /// Step two or three of the process - the payer is effectively doing a `CreateAccount` request
    /// to new_owner, and bundling the new_owner's `CreateLoginPacket` along with it.
    #[allow(clippy::too_many_arguments)]
    fn finalise_proxied_creation(
        &mut self,
        payer: PublicId,
        debit_proof: DebitAgreementProof,
        login_packet: LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        // Step three - store login_packet.
        let result = if self.login_packets.has(login_packet.destination()) {
            Err(NdError::LoginPacketExists)
        } else {
            self.login_packets
                .put(&login_packet)
                .map(|_| TransferRegistered { debit_proof })
                .map_err(|error| error.to_string().into())
        };
        Some(Action::RespondToClientHandlers {
            sender: *login_packet.destination(),
            rpc: Rpc::Response {
                response: Response::TransferRegistration(result),
                requester: payer,
                message_id,
                refund: None,
                proof: None,
            },
        })
    }

    // on client request
    fn initiate_update(
        &mut self,
        client_id: PublicId,
        updated_login_packet: LoginPacket,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::VoteFor(ConsensusAction::Forward {
            request: Request::LoginPacket(LoginPacketRequest::Update(updated_login_packet)),
            client_public_id: client_id,
            message_id,
        }))
    }

    // on consensus
    fn finalise_update(
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
                response: Response::Write(result),
                requester,
                message_id,
                // Updating the login packet is free
                refund: None,
                proof: None,
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

impl Display for LoginPackets {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
