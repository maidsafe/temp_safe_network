// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// A [`Client`] builder
pub mod client_builder;
mod cmds;
mod data;
mod file_apis;
mod queries;
mod register_apis;
mod spend_queries;
mod spentbook_apis;
mod transfers;

pub use client_builder::ClientBuilder;
pub use file_apis::QueriedDataReplicas;
pub use register_apis::RegisterWriteAheadLog;
pub use transfers::{
    select_inputs as select_dbc_inputs, send_tokens, Error as TransferError, ReissueInputs,
    ReissueOutputs,
};

use crate::{
    errors::{Error, Result},
    sessions::Session,
};

use sn_dbc::Owner;
use sn_interface::{
    messaging::data::{DataQuery, RegisterQuery},
    network_knowledge::SectionTree,
    types::{Chunk, Keypair, PublicKey, RegisterAddress},
};

use std::sync::Arc;
use tokio::{sync::RwLock, time::Duration};
use tracing::debug;
use uluru::LRUCache;

/// Name of the default network contacts file the Client uses. The file is
/// expected to be found at user's OS home directory, e.g. in Linux this
/// path would become: $HOME/.safe/network_contacts/default
pub const DEFAULT_NETWORK_CONTACTS_FILE_NAME: &str = "default";

// Maximum amount of Chunks to keep in our local Chunks cache.
// Each Chunk is at most self_encryption::MAX_CHUNK_SIZE.
const CHUNK_CACHE_SIZE: usize = 50;

// LRU cache to keep the Chunks we retrieve.
type ChunksCache = LRUCache<Chunk, CHUNK_CACHE_SIZE>;

/// Client object
#[derive(Clone, Debug)]
pub struct Client {
    keypair: Keypair,
    dbc_owner: Owner,
    session: Session,
    pub(crate) query_timeout: Option<Duration>,
    pub(crate) max_backoff_interval: Duration,
    pub(crate) cmd_timeout: Option<Duration>,
    chunks_cache: Arc<RwLock<ChunksCache>>,
}

/// Easily manage connections to/from The Safe Network with the client and its APIs.
/// Use a random client for read-only or one-time operations.
/// Supply an existing, `SecretKey` which holds a `SafeCoin` balance to be able to perform
/// write operations.
impl Client {
    /// Bootstrap this client to the network.
    ///
    /// In case of an existing SecretKey the client will attempt to retrieve the history
    /// of the key's balance in order to be ready for any token operations.
    #[instrument(skip_all, level = "debug")]
    pub async fn connect(&self) -> Result<()> {
        // TODO: The message being sent below is a temporary solution to fetch network info for
        // the client. Ideally the client should be able to send proper AE-Probe messages to
        // trigger the AE flows.

        // Generate a random query to send a dummy message
        let query = DataQuery::Register(RegisterQuery::Get(RegisterAddress {
            name: xor_name::rand::random(),
            tag: 1,
        }));
        debug!("Making initial contact with network. Probe msg: {query:?}");

        // Send the dummy message to probe the network for it's infrastructure details.
        match self.send_query_without_retry(query.clone()).await {
            Ok(response) if response.is_data_not_found() => {
                // A data-not-found response means it comes from the right set of Elders,
                // any AE retries should have taken place as well.
                let network_knowledge = self.session.network.read().await;
                let sections_count = network_knowledge.known_sections_count();
                let known_sap = network_knowledge.closest(&query.dst_name(), None);
                debug!(
                    "Client has some network knowledge. Current sections \
                    known: {sections_count}. SAP for our startup-query: {known_sap:?}"
                );
                Ok(())
            }
            result => {
                // we've failed
                Err(Error::NetworkContacts(format!(
                    "failed to make initial contact with network, resulted in: {result:?}"
                )))
            }
        }
    }

    /// Return the client's keypair.
    ///
    /// Useful for retrieving the `PublicKey` or `KeyPair` in the event you need to _sign_ something
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// Return the client's `PublicKey`.
    pub fn public_key(&self) -> PublicKey {
        self.keypair().public_key()
    }

    /// Return the client's DBC owner, which will be a secret key.
    ///
    /// This can then be used to sign output DBCs during a DBC reissue.
    pub fn dbc_owner(&self) -> &Owner {
        &self.dbc_owner
    }

    /// Check if the provided public key is a known section key
    /// based on our current knowledge of the network and sections chains.
    pub async fn is_known_section_key(&self, section_key: &sn_dbc::PublicKey) -> bool {
        self.session
            .network
            .read()
            .await
            .get_sections_dag()
            .has_key(section_key)
    }

    /// SectionTree used to bootstrap the client on the network.
    ///
    /// This is updated by the client as it receives Anti-Entropy/update messages from the network.
    /// Any user of this API is responsible for caching it so it can use it for any new `Client`
    /// instance not needing to obtain all this information from the network all over again.
    pub async fn section_tree(&self) -> SectionTree {
        self.session.network.read().await.clone()
    }

    /// Create a builder to instantiate a [`Client`]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::{
        create_test_client, create_test_client_with, get_dbc_owner_from_secret_key_hex,
    };
    use sn_interface::init_logger;

    use eyre::Result;
    use std::{
        collections::HashSet,
        net::{IpAddr, Ipv4Addr, SocketAddr},
    };

    #[tokio::test(flavor = "multi_thread")]
    async fn client_creation() -> Result<()> {
        init_logger();
        let _client = create_test_client().await?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn client_nonsense_bootstrap_fails() -> Result<()> {
        init_logger();

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
        init_logger();

        let full_id = Keypair::new_ed25519();
        let pk = full_id.public_key();

        let client = create_test_client_with(Some(full_id), None, None).await?;
        assert_eq!(pk, client.public_key());

        Ok(())
    }

    // Send is an important trait that assures futures can be run in a
    // multithreaded context. If a future depends on a non-Send future, directly
    // or indirectly, the future itself becomes non-Send and so on. Thus, it can
    // happen that high-level API functions will become non-Send by accident.
    #[test]
    fn client_is_send() {
        init_logger();

        fn require_send<T: Send>(_t: T) {}
        require_send(create_test_client());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn client_create_with_dbc_owner() -> Result<()> {
        init_logger();
        let dbc_owner = get_dbc_owner_from_secret_key_hex(
            "81ebce8339cb2a6e5cbf8b748215ba928acff7f92557b3acfb09a5b25e920d20",
        )?;

        let client = create_test_client_with(None, Some(dbc_owner.clone()), None).await?;
        assert_eq!(&dbc_owner, client.dbc_owner());
        Ok(())
    }
}
