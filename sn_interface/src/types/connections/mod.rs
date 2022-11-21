// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod link;

pub use link::{Link, SendToOneError};

use super::Peer;

use qp2p::Endpoint;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    sync::Arc,
};
use tokio::sync::RwLock;
use xor_name::XorName;

/// This is tailored to the use-case of connecting on send.
/// It keeps a Link instance per node, and is designed to make sure
/// underlying I/O connection resources are not leaked, overused or left dangling.
#[derive(Clone, Debug)]
pub struct PeerLinks {
    links: Arc<RwLock<BTreeMap<Peer, Link>>>,
    endpoint: Endpoint,
}

impl PeerLinks {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            links: Arc::new(RwLock::new(BTreeMap::new())),
            endpoint,
        }
    }

    #[allow(unused)]
    pub async fn linked_peers(&self) -> BTreeSet<Peer> {
        let links = self.links.read().await;

        links.keys().into_iter().cloned().collect()
    }

    #[allow(unused)]
    pub async fn is_connected(&self, peer: &Peer) -> bool {
        let link = self.links.read().await;
        if let Some(c) = link.get(peer) {
            // peer exists, check if connected
            return c.is_connected().await;
        }

        false
    }

    /// This method is tailored to the use-case of connecting on send.
    /// I.e. it will not connect here, but on calling send on the returned link.
    pub async fn get_or_create_link(&self, peer: &Peer, connect_now: bool) -> Link {
        if let Some(link) = self.get(peer).await {
            if connect_now {
                if let Err(error) = link.create_connection_if_none_exist(None).await {
                    error!(
                        "Error during create connection attempt for link to {:?}: {error:?}",
                        peer
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
                    if let Err(error) = link.create_connection_if_none_exist(None).await {
                        error!(
                            "Error during create connection attempt for link to {:?}: {error:?}",
                            peer
                        );
                    }
                }
                let _ = links.insert(*peer, link.clone());
                link
            }
        }
    }

    async fn get(&self, id: &Peer) -> Option<Link> {
        let links = self.links.read().await;
        links.get(id).cloned()
    }

    pub async fn get_peer_by_name(&self, name: &XorName) -> Option<Peer> {
        let links = self.links.read().await;

        for (peer, _link) in links.iter() {
            if peer.name() == *name {
                return Some(*peer);
            }
        }
        None
    }

    /// Removes a link from PeerLinks
    /// Does NOT disconnect it, as it could still be used to receveive
    /// messages on
    pub async fn remove_link_from_peer_links(&self, id: &Peer) {
        debug!("removing linkkk");
        let mut links = self.links.write().await;
        let _existing_link = links.remove(id);
    }
}
