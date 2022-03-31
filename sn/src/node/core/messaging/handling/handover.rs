// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{api::cmds::Cmd, core::Node, handover::SapCandidate};
use sn_consensus::{SignedVote, VoteResponse};

impl Node {
    pub(crate) async fn handle_handover_msg(&self, msg: SignedVote<SapCandidate>) -> Vec<Cmd> {
        vec![]
    }
}
