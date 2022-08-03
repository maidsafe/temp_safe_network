///! A builder to instantiate a [`Client`]
use crate::{Client, ClientConfig};

use qp2p::Config as Qp2pConfig;
use sn_dbc::Owner;
use sn_interface::types::Keypair;
use std::{net::SocketAddr, str::FromStr, time::Duration};

/// Environment variable used to convert into [`ClientConfig::query_timeout`]
pub const ENV_QUERY_TIMEOUT: &str = "SN_QUERY_TIMEOUT";
/// Environment variable used to convert into [`ClientConfig::cmd_timeout`]
pub const ENV_CMD_TIMEOUT: &str = "SN_CMD_TIMEOUT";
/// Environment variable used to convert into [`ClientConfig::cmd_ack_wait`]
pub const ENV_AE_WAIT: &str = "SN_AE_WAIT";

/// Build a [`crate::Client`]
#[derive(Debug, Default)]
pub struct ClientBuilder {
    keypair: Option<Keypair>,
    dbc_owner: Option<Owner>,

    // [`ClientConfig`] fields
    local_addr: Option<SocketAddr>,
    qp2p: Option<Qp2pConfig>,
    query_timeout: Option<Duration>,
    cmd_timeout: Option<Duration>,
    cmd_ack_wait: Option<Duration>,
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

    /// See [`ClientConfig.local_addr`]
    pub fn local_addr(mut self, addr: impl Into<Option<SocketAddr>>) -> Self {
        self.local_addr = addr.into();
        self
    }

    /// See [`ClientConfig.qp2p`]
    pub fn qp2p(mut self, cfg: impl Into<Option<Qp2pConfig>>) -> Self {
        self.qp2p = cfg.into();
        self
    }

    /// See [`ClientConfig.query_timeout`]
    pub fn query_timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.query_timeout = timeout.into();
        self
    }

    /// See [`ClientConfig.cmd_timeout`]
    pub fn cmd_timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.cmd_timeout = timeout.into();
        self
    }

    /// See [`ClientConfig.cmd_ack_wait`]
    pub fn cmd_ack_wait(mut self, time: impl Into<Option<Duration>>) -> Self {
        self.cmd_ack_wait = time.into();
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
    pub async fn build(self) -> Result<Client, crate::errors::Error> {
        let cfg = ClientConfig::new(
            self.local_addr,
            self.qp2p,
            self.query_timeout,
            self.cmd_timeout,
            self.cmd_ack_wait,
        )
        .await;

        let keypair = self.keypair.unwrap_or_else(Keypair::new_ed25519);
        let owner = self
            .dbc_owner
            .unwrap_or_else(|| Owner::from_random_secret_key(&mut rand::thread_rng()));

        Client::new(cfg, keypair, owner).await
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

#[cfg(test)]
mod tests {}
