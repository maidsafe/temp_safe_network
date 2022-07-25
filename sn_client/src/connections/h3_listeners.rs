// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Session;

use sn_interface::{messaging::WireMsg, types::Peer};

use bytes::{Buf, Bytes};

impl Session {
    pub(crate) fn spawn_h3_msg_listener_thread(
        session: Session,
        peer: Peer,
        mut stream: h3::client::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
        //send_request: h3::client::SendRequest<h3_quinn::OpenStreams, Bytes>,
        //client_endpoint: h3_quinn::Endpoint
    ) {
        let _handle = tokio::spawn(async move {
            // Read request stream to extract the ServiceMessage
            let mut bytes = Vec::new();
            while let Ok(Some(mut data)) = stream.recv_data().await {
                info!(
                    ">>>>> H3: Received response data: {} bytes",
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
            if !bytes.is_empty() {
                let msg_bytes = Bytes::copy_from_slice(bytes.as_slice());
                match WireMsg::deserialize(msg_bytes) {
                    Err(err) => {
                        error!(">>>>> H3: Failed to deserialise response data: {:?}", err);
                    }
                    Ok(msg_type) => {
                        info!(">>>>> H3: WireMsg deserialised response: {:?}", msg_type);

                        let _handle = tokio::spawn(async move {
                            if let Err(err) = Self::handle_msg(msg_type, peer, session).await {
                                error!("Error while handling incoming H3 msg: {:?}", err);
                            }
                        });
                    }
                }
            }
            //client_endpoint.wait_idle().await;
            //drop(send_request);
        });
    }
}
