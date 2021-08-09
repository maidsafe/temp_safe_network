// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::fetch::Range;
use crate::{ipc::BootstrapConfig, Error, Result};
use hex::encode;
use log::{debug, info};
use safe_network::client::{Client, Config, Error as ClientError};
use safe_network::types::{
    register::{Address, Entry, EntryHash, PrivatePermissions, PublicPermissions, User},
    ChunkAddress, Error as SafeNdError, Keypair,
};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};
use xor_name::XorName;

const APP_NOT_CONNECTED: &str = "Application is not connected to the network";

#[derive(Default, Clone)]
pub struct SafeAppClient {
    safe_client: Option<Client>,
    pub(crate) bootstrap_config: Option<HashSet<SocketAddr>>,
    config_path: Option<PathBuf>,
    timeout: Duration,
}

impl SafeAppClient {
    // Private helper to obtain the Safe Client instance
    fn get_safe_client(&self) -> Result<Client> {
        match &self.safe_client {
            Some(client) => Ok(client.clone()),
            None => Err(Error::ConnectionError(APP_NOT_CONNECTED.to_string())),
        }
    }

    pub fn new(timeout: Duration) -> Self {
        Self {
            safe_client: None,
            bootstrap_config: None,
            config_path: None,
            timeout,
        }
    }

    // Connect to the SAFE Network using the keypair if provided. Contacts list
    // are overriden if a 'bootstrap_config' is provided.
    pub async fn connect(
        &mut self,
        app_keypair: Option<Keypair>,
        config_path: Option<&Path>,
        bootstrap_config: Option<BootstrapConfig>,
    ) -> Result<()> {
        debug!("Connecting to SAFE Network...");
        if bootstrap_config.is_some() {
            self.bootstrap_config = bootstrap_config;
        }

        self.config_path = config_path.map(|p| p.to_path_buf());

        debug!(
            "Client to be instantiated with specific pk?: {:?}",
            app_keypair
        );
        debug!(
            "Bootstrap contacts list set to: {:?}",
            self.bootstrap_config
        );
        let config = Config::new(
            self.config_path.as_deref(),
            self.bootstrap_config.clone(),
            Some(self.timeout),
        )
        .await;
        let client = Client::new(app_keypair, config).await.map_err(|err| {
            Error::ConnectionError(format!("Failed to connect to the SAFE Network: {:?}", err))
        })?;

        self.safe_client = Some(client);

        debug!("Successfully connected to the Network!!!");
        Ok(())
    }

    pub fn keypair(&self) -> Result<Keypair> {
        let client = self.get_safe_client()?;
        Ok(client.keypair())
    }

    // // === Blob operations ===
    pub async fn store_public_blob(&self, data: &[u8], dry_run: bool) -> Result<XorName> {
        let address = if dry_run {
            let (_, address) = Client::blob_data_map(data.to_vec(), None).await?;
            address
        } else {
            let client = self.get_safe_client()?;
            client
                .store_public_blob(data)
                .await
                .map_err(|e| Error::NetDataError(format!("Failed to PUT Public Blob: {:?}", e)))?
        };

        Ok(*address.name())
    }

    pub async fn get_public_blob(&self, xorname: XorName, range: Range) -> Result<Vec<u8>> {
        debug!("Fetching immutable data: {:?}", &xorname);

        let client = self.get_safe_client()?;
        let blob_address = ChunkAddress::Public(xorname);
        let data = if let Some((start, end)) = range {
            let len = end.map(|end_index| end_index - start.unwrap_or(0));
            client
                .read_blob(
                    blob_address,
                    start.map(|val| val as usize),
                    len.map(|val| val as usize),
                )
                .await
        } else {
            client.read_blob(blob_address, None, None).await
        }
        .map_err(|e| Error::NetDataError(format!("Failed to GET Public Blob: {:?}", e)))?;

        debug!(
            "Public Blob data successfully retrieved from: {:?}",
            &xorname
        );

        Ok(data)
    }

    // === Register data operations ===
    pub async fn store_register(
        &self,
        name: Option<XorName>,
        tag: u64,
        _permissions: Option<String>,
        private: bool,
    ) -> Result<XorName> {
        debug!(
            "Storing {} Register data with tag type: {}, xorname: {:?}",
            if private { "Private" } else { "Public" },
            tag,
            name
        );

        let client = self.get_safe_client()?;
        let xorname = name.unwrap_or_else(rand::random);
        info!("Xorname for new Register storage: {:?}", &xorname);

        // The Register's owner will be the client's public key
        let my_pk = client.public_key();

        // Store the Register on the network
        let _ = if private {
            // Set read and write  permissions to this application
            let mut perms = BTreeMap::default();
            let _ = perms.insert(my_pk, PrivatePermissions::new(true, true));

            client
                .store_private_register(xorname, tag, my_pk, perms)
                .await
                .map_err(|e| {
                    Error::NetDataError(format!("Failed to store Private Register data: {:?}", e))
                })?
        } else {
            // Set write permissions to this application
            let user_app = User::Key(my_pk);
            let mut perms = BTreeMap::default();
            let _ = perms.insert(user_app, PublicPermissions::new(true));

            client
                .store_public_register(xorname, tag, my_pk, perms)
                .await
                .map_err(|e| {
                    Error::NetDataError(format!("Failed to store Public Register data: {:?}", e))
                })?
        };

        Ok(xorname)
    }

    pub async fn read_register(&self, address: Address) -> Result<BTreeSet<(EntryHash, Entry)>> {
        debug!("Fetching Register data at {:?}", address);

        let client = self.get_safe_client()?;

        client.read_register(address).await.map_err(|err| {
            if let ClientError::NetworkDataError(SafeNdError::NoSuchEntry) = err {
                Error::EmptyContent(format!("Empty Register found at {:?}", address))
            } else {
                Error::NetDataError(format!(
                    "Failed to read current value from Register data: {:?}",
                    err
                ))
            }
        })
    }

    pub async fn get_register_entry(&self, address: Address, hash: EntryHash) -> Result<Entry> {
        debug!("Fetching Register hash {:?} at {:?}", hash, address);

        let client = self.get_safe_client()?;
        let entry = client
            .get_register_entry(address, hash)
            .await
            .map_err(|err| {
                if let ClientError::NetworkDataError(SafeNdError::NoSuchEntry) = err {
                    Error::HashNotFound(hash)
                } else {
                    Error::NetDataError(format!(
                        "Failed to retrieve entry with hash '{}' from Register data: {:?}",
                        encode(hash),
                        err
                    ))
                }
            })?;

        Ok(entry.to_vec())
    }

    pub async fn write_to_register(
        &self,
        address: Address,
        data: Vec<u8>,
        parents: BTreeSet<EntryHash>,
    ) -> Result<EntryHash> {
        debug!("Writing to Register at {:?}", address);
        let client = self.get_safe_client()?;

        client
            .write_to_register(address, data, parents)
            .await
            .map_err(|e| Error::NetDataError(format!("Failed to write to Register: {:?}", e)))
    }
}
