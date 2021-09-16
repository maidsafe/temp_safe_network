// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use dashmap::DashMap;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

type MsgRegistry = DashMap<SocketAddr, Arc<RwLock<usize>>>;

#[derive(Clone)]
pub(super) struct MsgCount {
    incoming: Arc<MsgRegistry>,
    outgoing: Arc<MsgRegistry>,
}

///
#[derive(Debug)]
pub(super) struct MsgNumbers {
    pub(super) total: usize,
    pub(super) max_node: Option<(SocketAddr, usize)>,
    pub(super) avg: usize,
    pub(super) min_node: Option<(SocketAddr, usize)>,
}

impl MsgCount {
    pub(super) fn new() -> Self {
        Self {
            incoming: Arc::new(MsgRegistry::new()),
            outgoing: Arc::new(MsgRegistry::new()),
        }
    }

    /// Numbers for incoming msgs
    pub(super) async fn incoming(&self) -> MsgNumbers {
        Self::get(&self.incoming).await
    }

    /// Numbers for outgoing msgs
    pub(super) async fn outgoing(&self) -> MsgNumbers {
        Self::get(&self.outgoing).await
    }

    async fn get(registry: &MsgRegistry) -> MsgNumbers {
        let mut total = 0_usize;
        let mut max = 0_usize;
        let mut max_node = None;
        let mut min = usize::MAX;
        let mut min_node = None;

        for (node, value) in registry
            .iter()
            .map(|pair| (*pair.key(), pair.value().clone()))
        {
            let msg_count = *value.read().await;
            if msg_count > max {
                max = msg_count;
                max_node = Some(node);
            }
            if min > msg_count {
                min = msg_count;
                min_node = Some(node);
            }
            total += msg_count;
        }

        let avg = total / usize::max(1, registry.len());

        MsgNumbers {
            total,
            max_node: max_node.map(|sender| (sender, max)),
            avg,
            min_node: min_node.map(|sender| (sender, min)),
        }
    }

    pub(super) async fn increase_incoming(&self, sender: SocketAddr) {
        Self::increase(&self.incoming, sender).await
    }

    pub(super) async fn increase_outgoing(&self, recipient: SocketAddr) {
        Self::increase(&self.outgoing, recipient).await
    }

    async fn increase(registry: &MsgRegistry, node: SocketAddr) {
        match registry.get(&node) {
            Some(pair) => {
                let count = pair.value();
                *count.write().await += 1;
            }
            None => {
                // not perfect racey wise, but acceptable for now
                let _ = registry.insert(node, Arc::new(RwLock::new(1)));
            }
        }
    }
}
