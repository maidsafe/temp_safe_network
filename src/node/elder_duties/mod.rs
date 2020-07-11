// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod gateway;
mod metadata;
mod msg_decisions;
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
use crate::{cmd::NodeCmd, keys::NodeKeys, node::Init, Config, Result};
use routing::{Node as Routing, RoutingError};
use safe_transfers::TransferActor;
use safe_nd::{XorName, Keypair, BlsKeypairShare};
use threshold_crypto::{self, serde_impl::SerdeSecret};
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
    ) -> Result<Self> {
        // Gateway
        let gateway = Gateway::new(keys.clone(), &config, init_mode, routing.clone())?;

        // Metadata
        let metadata = Metadata::new(
            keys.clone(),
            &config,
            &total_used_space,
            init_mode,
            routing.clone(),
        )?;

        // (AT2 Replicas)
        let replica_manager = Self::replica_manager(routing)?;

        // Transfers
        let transfers = Transfers::new(keys.clone(), replica_manager.clone());

        // DataPayment
        let data_payment = DataPayment::new(keys.clone(), routing.clone(), replica_manager);

        // Rewards
        let keypair = key_pair(routing)?;
        let pk_set = replica_manager.borrow().replicas_pk_set().unwrap();
        let actor = TransferActor::new(keypair, pk_set, Validator { });
        let rewards = Rewards::new(keys.clone(), actor);

        Ok(Self {
            keys,
            gateway,
            metadata,
            transfers,
            data_payment,
            rewards,
            routing: routing.clone(),
        })
    }

    fn key_pair(routing: Rc<RefCell<Routing>>) -> Result<Keypair> {
        let node = routing.borrow();
        let index = node.our_index()?;
        let bls_secret_key = node.secret_key_share()?;
        let secret = SerdeSecret(bls_secret_key);
        let public = bls_secret_key.public_key_share()
        Ok(Keypair::BlsShare(BlsKeypairShare {
            index,
            secret,
            public,
        }))
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
        Rc::new(RefCell::new(replica_manager))
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

    /// Name of the node
    /// Age of the node
    pub fn member_left(&mut self, _name: XorName, _age: u8) -> Option<Vec<NodeCmd>> {
        None
        // For now, we skip chunk duplication logic.
        //self.metadata.trigger_chunk_duplication(XorName(name.0))
    }

    // Update our replica with the latest keys
    pub fn elders_changed(&mut self) -> Option<NodeCmd> {
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
}

impl Display for ElderDuties {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
