// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    interaction::push_state,
    messaging::{send, send_error, send_support, send_to_nodes},
    role::{AdultRole, ElderRole, Role},
};
use crate::messaging::MessageId;
use crate::node::{
    chunk_store::ChunkStore,
    event_mapping::MsgContext,
    node_ops::{NodeDuties, NodeDuty},
    Node, Result,
};
use crate::routing::ELDER_SIZE;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{debug, info};
use xor_name::XorName;

#[derive(Debug)]
pub enum NodeTask {
    None,
    Result(Box<(NodeDuties, Option<MsgContext>)>),
    Thread(JoinHandle<Result<NodeTask>>),
}

impl From<NodeDuties> for NodeTask {
    fn from(duties: NodeDuties) -> Self {
        Self::Result(Box::new((duties, None)))
    }
}

impl Node {
    ///
    pub async fn handle(&self, duty: NodeDuty) -> Result<NodeTask> {
        if !matches!(duty, NodeDuty::NoOp) {
            debug!("Handling NodeDuty: {:?}", duty);
        }

        match duty {
            NodeDuty::Genesis => {
                self.level_up().await?;
                let elder = self.as_elder().await?;
                *elder.received_initial_sync.write().await = true;
                Ok(NodeTask::None)
            }
            NodeDuty::EldersChanged {
                our_key,
                our_prefix,
                new_elders,
                newbie,
            } => {
                if newbie {
                    info!("Promoted to Elder on Churn");
                    self.level_up().await?;
                    if self.network_api.our_prefix().await.is_empty()
                        && self.network_api.section_chain().await.len() <= ELDER_SIZE
                    {
                        let elder = self.as_elder().await?;
                        *elder.received_initial_sync.write().await = true;
                    }
                    Ok(NodeTask::None)
                } else {
                    info!("Updating on elder churn");
                    let elder = self.as_elder().await?;
                    let network = self.network_api.clone();
                    let handle = tokio::spawn(async move {
                        let msg_id =
                            MessageId::combine(&[our_prefix.name().0, XorName::from(our_key).0]);
                        let ops = vec![push_state(&elder, our_prefix, msg_id, new_elders).await?];
                        let our_adults = network.our_adults().await;
                        elder
                            .meta_data
                            .write()
                            .await
                            .retain_members_only(our_adults)
                            .await?;
                        Ok(NodeTask::from(ops))
                    });
                    Ok(NodeTask::Thread(handle))
                }
            }
            NodeDuty::AdultsChanged {
                added,
                removed,
                remaining,
            } => {
                let our_name = self.our_name().await;
                let adult_role = self.as_adult().await?;
                let handle = tokio::spawn(async move {
                    Ok(NodeTask::from(
                        adult_role
                            .reorganize_chunks(our_name, added, removed, remaining)
                            .await?,
                    ))
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::SectionSplit {
                our_key,
                our_prefix,
                our_new_elders,
                their_new_elders,
                sibling_key,
                newbie,
            } => {
                debug!(
                    "@@@@@@ SPLIT: Our prefix: {:?}, neighbour: {:?}",
                    our_prefix,
                    our_prefix.sibling(),
                );
                debug!(
                    "@@@@@@ SPLIT: Our key: {:?}, neighbour: {:?}",
                    our_key, sibling_key
                );
                if newbie {
                    info!("Beginning split as Newbie");
                    self.begin_split_as_newbie(our_key).await?;
                    Ok(NodeTask::None)
                } else {
                    info!("Beginning split as Oldie");
                    let elder = self.as_elder().await?;
                    let network = self.network_api.clone();
                    let handle = tokio::spawn(async move {
                        Ok(NodeTask::from(
                            Self::begin_split_as_oldie(
                                &elder,
                                &network,
                                our_prefix,
                                our_key,
                                sibling_key,
                                our_new_elders,
                                their_new_elders,
                            )
                            .await?,
                        ))
                    });
                    Ok(NodeTask::Thread(handle))
                }
            }
            NodeDuty::ProposeOffline(unresponsive_adults) => {
                for adult in unresponsive_adults {
                    self.network_api.propose_offline(adult).await?;
                }
                Ok(NodeTask::None)
            }
            // a remote section asks for the replicas of their wallet
            NodeDuty::GetSectionElders { msg_id, origin } => {
                let network = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    Ok(NodeTask::from(
                        Self::get_section_elders(&network, msg_id, origin).await?,
                    ))
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::ProcessLostMember { name, .. } => {
                info!("Member Lost: {:?}", name);
                let elder = self.as_elder().await?;
                let network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    let our_adults = network_api.our_adults().await;
                    elder
                        .meta_data
                        .write()
                        .await
                        .retain_members_only(our_adults)
                        .await?;
                    Ok(NodeTask::from(vec![NodeDuty::SetNodeJoinsAllowed(true)]))
                });
                Ok(NodeTask::Thread(handle))
            }
            //
            // ---------- Levelling --------------
            NodeDuty::SynchState { metadata } => {
                let elder = self.as_elder().await?;
                let handle = tokio::spawn(async move {
                    Ok(NodeTask::from(vec![
                        Self::synch_state(&elder, metadata).await?,
                    ]))
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::LevelDown => {
                info!("Getting Demoted");
                let store =
                    ChunkStore::new(self.node_info.root_dir.as_path(), self.used_space.clone())
                        .await?;
                *self.role.write().await = Role::Adult(AdultRole {
                    chunks: Arc::new(store),
                });
                Ok(NodeTask::None)
            }
            //
            // -------- Immutable chunks --------
            NodeDuty::ReadChunk { read, msg_id } => {
                let adult = self.as_adult().await?;
                let our_section_pk = self.network_api.our_public_key_set().await?.public_key();
                let handle = tokio::spawn(async move {
                    let mut ops = vec![adult.chunks.read(&read, msg_id, our_section_pk).await];
                    ops.extend(adult.chunks.check_storage().await?);
                    Ok(NodeTask::from(ops))
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::WriteChunk {
                write,
                msg_id,
                client_signed,
            } => {
                let adult = self.as_adult().await?;
                let handle = tokio::spawn(async move {
                    let mut ops = vec![
                        adult
                            .chunks
                            .write(&write, msg_id, client_signed.public_key)
                            .await?,
                    ];
                    ops.extend(adult.chunks.check_storage().await?);
                    Ok(NodeTask::from(ops))
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::ProcessRepublish { chunk, msg_id, .. } => {
                info!("Processing republish with MessageId: {:?}", msg_id);
                let elder = self.as_elder().await?;
                let handle = tokio::spawn(async move {
                    Ok(NodeTask::from(vec![
                        elder.meta_data.read().await.republish_chunk(chunk).await?,
                    ]))
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::ReachingMaxCapacity => {
                let network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    Ok(NodeTask::from(vec![
                        Self::notify_section_of_our_storage(&network_api).await?,
                    ]))
                });
                Ok(NodeTask::Thread(handle))
            }
            //
            // ------- Misc ------------
            NodeDuty::IncrementFullNodeCount { node_id } => {
                let elder = self.as_elder().await?;
                let handle = tokio::spawn(async move {
                    elder
                        .meta_data
                        .read()
                        .await
                        .increase_full_node_count(node_id)
                        .await;
                    // Accept a new node in place for the full node.
                    Ok(NodeTask::from(vec![NodeDuty::SetNodeJoinsAllowed(true)]))
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::Send(msg) => {
                let network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    send(msg, &network_api).await?;
                    Ok(NodeTask::None)
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::SendError(msg) => {
                let network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    send_error(msg, &network_api).await?;
                    Ok(NodeTask::None)
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::SendSupport(msg) => {
                let network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    send_support(msg, &network_api).await?;
                    Ok(NodeTask::None)
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::SendToNodes {
                id,
                msg,
                targets,
                aggregation,
            } => {
                let network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    send_to_nodes(id, msg, targets, aggregation, &network_api).await?;
                    Ok(NodeTask::None)
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::SetNodeJoinsAllowed(joins_allowed) => {
                let mut network_api = self.network_api.clone();
                let handle = tokio::spawn(async move {
                    network_api
                        .set_joins_allowed(cfg!(feature = "always-joinable") || joins_allowed)
                        .await?;
                    Ok(NodeTask::None)
                });
                Ok(NodeTask::Thread(handle))
            }
            //
            // ------- Data ------------
            NodeDuty::ProcessRead {
                query,
                msg_id,
                client_signed,
                origin,
            } => {
                let elder = self.as_elder().await?;
                let handle = tokio::spawn(async move {
                    let duties = vec![
                        elder
                            .meta_data
                            // this is a write here as we write the liveness check for each adult
                            .write()
                            .await
                            .read(query, msg_id, client_signed.public_key, origin)
                            .await?,
                    ];
                    Ok(NodeTask::from(duties))
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::ProcessWrite {
                cmd,
                msg_id,
                origin,
                client_signed,
            } => {
                let elder = self.as_elder().await?;
                let handle = tokio::spawn(async move {
                    Ok(NodeTask::from(vec![
                        elder
                            .meta_data
                            .write()
                            .await
                            .write(cmd, msg_id, client_signed, origin)
                            .await?,
                    ]))
                });
                Ok(NodeTask::Thread(handle))
            }
            // --- Completion of Adult operations ---
            NodeDuty::RecordAdultReadLiveness {
                response,
                correlation_id,
                src,
            } => {
                let elder = self.as_elder().await?;
                let handle = tokio::spawn(async move {
                    Ok(NodeTask::from(
                        elder
                            .meta_data
                            .write()
                            .await
                            .record_adult_read_liveness(correlation_id, response, src)
                            .await?,
                    ))
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::ReplicateChunk { chunk, .. } => {
                let adult = self.as_adult().await?;
                let handle = tokio::spawn(async move {
                    Ok(NodeTask::from(vec![
                        adult.chunks.store_for_replication(chunk).await?,
                    ]))
                });
                Ok(NodeTask::Thread(handle))
            }
            NodeDuty::NoOp => Ok(NodeTask::None),
        }
    }

    async fn as_adult(&self) -> Result<AdultRole> {
        let role = self.role.read().await;
        Ok(role.as_adult()?.clone())
    }

    async fn as_elder(&self) -> Result<ElderRole> {
        let role = self.role.read().await;
        Ok(role.as_elder()?.clone())
    }
}
