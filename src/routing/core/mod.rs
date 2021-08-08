// Copyright 2021 MaidSafe.net limited.
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
mod delivery_group;
mod enduser_registry;
mod messaging;
mod msg_handling;
mod signature_aggregator;
mod split_barrier;

use crate::{
    dbs::UsedSpace,
    node::RegisterStorage,
    types::{CFOption, CFValue},
};
pub(crate) use bootstrap::{join_network, JoiningAsRelocated};
pub(crate) use comm::{Comm, ConnectionEvent, SendStatus};
pub use signature_aggregator::Error as AggregatorError;
pub(crate) use signature_aggregator::SignatureAggregator;
use std::path::PathBuf;

use self::{enduser_registry::EndUserRegistry, split_barrier::SplitBarrier};
use crate::messaging::{
    node::{NodeMsg, Proposal},
    MessageId,
};
use crate::routing::{
    dkg::{DkgVoter, ProposalAggregator},
    error::Result,
    network::NetworkLogic,
    node::Node,
    peer::PeerUtils,
    relocation::RelocationStatus,
    routing_api::command::Command,
    section::{Section, SectionKeysProvider, SectionLogic},
    Elders, Event, NodeElderChange, SectionAuthorityProviderUtils,
};
use itertools::Itertools;
use resource_proof::ResourceProof;
use std::collections::BTreeSet;
use tokio::sync::{mpsc, RwLock};
use xor_name::{Prefix, XorName};

use super::{dkg::SectionDkgOutcome, network::Network};

pub(super) const RESOURCE_PROOF_DATA_SIZE: usize = 64;
pub(super) const RESOURCE_PROOF_DIFFICULTY: u8 = 2;
const KEY_CACHE_SIZE: u8 = 5;

// State + logic of a routing node.
pub(crate) struct Core {
    pub(crate) comm: Comm,
    node: Node,
    section: Section,
    network: CFValue<Network>,
    section_keys: CFValue<SectionKeysProvider>,
    message_aggregator: SignatureAggregator,
    proposal_aggregator: ProposalAggregator,
    split_barrier: RwLock<SplitBarrier>,
    // Voter for Dkg
    dkg_voter: DkgVoter,
    relocation_state: CFOption<RelocationStatus>,
    pub(super) event_tx: mpsc::Sender<Event>,
    joins_allowed: CFValue<bool>,
    resource_proof: ResourceProof,
    end_users: EndUserRegistry,
    used_space: UsedSpace,
    pub(super) register_storage: RegisterStorage,
    root_storage_dir: PathBuf,
}

impl Core {
    // Creates `Core` for a regular node.
    pub(crate) fn new(
        comm: Comm,
        mut node: Node,
        section: Section,
        secret_key_share: Option<SectionDkgOutcome>,
        event_tx: mpsc::Sender<Event>,
        used_space: UsedSpace,
        root_storage_dir: PathBuf,
    ) -> Result<Self> {
        let section_keys = CFValue::new(SectionKeysProvider::new(KEY_CACHE_SIZE, secret_key_share));

        // make sure the Node has the correct local addr as Comm
        node.addr = comm.our_connection_info();

        let register_storage = RegisterStorage::new(&root_storage_dir, used_space.clone())?;

        Ok(Self {
            comm,
            node,
            section,
            network: CFValue::new(Network::new()),
            section_keys,
            proposal_aggregator: ProposalAggregator::default(),
            split_barrier: RwLock::new(SplitBarrier::new()),
            message_aggregator: SignatureAggregator::default(),
            dkg_voter: DkgVoter::default(),
            relocation_state: CFOption::new(),
            event_tx,
            joins_allowed: CFValue::new(true),
            resource_proof: ResourceProof::new(RESOURCE_PROOF_DATA_SIZE, RESOURCE_PROOF_DIFFICULTY),
            end_users: EndUserRegistry::new(),
            register_storage,
            root_storage_dir,
            used_space,
        })
    }

    ////////////////////////////////////////////////////////////////////////////
    // Miscellaneous
    ////////////////////////////////////////////////////////////////////////////

    pub(crate) async fn state_snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            is_elder: self.is_elder(),
            last_key: self.section.last_key().await,
            prefix: self.section.prefix().await,
            elders: self.section().authority_provider().await.names(),
        }
    }

    pub(crate) async fn update_state(&self, old: StateSnapshot) -> Result<Vec<Command>> {
        let mut commands = vec![];
        let new = self.state_snapshot().await;

        self.section_keys
            .get()
            .await
            .finalise_dkg(&self.section.last_key().await);

        if new.prefix != old.prefix {
            info!("Split");
        }

        if new.last_key != old.last_key {
            if new.is_elder {
                {
                    let provider = self.section.authority_provider().await;
                    let peers = provider.peers().format(", ");
                    info!(
                        "Section updated: prefix: ({:b}), key: {:?}, elders: {}",
                        new.prefix, new.last_key, peers,
                    );
                }

                if self.section_keys.get().await.has_key_share() {
                    commands.extend(self.promote_and_demote_elders().await?);
                    // Whenever there is an elders change, casting a round of joins_allowed
                    // proposals to sync.
                    let active_members: Vec<XorName> = self
                        .section
                        .active_members()
                        .await
                        .map(|peer| *peer.name())
                        .collect();
                    let msg_id = MessageId::from_content(&active_members)?;
                    commands.extend(
                        self.propose(Proposal::JoinsAllowed((
                            msg_id,
                            self.joins_allowed.clone().await,
                        )))
                        .await?,
                    );
                }

                self.print_network_stats().await;

                // Sending SectionKnowledge to other sections for new SAP.
                // TODO: remoev all this messaging once we have Anti-Entropy fully implemented.
                let signed_sap = self
                    .section
                    .section_signed_authority_provider()
                    .await
                    .clone();
                let node_msg = NodeMsg::SectionKnowledge {
                    src_info: (signed_sap, self.section.chain_clone().await),
                    msg: None,
                };
                let network = self.network.get().await;
                let sap_targets: Vec<_> = network
                    .all()
                    .await
                    .iter()
                    .map(|sap| {
                        let targets: Vec<_> = sap
                            .elders()
                            .iter()
                            .map(|(name, addr)| (*name, *addr))
                            .collect();
                        let dst_section_pk = sap.section_key();
                        (targets, dst_section_pk)
                    })
                    .collect();
                for (targets, dst_section_pk) in sap_targets {
                    trace!("Sending updated SectionInfo to all known sections");
                    let cmd = self
                        .send_direct_message_to_nodes(targets, node_msg.clone(), dst_section_pk)
                        .await?;

                    commands.push(cmd);
                }
            }

            if new.is_elder || old.is_elder {
                commands.extend(
                    self.send_sync(
                        &self.section,
                        self.network.get().await.as_ref().clone().await,
                    )
                    .await?,
                );
            }

            let current: BTreeSet<_> = self.section.authority_provider().await.names();
            let added = current.difference(&old.elders).copied().collect();
            let removed = old.elders.difference(&current).copied().collect();
            let remaining = old.elders.intersection(&current).copied().collect();

            let elders = Elders {
                prefix: new.prefix,
                key: new.last_key,
                remaining,
                added,
                removed,
            };

            let self_status_change = if !old.is_elder && new.is_elder {
                info!("Promoted to elder");
                NodeElderChange::Promoted
            } else if old.is_elder && !new.is_elder {
                info!("Demoted");
                self.network.set(Network::new()).await;
                self.section_keys
                    .set(SectionKeysProvider::new(KEY_CACHE_SIZE, None))
                    .await;
                NodeElderChange::Demoted
            } else {
                NodeElderChange::None
            };

            let sibling_elders = if new.prefix != old.prefix {
                self.network
                    .get()
                    .await
                    .get(&new.prefix.sibling())
                    .await
                    .map(|sec_auth| {
                        let current: BTreeSet<_> = sec_auth.names();
                        let added = current.difference(&old.elders).copied().collect();
                        let removed = old.elders.difference(&current).copied().collect();
                        let remaining = old.elders.intersection(&current).copied().collect();
                        Elders {
                            prefix: new.prefix.sibling(),
                            key: sec_auth.section_key(),
                            remaining,
                            added,
                            removed,
                        }
                    })
            } else {
                None
            };

            let event = if let Some(sibling_elders) = sibling_elders {
                Event::SectionSplit {
                    elders,
                    sibling_elders,
                    self_status_change,
                }
            } else {
                Event::EldersChanged {
                    elders,
                    self_status_change,
                }
            };

            self.send_event(event).await;
        }

        if !new.is_elder {
            commands.extend(self.return_relocate_promise().await);
        }

        Ok(commands)
    }

    pub(crate) async fn section_key_by_name(&self, name: &XorName) -> bls::PublicKey {
        if self.section.prefix().await.matches(name) {
            self.section.last_key().await
        } else if let Ok(key) = self.network.get().await.key_by_name(name).await {
            key
        } else if self.section.prefix().await.sibling().matches(name) {
            // For sibling with unknown key, use the previous key in our chain under the assumption
            // that it's the last key before the split and therefore the last key of theirs we know.
            // In case this assumption is not correct (because we already progressed more than one
            // key since the split) then this key would be unknown to them and they would send
            // us back their whole section chain. However, this situation should be rare.
            self.section.prev_key().await
        } else {
            self.section.root_key().await
        }
    }

    pub(crate) async fn print_network_stats(&self) {
        self.network
            .get()
            .await
            .network_stats(&self.section.authority_provider().await)
            .await
            .print()
    }
}

pub(crate) struct StateSnapshot {
    is_elder: bool,
    last_key: bls::PublicKey,
    prefix: Prefix,
    elders: BTreeSet<XorName>,
}
