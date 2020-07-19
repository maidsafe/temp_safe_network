// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub mod messaging;
mod network_events;
mod msg_analysis;
mod accumulation;

use network_events::NetworkEvents;
use msg_analysis::NetworkMsgAnalysis;
use accumulation::Accumulation;
use crate::{
    node::node_ops::{NodeDuty, GroupDecision, NodeOperation, MessagingDuty},
    node::{
        adult_duties::AdultDuties,
        elder_duties::ElderDuties,
        keys::NodeKeys,
        node_duties::messaging::{Messaging, Receiver, Received},
    },
    Config, Result,
};
use routing::Node as Routing;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[allow(clippy::large_enum_variant)]
pub enum AgeLevel {
    Infant,
    Adult(AdultDuties),
    Elder(ElderDuties),
}

pub struct NodeDuties {
    keys: NodeKeys,
    age_level: AgeLevel,
    network_events: NetworkEvents,
    messaging: Messaging,
    routing: Rc<RefCell<Routing>>,
    root_dir: Path,
}

impl NodeDuties {
    
    pub fn new(keys: NodeKeys, age_level: AgeLevel,
        routing: Rc<RefCell<Routing>>, root_dir: Path) -> Self {
            let network_events = NetworkEvents::new(NetworkMsgAnalysis::new(routing.clone()));
            let messaging = Messaging::new(routing.clone());
            Self {
                keys,
                age_level,
                network_events,
                messaging,
                routing,
                root_dir,
            }
    }

    pub fn adult_duties(&mut self) -> Option<&mut AdultDuties> {
        use AgeLevel::*;
        match &mut self.age_level {
            Adult(ref mut duties) => Some(duties),
            _ => None,
        }
    }

    pub fn elder_duties(&mut self) -> Option<&mut ElderDuties> {
        use AgeLevel::*;
        match &mut self.age_level {
            Elder(ref mut duties) => Some(duties),
            _ => None,
        }
    }

    pub fn process(&mut self, duty: NodeDuty) -> Option<NodeOperation> {
        use NodeDuty::*;
        match duty {
            BecomeAdult => self.become_adult(),
            BecomeElder => self.become_elder(),
            ProcessMessaging(duty) => self.messaging.process(duty),
            ProcessNetworkEvent(event) => self.network_events.process(event),
        }
    }

    fn become_adult(&mut self) -> Option<NodeOperation> {
        use AgeLevel::*;
        // let mut config = Config::default();
        // config.set_root_dir(self.root_dir.clone());
        let total_used_space = Rc::new(Cell::new(0));
        if let Ok(duties) = AdultDuties::new(self.keys.clone(), &self.root_dir, &total_used_space, Init::New) {
            self.age_level = Adult(duties);
            dump_state(false, root_dir, id)?;
        }
        None
    }

    fn become_elder(&mut self) -> Option<NodeOperation> {
        use AgeLevel::*;
        // let mut config = Config::default();
        // config.set_root_dir(self.root_dir.clone());
        let total_used_space = Rc::new(Cell::new(0));

        if let Ok(duties) = ElderDuties::new(
            self.id.clone(),
            self.keys.clone(),
            &self.root_dir,
            &total_used_space,
            Init::New,
            self.routing.clone(),
        ) {
            self.age_level = Elder(duties);
            dump_state(true, &root_dir, id)?;
        }
        None
    }
}
