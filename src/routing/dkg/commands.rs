// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    node::{DkgFailureSig, DkgFailureSigSet, DkgKey, Variant, WireMsg},
    DstInfo, DstLocation, SectionAuthorityProvider,
};
use crate::routing::{
    error::Result, messages::WireMsgUtils, node::Node, routing_api::command::Command,
    section::SectionKeyShare,
};
use bls::PublicKey as BlsPublicKey;
use bls_dkg::key_gen::message::Message as DkgMessage;
use std::{collections::BTreeSet, fmt::Debug, net::SocketAddr, time::Duration};
use xor_name::XorName;

#[derive(Debug)]
pub(crate) enum DkgCommand {
    SendMessage {
        recipients: Vec<(XorName, SocketAddr)>,
        dkg_key: DkgKey,
        message: DkgMessage,
    },
    ScheduleTimeout {
        duration: Duration,
        token: u64,
    },
    HandleOutcome {
        section_auth: SectionAuthorityProvider,
        outcome: SectionKeyShare,
    },
    SendFailureObservation {
        recipients: Vec<(XorName, SocketAddr)>,
        dkg_key: DkgKey,
        sig: DkgFailureSig,
        failed_participants: BTreeSet<XorName>,
    },
    HandleFailureAgreement(DkgFailureSigSet),
}

impl DkgCommand {
    fn into_command(self, node: &Node, key: BlsPublicKey) -> Result<Command> {
        match self {
            Self::SendMessage {
                recipients,
                dkg_key,
                message,
            } => {
                let variant = Variant::DkgMessage { dkg_key, message };
                let message =
                    WireMsg::single_src(node, DstLocation::DirectAndUnrouted(key), variant, key)?;

                Ok(Command::send_message_to_nodes(
                    recipients.clone(),
                    recipients.len(),
                    message,
                    DstInfo {
                        dst: XorName::random(),
                        dst_section_pk: key,
                    },
                ))
            }
            Self::ScheduleTimeout { duration, token } => {
                Ok(Command::ScheduleTimeout { duration, token })
            }
            Self::HandleOutcome {
                section_auth,
                outcome,
            } => Ok(Command::HandleDkgOutcome {
                section_auth,
                outcome,
            }),
            Self::SendFailureObservation {
                recipients,
                dkg_key,
                sig,
                failed_participants,
            } => {
                let variant = Variant::DkgFailureObservation {
                    dkg_key,
                    sig,
                    failed_participants,
                };
                let message =
                    WireMsg::single_src(node, DstLocation::DirectAndUnrouted(key), variant, key)?;

                Ok(Command::send_message_to_nodes(
                    recipients.clone(),
                    recipients.len(),
                    message,
                    DstInfo {
                        dst: XorName::random(),
                        dst_section_pk: key,
                    },
                ))
            }
            Self::HandleFailureAgreement(signeds) => Ok(Command::HandleDkgFailure(signeds)),
        }
    }
}

pub(crate) trait DkgCommands {
    fn into_commands(self, node: &Node, key: BlsPublicKey) -> Result<Vec<Command>>;
}

impl DkgCommands for Vec<DkgCommand> {
    fn into_commands(self, node: &Node, key: BlsPublicKey) -> Result<Vec<Command>> {
        self.into_iter()
            .map(|command| command.into_command(node, key))
            .collect()
    }
}

impl DkgCommands for Option<DkgCommand> {
    fn into_commands(self, node: &Node, key: BlsPublicKey) -> Result<Vec<Command>> {
        self.into_iter()
            .map(|command| command.into_command(node, key))
            .collect()
    }
}
