// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! The Safe Network Client.
//!
//! In order to connect to The Safe Network you'll need to send messages back and forth to network nodes.
//! The [Client] has everything needed to perform this communication, with APIs to enable
//! working with data.
//!
//! With these APIs you can easily:
//! - Connect to The Safe Network
//! - Read Public data from the network
//! - Write data to the network (assuming you have a SafeCoin balance)
//!
//! ## Basic Usage
//!
//! Setting up a random client for read only access:
//!
//! ```no_run
//! # // The Safe Client is an sync library so will need some kind of runtime. Here we use tokio.
//! # extern crate tokio; use anyhow::Result;
//! # use safe_network::client::utils::test_utils::read_network_conn_info;
//! use safe_network::client::{Client, DEFAULT_QUERY_TIMEOUT};
//! # #[tokio::main] async fn main() { let _: Result<()> = futures::executor::block_on( async {
//! # let bootstrap_contacts = Some(read_network_conn_info()?);
//! let client = Client::new(None, None, bootstrap_contacts, DEFAULT_QUERY_TIMEOUT).await?;
//! // Now for example you can perform read operations:
//! let _some_balance = client.get_balance().await?;
//! # Ok(()) } ); }
//! ```
//!
//! Or use a pre-existing SecretKey which has a SafeCoin balance to be able to write to the network:
//!
//! ```no_run
//! # // The Safe Client is an sync library so will need some kind of runtime. Here we use tokio.
//! # extern crate tokio; use anyhow::Result;
//! # use safe_network::client::{utils::test_utils::read_network_conn_info};
//! use safe_network::client::{Client, DEFAULT_QUERY_TIMEOUT};
//! use rand::rngs::OsRng;
//! use safe_network::types::Keypair;
//! # #[tokio::main] async fn main() { let _: Result<()> = futures::executor::block_on( async {
//! let id = Keypair::new_ed25519(&mut OsRng);
//! # let bootstrap_contacts = Some(read_network_conn_info()?);
//! let client = Client::new(Some(id), None, bootstrap_contacts, DEFAULT_QUERY_TIMEOUT).await?;
//! // Now for example you can perform read operations:
//! let _some_balance = client.get_balance().await?;
//! # Ok(()) } ); }
//! ```

mod connections;
mod errors;

// Export public API.

pub use client_api::Client;
pub use errors::{Error, ErrorMessage};
pub use qp2p::Config as QuicP2pConfig;

/// Client trait and related constants.
pub mod client_api;
/// Config file handling.
pub mod config_handler;

/// Default timeout in
pub const DEFAULT_QUERY_TIMEOUT: u64 = 20; // 20 seconds

/// Utility functions.
pub mod utils;
