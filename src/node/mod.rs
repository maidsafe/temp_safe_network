// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Implementation of the "Node" node for the SAFE Network.

mod capacity;
mod chaos;
mod chunks;
/// Configuration handling
pub mod config_handler;
mod data_store;
mod error;
mod event_mapping;
mod event_store;
mod metadata;
mod network;
mod node_api;
mod node_ops;
mod to_db_key;

/// Docs
pub mod state_db;

pub mod utils;

pub use crate::node::{
    config_handler::{add_connection_info, set_connection_info, Config},
    error::{Error, Result},
    node_api::Node,
};
