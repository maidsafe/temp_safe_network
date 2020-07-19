// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod chunks;

use self::chunks::Chunks;
use crate::{node::node_ops::{AdultDuty, ChunkDuty, NodeDuty, NodeOperation, MessagingDuty}, node::keys::NodeKeys, node::state_db::Init, Config, Result};
use safe_nd::MsgEnvelope;
use std::{
    cell::Cell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

pub(crate) struct AdultDuties {
    keys: NodeKeys,
    chunks: Chunks,
}

impl AdultDuties {
    pub fn new(
        keys: NodeKeys,
        config: &Config,
        total_used_space: &Rc<Cell<u64>>,
        init_mode: Init,
    ) -> Result<Self> {
        let chunks = Chunks::new(keys.clone(), &config, &total_used_space, init_mode)?;
        Ok(Self { keys, chunks })
    }

    pub fn process(&mut self, duty: &AdultDuty) -> Option<NodeOperation> {
        use NodeDuty::*;
        use AdultDuty::*;
        use ChunkDuty::*;
        use NodeOperation::*;
        
        let RunAsChunks(chunk_duty) = duty;
        let result = match chunk_duty {
            ReadChunk(msg)
            | WriteChunk(msg) => self.chunks.receive_msg(msg),
        };
        
        result.map(|c| RunAsNode(ProcessMessaging(c)))
    }
}

impl Display for AdultDuties {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.keys.public_key())
    }
}
