// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    flow_ctrl::{cmds::Cmd, dispatcher::Dispatcher, RejoinReason},
    Error,
};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::{mpsc, RwLock, Semaphore};
use xor_name::XorName;

/// 150 is an initial value, this could be dialled i, or even adjusted via env var.
/// 150 seemed to work well lcoally, with a reduction in long read times, while
/// maintaining stability
const DEFAULT_MSG_CONCURRENCY: usize = 150;
/// Takes care of spawning a new task for the processing of a cmd,
/// collecting resulting cmds from it, and sending it back to the calling context,
/// all the while logging the correlation between incoming and resulting cmds.
pub(crate) struct CmdCtrl {
    pub(crate) dispatcher: Arc<Dispatcher>,
    id_counter: Arc<AtomicUsize>,
    concurrency_limiter: Arc<Semaphore>,
}

impl CmdCtrl {
    pub(crate) fn new(dispatcher: Dispatcher) -> Self {
        #[cfg(feature = "statemap")]
        sn_interface::statemap::log_metadata();

        Self {
            dispatcher: Arc::new(dispatcher),
            id_counter: Arc::new(AtomicUsize::new(0)),
            concurrency_limiter: Arc::new(Semaphore::new(DEFAULT_MSG_CONCURRENCY)),
        }
    }

    pub(crate) fn node(&self) -> Arc<RwLock<crate::node::MyNode>> {
        self.dispatcher.node()
    }

    /// Processes the passed in cmd on a new task
    pub(crate) fn process_cmd_job(
        &self,
        cmd: Cmd,
        mut id: Vec<usize>,
        node_identifier: XorName,
        cmd_process_api: mpsc::Sender<(Cmd, Vec<usize>)>,
        rejoin_network_sender: mpsc::Sender<RejoinReason>,
    ) {
        if id.is_empty() {
            id.push(self.id_counter.fetch_add(1, Ordering::SeqCst));
        }

        let dispatcher = self.dispatcher.clone();
        let is_handle_msg = cmd.is_handle_msg();
        let is_response_msg = cmd.is_response_msg();

        debug!("Is handle msg: {is_handle_msg:?}");
        // TODO: adjust concurrency limit per CPU %.
        // Feedback if we can't get in right now? From here, or lower level.
        // Then client retries?
        let concurrency_limiter = self.concurrency_limiter.clone();

        let _handle = tokio::task::spawn(async move {
            let permit = if is_handle_msg && !is_response_msg {
                trace!("Await permit to process for unknown-node HandleMsg {cmd:?}, id: {id:?}");
                let permit = match concurrency_limiter.acquire_owned().await {
                    Ok(p) => p,
                    Err(error) => {
                        error!(
                            "Not processing {cmd:?}, id: {id:?}, acquiring permit failed: {error:?}"
                        );

                        return;
                    }
                };
                trace!("Permit acquired to process for unknown-node HandleMsg {cmd:?}, id: {id:?}");
                Some(permit)
            } else {
                None
            };

            trace!("Processing for {cmd:?}, id: {id:?}");

            #[cfg(feature = "statemap")]
            sn_interface::statemap::log_state(node_identifier.to_string(), cmd.statemap_state());

            match dispatcher.process_cmd(cmd).await {
                Ok(cmds) => {
                    drop(permit);
                    let _handle = tokio::task::spawn(async move {
                        for (child_nr, cmd) in cmds.into_iter().enumerate() {
                            // zero based, first child of first cmd => [0, 0], second child => [0, 1], first child of second child => [0, 1, 0]
                            let child_id = [id.clone(), [child_nr].to_vec()].concat();
                            match cmd_process_api.send((cmd, child_id)).await {
                                Ok(_) => (), // no issues
                                Err(error) => {
                                    let child_id = [id.clone(), [child_nr].to_vec()].concat();
                                    error!(
                                    "Could not enqueue child cmd with id: {child_id:?}: {error:?}",
                                );
                                }
                            }
                        }
                    });
                }
                Err(error) => {
                    drop(permit);
                    warn!("Error when processing cmd: {:?}", error);
                    if let Error::RejoinRequired(reason) = error {
                        if rejoin_network_sender.send(reason).await.is_err() {
                            error!("Could not send rejoin reason through channel.");
                        }
                    }
                }
            }

            #[cfg(feature = "statemap")]
            sn_interface::statemap::log_state(
                node_identifier.to_string(),
                sn_interface::statemap::State::Idle,
            );
        });
    }
}
