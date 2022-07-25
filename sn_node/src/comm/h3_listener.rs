// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::MsgEvent;
use crate::node::error::Result;
use sn_interface::{messaging::WireMsg, types::Peer};

use bytes::{Buf, Bytes};
use h3::server::{Connection, RequestStream};
use h3_quinn::{
    quinn::{Connecting as QuinnConnecting, Endpoint as QuinnEndpoint, Incoming},
    Connection as H3QuinnConnection,
};
use http::Request;
use rustls::{Certificate, PrivateKey};
use std::{net::SocketAddr, sync::Arc};
use tokio::{sync::mpsc, task};
use tracing::{error, info, warn, Instrument};

pub(crate) enum ListenerH3Event {
    Connected {
        peer: Peer,
        //connection: H3QuinnConnection,
        stream: RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
    },
}

#[derive(Clone)]
pub(crate) struct H3MsgListener {
    add_connection: mpsc::Sender<ListenerH3Event>,
    receive_msg: mpsc::Sender<MsgEvent>,
}

impl H3MsgListener {
    pub(crate) fn new(
        add_connection: mpsc::Sender<ListenerH3Event>,
        receive_msg: mpsc::Sender<MsgEvent>,
    ) -> Self {
        Self {
            add_connection,
            receive_msg,
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn new_endpoint(local_addr: SocketAddr) -> Result<(QuinnEndpoint, Incoming)> {
        let crypto = load_crypto().await.unwrap();
        let server_config = h3_quinn::quinn::ServerConfig::with_crypto(Arc::new(crypto));

        match QuinnEndpoint::server(server_config, local_addr) {
            Ok((endpoint, incoming)) => {
                info!(">>>>> H3: Listening on {}", local_addr);
                Ok((endpoint, incoming))
            }
            Err(err) => {
                error!(
                    ">>>>> H3: Failed to start listening on {}: {:?}",
                    local_addr, err
                );
                Err(err.into())
            }
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn listen(&self, conn: QuinnConnecting) {
        let clone = self.clone();
        let _ = task::spawn_local(clone.listen_internal(conn).in_current_span());
    }

    #[tracing::instrument(skip_all)]
    async fn listen_internal(self, conn: QuinnConnecting) {
        let remote_address = conn.remote_address();
        info!(
            ">>>>> H3: New connection being attempted from {}",
            remote_address
        );

        let receive_msg = self.receive_msg.clone();
        match conn.await {
            Ok(conn) => {
                info!(">>>>> H3: New connection now established");

                let mut h3_conn = Connection::new(H3QuinnConnection::new(conn)).await.unwrap();

                while let Ok(Some((req, stream))) = h3_conn.accept().await {
                    info!(">>>>> H3: New request: {:#?}", req);

                    let receive_msg = receive_msg.clone();
                    let add_connection = self.add_connection.clone();
                    let _ = task::spawn_local(async move {
                        match handle_request(req, stream, remote_address, receive_msg.clone()).await
                        {
                            Err(e) => error!(">>>>> H3: request failed: {}", e),
                            Ok(event) => {
                                let _ = add_connection.send(event).await;
                            }
                        }
                    });
                }
            }
            Err(err) => {
                warn!(">>>>> H3: accepting connection failed: {:?}", err);
            }
        }
    }
}

async fn handle_request(
    req: Request<()>,
    mut stream: RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
    remote_address: SocketAddr,
    receive_msg: mpsc::Sender<MsgEvent>,
) -> Result<ListenerH3Event, Box<dyn std::error::Error>> {
    // Read request stream to extract the ServiceMessage
    let mut bytes = Vec::new();
    while let Ok(Some(mut data)) = stream.recv_data().await {
        info!(
            ">>>>> H3: Received request data: {} bytes",
            data.remaining()
        );
        while data.remaining() > 0 {
            let n = bytes.len();
            for value in data.chunk() {
                bytes.push(*value);
            }
            info!(
                ">>>>> H3: Reading chunks, total number of bytes read so far: {}",
                bytes.len()
            );
            let delta = bytes.len() - n;
            data.advance(delta);
        }
    }

    info!(
        ">>>>> H3: Finished reading chunks, total number of bytes read: {}",
        bytes.len()
    );

    // Deserialise bytes read from stream to obtain the ServiceMessage
    let msg_bytes = Bytes::copy_from_slice(bytes.as_slice());
    match WireMsg::from(msg_bytes.clone()) {
        Err(err) => {
            error!(">>>>> H3: Failed to deserialise request data: {:?}", err);
            Err(Box::new(err))
        }
        Ok(wire_msg) => {
            let msg_id = wire_msg.msg_id();
            info!(
                ">>>>> H3: WireMsg deserialised from request data: {:?}",
                msg_id
            );
            let src_name = wire_msg.auth().src().name();
            let _send_res = receive_msg
                .send(MsgEvent::Received {
                    sender: Peer::new(src_name, remote_address),
                    wire_msg,
                    original_bytes: msg_bytes,
                })
                .await;

            Ok(ListenerH3Event::Connected {
                peer: Peer::new(src_name, remote_address),
                stream,
            })
        }
    }
}

// static ALPN: &[u8] = b"h3";
static ALPN: &[&[u8]] = &[b"h3", b"h3-29", b"h3-28", b"h3-27"];

async fn load_crypto() -> Result<rustls::ServerConfig, Box<dyn std::error::Error>> {
    let (cert, key) = build_certs();

    let mut crypto = rustls::ServerConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;
    crypto.max_early_data_size = u32::MAX;
    crypto.alpn_protocols = ALPN.iter().map(|&x| x.into()).collect();
    // crypto.alpn_protocols = vec![ALPN.into()];

    Ok(crypto)
}

fn build_certs() -> (Certificate, PrivateKey) {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let key = PrivateKey(cert.serialize_private_key_der());
    let cert = Certificate(cert.serialize_der().unwrap());
    (cert, key)
}
