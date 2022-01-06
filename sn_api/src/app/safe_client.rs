// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::resolver::Range;
use crate::{ipc::NodeConfig, Error, Result};
use bytes::Bytes;
use hex::encode;
use log::{debug, info};
use safe_network::client::{Client, ClientConfig, Error as ClientError, RegisterWriteAheadLog};
use safe_network::types::{
    register::{Entry, EntryHash, PrivatePermissions, PublicPermissions, User},
    BytesAddress, Error as SafeNdError, Keypair, RegisterAddress, Scope,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::Duration,
};
use xor_name::XorName;

const APP_NOT_CONNECTED: &str = "Application is not connected to the network";

#[derive(Default, Clone)]
pub struct SafeAppClient {
    safe_client: Option<Client>,
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
        node_config: NodeConfig,
    ) -> Result<()> {
        debug!("Connecting to SAFE Network...");

        self.config_path = config_path.map(|p| p.to_path_buf());

        debug!(
            "Client to be instantiated with specific pk?: {:?}",
            app_keypair
        );
        debug!("Bootstrap contacts list set to: {:?}", node_config);

        let config = ClientConfig::new(
            None,
            None,
            node_config.0,
            self.config_path.as_deref(),
            Some(self.timeout),
            None,
        )
        .await;
        let client = Client::new(config, node_config.1, app_keypair)
            .await
            .map_err(|err| {
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

    //
    // Blob operations
    //
    pub async fn store_bytes(&self, bytes: Bytes, dry_run: bool) -> Result<XorName> {
        let address = if dry_run {
            debug!(
                "Calculating network address for {} bytes of data",
                bytes.len()
            );
            Client::calculate_address(bytes, Scope::Public)?
        } else {
            debug!("Storing {} bytes of data", bytes.len());
            let client = self.get_safe_client()?;
            client.upload_and_verify(bytes, Scope::Public).await?
        };

        Ok(*address.name())
    }

    pub async fn get_bytes(&self, address: BytesAddress, range: Range) -> Result<Bytes> {
        debug!("Attempting to fetch data from {:?}", address.name());
        let client = self.get_safe_client()?;
        let data = if let Some((start, end)) = range {
            let start = start.map(|start_index| start_index as usize).unwrap_or(0);
            let len = end
                .map(|end_index| end_index as usize - start)
                .unwrap_or(usize::MAX);

            client.read_from(address, start, len).await
        } else {
            client.read_bytes(address).await
        }
        .map_err(|e| Error::NetDataError(format!("Failed to GET Blob: {:?}", e)))?;

        debug!(
            "{} bytes of data successfully retrieved from: {:?}",
            data.len(),
            address.name()
        );

        Ok(data)
    }

    // === Register data operations ===
    /// Low level method to create a register
    /// Returns a register operation batch that can be used to apply changes on the network.
    /// Nothing is sent to the network, without applying the batch, it's pretty much a dry run.
    pub async fn create_register(
        &self,
        name: Option<XorName>,
        tag: u64,
        _permissions: Option<String>,
        private: bool,
        dry_run: bool,
    ) -> Result<(XorName, RegisterWriteAheadLog)> {
        debug!(
            "Storing {} Register data with tag type: {}, xorname: {:?}, dry_run: {}",
            if private { "Private" } else { "Public" },
            tag,
            name,
            dry_run
        );

        let xorname = name.unwrap_or_else(rand::random);

        if dry_run {
            return Ok((xorname, RegisterWriteAheadLog::default()));
        }

        let client = self.get_safe_client()?;
        info!("Xorname for new Register storage: {:?}", &xorname);

        // The Register's owner will be the client's public key
        let my_pk = client.public_key();

        // Store the Register on the network
        let (_, op_batch) = if private {
            // Set read and write  permissions to this application
            let mut perms = BTreeMap::default();
            let _ = perms.insert(my_pk, PrivatePermissions::new(true, true));

            client
                .store_private_register(xorname, tag, my_pk, perms)
                .await
                .map_err(|e| {
                    Error::NetDataError(format!(
                        "Failed to prepare store Private Register operation: {:?}",
                        e
                    ))
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
                    Error::NetDataError(format!(
                        "Failed to prepare store Public Register: operation {:?}",
                        e
                    ))
                })?
        };

        Ok((xorname, op_batch))
    }

    /// Low level method to read all register entries
    pub async fn read_register(
        &self,
        address: RegisterAddress,
    ) -> Result<BTreeSet<(EntryHash, Entry)>> {
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

    /// Low level method to read a register entry
    pub async fn get_register_entry(
        &self,
        address: RegisterAddress,
        hash: EntryHash,
    ) -> Result<Entry> {
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
                        encode(hash.0),
                        err
                    ))
                }
            })?;

        Ok(entry)
    }

    /// Low level method to write to register
    /// Returns a register operation batch that can be used to apply changes on the network.
    /// Nothing is sent to the network, without applying the batch, it's pretty much a dry run.
    pub async fn write_to_register(
        &self,
        address: RegisterAddress,
        entry: Entry,
        parents: BTreeSet<EntryHash>,
    ) -> Result<(EntryHash, RegisterWriteAheadLog)> {
        debug!("Writing to Register at {:?}", address);
        let client = self.get_safe_client()?;

        client
            .write_to_register(address, entry, parents)
            .await
            .map_err(|e| {
                Error::NetDataError(format!(
                    "Failed to prepare write to Register operation: {:?}",
                    e
                ))
            })
    }

    /// Low level method to apply register operations batches and send them to the network
    pub async fn apply_register_ops(&self, batch: RegisterWriteAheadLog) -> Result<()> {
        let client = self.get_safe_client()?;
        client.publish_register_ops(batch).await?;
        Ok(())
    }
}
