// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{BlobDataExchange, MapDataExchange, SequenceDataExchange};
use crate::node_ops::{NodeDuties, OutgoingMsg};
use crate::{
    capacity::{Capacity, ChunkHolderDbs, RateLimit},
    metadata::{adult_reader::AdultReader, Metadata},
    node::{ElderRole, Role},
    node_ops::NodeDuty,
    section_funds::{reward_wallets::RewardWallets, SectionFunds},
    transfers::{
        get_replicas::{replica_info, transfer_replicas},
        Transfers,
    },
    Error, Node, Result,
};
use dashmap::DashMap;
use log::info;
use sn_data_types::{ActorHistory, NodeAge, PublicKey};
use sn_messaging::client::NodeQuery;
use sn_messaging::client::{Message, NodeSystemQuery};
use sn_messaging::{Aggregation, DstLocation, MessageId};
use sn_routing::XorName;
use std::collections::BTreeMap;

impl Node {
    /// If we are an oldie we'll have a transfer instance,
    /// This updates the replica info on it.
    pub async fn update_replicas(&mut self) -> Result<()> {
        let elder = self.role.as_elder_mut()?;
        let info = replica_info(&self.network_api).await?;
        elder.transfers.update_replica_info(info);
        Ok(())
    }

    /// Level up a newbie to an oldie on promotion
    pub async fn level_up(&mut self, is_genesis: bool) -> Result<NodeDuties> {
        self.used_space.reset().await?;

        //
        // start handling metadata
        let dbs = ChunkHolderDbs::new(self.node_info.path())?;
        let reader = AdultReader::new(self.network_api.clone());
        let meta_data =
            Metadata::new(&self.node_info.path(), &self.used_space, dbs, reader).await?;

        //
        // start handling transfers
        let dbs = ChunkHolderDbs::new(self.node_info.root_dir.as_path())?;
        let rate_limit = RateLimit::new(self.network_api.clone(), Capacity::new(dbs.clone()));
        let user_wallets = BTreeMap::<PublicKey, ActorHistory>::new();
        let replicas = transfer_replicas(&self.node_info, &self.network_api, user_wallets).await?;
        let transfers = Transfers::new(replicas, rate_limit);

        //
        // start handling node rewards
        let section_funds = SectionFunds::KeepingNodeWallets {
            wallets: RewardWallets::new(BTreeMap::<XorName, (NodeAge, PublicKey)>::new()),
            payments: DashMap::new(),
        };

        self.role = Role::Elder(ElderRole {
            meta_data,
            transfers,
            section_funds,
            is_caught_up: false,
        });

        if is_genesis {
            Ok(vec![])
        } else {
            let duty: NodeDuty = NodeDuty::Send(OutgoingMsg {
                msg: Message::NodeQuery {
                    query: NodeQuery::System(NodeSystemQuery::UpdateData {
                        our_name: self.network_api.our_name().await,
                        section_pk: self.network_api.section_public_key().await?,
                    }),
                    id: MessageId::new(),
                    target_section_pk: Some(self.network_api.section_public_key().await?),
                },
                dst: DstLocation::Section(self.our_prefix().await.name()),
                section_source: false,
                aggregation: Aggregation::None,
            });

            // TODO(drusu): return a  mutable reference to elder state?
            Ok(vec![duty])
        }
    }

    /// Continue the level up and handle more responsibilities.
    pub async fn synch_state(
        &mut self,
        node_wallets: BTreeMap<XorName, (NodeAge, PublicKey)>,
        user_wallets: BTreeMap<PublicKey, ActorHistory>,
    ) -> Result<NodeDuty> {
        let elder = self.role.as_elder_mut()?;

        // merge in provided user wallets
        elder.transfers.merge(user_wallets).await?;

        //  merge in provided node reward stages
        for (key, (age, wallet)) in &node_wallets {
            elder.section_funds.set_node_wallet(*key, *wallet, *age)
        }

        let node_id = self.network_api.our_name().await;
        let no_wallet_found = node_wallets.get(&node_id).is_none();
        if no_wallet_found {
            info!(
                "Registering wallet of node: {} (since not found in received state)",
                node_id,
            );
            Ok(NodeDuty::Send(self.register_wallet().await))
        } else {
            Ok(NodeDuty::NoOp)
        }
    }

    ///
    pub async fn fetch_all_data(&self) -> Result<BTreeMap<String, Vec<u8>>> {
        let elder_role = self.role.as_elder()?;

        // Prepare blob_register, map and sequence data
        let blob_register = elder_role.transfers.fetch_blob_register().await?;
        let (map_list, seq_list) = elder_role.meta_data.fetch_map_and_sequence()?;

        // Create an aggregated map of all the data
        let mut aggregated_map = BTreeMap::new();
        let _ = aggregated_map.insert(
            "BlobRegister".to_string(),
            bincode::serialize(&blob_register)?,
        );
        let _ = aggregated_map.insert("MapData".to_string(), bincode::serialize(&map_list)?);
        let _ = aggregated_map.insert("SequenceData".to_string(), bincode::serialize(&seq_list)?);
        Ok(aggregated_map)
    }

    ///
    pub async fn furnish(&mut self, data: BTreeMap<String, Vec<u8>>) -> Result<()> {
        info!("Furnishing Chunkstores with received data");
        let elder_role = self.role.as_elder_mut()?;

        if !elder_role.is_caught_up {
            let serialized_blob_reg = data.get("BlobRegister").ok_or_else(|| {
                Error::Logic("Cannot find Blob_register on received data".to_string())
            })?;
            let serialized_map_list = data
                .get("MapData")
                .ok_or_else(|| Error::Logic("Cannot find MapData on received data".to_string()))?;
            let serialized_seq_list = data.get("SequenceData").ok_or_else(|| {
                Error::Logic("Cannot find SequenceData on received data".to_string())
            })?;

            let blob_reg: BlobDataExchange = bincode::deserialize(serialized_blob_reg)?;
            let map_list: MapDataExchange = bincode::deserialize(serialized_map_list)?;
            let seq_list: SequenceDataExchange = bincode::deserialize(serialized_seq_list)?;

            elder_role.transfers.update_blob_register(blob_reg).await?;
            elder_role
                .meta_data
                .update_map_and_sequence((map_list, seq_list))
                .await?;
            elder_role.is_caught_up = true;
        } else {
            info!("We are caught up with the section already. Ignoring update");
        }
        Ok(())
    }
}
