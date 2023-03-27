// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::operations::node::{
    node_events, node_info, node_restart, node_stop, node_update, section_members,
};

use super::OutputFmt;

use clap::Subcommand;
use color_eyre::Result;
use std::net::SocketAddr;

/// Subcommands to send requests to safenode RPC interface
#[derive(Subcommand, Debug)]
pub enum NodeSubCommands {
    /// Retrieve information about the node iself
    #[clap(name = "info")]
    Info,
    /// Retrieve a list of section members the node is aware of
    #[clap(name = "section")]
    SectionMembers,
    /// Start listening for node events.
    /// Note this blocks the CLI and it will print events as they are broadcasted by the node
    #[clap(name = "events")]
    Events,
    /// Restart the node after the specified delay
    #[clap(name = "restart")]
    Restart {
        /// Delay in milliseconds before restartng the node
        #[clap(default_value = "0")]
        delay_millis: u64,
    },
    /// Stop the node after the specified delay
    #[clap(name = "stop")]
    Stop {
        /// Delay in milliseconds before stopping the node
        #[clap(default_value = "0")]
        delay_millis: u64,
    },
    #[clap(name = "update")]
    /// Update to latest `safenode` released version, and restart it
    Update {
        /// Delay in milliseconds before updating and restarting the node
        #[clap(default_value = "0")]
        delay_millis: u64,
    },
}

pub async fn node_commander(
    cmd: NodeSubCommands,
    addr: SocketAddr,
    output_fmt: OutputFmt,
) -> Result<()> {
    match cmd {
        NodeSubCommands::Info => node_info(addr, output_fmt).await,
        NodeSubCommands::SectionMembers => section_members(addr, output_fmt).await,
        NodeSubCommands::Events => node_events(addr, output_fmt).await,
        NodeSubCommands::Restart { delay_millis } => node_restart(addr, delay_millis).await,
        NodeSubCommands::Stop { delay_millis } => node_stop(addr, delay_millis).await,
        NodeSubCommands::Update { delay_millis } => node_update(addr, delay_millis).await,
    }
}
