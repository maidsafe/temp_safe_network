use std::collections::BTreeSet;

use crate::comms::{Comm, CommEvent, Error, MsgId, MsgTrait, NetworkMsg, NetworkNode};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum StableSetMsg {
    #[default]
    Ping,
    Pong,
}

impl MsgTrait for StableSetMsg {}
