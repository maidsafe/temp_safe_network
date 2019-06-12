// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    adult::Adult, coins_handler::CoinsHandler, destination_elder::DestinationElder, error::Result,
    source_elder::SourceElder,
};
use pickledb::PickleDb;
use quic_p2p::QuicP2p;
use safe_nd::{ClientPublicId, NodeFullId};
use std::{collections::HashMap, net::SocketAddr};

#[allow(clippy::large_enum_variant)]
enum State {
    Elder {
        src: SourceElder,
        dst: DestinationElder,
        coins_handler: CoinsHandler,
    },
    Adult(Adult),
}

/// Main vault struct.
pub struct Vault {
    id: NodeFullId,
    state: State,
    quic_p2p: QuicP2p,
}

impl Vault {
    /// Construct a new vault instance.
    pub fn new() -> Result<Self> {
        unimplemented!();
    }

    /// Run the main event loop.  Blocks until the vault is terminated.
    pub fn run(&mut self) {
        unimplemented!();
    }
}
