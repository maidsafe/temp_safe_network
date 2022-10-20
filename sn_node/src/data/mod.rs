// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Implementation of the data storage for the SAFE Network.

mod replicator;
mod storage;

pub use self::storage::{ChunkStorage, DataStorage, RegisterStorage, UsedSpace};

pub(crate) use self::storage::{Error, Result};

use self::replicator::{ReplicationJob, ReplicationQueue};

#[cfg(feature = "traceroute")]
use sn_interface::messaging::Traceroute;

use sn_interface::{
    data_copy_count,
    messaging::{
        data::{DataQueryVariant, StorageLevel},
        system::NodeQueryResponse as QueryResponse,
        EndUser, MsgId,
    },
    types::{register::User, DataAddress, Peer, ReplicatedData},
};

use std::{collections::BTreeSet, path::Path, sync::Arc};
use xor_name::XorName;

/// The [`Data`] struct is the topmost structure in the
/// independent data module.
#[derive(Debug)]
pub struct Data {
    storage: DataStorage,
    replication: ReplicationQueue,
}

impl Data {
    /// Set up a new `DataStorage` instance
    pub fn new(path: &Path, used_space: UsedSpace) -> Result<Self> {
        Ok(Self {
            storage: DataStorage::new(path, used_space)?,
            replication: ReplicationQueue::new(),
        })
    }

    /// Retrieve all keys/DataAddresses of stored data
    pub(crate) async fn keys(&self) -> Vec<DataAddress> {
        self.storage.data_addrs().await
    }

    /// Api for querying the data storage.
    pub async fn query(&self, query: &DataQueryVariant, requester: User) -> QueryResponse {
        self.storage.query(query, requester).await
    }

    /// Api for writing to the data storage.
    pub async fn handle(&self, cmd: Cmd) -> Result<Option<Event>> {
        use Cmd::*;
        match cmd {
            Store {
                data,
                section_pk,
                node_keypair,
            } => {
                let e = match self.storage.store(&data, section_pk, node_keypair).await {
                    Ok(None) => return Ok(None),
                    Ok(Some(storage_level)) => Event::StorageLevelIncreased(storage_level),
                    Err(err) => {
                        let data = if matches!(err, Error::NotEnoughSpace) {
                            // when NotEnoughSpace we propagate the data as well
                            Some(data)
                        } else {
                            // the rest seem to be non-problematic errors.. (?)
                            None
                        };
                        Event::StorageFailed {
                            error: Arc::new(err),
                            data,
                        }
                    }
                };
                Ok(Some(e))
            }
            HandleQuery {
                query,
                auth,
                requesting_elder,
                correlation_id,
                #[cfg(feature = "traceroute")]
                traceroute,
            } => {
                let response = self.query(&query, User::Key(auth.public_key)).await;
                Ok(Some(Event::QueryResponseProduced {
                    response,
                    requesting_elder,
                    correlation_id,
                    user: EndUser(auth.public_key.into()),
                    #[cfg(feature = "traceroute")]
                    traceroute,
                }))
            }
            PopReplicationQueue => match self.replication.pop_random().await {
                Some(ReplicationJob {
                    data_address,
                    recipients,
                }) => {
                    let data = self.storage.get_from_local_store(&data_address).await?;
                    Ok(Some(Event::ReplicationQueuePopped { data, recipients }))
                }
                None => Ok(None),
            },
            EnqueueReplicationJob {
                peer,
                currently_at_peer,
                other_peers,
            } => {
                use itertools::Itertools;

                let mut complementing_data = vec![];
                let locally_held_data = self.storage.data_addrs().await;

                for data in locally_held_data {
                    if currently_at_peer.contains(&data) {
                        continue;
                    }

                    let holder_adult_list: BTreeSet<_> = other_peers
                        .iter()
                        .sorted_by(|lhs, rhs| data.name().cmp_distance(lhs, rhs))
                        .take(data_copy_count())
                        .collect();

                    if holder_adult_list.contains(&peer.name()) {
                        complementing_data.push(data);
                    }
                }

                if complementing_data.is_empty() {
                    trace!("We have no data worth sending");
                    return Ok(None);
                }

                self.replication.enqueue(peer, complementing_data);

                Ok(None)
            }
        }
    }
}

/// Cmds for interacting with the data storage module.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Cmd {
    /// Request to store data.
    Store {
        /// The data to be stored.
        data: ReplicatedData,
        /// Temp field while dbcs are integrated.
        section_pk: sn_interface::types::PublicKey,
        /// Temp field while dbcs are integrated.
        node_keypair: sn_interface::types::Keypair,
    },
    /// Request to serve back data held.
    HandleQuery {
        /// The query
        query: DataQueryVariant,
        /// Client signature
        auth: sn_interface::messaging::ClientAuth,
        /// A correlation id
        correlation_id: MsgId,
        /// The relaying elder to respond to.
        requesting_elder: Peer,
        #[cfg(feature = "traceroute")]
        /// Traceroute used for client.
        traceroute: Traceroute,
    },
    /// Requesting next item to be replicated.
    PopReplicationQueue,
    /// Requesting the eventual complementation of the specified peer's held data.
    EnqueueReplicationJob {
        /// The peer who might need complementation of what it holds.
        peer: Peer,
        /// The data this peer currently holds.
        currently_at_peer: Vec<DataAddress>,
        /// Other peers we know of.
        other_peers: Vec<XorName>,
    },
}

/// Events produced in the data module.
#[derive(Debug, Clone)]
pub enum Event {
    /// There was an error when trying to store data.
    StorageFailed {
        /// The error.
        error: Arc<Error>,
        /// The data that we tried to store.
        data: Option<ReplicatedData>,
    },
    /// A random item from the queue was popped.
    ReplicationQueuePopped {
        /// The data item.
        data: ReplicatedData,
        /// Peers to whom the data should be replicated.
        recipients: BTreeSet<Peer>,
    },
    /// Used space for storage increased.
    StorageLevelIncreased(StorageLevel),
    /// A response to a query was produced.
    QueryResponseProduced {
        /// The response.
        response: QueryResponse,
        /// The elder from whom we received the query, and to whom we will return the response.
        requesting_elder: Peer,
        /// A correlation id,
        correlation_id: MsgId,
        /// The user where the query originated.
        user: EndUser,
        #[cfg(feature = "traceroute")]
        /// Traceroute used for client.
        traceroute: Traceroute,
    },
}
