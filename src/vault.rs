// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    action::Action, adult::Adult, coins_handler::CoinsHandler, destination_elder::DestinationElder,
    error::Result, source_elder::SourceElder,
};
use log::info;
use pickledb::PickleDb;
use quic_p2p::{Config as QuickP2pConfig, Event, QuicP2p};
use safe_nd::{ClientPublicId, NodeFullId, Request};
use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::Receiver,
    net::SocketAddr,
};

#[allow(clippy::large_enum_variant)]
enum State {
    Elder {
        src: SourceElder,
        //dst: DestinationElder,
        //coins_handler: CoinsHandler,
    },
    Adult(Adult),
}

/// Main vault struct.
pub struct Vault {
    //id: NodeFullId,
    state: State,
    event_receiver: Receiver<Event>,
}

impl Vault {
    /// Construct a new vault instance.
    pub fn new(config: QuickP2pConfig) -> Result<Self> {
        let (src, event_receiver) = SourceElder::new(config);

        Ok(Self {
            //id: Default::default(),
            state: State::Elder { src },
            event_receiver,
        })
    }

    /// Run the main event loop.  Blocks until the vault is terminated.
    pub fn run(&mut self) {
        for event in self.event_receiver.iter() {
            match event {
                Event::ConnectedTo { peer } => {
                    info!("Connected to {:?}", peer);
                }
                event => info!("Unexpected event: {:?}", event),
            }
        }
    }

    fn handle_client_request(
        &mut self,
        client_id: &ClientPublicId,
        msg: Vec<u8>,
    ) -> Option<Action> {
        self.source_elder_mut()
            .and_then(|source_elder| source_elder.handle_client_request(client_id, msg))
    }

    fn source_elder(&self) -> Option<&SourceElder> {
        match &self.state {
            State::Elder { ref src, .. } => Some(src),
            State::Adult(_) => None,
        }
    }

    fn source_elder_mut(&mut self) -> Option<&mut SourceElder> {
        match &mut self.state {
            State::Elder { ref mut src, .. } => Some(src),
            State::Adult(_) => None,
        }
    }
}
