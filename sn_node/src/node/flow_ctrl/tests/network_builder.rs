// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::test_utils::gen_node_infos_with_comm;
use crate::{
    node::{cfg::create_test_capacity_and_root_storage, MyNode, NodeEventsChannel},
    UsedSpace,
};

use sn_comms::{Comm, CommEvent};
use sn_interface::{
    messaging::system::SectionSigned,
    network_knowledge::{
        MyNodeInfo, NetworkKnowledge, NodeState, SectionAuthorityProvider, SectionKeyShare,
        SectionKeysProvider, SectionTree, SectionTreeUpdate, SectionsDAG,
    },
    test_utils::*,
    types::{NodeId, PublicKey},
};

use bls::SecretKeySet;
use eyre::{eyre, Context, Result};
use rand::RngCore;
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    iter,
};
use tokio::sync::mpsc::{self, Receiver};
use xor_name::Prefix;

// The default elder age pattern
pub(crate) const ELDER_AGE_PATTERN: &[u8] = &[50, 45, 40, 35, 30, 25, 20];
// the Rx channel for each node
pub(crate) type TestCommRx = BTreeMap<PublicKey, Option<Receiver<CommEvent>>>;

#[derive(Clone, Debug)]
enum TestMemberType {
    Elder,
    Adult,
}

/// Helper to build the `TestNetwork` struct
pub(crate) struct TestNetworkBuilder<R: RngCore> {
    #[allow(clippy::type_complexity)]
    sections: Vec<(
        SectionAuthorityProvider,
        Vec<(MyNodeInfo, Comm, TestMemberType)>,
        SecretKeySet,
    )>,
    receivers: TestCommRx,
    rng: R,
    n_churns_each_section: usize,
}

impl<R: RngCore> TestNetworkBuilder<R> {
    /// Initializes the builder. Provide custom rng or just use `thread_rng()`
    pub(crate) fn new(rng: R) -> TestNetworkBuilder<R> {
        TestNetworkBuilder {
            sections: Vec::new(),
            rng,
            receivers: BTreeMap::new(),
            n_churns_each_section: 1,
        }
    }

    /// The number of churn events that can happen within a single `Prefix`. This will create extra
    /// SAPs chained to each other for the same prefix. The n_churns_each_section will only be applied
    /// for the Prefixes for which the user has not provided the SAPs.
    pub(crate) fn set_n_churns(mut self, churns: usize) -> TestNetworkBuilder<R> {
        self.n_churns_each_section = churns;
        self
    }

    /// Add a SAP to the network by providing values to a `TestSapBuilder`. If multiple SAPs
    /// are provided for the same Prefix, they are considered to have gone through churns in the
    /// order they are provided.
    ///
    /// Calling `TestSapBuilder::new()` will set the following defaults
    ///
    /// elder_count = elder_count()
    /// adult_count = 0
    /// membership_gen = 0
    pub(crate) fn sap(mut self, mut sap_builder: TestSapBuilder) -> TestNetworkBuilder<R> {
        // provide default age pattern if nothing is provided
        if sap_builder.elder_age_pattern.is_none() {
            sap_builder.elder_age_pattern = Some(Vec::from(ELDER_AGE_PATTERN));
        }

        let (sap, nodes, sk_set, comm_rx) = self.build_sap(sap_builder);

        self.sections.push((sap, nodes, sk_set));
        self.receivers.extend(comm_rx.into_iter());
        self
    }

    /// Create a new SAP with the provided set of members for the given Prefix. This is useful if
    /// you want to retain some of the members from a previous churn.
    ///
    /// Use `build_node_infos_with_comm()` to build the nodes;
    /// Note: The `mpsc::Receiver` for the nodes are assumed to be held by the caller. Hence trying
    /// to retrieve it from `TestNetwork.get_receivers()` will cause a panic
    pub(crate) fn sap_with_members<E, M>(
        mut self,
        prefix: Prefix,
        elders: E,
        members: M,
    ) -> TestNetworkBuilder<R>
    where
        E: IntoIterator<Item = (MyNodeInfo, Comm)>,
        M: IntoIterator<Item = (MyNodeInfo, Comm)>,
    {
        let members = members.into_iter().collect::<Vec<_>>();
        let elders = elders.into_iter().collect::<Vec<_>>();
        // map of the nodes for comparing.
        let elder_keys = elders
            .iter()
            .map(|(n, _)| n.public_key())
            .collect::<BTreeSet<_>>();
        let member_keys = members
            .iter()
            .map(|(n, _)| n.public_key())
            .collect::<BTreeSet<_>>();
        assert!(
            elder_keys.iter().all(|k| member_keys.contains(k)),
            "some elders are not part of the members list"
        );

        let elder_count = elders.len();
        let elders_iter = elders.iter().map(|(node, _)| node.id());
        let members_iter = members
            .iter()
            .map(|(node, _)| NodeState::joined(node.id(), None));
        let sk_set = gen_sk_set(&mut self.rng, elder_count, None);
        let sap = SectionAuthorityProvider::new(
            elders_iter,
            prefix,
            members_iter,
            sk_set.public_keys(),
            0,
        );

        let nodes = members
            .into_iter()
            .map(|(info, comm)| {
                let t = if elder_keys.contains(&info.public_key()) {
                    TestMemberType::Elder
                } else {
                    TestMemberType::Adult
                };
                // Since we just get `(MyNodeInfo, Comm)` from the user, the user is expected to
                // hold the `Receiver<MsgReceived>` (or this node might've been from a previous
                // churn that is already inserted), hence insert None only if the entry is Vacant.
                if let Entry::Vacant(entry) = self.receivers.entry(info.public_key()) {
                    let _ = entry.insert(None);
                }
                (info, comm, t)
            })
            .collect::<Vec<(MyNodeInfo, Comm, TestMemberType)>>();
        self.sections.push((sap, nodes, sk_set));
        self
    }

    /// Builds the `TestNetwork` struct.
    ///
    /// Will fill in the gap in prefixes left by the user. For e.g., if the user has provided just
    /// Prefix(100), then all the siblings (2^max_bit_count == 2^3 siblings) and their ancestors are auto
    /// generated to create a complete Network.
    ///
    /// i.e., A single sibling branch will look like this: SAP() -> SAP(0) -> SAP(01) -> SAP(010).
    /// This is done for all the 2^max_bit_count siblings.
    ///
    /// And if n_churns_each_section = 2 , then we will get SAP1() -> SAP2() -> SAP1(1) -> SAP2(1) -> SAP1(10)
    /// -> SAP2(10) -> user_SAP(100)
    pub(crate) fn build(mut self) -> Result<TestNetwork> {
        // initially we will have only the user provided saps; hence get the missing prefixes and
        // the user provided prefixes.
        let (missing_prefixes, max_prefixes) = {
            let user_prefixes: BTreeSet<Prefix> =
                self.sections.iter().map(|(sap, ..)| sap.prefix()).collect();

            let max_bit_count = user_prefixes
                .iter()
                .map(|p| p.bit_count())
                .max()
                .ok_or_else(|| eyre!("Please provide atleast a single SAP"))?;

            // max_prefixes are used to construct the `SectionTree`
            let bits = ["0", "1"];
            let max_prefixes = if max_bit_count == 0 {
                BTreeSet::from([Prefix::default()])
            } else if max_bit_count == 1 {
                bits.iter().map(|bit| prefix(bit)).collect()
            } else {
                // the permutations of 0,1 with len = max_bit_count gives us the max_prefixes
                // works only if max_bit_count >= 2
                let max_prefixes: Vec<_> = (2..max_bit_count).fold(
                    bits.iter()
                        .flat_map(|b1| bits.iter().map(|&b2| b2.to_string() + *b1))
                        .collect(),
                    |acc, _| {
                        acc.iter()
                            .flat_map(|b1| bits.iter().map(|&b2| b2.to_string() + b1))
                            .collect()
                    },
                );
                max_prefixes.into_iter().map(|str| prefix(&str)).collect()
            };

            // missing_prefixes are used to construct saps
            let mut missing_prefixes: BTreeSet<Prefix> = BTreeSet::new();
            for pref in max_prefixes.iter() {
                let missing = pref
                    .ancestors()
                    .chain(iter::once(*pref))
                    .filter(|anc| !user_prefixes.contains(anc))
                    .collect::<BTreeSet<Prefix>>();
                missing_prefixes.extend(missing);
            }

            (missing_prefixes, max_prefixes)
        };

        // insert the user provided saps
        let mut sections = BTreeMap::new();
        let mut node_infos = BTreeMap::new();
        for (sap, infos, sk_set) in self.sections.iter() {
            let sap = TestKeys::get_section_signed(&sk_set.secret_key(), sap.clone())?;
            let prefix = sap.prefix();

            match node_infos.entry(prefix) {
                Entry::Vacant(entry) => {
                    let _ = entry.insert(vec![infos.clone()]);
                }
                Entry::Occupied(mut entry) => entry.get_mut().push(infos.clone()),
            }

            match sections.entry(prefix) {
                Entry::Vacant(entry) => {
                    let _ = entry.insert(vec![(sap, sk_set.clone())]);
                }
                Entry::Occupied(mut entry) => {
                    // push the next sap for the same prefix
                    entry.get_mut().push((sap, sk_set.clone()));
                }
            };
        }

        // build the SAPs for the missing prefixes
        for prefix in missing_prefixes {
            let mut s = Vec::new();
            let mut n_i = Vec::new();
            let n_churns_each_section = self.n_churns_each_section;
            for _ in 0..n_churns_each_section {
                // infos are sorted by age
                let (sap, infos, sk_set, comm_rx) = self.build_sap(
                    TestSapBuilder::new(prefix).elder_age_pattern(Vec::from(ELDER_AGE_PATTERN)),
                );
                let sap = TestKeys::get_section_signed(&sk_set.secret_key(), sap)?;
                // the CommRx for the user provided SAPs prior to calling `build`. Hence we just
                // need to insert the ones that we get now.
                self.receivers.extend(comm_rx.into_iter());
                s.push((sap, sk_set));
                n_i.push(infos);
            }
            let _ = node_infos.insert(prefix, n_i);
            let _ = sections.insert(prefix, s);
        }

        let section_tree = Self::build_section_tree(&sections, max_prefixes)?;
        Ok(TestNetwork {
            sections,
            section_tree,
            nodes: node_infos,
            receivers: self.receivers,
        })
    }

    /// Helper to build the `SectionTree`
    fn build_section_tree(
        sections: &BTreeMap<Prefix, Vec<(SectionSigned<SectionAuthorityProvider>, SecretKeySet)>>,
        max_prefixes: BTreeSet<Prefix>,
    ) -> Result<SectionTree> {
        let gen_prefix = &sections
            .get(&Prefix::default())
            .ok_or_else(|| eyre!("Genesis section is absent. Provide atleast a single SAP"))?;
        let gen_sap = gen_prefix[0].0.clone();

        let mut section_tree =
            SectionTree::new(gen_sap).wrap_err("gen_sap should belongs to the genesis prefix")?;
        let mut completed = BTreeSet::new();
        // need to insert the default prefix first
        let prefix_iter = if max_prefixes.contains(&Prefix::default()) {
            max_prefixes
        } else {
            iter::once(Prefix::default())
                .chain(max_prefixes.into_iter())
                .collect()
        };

        for max_prefix in prefix_iter.iter() {
            let (first_unique_prefix, mut parent) = if *max_prefix == Prefix::default() {
                // if we have the default prefix, then we haven't inserted anything yet. So the first churn
                // should be inserted.
                let parent = sections
                    .get(max_prefix)
                    .ok_or_else(|| eyre!("{max_prefix} is not present in 'sections'"))?
                    .first()
                    .ok_or_else(|| eyre!("{max_prefix:?} does not contain even a single SAP"))?
                    .1
                    .secret_key();
                (*max_prefix, parent)
            } else {
                // if max_prefix is not the default prefix, then it means that the user has
                // provided something greater than Prefix() and hence find the ancestor that
                // we have not inserted yet.
                let first_unique_prefix = max_prefix
                    .ancestors()
                    .chain(iter::once(*max_prefix))
                    .find(|anc| !completed.contains(anc))
                    .ok_or_else(|| {
                        eyre!("Ancestors starts from genesis, so it should always return something")
                    })?;
                // completed_till is the smallest prefix that we have inserted from the ancestor list
                // of our max_prefix. If the prefix has n_churns, the last key from the last churn is
                // considered as the parent
                let completed_till = first_unique_prefix.popped();
                let parent = sections
                    .get(&completed_till)
                    .ok_or_else(|| eyre!("{completed_till:?} is not present in 'sections'"))?
                    .last()
                    .ok_or_else(|| eyre!("{completed_till:?} does not contain even a single SAP"))?
                    .1
                    .secret_key();
                (first_unique_prefix, parent)
            };

            let mut genesis_section_skipped = false;
            // insert from the first_unique_prefix to the max_prefix
            for anc in max_prefix
                .ancestors()
                .skip_while(|anc| *anc != first_unique_prefix)
                .chain(iter::once(*max_prefix))
            {
                // each anc can have multiple churns
                for (sap, sk) in sections
                    .get(&anc)
                    .ok_or_else(|| eyre!("The ancestor {anc} is absent in 'sections'"))?
                {
                    // to skip the first iteration of Prefix() since we have used that to create the
                    // SectionTree
                    if !genesis_section_skipped && anc == Prefix::default() {
                        genesis_section_skipped = true;
                        continue;
                    }
                    let sk = sk.secret_key();
                    let sig = TestKeys::sign(&parent, &sk.public_key())?;
                    let mut proof_chain = SectionsDAG::new(parent.public_key());
                    proof_chain
                        .verify_and_insert(&parent.public_key(), sk.public_key(), sig)
                        .wrap_err("This should not fail as the parent is already present in the proof_chain")?;
                    let update =
                        TestSectionTree::get_section_tree_update(sap, &proof_chain, &parent)?;
                    let _ = section_tree
                        .update_the_section_tree(update)
                        .wrap_err("Failed to update section_tree")?;
                    parent = sk;
                    let _ = completed.insert(anc);
                }
            }
        }

        Ok(section_tree)
    }

    fn build_sap(
        &mut self,
        sap_builder: TestSapBuilder,
    ) -> (
        SectionAuthorityProvider,
        Vec<(MyNodeInfo, Comm, TestMemberType)>,
        SecretKeySet,
        TestCommRx,
    ) {
        let (sap, sk_set, elders, adults, comm_rx) = sap_builder.build_with_comm(&mut self.rng);

        let nodes = elders
            .into_iter()
            .map(|(n, c)| (n, c, TestMemberType::Elder))
            .chain(
                adults
                    .into_iter()
                    .map(|(n, c)| (n, c, TestMemberType::Adult)),
            )
            .collect();

        (sap, nodes, sk_set, comm_rx)
    }
}

/// Test utility to build a valid and functional network. Use the above builder to construct the
/// utility. The user can just state that he needs a network with Prefix(100) and the builder will
/// construct a valid network by filling in the gaps in the Prefix and also create a valid
/// `SectionTree`.
///
/// The constructed `TestNetwork` utility can be used to obtain `MyNode` instance from any section
/// in the network. It can also provide the `NetworkKnowledge` and `SectionKeyShare` for a given
/// section.
pub(crate) struct TestNetwork {
    // All the sections per Prefix, ordered by churn_idx
    sections: BTreeMap<Prefix, Vec<(SectionSigned<SectionAuthorityProvider>, SecretKeySet)>>,
    // The SectionTree of the entire network
    section_tree: SectionTree,
    // All the Nodes per Prefix, ordered by churn_idx
    #[allow(clippy::type_complexity)]
    nodes: BTreeMap<Prefix, Vec<Vec<(MyNodeInfo, Comm, TestMemberType)>>>,
    // The mpsc receiver for each node. Will be moved out once retrieved
    receivers: TestCommRx,
}

impl TestNetwork {
    /// Build elder/adult `MyNode` instances for a given `Prefix`. The elder_count and adult_count
    /// should be <= the actual count specified in the SAP.
    /// The created instance has knowledge about the Network only from the genesis section to its
    /// current section.
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get
    /// a specific SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_nodes(
        &self,
        prefix: Prefix,
        elder_count: usize,
        adult_count: usize,
        churn_idx: Option<usize>,
    ) -> Result<Vec<MyNode>> {
        let nodes = self.get_nodes_single_churn(prefix, churn_idx)?;
        let sap_details = self.get_sap_single_churn(prefix, churn_idx)?;

        assert!(
            elder_count <= sap_details.0.elder_count(),
            "elder_count should be <= {}",
            sap_details.0.elder_count()
        );
        let sap_adult_count = sap_details.0.members().count() - sap_details.0.elder_count();
        assert!(
            adult_count <= sap_adult_count,
            "adult_count should be <= {sap_adult_count}",
        );

        let network_knowledge = self.build_network_knowledge(&sap_details.0, &sap_details.1)?;

        let mut my_nodes = Vec::new();
        let nodes_iter = {
            let elder_iter = nodes
                .iter()
                .filter(|(.., t)| matches!(t, TestMemberType::Elder))
                .enumerate()
                .map(|(idx, node)| {
                    let sk_share = TestKeys::get_section_key_share(&sap_details.1, idx);
                    (node, Some(sk_share))
                })
                .take(elder_count);
            let adult_iter = nodes
                .iter()
                .filter(|(.., t)| matches!(t, TestMemberType::Adult))
                .map(|node| (node, None))
                .take(adult_count);
            elder_iter.chain(adult_iter)
        };

        for ((info, comm, _), sk_share) in nodes_iter {
            let my_node = self.build_my_node_instance(
                prefix,
                churn_idx,
                &network_knowledge,
                info,
                comm,
                &sk_share,
            )?;
            my_nodes.push(my_node);
        }

        Ok(my_nodes)
    }

    /// Build the `MyNode` instance given the node's `PublicKey` for a particular `Prefix`.
    /// The created instance has knowledge about the Network only from the genesis section to its
    /// current section.
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get
    /// a specific SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_node_by_key(
        &self,
        prefix: Prefix,
        node_pk: PublicKey,
        churn_idx: Option<usize>,
    ) -> Result<MyNode> {
        let nodes = self.get_nodes_single_churn(prefix, churn_idx)?;
        let node_idx = nodes
            .iter()
            .position(|(info, ..)| info.public_key() == node_pk)
            .ok_or_else(|| {
                eyre!("The node with the given pk is not present for the given prefix/churn")
            })?;
        let node = &nodes[node_idx];

        let sap_details = self.get_sap_single_churn(prefix, churn_idx)?;
        let network_knowledge = self.build_network_knowledge(&sap_details.0, &sap_details.1)?;

        let sk_share = if matches!(node.2, TestMemberType::Elder) {
            let sk_share = TestKeys::get_section_key_share(&sap_details.1, node_idx);
            Some(sk_share)
        } else {
            None
        };

        self.build_my_node_instance(
            prefix,
            churn_idx,
            &network_knowledge,
            &node.0,
            &node.1,
            &sk_share,
        )
    }

    /// Take the `mspc::Receiver` for a given node. The receiver will be moved out of the `TestNetwork`.
    ///
    /// Will panic if called more than once for a single node or if the `node_pk` is not part of the `TestNetwork`
    pub(crate) fn take_comm_rx(&mut self, node_pk: PublicKey) -> Receiver<CommEvent> {
        match self.receivers.entry(node_pk) {
            Entry::Vacant(_) => {
                panic!("Something went wrong, the key must be present in self.receivers")
            }
            Entry::Occupied(entry) => {
                let rx = entry.into_mut().take();
                rx.expect("The receiver for the node has already been consumed")
            }
        }
    }

    /// Get elder/adult id for a given `Prefix`. The elder_count and adult_count
    /// should be <= the actual count specified in the SAP. Also return the `SectionKeyShare` for
    /// elder nodes.
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get a specific
    /// SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_node_ids(
        &self,
        prefix: Prefix,
        elder_count: usize,
        adult_count: usize,
        churn_idx: Option<usize>,
    ) -> Result<Vec<NodeId>> {
        let nodes = self.get_nodes_single_churn(prefix, churn_idx)?;
        let sap_details = self.get_sap_single_churn(prefix, churn_idx)?;

        assert!(
            elder_count <= sap_details.0.elder_count(),
            "elder_count should be <= {}",
            sap_details.0.elder_count()
        );
        let sap_adult_count = sap_details.0.members().count() - sap_details.0.elder_count();
        assert!(
            adult_count <= sap_adult_count,
            "adult_count should be <= {sap_adult_count}"
        );

        let elder_iter = nodes.iter().take(elder_count).map(|(node, ..)| node.id());
        let adult_iter = nodes
            .iter()
            .skip(sap_details.0.elder_count())
            .map(|(node, ..)| node.id())
            .take(adult_count);
        Ok(elder_iter.chain(adult_iter).collect())
    }

    /// Get the `SectionKeyShare` given the elder node's `PublicKey` for a particular `Prefix`.
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get
    /// a specific SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_section_key_share(
        &self,
        prefix: Prefix,
        node_pk: PublicKey,
        churn_idx: Option<usize>,
    ) -> Result<SectionKeyShare> {
        let nodes = self.get_nodes_single_churn(prefix, churn_idx)?;
        let sap_details = self.get_sap_single_churn(prefix, churn_idx)?;

        // the node_pk should be an elder
        let share_idx = nodes
            .iter()
            .filter(|(.., t)| matches!(t, TestMemberType::Elder))
            .position(|(info, ..)| info.public_key() == node_pk)
            .expect("The elder with the given node_pk is not present for the given prefix/churn");

        Ok(TestKeys::get_section_key_share(&sap_details.1, share_idx))
    }

    /// Get the `SecretKeySet` for a given `Prefix`
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get
    /// a specific SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_secret_key_set(
        &self,
        prefix: Prefix,
        churn_idx: Option<usize>,
    ) -> Result<SecretKeySet> {
        Ok(self.get_sap_single_churn(prefix, churn_idx)?.1.clone())
    }

    /// Get the `NetworkKnowledge` for a given `Prefix`
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get a specific
    /// SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_network_knowledge(
        &self,
        prefix: Prefix,
        churn_idx: Option<usize>,
    ) -> Result<NetworkKnowledge> {
        let sap_details = self.get_sap_single_churn(prefix, churn_idx)?;
        self.build_network_knowledge(&sap_details.0, &sap_details.1)
    }

    /// Get `SectionSigned<SectionAuthorityProvider>` for a given `Prefix`.
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get a specific
    /// SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_sap(
        &self,
        prefix: Prefix,
        churn_idx: Option<usize>,
    ) -> Result<SectionSigned<SectionAuthorityProvider>> {
        let sap_details = self.get_sap_single_churn(prefix, churn_idx)?;
        Ok(self
            .build_network_knowledge(&sap_details.0, &sap_details.1)?
            .signed_sap())
    }

    // Creates a single `MyNode` instance
    fn build_my_node_instance(
        &self,
        prefix: Prefix,
        churn_idx: Option<usize>,
        network_knowledge: &NetworkKnowledge,
        info: &MyNodeInfo,
        comm: &Comm,
        sk_share: &Option<SectionKeyShare>,
    ) -> Result<MyNode> {
        let (min_capacity, max_capacity, root_storage_dir) =
            create_test_capacity_and_root_storage().wrap_err("Failed to create root storage")?;
        let mut my_node = MyNode::new(
            comm.clone(),
            info.keypair.clone(),
            bls::SecretKey::random().public_key(),
            network_knowledge.clone(),
            sk_share.clone(),
            UsedSpace::new(min_capacity, max_capacity),
            root_storage_dir,
            mpsc::channel(10).0,
            NodeEventsChannel::default(),
        )
        .wrap_err("Failed to create MyNode")?;

        // the node might've been an elder in any of the ancestor saps of the current prefix
        // or the neighboring prefix. So insert those sk_shares.
        let mut key_provider = SectionKeysProvider::new(None);

        // check if it was an elder for any of the current prefix's churned SAP;
        // Obtain the sk_share of the current churn as well since we will be overriding
        // my_node.section_keys_provider
        let churn_idx = churn_idx.unwrap_or(self.get_sap_all_churns(prefix)?.len() - 1);
        for c_idx in (0..churn_idx + 1).rev() {
            let nodes_of_sap = self.get_nodes_single_churn(prefix, Some(c_idx))?;
            if let Some(share_idx) = nodes_of_sap
                .iter()
                .filter(|(.., t)| matches!(t, TestMemberType::Elder))
                .position(|(node, ..)| node.name() == info.name())
            {
                // get the respective churn's sk_set
                let (_, sk_set) = self.get_sap_single_churn(prefix, Some(c_idx))?;
                let sk_share = TestKeys::get_section_key_share(sk_set, share_idx);
                key_provider.insert(sk_share);
            }
        }

        // the node might've been an elder in any of the SAP of our prefix's ancestor. So get
        // them.
        for anc in prefix.ancestors() {
            for (nodes_of_sap, (_, sk_set)) in self
                .get_nodes_all_churns(anc)?
                .iter()
                .zip(self.get_sap_all_churns(anc)?)
            {
                if let Some(share_idx) = nodes_of_sap
                    .iter()
                    .filter(|(.., t)| matches!(t, TestMemberType::Elder))
                    .position(|(node, ..)| node.name() == info.name())
                {
                    let sk_share = TestKeys::get_section_key_share(sk_set, share_idx);
                    key_provider.insert(sk_share);
                }
            }
        }

        // deal with all other prefixes. Requires us to get max_prefixes; can be obtained from
        // SectionTree::sections, but its private. So maybe have a extra field in our struct.
        // Todo: implement only if any test requires it

        my_node.section_keys_provider = key_provider;
        Ok(my_node)
    }

    // Currently builds NetworkKnowledge with only a single chain i.e., from gen prefix to the
    // provided prefix, while the other user provided prefixes are ignored. (maybe include them?)
    fn build_network_knowledge(
        &self,
        sap: &SectionSigned<SectionAuthorityProvider>,
        _sk_set: &SecretKeySet,
    ) -> Result<NetworkKnowledge> {
        let gen_sap = self
            .get_sap_single_churn(Prefix::default(), Some(0))?
            .0
            .clone();
        let gen_section_key = gen_sap.section_key();

        if &gen_section_key != self.section_tree.genesis_key() {
            return Err(eyre!(
                "The gen_key of the section_Tree should be the same one as stored in 'sections'"
            ));
        }

        let mut tree =
            SectionTree::new(gen_sap).wrap_err("gen_sap should belong to the default prefix")?;
        if gen_section_key != sap.section_key() {
            let proof_chain = self
                .section_tree
                .get_sections_dag()
                .partial_dag(&gen_section_key, &sap.section_key())
                .wrap_err("Failed to create proof_chain")?;
            let update = SectionTreeUpdate::new(sap.clone(), proof_chain);
            let _ = tree
                .update_the_section_tree(update)
                .wrap_err("Error updating the SectionTree")?;
        };

        let mut nw = NetworkKnowledge::new(sap.prefix(), tree)
            .wrap_err("Failed to create NetworkKnowledge")?;
        let initial_members: BTreeSet<_> = sap.members().cloned().collect();
        nw.reset_initial_members(initial_members);

        Ok(nw)
    }

    // Get the SAP, sk_set for a particular churn of a section.
    fn get_sap_single_churn(
        &self,
        prefix: Prefix,
        churn_idx: Option<usize>,
    ) -> Result<&(SectionSigned<SectionAuthorityProvider>, SecretKeySet)> {
        let all_sap_details = self.get_sap_all_churns(prefix)?;
        // select the last churn
        let churn_idx = churn_idx.unwrap_or(all_sap_details.len() - 1);
        all_sap_details
            .get(churn_idx)
            .ok_or_else(|| eyre!("invalid churn idx: {churn_idx}"))
    }

    // Get the SAP, sk_set for all the churns of a section
    fn get_sap_all_churns(
        &self,
        prefix: Prefix,
    ) -> Result<&Vec<(SectionSigned<SectionAuthorityProvider>, SecretKeySet)>> {
        self.sections
            .get(&prefix)
            .ok_or_else(|| eyre!("section not found for {prefix:?}"))
    }

    // Get the all the nodes for a particular churn of a section
    fn get_nodes_single_churn(
        &self,
        prefix: Prefix,
        churn_idx: Option<usize>,
    ) -> Result<&Vec<(MyNodeInfo, Comm, TestMemberType)>> {
        let nodes = self.get_nodes_all_churns(prefix)?;
        let churn_idx = churn_idx.unwrap_or(nodes.len() - 1);
        nodes
            .get(churn_idx)
            .ok_or_else(|| eyre!("invalid churn idx: {churn_idx}"))
    }

    // Get all the nodes for all the churns of a section
    #[allow(clippy::type_complexity)]
    fn get_nodes_all_churns(
        &self,
        prefix: Prefix,
    ) -> Result<&Vec<Vec<(MyNodeInfo, Comm, TestMemberType)>>> {
        self.nodes
            .get(&prefix)
            .ok_or_else(|| eyre!("Nodes not found for {prefix:?}"))
    }
}

/// Expand TestSapBuilder to support `Comms`
trait WithComms {
    #[allow(clippy::type_complexity)]
    fn build_with_comm(
        self,
        rng: impl RngCore,
    ) -> (
        SectionAuthorityProvider,
        SecretKeySet,
        Vec<(MyNodeInfo, Comm)>,
        Vec<(MyNodeInfo, Comm)>,
        TestCommRx,
    );
}
impl WithComms for TestSapBuilder {
    /// Build the final SAP with the provided rng. Also returns the `SecretKeySet` used by the SAP along with the
    /// set of elder, adult nodes.
    fn build_with_comm(
        self,
        rng: impl RngCore,
    ) -> (
        SectionAuthorityProvider,
        SecretKeySet,
        Vec<(MyNodeInfo, Comm)>,
        Vec<(MyNodeInfo, Comm)>,
        TestCommRx,
    ) {
        // Todo: use custom rng to generate the random nodes. `gen_keypair` requires `rand-0.7`
        // version and `SecretKeySet` requires `rand-0.8`; wait for the other one to be bumped.
        let (elder_nodes, adult_nodes, comm_rx) = gen_node_infos_with_comm(
            &self.prefix,
            self.elder_count,
            self.adult_count,
            self.elder_age_pattern.as_deref(),
            self.adult_age_pattern.as_deref(),
        );
        let members = elder_nodes
            .iter()
            .chain(adult_nodes.iter())
            .map(|i| NodeState::joined(i.0.id(), None));

        let sk_set = if let Some(sk) = self.sk_set {
            sk
        } else {
            gen_sk_set(rng, self.elder_count, self.sk_threshold_size)
        };

        let sap = SectionAuthorityProvider::new(
            elder_nodes.iter().map(|i| i.0.id()),
            self.prefix,
            members,
            sk_set.public_keys(),
            self.membership_gen as u64,
        );
        (sap, sk_set, elder_nodes, adult_nodes, comm_rx)
    }
}
