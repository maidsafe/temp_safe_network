// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{Error, Result};
use log::error;
use safe_nd::{Request, XorName};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub(super) enum RpcState {
    // Request sent to chunk holder.
    Sent,
    // Response received from chunk holder.
    Actioned,
    // Holder has left the section without responding.
    HolderGone,
    // Holder hasn't responded within the required time.
    TimedOut,
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
pub(super) struct IDataOp {
    client: XorName,
    request: Request,
    pub rpc_states: BTreeMap<XorName, RpcState>,
}

#[allow(unused)]
impl IDataOp {
    pub(super) fn new(client: XorName, request: Request, holders: Vec<XorName>) -> Result<Self> {
        use Request::*;
        match request {
            PutIData(_) | GetIData(_) | DeleteUnpubIData(_) => (),
            _ => {
                error!("Logic error. Only add Immutable Data requests here.");
                return Err(Error::Logic);
            }
        }

        Ok(Self {
            client,
            request,
            rpc_states: holders
                .into_iter()
                .map(|holder| (holder, RpcState::Sent))
                .collect(),
        })
    }

    pub(super) fn client(&self) -> &XorName {
        &self.client
    }

    pub(super) fn request(&self) -> &Request {
        &self.request
    }

    pub(super) fn is_actioned(&self) -> bool {
        self.rpc_states
            .values()
            .any(|rpc_state| rpc_state == &RpcState::Actioned)
    }
}
