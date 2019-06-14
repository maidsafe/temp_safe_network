// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use log::{info, trace};
use crate::action::Action;
use pickledb::PickleDb;
use quic_p2p::{Config as QuickP2pConfig, Event, QuicP2p};
use safe_nd::ClientPublicId;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::mpsc::{channel, Receiver, Sender},
};
use unwrap::unwrap;

pub(crate) struct SourceElder {
    //client_accounts: PickleDb,
    clients: HashMap<SocketAddr, ClientPublicId>,
    quic_p2p: QuicP2p,
}

impl SourceElder {
    pub fn new(config: QuickP2pConfig) -> (Self, Receiver<Event>) {
        let (quic_p2p, event_receiver) = {
            let (event_sender, event_receiver) = channel();
            let mut quic_p2p = unwrap!(quic_p2p::Builder::new(event_sender)
                .with_config(config)
                .build());
            let our_conn_info = unwrap!(quic_p2p.our_connection_info());
            info!("QuickP2p started on {}", our_conn_info.peer_addr);
            println!(
                "Our connection info:\n{}\n",
                unwrap!(serde_json::to_string(&our_conn_info))
            );
            (quic_p2p, event_receiver)
        };

        (
            Self {
                clients: Default::default(),
                quic_p2p,
            },
            event_receiver,
        )
    }

    pub fn handle_client_request(
        &mut self,
        client_id: &ClientPublicId,
        msg: Vec<u8>,
    ) -> Option<Action> {
        unimplemented!();
    }
}
