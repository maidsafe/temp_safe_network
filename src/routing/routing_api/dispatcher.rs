// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Command, Event};
use crate::messaging::{
    node::{NodeMsg, Section},
    DstLocation, MsgKind, WireMsg,
};
use crate::routing::{
    core::{Core, SendStatus},
    error::Result,
    messages::WireMsgUtils,
    node::Node,
    peer::PeerUtils,
    section::SectionPeersUtils,
    section::SectionUtils,
    Error, XorName,
};
use itertools::Itertools;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    sync::{watch, RwLock},
    time,
};
use tracing::Instrument;

// `Command` Dispatcher.
pub(crate) struct Dispatcher {
    pub(super) core: RwLock<Core>,

    cancel_timer_tx: watch::Sender<bool>,
    cancel_timer_rx: watch::Receiver<bool>,
}

impl Dispatcher {
    pub fn new(core: Core) -> Self {
        let (cancel_timer_tx, cancel_timer_rx) = watch::channel(false);
        Self {
            core: RwLock::new(core),
            cancel_timer_tx,
            cancel_timer_rx,
        }
    }

    // Terminate this routing instance - cancel all scheduled timers including any future ones,
    // close all network connections and stop accepting new connections.
    pub async fn terminate(&self) {
        let _ = self.cancel_timer_tx.send(true);
        self.core.read().await.comm.terminate().await;
    }

    /// Handles the given command and transitively any new commands that are produced during its
    /// handling.
    pub async fn handle_commands(self: Arc<Self>, command: Command) -> Result<()> {
        let commands = self.handle_command(command).await?;
        for command in commands {
            self.clone().spawn_handle_commands(command)
        }

        Ok(())
    }

    // Note: this indirecton is needed. Trying to call `spawn(self.handle_commands(...))` directly
    // inside `handle_commands` causes compile error about type check cycle.
    fn spawn_handle_commands(self: Arc<Self>, command: Command) {
        let _ = tokio::spawn(self.handle_commands(command));
    }

    /// Handles a single command.
    pub async fn handle_command(&self, command: Command) -> Result<Vec<Command>> {
        // Create a tracing span containing info about the current node. This is very useful when
        // analyzing logs produced by running multiple nodes within the same process, for example
        // from integration tests.
        let span = {
            let core = self.core.read().await;
            trace_span!(
                "handle_command",
                name = %core.node().name(),
                prefix = format_args!("({:b})", core.section().prefix()),
                age = core.node().age(),
                elder = core.is_elder(),
            )
        };

        async {
            trace!(?command);

            self.try_handle_command(command).await.map_err(|error| {
                error!("Error encountered when handling command: {}", error);
                error
            })
        }
        .instrument(span)
        .await
    }

    async fn try_handle_command(&self, command: Command) -> Result<Vec<Command>> {
        match command {
            Command::HandleMessage { sender, wire_msg } => {
                self.core
                    .write()
                    .await
                    .handle_message(sender, wire_msg)
                    .await
            }
            Command::HandleTimeout(token) => self.core.write().await.handle_timeout(token),
            Command::HandleAgreement { proposal, sig } => {
                self.core
                    .write()
                    .await
                    .handle_agreement(proposal, sig)
                    .await
            }
            Command::HandleConnectionLost(addr) => {
                self.core.read().await.handle_connection_lost(addr)
            }
            Command::HandlePeerLost(addr) => self.core.read().await.handle_peer_lost(&addr),
            Command::HandleDkgOutcome {
                section_auth,
                outcome,
            } => self
                .core
                .write()
                .await
                .handle_dkg_outcome(section_auth, outcome),
            Command::HandleDkgFailure(signeds) => self
                .core
                .write()
                .await
                .handle_dkg_failure(signeds)
                .map(|command| vec![command]),
            Command::SendMessage {
                // send to network from routing layer
                recipients,
                delivery_group_size,
                wire_msg,
            } => {
                self.send_message(&recipients, delivery_group_size, wire_msg)
                    .await
            }
            Command::RelayMessage(wire_msg) => {
                if let Some(cmd) = self.core.write().await.relay_message(wire_msg).await? {
                    Ok(vec![cmd])
                } else {
                    Ok(vec![])
                }
            }
            Command::ScheduleTimeout { duration, token } => Ok(self
                .handle_schedule_timeout(duration, token)
                .await
                .into_iter()
                .collect()),
            Command::HandleRelocationComplete { node, section } => {
                self.handle_relocation_complete(node, section).await;
                Ok(vec![])
            }
            Command::SetJoinsAllowed(joins_allowed) => {
                self.core.read().await.set_joins_allowed(joins_allowed)
            }
            Command::ProposeOnline {
                mut peer,
                previous_name,
                dst_key,
            } => {
                // The reachability check was completed during the initial bootstrap phase
                peer.set_reachable(true);
                self.core
                    .read()
                    .await
                    .make_online_proposal(peer, previous_name, dst_key)
                    .await
            }
            Command::ProposeOffline(name) => self.core.read().await.propose_offline(name),
            Command::StartConnectivityTest(name) => {
                let msg = {
                    let core = self.core.read().await;
                    let node = core.node();
                    let section_pk = *core.section().chain.last_key();
                    WireMsg::single_src(
                        node,
                        DstLocation::Section {
                            name: core.node().name(),
                            section_pk,
                        },
                        NodeMsg::StartConnectivityTest(name),
                        section_pk,
                    )?
                };
                let our_name = self.core.read().await.node().name();
                let peers = self
                    .core
                    .read()
                    .await
                    .section()
                    .active_members()
                    .filter(|peer| peer.name() != &name && peer.name() != &our_name)
                    .cloned()
                    .collect_vec();
                Ok(self.core.read().await.send_or_handle(msg, &peers))
            }
            Command::TestConnectivity(name) => {
                let mut commands = vec![];
                if let Some(peer) = self
                    .core
                    .read()
                    .await
                    .section()
                    .members()
                    .get(&name)
                    .map(|member_info| member_info.peer)
                {
                    if self
                        .core
                        .read()
                        .await
                        .comm
                        .is_reachable(peer.addr())
                        .await
                        .is_err()
                    {
                        commands.push(Command::ProposeOffline(*peer.name()));
                    }
                }
                Ok(commands)
            }
        }
    }

    async fn send_message(
        &self,
        recipients: &[(XorName, SocketAddr)],
        delivery_group_size: usize,
        wire_msg: WireMsg,
    ) -> Result<Vec<Command>> {
        let cmds = match wire_msg.msg_kind() {
            MsgKind::NodeSignedMsg(_)
            | MsgKind::NodeBlsShareSignedMsg(_)
            | MsgKind::SectionSignedMsg(_) => {
                let status = self
                    .core
                    .read()
                    .await
                    .comm
                    .send(recipients, delivery_group_size, wire_msg)
                    .await?;

                match status {
                    SendStatus::MinDeliveryGroupSizeReached(failed_recipients)
                    | SendStatus::MinDeliveryGroupSizeFailed(failed_recipients) => {
                        Ok(failed_recipients
                            .into_iter()
                            .map(Command::HandlePeerLost)
                            .collect())
                    }
                    _ => Ok(vec![]),
                }
                .map_err(|e: Error| e)?
            }
            MsgKind::ClientMsg(_) => {
                // TODO: send them all together without cloning WireMsg
                // by having the send_on_existing_connection to return SendStatus
                for (name, addr) in recipients {
                    if self
                        .core
                        .read()
                        .await
                        .comm
                        .send_on_existing_connection(&[(*name, *addr)], wire_msg.clone())
                        .await
                        .is_err()
                    {
                        trace!(
                            "Lost connection to client {:?} when sending message {:?}",
                            addr,
                            wire_msg
                        );
                        self.core
                            .read()
                            .await
                            .send_event(Event::ClientLost(*addr))
                            .await
                    }
                }
                vec![]
            }
            MsgKind::SectionInfoMsg => {
                let _ = self
                    .core
                    .read()
                    .await
                    .comm
                    .send_on_existing_connection(recipients, wire_msg)
                    .await;

                vec![]
            }
        };

        Ok(cmds)
    }

    async fn handle_schedule_timeout(&self, duration: Duration, token: u64) -> Option<Command> {
        let mut cancel_rx = self.cancel_timer_rx.clone();

        if *cancel_rx.borrow() {
            // Timers are already cancelled, do nothing.
            return None;
        }

        tokio::select! {
            _ = time::sleep(duration) => Some(Command::HandleTimeout(token)),
            _ = cancel_rx.changed() => None,
        }
    }

    async fn handle_relocation_complete(&self, new_node: Node, new_section: Section) {
        let previous_name = self.core.read().await.node().name();
        let new_keypair = new_node.keypair.clone();

        let mut core = self.core.write().await;
        *core = core.relocated(new_node, new_section).await;

        core.send_event(Event::Relocated {
            previous_name,
            new_keypair,
        })
        .await;
    }
}
