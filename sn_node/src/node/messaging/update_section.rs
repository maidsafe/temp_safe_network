// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::integration::{Cmd, Peers};
use crate::node::MyNode;

use sn_interface::{
    data_copy_count,
    messaging::system::{NodeCmd, NodeMsg},
    types::{log_markers::LogMarker, DataAddress, Peer},
};

use itertools::Itertools;
use std::collections::BTreeSet;

impl MyNode {
    /// Given what data the peer has, we shall calculate what data the peer is missing that
    /// we have, and send such data to the peer.
    #[instrument(skip(self, currently_at_peer))]
    pub(crate) fn get_missing_data_for_node(
        &self,
        peer: Peer,
        currently_at_peer: Vec<DataAddress>,
    ) -> Cmd {
        let adults = self.network_knowledge.adults();
        let other_peers = adults.iter().map(|p2p_node| p2p_node.name()).collect();

        let cmd = crate::data::Cmd::EnqueueReplicationJob {
            peer,
            currently_at_peer,
            other_peers,
        };

        Cmd::Data(cmd)
    }

    /// If we are an adult, this will send a list of currently
    /// known/owned data to relevant nodes.
    /// These nodes should send back anything missing (in batches).
    ///
    /// This below comment seems not yet implemented:
    /// Relevant nodes should be all _prior_ neighbours + _new_ elders.
    #[instrument(skip(self))]
    pub(crate) fn ask_peers_for_data(&self, currently_held_data: Vec<DataAddress>) -> Cmd {
        trace!("{:?}", LogMarker::DataReorganisationUnderway);
        let adults = self.network_knowledge.adults();
        let elders = self.network_knowledge.elders();
        let my_name = self.name();

        // find data targets that are not us.
        let mut peers = adults
            .into_iter()
            .sorted_by(|lhs, rhs| my_name.cmp_distance(&lhs.name(), &rhs.name()))
            .filter(|peer| peer.name() != my_name)
            .take(data_copy_count())
            .collect::<BTreeSet<_>>();

        trace!(
            "nearest neighbours for data req: {}: {:?}",
            peers.len(),
            peers
        );

        // also send to our elders in case they are holding but were just promoted
        for elder in elders {
            let _existed = peers.insert(elder);
        }

        if peers.is_empty() {
            warn!("We have no peers to ask for data!");
        } else {
            trace!("Sending our data list to: {:?}", peers);
        }

        let msg = NodeMsg::NodeCmd(NodeCmd::ReturnMissingData(currently_held_data));
        self.send_system_msg(msg, Peers::Multiple(peers))
    }
}
