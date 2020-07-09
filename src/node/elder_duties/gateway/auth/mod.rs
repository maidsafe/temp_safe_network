// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod auth_keys;

pub use self::auth_keys::AuthKeysDb;

use crate::{
    cmd::{ConsensusAction, GatewayCmd},
    utils,
};
use log::{error, warn};
use safe_nd::{
    AppPermissions, AppPublicId, AuthCmd, AuthQuery, ClientAuth, CmdError,
    DataAuthKind, Duty, ElderDuty, Error as NdError, Message, MessageId,
    MiscAuthKind, MoneyAuthKind, MsgEnvelope, MsgSender, NodePublicId, PublicId, PublicKey,
    RequestAuthKind, Signature, Cmd, QueryResponse,
};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub public_id: PublicId,
}

pub(super) struct Auth {
    id: NodePublicId,
    auth_keys: AuthKeysDb,
}

impl Auth {
    pub fn new(id: NodePublicId, auth_keys: AuthKeysDb) -> Self {
        Self { id, auth_keys }
    }

    pub(super) fn initiate(
        &mut self,
        client: PublicId,
        msg: MsgEnvelope,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let auth_cmd = match msg.message {
            Message::Cmd { cmd: Cmd::Auth(auth_cmd), .. } => auth_cmd,
            _ => return None,
        };
        use AuthCmd::*;
        match auth_cmd {
            InsAuthKey { .. }
            | DelAuthKey { .. } => {
                self.set_proxy(&mut msg);
                Some(GatewayCmd::VoteFor(ConsensusAction::Forward(msg)))
            }
        }
    }

    pub fn query(
        &mut self,
        client: PublicId,
        msg: MsgEnvelope,
    ) -> Option<NodeCmd> {
        self.list_keys_and_version(client, message_id)
    }

    // If the client is app, check if it is authorised to perform the given request.
    pub fn authorise_app(
        &mut self,
        public_id: &PublicId,
        msg: &MsgEnvelope,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        let app_id = match public_id {
            PublicId::App(app_id) => app_id,
            _ => return None,
        };

        match msg.most_recent_sender() {
            MsgSender::Client { .. } => (),
            _ => return None,
        };

        let auth_kind = match msg.message {
            Message::Cmd { cmd, .. } => cmd.authorisation_kind(),
            Message::Query { query, .. } => query.authorisation_kind(),
            _ => return None,
        };

        let result = match auth_kind {
            RequestAuthKind::Data(DataAuthKind::PublicRead) => Ok(()),
            RequestAuthKind::Data(DataAuthKind::PrivateRead) => {
                self.check_app_permissions(app_id, |_| true)
            }
            RequestAuthKind::Money(MoneyAuthKind::ReadBalance) => {
                self.check_app_permissions(app_id, |perms| perms.read_balance)
            }
            RequestAuthKind::Money(MoneyAuthKind::ReadHistory) => {
                self.check_app_permissions(app_id, |perms| perms.read_transfer_history)
            }
            RequestAuthKind::Data(DataAuthKind::Write) => {
                self.check_app_permissions(app_id, |perms| perms.data_mutations)
            }
            RequestAuthKind::Money(MoneyAuthKind::Transfer) => {
                self.check_app_permissions(app_id, |perms| perms.transfer_money)
            }
            RequestAuthKind::Misc(MiscAuthKind::WriteAndTransfer) => self
                .check_app_permissions(app_id, |perms| {
                    perms.transfer_money && perms.data_mutations
                }),
            RequestAuthKind::Misc(MiscAuthKind::ManageAppKeys) => Err(NdError::AccessDenied),
            RequestAuthKind::None => Err(NdError::AccessDenied),
        };

        if let Err(error) = result {
            let message = Message::CmdError {
                error: CmdError::Auth(error),
                id: MessageId::new(), // or hash of msg.message.id() ? But will that be problem at client? hash(self.id() + msg.id()) is better
                correlation_id: msg.message.id(),
                cmd_origin: msg.origin.address(),
            };
            // origin signs the message, while proxies sign the envelope
            let signature = &utils::sign(self.routing.borrow(), &utils::serialise(&message));
            let cmd_error = MsgEnvelope {
                message,
                origin: MsgSender {
                    id: id.public_id().public_key(),
                    duty: Duty::Elder(ElderDuty::Gateway),
                    signature,
                },
            };
            Some(GatewayCmd::PushToClient(cmd_error))
        } else {
            None
        }
    }

    // client query
    fn list_keys_and_version(
        &mut self,
        client: PublicId,
        msg: MsgEnvelope,
    ) -> Option<NodeCmd> {
        use AuthQuery::*;
        match msg.message {
            Message::Query { cmd: Query::Auth(ListAuthKeysAndVersion), .. } => (),
            _ => return None,
        };
        let result = Ok(self
            .auth_keys
            .list_keys_and_version(utils::client(&client)?));
        Some(NodeCmd::SendToClient(MsgEnvelope {
            message: Message::QueryResponse {
                response: QueryResponse::ListAuthKeysAndVersion(result),
                id: MessageId::new(),
                /// ID of causing query.
                correlation_id: msg.message.id(),
                /// The sender of the causing query.
                query_origin: msg.origin,
            },
            origin: MsgSender {
                id: ,
                duty: Duty::Elder(ElderDuty::Gateway),
                signature,
            },
            proxies: Default::default(),
        }))
    }

    // on consensus
    pub(super) fn finalise(
        &mut self,
        requester: PublicId,
        msg: MsgEnvelope,
        message_id: MessageId,
    ) -> Option<NodeCmd> {
        use AuthCmd::*;
        let auth_cmd = match msg.message {
            Message::Cmd { cmd: Cmd::Auth(auth_cmd), .. } => auth_cmd,
            _ => return None,
        };
        match auth_cmd {
            InsAuthKey {
                key,
                version,
                permissions,
            } => {
                let result = self.auth_keys
                    .insert(utils::client(&requester)?, key, new_version, permissions);
                None
            },
            DelAuthKey { key, version } => {
                let result = self.auth_keys
                    .delete(utils::client(&requester)?, key, new_version);
                None
            }
        }
    }

    // Verify that valid signature is provided if the request requires it.
    pub fn verify_signature(
        &mut self,
        public_id: PublicId,
        request: &ClientRequest,
        message_id: MessageId,
        signature: Option<Signature>,
    ) -> Option<NodeCmd> {
        match request.authorisation_kind() {
            RequestAuthKind::Data(DataAuthKind::PublicRead) => None,
            _ => {
                let valid = if let Some(signature) = signature {
                    self.is_valid_client_signature(public_id, request, &message_id, &signature)
                } else {
                    warn!(
                        "{}: ({:?}/{:?}) from {} is unsigned",
                        self, request, message_id, public_id
                    );
                    false
                };

                if valid {
                    None
                } else {
                    Some(GatewayCmd::RespondToClient {
                        message_id,
                        response: request.error_response(NdError::InvalidSignature),
                    })
                }
            }
        }
    }

    fn check_app_permissions(
        &self,
        app_id: &AppPublicId,
        check: impl FnOnce(AppPermissions) -> bool,
    ) -> Result<(), NdError> {
        if self
            .auth_keys
            .app_permissions(app_id)
            .map(check)
            .unwrap_or(false)
        {
            Ok(())
        } else {
            Err(NdError::AccessDenied)
        }
    }

    fn is_valid_client_signature(
        &self,
        client_id: PublicId,
        msg: &MsgEnvelope,
    ) -> bool {
        let signature = match msg.origin {
            MsgSender::Client {
                signature, ..
            } => signature,
            _ => return false,
        };
        let pub_key = match utils::own_key(&client_id) {
            Some(pk) => pk,
            None => {
                error!("{}: Logic error.  This should be unreachable.", self);
                return false;
            }
        };
        match pub_key.verify(&signature, utils::serialise(&msg.message)) {
            Ok(_) => true,
            Err(error) => {
                warn!(
                    "{}: ({:?}/{:?}) from {} is invalid: {}",
                    self, "msg.get_type()", msg.message.id(), client_id, error
                );
                false
            }
        }
    }

    fn set_proxy(&self, msg: &mut MsgEnvelope) {
        // origin signs the message, while proxies sign the envelope
        let signature = &utils::sign(self.routing.borrow(), &utils::serialise(&msg));
        msg.add_proxy(MsgSender {
            id: self.id.into(),
            duty: Duty::Elder(ElderDuty::Gateway),
            signature,
        })
    }
}

impl Display for Auth {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}
