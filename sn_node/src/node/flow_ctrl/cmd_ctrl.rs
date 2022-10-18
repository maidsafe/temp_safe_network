// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::flow_ctrl::{cmds::Cmd, dispatcher::Dispatcher};

use std::sync::Arc;
use tokio::sync::RwLock;
use xor_name::XorName;

/// A module for enhanced flow control.
///
/// Orders the incoming cmds (work) according to their priority,
/// and executes them in a controlled way, taking into
/// account the rate limits of our system load monitoring.
///
/// Stacking up direct method calls in the async runtime is sort of
/// like saying to a node "do everything everyone is asking of you, now".
/// We're now saying to a node "do as much as you can without choking,
/// and start with the most important things first".
pub(crate) struct CmdCtrl {
    pub(crate) dispatcher: Arc<Dispatcher>,
    #[allow(dead_code)]
    id_counter: usize,
}

impl CmdCtrl {
    pub(crate) fn new(dispatcher: Dispatcher) -> Self {
        #[cfg(feature = "statemap")]
        sn_interface::statemap::log_metadata();

        Self {
            dispatcher: Arc::new(dispatcher),
            id_counter: 0,
        }
    }

    pub(crate) fn node(&self) -> Arc<RwLock<crate::node::MyNode>> {
        self.dispatcher.node()
    }

    /// Processes the next priority cmd
    pub(crate) async fn process_cmd_job(
        dispatcher: Arc<Dispatcher>,
        cmd: Cmd,
        id: Option<usize>,
        node_identifier: XorName,
        cmd_process_api: tokio::sync::mpsc::Sender<(Cmd, Option<usize>)>,
    ) {
        trace!("Processing cmd: {cmd:?}");

        trace!("about to spawn for processing cmd: {cmd:?}");
        let _ = tokio::task::spawn(async move {
            debug!("> spawned process for cmd {cmd:?}");

            #[cfg(feature = "statemap")]
            sn_interface::statemap::log_state(node_identifier.to_string(), cmd.statemap_state());

            debug!("* spawned process for cmd {cmd:?}, node read done");
            match dispatcher.process_cmd(cmd).await {
                Ok(cmds) => {
                    for cmd in cmds {
                        match cmd_process_api.send((cmd, id)).await {
                            Ok(_) => {
                                //no issues
                            }
                            Err(error) => {
                                error!("Could not enqueue child Cmd: {:?}", error);
                            }
                        }
                    }
                }
                Err(error) => {
                    debug!("Error when processing command: {:?}", error);
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
