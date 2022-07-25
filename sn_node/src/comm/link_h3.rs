// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::H3MsgListener;

use sn_interface::types::Peer;

use bytes::Bytes;
use h3_quinn::quinn::Endpoint;

/// A link to a peer in our network.
///
/// The upper layers will add incoming streams to the link,
/// and use the link to send msgs.
/// Using the link will open a connection if there is none there.
/// The link is a way to keep streams to a peer in one place
/// and use them efficiently; converge to a single one regardless of concurrent
/// comms initiation between the peers, and so on.
/// Unused streams will expire, so the LinkH3 is cheap to keep around.
/// The LinkH3 is kept around as long as the peer is deemed worth to keep contact with.
pub(crate) struct LinkH3 {
    peer: Peer,
    endpoint: Endpoint,
    streams: Vec<h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>>,
    listener: H3MsgListener,
}

impl LinkH3 {
    pub(crate) fn new(peer: Peer, endpoint: Endpoint, listener: H3MsgListener) -> Self {
        Self {
            peer,
            endpoint,
            streams: Vec::new(),
            listener,
        }
    }

    pub(crate) async fn new_with(
        peer: Peer,
        endpoint: Endpoint,
        listener: H3MsgListener,
        stream: h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
    ) -> Self {
        let mut instance = Self::new(peer, endpoint, listener);
        instance.insert(stream);
        instance
    }

    #[cfg(feature = "test-utils")]
    pub(crate) fn peer(&self) -> &Peer {
        &self.peer
    }

    pub(crate) async fn add(
        &mut self,
        stream: h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>,
    ) {
        self.insert(stream);
    }

    /// Disposes of the link and all underlying resources.
    /// Also any clones of this link that are held, will be cleaned up.
    /// This is due to the fact that we do never leak the `h3_quinn::Connection` outside of this struct,
    /// since that struct is cloneable and uses Arc internally.
    pub(crate) async fn disconnect(&mut self) {
        for stream in self.streams.iter_mut() {
            if let Err(err) = stream.finish().await {
                error!("Failed when trying ot finish a client H3 stream: {:?}", err);
            }
        }

        self.streams.clear();
    }

    /// Send a message to the peer with default retry configuration.
    #[instrument(skip_all)]
    pub(crate) async fn send(&mut self, msg: Bytes) -> Result<(), SendToOneError> {
        let stream = self.get_stream().await?;
        trace!(">>>>> H3: We have open streams to node");

        match stream.send_data(msg).await {
            Ok(()) => {
                info!(">>>>> H3: Response on stream sent successfully");
            }
            Err(err) => {
                error!(
                    ">>>>> H3: Unable to send response on stream to connection peer: {:?}",
                    err
                );
                return Err(SendToOneError::Send(err));
            }
        }

        let resp = http::Response::builder()
            .status(http::StatusCode::OK)
            .body(())
            .map_err(|err| {
                error!(
                    ">>>>> H3: Unable to build response to connection peer: {:?}",
                    err
                );

                SendToOneError::Http(err)
            })?;

        match stream.send_response(resp).await {
            Ok(()) => {
                info!(">>>>> H3: Response to connection successful");
                if let Err(err) = stream.finish().await {
                    error!(
                        "Failed when trying ot finish the client H3 stream: {:?}",
                        err
                    );
                }
                Ok(())
            }
            Err(err) => {
                error!(
                    ">>>>> H3: Unable to send response to connection peer: {:?}",
                    err
                );
                Err(SendToOneError::Send(err))
            }
        }
    }

    /// Is this LinkH3 currently connected?
    #[allow(unused)]
    pub(crate) fn is_connected(&self) -> bool {
        !self.streams.is_empty()
    }

    async fn get_stream(
        &mut self,
    ) -> Result<&mut h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>, SendToOneError>
    {
        match self.streams.iter_mut().next() {
            Some(stream) => Ok(stream),
            None => {
                error!(">>>>> H3: cannot find stream");
                Err(SendToOneError::StreamNotFound)
            }
        }
    }

    fn insert(&mut self, stream: h3::server::RequestStream<h3_quinn::BidiStream<Bytes>, Bytes>) {
        self.streams.push(stream);
    }
}

/// Errors that can be returned from `Comm::send_to_one`.
#[derive(Debug)]
pub(crate) enum SendToOneError {
    ///
    Http(http::Error),
    ///
    StreamNotFound,
    ///
    Send(h3::Error),
}
