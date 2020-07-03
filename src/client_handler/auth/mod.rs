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
    action::{Action, ConsensusAction},
    rpc::Rpc,
    utils,
};
use log::{error, warn};
use safe_nd::{
    AppPermissions, AppPublicId, ClientAuth, ClientRequest, DataAuthKind, Error as NdError,
    MessageId, MiscAuthKind, MoneyAuthKind, NodePublicId, PublicId, PublicKey, Request,
    RequestAuthKind, Response, Signature, SystemOp,
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

    pub fn initiate(
        &mut self,
        client: PublicId,
        request: ClientAuth,
        message_id: MessageId,
    ) -> Option<Action> {
        use ClientAuth::*;
        match request {
            ListAuthKeysAndVersion => self.list_keys_and_version(client, message_id),
            InsAuthKey {
                key,
                version,
                permissions,
            } => self.initiate_key_insertion(client, key, version, permissions, message_id),
            DelAuthKey { key, version } => {
                self.initiate_key_deletion(client, key, version, message_id)
            }
        }
    }

    // If the client is app, check if it is authorised to perform the given request.
    pub fn authorise_app(
        &mut self,
        public_id: &PublicId,
        request: &Request,
        message_id: MessageId,
    ) -> Option<Action> {
        let app_id = match public_id {
            PublicId::App(app_id) => app_id,
            _ => return None,
        };

        let result = match request.authorisation_kind() {
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
            Some(Action::RespondToClient {
                message_id,
                response: request.error_response(error),
            })
        } else {
            None
        }
    }

    // client query
    fn list_keys_and_version(&mut self, client: PublicId, message_id: MessageId) -> Option<Action> {
        let result = Ok(self
            .auth_keys
            .list_keys_and_version(utils::client(&client)?));
        Some(Action::RespondToClient {
            message_id,
            response: Response::ListAuthKeysAndVersion(result),
        })
    }

    // on consensus
    pub(super) fn finalise(
        &mut self,
        requester: PublicId,
        request: ClientAuth,
        message_id: MessageId,
    ) -> Option<Action> {
        use ClientAuth::*;
        match request {
            InsAuthKey {
                key,
                version,
                permissions,
            } => self.finalise_key_insertion(requester, key, version, permissions, message_id),
            DelAuthKey { key, version } => {
                self.finalise_key_deletion(requester, key, version, message_id)
            }
            ListAuthKeysAndVersion => {
                error!(
                    "{}: Should not receive {:?} as a client handler.",
                    self, request
                );
                None
            }
        }
    }

    // on client request
    fn initiate_key_insertion(
        &self,
        client_public_id: PublicId,
        key: PublicKey,
        new_version: u64,
        permissions: AppPermissions,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::VoteFor(ConsensusAction::Forward {
            request: Request::Client(ClientRequest::System(SystemOp::ClientAuth(
                ClientAuth::InsAuthKey {
                    key,
                    version: new_version,
                    permissions,
                },
            ))),
            client_public_id,
            message_id,
        }))
    }

    // on consensus
    fn finalise_key_insertion(
        &mut self,
        requester: PublicId,
        key: PublicKey,
        new_version: u64,
        permissions: AppPermissions,
        message_id: MessageId,
    ) -> Option<Action> {
        let result =
            self.auth_keys
                .insert(utils::client(&requester)?, key, new_version, permissions);
        Some(Action::RespondToClientHandlers {
            sender: *self.id.name(),
            rpc: Rpc::Response {
                response: Response::Write(result),
                requester,
                message_id,
                refund: None,
                proof: None,
            },
        })
    }

    // on client request
    fn initiate_key_deletion(
        &mut self,
        client_public_id: PublicId,
        key: PublicKey,
        new_version: u64,
        message_id: MessageId,
    ) -> Option<Action> {
        Some(Action::VoteFor(ConsensusAction::Forward {
            request: Request::Client(ClientRequest::System(SystemOp::ClientAuth(
                ClientAuth::DelAuthKey {
                    key,
                    version: new_version,
                },
            ))),
            client_public_id,
            message_id,
        }))
    }

    // on consensus
    pub fn finalise_key_deletion(
        &mut self,
        requester: PublicId,
        key: PublicKey,
        new_version: u64,
        message_id: MessageId,
    ) -> Option<Action> {
        let result = self
            .auth_keys
            .delete(utils::client(&requester)?, key, new_version);
        Some(Action::RespondToClientHandlers {
            sender: *self.id.name(),
            rpc: Rpc::Response {
                response: Response::Write(result),
                requester,
                message_id,
                refund: None,
                proof: None,
            },
        })
    }

    // Verify that valid signature is provided if the request requires it.
    pub fn verify_signature(
        &mut self,
        public_id: PublicId,
        request: &Request,
        message_id: MessageId,
        signature: Option<Signature>,
    ) -> Option<Action> {
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
                    Some(Action::RespondToClient {
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
        request: &Request,
        message_id: &MessageId,
        signature: &Signature,
    ) -> bool {
        let pub_key = match utils::own_key(&client_id) {
            Some(pk) => pk,
            None => {
                error!("{}: Logic error.  This should be unreachable.", self);
                return false;
            }
        };
        match pub_key.verify(signature, utils::serialise(&(request, message_id))) {
            Ok(_) => true,
            Err(error) => {
                warn!(
                    "{}: ({:?}/{:?}) from {} is invalid: {}",
                    self, request, message_id, client_id, error
                );
                false
            }
        }
    }
}

impl Display for Auth {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}
