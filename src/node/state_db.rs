// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{Config, Result, node::node_duties::AgeLevel};
use crate::{
    node::{
        node_ops::{
            GroupDecision, MessagingDuty, NodeDuty, NodeOperation, GatewayDuty, PaymentDuty,
            MetadataDuty, ChunkDuty, RewardDuty, TransferDuty, ElderDuty, AdultDuty,
        },
        node_duties::{NodeDuties, messaging::{Receiver, Received}},
        adult_duties::AdultDuties,
        elder_duties::ElderDuties,
        keys::NodeKeys,
    },
    utils,
};
use log::{error, info, warn};
use routing::Node as Routing;
use safe_nd::NodeFullId;
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    fs,
    net::SocketAddr,
    path::Path,
    rc::Rc,
};

const STATE_FILENAME: &str = "state";

pub fn dump_state(is_elder: bool, root_dir: &Path, id: NodeFullId) -> Result<()> {
    let path = self.root_dir.join(STATE_FILENAME);
    Ok(fs::write(path, utils::serialise(&(is_elder, &self.id)))?)
}

/// Returns Some((is_elder, ID)) or None if file doesn't exist.
pub fn read_state(root_dir: &Path) -> Result<Option<(bool, NodeFullId)>> {
    let path = root_dir.join(STATE_FILENAME);
    if !path.is_file() {
        return Ok(None);
    }
    let contents = fs::read(path)?;
    Ok(Some(bincode::deserialize(&contents)?))
}

/// Specifies whether to try loading cached data from disk, or to just construct a new instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Init {
    Load,
    New,
}

/// Command that the user can send to a running node to control its execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Shutdown the vault
    Shutdown,
}

