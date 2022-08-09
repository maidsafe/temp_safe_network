//! A builder to instantiate a [`Client`]
//!
//! # Example
//!
//! ```no_run
//! # #[tokio::main]
//! # async fn main() -> Result<(), sn_client::Error> {
//! use sn_client::api::Client;
//! use xor_name::XorName;
//!
//! let client = Client::builder().build().await?;
//! let _bytes = client.read_bytes(XorName::from_content("example".as_bytes())).await?;
//!
//! # Ok(())
//! # }
//! ```
use crate::{connections::Session, Client, DEFAULT_PREFIX_HARDLINK_NAME};

use qp2p::Config as Qp2pConfig;
use sn_dbc::Owner;
use sn_interface::{
    network_knowledge::{prefix_map::NetworkPrefixMap, utils::read_prefix_map_from_disk},
    types::Keypair,
};
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;

/// Environment variable used to convert into [`ClientBuilder::query_timeout`] (seconds)
pub const ENV_QUERY_TIMEOUT: &str = "SN_QUERY_TIMEOUT";
/// Environment variable used to convert into [`ClientBuilder::cmd_timeout`] (seconds)
pub const ENV_CMD_TIMEOUT: &str = "SN_CMD_TIMEOUT";
/// Environment variable used to convert into [`ClientBuilder::cmd_ack_wait`] (seconds)
pub const ENV_AE_WAIT: &str = "SN_AE_WAIT";

/// Bind by default to all network interfaces on a OS assigned port
pub const DEFAULT_LOCAL_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::UNSPECIFIED, 0);
/// Default timeout to use before timing out queries and commands
pub const DEFAULT_QUERY_CMD_TIMEOUT: Duration = Duration::from_secs(120);
/// Default timeout for waiting for potential Anti-Entropy messages
pub const DEFAULT_ACK_WAIT: Duration = Duration::from_secs(10);

/// Build a [`crate::Client`]
#[derive(Debug, Default)]
pub struct ClientBuilder {
    keypair: Option<Keypair>,
    dbc_owner: Option<Owner>,
    local_addr: Option<SocketAddr>,
    qp2p: Option<Qp2pConfig>,
    query_timeout: Option<Duration>,
    cmd_timeout: Option<Duration>,
    cmd_ack_wait: Option<Duration>,
    prefix_map: Option<NetworkPrefixMap>,
}

impl ClientBuilder {
    /// Instantiate a builder with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the keypair associated with the queries sent from this client.
    pub fn keypair(mut self, kp: impl Into<Option<Keypair>>) -> Self {
        self.keypair = kp.into();
        self
    }

    /// Set the DBC owner associated with this client.
    pub fn dbc_owner(mut self, owner: impl Into<Option<Owner>>) -> Self {
        self.dbc_owner = owner.into();
        self
    }

    /// Local address to bind client endpoint to
    pub fn local_addr(mut self, addr: impl Into<Option<SocketAddr>>) -> Self {
        self.local_addr = addr.into();
        self
    }

    /// QuicP2p options
    pub fn qp2p(mut self, cfg: impl Into<Option<Qp2pConfig>>) -> Self {
        self.qp2p = cfg.into();
        self
    }

    /// Time to wait for responses to queries before giving up and returning an error
    pub fn query_timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.query_timeout = timeout.into();
        self
    }

    /// Time to wait for cmds to not error before giving up and returning an error
    pub fn cmd_timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.cmd_timeout = timeout.into();
        self
    }

    /// Time to wait after a cmd is sent for AE flows to complete
    pub fn cmd_ack_wait(mut self, time: impl Into<Option<Duration>>) -> Self {
        self.cmd_ack_wait = time.into();
        self
    }

    /// NetworkPrefixMap used to bootstrap the client on the network
    pub fn prefix_map(mut self, pm: impl Into<Option<NetworkPrefixMap>>) -> Self {
        self.prefix_map = pm.into();
        self
    }

    /// Read options from environment variables:
    /// - [`Self::query_timeout()`] from [`ENV_QUERY_TIMEOUT`]
    /// - [`Self::cmd_timeout()`] from [`ENV_CMD_TIMEOUT`]
    /// - [`Self::cmd_ack_wait()`] from [`ENV_AE_WAIT`]
    pub fn from_env(mut self) -> Self {
        if let Ok(Some(v)) = env_parse(ENV_QUERY_TIMEOUT) {
            self.query_timeout = Some(Duration::from_secs(v));
        }
        if let Ok(Some(v)) = env_parse(ENV_CMD_TIMEOUT) {
            self.cmd_timeout = Some(Duration::from_secs(v));
        }
        if let Ok(Some(v)) = env_parse(ENV_AE_WAIT) {
            self.cmd_ack_wait = Some(Duration::from_secs(v));
        }

        self
    }

    /// Instantiate the [`Client`] using the parameters passed to this builder.
    ///
    /// In case parameters have not been passed to this builder, defaults will be used:
    /// - `[Self::keypair]` and `[Self::dbc_owner]` are randomly generated
    /// - `[Self::query_timeout`] and `[Self::cmd_timeout]` default to [`DEFAULT_QUERY_CMD_TIMEOUT`]
    /// - `[Self::cmd_ack_wait`] defaults to [`DEFAULT_ACK_WAIT`]
    /// - [`qp2p::Config`] will default to it's [`Default`] impl
    /// - Prefix map will be read from a standard location
    pub async fn build(self) -> Result<Client, crate::errors::Error> {
        let query_timeout = self.query_timeout.unwrap_or(DEFAULT_QUERY_CMD_TIMEOUT);
        let cmd_timeout = self.cmd_timeout.unwrap_or(DEFAULT_QUERY_CMD_TIMEOUT);
        let cmd_ack_wait = self.cmd_ack_wait.unwrap_or(DEFAULT_ACK_WAIT);

        let prefix_map_dir = default_prefix_map_path()?;
        let prefix_map = match self.prefix_map {
            Some(pm) => pm,
            None => read_prefix_map_from_disk(&prefix_map_dir).await?,
        };

        let session = Session::new(
            self.qp2p.unwrap_or_default(),
            self.local_addr
                .unwrap_or_else(|| SocketAddr::from(DEFAULT_LOCAL_ADDR)),
            cmd_ack_wait,
            prefix_map,
            prefix_map_dir,
        )?;

        let keypair = self.keypair.unwrap_or_else(Keypair::new_ed25519);
        let dbc_owner = self
            .dbc_owner
            .unwrap_or_else(|| Owner::from_random_secret_key(&mut rand::thread_rng()));

        let client = Client {
            keypair,
            dbc_owner,
            session,
            query_timeout,
            cmd_timeout,
            chunks_cache: Arc::new(RwLock::new(Default::default())),
        };
        client.connect().await?;

        Ok(client)
    }
}

/// Parse environment variable. Returns `Ok(None)` if environment variable isn't set.
fn env_parse<F: FromStr>(s: &str) -> Result<Option<F>, F::Err> {
    let v = match std::env::var(s) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    F::from_str(&v).map(|v| Some(v))
}

fn default_prefix_map_path() -> Result<PathBuf, crate::Error> {
    // Use `$User/.safe/prefix_maps` directory
    let prefix_maps_dir = dirs_next::home_dir()
        .ok_or_else(|| {
            crate::Error::NetworkContacts("Could not read user's home directory".to_string())
        })?
        .join(".safe")
        .join("prefix_maps");
    let path = prefix_maps_dir.join(DEFAULT_PREFIX_HARDLINK_NAME);

    Ok(path)
}

#[cfg(test)]
mod tests {}
