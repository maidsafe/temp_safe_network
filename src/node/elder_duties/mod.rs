// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod gateway;
mod metadata;
mod payment;
mod rewards;
mod transfers;

use self::{
    gateway::Gateway,
    metadata::Metadata,
    payment::DataPayment,
    rewards::{Rewards, Validator},
    transfers::{replica_manager::ReplicaManager, Transfers},
};
use crate::{
    cmd::MessagingDuty, messaging::ClientMessaging, node::keys::NodeKeys,
    node::section_querying::SectionQuerying, node::Init, utils, Config, Result,
};
use routing::{Node as Routing, RoutingError};
use safe_nd::{XorName, MsgEnvelope, NetworkEvent, Message, NetworkCmd};
use safe_transfers::TransferActor;
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(crate) struct ElderDuties {
    keys: NodeKeys,
    metadata: Metadata,
    transfers: Transfers,
    gateway: Gateway,
    data_payment: DataPayment,
    rewards: Rewards,
    routing: Rc<RefCell<Routing>>,
}

impl ElderDuties {
    pub fn new(
        keys: NodeKeys,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
        routing: Rc<RefCell<Routing>>,
        messaging: ClientMessaging,
    ) -> Result<Self> {
        // Gateway
        let gateway = Gateway::new(keys.clone(), &config, init_mode, messaging)?;

        let section_querying = SectionQuerying::new(routing.clone());
        // Metadata
        let metadata = Metadata::new(
            keys.clone(),
            &config,
            &total_used_space,
            init_mode,
            section_querying,
        )?;

        // (AT2 Replicas)
        let replica_manager = Self::replica_manager(routing.clone())?;

        // Transfers
        let transfers = Transfers::new(keys.clone(), replica_manager.clone());

        // DataPayment
        let data_payment = DataPayment::new(keys.clone(), replica_manager.clone());

        // Rewards
        let keypair = utils::key_pair(routing.clone())?;
        let pk_set = replica_manager.borrow().replicas_pk_set().unwrap();
        let actor = TransferActor::new(keypair, pk_set, Validator {});
        let rewards = Rewards::new(keys.clone(), actor);

        Ok(Self {
            keys,
            gateway,
            metadata,
            transfers,
            data_payment,
            rewards,
            routing,
        })
    }

    fn replica_manager(routing: Rc<RefCell<Routing>>) -> Result<Rc<RefCell<ReplicaManager>>> {
        let node = routing.borrow();
        let public_key_set = node.public_key_set()?;
        let secret_key_share = node.secret_key_share()?;
        let key_index = node.our_index()?;
        let proof_chain = node.our_history().ok_or(RoutingError::InvalidState)?;
        let replica_manager = ReplicaManager::new(
            secret_key_share,
            key_index,
            public_key_set,
            vec![],
            proof_chain.clone(),
        )?;
        Ok(Rc::new(RefCell::new(replica_manager)))
    }

    pub fn gateway(&mut self) -> &mut Gateway {
        &mut self.gateway
    }

    pub fn data_payment(&mut self) -> &mut DataPayment {
        &mut self.data_payment
    }

    pub fn metadata(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    pub fn transfers(&mut self) -> &mut Transfers {
        &mut self.transfers
    }

    pub fn rewards(&mut self) -> &mut Rewards {
        &mut self.rewards
    }

    pub fn relocated_member_joined(
        &mut self,
        old_node_id: XorName,
        new_node_id: XorName,
    ) -> Option<Vec<MessagingDuty>> {
        // marks the reward account as
        // awaiting claiming of the counter
        if let Some(msg) = self.rewards.add_relocated_account(old_node_id, new_node_id) {
            Some(vec![msg])
        } else {
            None
        }
        // For now, we skip chunk duplication logic.
        //self.metadata.trigger_chunk_duplication(XorName(name.0))
    }

    /// Name of the node
    /// Age of the node
    pub fn member_left(&mut self, node_id: XorName, _age: u8) -> Option<Vec<MessagingDuty>> {
        // marks the reward account as
        // awaiting claiming of the counter
        if let Some(msg) = self.rewards.node_left(node_id) {
            Some(vec![msg])
        } else {
            None
        }
        // For now, we skip chunk duplication logic.
        //self.metadata.trigger_chunk_duplication(XorName(name.0))
    }

    // Update our replica with the latest keys
    pub fn elders_changed(&mut self) -> Option<MessagingDuty> {
        let pub_key_set = self.routing.borrow().public_key_set().ok()?.clone();
        let sec_key_share = self.routing.borrow().secret_key_share().ok()?.clone();
        let proof_chain = self.routing.borrow().our_history()?.clone();
        let our_index = self.routing.borrow().our_index().ok()?;
        self.transfers.update_replica_on_churn(
            pub_key_set,
            sec_key_share,
            our_index,
            proof_chain,
        )?;
        None
    }

    pub fn receive_reward_msg(&mut self, msg: &MsgEnvelope) -> Option<MessagingDuty> {
        match &msg.message {
            Message::NetworkCmd {
                cmd:
                    NetworkCmd::ClaimRewardCounter {
                        old_node_id,
                        new_node_id,
                    },
                id,
            } => self.rewards.claim_rewards(*old_node_id, *new_node_id, *id, &msg.origin),
            Message::NetworkEvent {
                event:
                    NetworkEvent::RewardCounterClaimed {
                        new_node_id,
                        account_id,
                        counter,
                    },
                ..
            } => self.rewards.receive_claimed_rewards(*account_id, new_node_id, counter),
            _ => None,
        }
    }
}

impl Display for ElderDuties {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
