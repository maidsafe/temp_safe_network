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
    cmd::OutboundMsg, node::keys::NodeKeys, node::msg_decisions::ElderMsgDecisions, utils,
};
use log::warn;
use safe_nd::{
    AppPermissions, AppPublicId, AuthCmd, Cmd, CmdError, DataAuthKind, Error as NdError, Message,
    MessageId, MiscAuthKind, MoneyAuthKind, MsgEnvelope, MsgSender, PublicId, Query, QueryResponse,
};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub public_id: PublicId,
}

pub(super) struct Auth {
    keys: NodeKeys,
    auth_keys: AuthKeysDb,
    decisions: ElderMsgDecisions,
}

impl Auth {
    pub fn new(keys: NodeKeys, auth_keys: AuthKeysDb, decisions: ElderMsgDecisions) -> Self {
        Self {
            keys,
            auth_keys,
            decisions,
        }
    }

    pub(super) fn initiate(&mut self, msg: MsgEnvelope) -> Option<OutboundMsg> {
        let auth_cmd = match msg.message {
            Message::Cmd {
                cmd: Cmd::Auth(auth_cmd),
                ..
            } => auth_cmd,
            _ => return None,
        };
        use AuthCmd::*;
        match auth_cmd {
            InsAuthKey { .. } | DelAuthKey { .. } => self.decisions.vote(msg),
        }
    }

    // If the client is app, check if it is authorised to perform the given request.
    pub fn authorise_app(
        &mut self,
        public_id: &PublicId,
        msg: &MsgEnvelope,
    ) -> Option<OutboundMsg> {
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
            self.decisions.error(
                CmdError::Auth(error),
                msg.message.id(),
                msg.origin.address(),
            )
        }
        None
    }

    // client query
    pub fn list_keys_and_version(&mut self, msg: MsgEnvelope) -> Option<OutboundMsg> {
        match msg.message {
            Message::Query {
                query: Query::Auth(ListAuthKeysAndVersion),
                ..
            } => (),
            _ => return None,
        };
        let result = Ok(self.auth_keys.list_keys_and_version(msg.origin.id()));
        self.decisions.send(Message::QueryResponse {
            response: QueryResponse::ListAuthKeysAndVersion(result),
            id: MessageId::new(),
            /// ID of causing query.
            correlation_id: msg.message.id(),
            /// The sender of the causing query.
            query_origin: msg.origin.address(),
        })
    }

    // on consensus
    pub(super) fn finalise(&mut self, msg: MsgEnvelope) -> Option<OutboundMsg> {
        use AuthCmd::*;
        let auth_cmd = match msg.message {
            Message::Cmd {
                cmd: Cmd::Auth(auth_cmd),
                ..
            } => auth_cmd,
            _ => return None,
        };
        let result = match auth_cmd {
            InsAuthKey {
                key,
                version,
                permissions,
                ..
            } => self
                .auth_keys
                .insert(msg.origin.id(), key, version, permissions),
            DelAuthKey { key, version, .. } => self.auth_keys.delete(msg.origin.id(), key, version),
        };
        if let Err(error) = result {
            self.decisions.error(
                CmdError::Auth(error),
                msg.message.id(),
                msg.origin.address(),
            )
        }
        None
    }

    // Verify that valid signature is provided if the request requires it.
    pub fn verify_client_signature(&mut self, msg: MsgEnvelope) -> Option<OutboundMsg> {
        match msg.authorisation_kind() {
            RequestAuthKind::Data(DataAuthKind::PublicRead) => None,
            _ => {
                if self.is_valid_client_signature(&msg) {
                    None
                } else {
                    self.decisions.error(
                        CmdError::Auth(NdError::AccessDenied),
                        msg.id(),
                        msg.origin,
                    )
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

    fn is_valid_client_signature(&self, msg: &MsgEnvelope) -> bool {
        let signature = match msg.origin {
            MsgSender::Client { signature, .. } => signature,
            _ => return false,
        };
        match msg
            .origin
            .id()
            .verify(&signature, utils::serialise(&msg.message))
        {
            Ok(_) => true,
            Err(error) => {
                warn!(
                    "{}: ({:?}/{:?}) from {} is invalid: {}",
                    self,
                    "msg.get_type()",
                    msg.message.id(),
                    msg.origin.id(),
                    error
                );
                false
            }
        }
    }
}

impl Display for Auth {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
