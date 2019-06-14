// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use quic_p2p::Peer;
use safe_nd::{ClientPublicId, Requester};
use std::net::SocketAddr;

#[allow(clippy::large_enum_variant)]
pub(crate) enum Action {
    ClientRequest {
        client_id: ClientPublicId,
        msg: Vec<u8>,
    },
}
