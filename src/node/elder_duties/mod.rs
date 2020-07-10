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
mod transfers;
mod rewards;

use self::{
    gateway::Gateway,
    metadata::Metadata,
    payment::DataPayment,
    rewards::Rewards,
    transfers::{replica_manager::ReplicaManager, Transfers},
};
use crate::{
    cmd::{ConsensusAction, NodeCmd, GatewayCmd, NodeCmd},
    msg::Message,
    node::Init,
    utils, Config, Result,
};
use bytes::Bytes;
use log::trace;
use rand::{CryptoRng, Rng};
use routing::{Node as Routing, RoutingError, SrcLocation};
use safe_nd::{
    Message, MessageId, NodePublicId, PublicId, XorName,
};
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    rc::Rc,
};
use threshold_crypto::Signature;

pub(crate) struct ElderDuties {
    id: NodePublicId,
    metadata: Metadata,
    transfers: Transfers,
    gateway: Gateway,
    data_payment: DataPayment,
    rewards: Rewards,
    routing: Rc<RefCell<Routing>>,
}

impl ElderDuties {
    pub fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
        routing: Rc<RefCell<Routing>>,
    ) -> Result<Self> {
        // Gateway
        let gateway = Gateway::new(id.clone(), &config, init_mode, routing.clone())?;

        // Metadata
        let metadata = Metadata::new(
            id.clone(),
            &config,
            &total_used_space,
            init_mode,
            routing.clone(),
        )?;

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
        let replica_manager = Rc::new(RefCell::new(replica_manager));

        // Transfers
        let transfers = Transfers::new(id.clone(), replica_manager.clone());

        // DataPayment
        let data_payment = DataPayment::new(id.clone(), routing.clone(), replica_manager);

        let actor = TransferActor::new();
        let rewards = Rewards::new(actor);

        Ok(Self {
            id,
            gateway,
            metadata,
            transfers,
            data_payment,
            routing: routing.clone(),
        })
    }

    pub fn gateway(&mut self) -> &mut Gateway {
        self.gateway
    }

    pub fn data_payment(&mut self) -> &mut DataPayment {
        self.data_payment
    }

    pub fn metadata(&mut self) -> &mut Metadata {
        self.metadata
    }

    pub fn transfers(&mut self) -> &mut Transfers {
        self.transfers
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
        write!(formatter, "{}", self.id.name())
    }
}
