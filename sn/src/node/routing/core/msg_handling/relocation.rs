// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::elder_count;
use crate::messaging::{
    system::{RelocateDetails, RelocatePromise, SystemMsg},
    AuthorityProof, SectionAuth,
};
use crate::node::{
    error::Result,
    routing::{
        api::command::Command,
        core::{
            bootstrap::JoiningAsRelocated,
            relocation::{self, RelocateAction, RelocateDetailsUtils, RelocateState},
            Core, Proposal,
        },
        Event,
    },
};
use crate::peer::Peer;
use crate::types::log_markers::LogMarker;

use xor_name::XorName;

// Relocation
impl Core {
    pub(crate) async fn relocate_peers(
        &self,
        churn_name: &XorName,
        churn_signature: &bls::Signature,
    ) -> Result<Vec<Command>> {
        let mut commands = vec![];

        // Do not carry out relocation when there is not enough elder nodes.
        if self
            .network_knowledge
            .authority_provider()
            .await
            .elder_count()
            < elder_count()
        {
            return Ok(commands);
        }

        // Consider: Set <= 4, as to not carry out relocations in first 16 sections.
        // TEMP: Do not carry out relocations in the first section
        if self.network_knowledge.prefix().await.bit_count() < 1 {
            return Ok(commands);
        }

        let relocations = relocation::actions(&self.network_knowledge, churn_name, churn_signature);

        for (info, action) in relocations.await {
            // The newly joined node is not being relocated immediately.
            if &info.name() == churn_name {
                continue;
            }

            let peer = info.peer().clone();

            debug!(
                "Relocating {:?} to {} (on churn of {})",
                peer,
                action.dst(),
                churn_name
            );

            commands.extend(
                self.propose(Proposal::Offline(info.relocate(*action.dst())))
                    .await?,
            );

            match action {
                RelocateAction::Instant(details) => {
                    commands.extend(self.send_relocate(peer, details).await?)
                }
                RelocateAction::Delayed(promise) => {
                    commands.extend(self.send_relocate_promise(peer, promise).await?)
                }
            }
        }

        Ok(commands)
    }

    pub(crate) async fn relocate_rejoining_peer(
        &self,
        peer: Peer,
        age: u8,
    ) -> Result<Vec<Command>> {
        let details =
            RelocateDetails::with_age(&self.network_knowledge, &peer, peer.name(), age).await;

        trace!(
            "Relocating {:?} to {} with age {} due to rejoin",
            peer,
            details.dst,
            details.age
        );

        self.send_relocate(peer, details).await
    }

    pub(crate) async fn handle_relocate(
        &self,
        relocate_details: RelocateDetails,
        node_msg: SystemMsg,
        section_auth: AuthorityProof<SectionAuth>,
    ) -> Result<Option<Command>> {
        if relocate_details.pub_id != self.node.read().await.name() {
            // This `Relocate` message is not for us - it's most likely a duplicate of a previous
            // message that we already handled.
            return Ok(None);
        }

        debug!(
            "Received Relocate message to join the section at {}",
            relocate_details.dst
        );

        match *self.relocate_state.read().await {
            Some(RelocateState::InProgress(_)) => {
                trace!("Ignore Relocate - relocation already in progress");
                return Ok(None);
            }
            Some(RelocateState::Delayed(_)) => (),
            None => {
                trace!("{}", LogMarker::RelocateStart);
                self.send_event(Event::RelocationStarted {
                    previous_name: self.node.read().await.name(),
                })
                .await;
            }
        }

        // Create a new instance of JoiningAsRelocated to start the relocation
        // flow. This same instance will handle responses till relocation is complete.
        let genesis_key = *self.network_knowledge.genesis_key();

        let bootstrap_addrs = if let Ok(sap) = self
            .network_knowledge
            .section_by_name(&relocate_details.dst)
        {
            sap.addresses()
        } else {
            self.network_knowledge
                .authority_provider()
                .await
                .addresses()
        };
        let mut joining_as_relocated = JoiningAsRelocated::new(
            self.node.read().await.clone(),
            genesis_key,
            relocate_details,
            node_msg,
            section_auth,
        )?;

        let cmd = joining_as_relocated.start(bootstrap_addrs)?;

        *self.relocate_state.write().await =
            Some(RelocateState::InProgress(Box::new(joining_as_relocated)));

        Ok(Some(cmd))
    }

    pub(crate) async fn handle_relocate_promise(
        &self,
        promise: RelocatePromise,
        msg: SystemMsg,
    ) -> Result<Vec<Command>> {
        // Check if we need to filter out the `RelocatePromise`.
        if promise.name == self.node.read().await.name() {
            // Promise to relocate us.
            if self.relocate_state.read().await.is_some() {
                // Already received a promise or already relocating. discard.
                return Ok(vec![]);
            }
        } else {
            // Promise returned from a node to be relocated, to be exchanged for the actual
            // `Relocate` message.
            if self.is_not_elder().await || self.network_knowledge.is_elder(&promise.name).await {
                // If we are not elder, maybe we just haven't processed our promotion yet.
                // If otherwise they are still elder, maybe we just haven't processed their demotion yet.
                return Ok(vec![]);
            }
        }

        let mut commands = vec![];

        if promise.name == self.node.read().await.name() {
            // Store the `RelocatePromise` message and send it back after we are demoted.
            // Keep it around even if we are not elder anymore, in case we need to resend it.
            match *self.relocate_state.read().await {
                None => {
                    trace!("Received RelocatePromise to section at {}", promise.dst);
                    *self.relocate_state.write().await = Some(RelocateState::Delayed(msg.clone()));
                    self.send_event(Event::RelocationStarted {
                        previous_name: self.node.read().await.name(),
                    })
                    .await;
                }
                Some(RelocateState::InProgress(_)) => {
                    trace!("ignore RelocatePromise - relocation already in progress");
                }
                Some(RelocateState::Delayed(_)) => {
                    trace!("ignore RelocatePromise - already have one");
                }
            }

            // We are no longer elder. Send the promise back already.
            if self.is_not_elder().await {
                commands.push(self.send_message_to_our_elders(msg).await?);
            }

            return Ok(commands);
        }

        if self.network_knowledge.is_elder(&promise.name).await {
            error!(
                "ignore returned RelocatePromise from {} - node is still elder",
                promise.name
            );
            return Ok(commands);
        }

        if let Some(info) = self.network_knowledge.get_section_member(&promise.name) {
            let peer = info.peer();
            let details = RelocateDetails::new(&self.network_knowledge, peer, promise.dst).await;
            commands.extend(self.send_relocate(peer.clone(), details).await?);
        } else {
            error!(
                "ignore returned RelocatePromise from {} - unknown node",
                promise.name
            );
        }

        Ok(commands)
    }
}
