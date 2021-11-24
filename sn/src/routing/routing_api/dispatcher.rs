// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Command, Event};
use crate::messaging::{
    serialisation::{
        AE_MSG_PRIORITY, DKG_MSG_PRIORITY, INFRASTRUCTURE_MSG_PRIORITY, JOIN_RESPONSE_PRIORITY,
        NODE_DATA_MSG_PRIORITY,
    },
    system::SystemMsg,
    DstLocation, EndUser, MsgKind, WireMsg,
};
use crate::routing::{
    core::{Core, Proposal, SendStatus},
    error::Result,
    log_markers::LogMarker,
    network_knowledge::NetworkKnowledge,
    node::Node,
    Error, Peer,
};
use std::collections::BTreeMap;
use std::{sync::Arc, time::Duration};
use tokio::time::MissedTickBehavior;
use tokio::{
    sync::{watch, OwnedSemaphorePermit, RwLock, Semaphore},
    time,
};
use tracing::Instrument;

const PROBE_INTERVAL: Duration = Duration::from_secs(30);
const CONCURRENCY_COUNT: usize = 7;

// A command/subcommand id e.g. "963111461", "963111461.0"
type CmdId = String;

fn get_root_cmd_id(cmd_id: &str) -> CmdId {
    let mut root_cmd_id = cmd_id.to_string();
    root_cmd_id.truncate(cmd_id.find('.').unwrap_or_else(|| cmd_id.len()));
    root_cmd_id
}

// `Command` Dispatcher.
pub(super) struct Dispatcher {
    pub(super) core: Core,

    cancel_timer_tx: watch::Sender<bool>,
    cancel_timer_rx: watch::Receiver<bool>,
    join_permits: Arc<Semaphore>,
    ae_permits: Arc<Semaphore>,
    infra_permits: Arc<Semaphore>,
    node_data_permits: Arc<Semaphore>,
    dkg_permits: Arc<Semaphore>,
    cmd_permit_map: Arc<RwLock<BTreeMap<CmdId, OwnedSemaphorePermit>>>,
}

impl Drop for Dispatcher {
    fn drop(&mut self) {
        // Cancel all scheduled timers including any future ones.
        let _res = self.cancel_timer_tx.send(true);
    }
}

impl Dispatcher {
    pub(super) fn new(core: Core) -> Self {
        let (cancel_timer_tx, cancel_timer_rx) = watch::channel(false);
        Self {
            core,
            cancel_timer_tx,
            cancel_timer_rx,
            join_permits: Arc::new(Semaphore::new(CONCURRENCY_COUNT)),
            ae_permits: Arc::new(Semaphore::new(CONCURRENCY_COUNT)),
            infra_permits: Arc::new(Semaphore::new(CONCURRENCY_COUNT)),
            dkg_permits: Arc::new(Semaphore::new(CONCURRENCY_COUNT)),
            node_data_permits: Arc::new(Semaphore::new(CONCURRENCY_COUNT)),
            cmd_permit_map: Arc::new(RwLock::new(BTreeMap::default())),
        }
    }

    /// block progress until there are no tasks pending in this semaphore
    /// intended to allow us to wait for super high priority tasks before doing others...
    /// It should only be used after checking that no permits are held by a root cmd eg
    async fn wait_for_priority_commands_to_finish(&self, semaphore: Arc<Semaphore>, count: usize) {
        // there's probably a neater way to do this
        while semaphore.available_permits() != count {
            time::sleep(Duration::from_millis(100)).await
        }
    }

    /// Based upon message priority will wait for any higher priority commands to be completed before continuing
    async fn acquire_permit_or_wait(&self, prio: i32, cmd_id: CmdId) {
        let root_cmd_id = get_root_cmd_id(&cmd_id);

        let permit = match prio {
            JOIN_RESPONSE_PRIORITY => Some(
                self.join_permits
                    .clone()
                    .acquire_owned()
                    .await
                    .map_err(|_| Error::SemaphoreClosed),
            ),
            DKG_MSG_PRIORITY => {
                self.wait_for_priority_commands_to_finish(
                    self.join_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;

                Some(
                    self.join_permits
                        .clone()
                        .acquire_owned()
                        .await
                        .map_err(|_| Error::SemaphoreClosed),
                )
            }
            AE_MSG_PRIORITY => {
                self.wait_for_priority_commands_to_finish(
                    self.join_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;
                self.wait_for_priority_commands_to_finish(
                    self.dkg_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;

                Some(
                    self.ae_permits
                        .clone()
                        .acquire_owned()
                        .await
                        .map_err(|_| Error::SemaphoreClosed),
                )
            }
            INFRASTRUCTURE_MSG_PRIORITY => {
                self.wait_for_priority_commands_to_finish(
                    self.join_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;
                self.wait_for_priority_commands_to_finish(
                    self.dkg_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;
                self.wait_for_priority_commands_to_finish(
                    self.ae_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;

                Some(
                    self.infra_permits
                        .clone()
                        .acquire_owned()
                        .await
                        .map_err(|_| Error::SemaphoreClosed),
                )
            }
            NODE_DATA_MSG_PRIORITY => {
                self.wait_for_priority_commands_to_finish(
                    self.join_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;
                self.wait_for_priority_commands_to_finish(
                    self.dkg_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;
                self.wait_for_priority_commands_to_finish(
                    self.ae_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;
                self.wait_for_priority_commands_to_finish(
                    self.infra_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;

                Some(
                    self.node_data_permits
                        .clone()
                        .acquire_owned()
                        .await
                        .map_err(|_| Error::SemaphoreClosed),
                )
            }
            _ => {
                self.wait_for_priority_commands_to_finish(
                    self.join_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;
                self.wait_for_priority_commands_to_finish(
                    self.dkg_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;
                self.wait_for_priority_commands_to_finish(
                    self.ae_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;
                self.wait_for_priority_commands_to_finish(
                    self.infra_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;
                self.wait_for_priority_commands_to_finish(
                    self.node_data_permits.clone(),
                    CONCURRENCY_COUNT,
                )
                .await;

                None
            }
        };

        if let Some(permit) = permit {
            match permit {
                Ok(permit) => {
                    let permit_map = self.cmd_permit_map.clone();
                    let mut permit_map_write_guard = permit_map.write().await;
                    let _old_permit = permit_map_write_guard.insert(root_cmd_id, permit);
                }
                Err(error) => {
                    // log error, it can only be permi acquisition here, so that's okay and we ignore it / drop command as we've bigger issues
                    error!("{:?}", error)
                }
            }
        }
    }

    /// Enqueues the given command and handles whatever command is in the next priority queue and triggers handling after any required waits for higher priority tasks
    pub(super) async fn enqueue_and_handle_next_command_and_any_offshoots(
        self: Arc<Self>,
        command: Command,
        cmd_id: Option<String>,
    ) -> Result<()> {
        let _ = tokio::spawn(async {
            let cmd_id = cmd_id.unwrap_or_else(|| rand::random::<u32>().to_string());
            self.acquire_permit_or_wait(command.priority()?, cmd_id.clone())
                .await;
            self.handle_command_and_any_offshoots(command, cmd_id).await
        });
        Ok(())
    }

    /// Handles command and transitively queues any new commands that are
    /// produced during its handling. Trace logs will include the provided command id,
    /// and any sub-commands produced will have it as a common root cmd id.
    /// If a command id string is not provided a random one will be generated.
    pub(super) async fn handle_command_and_any_offshoots(
        self: Arc<Self>,
        command: Command,
        cmd_id: String,
    ) -> Result<()> {
        let cmd_id_clone = cmd_id.clone();
        let command_display = command.to_string();
        let _ = tokio::spawn(async move {
            if let Ok(commands) = self.process_command(command, &cmd_id).await {
                for (sub_cmd_count, command) in commands.into_iter().enumerate() {
                    let sub_cmd_id = format!("{}.{}", cmd_id, sub_cmd_count);
                    // Error here is only related to queueing, and so a dropped command will be logged
                    let _result = self.clone().spawn_handle_commands(command, sub_cmd_id);
                }
            }
        });

        trace!(
            "{:?} {} cmd_id={}",
            LogMarker::CommandHandleSpawned,
            command_display,
            cmd_id_clone
        );

        Ok(())
    }

    // Note: this indirecton is needed. Trying to call `spawn(self.handle_commands(...))` directly
    // inside `handle_commands` causes compile error about type check cycle.
    fn spawn_handle_commands(self: Arc<Self>, command: Command, cmd_id: String) -> Result<()> {
        // self.enqueue_command(command)?;
        // if let Some(command) = self.get_next_command_to_handle() {
        let _ = tokio::spawn(
            self.enqueue_and_handle_next_command_and_any_offshoots(command, Some(cmd_id)),
        );
        // }
        Ok(())
    }

    pub(super) async fn start_network_probing(self: Arc<Self>) {
        info!("Starting to probe network");
        let _handle = tokio::spawn(async move {
            let dispatcher = self.clone();
            let mut interval = tokio::time::interval(PROBE_INTERVAL);
            interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

            loop {
                let _instant = interval.tick().await;

                // Send a probe message if we are an elder
                let core = &dispatcher.core;
                if core.is_elder().await && !core.network_knowledge().prefix().await.is_empty() {
                    match core.generate_probe_message().await {
                        Ok(command) => {
                            info!("Sending ProbeMessage");
                            if let Err(e) = dispatcher
                                .clone()
                                .enqueue_and_handle_next_command_and_any_offshoots(command, None)
                                .await
                            {
                                error!("Error sending a Probe message to the network: {:?}", e);
                            }
                        }
                        Err(error) => error!("Problem generating probe message: {:?}", error),
                    }
                }
            }
        });
    }

    pub(super) async fn write_prefixmap_to_disk(self: Arc<Self>) {
        info!("Writing our PrefixMap to disk");
        self.clone().core.write_prefix_map().await
    }

    /// Handles a single command.
    pub(super) async fn process_command(
        &self,
        command: Command,
        cmd_id: &str,
    ) -> Result<Vec<Command>> {
        // Create a tracing span containing info about the current node. This is very useful when
        // analyzing logs produced by running multiple nodes within the same process, for example
        // from integration tests.
        let span = {
            let core = &self.core;

            let prefix = core.network_knowledge().prefix().await;
            let is_elder = core.is_elder().await;
            let section_key = core.network_knowledge().section_key().await;
            let age = core.node.read().await.age();
            trace_span!(
                "handle_command",
                name = %core.node.read().await.name(),
                prefix = format_args!("({:b})", prefix),
                age,
                elder = is_elder,
                cmd_id = %cmd_id,
                section_key = ?section_key,
                %command,
            )
        };

        async {
            trace!("{:?}", LogMarker::CommandHandleStart);

            let command_display = command.to_string();
            let res = match self.try_processing_a_command(command).await {
                Ok(outcome) => {
                    trace!("{:?} {}", LogMarker::CommandHandleEnd, command_display);

                    Ok(outcome)
                }
                Err(error) => {
                    error!(
                        "Error encountered when handling command (cmd_id {}): {:?}",
                        cmd_id, error
                    );
                    trace!(
                        "{:?} {}: {:?}",
                        LogMarker::CommandHandleError,
                        command_display,
                        error
                    );
                    Err(error)
                }
            };
            // and now we're done, free up the permit
            let root_cmd_id = get_root_cmd_id(cmd_id);
            let permit_map = self.cmd_permit_map.clone();
            let mut permit_map_write_guard = permit_map.write().await;
            let _used_permit = permit_map_write_guard.remove(&root_cmd_id);
            res
        }
        .instrument(span)
        .await
    }

    /// Actually process the command
    async fn try_processing_a_command(&self, command: Command) -> Result<Vec<Command>> {
        match command {
            Command::HandleSystemMessage {
                sender,
                msg_id,
                msg_authority,
                dst_location,
                msg,
                payload,
                known_keys,
            } => {
                self.core
                    .handle_system_message(
                        sender,
                        msg_id,
                        msg_authority,
                        dst_location,
                        msg,
                        payload,
                        known_keys,
                    )
                    .await
            }
            Command::PrepareNodeMsgToSend { msg, dst } => {
                self.core.prepare_node_msg(msg, dst).await
            }
            Command::HandleMessage {
                sender,
                wire_msg,
                original_bytes,
            } => {
                self.core
                    .handle_message(sender, wire_msg, original_bytes)
                    .await
            }
            Command::HandleTimeout(token) => self.core.handle_timeout(token).await,
            Command::HandleAgreement { proposal, sig } => {
                self.core.handle_general_agreements(proposal, sig).await
            }
            Command::HandleNewNodeOnline(auth) => {
                self.core
                    .handle_online_agreement(auth.value.into_state(), auth.sig)
                    .await
            }
            Command::HandleElderAgreement { proposal, sig } => match proposal {
                Proposal::OurElders(section_auth) => {
                    self.core
                        .handle_our_elders_agreement(section_auth, sig)
                        .await
                }
                _ => {
                    error!("Other agreement messages should be handled in `HandleAgreement`, which is non-blocking ");
                    Ok(vec![])
                }
            },
            Command::HandlePeerLost(peer) => self.core.handle_peer_lost(&peer.addr()).await,
            Command::HandleDkgOutcome {
                section_auth,
                outcome,
            } => self.core.handle_dkg_outcome(section_auth, outcome).await,
            Command::HandleDkgFailure(signeds) => self
                .core
                .handle_dkg_failure(signeds)
                .await
                .map(|command| vec![command]),
            Command::SendMessage {
                recipients,
                wire_msg,
            } => {
                self.send_message(&recipients, recipients.len(), wire_msg)
                    .await
            }
            Command::SendMessageDeliveryGroup {
                recipients,
                delivery_group_size,
                wire_msg,
            } => {
                self.send_message(&recipients, delivery_group_size, wire_msg)
                    .await
            }
            Command::ParseAndSendWireMsg(wire_msg) => self.send_wire_message(wire_msg).await,
            Command::ScheduleTimeout { duration, token } => Ok(self
                .handle_schedule_timeout(duration, token)
                .await
                .into_iter()
                .collect()),
            Command::HandleRelocationComplete { node, section } => {
                self.handle_relocation_complete(node, section).await?;
                Ok(vec![])
            }
            Command::SetJoinsAllowed(joins_allowed) => {
                self.core.set_joins_allowed(joins_allowed).await
            }
            Command::SendAcceptedOnlineShare {
                peer,
                previous_name,
            } => {
                self.core
                    .send_accepted_online_share(peer, previous_name)
                    .await
            }
            Command::ProposeOffline(name) => self.core.propose_offline(name).await,
            Command::StartConnectivityTest(name) => Ok(vec![
                self.core
                    .send_message_to_our_elders(SystemMsg::StartConnectivityTest(name))
                    .await?,
            ]),
            Command::TestConnectivity(name) => {
                let mut commands = vec![];
                if let Some(member_info) = self.core.network_knowledge().members().get(&name) {
                    if self
                        .core
                        .comm
                        .is_reachable(&member_info.addr())
                        .await
                        .is_err()
                    {
                        commands.push(Command::ProposeOffline(member_info.name()));
                    }
                }
                Ok(commands)
            }
        }
    }

    async fn send_message(
        &self,
        recipients: &[Peer],
        delivery_group_size: usize,
        wire_msg: WireMsg,
    ) -> Result<Vec<Command>> {
        let cmds = match wire_msg.msg_kind() {
            MsgKind::NodeAuthMsg(_) | MsgKind::NodeBlsShareAuthMsg(_) => {
                self.deliver_messages(recipients, delivery_group_size, wire_msg)
                    .await?
            }
            MsgKind::ServiceMsg(_) => {
                let _res = self
                    .core
                    .comm
                    .send_on_existing_connection(recipients, wire_msg)
                    .await;

                vec![]
            }
        };

        Ok(cmds)
    }

    async fn deliver_messages(
        &self,
        recipients: &[Peer],
        delivery_group_size: usize,
        wire_msg: WireMsg,
    ) -> Result<Vec<Command>> {
        let status = self
            .core
            .comm
            .send(recipients, delivery_group_size, wire_msg)
            .await?;

        match status {
            SendStatus::MinDeliveryGroupSizeReached(failed_recipients)
            | SendStatus::MinDeliveryGroupSizeFailed(failed_recipients) => Ok(failed_recipients
                .into_iter()
                .map(Command::HandlePeerLost)
                .collect()),
            _ => Ok(vec![]),
        }
        .map_err(|e: Error| e)
    }

    /// Send a message, either section to section, node to node, or to an end user.
    pub(super) async fn send_wire_message(&self, wire_msg: WireMsg) -> Result<Vec<Command>> {
        if let DstLocation::EndUser(EndUser(_)) = wire_msg.dst_location() {
            error!(
                "End user msg dropped at send. You need to remember the Peer, and use a different send API for service messages.",
            );
            Ok(vec![])
        } else {
            // This message is not for an end user, then send it to peer/s over the network
            let cmd = self.core.send_msg_to_peers(wire_msg).await?;
            Ok(vec![cmd])
        }
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

    async fn handle_relocation_complete(
        &self,
        new_node: Node,
        new_section: NetworkKnowledge,
    ) -> Result<()> {
        let previous_name = self.core.node.read().await.name();
        let new_keypair = new_node.keypair.clone();
        let age = new_node.age();
        self.core.relocate(new_node, new_section).await?;

        self.core
            .send_event(Event::Relocated {
                previous_name,
                new_keypair,
            })
            .await;

        info!("Relocated, our Age: {:?}", age);

        Ok(())
    }
}
