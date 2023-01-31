// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod link;

pub(crate) use link::Link;
pub use link::LinkError;

use sn_interface::{messaging::MsgId, types::Peer};

use qp2p::Endpoint;
use std::{collections::BTreeMap, fmt::Debug, sync::Arc};
use tokio::sync::RwLock;

/// This is tailored to the use-case of connecting on send.
/// It keeps a Link instance per node, and it's designed to make sure
/// underlying I/O connection resources are not leaked, overused or left dangling.
#[derive(Clone, Debug)]
pub(super) struct PeerLinks {
    links: Arc<RwLock<BTreeMap<Peer, Link>>>,
    endpoint: Endpoint,
}

impl PeerLinks {
    pub(super) fn new(endpoint: Endpoint) -> Self {
        Self {
            links: Arc::new(RwLock::new(BTreeMap::new())),
            endpoint,
        }
    }

    /// This method is tailored to the use-case of connecting on send.
    /// I.e. it will not connect here, but on calling send on the returned link.
    pub(super) async fn get_or_create_link(
        &self,
        peer: &Peer,
        connect_now: bool,
        msg_id: Option<MsgId>,
    ) -> Link {
        if let Some(link) = self.get(peer).await {
            if connect_now {
                if let Err(error) = link.create_connection_if_none_exist(msg_id).await {
                    error!(
                        "Error during create connection attempt for link to {peer:?}: {error:?}"
                    );
                }
            }
            return link;
        }

        // if peer is not in list, the entire list needs to be locked
        // i.e. first comms to any node, will impact all sending at that instant..
        // however, first comms should be a minor part of total time spent using link,
        // so that is ok
        let mut links = self.links.write().await;
        match links.get(peer).cloned() {
            // someone else inserted in the meanwhile, so use that
            Some(link) => link,
            // still not in list, go ahead and create + insert
            None => {
                let link = Link::new(*peer, self.endpoint.clone());
                if connect_now {
                    if let Err(error) = link.create_connection_if_none_exist(msg_id).await {
                        error!("Error during create connection attempt for link to {peer:?}: {error:?}");
                    }
                }
                let _ = links.insert(*peer, link.clone());
                link
            }
        }
    }

    async fn get(&self, peer: &Peer) -> Option<Link> {
        let links = self.links.read().await;
        links.get(peer).cloned()
    }

    /// Removes a link from PeerLinks.
    /// It does NOT disconnect it, as it could still be used to receveive messages on
    pub(super) async fn remove_link_from_peer_links(&self, peer: &Peer) {
        debug!("removing link with {peer:?}");
        let mut links = self.links.write().await;
        let _existing_link = links.remove(peer);
    }
}
