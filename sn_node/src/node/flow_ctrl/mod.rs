// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub(crate) mod cmd_ctrl;
pub(crate) mod cmds;
pub(super) mod dispatcher;
pub(super) mod event;
pub(super) mod event_channel;
mod periodic_checks;
#[cfg(test)]
pub(crate) mod tests;

pub(crate) use cmd_ctrl::CmdCtrl;
use periodic_checks::PeriodicChecksTimestamps;

use crate::comm::MsgEvent;
use crate::node::{flow_ctrl::cmds::Cmd, MyNode, Result};

use sn_interface::types::log_markers::LogMarker;

use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Listens for incoming msgs and forms Cmds for each,
/// Periodically triggers other Cmd Processes (eg health checks, dysfunction etc)
pub(crate) struct FlowCtrl {
    node: Arc<RwLock<MyNode>>,
    // cmd_ctrl: CmdCtrl,
    cmd_sender_channel: mpsc::Sender<(Cmd, Option<usize>)>,
    // outgoing_node_event_sender: EventSender,
    timestamps: PeriodicChecksTimestamps,
}

impl FlowCtrl {
    /// Constructs a FlowCtrl instance, spawnning a task which starts processing messages,
    /// returning the channel where it can receive commands on
    pub(crate) fn start(
        cmd_ctrl: CmdCtrl,
        mut incoming_msg_events: mpsc::Receiver<MsgEvent>,
        // outgoing_node_event_sender: EventSender,
    ) -> mpsc::Sender<(Cmd, Option<usize>)> {
        let dispatcher = cmd_ctrl.dispatcher.clone();
        let node = cmd_ctrl.node();
        let (cmd_sender_channel, mut incoming_cmds_from_apis) = mpsc::channel(10_000);
        let flow_ctrl = Self {
            // cmd_ctrl,
            node,
            cmd_sender_channel: cmd_sender_channel.clone(),
            // outgoing_node_event_sender,
            timestamps: PeriodicChecksTimestamps::now(),
        };

        let _ =
            tokio::task::spawn(
                async move { flow_ctrl.process_messages_and_periodic_checks().await },
            );

        let cmd_channel = cmd_sender_channel.clone();
        let cmd_channel_for_msgs = cmd_sender_channel.clone();

        // start a new thread to kick off incoming cmds
        let _ = tokio::task::spawn(async move {
            while let Some((cmd, cmd_id)) = incoming_cmds_from_apis.recv().await {
                CmdCtrl::process_cmd_job(dispatcher.clone(), cmd, cmd_id, cmd_channel.clone()).await
            }
        });

        // start a new thread to convert msgs to Cmds
        let _ = tokio::task::spawn(async move {
            while let Some(peer_msg) = incoming_msg_events.recv().await {
                let cmd = match Self::handle_new_msg_event(peer_msg).await {
                    Ok(cmd) => cmd,
                    Err(error) => {
                        error!("Could not handle incoming msg event: {error:?}");
                        continue;
                    }
                };

                if let Err(error) = cmd_channel_for_msgs.send((cmd.clone(), None)).await {
                    error!("Error sending msg onto cmd channel {error:?}");
                }
            }
        });

        cmd_sender_channel
    }

    /// This is a never ending loop as long as the node is live.
    /// This loop drives the periodic events internal to the node.
    async fn process_messages_and_periodic_checks(mut self) {
        // the internal process loop
        loop {
            debug!(" ----> Before checks");

            self.perform_periodic_checks().await;
            debug!(" ----> After checks");

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    // Listen for a new incoming connection event and handle it.
    async fn handle_new_msg_event(event: MsgEvent) -> Result<Cmd> {
        match event {
            MsgEvent::Received {
                sender,
                wire_msg,
                send_stream,
            } => {
                let (header, dst, payload) = wire_msg.serialize()?;
                let original_bytes_len = header.len() + dst.len() + payload.len();

                debug!(
                    "New message ({} bytes) received from: {:?}",
                    original_bytes_len, sender
                );

                let span = {
                    // let name = node_info.name();
                    trace_span!("handle_message", ?sender, msg_id = ?wire_msg.msg_id())
                };
                let _span_guard = span.enter();

                trace!(
                    "{:?} from {:?} length {}",
                    LogMarker::DispatchHandleMsgCmd,
                    sender,
                    original_bytes_len
                );

                #[cfg(feature = "test-utils")]
                let wire_msg = if let Ok(msg) = wire_msg.into_msg() {
                    wire_msg.set_payload_debug(msg)
                } else {
                    wire_msg
                };

                Ok(Cmd::ValidateMsg {
                    origin: sender,
                    wire_msg,
                    send_stream,
                })
            }
        }
    }
}
