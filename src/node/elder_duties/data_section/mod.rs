// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod metadata;
mod rewards;

use self::{
    metadata::Metadata,
    rewards::{Rewards, Validator},
};

use crate::{
    node::node_ops::{DataSectionDuty, NodeOperation, RewardDuty},
    node::section_querying::SectionQuerying,
    node::state_db::NodeInfo,
    utils, Result,
};
use routing::Node as Routing;
use safe_nd::XorName;
use safe_transfers::TransferActor;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

///
pub struct DataSection {
    ///
    metadata: Metadata,
    ///
    rewards: Rewards,
}

impl DataSection {
    ///
    pub fn new(
        info: NodeInfo,
        total_used_space: &Rc<Cell<u64>>,
        routing: Rc<RefCell<Routing>>,
    ) -> Result<Self> {
        let section_querying = SectionQuerying::new(routing.clone());

        // Metadata
        let metadata = Metadata::new(info.clone(), &total_used_space, section_querying)?;

        // Rewards
        let keypair = utils::key_pair(routing.clone())?;
        let public_key_set = routing.borrow().public_key_set()?.clone();
        let actor = TransferActor::new(keypair, public_key_set, Validator {});
        let rewards = Rewards::new(info.keys.clone(), actor);

        Ok(Self { metadata, rewards })
    }

    pub fn process(&mut self, duty: DataSectionDuty) -> Option<NodeOperation> {
        use DataSectionDuty::*;
        match duty {
            RunAsMetadata(duty) => self.metadata.process(&duty),
            RunAsRewards(duty) => self.rewards.process(duty),
        }
    }

    pub fn relocated_member_joined(
        &mut self,
        old_node_id: XorName,
        new_node_id: XorName,
    ) -> Option<NodeOperation> {
        // marks the reward account as
        // awaiting claiming of the counter
        self.rewards.process(RewardDuty::AddRelocatedAccount {
            old_node_id,
            new_node_id,
        })
        // For now, we skip chunk duplication logic.
        //self.metadata.trigger_chunk_duplication(XorName(name.0))
    }

    /// Name of the node
    /// Age of the node
    pub fn member_left(&mut self, node_id: XorName, _age: u8) -> Option<NodeOperation> {
        // marks the reward account as
        // awaiting claiming of the counter
        self.rewards
            .process(RewardDuty::PrepareAccountMove { node_id })
        // For now, we skip chunk duplication logic.
        //self.metadata.trigger_chunk_duplication(XorName(name.0))
    }
}
