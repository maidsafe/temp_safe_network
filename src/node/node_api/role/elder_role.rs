// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// use crate::node::metadata::Metadata;
use crate::node::metadata::Metadata;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub(crate) struct ElderRole {
    // data operations
    pub(crate) meta_data: Arc<RwLock<Metadata>>,
    // denotes if we received initial sync
    pub(crate) received_initial_sync: Arc<RwLock<bool>>,
}

impl ElderRole {
    pub(crate) fn new(meta_data: Metadata, received_initial_sync: bool) -> Self {
        ElderRole {
            meta_data: Arc::new(RwLock::new(meta_data)),
            received_initial_sync: Arc::new(RwLock::new(received_initial_sync)),
        }
    }
}
