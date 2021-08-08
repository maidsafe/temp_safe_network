// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::node::Proposal;
use crate::routing::{
    error::Result,
    peer::PeerUtils,
    routing_api::command::Command,
    section::{NodeStateUtils, SectionLogic, SectionPeersLogic},
    SectionAuthorityProviderUtils,
};
use std::{collections::BTreeSet, iter, net::SocketAddr};
use xor_name::XorName;

impl Core {
    pub(crate) async fn handle_connection_lost(&self, addr: SocketAddr) -> Result<Vec<Command>> {
        if let Some(peer) = self.section.find_joined_member_by_addr(&addr).await {
            debug!(
                "Possible connection loss detected with known peer {:?}",
                peer
            )
        } else if let Some(end_user) = self.get_enduser_by_addr(&addr) {
            debug!(
                "Possible connection loss detected with known client {:?}",
                end_user
            )
        } else {
            debug!("Possible connection loss detected with addr: {:?}", addr);
        }
        Ok(vec![])
    }

    pub(crate) async fn handle_peer_lost(&self, addr: &SocketAddr) -> Result<Vec<Command>> {
        let name = if let Some(peer) = self.section.find_joined_member_by_addr(addr).await {
            debug!("Lost known peer {}", peer);
            *peer.name()
        } else {
            trace!("Lost unknown peer {}", addr);
            return Ok(vec![]);
        };

        if self.is_not_elder() {
            // Adults cannot complain about connectivity.
            return Ok(vec![]);
        }

        let mut commands = self.propose_offline(name).await?;
        commands.push(Command::StartConnectivityTest(name));
        Ok(commands)
    }

    pub(crate) async fn propose_offline(&self, name: XorName) -> Result<Vec<Command>> {
        self.cast_offline_proposals(&iter::once(name).collect())
            .await
    }

    pub(crate) async fn cast_offline_proposals(
        &self,
        names: &BTreeSet<XorName>,
    ) -> Result<Vec<Command>> {
        // Don't send the `Offline` proposal to the peer being lost as that send would fail,
        // triggering a chain of further `Offline` proposals.
        let elders: Vec<_> = self
            .section
            .authority_provider()
            .await
            .peers()
            .filter(|peer| !names.contains(peer.name()))
            .collect();
        let mut result: Vec<Command> = Vec::new();
        for name in names.iter() {
            if let Some(info) = self.section.members().get(name).await {
                let info = info.leave()?;
                if let Ok(commands) = self.send_proposal(&elders, Proposal::Offline(info)).await {
                    result.extend(commands);
                }
            }
        }
        Ok(result)
    }
}
