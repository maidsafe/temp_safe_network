// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod blob_apis;
mod commands;
mod data;
mod queries;
mod register_apis;

use crate::client::{connections::Session, errors::Error, Config};
use crate::messaging::data::{CmdError, DataQuery, ServiceMsg};
use crate::types::{ChunkAddress, Keypair, PublicKey};

use crate::messaging::{ServiceAuth, WireMsg};
use crate::prefix_map::NetworkPrefixMap;
use itertools::Itertools;
use rand::rngs::OsRng;
use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::{
    sync::{mpsc::Receiver, RwLock},
    time::Duration,
};
use tracing::{debug, info};
use xor_name::XorName;

/// Client object
#[derive(Clone, Debug)]
pub struct Client {
    keypair: Keypair,
    incoming_errors: Arc<RwLock<Receiver<CmdError>>>,
    session: Session,
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
    /// TODO: update once data types are crdt compliant
    ///
    ///
    #[instrument(skip_all, level = "debug", name = "New client")]
    pub async fn new(
        config: Config,
        bootstrap_nodes: BTreeSet<SocketAddr>,
        optional_keypair: Option<Keypair>,
    ) -> Result<Self, Error> {
        Client::create_with(config, bootstrap_nodes, optional_keypair, true).await
    }

    pub(crate) async fn create_with(
        config: Config,
        bootstrap_nodes: BTreeSet<SocketAddr>,
        optional_keypair: Option<Keypair>,
        read_prefixmap: bool,
    ) -> Result<Self, Error> {
        let mut rng = OsRng;

        let keypair = match optional_keypair {
            Some(id) => {
                info!("Client started for specific pk: {:?}", id.public_key());
                id
            }
            None => {
                let keypair = Keypair::new_ed25519(&mut rng);
                info!(
                    "Client started for new randomly created pk: {:?}",
                    keypair.public_key()
                );
                keypair
            }
        };

        let mut prefix_map_dir = dirs_next::home_dir()
            .ok_or_else(|| Error::Generic("Error opening home dir".to_string()))?;
        prefix_map_dir.push(".safe");
        prefix_map_dir.push("prefix_map");

        // Read NetworkPrefixMap from disk if present else create a new one
        let prefix_map = if let (Ok(mut prefix_map_file), true) =
            (File::open(prefix_map_dir).await, read_prefixmap)
        {
            let mut prefix_map_contents = vec![];
            let _ = prefix_map_file
                .read_to_end(&mut prefix_map_contents)
                .await?;

            let prefix_map: NetworkPrefixMap = rmp_serde::from_slice(&prefix_map_contents)
                .map_err(|err| {
                    Error::Generic(format!(
                        "Error deserializing PrefixMap from disk: {:?}",
                        err
                    ))
                })?;
            prefix_map
        } else {
            NetworkPrefixMap::new(config.genesis_key)
        };

        if config.genesis_key != prefix_map.genesis_key() {
            return Err(Error::Generic(
                "Genesis Key from the config and the PrefixMap mismatch".to_string(),
            ));
        }

        // Incoming error notifiers
        let (err_sender, err_receiver) = tokio::sync::mpsc::channel::<CmdError>(10);

        let client_pk = keypair.public_key();

        // Bootstrap to the network, connecting to a section based
        // on a public key of our choice.
        debug!(
            "Creating new session with genesis key: {} ...",
            hex::encode(config.genesis_key.to_bytes())
        );

        // Create a session with the network
        let session = Session::new(
            client_pk,
            config.genesis_key,
            config.qp2p,
            err_sender,
            config.local_addr,
            config.standard_wait,
            prefix_map,
        )
        .await?;

        let client = Self {
            keypair,
            session,
            incoming_errors: Arc::new(RwLock::new(err_receiver)),
            query_timeout: config.query_timeout,
        };

        // TODO: The message being sent below is a temporary solution to fetch network info for
        // the client. Ideally the client should be able to send proper AE-Probe messages to the
        // trigger the AE flows.

        // Generate a random query to send a dummy message
        let random_dst_addr = XorName::random();
        let serialised_cmd = {
            let msg = ServiceMsg::Query(DataQuery::GetChunk(ChunkAddress(random_dst_addr)));
            WireMsg::serialize_msg_payload(&msg)?
        };
        let signature = client.keypair.sign(&serialised_cmd);
        let auth = ServiceAuth {
            public_key: client_pk,
            signature,
        };

        let bootstrap_nodes = bootstrap_nodes.iter().copied().collect_vec();

        // TODO: check for the initial msg id and DO NOT RESEND in ae retry.
        // Send the dummy message to probe the network for it's infrastructure details.
        client
            .session
            .make_contact_with_nodes(
                bootstrap_nodes.clone(),
                random_dst_addr,
                auth,
                serialised_cmd,
            )
            .await?;

        Ok(client)
    }

    /// Return the client's keypair.
    ///
    /// Useful for retrieving the PublicKey or KeyPair in the event you need to _sign_ something
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub fn keypair(&self) -> Keypair {
        self.keypair.clone()
    }

    /// Return the client's PublicKey.
    ///
    /// # Examples
    ///
    /// TODO: update once data types are crdt compliant
    ///
    pub fn public_key(&self) -> PublicKey {
        self.keypair().public_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::utils::test_utils::{
        create_test_client, create_test_client_with, init_test_logger,
    };
    use crate::types::utils::random_bytes;
    use crate::url::Scope;
    use eyre::Result;
    use std::{
        collections::HashSet,
        net::{IpAddr, Ipv4Addr, SocketAddr},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn client_creation() -> Result<()> {
        init_test_logger();
        let _client = create_test_client().await?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn client_nonsense_bootstrap_fails() -> Result<()> {
        init_test_logger();

        let mut nonsense_bootstrap = HashSet::new();
        let _ = nonsense_bootstrap.insert(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            3033,
        ));
        //let setup = create_test_client_with(None, Some(nonsense_bootstrap)).await;
        //assert!(setup.is_err());
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn client_creation_with_existing_keypair() -> Result<()> {
        init_test_logger();

        let mut rng = OsRng;
        let full_id = Keypair::new_ed25519(&mut rng);
        let pk = full_id.public_key();

        let client = create_test_client_with(Some(full_id), None, true).await?;
        assert_eq!(pk, client.public_key());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn long_lived_connection_survives() -> Result<()> {
        init_test_logger();

        let client = create_test_client().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(40)).await;
        let bytes = random_bytes(self_encryption::MIN_ENCRYPTABLE_BYTES / 2);
        let _ = client.upload(bytes, Scope::Public).await?;
        Ok(())
    }

    // Send is an important trait that assures futures can be run in a
    // multithreaded context. If a future depends on a non-Send future, directly
    // or indirectly, the future itself becomes non-Send and so on. Thus, it can
    // happen that high-level API functions will become non-Send by accident.
    #[test]
    fn client_is_send() {
        init_test_logger();

        fn require_send<T: Send>(_t: T) {}
        require_send(create_test_client());
    }
}
