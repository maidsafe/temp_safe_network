// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::node::{Network, Section};
use crate::routing::{
    error::Result, network::NetworkUtils, peer::PeerUtils, routing_api::command::Command,
    section::SectionUtils, Event,
};
use std::collections::BTreeSet;

impl Core {
    pub(crate) async fn handle_sync(
        &mut self,
        section: &Section,
        network: &Network,
    ) -> Result<Vec<Command>> {
        let old_adults: BTreeSet<_> = self
            .section
            .live_adults()
            .map(|p| p.name())
            .copied()
            .collect();

        let snapshot = self.state_snapshot();
        trace!(
            "Updating knowledge of own section \n    elders: {:?} \n    members: {:?}",
            section.authority_provider(),
            section.members()
        );
        self.section.merge(section.clone())?;
        self.network.merge(network.clone(), self.section.chain());

        if self.is_not_elder() {
            let current_adults: BTreeSet<_> = self
                .section
                .live_adults()
                .map(|p| p.name())
                .copied()
                .collect();
            let added: BTreeSet<_> = current_adults.difference(&old_adults).copied().collect();
            let removed: BTreeSet<_> = old_adults.difference(&current_adults).copied().collect();

            if !added.is_empty() || !removed.is_empty() {
                self.send_event(Event::AdultsChanged {
                    remaining: old_adults.intersection(&current_adults).copied().collect(),
                    added,
                    removed,
                })
                .await;
            }
        }

        self.update_state(snapshot).await
    }
}
