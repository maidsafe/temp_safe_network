// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// User Account information.
pub mod account;
/// Core client used for testing purposes.
#[cfg(any(test, feature = "testing"))]
pub mod core_client;
/// `MapInfo` utilities.
pub mod map_info;
/// Various APIs wrapped to provide resiliance for common network operations.
pub mod recoverable_apis;
/// Safe Transfers wrapper, with Money APIs
pub mod transfer_actor;

use async_trait::async_trait;

// safe-transfers wrapper
pub use self::transfer_actor::{ClientTransferValidator, TransferActor};

pub use self::account::ClientKeys;
pub use self::map_info::MapInfo;
pub use safe_nd::SafeKey;

use crate::config_handler::Config;
use crate::connection_manager::ConnectionManager;
use crate::crypto::{shared_box, shared_secretbox};
use crate::errors::CoreError;
use crate::network_event::NetworkTx;
use core::time::Duration;
use futures::{channel::mpsc, lock::Mutex};
use log::{debug, info, trace};
use lru::LruCache;
use quic_p2p::Config as QuicP2pConfig;
use safe_nd::{
    AppPermissions, AuthQuery, Blob, BlobAddress, BlobRead, ClientFullId, Cmd, DataQuery, Map,
    MapAddress, MapEntries, MapEntryActions, MapPermissionSet, MapRead, MapSeqEntries,
    MapSeqEntryActions, MapSeqValue, MapUnseqEntryActions, MapValue, MapValues, Message, MessageId,
    Money, PublicId, PublicKey, Query, QueryResponse, SeqMap, Sequence, SequenceAction,
    SequenceAddress, SequenceEntries, SequenceEntry, SequenceIndex, SequenceOwner,
    SequencePrivUserPermissions, SequencePrivatePermissions, SequencePubUserPermissions,
    SequencePublicPermissions, SequenceRead, SequenceUser, SequenceUserPermissions, UnseqMap,
};
use std::sync::Arc;

use xor_name::XorName;

use rand::{thread_rng, Rng};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    net::SocketAddr,
};

/// Capacity of the immutable data cache.
pub const IMMUT_DATA_CACHE_SIZE: usize = 300;

/// Capacity of the Sequence CRDT local replica size.
pub const SEQUENCE_CRDT_REPLICA_SIZE: usize = 300;

/// Expected cost of mutation operations.
pub const COST_OF_PUT: Money = Money::from_nano(1);

/// Return the `crust::Config` associated with the `crust::Service` (if any).
pub fn bootstrap_config() -> Result<HashSet<SocketAddr>, CoreError> {
    Ok(Config::new().quic_p2p.hard_coded_contacts)
}

// Build and sign Cmd Message Envelope
pub(crate) fn create_cmd_message(msg_contents: Cmd) -> Message {
    trace!("Creating cmd message");
    let mut rng = thread_rng();
    let random_xor = rng.gen::<XorName>();
    let id = MessageId(random_xor);
    println!("cmd msg id: {:?}", id);

    Message::Cmd {
        cmd: msg_contents,
        id,
    }
}

// Build and sign Query Message Envelope
pub(crate) fn create_query_message(msg_contents: Query) -> Message {
    trace!("Creating query message");

    let mut rng = thread_rng();
    let random_xor = rng.gen::<XorName>();
    let id = MessageId(random_xor);

    println!("query msg id: {:?}", id);
    Message::Query {
        query: msg_contents,
        id,
    }
}

async fn send_query(client: &impl Client, query: Query) -> Result<QueryResponse, CoreError> {
    // `sign` should be false for GETs on published data, true otherwise.

    println!("-->>Request going out: {:?}", query);

    let message = create_query_message(query);
    let inner = client.inner();
    let cm = &mut inner.lock().await.connection_manager;
    cm.send_query(&message).await
}

/// Trait providing an interface for self-authentication client implementations, so they can
/// interface all requests from high-level APIs to the actual routing layer and manage all
/// interactions with it. Clients are non-blocking, with an asynchronous API using the futures
/// abstraction from the futures-rs crate.
#[async_trait]
pub trait Client: Clone + Send + Sync {
    /// Associated message type.
    type Context;

    /// Return the client's ID.
    async fn full_id(&self) -> SafeKey;

    /// Return the client's public ID.
    async fn public_id(&self) -> PublicId {
        self.full_id().await.public_id()
    }

    /// Returns the client's public key.
    async fn public_key(&self) -> PublicKey {
        self.full_id().await.public_key()
    }

    /// Returns the client's owner key.
    async fn owner_key(&self) -> PublicKey;

    /// Return a `crust::Config` if the `Client` was initialized with one.
    async fn config(&self) -> Option<HashSet<SocketAddr>>;

    /// Return an associated `ClientInner` type which is expected to contain fields associated with
    /// the implementing type.
    fn inner(&self) -> Arc<Mutex<Inner>>
    where
        Self: Sized;

    /// Return the TransferActor for this client
    async fn transfer_actor(&self) -> Option<TransferActor>;

    /// Return the public encryption key.
    async fn public_encryption_key(&self) -> threshold_crypto::PublicKey;

    /// Return the secret encryption key.
    async fn secret_encryption_key(&self) -> shared_box::SecretKey;

    /// Return the public and secret encryption keys.
    async fn encryption_keypair(&self) -> (threshold_crypto::PublicKey, shared_box::SecretKey) {
        let enc_key = self.public_encryption_key().await;
        let sec_key = self.secret_encryption_key().await;
        (enc_key, sec_key)
    }

    /// Return the symmetric encryption key.
    async fn secret_symmetric_key(&self) -> shared_secretbox::Key;

    // /// Create a `Message` from the given request.
    // /// This function adds the requester signature and message ID.
    // async fn compose_message(&self, request: Request, sign: bool) -> Result<Message, CoreError> {
    //     let message_id = MessageId::new();

    //     let signature = if sign {
    //         match request.clone() {
    //             Query::Data(req) => {
    //                 let serialised_req =
    //                     bincode::serialize(&(&req, message_id)).map_err(CoreError::from)?;
    //                 Some(self.full_id().await.sign(&serialised_req))
    //             }
    //             // Request::Node(req) => {
    //             //     let serialised_req =
    //             //         bincode::serialize(&(&req, message_id)).map_err(CoreError::from)?;
    //             //     Some(self.full_id().await.sign(&serialised_req))
    //             // }
    //         }
    //     } else {
    //         None
    //     };

    //     Ok(Message::Request {
    //         request,
    //         message_id,
    //         signature,
    //     })
    // }

    /// Set request timeout.
    async fn set_timeout(&self, duration: Duration) {
        let inner = self.inner();
        inner.lock().await.timeout = duration;
    }

    /// Put unsequenced mutable data to the network
    async fn put_unseq_mutable_data(&self, data: UnseqMap) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        trace!("Put Unsequenced Map at {:?}", data.name());

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.new_map(Map::Unseq(data)).await
    }

    /// Transfer coin balance
    async fn transfer_money(
        &self,
        // client_id: Option<&ClientFullId>,
        to: PublicKey,
        amount: Money,
    ) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        info!("Transfer {} money to {:?}", amount, to);
        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.send_money(to, amount).await
    }

    /// Transfer coin balance
    async fn transfer_money_as(
        &self,
        _from: Option<PublicKey>,
        to: PublicKey,
        amount: Money,
    ) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        info!("Transfer {} money to {:?}", amount, to);
        // TODO: retrieve our actor for this clientID....
        // we can remove that and set up an API for transfer_as if needs be...

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.send_money(to, amount).await
    }

    // TODO: is this API needed at all? Why not just transfer?
    /// Creates a new balance on the network. Currently same as transfer...
    async fn create_balance(
        &self,
        _client_id: Option<&ClientFullId>,
        new_balance_owner: PublicKey,
        amount: Money,
    ) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        info!(
            "Create a new balance for {:?} with {} money.",
            new_balance_owner, amount
        );

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.send_money(new_balance_owner, amount).await
    }

    /// Get the current coin balance via TransferActor for this client.
    async fn get_balance(&self, client_id: Option<&ClientFullId>) -> Result<Money, CoreError>
    where
        Self: Sized,
    {
        trace!("Get balance for {:?}", client_id);
        // TODO: another api for getting local only...
        // TODO: handle client_id passed in, or remove

        match self.full_id().await {
            SafeKey::Client(_) => {
                // we're a standard client grabbing our own key's balance
                self.transfer_actor()
                    .await
                    .ok_or(CoreError::from("No TransferActor found for client."))?
                    .get_balance_from_network(None)
                    .await
            }
            SafeKey::App(_) => {
                // we're an app. We have no balance made at this key (yet in general)
                // so we want to check our owner's balance.
                // TODO: Apps should have their own keys w/ loaded amounts.
                // ownership / perms should come down to keys on a wallet... (how would this look vault side?)
                self.transfer_actor()
                    .await
                    .ok_or(CoreError::from("No TransferActor found for app client."))?
                    .get_balance_from_network(Some(self.owner_key().await))
                    .await
            }
        }
    }

    /// Put immutable data to the network.
    async fn put_blob<D: Into<Blob> + Send>(&self, data: D) -> Result<(), CoreError>
    where
        Self: Sized + Send,
    {
        let blob: Blob = data.into();
        trace!("Put Blob at {:?}", blob.name());

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.new_blob(blob).await
    }

    /// Get immutable data from the network. If the data exists locally in the cache then it will be
    /// immediately returned without making an actual network request.
    async fn get_blob(&self, address: BlobAddress) -> Result<Blob, CoreError>
    where
        Self: Sized,
    {
        trace!("Fetch Blob");

        let inner = self.inner();
        if let Some(data) = inner.lock().await.blob_cache.get_mut(&address) {
            trace!("Blob found in cache.");
            return Ok(data.clone());
        }

        let inner = Arc::downgrade(&self.inner());
        let res = send_query(self, Query::Data(DataQuery::Blob(BlobRead::Get(address)))).await?;
        let data = match res {
            QueryResponse::GetBlob(res) => res.map_err(CoreError::from),
            _ => return Err(CoreError::ReceivedUnexpectedEvent),
        }?;

        if let Some(inner) = inner.upgrade() {
            // Put to cache
            let _ = inner
                .lock()
                .await
                .blob_cache
                .put(*data.address(), data.clone());
        };
        Ok(data)
    }

    /// Delete unpublished immutable data from the network.
    async fn del_unpub_blob(&self, name: XorName) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        let inner = self.inner();
        if inner
            .lock()
            .await
            .blob_cache
            .pop(&BlobAddress::Private(name))
            .is_some()
        {
            trace!("Deleted PrivateBlob from cache.");
        }

        let inner = self.inner().clone();

        let _ = Arc::downgrade(&inner);
        trace!("Delete Private Blob at {:?}", name);

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.delete_blob(BlobAddress::Private(name)).await
    }

    /// Put sequenced mutable data to the network
    async fn put_seq_mutable_data(&self, data: SeqMap) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        trace!("Put Sequenced Map at {:?}", data.name());

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.new_map(Map::Seq(data)).await
    }

    /// Fetch unpublished mutable data from the network
    async fn get_unseq_map(&self, name: XorName, tag: u64) -> Result<UnseqMap, CoreError>
    where
        Self: Sized,
    {
        trace!("Fetch Unsequenced Mutable Data");

        match send_query(
            self,
            wrap_map_read(MapRead::Get(MapAddress::Unseq { name, tag })),
        )
        .await?
        {
            QueryResponse::GetMap(res) => res.map_err(CoreError::from).and_then(|map| match map {
                Map::Unseq(data) => Ok(data),
                Map::Seq(_) => Err(CoreError::ReceivedUnexpectedData),
            }),
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Fetch the value for a given key in a sequenced mutable data
    async fn get_seq_map_value(
        &self,
        name: XorName,
        tag: u64,
        key: Vec<u8>,
    ) -> Result<MapSeqValue, CoreError>
    where
        Self: Sized,
    {
        trace!("Fetch MapValue for {:?}", name);

        match send_query(
            self,
            wrap_map_read(MapRead::GetValue {
                address: MapAddress::Seq { name, tag },
                key,
            }),
        )
        .await?
        {
            QueryResponse::GetMapValue(res) => {
                res.map_err(CoreError::from).and_then(|value| match value {
                    MapValue::Seq(val) => Ok(val),
                    MapValue::Unseq(_) => Err(CoreError::ReceivedUnexpectedData),
                })
            }
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Fetch the value for a given key in a sequenced mutable data
    async fn get_unseq_map_value(
        &self,
        name: XorName,
        tag: u64,
        key: Vec<u8>,
    ) -> Result<Vec<u8>, CoreError>
    where
        Self: Sized,
    {
        trace!("Fetch MapValue for {:?}", name);

        match send_query(
            self,
            wrap_map_read(MapRead::GetValue {
                address: MapAddress::Unseq { name, tag },
                key,
            }),
        )
        .await?
        {
            QueryResponse::GetMapValue(res) => {
                res.map_err(CoreError::from).and_then(|value| match value {
                    MapValue::Unseq(val) => Ok(val),
                    MapValue::Seq(_) => Err(CoreError::ReceivedUnexpectedData),
                })
            }
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Fetch sequenced mutable data from the network
    async fn get_seq_map(&self, name: XorName, tag: u64) -> Result<SeqMap, CoreError>
    where
        Self: Sized,
    {
        trace!("Fetch Sequenced Mutable Data");

        match send_query(
            self,
            wrap_map_read(MapRead::Get(MapAddress::Seq { name, tag })),
        )
        .await?
        {
            QueryResponse::GetMap(res) => res.map_err(CoreError::from).and_then(|map| match map {
                Map::Seq(data) => Ok(data),
                Map::Unseq(_) => Err(CoreError::ReceivedUnexpectedData),
            }),
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Mutates sequenced `Map` entries in bulk
    async fn mutate_seq_map_entries(
        &self,
        name: XorName,
        tag: u64,
        actions: MapSeqEntryActions,
    ) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        trace!("Mutate Map for {:?}", name);

        let map_actions = MapEntryActions::Seq(actions);
        let address = MapAddress::Seq { name, tag };

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.edit_map_entries(address, map_actions).await
    }

    /// Mutates unsequenced `Map` entries in bulk
    async fn mutate_unseq_map_entries(
        &self,
        name: XorName,
        tag: u64,
        actions: MapUnseqEntryActions,
    ) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        trace!("Mutate Map for {:?}", name);

        let map_actions = MapEntryActions::Unseq(actions);
        let address = MapAddress::Unseq { name, tag };

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.edit_map_entries(address, map_actions).await
    }

    /// Get a shell (bare bones) version of `Map` from the network.
    async fn get_seq_map_shell(&self, name: XorName, tag: u64) -> Result<SeqMap, CoreError>
    where
        Self: Sized,
    {
        trace!("GetMapShell for {:?}", name);

        match send_query(
            self,
            wrap_map_read(MapRead::GetShell(MapAddress::Seq { name, tag })),
        )
        .await?
        {
            QueryResponse::GetMapShell(res) => {
                res.map_err(CoreError::from).and_then(|map| match map {
                    Map::Seq(data) => Ok(data),
                    _ => Err(CoreError::ReceivedUnexpectedData),
                })
            }
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Get a shell (bare bones) version of `Map` from the network.
    async fn get_unseq_map_shell(&self, name: XorName, tag: u64) -> Result<UnseqMap, CoreError>
    where
        Self: Sized,
    {
        trace!("GetMapShell for {:?}", name);

        match send_query(
            self,
            wrap_map_read(MapRead::GetShell(MapAddress::Unseq { name, tag })),
        )
        .await?
        {
            QueryResponse::GetMapShell(res) => {
                res.map_err(CoreError::from).and_then(|map| match map {
                    Map::Unseq(data) => Ok(data),
                    _ => Err(CoreError::ReceivedUnexpectedData),
                })
            }
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Get a current version of `Map` from the network.
    async fn get_map_version(&self, address: MapAddress) -> Result<u64, CoreError>
    where
        Self: Sized,
    {
        trace!("GetMapVersion for {:?}", address);

        match send_query(self, wrap_map_read(MapRead::GetVersion(address))).await? {
            QueryResponse::GetMapVersion(res) => res.map_err(CoreError::from),
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Return a complete list of entries in `Map`.
    async fn list_unseq_map_entries(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<BTreeMap<Vec<u8>, Vec<u8>>, CoreError>
    where
        Self: Sized,
    {
        trace!("ListMapEntries for {:?}", name);

        match send_query(
            self,
            wrap_map_read(MapRead::ListEntries(MapAddress::Unseq { name, tag })),
        )
        .await?
        {
            QueryResponse::ListMapEntries(res) => {
                res.map_err(CoreError::from)
                    .and_then(|entries| match entries {
                        MapEntries::Unseq(data) => Ok(data),
                        MapEntries::Seq(_) => Err(CoreError::ReceivedUnexpectedData),
                    })
            }
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Return a complete list of entries in `Map`.
    async fn list_seq_map_entries(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<MapSeqEntries, CoreError>
    where
        Self: Sized,
    {
        trace!("ListSeqMapEntries for {:?}", name);

        match send_query(
            self,
            wrap_map_read(MapRead::ListEntries(MapAddress::Seq { name, tag })),
        )
        .await?
        {
            QueryResponse::ListMapEntries(res) => {
                res.map_err(CoreError::from)
                    .and_then(|entries| match entries {
                        MapEntries::Seq(data) => Ok(data),
                        MapEntries::Unseq(_) => Err(CoreError::ReceivedUnexpectedData),
                    })
            }
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Return a list of keys in `Map` stored on the network.
    async fn list_map_keys(&self, address: MapAddress) -> Result<BTreeSet<Vec<u8>>, CoreError>
    where
        Self: Sized,
    {
        trace!("ListMapKeys for {:?}", address);

        match send_query(self, wrap_map_read(MapRead::ListKeys(address))).await? {
            QueryResponse::ListMapKeys(res) => res.map_err(CoreError::from),
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Return a list of values in a Sequenced Mutable Data
    async fn list_seq_map_values(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<Vec<MapSeqValue>, CoreError>
    where
        Self: Sized,
    {
        trace!("List MapValues for {:?}", name);

        match send_query(
            self,
            wrap_map_read(MapRead::ListValues(MapAddress::Seq { name, tag })),
        )
        .await?
        {
            QueryResponse::ListMapValues(res) => {
                res.map_err(CoreError::from)
                    .and_then(|values| match values {
                        MapValues::Seq(data) => Ok(data),
                        MapValues::Unseq(_) => Err(CoreError::ReceivedUnexpectedData),
                    })
            }
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Return the permissions set for a particular user
    async fn list_map_user_permissions(
        &self,
        address: MapAddress,
        user: PublicKey,
    ) -> Result<MapPermissionSet, CoreError>
    where
        Self: Sized,
    {
        trace!("GetMapUserPermissions for {:?}", address);

        match send_query(
            self,
            wrap_map_read(MapRead::ListUserPermissions { address, user }),
        )
        .await?
        {
            QueryResponse::ListMapUserPermissions(res) => res.map_err(CoreError::from),
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Returns a list of values in an Unsequenced Mutable Data
    async fn list_unseq_map_values(
        &self,
        name: XorName,
        tag: u64,
    ) -> Result<Vec<Vec<u8>>, CoreError>
    where
        Self: Sized,
    {
        trace!("List MapValues for {:?}", name);

        match send_query(
            self,
            wrap_map_read(MapRead::ListValues(MapAddress::Unseq { name, tag })),
        )
        .await?
        {
            QueryResponse::ListMapValues(res) => {
                res.map_err(CoreError::from)
                    .and_then(|values| match values {
                        MapValues::Unseq(data) => Ok(data),
                        MapValues::Seq(_) => Err(CoreError::ReceivedUnexpectedData),
                    })
            }
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    // ======= Sequence Data =======
    //
    /// Store Private Sequence Data into the Network
    async fn store_private_sequence(
        &self,
        name: XorName,
        tag: u64,
        owner: PublicKey,
        permissions: BTreeMap<PublicKey, SequencePrivUserPermissions>,
    ) -> Result<SequenceAddress, CoreError> {
        trace!("Store Private Sequence Data {:?}", name);
        let mut data = Sequence::new_private(self.public_key().await, name, tag);
        let address = *data.address();
        let _ = data.set_private_permissions(permissions)?;
        let _ = data.set_owner(owner);

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.new_sequence(data.clone()).await?;

        // Store in local Sequence CRDT replica
        let _ = self
            .inner()
            .lock()
            .await
            .sequence_cache
            .put(*data.address(), data);

        Ok(address)
    }

    /// Store Public Sequence Data into the Network
    async fn store_pub_sequence(
        &self,
        name: XorName,
        tag: u64,
        owner: PublicKey,
        permissions: BTreeMap<SequenceUser, SequencePubUserPermissions>,
    ) -> Result<SequenceAddress, CoreError> {
        trace!("Store Public Sequence Data {:?}", name);
        let mut data = Sequence::new_pub(self.public_key().await, name, tag);
        let address = *data.address();
        let _ = data.set_pub_permissions(permissions)?;
        let _ = data.set_owner(owner);

        //we can send the mutation to the network's replicas
        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.new_sequence(data.clone()).await?;

        // Store in local Sequence CRDT replica
        let _ = self
            .inner()
            .lock()
            .await
            .sequence_cache
            .put(*data.address(), data);

        Ok(address)
    }

    /// Get Sequence Data from the Network
    async fn get_sequence(&self, address: SequenceAddress) -> Result<Sequence, CoreError> {
        trace!("Get Sequence Data at {:?}", address.name());
        // First try to fetch it from local CRDT replica
        // TODO: implement some logic to refresh data from the network if local replica
        // is too old, to mitigate the risk of successfully apply mutations locally but which
        // can fail on other replicas, e.g. due to being out of sync with permissions/owner
        if let Some(sequence) = self.inner().lock().await.sequence_cache.get(&address) {
            trace!("Sequence found in local CRDT replica");
            return Ok(sequence.clone());
        }

        trace!("Sequence not found in local CRDT replica");
        // Let's fetch it from the network then
        let sequence = match send_query(self, wrap_seq_read(SequenceRead::Get(address))).await? {
            QueryResponse::GetSequence(res) => res.map_err(CoreError::from),
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }?;

        trace!("Store Sequence in local CRDT replica");
        // Store in local Sequence CRDT replica
        let _ = self
            .inner()
            .lock()
            .await
            .sequence_cache
            .put(*sequence.address(), sequence.clone());

        Ok(sequence)
    }

    /// Get the last data entry from a Sequence Data.
    async fn get_sequence_last_entry(
        &self,
        address: SequenceAddress,
    ) -> Result<(u64, SequenceEntry), CoreError> {
        trace!(
            "Get latest entry from Sequence Data at {:?}",
            address.name()
        );

        let sequence = self.get_sequence(address).await?;
        match sequence.last_entry() {
            Some(entry) => Ok((sequence.entries_index() - 1, entry.to_vec())),
            None => Err(CoreError::from(safe_nd::Error::NoSuchEntry)),
        }
    }

    /// Get a set of Entries for the requested range from a Sequence.
    async fn get_sequence_range(
        &self,
        address: SequenceAddress,
        range: (SequenceIndex, SequenceIndex),
    ) -> Result<SequenceEntries, CoreError> {
        trace!(
            "Get range of entries from Sequence Data at {:?}",
            address.name()
        );

        let sequence = self.get_sequence(address).await?;
        sequence
            .in_range(range.0, range.1)
            .ok_or_else(|| CoreError::from(safe_nd::Error::NoSuchEntry))
    }

    /// Append to Sequence Data
    async fn sequence_append(
        &self,
        address: SequenceAddress,
        entry: SequenceEntry,
    ) -> Result<(), CoreError> {
        // First we fetch it so we can get the causality info,
        // either from local CRDT replica or from the network if not found
        let mut sequence = self.get_sequence(address).await?;

        // We do a permissions check just to make sure it won't fail when the operation
        // is broadcasted to the network, assuming our replica is in sync and up to date
        // with the permissions and ownership information compared with the replicas on the network.
        sequence.check_permission(SequenceAction::Append, self.public_id().await.public_key())?;

        // We can now append the entry to the Sequence
        let op = sequence.append(entry);

        // Update the local Sequence CRDT replica
        let _ = self
            .inner()
            .lock()
            .await
            .sequence_cache
            .put(*sequence.address(), sequence.clone());

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        // Finally we can send the mutation to the network's replicas
        actor.append_to_sequence(op).await
    }

    /// Get the set of Permissions of a Public Sequence.
    async fn get_sequence_pub_permissions(
        &self,
        address: SequenceAddress,
    ) -> Result<SequencePublicPermissions, CoreError> {
        trace!(
            "Get permissions from Public Sequence Data at {:?}",
            address.name()
        );

        // TODO: perhaps we want to grab it directly from the network and update local replica
        let sequence = self.get_sequence(address).await?;
        let perms = sequence
            .pub_permissions(sequence.permissions_index() - 1)
            .map_err(CoreError::from)?;

        Ok(perms.clone())
    }

    /// Get the set of Permissions of a Private Sequence.
    async fn get_sequence_private_permissions(
        &self,
        address: SequenceAddress,
    ) -> Result<SequencePrivatePermissions, CoreError> {
        trace!(
            "Get permissions from Private Sequence Data at {:?}",
            address.name()
        );

        // TODO: perhaps we want to grab it directly from the network and update local replica
        let sequence = self.get_sequence(address).await?;
        let perms = sequence
            .private_permissions(sequence.permissions_index() - 1)
            .map_err(CoreError::from)?;

        Ok(perms.clone())
    }

    /// Get the set of Permissions for a specific user in a Sequence.
    async fn get_sequence_user_permissions(
        &self,
        address: SequenceAddress,
        user: SequenceUser,
    ) -> Result<SequenceUserPermissions, CoreError> {
        trace!(
            "Get permissions for user {:?} from Sequence Data at {:?}",
            user,
            address.name()
        );

        // TODO: perhaps we want to grab it directly from the network and update local replica
        let sequence = self.get_sequence(address).await?;
        let perms = sequence
            .user_permissions(user, sequence.permissions_index() - 1)
            .map_err(CoreError::from)?;

        Ok(perms)
    }

    /// Set permissions to Public Sequence Data
    async fn sequence_set_pub_permissions(
        &self,
        address: SequenceAddress,
        permissions: BTreeMap<SequenceUser, SequencePubUserPermissions>,
    ) -> Result<(), CoreError> {
        // First we fetch it either from local CRDT replica or from the network if not found
        let mut sequence = self.get_sequence(address).await?;

        // We do a permissions check just to make sure it won't fail when the operation
        // is broadcasted to the network, assuming our replica is in sync and up to date
        // with the permissions information compared with the replicas on the network.
        sequence.check_permission(
            SequenceAction::ManagePermissions,
            self.public_id().await.public_key(),
        )?;

        // We can now set the new permissions to the Sequence
        let op = sequence.set_pub_permissions(permissions)?;

        // Update the local Sequence CRDT replica
        let _ = self
            .inner()
            .lock()
            .await
            .sequence_cache
            .put(*sequence.address(), sequence.clone());

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        // Finally we can send the mutation to the network's replicas
        actor.edit_sequence_public_perms(op).await
    }

    /// Set permissions to Private Sequence Data
    async fn sequence_set_private_permissions(
        &self,
        address: SequenceAddress,
        permissions: BTreeMap<PublicKey, SequencePrivUserPermissions>,
    ) -> Result<(), CoreError> {
        // First we fetch it either from local CRDT replica or from the network if not found
        let mut sequence = self.get_sequence(address).await?;

        // We do a permissions check just to make sure it won't fail when the operation
        // is broadcasted to the network, assuming our replica is in sync and up to date
        // with the permissions information compared with the replicas on the network.
        // TODO: if it fails, try to sync-up perms with rmeote replicas and try once more
        sequence.check_permission(
            SequenceAction::ManagePermissions,
            self.public_id().await.public_key(),
        )?;

        // We can now set the new permissions to the Sequence
        let op = sequence.set_private_permissions(permissions)?;

        // Update the local Sequence CRDT replica
        let _ = self
            .inner()
            .lock()
            .await
            .sequence_cache
            .put(*sequence.address(), sequence.clone());

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        // Finally we can send the mutation to the network's replicas
        actor.edit_sequence_private_perms(op).await
    }

    /// Get the owner of a Sequence.
    async fn get_sequence_owner(
        &self,
        address: SequenceAddress,
    ) -> Result<SequenceOwner, CoreError> {
        trace!("Get owner of the Sequence Data at {:?}", address.name());

        // TODO: perhaps we want to grab it directly from the network and update local replica
        let sequence = self.get_sequence(address).await?;
        let owner = sequence.owner(sequence.owners_index() - 1).ok_or_else(|| {
            CoreError::from("Unexpectedly failed to obtain current owner of Sequence")
        })?;

        Ok(*owner)
    }

    /// Set the new owner of a Sequence Data
    async fn sequence_set_owner(
        &self,
        address: SequenceAddress,
        owner: PublicKey,
    ) -> Result<(), CoreError> {
        // First we fetch it either from local CRDT replica or from the network if not found
        let mut sequence = self.get_sequence(address).await?;

        // We do a permissions check just to make sure it won't fail when the operation
        // is broadcasted to the network, assuming our replica is in sync and up to date
        // with the ownership information compared with the replicas on the network.
        sequence.check_permission(
            SequenceAction::ManagePermissions,
            self.public_id().await.public_key(),
        )?;

        // We can now set the new owner to the Sequence
        let op = sequence.set_owner(owner);

        // Update the local Sequence CRDT replica
        let _ = self
            .inner()
            .lock()
            .await
            .sequence_cache
            .put(*sequence.address(), sequence.clone());

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        // Finally we can send the mutation to the network's replicas
        actor.set_sequence_owner(op).await
    }

    /// Delete Private Sequence Data from the Network
    async fn delete_sequence(&self, address: SequenceAddress) -> Result<(), CoreError> {
        trace!("Delete Private Sequence Data {:?}", address.name());

        // Delete it from local Sequence CRDT replica
        let _ = self.inner().lock().await.sequence_cache.pop(&address);

        trace!("Deleted local Private Sequence {:?}", address.name());

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.delete_sequence(address).await
    }

    // ========== END of Sequence Data functions =========

    /// Return a list of permissions in `Map` stored on the network.
    async fn list_map_permissions(
        &self,
        address: MapAddress,
    ) -> Result<BTreeMap<PublicKey, MapPermissionSet>, CoreError>
    where
        Self: Sized,
    {
        trace!("List MapPermissions for {:?}", address);

        match send_query(self, wrap_map_read(MapRead::ListPermissions(address))).await? {
            QueryResponse::ListMapPermissions(res) => res.map_err(CoreError::from),
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Updates or inserts a permissions set for a user
    async fn set_map_user_permissions(
        &self,
        address: MapAddress,
        user: PublicKey,
        permissions: MapPermissionSet,
        version: u64,
    ) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        trace!("SetMapUserPermissions for {:?}", address);

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor
            .set_map_user_perms(address, user, permissions, version)
            .await
    }

    /// Updates or inserts a permissions set for a user
    async fn del_map_user_permissions(
        &self,
        address: MapAddress,
        user: PublicKey,
        version: u64,
    ) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        trace!("DelMapUserPermissions for {:?}", address);

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.delete_map_user_perms(address, user, version).await
    }

    /// Sends an ownership transfer request.
    #[allow(unused)]
    fn change_map_owner(
        &self,
        name: XorName,
        tag: u64,
        new_owner: PublicKey,
        version: u64,
    ) -> Result<(), CoreError> {
        unimplemented!();
    }

    /// Set the coin balance to a specific value for testing
    #[cfg(any(test, feature = "testing"))]
    async fn test_simulate_farming_payout_client(&self, amount: Money) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        debug!(
            "Set the coin balance of {:?} to {:?}",
            self.public_key().await,
            amount,
        );

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor
            .trigger_simulated_farming_payout(self.public_key().await, amount)
            .await
    }
}

/// Creates a throw-away client to execute requests sequentially.
pub async fn temp_client<F, R>(identity: &ClientFullId, mut func: F) -> Result<R, CoreError>
where
    F: FnMut(&mut ConnectionManager, &SafeKey) -> Result<R, CoreError>,
{
    let full_id = SafeKey::client(identity.clone());

    let (net_tx, _net_rx) = mpsc::unbounded();

    let mut cm = attempt_bootstrap(&Config::new().quic_p2p, &net_tx, full_id.clone()).await?;

    func(&mut cm, &full_id)
}

/// Create a new mock balance at an arbitrary address.
pub async fn test_create_balance(owner: &ClientFullId, amount: Money) -> Result<(), CoreError> {
    trace!("Create test balance of {} for {:?}", amount, owner);

    let full_id = SafeKey::client(owner.clone());

    let (net_tx, _net_rx) = mpsc::unbounded();

    let cm = attempt_bootstrap(&Config::new().quic_p2p, &net_tx, full_id.clone()).await?;

    // actor starts with 10....
    let mut actor = TransferActor::new(full_id.clone(), cm).await?;

    let public_id = full_id.public_id();

    // Create the balance for the client
    let _new_balance_owner = match public_id.clone() {
        PublicId::Client(id) => *id.public_key(),
        x => return Err(CoreError::from(format!("Unexpected ID type {:?}", x))),
    };

    let public_key = full_id.public_key();

    actor
        .trigger_simulated_farming_payout(public_key, amount)
        .await?;

    Ok(())
}

/// This trait implements functions that are supposed to be called only by `CoreClient` and `AuthClient`.
/// Applications are not allowed to `DELETE Map` and get/mutate auth keys, hence `AppClient` does not implement
/// this trait.
#[async_trait]
pub trait AuthActions: Client + Clone + 'static {
    /// Fetches a list of authorised keys and version.
    async fn list_auth_keys_and_version(
        &self,
    ) -> Result<(BTreeMap<PublicKey, AppPermissions>, u64), CoreError>
    where
        Self: Sized,
    {
        trace!("ListAuthKeysAndVersion");

        let client_pk = self.public_key().await;

        match send_query(
            self,
            wrap_client_auth_query(AuthQuery::ListAuthKeysAndVersion { client: client_pk }),
        )
        .await?
        {
            QueryResponse::ListAuthKeysAndVersion(res) => res.map_err(CoreError::from),
            _ => Err(CoreError::ReceivedUnexpectedEvent),
        }
    }

    /// Adds a new authorised key.
    async fn ins_auth_key(
        &self,
        key: PublicKey,
        permissions: AppPermissions,
        version: u64,
    ) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        trace!("InsAuthKey ({:?})", key);

        self.transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?
            .insert_auth_key(key, permissions, version)
            .await
    }

    /// Removes an authorised key.
    async fn del_auth_key(&self, key: PublicKey, version: u64) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        trace!("DelAuthKey ({:?})", key);

        self.transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?
            .delete_auth_key(key, version)
            .await
    }

    /// Delete Map from network
    async fn delete_map(&self, address: MapAddress) -> Result<(), CoreError>
    where
        Self: Sized,
    {
        trace!("Delete entire Mutable Data at {:?}", address);

        let mut actor = self
            .transfer_actor()
            .await
            .ok_or(CoreError::from("No TransferActor found for client."))?;

        actor.delete_map(address).await
    }
}

// TODO: Consider deprecating this struct once trait fields are stable. See
// https://github.com/nikomatsakis/fields-in-traits-rfc.
/// Struct containing fields expected by the `Client` trait. Implementers of `Client` should be
/// composed around this struct.
#[allow(unused)] // FIXME
pub struct Inner {
    connection_manager: ConnectionManager,
    blob_cache: LruCache<BlobAddress, Blob>,
    /// Sequence CRDT replica
    sequence_cache: LruCache<SequenceAddress, Sequence>,
    timeout: Duration,
    net_tx: NetworkTx,
}

impl Inner {
    /// Create a new `ClientInner` object.
    #[allow(clippy::too_many_arguments)]
    pub fn new(connection_manager: ConnectionManager, timeout: Duration, net_tx: NetworkTx) -> Inner
    where
        Self: Sized,
    {
        Self {
            connection_manager,
            blob_cache: LruCache::new(IMMUT_DATA_CACHE_SIZE),
            sequence_cache: LruCache::new(SEQUENCE_CRDT_REPLICA_SIZE),
            timeout,
            net_tx,
        }
    }

    /// Get the connection manager associated with the client
    pub fn cm(&mut self) -> &mut ConnectionManager
    where
        Self: Sized,
    {
        &mut self.connection_manager
    }
}

/// Send a request and wait for a response.
/// This function is blocking.
// pub async fn req(
//     cm: &mut ConnectionManager,
//     request: Request,
//     full_id_new: &SafeKey,
// ) -> Result<Response, CoreError> {
//     let message_id = MessageId::new();
//     let signature = full_id_new.sign(&unwrap!(bincode::serialize(&(&request, message_id))));

//     cm.send_query(
//         &full_id_new.public_id(),
//         &Message::Request {
//             request,
//             message_id,
//             signature: Some(signature),
//         },
//     )
//     .await
// }

/// Utility function that bootstraps a client to the network. If there is a failure then it retries.
/// After a maximum of three attempts if the boostrap process still fails, then an error is returned.
pub async fn attempt_bootstrap(
    qp2p_config: &QuicP2pConfig,
    net_tx: &NetworkTx,
    safe_key: SafeKey,
) -> Result<ConnectionManager, CoreError> {
    let mut attempts: u32 = 0;

    loop {
        let mut connection_manager = ConnectionManager::new(qp2p_config.clone(), safe_key.clone())?;
        let res = connection_manager.bootstrap().await;
        match res {
            Ok(()) => return Ok(connection_manager),
            Err(err) => {
                attempts += 1;
                if attempts < 3 {
                    trace!("Error connecting to network! Retrying... ({})", attempts);
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
    use crate::utils::{
        generate_random_vector,
        test_utils::{calculate_new_balance, gen_bls_keypair, random_client},
    };
    use safe_nd::{
        Error as SndError, MapAction, MapKind, Money, PrivateBlob, PublicBlob,
        SequencePrivUserPermissions,
    };
    use std::str::FromStr;
    use unwrap::unwrap;
    use xor_name::XorName;

    // Test putting and getting pub blob.
    #[tokio::test]
    async fn pub_blob_test() -> Result<(), CoreError> {
        let client = random_client()?;
        // The `random_client()` initializes the client with 10 money.
        let start_bal = unwrap!(Money::from_str("10"));

        let value = generate_random_vector::<u8>(10);
        let data = PublicBlob::new(value.clone());
        let address = *data.address();
        let pk = gen_bls_keypair().public_key();

        let test_data = PrivateBlob::new(value, pk);
        let res = client
            // Get inexistent blob
            .get_blob(address)
            .await;
        match res {
            Ok(data) => panic!("Pub blob should not exist yet: {:?}", data),
            Err(CoreError::DataError(SndError::NoSuchData)) => (),
            Err(e) => panic!("Unexpected: {:?}", e),
        }
        // Put blob
        client.put_blob(data.clone()).await?;
        let res = client.put_blob(test_data.clone()).await;
        match res {
            Ok(_) => panic!("Unexpected Success: Validating owners should fail"),
            Err(CoreError::DataError(SndError::InvalidOwners)) => (),
            Err(e) => panic!("Unexpected: {:?}", e),
        }

        let balance = client.get_balance(None).await?;
        let expected_bal = calculate_new_balance(start_bal, Some(2), None);
        assert_eq!(balance, expected_bal);
        // Fetch blob
        let fetched_data = client.get_blob(address).await?;
        assert_eq!(*fetched_data.address(), address);
        Ok(())
    }

    // Test putting, getting, and deleting unpub blob.
    #[tokio::test]
    async fn unpub_blob_test() -> Result<(), CoreError> {
        println!("blob_Test________");
        crate::utils::test_utils::init_log();
        // The `random_client()` initializes the client with 10 money.
        let start_bal = unwrap!(Money::from_str("10"));
        println!("blob_Test_______pre client_");

        let client = random_client()?;
        println!("blob_Test_______post client_");

        let client9 = client.clone();

        let value = generate_random_vector::<u8>(10);
        let data = PrivateBlob::new(value.clone(), client.public_key().await);
        let data2 = data.clone();
        let data3 = data.clone();
        let address = *data.address();
        assert_eq!(address, *data2.address());

        let pub_data = PublicBlob::new(value);

        let res = client
            // Get inexistent blob
            .get_blob(address)
            .await;
        match res {
            Ok(_) => panic!("Private blob should not exist yet"),
            Err(CoreError::DataError(SndError::NoSuchData)) => (),
            Err(e) => panic!("Unexpected: {:?}", e),
        }

        // Put blob
        client.put_blob(data.clone()).await?;
        // Test putting unpub blob with the same value.
        // Should conflict because duplication does .await?;not apply to unpublished data.
        let res = client.put_blob(data2.clone()).await;
        match res {
            Err(CoreError::DataError(SndError::DataExists)) => (),
            res => panic!("Unexpected: {:?}", res),
        }
        let balance = client.get_balance(None).await?;
        // mutation_count of 3 as even our failed op counts as a mutation
        let expected_bal = calculate_new_balance(start_bal, Some(3), None);
        assert_eq!(balance, expected_bal);

        // Test putting published blob with the same value. Should not conflict.
        client.put_blob(pub_data).await?;
        // Fetch blob
        let fetched_data = client.get_blob(address).await?;

        assert_eq!(*fetched_data.address(), address);

        // Delete blob
        client.del_unpub_blob(*address.name()).await?;
        // Make sure blob was deleted
        let res = client.get_blob(address).await;
        match res {
            Ok(_) => panic!("Private blob still exists after deletion"),
            Err(error) => assert!(error.to_string().contains("Chunk not found")),
        }

        // Test putting unpub blob with the same value again. Should not conflict.
        client9.put_blob(data3.clone()).await?;
        Ok(())
    }

    // 1. Create unseq. map with some entries and perms and put it on the network
    // 2. Fetch the shell version, entries, keys, values anv verify them
    // 3. Fetch the entire. data object and verify
    #[tokio::test]
    pub async fn unseq_map_test() -> Result<(), CoreError> {
        let client = random_client()?;

        let name = XorName(rand::random());
        let tag = 15001;
        let mut entries: BTreeMap<Vec<u8>, Vec<u8>> = Default::default();
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new().allow(MapAction::Read);
        let _ = permissions.insert(client.public_key().await, permission_set);
        let _ = entries.insert(b"key".to_vec(), b"value".to_vec());
        let entries_keys = entries.keys().cloned().collect();
        let entries_values: Vec<Vec<u8>> = entries.values().cloned().collect();

        let data = UnseqMap::new_with_data(
            name,
            tag,
            entries.clone(),
            permissions,
            client.public_key().await,
        );
        client.put_unseq_mutable_data(data.clone()).await?;
        println!("Put unseq. Map successfully");

        let version = client
            .get_map_version(MapAddress::Unseq { name, tag })
            .await?;
        assert_eq!(version, 0);
        let fetched_entries = client.list_unseq_map_entries(name, tag).await?;
        assert_eq!(fetched_entries, entries);
        let keys = client
            .list_map_keys(MapAddress::Unseq { name, tag })
            .await?;
        assert_eq!(keys, entries_keys);
        let values = client.list_unseq_map_values(name, tag).await?;
        assert_eq!(values, entries_values);
        let fetched_data = client.get_unseq_map(*data.name(), data.tag()).await?;
        assert_eq!(fetched_data.name(), data.name());
        assert_eq!(fetched_data.tag(), data.tag());
        Ok(())
    }

    // 1. Create an put seq. map on the network with some entries and permissions.
    // 2. Fetch the shell version, entries, keys, values anv verify them
    // 3. Fetch the entire. data object and verify
    #[tokio::test]
    pub async fn seq_map_test() -> Result<(), CoreError> {
        let client = random_client()?;

        let name = XorName(rand::random());
        let tag = 15001;
        let mut entries: MapSeqEntries = Default::default();
        let _ = entries.insert(
            b"key".to_vec(),
            MapSeqValue {
                data: b"value".to_vec(),
                version: 0,
            },
        );
        let entries_keys = entries.keys().cloned().collect();
        let entries_values: Vec<MapSeqValue> = entries.values().cloned().collect();
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new().allow(MapAction::Read);
        let _ = permissions.insert(client.public_key().await, permission_set);
        let data = SeqMap::new_with_data(
            name,
            tag,
            entries.clone(),
            permissions,
            client.public_key().await,
        );

        client.put_seq_mutable_data(data.clone()).await?;
        println!("Put seq. Map successfully");

        let fetched_entries = client.list_seq_map_entries(name, tag).await?;
        assert_eq!(fetched_entries, entries);
        let map_shell = client.get_seq_map_shell(name, tag).await?;
        assert_eq!(*map_shell.name(), name);
        assert_eq!(map_shell.tag(), tag);
        assert_eq!(map_shell.entries().len(), 0);
        let keys = client.list_map_keys(MapAddress::Seq { name, tag }).await?;
        assert_eq!(keys, entries_keys);
        let values = client.list_seq_map_values(name, tag).await?;
        assert_eq!(values, entries_values);
        let fetched_data = client.get_seq_map(name, tag).await?;
        assert_eq!(fetched_data.name(), data.name());
        assert_eq!(fetched_data.tag(), data.tag());
        assert_eq!(fetched_data.entries().len(), 1);
        Ok(())
    }

    // 1. Put seq. map on the network and then delete it
    // 2. Try getting the data object. It should panic
    #[tokio::test]
    pub async fn del_seq_map_test() -> Result<(), CoreError> {
        let client = random_client()?;
        let name = XorName(rand::random());
        let tag = 15001;
        let mapref = MapAddress::Seq { name, tag };
        let data = SeqMap::new_with_data(
            name,
            tag,
            Default::default(),
            Default::default(),
            client.public_key().await,
        );

        client.put_seq_mutable_data(data.clone()).await?;
        client.delete_map(mapref).await?;
        let res = client.get_unseq_map(*data.name(), data.tag()).await;
        match res {
            Err(CoreError::DataError(SndError::NoSuchData)) => (),
            _ => panic!("Unexpected success"),
        }
        Ok(())
    }

    // 1. Put unseq. map on the network and then delete it
    // 2. Try getting the data object. It should panic
    #[tokio::test]
    pub async fn del_unseq_map_test() -> Result<(), CoreError> {
        let client = random_client()?;
        let name = XorName(rand::random());
        let tag = 15001;
        let mapref = MapAddress::Unseq { name, tag };
        let data = UnseqMap::new_with_data(
            name,
            tag,
            Default::default(),
            Default::default(),
            client.public_key().await,
        );

        client.put_unseq_mutable_data(data.clone()).await?;
        client.delete_map(mapref).await?;

        let res = client.get_unseq_map(*data.name(), data.tag()).await;
        match res {
            Err(CoreError::DataError(SndError::NoSuchData)) => (),
            _ => panic!("Unexpected success"),
        }

        Ok(())
    }

    // TODO: Wallet only client doesn't currently exist.
    // 1. Create 2 accounts and create a wallet only for account A.
    // 2. Try to transfer money from A to inexistent wallet. This request should fail.
    // 3. Try to request balance of wallet B. This request should fail.
    // 4. Now create a wallet for account B and transfer some money to A. This should pass.
    // 5. Try to request transfer from wallet A using account B. This request should succeed
    // (because transfers are always open).
    #[tokio::test]
    #[ignore]
    async fn money_permissions() {
        let client = random_client().unwrap();
        let wallet_a_addr = client.public_key().await;
        let random_client_key = *ClientFullId::new_bls(&mut rand::thread_rng())
            .public_id()
            .public_key();
        let res = client
            .transfer_money(random_client_key, unwrap!(Money::from_str("5.0")))
            .await;
        match res {
            Err(CoreError::DataError(SndError::NoSuchBalance)) => (),
            res => panic!("Unexpected result: {:?}", res),
        }

        let client = random_client().unwrap();
        let res = client.get_balance(None).await;
        // Subtract to cover the cost of inserting the login packet
        let expected_amt = unwrap!(Money::from_str("10")
            .ok()
            .and_then(|x| x.checked_sub(COST_OF_PUT)));
        match res {
            Ok(fetched_amt) => assert_eq!(expected_amt, fetched_amt),
            res => panic!("Unexpected result: {:?}", res),
        }
        client
            .test_simulate_farming_payout_client(unwrap!(Money::from_str("50.0")))
            .await
            .unwrap();
        let _ = client
            .transfer_money(wallet_a_addr, unwrap!(Money::from_str("10")))
            .await;

        let res = client.get_balance(None).await;
        let expected_amt = unwrap!(Money::from_str("40"));
        match res {
            Ok(fetched_amt) => assert_eq!(expected_amt, fetched_amt),
            res => panic!("Unexpected result: {:?}", res),
        }
    }

    // TODO: Update when login packet is decided to sort out "anonymous" wallets (and eg key clients)
    // 1. Create a client with a wallet. Create an anonymous wallet preloading it from the client's wallet.
    // 2. Transfer some safecoin from the anonymous wallet to the client.
    // 3. Fetch the balances of both the wallets and verify them.
    // 5. Try to create a balance using an inexistent wallet. This should fail.
    #[tokio::test]
    async fn random_clients() {
        let client = random_client().unwrap();
        // starter amount after creating login packet
        let wallet1 = client.public_key().await;
        let init_bal = unwrap!(Money::from_str("490.0")); // 500 in total

        let client2 = random_client().unwrap();

        let bls_pk = client2.public_id().await.public_key();

        client
            .test_simulate_farming_payout_client(init_bal)
            .await
            .unwrap();
        assert_eq!(
            client.get_balance(None).await.unwrap(),
            unwrap!(Money::from_str("499.999999999"))
        ); // 500 - 1nano for encrypted-account-data

        let _ = client
            .create_balance(None, bls_pk, unwrap!(Money::from_str("100.0")))
            .await
            .unwrap();

        assert_eq!(
            client.get_balance(None).await.unwrap(),
            unwrap!(Money::from_str("399.999999999"))
        );
        assert_eq!(
            client2.get_balance(None).await.unwrap(),
            unwrap!(Money::from_str("109.999999999"))
        );

        let _ = client2
            .transfer_money(wallet1, unwrap!(Money::from_str("5.0")))
            .await
            .unwrap();

        let balance = client2.get_balance(None).await.unwrap();
        assert_eq!(balance, unwrap!(Money::from_str("104.999999999")));
        let balance = client.get_balance(None).await.unwrap();

        // we add ten when testing to created clients
        let initial_bal_with_default_ten = Money::from_str("500").unwrap();
        let expected = calculate_new_balance(
            initial_bal_with_default_ten,
            Some(1),
            Some(unwrap!(Money::from_str("95"))),
        );
        assert_eq!(balance, expected);
        let random_pk = gen_bls_keypair().public_key();

        let nonexistent_client = random_client().unwrap();

        let res = nonexistent_client
            .create_balance(None, random_pk, unwrap!(Money::from_str("100.0")))
            .await;
        match res {
            Err(CoreError::DataError(e)) => {
                assert_eq!(e.to_string(), "Not enough money to complete this operation");
            }
            res => panic!("Unexpected result: {:?}", res),
        }
    }

    // 1. Create a client A with a wallet and allocate some test safecoin to it.
    // 2. Get the balance and verify it.
    // 3. Create another client B with a wallet holding some safecoin.
    // 4. Transfer some money from client B to client A and verify the new balance.
    // 5. Fetch the transfer using the transfer ID and verify the amount.
    // 6. Try to do a coin transfer without enough funds, it should return `InsufficientBalance`
    // 7. Try to do a coin transfer with the amount set to 0, it should return `InvalidOperation`
    // 8. Set the client's balance to zero and try to put data. It should fail.
    #[tokio::test]
    async fn money_balance_transfer() {
        let client = random_client().unwrap();

        // let wallet1: XorName =
        let _owner_key = client.owner_key().await;
        let wallet1 = client.public_key().await;

        client
            .test_simulate_farming_payout_client(unwrap!(Money::from_str("100.0")))
            .await
            .unwrap();
        let balance = client.get_balance(None).await.unwrap();
        assert_eq!(balance, unwrap!(Money::from_str("109.999999999"))); // 10 coins added automatically w/ farming sim on account creation. 1 nano paid.

        let client = random_client().unwrap();
        let init_bal = unwrap!(Money::from_str("10"));
        let orig_balance = client.get_balance(None).await.unwrap();
        let _ = client
            .transfer_money(wallet1, unwrap!(Money::from_str("5.0")))
            .await
            .unwrap();
        let new_balance = client.get_balance(None).await.unwrap();
        assert_eq!(
            new_balance,
            unwrap!(orig_balance.checked_sub(unwrap!(Money::from_str("5.0")))),
        );

        let res = client
            .transfer_money(wallet1, unwrap!(Money::from_str("5000")))
            .await;
        match res {
            Err(CoreError::DataError(SndError::InsufficientBalance)) => (),
            res => panic!("Unexpected result: {:?}", res),
        };
        // Check if money is refunded
        let balance = client.get_balance(None).await.unwrap();
        let expected =
            calculate_new_balance(init_bal, Some(1), Some(unwrap!(Money::from_str("5"))));
        assert_eq!(balance, expected);

        let client_to_get_all_money = random_client().unwrap();
        // send all our money elsewhere to make sure we fail the next put
        let _ = client
            .transfer_money(
                client_to_get_all_money.public_key().await,
                unwrap!(Money::from_str("4.999999999")),
            )
            .await
            .unwrap();
        let data = PublicBlob::new(generate_random_vector::<u8>(10));
        let res = client.put_blob(data).await;
        match res {
            Err(CoreError::DataError(SndError::InsufficientBalance)) => (),
            res => panic!(
                "Unexpected result in money transfer test, putting without balance: {:?}",
                res
            ),
        };
    }

    // 1. Create a client that PUTs some map on the network
    // 2. Create a different client that tries to delete the data. It should panic.
    #[tokio::test]
    pub async fn del_unseq_map_permission_test() -> Result<(), CoreError> {
        let name = XorName(rand::random());
        let tag = 15001;
        let mapref = MapAddress::Unseq { name, tag };

        let client = random_client()?;
        let data = UnseqMap::new_with_data(
            name,
            tag,
            Default::default(),
            Default::default(),
            client.public_key().await,
        );

        client.put_unseq_mutable_data(data).await?;

        let client = random_client()?;
        let res = client.delete_map(mapref).await;
        match res {
            Err(CoreError::DataError(SndError::AccessDenied)) => (),
            res => panic!("Unexpected result: {:?}", res),
        }

        Ok(())
    }

    #[tokio::test]
    pub async fn map_cannot_initially_put_data_with_another_owner_than_current_client(
    ) -> Result<(), CoreError> {
        let client = random_client()?;
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new()
            .allow(MapAction::Read)
            .allow(MapAction::Insert)
            .allow(MapAction::ManagePermissions);
        let user = client.public_key().await;
        let random_user = gen_bls_keypair().public_key();
        let random_pk = gen_bls_keypair().public_key();

        let _ = permissions.insert(user, permission_set.clone());
        let _ = permissions.insert(random_user, permission_set);

        let test_data_name = XorName(rand::random());
        let test_data_with_different_owner_than_client = SeqMap::new_with_data(
            test_data_name.clone(),
            15000,
            Default::default(),
            permissions,
            random_pk,
        );

        client
            .put_seq_mutable_data(test_data_with_different_owner_than_client.clone())
            .await?;
        let res = client.get_seq_map_shell(test_data_name, 1500).await;
        match res {
            Err(CoreError::DataError(SndError::NoSuchData)) => (),
            Ok(_) => panic!("Unexpected Success: Validating owners should fail"),
            Err(e) => panic!("Unexpected: {:?}", e),
        };

        // TODO: Refunds not yet in place.... Reenable this check when that's the case

        // Check money was not taken
        // let balance = client.get_balance(None).await?;
        // let expected_bal = calculate_new_balance(start_bal, Some(2), None);
        // assert_eq!(balance, expected_bal);

        Ok(())
    }

    // 1. Create a mutable data with some permissions and store it on the network.
    // 2. Modify the permissions of a user in the permission set.
    // 3. Fetch the list of permissions and verify the edit.
    // 4. Delete a user's permissions from the permission set and verify the deletion.
    #[tokio::test]
    pub async fn map_can_modify_permissions_test() -> Result<(), CoreError> {
        let client = random_client()?;
        let name = XorName(rand::random());
        let tag = 15001;
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new()
            .allow(MapAction::Read)
            .allow(MapAction::Insert)
            .allow(MapAction::ManagePermissions);
        let user = client.public_key().await;
        let random_user = gen_bls_keypair().public_key();

        let _ = permissions.insert(user, permission_set.clone());
        let _ = permissions.insert(random_user, permission_set);

        let data = SeqMap::new_with_data(
            name,
            tag,
            Default::default(),
            permissions.clone(),
            client.public_key().await,
        );

        client.put_seq_mutable_data(data).await?;

        let new_perm_set = MapPermissionSet::new()
            .allow(MapAction::ManagePermissions)
            .allow(MapAction::Read);
        client
            .set_map_user_permissions(MapAddress::Seq { name, tag }, user, new_perm_set, 1)
            .await?;
        println!("Modified user permissions");

        let permissions = client
            .list_map_user_permissions(MapAddress::Seq { name, tag }, user)
            .await?;
        assert!(!permissions.is_allowed(MapAction::Insert));
        assert!(permissions.is_allowed(MapAction::Read));
        assert!(permissions.is_allowed(MapAction::ManagePermissions));
        println!("Verified new permissions");

        client
            .del_map_user_permissions(MapAddress::Seq { name, tag }, random_user, 2)
            .await?;
        println!("Deleted permissions");
        let permissions = client
            .list_map_permissions(MapAddress::Seq { name, tag })
            .await?;
        assert_eq!(permissions.len(), 1);
        println!("Permission set verified");

        Ok(())
    }

    // 1. Create a mutable data and store it on the network
    // 2. Create some entry actions and mutate the data on the network.
    // 3. List the entries and verify that the mutation was applied.
    // 4. Fetch a value for a particular key and verify
    #[tokio::test]
    pub async fn map_mutations_test() -> Result<(), CoreError> {
        let client = random_client()?;

        let name = XorName(rand::random());
        let tag = 15001;
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new()
            .allow(MapAction::Read)
            .allow(MapAction::Insert)
            .allow(MapAction::Update)
            .allow(MapAction::Delete);
        let user = client.public_key().await;
        let _ = permissions.insert(user, permission_set);
        let mut entries: MapSeqEntries = Default::default();
        let _ = entries.insert(
            b"key1".to_vec(),
            MapSeqValue {
                data: b"value".to_vec(),
                version: 0,
            },
        );
        let _ = entries.insert(
            b"key2".to_vec(),
            MapSeqValue {
                data: b"value".to_vec(),
                version: 0,
            },
        );
        let data = SeqMap::new_with_data(
            name,
            tag,
            entries.clone(),
            permissions,
            client.public_key().await,
        );
        client.put_seq_mutable_data(data).await?;

        let fetched_entries = client.list_seq_map_entries(name, tag).await?;

        assert_eq!(fetched_entries, entries);
        let entry_actions: MapSeqEntryActions = MapSeqEntryActions::new()
            .update(b"key1".to_vec(), b"newValue".to_vec(), 1)
            .del(b"key2".to_vec(), 1)
            .ins(b"key3".to_vec(), b"value".to_vec(), 0);

        client
            .mutate_seq_map_entries(name, tag, entry_actions)
            .await?;

        let fetched_entries = client.list_seq_map_entries(name, tag).await?;
        let mut expected_entries: BTreeMap<_, _> = Default::default();
        let _ = expected_entries.insert(
            b"key1".to_vec(),
            MapSeqValue {
                data: b"newValue".to_vec(),
                version: 1,
            },
        );
        let _ = expected_entries.insert(
            b"key3".to_vec(),
            MapSeqValue {
                data: b"value".to_vec(),
                version: 0,
            },
        );

        assert_eq!(fetched_entries, expected_entries);

        let fetched_value = client
            .get_seq_map_value(name, tag, b"key3".to_vec())
            .await?;

        assert_eq!(
            fetched_value,
            MapSeqValue {
                data: b"value".to_vec(),
                version: 0
            }
        );

        let res = client
            .get_seq_map_value(name, tag, b"wrongKey".to_vec())
            .await;
        match res {
            Ok(_) => panic!("Unexpected: Entry should not exist"),
            Err(CoreError::DataError(SndError::NoSuchEntry)) => (),
            Err(err) => panic!("Unexpected error: {:?}", err),
        };

        let client = random_client()?;
        let name = XorName(rand::random());
        let tag = 15001;
        let mut permissions: BTreeMap<_, _> = Default::default();
        let permission_set = MapPermissionSet::new()
            .allow(MapAction::Read)
            .allow(MapAction::Insert)
            .allow(MapAction::Update)
            .allow(MapAction::Delete);
        let user = client.public_key().await;
        let _ = permissions.insert(user, permission_set);
        let mut entries: BTreeMap<Vec<u8>, Vec<u8>> = Default::default();
        let _ = entries.insert(b"key1".to_vec(), b"value".to_vec());
        let _ = entries.insert(b"key2".to_vec(), b"value".to_vec());
        let data = UnseqMap::new_with_data(
            name,
            tag,
            entries.clone(),
            permissions,
            client.public_key().await,
        );
        client.put_unseq_mutable_data(data).await?;
        println!("Put unseq. Map successfully");

        let fetched_entries = client.list_unseq_map_entries(name, tag).await?;
        assert_eq!(fetched_entries, entries);
        let entry_actions: MapUnseqEntryActions = MapUnseqEntryActions::new()
            .update(b"key1".to_vec(), b"newValue".to_vec())
            .del(b"key2".to_vec())
            .ins(b"key3".to_vec(), b"value".to_vec());

        client
            .mutate_unseq_map_entries(name, tag, entry_actions)
            .await?;
        let fetched_entries = client.list_unseq_map_entries(name, tag).await?;
        let mut expected_entries: BTreeMap<_, _> = Default::default();
        let _ = expected_entries.insert(b"key1".to_vec(), b"newValue".to_vec());
        let _ = expected_entries.insert(b"key3".to_vec(), b"value".to_vec());
        assert_eq!(fetched_entries, expected_entries);
        let fetched_value = client
            .get_unseq_map_value(name, tag, b"key1".to_vec())
            .await?;
        assert_eq!(fetched_value, b"newValue".to_vec());
        let res = client
            .get_unseq_map_value(name, tag, b"wrongKey".to_vec())
            .await;
        match res {
            Ok(_) => panic!("Unexpected: Entry should not exist"),
            Err(CoreError::DataError(SndError::NoSuchEntry)) => Ok(()),
            Err(err) => panic!("Unexpected error: {:?}", err),
        }
    }

    // // 1. Create a random BLS key and create a wallet for it with some test safecoin.
    // // 2. Without a client object, try to get the balance, create new wallets and transfer safecoin.
    // #[tokio::test]
    // pub async fn wallet_transactions_without_client() -> Result<(), CoreError> {
    //     let client_id = gen_client_id();

    //     test_create_balance(&client_id, unwrap!(Coins::from_str("50"))).await?;

    //     let balance = wallet_get_balance(&client_id).await?;
    //     let ten_coins = unwrap!(Coins::from_str("10"));
    //     assert_eq!(balance, unwrap!(Coins::from_str("50")));

    //     let new_client_id = gen_client_id();
    //     let new_client_pk = new_client_id.public_id().public_key();
    //     let new_wallet: XorName = *new_client_id.public_id().name();
    //     let txn = wallet_create_balance(&client_id, *new_client_pk, ten_coins, None).await?;
    //     assert_eq!(txn.amount, ten_coins);
    //     let txn2 = wallet_transfer_coins(&client_id, new_wallet, ten_coins, None).await?;
    //     assert_eq!(txn2.amount, ten_coins);

    //     let client_balance = wallet_get_balance(&client_id).await?;
    //     let expected = unwrap!(Coins::from_str("30"));
    //     let expected = unwrap!(expected.checked_sub(COST_OF_PUT));
    //     assert_eq!(client_balance, expected);

    //     let new_client_balance = wallet_get_balance(&new_client_id).await?;
    //     assert_eq!(new_client_balance, unwrap!(Coins::from_str("20")));

    //     Ok(())
    // }

    #[tokio::test]
    pub async fn blob_deletions_should_cost_put_price() -> Result<(), CoreError> {
        let client = random_client()?;

        let blob = PrivateBlob::new(generate_random_vector::<u8>(10), client.public_key().await);
        let blob_address = *blob.name();
        client.put_blob(blob).await?;

        let balance_before_delete = client.get_balance(None).await?;
        client.del_unpub_blob(blob_address).await?;
        let new_balance = client.get_balance(None).await?;

        // make sure we have _some_ balance
        assert_ne!(balance_before_delete, Money::from_str("0")?);
        assert_ne!(balance_before_delete, new_balance);

        Ok(())
    }

    #[tokio::test]
    pub async fn map_deletions_should_cost_put_price() -> Result<(), CoreError> {
        let name = XorName(rand::random());
        let tag = 10;
        let client = random_client()?;

        let map = UnseqMap::new(name, tag, client.public_key().await);
        client.put_unseq_mutable_data(map).await?;

        let map_address = MapAddress::from_kind(MapKind::Unseq, name, tag);

        let balance_before_delete = client.get_balance(None).await?;
        client.delete_map(map_address).await?;
        let new_balance = client.get_balance(None).await?;

        // make sure we have _some_ balance
        assert_ne!(balance_before_delete, Money::from_str("0")?);
        assert_ne!(balance_before_delete, new_balance);

        Ok(())
    }

    #[tokio::test]
    pub async fn sequence_deletions_should_cost_put_price() -> Result<(), CoreError> {
        let name = XorName(rand::random());
        let tag = 10;
        let client = random_client()?;
        let owner = client.public_key().await;
        let perms = BTreeMap::<PublicKey, SequencePrivUserPermissions>::new();
        let sequence_address = client
            .store_private_sequence(name, tag, owner, perms)
            .await?;

        let balance_before_delete = client.get_balance(None).await?;
        client.delete_sequence(sequence_address).await?;
        let new_balance = client.get_balance(None).await?;

        // make sure we have _some_ balance
        assert_ne!(balance_before_delete, Money::from_str("0")?);
        assert_ne!(balance_before_delete, new_balance);

        Ok(())
    }

    /// Sequence data tests ///

    #[tokio::test]
    pub async fn sequence_basics_test() -> Result<(), CoreError> {
        let client = random_client()?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key().await;

        // store a Private Sequence
        let mut perms = BTreeMap::<PublicKey, SequencePrivUserPermissions>::new();
        let _ = perms.insert(owner, SequencePrivUserPermissions::new(true, true, true));
        let address = client
            .store_private_sequence(name, tag, owner, perms)
            .await?;
        let sequence = client.get_sequence(address).await?;
        assert!(sequence.is_private());
        assert_eq!(*sequence.name(), name);
        assert_eq!(sequence.tag(), tag);
        assert_eq!(sequence.permissions_index(), 1);
        assert_eq!(sequence.owners_index(), 1);
        assert_eq!(sequence.entries_index(), 0);

        // store a Public Sequence
        let mut perms = BTreeMap::<SequenceUser, SequencePubUserPermissions>::new();
        let _ = perms.insert(
            SequenceUser::Anyone,
            SequencePubUserPermissions::new(true, true),
        );
        let address = client.store_pub_sequence(name, tag, owner, perms).await?;
        let sequence = client.get_sequence(address).await?;
        assert!(sequence.is_pub());
        assert_eq!(*sequence.name(), name);
        assert_eq!(sequence.tag(), tag);
        assert_eq!(sequence.permissions_index(), 1);
        assert_eq!(sequence.owners_index(), 1);
        assert_eq!(sequence.entries_index(), 0);

        Ok(())
    }

    #[tokio::test]
    pub async fn sequence_private_permissions_test() -> Result<(), CoreError> {
        let client = random_client()?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key().await;
        let mut perms = BTreeMap::<PublicKey, SequencePrivUserPermissions>::new();
        let _ = perms.insert(owner, SequencePrivUserPermissions::new(true, true, true));
        let address = client
            .store_private_sequence(name, tag, owner, perms)
            .await?;

        let data = client.get_sequence(address).await?;
        assert_eq!(data.entries_index(), 0);
        assert_eq!(data.owners_index(), 1);
        assert_eq!(data.permissions_index(), 1);

        let private_permissions = client.get_sequence_private_permissions(address).await?;
        let user_perms = private_permissions
            .permissions
            .get(&owner)
            .ok_or_else(|| CoreError::from("Unexpectedly failed to get user permissions"))?;
        assert!(user_perms.is_allowed(SequenceAction::Read));
        assert!(user_perms.is_allowed(SequenceAction::Append));
        assert!(user_perms.is_allowed(SequenceAction::ManagePermissions));

        match client
            .get_sequence_user_permissions(address, SequenceUser::Key(owner))
            .await?
        {
            SequenceUserPermissions::Priv(user_perms) => {
                assert!(user_perms.is_allowed(SequenceAction::Read));
                assert!(user_perms.is_allowed(SequenceAction::Append));
                assert!(user_perms.is_allowed(SequenceAction::ManagePermissions));
            }
            SequenceUserPermissions::Public(_) => {
                return Err(CoreError::from(
                    "Unexpectedly obtained incorrect user permissions",
                ))
            }
        }

        let sim_client = gen_bls_keypair().public_key();
        let mut perms2 = BTreeMap::<PublicKey, SequencePrivUserPermissions>::new();
        let _ = perms2.insert(
            sim_client,
            SequencePrivUserPermissions::new(false, true, false),
        );
        client
            .sequence_set_private_permissions(address, perms2)
            .await?;

        let private_permissions = client.get_sequence_private_permissions(address).await?;
        let user_perms = private_permissions
            .permissions
            .get(&sim_client)
            .ok_or_else(|| CoreError::from("Unexpectedly failed to get user permissions"))?;
        assert!(!user_perms.is_allowed(SequenceAction::Read));
        assert!(user_perms.is_allowed(SequenceAction::Append));
        assert!(!user_perms.is_allowed(SequenceAction::ManagePermissions));

        match client
            .get_sequence_user_permissions(address, SequenceUser::Key(sim_client))
            .await?
        {
            SequenceUserPermissions::Priv(user_perms) => {
                assert!(!user_perms.is_allowed(SequenceAction::Read));
                assert!(user_perms.is_allowed(SequenceAction::Append));
                assert!(!user_perms.is_allowed(SequenceAction::ManagePermissions));
                Ok(())
            }
            SequenceUserPermissions::Public(_) => Err(CoreError::from(
                "Unexpectedly obtained incorrect user permissions",
            )),
        }
    }

    #[tokio::test]
    pub async fn sequence_pub_permissions_test() -> Result<(), CoreError> {
        let client = random_client()?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key().await;
        let mut perms = BTreeMap::<SequenceUser, SequencePubUserPermissions>::new();
        let _ = perms.insert(
            SequenceUser::Key(owner),
            SequencePubUserPermissions::new(None, true),
        );
        let address = client.store_pub_sequence(name, tag, owner, perms).await?;

        let data = client.get_sequence(address).await?;
        assert_eq!(data.entries_index(), 0);
        assert_eq!(data.owners_index(), 1);
        assert_eq!(data.permissions_index(), 1);

        let pub_permissions = client.get_sequence_pub_permissions(address).await?;
        let user_perms = pub_permissions
            .permissions
            .get(&SequenceUser::Key(owner))
            .ok_or_else(|| CoreError::from("Unexpectedly failed to get user permissions"))?;
        assert_eq!(Some(true), user_perms.is_allowed(SequenceAction::Read));
        assert_eq!(None, user_perms.is_allowed(SequenceAction::Append));
        assert_eq!(
            Some(true),
            user_perms.is_allowed(SequenceAction::ManagePermissions)
        );

        match client
            .get_sequence_user_permissions(address, SequenceUser::Key(owner))
            .await?
        {
            SequenceUserPermissions::Public(user_perms) => {
                assert_eq!(Some(true), user_perms.is_allowed(SequenceAction::Read));
                assert_eq!(None, user_perms.is_allowed(SequenceAction::Append));
                assert_eq!(
                    Some(true),
                    user_perms.is_allowed(SequenceAction::ManagePermissions)
                );
            }
            SequenceUserPermissions::Priv(_) => {
                return Err(CoreError::from(
                    "Unexpectedly obtained incorrect user permissions",
                ))
            }
        }

        let sim_client = gen_bls_keypair().public_key();
        let mut perms2 = BTreeMap::<SequenceUser, SequencePubUserPermissions>::new();
        let _ = perms2.insert(
            SequenceUser::Key(sim_client),
            SequencePubUserPermissions::new(false, false),
        );
        client.sequence_set_pub_permissions(address, perms2).await?;

        let pub_permissions = client.get_sequence_pub_permissions(address).await?;
        let user_perms = pub_permissions
            .permissions
            .get(&SequenceUser::Key(sim_client))
            .ok_or_else(|| CoreError::from("Unexpectedly failed to get user permissions"))?;
        assert_eq!(Some(true), user_perms.is_allowed(SequenceAction::Read));
        assert_eq!(Some(false), user_perms.is_allowed(SequenceAction::Append));
        assert_eq!(
            Some(false),
            user_perms.is_allowed(SequenceAction::ManagePermissions)
        );

        match client
            .get_sequence_user_permissions(address, SequenceUser::Key(sim_client))
            .await?
        {
            SequenceUserPermissions::Public(user_perms) => {
                assert_eq!(Some(true), user_perms.is_allowed(SequenceAction::Read));
                assert_eq!(Some(false), user_perms.is_allowed(SequenceAction::Append));
                assert_eq!(
                    Some(false),
                    user_perms.is_allowed(SequenceAction::ManagePermissions)
                );
                Ok(())
            }
            SequenceUserPermissions::Priv(_) => Err(CoreError::from(
                "Unexpectedly obtained incorrect user permissions",
            )),
        }
    }

    #[tokio::test]
    pub async fn sequence_append_test() -> Result<(), CoreError> {
        let name = XorName(rand::random());
        let tag = 10;
        let client = random_client()?;

        let owner = client.public_key().await;
        let mut perms = BTreeMap::<SequenceUser, SequencePubUserPermissions>::new();
        let _ = perms.insert(
            SequenceUser::Key(owner),
            SequencePubUserPermissions::new(true, true),
        );
        let address = client.store_pub_sequence(name, tag, owner, perms).await?;

        client.sequence_append(address, b"VALUE1".to_vec()).await?;

        let (index, data) = client.get_sequence_last_entry(address).await?;
        assert_eq!(0, index);
        assert_eq!(unwrap!(std::str::from_utf8(&data)), "VALUE1");

        client.sequence_append(address, b"VALUE2".to_vec()).await?;

        let (index, data) = client.get_sequence_last_entry(address).await?;
        assert_eq!(1, index);
        assert_eq!(unwrap!(std::str::from_utf8(&data)), "VALUE2");

        let data = client
            .get_sequence_range(
                address,
                (SequenceIndex::FromStart(0), SequenceIndex::FromEnd(0)),
            )
            .await?;
        assert_eq!(unwrap!(std::str::from_utf8(&data[0])), "VALUE1");
        assert_eq!(unwrap!(std::str::from_utf8(&data[1])), "VALUE2");

        Ok(())
    }

    #[tokio::test]
    pub async fn sequence_owner_test() -> Result<(), CoreError> {
        let name = XorName(rand::random());
        let tag = 10;
        let client = random_client()?;

        let owner = client.public_key().await;
        let mut perms = BTreeMap::<PublicKey, SequencePrivUserPermissions>::new();
        let _ = perms.insert(owner, SequencePrivUserPermissions::new(true, true, true));
        let address = client
            .store_private_sequence(name, tag, owner, perms)
            .await?;

        client.sequence_append(address, b"VALUE1".to_vec()).await?;
        client.sequence_append(address, b"VALUE2".to_vec()).await?;

        let data = client.get_sequence(address).await?;
        assert_eq!(data.entries_index(), 2);
        assert_eq!(data.owners_index(), 1);
        assert_eq!(data.permissions_index(), 1);

        let current_owner = client.get_sequence_owner(address).await?;
        assert_eq!(owner, current_owner.public_key);

        let sim_client = gen_bls_keypair().public_key();
        client.sequence_set_owner(address, sim_client).await?;

        let current_owner = client.get_sequence_owner(address).await?;
        assert_eq!(sim_client, current_owner.public_key);

        Ok(())
    }

    #[tokio::test]
    pub async fn sequence_can_delete_private_test() -> Result<(), CoreError> {
        let client = random_client()?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key().await;

        // store a Private Sequence
        let mut perms = BTreeMap::<PublicKey, SequencePrivUserPermissions>::new();
        let _ = perms.insert(owner, SequencePrivUserPermissions::new(true, true, true));
        let address = client
            .store_private_sequence(name, tag, owner, perms)
            .await?;
        let sequence = client.get_sequence(address).await?;
        assert!(sequence.is_private());

        client.delete_sequence(address).await?;

        match client.get_sequence(address).await {
            Err(CoreError::DataError(SndError::NoSuchData)) => Ok(()),
            Err(err) => {
                return Err(CoreError::from(format!(
                    "Unexpected error returned when deleting a nonexisting Private Sequence: {}",
                    err
                )))
            }
            Ok(_res) => {
                return Err(CoreError::from(
                    "Unexpectedly retrieved a deleted Private Sequence!",
                ))
            }
        }
    }

    #[tokio::test]
    pub async fn sequence_cannot_delete_public_test() -> Result<(), CoreError> {
        let client = random_client()?;

        let name = XorName(rand::random());
        let tag = 15000;
        let owner = client.public_key().await;

        // store a Public Sequence
        let mut perms = BTreeMap::<SequenceUser, SequencePubUserPermissions>::new();
        let _ = perms.insert(
            SequenceUser::Anyone,
            SequencePubUserPermissions::new(true, true),
        );
        let address = client.store_pub_sequence(name, tag, owner, perms).await?;
        let sequence = client.get_sequence(address).await?;
        assert!(sequence.is_pub());

        client.delete_sequence(address).await?;

        // Check that our data still exists.
        match client.get_sequence(address).await {
            Err(CoreError::DataError(SndError::InvalidOperation)) => Ok(()),
            Err(err) => {
                return Err(CoreError::from(format!(
                    "Unexpected error returned when attempting to get a Public Sequence: {}",
                    err
                )))
            }
            Ok(_data) => Ok(()),
        }
    }
}

/*
fn wrap_blob_read(read: BlobRead) -> Query {
    Query::Data(DataQuery::Blob(read))
}

fn wrap_account_read(read: AccountRead) -> Query {
    Query::Data(DataQuery::Account(read))
}

fn wrap_client_auth_cmd(auth_cmd: AuthCmd) -> Cmd {
    Cmd::Auth(auth_cmd)
}
*/

fn wrap_map_read(read: MapRead) -> Query {
    Query::Data(DataQuery::Map(read))
}

fn wrap_seq_read(read: SequenceRead) -> Query {
    Query::Data(DataQuery::Sequence(read))
}

fn wrap_client_auth_query(auth_query: AuthQuery) -> Query {
    Query::Auth(auth_query)
}

// 1. Store different variants of unpublished data on the network.
// 2. Get the balance of the client.
// 3. Delete data from the network.
// 4. Verify that the balance has not changed since deletions are free.
// #[tokio::test]
// pub async fn deletions_should_be_free() -> Result<(), CoreError> {
//     let name = XorName(rand::random());
//     let tag = 10;
//     let client = random_client()?;

//     let blob = PrivateBlob::new(
//         unwrap!(generate_random_vector::<u8>(10)),
//         client.public_key().await,
//     );
//     let address = *blob.name();
//     client.put_blob(blob).await?;
//     let mut adata = PrivateSeqAppendOnlyData::new(name, tag);
//     let owner = ADataOwner {
//         public_key: client.public_key().await,
//         entries_index: 0,
//         permissions_index: 0,
//     };
//     unwrap!(adata.append_owner(owner, 0));
//     client.put_adata(adata.into()).await?;
//     let map = UnseqMap::new(name, tag, client.public_key().await);
//     client.put_unseq_mutable_data(map).await?;

// /// Insert a given login packet at the specified destination
// async fn insert_login_packet_for(
//     &self,
//     // client_id: Option<&ClientFullId>,
//     new_owner: PublicKey,
//     amount: Money,
//     new_login_packet: Account,
// ) -> Result<TransferRegistered, CoreError>
// where
//     Self: Sized,
// {
//     trace!(
//         "Insert a login packet for {:?} preloading the wallet with {} money.",
//         new_owner,
//         amount
//     );

//     let mut actor = self
//         .transfer_actor()
//         .await
//         .ok_or(CoreError::from("No TransferActor found for client."))?;

//     let response = actor
//         .create_login_for(new_owner, amount, new_login_packet)
//         .await;

//     match response {
//         Ok(res) => match res {
//             QueryResponse::TransferRegistration(result) => match result {
//                 Ok(transfer) => Ok(transfer),
//                 Err(error) => Err(CoreError::from(error)),
//             },
//             _ => Err(CoreError::ReceivedUnexpectedEvent),
//         },

//         Err(error) => Err(error),
//     }
// }
