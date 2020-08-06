// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use tokio::time::timeout;

use crate::{client::SafeKey, network_event::NetworkEvent, network_event::NetworkTx, CoreError};
use log::{error, info, trace};
use safe_nd::{
    BlsProof, DebitAgreementProof, HandshakeRequest, HandshakeResponse, Message, MessageId,
    MsgEnvelope, MsgSender, NodePublicId, Proof, PublicId, QueryResponse,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

use bincode::{deserialize, serialize};
use bytes::Bytes;
use quic_p2p::{self, Config as QuicP2pConfig, Connection, QuicP2pAsync};
use std::net::SocketAddr;

const CONNECTION_TIMEOUT_SECS: u64 = 30;

/// Initialises `QuicP2p` instance. Establishes new connections.
pub struct ConnectionManager {
    //config: QuicP2pConfig,
    full_id: SafeKey,
    quic_p2p: QuicP2pAsync,
    elders: Vec<Connection>,
}

impl ConnectionManager {
    /// Create a new connection manager.
    pub fn new(mut config: QuicP2pConfig, full_id: SafeKey) -> Result<Self, CoreError> {
        config.port = Some(0); // Make sure we always use a random port for client connections.
        let quic_p2p = QuicP2pAsync::with_config(Some(config), Default::default(), false)?;

        Ok(Self {
            full_id,
            quic_p2p,
            elders: Vec::default(),
        })
    }

    /// Connect to Client Handlers that manage the provided ID.
    pub async fn bootstrap(&mut self) -> Result<(), CoreError> {
        trace!(
            "Trying to bootstrap to the network with public_id: {:?}",
            self.full_id.public_id()
        );

        // Bootstrap and send a handshake request to receive
        // the list of Elders we can then connect to
        let elders_addrs = self.bootstrap_and_handshake().await?;

        // Let's now connect to all Elders
        self.connect_to_elders(elders_addrs).await
    }

    /// Send `message` via the `ConnectionGroup` specified by our given `pub_id`.
    pub async fn send_cmd(&mut self, pub_id: &PublicId, msg: &Message) -> Result<(), CoreError> {
        info!("Sending command message {:?} w/ id: {:?}", &msg, &msg.id());
        let envelope = self.get_envelope_for_message(msg.clone())?;
        let msg_bytes = Bytes::from(serialize(&envelope)?);
        // send withou awaiting for a response
        trace!("Sending command to Elders...");

        // TODO: send to all elders in parallel and find majority on responses
        self.elders[0]
            .send_only(msg_bytes)
            .await
            .map_err(|err| CoreError::from(err))
    }

    /// Send `message` via the `ConnectionGroup` specified by our given `pub_id`.
    pub async fn send_query(
        &mut self,
        pub_id: &PublicId,
        msg: &Message,
    ) -> Result<QueryResponse, CoreError> {
        info!("Sending query message {:?} w/ id: {:?}", &msg, &msg.id());
        let envelope = self.get_envelope_for_message(msg.clone())?;
        let msg_bytes = Bytes::from(serialize(&envelope)?);
        // send and await response
        trace!("Sending message to Elders...");

        // TODO: send to all elders in parallel and find majority on responses
        let response = self.elders[0].send(msg_bytes).await?;

        match deserialize(&response) {
            Ok(res) => {
                trace!("Query response received");
                Ok(res)
            }
            Ok(_) => {
                let err_msg = "Unexpected message type when expecting a 'Response'.".to_string();
                error!("{}", err_msg);
                Err(CoreError::Unexpected(err_msg))
            }
            Err(e) => {
                let err_msg = format!("Unexpected error: {:?}", e);
                error!("{}", err_msg);
                Err(CoreError::Unexpected(err_msg))
            }
        }
    }

    // Private helpers

    fn get_envelope_for_message(&self, message: Message) -> Result<MsgEnvelope, CoreError> {
        trace!("Putting message in envelope: {:?}", message);
        let sign = self.full_id.sign(&serialize(&message)?);
        let msg_proof = BlsProof {
            public_key: self.full_id.public_key().bls().unwrap(),
            signature: sign.into_bls().unwrap(),
        };

        Ok(MsgEnvelope {
            message,
            origin: MsgSender::Client(Proof::Bls(msg_proof)),
            proxies: Default::default(),
        })
    }

    async fn bootstrap_and_handshake(&mut self) -> Result<Vec<SocketAddr>, CoreError> {
        trace!("Bootstrapping with contacts...");
        let mut node_connection = self.quic_p2p.bootstrap().await?;

        trace!("Sending handshake request to bootstrapped node...");
        let public_id = self.full_id.public_id();
        let handshake = HandshakeRequest::Bootstrap(public_id);
        let msg = Bytes::from(serialize(&handshake)?);
        let response = node_connection.send(msg).await?;

        match deserialize(&response) {
            Ok(HandshakeResponse::Rebootstrap(_elders)) => {
                trace!("HandshakeResponse::Rebootstrap, trying again");
                // TODO: initialise `hard_coded_contacts` with received `elders`.
                unimplemented!();
            }
            Ok(HandshakeResponse::Join(elders)) => {
                trace!("HandshakeResponse::Join Elders: ({:?})", elders);

                // Obtain the addresses of the Elders
                let elders_addrs = elders.into_iter().map(|(_xor_name, ci)| ci).collect();
                Ok(elders_addrs)
            }
            Ok(_msg) => Err(CoreError::from(
                "Unexpected message type received while expecting list of Elders to join.",
            )),
            Err(e) => Err(CoreError::from(format!("Unexpected error {:?}", e))),
        }
    }

    async fn connect_to_elders(&mut self, elders_addrs: Vec<SocketAddr>) -> Result<(), CoreError> {
        // TODO: connect to all Elders in parallel
        let peer_addr = elders_addrs[0];

        let mut conn = self.quic_p2p.connect_to(peer_addr).await?;

        let handshake = HandshakeRequest::Join(self.full_id.public_id());
        let msg = Bytes::from(serialize(&handshake)?);
        let join_response = conn.send(msg).await?;
        match deserialize(&join_response) {
            Ok(HandshakeResponse::Challenge(PublicId::Node(node_public_id), challenge)) => {
                trace!(
                    "Got the challenge from {:?}, public id: {}",
                    peer_addr,
                    node_public_id
                );
                let response = HandshakeRequest::ChallengeResult(self.full_id.sign(&challenge));
                let msg = Bytes::from(serialize(&response)?);
                conn.send_only(msg).await?;
                self.elders = vec![conn];
                Ok(())
            }
            Ok(_) => Err(CoreError::from(format!(
                "Unexpected message type while expeccting challenge from Elder."
            ))),
            Err(e) => Err(CoreError::from(format!("Unexpected error {:?}", e))),
        }
    }
}
