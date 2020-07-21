// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{Config, Result, node::node_duties::DutyLevel};
use crate::{
    node::keys::NodeKeys,
    utils,
};
use log::{error, info, warn};
use routing::Node as Routing;
use serde::{Serialize, Deserialize};
use safe_nd::{NodeFullId, NodePublicId};
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    rc::Rc,
};

const STATE_FILENAME: &str = "state";

pub fn dump_state(age_group: AgeGroup, root_dir: &Path, id: &NodeFullId) -> Result<()> {
    let path = root_dir.join(STATE_FILENAME);
    Ok(fs::write(path, utils::serialise(&(age_group, id)))?)
}

/// Returns Some((age_group, ID)) or None if file doesn't exist.
pub fn read_state(root_dir: &Path) -> Result<Option<(AgeGroup, NodeFullId)>> {
    let path = root_dir.join(STATE_FILENAME);
    if !path.is_file() {
        return Ok(None);
    }
    let contents = fs::read(path)?;
    Ok(Some(bincode::deserialize(&contents)?))
}

#[derive(Serialize, Deserialize)]
pub enum AgeGroup {
    Infant,
    Adult,
    Elder,
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

/// Command that the user can send to a running node to control its execution.
#[derive(Clone)]
pub struct NodeInfo {
    pub id: NodePublicId,
    pub keys: NodeKeys,
    pub root_dir: PathBuf,
    pub init_mode: Init,
    /// Upper limit in bytes for allowed network storage on this node.
    /// An Adult would be using the space for chunks, 
    /// while an Elder uses it for metadata.
    pub max_storage_capacity: u64,
}

impl NodeInfo {
    pub fn path(&self) -> &Path {
        self.root_dir.as_path()
    }

    pub fn keys(&self) -> NodeKeys {
        self.keys.clone()
    }
}

// pub struct AdultInfo {
//     pub node_info: NodeInfo,
//     /// Upper limit in bytes for allowed network storage on this node.
//     pub max_storage_capacity: u64,
// }

// pub struct ElderInfo<R: CryptoRng + Rng> {
//     pub node_info: NodeInfo,
//     pub rng: R,
// }