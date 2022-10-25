// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::io;
use thiserror::Error;

/// The type returned by the `sn_routing` message handling methods.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Internal error.
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum Error {
    /// Node error
    #[error("Node error: {0}")]
    Node(#[from] crate::node::Error),
    #[error("Comm error: {0}")]
    Comm(#[from] crate::comm::Error),
    /// Data error.
    #[error("Data error:: {0}")]
    Data(#[from] crate::data::Error),
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),
    /// Messaging error.
    #[error("Messaging error: {0}")]
    Messaging(#[from] sn_interface::messaging::Error),
    /// Network knowledge error.
    #[error("Network knowledge error: {0}")]
    NetworkKnowledge(#[from] sn_interface::network_knowledge::Error),
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    // /// JSON serialisation error.
    // #[error("JSON serialisation error:: {0}")]
    // JsonSerialisation(#[from] serde_json::Error),
}
