// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Core;
use crate::messaging::node::NetworkDto;
use crate::routing::section::Section;
use crate::routing::{
    error::Result, network::NetworkLogic, peer::PeerUtils, routing_api::command::Command,
    section::SectionLogic, Event,
};
use std::collections::BTreeSet;

impl Core {
    pub(crate) async fn handle_sync(
        &self,
        section: Section,
        network: &NetworkDto,
    ) -> Result<Vec<Command>> {
        let old_adults: BTreeSet<_> = self
            .section
            .live_adults()
            .await
            .map(|p| *p.name())
            .collect();

        let snapshot = self.state_snapshot().await;
        let auth_provider = section.authority_provider().await;
        trace!(
            "Updating knowledge of own section \n    elders: {:?} \n    members: {:?}",
            auth_provider,
            section.members()
        );
        self.section.merge(section.clone().await).await?;
        self.network
            .get()
            .await
            .merge(network.clone(), self.section.chain_clone().await)
            .await;

        if self.is_not_elder() {
            let current_adults: BTreeSet<_> = self
                .section
                .live_adults()
                .await
                .map(|p| *p.name())
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
