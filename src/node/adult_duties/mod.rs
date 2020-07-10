// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod chunks;

use self::chunks::Chunks;
use crate::{cmd::NodeCmd, node::Init, Config, Result};
use routing::{Node as Routing, SrcLocation};
use safe_nd::{NodePublicId, MsgEnvelope};
use std::{
    cell::{Cell, RefCell},
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(crate) struct AdultDuties {
    id: NodePublicId,
    chunks: Chunks,
    _routing: Rc<RefCell<Routing>>,
}

impl AdultDuties {
    pub fn new(
        id: NodePublicId,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
        routing: Rc<RefCell<Routing>>,
    ) -> Result<Self> {
        let chunks = Chunks::new(
            id.clone(),
            &config,
            &total_used_space,
            init_mode,
            routing.clone(),
        )?;
        Ok(Self {
            id,
            chunks,
            _routing: routing,
        })
    }

    pub fn receive_msg(
        &mut self,
        msg: MsgEnvelope,
    ) -> Option<NodeCmd> {
        self.chunks.receive_msg(msg)
    }
}

impl Display for AdultDuties {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.id.name())
    }
}
