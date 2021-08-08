// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::routing::{
    dkg::{
        dkg_msgs_utils::{DkgFailChecker, DkgFailureSigSetUtils, DkgFailureSigUtils},
        SectionDkgOutcome,
    },
    ed25519,
    error::Result,
    messages::WireMsgUtils,
    node::Node,
    routing_api::command::{next_timer_token, Command},
    SectionAuthorityProviderUtils,
};
use crate::{
    messaging::{
        node::{DkgFailureSig, DkgKey, ElderCandidates, NodeMsg},
        DstLocation, SectionAuthorityProvider, WireMsg,
    },
    types::CFValue,
};
use async_recursion::async_recursion;
use bls::PublicKey as BlsPublicKey;
use bls_dkg::key_gen::{message::Message as DkgMessage, KeyGen};
use itertools::Itertools;
use rand::rngs::OsRng;
use std::{
    collections::{BTreeSet, VecDeque},
    iter,
    net::SocketAddr,
    time::Duration,
};
use tokio::sync::RwLock;
use xor_name::XorName;

// Interval to progress DKG timed phase
const DKG_PROGRESS_INTERVAL: Duration = Duration::from_secs(30);

const BACKLOG_CAPACITY: usize = 100;

// Data for a DKG participant.
pub(crate) struct Session {
    pub(crate) elder_candidates: ElderCandidates,
    pub(crate) participant_index: usize,
    pub(crate) key_gen: RwLock<KeyGen>,
    pub(crate) timer_token: CFValue<u64>,
    pub(crate) failures: RwLock<DkgFailChecker>,
    // Flag to track whether this session has completed (either with success or failure). We don't
    // remove complete sessions because the other participants might still need us to respond to
    // their messages.
    pub(crate) complete: CFValue<bool>,
}

impl Session {
    pub(crate) async fn timer_token(&self) -> u64 {
        *self.timer_token.get().await
    }

    #[async_recursion]
    pub(crate) async fn process_message(
        &self,
        node: &Node,
        dkg_key: &DkgKey,
        message: DkgMessage,
        section_pk: BlsPublicKey,
    ) -> Result<Vec<Command>> {
        trace!("process DKG message {:?}", message);
        let responses = self
            .key_gen
            .write()
            .await
            .handle_message(&mut rand::thread_rng(), message)
            .unwrap_or_default();

        // Only a valid DkgMessage, which results in some responses, shall reset the ticker.
        let add_reset_timer = !responses.is_empty();

        let mut commands = vec![];
        for response in responses.into_iter() {
            commands.extend(self.broadcast(node, dkg_key, response, section_pk).await?);
        }
        if add_reset_timer {
            commands.push(self.reset_timer().await);
        }
        commands.extend(self.try_get_dkg_outcome(node, dkg_key, section_pk).await?);
        Ok(commands)
    }

    fn recipients(&self) -> Vec<(XorName, SocketAddr)> {
        self.elder_candidates
            .elders
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != self.participant_index)
            .map(|(_, (name, addr))| (*name, *addr))
            .collect()
    }

    pub(crate) async fn broadcast(
        &self,
        node: &Node,
        dkg_key: &DkgKey,
        message: DkgMessage,
        section_pk: BlsPublicKey,
    ) -> Result<Vec<Command>> {
        let mut commands = vec![];

        let recipients = self.recipients();
        if !recipients.is_empty() {
            trace!("broadcasting DKG message {:?} to {:?}", message, recipients);
            let node_msg = NodeMsg::DkgMessage {
                dkg_key: *dkg_key,
                message: message.clone(),
            };
            let wire_msg = WireMsg::single_src(
                node,
                DstLocation::DirectAndUnrouted(section_pk),
                node_msg,
                section_pk,
            )?;

            commands.push(Command::SendMessage {
                recipients,
                wire_msg,
            });
        }

        commands.extend(
            self.process_message(node, dkg_key, message, section_pk)
                .await?,
        );
        Ok(commands)
    }

    pub(crate) async fn handle_timeout(
        &self,
        node: &Node,
        dkg_key: &DkgKey,
        section_pk: BlsPublicKey,
    ) -> Result<Vec<Command>> {
        if *self.complete.get().await {
            return Ok(vec![]);
        }

        trace!("DKG for {:?} progressing", self.elder_candidates);

        match self
            .key_gen
            .write()
            .await
            .timed_phase_transition(&mut OsRng)
        {
            Ok(messages) => {
                let mut commands = vec![];
                for message in messages.into_iter() {
                    commands.extend(self.broadcast(node, dkg_key, message, section_pk).await?);
                }
                commands.push(self.reset_timer().await);
                commands.extend(self.try_get_dkg_outcome(node, dkg_key, section_pk).await?);
                Ok(commands)
            }
            Err(error) => {
                trace!("DKG for {:?} failed: {}", self.elder_candidates, error);
                self.report_failure(node, dkg_key, BTreeSet::new(), section_pk)
                    .await
            }
        }
    }

    // Check whether a key generator is finalized to give a DKG outcome.
    async fn try_get_dkg_outcome(
        &self,
        node: &Node,
        dkg_key: &DkgKey,
        section_pk: BlsPublicKey,
    ) -> Result<Vec<Command>> {
        if *self.complete.get().await {
            return Ok(vec![]);
        }

        if !self.key_gen.read().await.is_finalized() {
            return Ok(vec![]);
        }

        let (participants, outcome) = if let Some(tuple) = self.key_gen.read().await.generate_keys()
        {
            tuple
        } else {
            return Ok(vec![]);
        };

        // Less than 100% participation
        if !participants.iter().eq(self.elder_candidates.elders.keys()) {
            trace!(
                "DKG for {:?} failed: unexpected participants: {:?}",
                self.elder_candidates,
                participants.iter().format(", ")
            );

            let failed_participants: BTreeSet<_> = self
                .elder_candidates
                .elders
                .keys()
                .filter_map(|elder| {
                    if !participants.contains(elder) {
                        Some(*elder)
                    } else {
                        None
                    }
                })
                .collect();

            return self
                .report_failure(node, dkg_key, failed_participants, section_pk)
                .await;
        }

        // Corrupted DKG outcome. This can happen when a DKG session is restarted using the same set
        // of participants and the same generation, but some of the participants are unaware of the
        // restart (due to lag, etc...) and keep sending messages for the original session which
        // then get mixed with the messages of the restarted session.
        if outcome
            .public_key_set
            .public_key_share(self.participant_index)
            != outcome.secret_key_share.public_key_share()
        {
            trace!(
                "DKG for {:?} failed: corrupted outcome",
                self.elder_candidates
            );
            return self
                .report_failure(node, dkg_key, BTreeSet::new(), section_pk)
                .await;
        }

        trace!(
            "DKG for {:?} complete: {:?}",
            self.elder_candidates,
            outcome.public_key_set.public_key()
        );

        self.complete.set(true).await;
        let section_auth = SectionAuthorityProvider::from_elder_candidates(
            self.elder_candidates.clone(),
            outcome.public_key_set.clone(),
        );

        let outcome = SectionDkgOutcome {
            public_key_set: outcome.public_key_set,
            index: self.participant_index,
            secret_key_share: outcome.secret_key_share,
        };

        Ok(vec![Command::HandleDkgOutcome {
            section_auth,
            outcome,
        }])
    }

    async fn report_failure(
        &self,
        node: &Node,
        dkg_key: &DkgKey,
        failed_participants: BTreeSet<XorName>,
        section_pk: BlsPublicKey,
    ) -> Result<Vec<Command>> {
        let sig = DkgFailureSig::new(&node.keypair, &failed_participants, dkg_key);

        if !self
            .failures
            .write()
            .await
            .insert(sig, &failed_participants)
        {
            return Ok(vec![]);
        }

        let cmds = self
            .check_failure_agreement()
            .await
            .into_iter()
            .chain(iter::once({
                let node_msg = NodeMsg::DkgFailureObservation {
                    dkg_key: *dkg_key,
                    sig,
                    failed_participants,
                };
                let wire_msg = WireMsg::single_src(
                    node,
                    DstLocation::DirectAndUnrouted(section_pk),
                    node_msg,
                    section_pk,
                )?;

                Command::SendMessage {
                    recipients: self.recipients(),
                    wire_msg,
                }
            }))
            .collect();

        Ok(cmds)
    }

    pub(crate) async fn process_failure(
        &self,
        dkg_key: &DkgKey,
        failed_participants: &BTreeSet<XorName>,
        signed: DkgFailureSig,
    ) -> Option<Command> {
        if !self
            .elder_candidates
            .elders
            .contains_key(&ed25519::name(&signed.public_key))
        {
            return None;
        }

        if !signed.verify(dkg_key, failed_participants) {
            return None;
        }

        if !self
            .failures
            .write()
            .await
            .insert(signed, failed_participants)
        {
            return None;
        }

        self.check_failure_agreement().await
    }

    async fn check_failure_agreement(&self) -> Option<Command> {
        if self
            .failures
            .read()
            .await
            .has_agreement(&self.elder_candidates)
        {
            self.complete.set(true).await;
            let set_a = self.failures.read().await.dto();
            *self.failures.write().await = DkgFailChecker::new();
            Some(Command::HandleDkgFailure(set_a))
        } else {
            None
        }
    }

    async fn reset_timer(&self) -> Command {
        self.timer_token.set(next_timer_token()).await;
        Command::ScheduleTimeout {
            duration: DKG_PROGRESS_INTERVAL,
            token: self.timer_token.clone().await,
        }
    }
}

pub(crate) struct Backlog(VecDeque<(DkgKey, DkgMessage)>);

impl Backlog {
    pub(crate) fn new() -> Self {
        Self(VecDeque::with_capacity(BACKLOG_CAPACITY))
    }

    pub(crate) fn push(&mut self, dkg_key: DkgKey, message: DkgMessage) {
        if self.0.len() == self.0.capacity() {
            let _ = self.0.pop_front();
        }

        self.0.push_back((dkg_key, message))
    }

    pub(crate) fn take(&mut self, dkg_key: &DkgKey) -> Vec<DkgMessage> {
        let mut output = Vec::new();
        let max = self.0.len();

        for _ in 0..max {
            if let Some((message_dkg_key, message)) = self.0.pop_front() {
                if &message_dkg_key == dkg_key {
                    output.push(message)
                } else {
                    self.0.push_back((message_dkg_key, message))
                }
            }
        }

        output
    }

    pub(crate) fn prune(&mut self, dkg_key: &DkgKey) {
        self.0
            .retain(|(old_dkg_key, _)| old_dkg_key.generation >= dkg_key.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::MessageType;
    use crate::routing::{
        dkg::voter::DkgVoter, dkg::DkgKeyUtils, ed25519, node::test_utils::arbitrary_unique_nodes,
        node::Node, section::section_authority_provider::ElderCandidatesUtils,
        section::test_utils::gen_addr, ELDER_SIZE, MIN_ADULT_AGE,
    };
    use assert_matches::assert_matches;
    use eyre::{bail, ContextCompat, Result};
    use proptest::prelude::*;
    use rand::{rngs::SmallRng, SeedableRng};
    use std::{collections::HashMap, iter};
    use xor_name::Prefix;

    #[tokio::test]
    async fn single_participant() -> Result<()> {
        // If there is only one participant, the DKG should complete immediately.

        let voter = DkgVoter::default();
        let section_pk = bls::SecretKey::random().public_key();

        let node = Node::new(
            ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE),
            gen_addr(),
        );
        let elder_candidates = ElderCandidates::new(iter::once(node.peer()), Prefix::default());
        let dkg_key = DkgKey::new(&elder_candidates, 0);

        let commands = voter
            .start(&node, dkg_key, elder_candidates, section_pk)
            .await?;
        assert_matches!(&commands[..], &[Command::HandleDkgOutcome { .. }]);

        Ok(())
    }

    proptest! {
        // Run a DKG session where every participant handles every message sent to them.
        // Expect the session to successfully complete without timed transitions.
        // NOTE: `seed` is for seeding the rng that randomizes the message order.
        #[test]
        fn proptest_full_participation(nodes in arbitrary_elder_nodes(), seed in any::<u64>()) {
            prop_assert!(proptest_full_participation_impl(nodes, seed).is_ok());
        }
    }

    fn proptest_full_participation_impl(nodes: Vec<Node>, seed: u64) -> Result<()> {
        // Rng used to randomize the message order.
        let mut rng = SmallRng::seed_from_u64(seed);
        let section_pk = bls::SecretKey::random().public_key();
        let mut messages = Vec::new();

        let elder_candidates =
            ElderCandidates::new(nodes.iter().map(Node::peer), Prefix::default());
        let dkg_key = DkgKey::new(&elder_candidates, 0);

        let mut actors: HashMap<_, _> = nodes
            .into_iter()
            .map(|node| (node.addr, Actor::new(node)))
            .collect();

        use futures::executor::block_on as block;
        for actor in actors.values_mut() {
            let commands = block(actor.voter.start(
                &actor.node,
                dkg_key,
                elder_candidates.clone(),
                section_pk,
            ))?;

            for command in commands {
                messages.extend(actor.handle(command, &dkg_key)?)
            }
        }

        loop {
            match actors
                .values()
                .filter_map(|actor| actor.outcome.as_ref())
                .unique()
                .count()
            {
                0 => {}
                1 => return Ok(()),
                _ => bail!("Inconsistent DKG outcomes"),
            }

            // NOTE: this panics if `messages` is empty, but that's OK because it would mean
            // failure anyway.
            let index = rng.gen_range(0, messages.len());
            let (addr, message) = messages.swap_remove(index);

            let actor = actors.get_mut(&addr).context("Unknown message recipient")?;
            let commands = block(actor.voter.process_message(
                &actor.node,
                &dkg_key,
                message,
                section_pk,
            ))?;

            for command in commands {
                messages.extend(actor.handle(command, &dkg_key)?)
            }
        }
    }

    struct Actor {
        node: Node,
        voter: DkgVoter,
        outcome: Option<bls::PublicKey>,
    }

    impl Actor {
        fn new(node: Node) -> Self {
            Self {
                node,
                voter: DkgVoter::default(),
                outcome: None,
            }
        }

        fn handle(
            &mut self,
            command: Command,
            expected_dkg_key: &DkgKey,
        ) -> Result<Vec<(SocketAddr, DkgMessage)>> {
            match command {
                Command::SendMessage {
                    recipients,
                    wire_msg,
                } => match wire_msg.into_message()? {
                    MessageType::Node {
                        msg: NodeMsg::DkgMessage { dkg_key, message },
                        ..
                    } => {
                        assert_eq!(dkg_key, *expected_dkg_key);
                        Ok(recipients
                            .into_iter()
                            .map(|addr| (addr.1, message.clone()))
                            .collect())
                    }
                    other_message => bail!("Unexpected message: {:?}", other_message),
                },
                Command::HandleDkgOutcome { outcome, .. } => {
                    self.outcome = Some(outcome.public_key_set.public_key());
                    Ok(vec![])
                }
                Command::ScheduleTimeout { .. } => Ok(vec![]),
                other_command => {
                    bail!("Unexpected command: {:?}", other_command)
                }
            }
        }
    }

    fn arbitrary_elder_nodes() -> impl Strategy<Value = Vec<Node>> {
        arbitrary_unique_nodes(2..=ELDER_SIZE)
    }
}
