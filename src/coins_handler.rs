// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{utils, vault::Init, Result};
use pickledb::PickleDb;
use safe_nd::NodePublicId;
use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};

const COINS_DB_NAME: &str = "coins.db";

pub(crate) struct CoinsHandler {
    id: NodePublicId,
    // The total safecoin farmed from this section.
    _farmed: PickleDb,
}

impl CoinsHandler {
    pub fn new<P: AsRef<Path>>(id: NodePublicId, root_dir: P, init_mode: Init) -> Result<Self> {
        let _farmed = utils::new_db(root_dir, COINS_DB_NAME, init_mode)?;
        Ok(Self { id, _farmed })
    }
}

impl Display for CoinsHandler {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}
