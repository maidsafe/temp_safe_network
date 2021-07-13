// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::routing::XorName;
use std::collections::BTreeSet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub(crate) struct AdultsStorageInfo {
    pub(crate) full_adults: Arc<RwLock<BTreeSet<XorName>>>,
}

impl AdultsStorageInfo {
    ///
    pub(crate) fn new() -> Self {
        let full_adults = Arc::new(RwLock::new(BTreeSet::new()));
        Self { full_adults }
    }
}
