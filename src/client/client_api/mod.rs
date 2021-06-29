// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod blob_apis;
mod blob_storage;
mod commands;
mod map_apis;
mod queries;
mod register_apis;
mod sequence_apis;

use crate::client::{config_handler::Config, connections::Session, errors::Error};
use crate::messaging::client::{Cmd, CmdError, DataCmd};
use crate::types::{Chunk, ChunkAddress, Keypair, PublicKey};
use lru::LruCache;
use rand::rngs::OsRng;
use std::{
    path::Path,
    {collections::HashSet, net::SocketAddr, sync::Arc},
};
use tokio::sync::mpsc::Receiver;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Duration;
use tracing::{debug, info, trace};

// Number of attempts to make when trying to bootstrap to the network
const NUM_OF_BOOTSTRAPPING_ATTEMPTS: u8 = 1;
const BLOB_CACHE_CAP: usize = 150;

/// Client object
#[derive(Clone, Debug)]
pub struct Client {
    keypair: Keypair,
    incoming_errors: Arc<RwLock<Receiver<CmdError>>>,
    session: Session,
    blob_cache: Arc<Mutex<LruCache<ChunkAddress, Chunk>>>,
    pub(crate) query_timeout: Duration,
}

/// Easily manage connections to/from The Safe Network with the client and its APIs.
/// Use a random client for read-only or one-time operations.
/// Supply an existing, SecretKey which holds a SafeCoin balance to be able to perform
/// write operations.
impl Client {
    /// Create a Safe Network client instance. Either for an existing SecretKey (in which case) the client will attempt
    /// to retrieve the history of the key's balance in order to be ready for any token operations. Or if no SecreteKey
    /// is passed, a random keypair will be used, which provides a client that can only perform Read operations (at
    /// least until the client's SecretKey receives some token).
    ///
    /// # Examples
    ///
    /// Create a random client
    /// ```no_run
    /// # extern crate tokio; use anyhow::Result;
    /// # use safe_network::client::utils::test_utils::read_network_conn_info;
    /// use safe_network::client::{Client, DEFAULT_QUERY_TIMEOUT};
    ///
    /// # #[tokio::main] async fn main() {
    /// let _: Result<()> = futures::executor::block_on( async {
    ///
    /// # let bootstrap_contacts = Some(read_network_conn_info()?);
    /// let client = Client::new(None, None, bootstrap_contacts, DEFAULT_QUERY_TIMEOUT).await?;
    /// // Now for example you can perform read operations:
    /// let _some_balance = client.get_balance().await?;
    /// # Ok(()) } ); }
    /// ```
    pub async fn new(
        optional_keypair: Option<Keypair>,
        config_file_path: Option<&Path>,
        bootstrap_config: Option<HashSet<SocketAddr>>,
        query_timeout: u64,
    ) -> Result<Self, Error> {
        let mut rng = OsRng;

        let (keypair, _is_random_client) = match optional_keypair {
            Some(id) => {
                info!("Client started for specific pk: {:?}", id.public_key());
                (id, false)
            }
            None => {
                let keypair = Keypair::new_ed25519(&mut rng);
                info!(
                    "Client started for new randomly created pk: {:?}",
                    keypair.public_key()
                );
                (keypair, true)
            }
        };

        let mut qp2p_config = Config::new(config_file_path, bootstrap_config).qp2p;
        // We use feature `no-igd` so this will use the echo service only
        qp2p_config.forward_port = true;

        // Incoming error notifiers
        let (err_sender, err_receiver) = tokio::sync::mpsc::channel::<CmdError>(10);

        // Create the session with the network
        let mut session = Session::new(qp2p_config, err_sender)?;

        let client_pk = keypair.public_key();

        // Bootstrap to the network, connecting to the section responsible
        // for our client public key
        debug!("Bootstrapping to the network...");
        attempt_bootstrap(&mut session, client_pk).await?;

        let client = Self {
            keypair,
            session,
            incoming_errors: Arc::new(RwLock::new(err_receiver)),
            query_timeout: Duration::from_secs(query_timeout),
            blob_cache: Arc::new(Mutex::new(LruCache::new(BLOB_CACHE_CAP))),
        };

        Ok(client)
    }

    /// Return the client's FullId.
    ///
    /// Useful for retrieving the PublicKey or KeyPair in the event you need to _sign_ something
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate tokio; use anyhow::Result;
    /// # use safe_network::client::utils::test_utils::read_network_conn_info;
    /// use safe_network::client::{Client, DEFAULT_QUERY_TIMEOUT};
    /// # #[tokio::main] async fn main() { let _: Result<()> = futures::executor::block_on( async {
    /// # let bootstrap_contacts = Some(read_network_conn_info()?);
    /// let client = Client::new(None, None, bootstrap_contacts, DEFAULT_QUERY_TIMEOUT).await?;
    /// let _keypair = client.keypair();
    ///
    /// # Ok(()) } ); }
    /// ```
    pub fn keypair(&self) -> Keypair {
        self.keypair.clone()
    }

    /// Return the client's PublicKey.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate tokio; use anyhow::Result;
    /// # use safe_network::client::utils::test_utils::read_network_conn_info;
    /// use safe_network::client::{Client, DEFAULT_QUERY_TIMEOUT};
    /// # #[tokio::main] async fn main() { let _: Result<()> = futures::executor::block_on( async {
    /// # let bootstrap_contacts = Some(read_network_conn_info()?);
    /// let client = Client::new(None, None, bootstrap_contacts, DEFAULT_QUERY_TIMEOUT).await?;
    /// let _pk = client.public_key();
    /// # Ok(()) } ); }
    /// ```
    pub fn public_key(&self) -> PublicKey {
        self.keypair().public_key()
    }

    // Private helper to obtain payment proof for a data command, send it to the network,
    // and also apply the payment to local replica actor.
    async fn pay_and_send_data_command(&self, cmd: DataCmd) -> Result<(), Error> {
        // The _actual_ message
        let cmd = Cmd::Data { cmd };

        self.send_cmd(cmd, None).await
    }

    #[cfg(test)]
    pub async fn expect_cmd_error(&mut self) -> Option<CmdError> {
        self.incoming_errors.write().await.recv().await
    }
}

/// Utility function that bootstraps a client to the network. If there is a failure then it retries.
/// After a maximum of three attempts if the boostrap process still fails, then an error is returned.
async fn attempt_bootstrap(session: &mut Session, client_pk: PublicKey) -> Result<(), Error> {
    let mut attempts: u8 = 0;
    loop {
        match session.bootstrap(client_pk).await {
            Ok(()) => return Ok(()),
            Err(err) => {
                attempts += 1;
                if attempts < NUM_OF_BOOTSTRAPPING_ATTEMPTS {
                    trace!(
                        "Error connecting to network! {:?}\nRetrying... ({})",
                        err,
                        attempts
                    );
                } else {
                    return Err(err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::utils::test_utils::{create_test_client, create_test_client_with};
    use anyhow::Result;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    pub async fn client_creation() -> Result<()> {
        let _client = create_test_client(None).await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    pub async fn client_nonsense_bootstrap_fails() -> Result<()> {
        let mut nonsense_bootstrap = HashSet::new();
        let _ = nonsense_bootstrap.insert(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            3033,
        ));
        //let setup = create_test_client_with(None, Some(nonsense_bootstrap)).await;
        //assert!(setup.is_err());
        Ok(())
    }

    #[tokio::test]
    pub async fn client_creation_with_existing_keypair() -> Result<()> {
        let mut rng = OsRng;
        let full_id = Keypair::new_ed25519(&mut rng);
        let pk = full_id.public_key();

        let client = create_test_client_with(Some(full_id), None).await?;
        assert_eq!(pk, client.public_key());

        Ok(())
    }

    #[tokio::test]
    pub async fn long_lived_connection_survives() -> Result<()> {
        let _client = create_test_client(None).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(40)).await;

        // TODO: make a network call

        Ok(())
    }
}
