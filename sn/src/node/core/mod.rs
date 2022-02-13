// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod api;
mod bootstrap;
mod comm;
mod connectivity;
mod data;
mod delivery_group;
mod messaging;
mod proposal;
mod relocation;
mod split_barrier;

pub(crate) use bootstrap::{join_network, JoiningAsRelocated};
pub(crate) use comm::{Comm, ConnectionEvent, SendStatus};
pub(crate) use data::MIN_LEVEL_WHEN_FULL;
pub(crate) use proposal::Proposal;
#[cfg(test)]
pub(crate) use relocation::{check as relocation_check, ChurnId};

use super::{
    api::cmds::Cmd,
    dkg::DkgVoter,
    error::Result,
    network_knowledge::{NetworkKnowledge, SectionKeyShare, SectionKeysProvider},
    Elders, Event, NodeElderChange, NodeInfo,
};

use crate::messaging::{
    signature_aggregator::SignatureAggregator,
    system::{DkgSessionId, SystemMsg},
    AuthorityProof, SectionAuth,
};
use crate::types::{log_markers::LogMarker, utils::compare_and_write_prefix_map_to_disk, Peer};
use crate::UsedSpace;

use backoff::ExponentialBackoff;
use data::Data;
use itertools::Itertools;
use messaging::MsgSender;
use resource_proof::ResourceProof;
use split_barrier::SplitBarrier;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc, RwLock, Semaphore};
use uluru::LRUCache;
use xor_name::{Prefix, XorName};

pub(super) const RESOURCE_PROOF_DATA_SIZE: usize = 128;
pub(super) const RESOURCE_PROOF_DIFFICULTY: u8 = 10;

pub(crate) const CONCURRENT_JOINS: usize = 7;
// This prevents pending query limit unbound growth
pub(crate) const DATA_QUERY_LIMIT: usize = 100;
// per query we can have this many peers, so the total peers waiting can be QUERY_LIMIT * MAX_WAITING_PEERS_PER_QUERY
pub(crate) const MAX_WAITING_PEERS_PER_QUERY: usize = 100;

const BACKOFF_CACHE_LIMIT: usize = 100;

// How long to hold on to correlated `Peer`s for data queries. Since data queries are forwarded
// from elders (with whom the client is connected) to adults (who hold the data), the elder handling
// the query cannot reply immediately. For now, they stash a reference to the client `Peer` in
// `Core::pending_data_queries`, which is a cache with duration-based expiry.
// TODO: The value chosen here is longer than the default client timeout (see
// `crate::client::SN_CLIENT_QUERY_TIMEOUT`), but the timeout is configurable. Ideally this would be
// based on liveness properties (e.g. the timeout should be dynamic based on the responsiveness of
// the section).
const DATA_QUERY_TIMEOUT: Duration = Duration::from_secs(60 * 5 /* 5 mins */);

// Store up to 100 in use backoffs
pub(crate) type AeBackoffCache =
    Arc<RwLock<LRUCache<(Peer, ExponentialBackoff), BACKOFF_CACHE_LIMIT>>>;

#[derive(Clone)]
pub(crate) struct DkgSessionInfo {
    pub(crate) prefix: Prefix,
    pub(crate) elders: BTreeMap<XorName, SocketAddr>,
    pub(crate) authority: AuthorityProof<SectionAuth>,
}

// Core state + logic of a node.
pub(crate) struct Node {
    pub(super) event_tx: mpsc::Sender<Event>,
    pub(crate) info: Arc<RwLock<NodeInfo>>,

    pub(crate) comm: Comm,
    pub(crate) section_keys_provider: SectionKeysProvider,

    data: Data,
    resource_proof: ResourceProof,
    network_knowledge: NetworkKnowledge,
    message_aggregator: SignatureAggregator,

    proposal_aggregator: SignatureAggregator,
    split_barrier: Arc<RwLock<SplitBarrier>>,
    dkg_sessions: Arc<RwLock<HashMap<DkgSessionId, DkgSessionInfo>>>,
    dkg_voter: DkgVoter,

    relocate_state: Arc<RwLock<Option<Box<JoiningAsRelocated>>>>,

    joins_allowed: Arc<RwLock<bool>>,        // Elder only
    current_joins_semaphore: Arc<Semaphore>, // Elder only

    ae_backoff_cache: AeBackoffCache,

    msg_sender: MsgSender,
}

impl Node {
    // Creates `Core` for a regular node.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn new(
        comm: Comm,
        mut info: NodeInfo,
        network_knowledge: NetworkKnowledge,
        section_key_share: Option<SectionKeyShare>,
        event_tx: mpsc::Sender<Event>,
        used_space: UsedSpace,
        root_storage_dir: PathBuf,
    ) -> Result<Self> {
        let section_keys_provider = SectionKeysProvider::new(section_key_share).await;

        // make sure the Node has the correct local addr as Comm
        info.addr = comm.our_connection_info();

        let info = Arc::new(RwLock::new(info));

        let data = Data::new(
            &root_storage_dir,
            used_space.clone(),
            network_knowledge.clone(),
            info.clone(),
        )
        .await?;

        let msg_sender = MsgSender::new(network_knowledge.clone(), info.clone()).await;

        Ok(Self {
            event_tx,
            info,
            comm,
            section_keys_provider,
            data,
            network_knowledge,
            dkg_sessions: Arc::new(RwLock::new(HashMap::default())),
            proposal_aggregator: SignatureAggregator::default(),
            split_barrier: Arc::new(RwLock::new(SplitBarrier::new())),
            message_aggregator: SignatureAggregator::default(),
            dkg_voter: DkgVoter::default(),
            relocate_state: Arc::new(RwLock::new(None)),
            joins_allowed: Arc::new(RwLock::new(true)),
            current_joins_semaphore: Arc::new(Semaphore::new(CONCURRENT_JOINS)),
            resource_proof: ResourceProof::new(RESOURCE_PROOF_DATA_SIZE, RESOURCE_PROOF_DIFFICULTY),
            ae_backoff_cache: AeBackoffCache::default(),
            msg_sender,
        })
    }

    ////////////////////////////////////////////////////////////////////////////
    // Miscellaneous
    ////////////////////////////////////////////////////////////////////////////

    pub(crate) async fn generate_probe_msg(&self) -> Result<Cmd> {
        // Generate a random address not belonging to our Prefix
        let mut dst = XorName::random();

        // We don't probe ourselves
        while self.network_knowledge.prefix().await.matches(&dst) {
            dst = XorName::random();
        }

        let matching_section = self.network_knowledge.section_by_name(&dst)?;

        let message = SystemMsg::AntiEntropyProbe(dst);
        let section_key = matching_section.section_key();
        let dst_name = matching_section.prefix().name();
        let recipients = matching_section.elders_vec();

        info!(
            "ProbeMsg target {:?} w/key {:?}",
            matching_section.prefix(),
            section_key
        );

        self.send_direct_msg_to_nodes(recipients, message, dst_name, section_key)
            .await
    }

    pub(crate) async fn write_prefix_map(&self) {
        info!("Writing our latest PrefixMap to disk");
        // TODO: Make this serialization human readable

        let prefix_map = self.network_knowledge.prefix_map().clone();

        let _ = tokio::spawn(async move {
            // Compare and write Prefix to `~/.safe/prefix_maps` dir
            if let Err(e) = compare_and_write_prefix_map_to_disk(&prefix_map).await {
                error!("Error writing PrefixMap to `~/.safe` dir: {:?}", e);
            }
        });
    }

    pub(super) async fn state_snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            is_elder: self.is_elder().await,
            section_key: self.network_knowledge.section_key().await,
            prefix: self.network_knowledge.prefix().await,
            elders: self.network_knowledge().authority_provider().await.names(),
        }
    }

    /// Generate cmds and fire events based upon any node state changes.
    pub(super) async fn update_self_for_new_node_state(
        &self,
        old: StateSnapshot,
    ) -> Result<Vec<Cmd>> {
        let mut cmds = vec![];
        let new = self.state_snapshot().await;

        if new.section_key != old.section_key {
            if new.is_elder {
                let sap = self.network_knowledge.authority_provider().await;
                info!(
                    "Section updated: prefix: ({:b}), key: {:?}, elders: {}",
                    new.prefix,
                    new.section_key,
                    sap.elders().format(", ")
                );

                if !self.section_keys_provider.is_empty().await {
                    cmds.extend(self.promote_and_demote_elders().await?);

                    // Whenever there is an elders change, casting a round of joins_allowed
                    // proposals to sync.
                    cmds.extend(
                        self.propose(Proposal::JoinsAllowed(*self.joins_allowed.read().await))
                            .await?,
                    );
                }

                self.print_network_stats().await;
            }

            if new.is_elder || old.is_elder {
                cmds.extend(self.send_ae_update_to_our_section().await);
            }

            let current: BTreeSet<_> = self.network_knowledge.authority_provider().await.names();
            let added = current.difference(&old.elders).copied().collect();
            let removed = old.elders.difference(&current).copied().collect();
            let remaining = old.elders.intersection(&current).copied().collect();

            let elders = Elders {
                prefix: new.prefix,
                key: new.section_key,
                remaining,
                added,
                removed,
            };

            let self_status_change = if !old.is_elder && new.is_elder {
                trace!("{}: {:?}", LogMarker::PromotedToElder, new.prefix);
                NodeElderChange::Promoted
            } else if old.is_elder && !new.is_elder {
                trace!("{}", LogMarker::DemotedFromElder);
                info!("Demoted");
                self.section_keys_provider.wipe().await;
                NodeElderChange::Demoted
            } else {
                NodeElderChange::None
            };

            // During the split, sibling's SAP could be unknown to us yet.
            // Hence, fire the SectionSplit event whenever detect a prefix change.
            // We also need to update other nodes w/ our known data.
            let event = if (new.prefix != old.prefix) && new.is_elder {
                info!("{}: {:?}", LogMarker::SplitSuccess, new.prefix);

                if old.is_elder {
                    info!("{}: {:?}", LogMarker::StillElderAfterSplit, new.prefix);
                }

                cmds.extend(self.send_updates_to_sibling_section(&old).await?);
                self.data.update_member_tracking().await?;

                Event::SectionSplit {
                    elders,
                    self_status_change,
                }
            } else {
                cmds.extend(
                    self.send_metadata_updates_to_nodes(
                        self.network_knowledge
                            .authority_provider()
                            .await
                            .elders_vec(),
                        &self.network_knowledge.prefix().await,
                        new.section_key,
                    )
                    .await?,
                );

                Event::EldersChanged {
                    elders,
                    self_status_change,
                }
            };

            cmds.extend(
                self.send_metadata_updates_to_nodes(
                    self.network_knowledge
                        .authority_provider()
                        .await
                        .elders()
                        .filter(|peer| !old.elders.contains(&peer.name()))
                        .cloned()
                        .collect(),
                    &self.network_knowledge.prefix().await,
                    new.section_key,
                )
                .await?,
            );

            self.send_event(event).await
        }

        Ok(cmds)
    }

    pub(super) async fn print_network_stats(&self) {
        self.network_knowledge
            .prefix_map()
            .network_stats(&self.network_knowledge.authority_provider().await)
            .print();
        self.comm.print_stats();
    }

    pub(super) async fn log_section_stats(&self) {
        let adults = self.network_knowledge.adults().await.len();
        let elders = self
            .network_knowledge
            .authority_provider()
            .await
            .elder_count();
        let prefix = self.network_knowledge.prefix().await;
        debug!("{:?}: {:?} Elders, {:?} Adults.", prefix, elders, adults);
    }

    pub(super) async fn list_section_members(&self) -> BTreeSet<XorName> {
        self.network_knowledge
            .section_members()
            .await
            .iter()
            .map(|state| state.name())
            .collect()
    }
}

pub(crate) struct StateSnapshot {
    is_elder: bool,
    section_key: bls::PublicKey,
    prefix: Prefix,
    pub(crate) elders: BTreeSet<XorName>,
}
