// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::role::{ElderRole, Role};
use crate::messaging::client::DataExchange;
use crate::node::{
    capacity::{AdultsStorageInfo, Capacity, CapacityReader, CapacityWriter},
    metadata::{adult_reader::AdultReader, Metadata},
    node_ops::NodeDuty,
    Node, Result,
};
use tracing::info;

impl Node {
    /// Level up a newbie to an oldie on promotion
    pub async fn level_up(&self) -> Result<()> {
        self.used_space.reset().await?;

        let adult_storage_info = AdultsStorageInfo::new();
        let adult_reader = AdultReader::new(self.network_api.clone());
        let capacity_reader = CapacityReader::new(adult_storage_info.clone(), adult_reader.clone());
        let capacity_writer = CapacityWriter::new(adult_storage_info.clone(), adult_reader.clone());
        let capacity = Capacity::new(capacity_reader.clone(), capacity_writer);

        //
        // start handling metadata
        let max_capacity = self.used_space.max_capacity().await;
        let meta_data =
            Metadata::new(&self.node_info.path(), max_capacity, capacity.clone()).await?;

        *self.role.write().await = Role::Elder(ElderRole::new(meta_data, false));

        Ok(())
    }

    /// Continue the level up and handle more responsibilities.
    pub(crate) async fn synch_state(elder: &ElderRole, metadata: DataExchange) -> Result<NodeDuty> {
        if *elder.received_initial_sync.read().await {
            info!("We are already received the initial sync from our section. Ignoring update");
            return Ok(NodeDuty::NoOp);
        }
        // --------- merge in provided metadata ---------
        elder.meta_data.write().await.update(metadata).await?;

        *elder.received_initial_sync.write().await = true;

        Ok(NodeDuty::NoOp)
    }
}
