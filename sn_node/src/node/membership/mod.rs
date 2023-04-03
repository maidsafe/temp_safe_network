// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.
use bls::{PublicKeySet, SecretKeyShare};
use core::fmt::Debug;
use sn_consensus::mvba::{
    bundle::{Bundle, Outgoing},
    consensus::Consensus,
    tag::{Domain, Tag},
    Decision, NodeId,
};
use sn_interface::{
    messaging::system::DkgSessionId,
    network_knowledge::{
        node_state::NodeState, partition_by_prefix, recommended_section_size, MembershipState,
        SectionAuthorityProvider,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use std::{sync::Mutex, time::Instant};
use thiserror::Error;
use xor_name::{Prefix, XorName};

pub(crate) type Generation = u64;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Consensus error {0}")]
    Consensus(#[from] sn_consensus::mvba::error::Error),
    #[error("We are behind the voter, caller should request anti-entropy")]
    RequestAntiEntropy, // TODO: we can remove it
    #[error("Invalid proposal")]
    InvalidProposal,
    #[error("Invalid generation {0}")]
    InvalidGeneration(u64),
    #[error("Network Knowledge error {0:?}")]
    NetworkKnowledge(#[from] sn_interface::network_knowledge::Error),
    #[error("Custom {0}")]
    Custom(String),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

fn get_split_info(
    prefix: Prefix,
    members: &BTreeMap<XorName, NodeState>,
) -> Option<(BTreeSet<NodeState>, BTreeSet<NodeState>)> {
    let (zero, one) = partition_by_prefix(&prefix, members.keys().copied())?;

    // make sure the sections contain enough entries
    let split_threshold = recommended_section_size();
    if zero.len() < split_threshold || one.len() < split_threshold {
        return None;
    }

    Some((
        BTreeSet::from_iter(zero.into_iter().map(|n| members[&n].clone())),
        BTreeSet::from_iter(one.into_iter().map(|n| members[&n].clone())),
    ))
}

/// Checks if we can split the section
/// If we have enough nodes for both subsections, returns the `DkgSessionId`'s
pub(crate) fn try_split_dkg(
    members: &BTreeMap<XorName, NodeState>,
    sap: &SectionAuthorityProvider,
    section_chain_len: u64,
    membership_gen: Generation,
) -> Option<(DkgSessionId, DkgSessionId)> {
    let prefix = sap.prefix();

    let (zero, one) = get_split_info(prefix, members)?;

    // get elders for section ...0
    let zero_prefix = prefix.pushed(false);
    let zero_elders = elder_candidates(zero.iter().cloned(), sap);

    // get elders for section ...1
    let one_prefix = prefix.pushed(true);
    let one_elders = elder_candidates(one.iter().cloned(), sap);

    // create the DKG session IDs
    let zero_id = DkgSessionId {
        prefix: zero_prefix,
        elders: BTreeMap::from_iter(zero_elders.iter().map(|node| (node.name(), node.addr()))),
        section_chain_len,
        bootstrap_members: zero,
        membership_gen,
    };
    let one_id = DkgSessionId {
        prefix: one_prefix,
        elders: BTreeMap::from_iter(one_elders.iter().map(|node| (node.name(), node.addr()))),
        section_chain_len,
        bootstrap_members: one,
        membership_gen,
    };

    Some((zero_id, one_id))
}

/// Returns the nodes that should be candidates to become the next elders, sorted by names.
pub(crate) fn elder_candidates(
    candidates: impl IntoIterator<Item = NodeState>,
    current_elders: &SectionAuthorityProvider,
) -> BTreeSet<NodeState> {
    use itertools::Itertools;
    use std::cmp::Ordering;

    // Compare candidates for the next elders. The one comparing `Less` wins.
    fn cmp_elder_candidates(
        lhs: &NodeState,
        rhs: &NodeState,
        current_elders: &SectionAuthorityProvider,
    ) -> Ordering {
        // Older nodes are preferred. In case of a tie, prefer current elders. If still a tie, break
        // it comparing by the signed signatures because it's impossible for a node to predict its
        // signature and therefore game its chances of promotion.
        rhs.age()
            .cmp(&lhs.age())
            .then_with(|| {
                let lhs_is_elder = current_elders.contains_elder(&lhs.name());
                let rhs_is_elder = current_elders.contains_elder(&rhs.name());

                match (lhs_is_elder, rhs_is_elder) {
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    _ => Ordering::Equal,
                }
            })
            .then_with(|| lhs.name().cmp(&rhs.name()))
        // TODO: replace name cmp above with sig cmp.
        // .then_with(|| lhs.sig.signature.cmp(&rhs.sig.signature))
    }

    candidates
        .into_iter()
        .sorted_by(|lhs, rhs| cmp_elder_candidates(lhs, rhs, current_elders))
        .take(sn_interface::elder_count())
        .collect()
}

#[derive(Clone)]
struct ValidatorContext {
    gen: u64,
    prefix: Prefix,
    section_members: BTreeMap<XorName, NodeState>,
    archived_members: BTreeSet<XorName>,
}

fn proposal_validator(
    context: &ValidatorContext,
    domain: &Domain,
    _node_id: NodeId,
    proposal: &NodeState,
) -> bool {
    // We need to pass current state:
    //   1- Clone: 3rd  argument as Any
    //   2- Closure: To not pass 3rd argument?
    //   3- Generic: 3rd  argument as generic

    // cast any to something that possible to cast

    let proposal_tag = domain.seq as u64;
    if proposal_tag != context.gen {
        return false;
    }
    let members = BTreeMap::from_iter(context.section_members.clone().into_iter());
    let archived_members = &context.archived_members;

    if let Err(err) = proposal.validate_node_state(&context.prefix, &members, &archived_members) {
        warn!("Failed to validate {proposal:?} with error {:?}", err);
        // TODO: certain errors need AE?
        warn!(
            "Members at generation {} are: {:?}",
            proposal_tag - 1,
            members
        );
        warn!("Archived members are {:?}", archived_members);
        return false;
    }

    true
}

#[derive(Clone)]
pub(crate) struct Membership {
    //We wrap the Consensus under Arc<Mutex<...>>, because it can't be cloned itself.
    consensus_guard_opt:
        Option<Arc<Mutex<(ValidatorContext, Consensus<ValidatorContext, NodeState>)>>>,
    self_id: NodeId,
    bootstrap_members: BTreeSet<NodeState>,
    pub(crate) gen: Generation, // current generation
    secret_key: (NodeId, SecretKeyShare),
    elders: PublicKeySet,
    elders_id: Vec<NodeId>,
    history: BTreeMap<Generation, Decision<NodeState>>, // TODO: we can use vector here
    // last membership vote timestamp
    last_received_vote_time: Option<Instant>,
    outgoing_log: Vec<Outgoing<NodeState>>,
}

impl Membership {
    pub(crate) fn from(
        secret_key: (NodeId, SecretKeyShare),
        elders: PublicKeySet,
        n_elders: usize,
        gen: u64,
        bootstrap_members: BTreeSet<NodeState>,
    ) -> Self {
        trace!("Membership - Creating new membership instance");

        let mut elders_id = Vec::new();
        for i in 0..n_elders {
            elders_id.push(i);
        }

        Membership {
            consensus_guard_opt: None,
            self_id: secret_key.0,
            bootstrap_members,
            gen,
            secret_key,
            elders,
            elders_id,
            history: BTreeMap::default(),
            last_received_vote_time: None,
            outgoing_log: Vec::new(),
        }
    }

    pub(crate) fn section_key_set(&self) -> PublicKeySet {
        self.elders.clone()
    }

    pub(crate) fn last_received_vote_time(&self) -> Option<Instant> {
        self.last_received_vote_time
    }

    pub(crate) fn generation(&self) -> Generation {
        self.gen
    }

    #[cfg(test)]
    pub(crate) fn is_churn_in_progress(&self) -> bool {
        self.consensus_guard_opt.is_some()
    }

    #[cfg(test)]
    pub(crate) fn force_bootstrap(&mut self, state: NodeState) {
        let _ = self.bootstrap_members.insert(state);
    }

    // fn consensus_at_gen(&self, gen: Generation) -> Result<&Consensus<NodeState>> {
    //     if gen == self.gen + 1 {
    //         Ok(&self.consensus)
    //     } else {
    //         self.history
    //             .get(&gen)
    //             .map(|(_, c)| c)
    //             .ok_or(Error::Consensus(sn_consensus::Error::BadGeneration {
    //                 requested_gen: gen,
    //                 gen: self.gen,
    //             }))
    //     }
    // }

    // fn consensus_at_gen_mut(&mut self, gen: Generation) -> Result<&mut Consensus<NodeState>> {
    //     if gen == self.gen + 1 {
    //         Ok(&mut self.consensus)
    //     } else {
    //         self.history
    //             .get_mut(&gen)
    //             .map(|(_, c)| c)
    //             .ok_or(Error::Consensus(sn_consensus::Error::BadGeneration {
    //                 requested_gen: gen,
    //                 gen: self.gen,
    //             }))
    //     }
    // }

    pub(crate) fn archived_members(&self) -> BTreeSet<XorName> {
        let mut members = BTreeSet::from_iter(
            self.bootstrap_members
                .iter()
                .filter(|n| {
                    matches!(
                        n.state(),
                        MembershipState::Left | MembershipState::Relocated(..)
                    )
                })
                .map(|n| n.name()),
        );

        for decision in self.history.values() {
            let node_state = &decision.proposal;
            match node_state.state() {
                MembershipState::Joined => {
                    continue;
                }
                MembershipState::Left | MembershipState::Relocated(_) => {
                    let _ = members.insert(node_state.name());
                }
            }
        }

        members
    }

    /// get only section members reporting Joined till gen
    fn section_members(&self, gen: Generation) -> Result<BTreeMap<XorName, NodeState>> {
        let mut members = BTreeMap::from_iter(
            self.bootstrap_members
                .iter()
                .cloned()
                .filter(|n| matches!(n.state(), MembershipState::Joined))
                .map(|n| (n.name(), n)),
        );

        if gen == 1 {
            return Ok(members);
        }

        for (history_gen, decision) in &self.history {
            let node_state = &decision.proposal;
            match node_state.state() {
                MembershipState::Joined => {
                    let _ = members.insert(node_state.name(), node_state.clone());
                }
                MembershipState::Left => {
                    let _ = members.remove(&node_state.name());
                }
                MembershipState::Relocated(_) => {
                    let _ = members.remove(&node_state.name());
                }
            }

            if history_gen == &gen {
                return Ok(members);
            }
        }

        Err(Error::InvalidGeneration(gen))
    }

    pub(crate) fn propose(
        &mut self,
        node_state: NodeState,
        prefix: &Prefix,
    ) -> Result<Vec<Outgoing<NodeState>>> {
        // TODO: no unwrap
        if self.consensus_guard_opt.is_some() {
            return Err(Error::InvalidProposal);
        }

        self.gen += 1;
        let validator_context = ValidatorContext {
            gen: self.gen,
            prefix: prefix.clone(),
            archived_members: self.archived_members(),
            section_members: self.section_members(self.gen)?,
        };

        let domain = Domain::new("membership", self.gen as usize);
        let mut consensus = Consensus::init(
            domain,
            self.secret_key.0,
            self.secret_key.1.clone(),
            self.elders.clone(),
            self.elders_id.clone(),
            proposal_validator,
            validator_context.clone(),
        );

        // clear all outgoings, this prevents to send expired messages
        self.outgoing_log.clear();

        let domain = consensus.domain();
        self.validate_proposals(&validator_context, &domain, &node_state, prefix)?;

        let outgoings = consensus.propose(node_state)?;
        self.outgoing_log.append(&mut outgoings.clone());

        self.consensus_guard_opt = Some(Arc::new(Mutex::new((validator_context, consensus))));

        return Ok(outgoings);
    }

    // This will simplified other part of the code (we can remove `MembershipAE(Generation)` from `NodeMsg`)
    pub(crate) fn anti_entropy(&self, from_gen: Generation) -> Vec<Decision<NodeState>> {
        let msgs = self
            .history
            .iter() // history is a BTreeSet, .iter() is ordered by generation
            .filter(|(gen, _)| **gen >= from_gen)
            .map(|(_, decision)| decision.clone())
            .collect::<Vec<_>>();

        msgs
    }

    pub(crate) fn id(&self) -> &NodeId {
        &self.self_id
    }

    pub(crate) fn handle_signed_vote(
        &mut self,
        bundle: Bundle<NodeState>,
        _prefix: &Prefix,
    ) -> Result<(Vec<Outgoing<NodeState>>, Option<Decision<NodeState>>)> {
        let bundle_gen = bundle.domain().seq as u64;
        if bundle_gen < self.gen {
            // The node is behind us, send him decided proposal
            match self.history.get(&bundle_gen) {
                Some(decision) => Ok((vec![], Some(decision.clone()))),
                None => Err(Error::Custom(format!(
                    "we don't have the decided proposal for {bundle_gen} generation"
                ))),
            }
        } else if bundle_gen > self.gen {
            // we are behind of the network, should ask for missed proposal

            // TODO: how?
            Ok((vec![], None))
        } else {
            let mut outgoings = Vec::new();
            let mut decision_opt = None;

            {
                let consensus_guard = self.consensus_guard_opt.as_mut().unwrap();
                let consensus = &mut consensus_guard.lock().unwrap().1; // TODO: no unwrap

                let cons_decision_opt = consensus.decided_proposal();
                if let Some(decision) = &cons_decision_opt {
                    info!(
                        "Membership - updated generation from {:?} to {:?}",
                        self.gen, decision.domain.seq
                    );
                    if let Some(old_decision) = self.history.insert(self.gen, decision.clone()) {
                        error!("there is an old decision for {} generation: old: {old_decision:?}, new: {decision:?}", self.gen);
                    }
                    self.gen = decision.domain.seq as u64 + 1;

                    decision_opt = cons_decision_opt.clone();
                } else {
                    let consensus_outgoing = consensus.process_bundle(&bundle)?;

                    outgoings.append(&mut consensus_outgoing.clone());
                    if !self.outgoing_log.is_empty() {
                        // TODO:
                        // Why uncommenting this line cause test failure?

                        // Adding a random message, in case of message loss in the network
                        // let index: usize = rand::random();
                        // outgoings.push(self.outgoing_log[index % self.outgoing_log.len()].clone());
                    }

                    self.outgoing_log.append(&mut outgoings.clone());
                }
            }

            if decision_opt.is_some() {
                self.consensus_guard_opt = None;
            }

            Ok((outgoings, decision_opt))
        }
    }

    /// Returns true if the proposal is valid
    fn validate_proposals(
        &self,
        validator_context: &ValidatorContext,
        domain: &Domain,
        proposal: &NodeState,
        prefix: &Prefix,
    ) -> Result<()> {
        if !proposal_validator(validator_context, domain, self.self_id, proposal) {
            return Err(Error::Custom(format!(
                "our proposal is invalid: {proposal:?}"
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use crate::node::flow_ctrl::tests::network_builder::TestNetworkBuilder;
    use sn_interface::{
        init_logger,
        network_knowledge::NodeState,
        test_utils::{gen_node_id, TestSapBuilder},
    };

    use assert_matches::assert_matches;
    use eyre::Result;
    use rand::thread_rng;
    use xor_name::Prefix;

    #[tokio::test]
    async fn multiple_proposals_in_a_single_generation_should_not_be_possible() -> Result<()> {
        let prefix = Prefix::default();
        let env = TestNetworkBuilder::new(thread_rng())
            .sap(TestSapBuilder::new(prefix))
            .build()?;

        let mut membership = env
            .get_nodes(prefix, 1, 0, None)?
            .remove(0)
            .membership
            .expect("Membership for the elder should've been initialized");

        let state1 = NodeState::joined(gen_node_id(5), None);
        let state2 = NodeState::joined(gen_node_id(5), None);

        let _ = membership.propose(state1, &prefix)?;
        assert_matches!(
            membership.propose(state2, &prefix),
            Err(Error::InvalidProposal)
        );

        Ok(())
    }
}
