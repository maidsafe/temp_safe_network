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
use event_channel::EventSender;
use periodic_checks::PeriodicChecksTimestamps;

use crate::comm::MsgEvent;
use crate::node::{flow_ctrl::cmds::Cmd, Error, MyNode, Result};

use sn_interface::types::log_markers::LogMarker;

use std::sync::Arc;
use tokio::sync::{
    mpsc::{self, error::TryRecvError},
    RwLock,
};

// const PROCESS_BATCH_COUNT: usize = 25;

/// Listens for incoming msgs and forms Cmds for each,
/// Periodically triggers other Cmd Processes (eg health checks, dysfunction etc)
pub(crate) struct FlowCtrl {
    node: Arc<RwLock<MyNode>>,
    cmd_ctrl: CmdCtrl,
    // incoming_msg_events: mpsc::Receiver<MsgEvent>,
    // incoming_cmds_from_apis: mpsc::Receiver<(Cmd, Option<usize>)>,
    cmd_sender_channel: mpsc::Sender<(Cmd, Option<usize>)>,
    outgoing_node_event_sender: EventSender,
    timestamps: PeriodicChecksTimestamps,
}

impl FlowCtrl {
    /// Constructs a FlowCtrl instance, spawnning a task which starts processing messages,
    /// returning the channel where it can receive commands on
    pub(crate) fn start(
        cmd_ctrl: CmdCtrl,
        mut incoming_msg_events: mpsc::Receiver<MsgEvent>,
        outgoing_node_event_sender: EventSender,
    ) -> mpsc::Sender<(Cmd, Option<usize>)> {
        let dispatcher = cmd_ctrl.dispatcher.clone();
        let node = cmd_ctrl.node();
        let (cmd_sender_channel, mut incoming_cmds_from_apis) = mpsc::channel(10_000);
        let flow_ctrl = Self {
            cmd_ctrl,
            node,
            // incoming_msg_events,
            // incoming_cmds_from_apis,
            cmd_sender_channel: cmd_sender_channel.clone(),
            outgoing_node_event_sender,
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
                // Self::process_next_cmd(cmd).await;

                CmdCtrl::process_cmd_job(
                    dispatcher.clone(),
                    cmd,
                    cmd_id,
                    cmd_channel.clone(),
                    // self.outgoing_node_event_sender.clone(),
                )
                .await
            }
        });

        // start a new thread to convert msgs to Cmds
        let _ = tokio::task::spawn(async move {
            while let Some(peer_msg) = incoming_msg_events.recv().await {
                // Self::process_next_cmd(cmd).await;

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

                // CmdCtrl::process_cmd_job(
                //     dispatcher.clone(),
                //     cmd,
                //     cmd_id,
                //     cmd_channel.clone(),
                //     // self.outgoing_node_event_sender.clone(),
                // )
                // .await
            }
        });

        cmd_sender_channel
    }

    // /// Process the next pending cmds
    // async fn process_next_cmd(&self, ) {

    //     let dispatcher = self.cmd_ctrl.dispatcher.clone();
    //     // if let Some(next_cmd_job) = self.cmd_ctrl.next_cmd() {
    //         CmdCtrl::process_cmd_job(
    //                 dispatcher,
    //                 next_cmd_job,
    //                 self.cmd_sender_channel.clone(),
    //                 self.outgoing_node_event_sender.clone(),
    //             )
    //             .await
    //     // }
    // }

    // /// Pull and queue up all pending cmds from the CmdChannel
    // async fn enqeue_new_cmds_from_channel(&mut self) -> Result<()> {
    //     loop {
    //         match self.incoming_cmds_from_apis.try_recv() {
    //             Ok((cmd, id)) => self.fire_and_forget(cmd, id).await,
    //             Err(TryRecvError::Empty) => {
    //                 // do nothing else
    //                 return Ok(());
    //             }
    //             Err(TryRecvError::Disconnected) => {
    //                 error!("Senders to `incoming_cmds_from_apis` have disconnected.");
    //                 return Err(Error::CmdCtrlChannelDropped);
    //             }
    //         }
    //     }
    // }

    /// Pull and queue up all pending msgs from the MsgSender
    // async fn enqueue_new_incoming_msgs(&mut self) -> Result<()> {
    //     loop {
    //         match self.incoming_msg_events.try_recv() {
    //             Ok(msg) => {
    //                 let cmd = match Self::handle_new_msg_event(msg).await {
    //                     Ok(cmd) => cmd,
    //                     Err(error) => {
    //                         error!("Could not handle incoming msg event: {error:?}");
    //                         continue;
    //                     }
    //                 };

    //                 if let Err(error) = self.cmd_sender_channel.send((cmd.clone(), None)).await {
    //                     error!("Error sending msg onto cmd channel {error:?}");
    //                 }
    //             }
    //             Err(TryRecvError::Empty) => {
    //                 // do nothing else
    //                 return Ok(());
    //             }
    //             Err(TryRecvError::Disconnected) => {
    //                 error!("Senders to `incoming_cmds_from_apis` have disconnected.");
    //                 return Err(Error::MsgChannelDropped);
    //             }
    //         }
    //     }
    // }

    /// This is a never ending loop as long as the node is live.
    /// This loop drives the periodic events internal to the node.
    async fn process_messages_and_periodic_checks(mut self) {
        debug!("Starting internal processing...");
        // the internal process loop
        loop {
            debug!(" ---------------------------------------------->>>>>> ");
            debug!(" ----> Starting the process loop");
            // if we want to throttle cmd throughput, we do that here.
            // if there is nothing in the cmd queue, we wait here too.
            // self.cmd_ctrl
            //     .wait_if_not_processing_at_expected_rate()
            //     .await;

            // if let Err(error) = self.enqueue_new_incoming_msgs().await {
            //     error!("{error:?}");
            //     break;
            // }

            debug!(" ----> New msgs enqueued");

            // if let Err(error) = self.enqeue_new_cmds_from_channel().await {
            //     error!("{error:?}");
            //     break;
            // }

            // debug!(" ----> New cmds enqueued");
            // we go through all pending cmds in this loop
            // while self.cmd_ctrl.has_items_queued() {
            //     debug!(" ----> About to kick cmd off");
            //     self.process_next_cmd().await;

            //     debug!(" ----> About to Q more cmds");
            //     if let Err(error) = self.enqeue_new_cmds_from_channel().await {
            //         error!("{error:?}");
            //         break;
            //     }

            //     // if let Err(error) = self.enqueue_new_incoming_msgs().await {
            //     //     error!("{error:?}");
            //     //     break;
            //     // }
            // }

            debug!(" ----> Before checks");

            self.perform_periodic_checks().await;
            debug!(" ----> After checks");

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        error!("Internal processing ended.")
    }

    /// Does not await the completion of the cmd.
    pub(crate) async fn fire_and_forget(&mut self, cmd: Cmd, parent_id: Option<usize>) {
        trace!("Enqueuing cmd: {cmd:?}");
        self.cmd_ctrl.push(cmd, parent_id).await
    }

    // Listen for a new incoming connection event and handle it.
    async fn handle_new_msg_event(event: MsgEvent) -> Result<Cmd> {
        match event {
            MsgEvent::Received { sender, wire_msg } => {
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
                })
            }
        }
    }
}
