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
    AppPermissions, AppPublicId, AuthCmd, ClientAuth, Cmd, CmdError, DataAuthKind, Duty, ElderDuty,
    Error as NdError, Message, MessageId, MiscAuthKind, MoneyAuthKind, MsgEnvelope, MsgSender,
    NodePublicId, PublicId, PublicKey, QueryResponse, RequestAuthKind, Signature,
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

    pub(super) fn initiate(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        let auth_cmd = match msg.message {
            Message::Cmd {
                cmd: Cmd::Auth(auth_cmd),
                ..
            } => auth_cmd,
            _ => return None,
        };
        use AuthCmd::*;
        match auth_cmd {
            InsAuthKey { .. } | DelAuthKey { .. } => {
                self.set_proxy(&mut msg);
                Some(NodeCmd::VoteFor(ConsensusAction::Forward(msg)))
            }
        }
    }

    // If the client is app, check if it is authorised to perform the given request.
    pub fn authorise_app(&mut self, public_id: &PublicId, msg: &MsgEnvelope) -> Option<NodeCmd> {
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

        self.ok_or_error(result, msg.message.id(), msg.origin.address())
    }

    // client query
    pub fn list_keys_and_version(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        match msg.message {
            Message::Query {
                cmd: Query::Auth(ListAuthKeysAndVersion),
                ..
            } => (),
            _ => return None,
        };
        let result = Ok(self.auth_keys.list_keys_and_version(msg.origin.id()));
        self.wrap(Message::QueryResponse {
            response: QueryResponse::ListAuthKeysAndVersion(result),
            id: MessageId::new(),
            /// ID of causing query.
            correlation_id: msg.message.id(),
            /// The sender of the causing query.
            query_origin: msg.origin,
        })
    }

    // on consensus
    pub(super) fn finalise(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        use AuthCmd::*;
        let auth_cmd = match msg.message {
            Message::Cmd {
                cmd: Cmd::Auth(auth_cmd),
                ..
            } => auth_cmd,
            _ => return None,
        };
        let auth_error = |error: NdError| {
            self.wrap(Message::CmdError {
                error: CmdError::Auth(error),
                id: MessageId::new(),
                cmd_origin: msg.origin.address(),
                correlation_id: msg.id(),
            })
        };
        match auth_cmd {
            InsAuthKey {
                key,
                version,
                permissions,
            } => {
                match self
                    .auth_keys
                    .insert(msg.origin.id(), key, new_version, permissions)
                {
                    Ok(()) => None,
                    Err(error) => auth_error(error),
                }
            }
            DelAuthKey { key, version } => {
                match self.auth_keys.delete(msg.origin.id(), key, new_version) {
                    Ok(()) => None,
                    Err(error) => auth_error(error),
                }
            }
        }
    }

    // Verify that valid signature is provided if the request requires it.
    pub fn verify_client_signature(&mut self, msg: MsgEnvelope) -> Option<NodeCmd> {
        match request.authorisation_kind() {
            RequestAuthKind::Data(DataAuthKind::PublicRead) => None,
            _ => {
                if self.is_valid_client_signature(msg) {
                    None
                } else {
                    self.wrap(Message::CmdError)
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
                    pub_key,
                    error
                );
                false
            }
        }
    }

    fn set_proxy(&self, msg: &mut MsgEnvelope) {
        // origin signs the message, while proxies sign the envelope
        msg.add_proxy(self.sign(msg))
    }

    fn wrap(&self, message: Message) -> Option<NodeCmd> {
        let msg = MsgEnvelope {
            message,
            origin: self.sign(message),
            proxies: Default::default(),
        };
        Some(NodeCmd::SendToClient(msg))
    }

    fn sign<T: Serialize>(&self, data: &T) -> MsgSender {
        let signature = &utils::sign(self.routing.borrow(), &utils::serialise(data));
        MsgSender::Node {
            id: self.public_key(),
            duty: Duty::Elder(ElderDuty::Gateway),
            signature,
        }
    }

    fn ok_or_error(
        &self,
        result: Result<()>,
        msg_id: MessageId,
        origin: MsgSender,
    ) -> Option<NodeCmd> {
        let error = match result {
            Ok(()) => return None,
            Err(error) => error,
        };
        let message = Message::CmdError {
            id: MessageId::new(),
            error: CmdError::Auth(error),
            correlation_id: msg_id,
            cmd_origin: origin,
        };
        self.wrap(message)
    }

    fn public_key(&self) -> PublicKey {
        PublicKey::Bls(self.id.public_id().bls_public_key())
    }
}

impl Display for Auth {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}
