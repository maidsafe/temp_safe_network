use crate::{
    comm::{Comm, MsgFromPeer},
    node::{
        cfg::create_test_max_capacity_and_root_storage,
        core::MyNode,
        flow_ctrl::{dispatcher::Dispatcher, event_channel},
    },
    UsedSpace,
};
use sn_interface::{
    elder_count,
    messaging::system::SectionSigned,
    network_knowledge::{
        supermajority, MyNodeInfo, NetworkKnowledge, NodeState, SectionAuthorityProvider,
        SectionKeyShare, SectionKeysProvider, SectionTree, SectionTreeUpdate, SectionsDAG,
        MIN_ADULT_AGE,
    },
    test_utils::*,
    types::{keys::ed25519::gen_keypair, Peer, PublicKey},
};

use bls::SecretKeySet;
use itertools::Itertools;
use rand::RngCore;
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    iter,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::{
    runtime::Handle,
    sync::{
        mpsc::{self, Receiver},
        RwLock,
    },
};
use xor_name::Prefix;

pub(crate) static TEST_EVENT_CHANNEL_SIZE: usize = 20;
// the Rx channel for each node
pub(crate) type TestCommRx = BTreeMap<PublicKey, Option<Receiver<MsgFromPeer>>>;
#[derive(Clone, Debug)]
pub(crate) enum TestMemberType {
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
    n_churns: usize,
}

impl<R: RngCore> TestNetworkBuilder<R> {
    /// Initializes the builder. Provide custom rng or just use `thread_rng()`
    pub(crate) fn new(rng: R) -> TestNetworkBuilder<R> {
        TestNetworkBuilder {
            sections: Vec::new(),
            rng,
            receivers: BTreeMap::new(),
            n_churns: 1,
        }
    }

    /// The number of churn events that can happen within a single `Prefix`. This will create extra
    /// SAPs chained to each other for the same prefix. The `n_churns` will be applied only for the
    /// `Prefix`es for which the user has not provided the SAPs.
    pub(crate) fn set_n_churns(mut self, churns: usize) -> TestNetworkBuilder<R> {
        self.n_churns = churns;
        self
    }

    /// Provide values to create a `SectionAuthorityProvider` for a given Prefix. If multiple SAPs
    /// are provided for the same Prefix, they are considered to have gone through churns in the
    /// order they are provided.
    ///
    /// The total number of members in the section will be `elder_count` + `adult_count`. A lot of
    /// tests don't require adults in the section, so zero is an acceptable value for
    /// `adult_count`.
    ///
    /// Optionally provide `age_pattern` to create elders with specific ages.
    /// If None = elder's age is set to `MIN_ADULT_AGE`
    /// If age_pattern.len() == elder, then apply the respective ages to each node
    /// If age_pattern.len() < elder, then the last element's value is taken as the age for the remaining nodes.
    /// If age_pattern.len() > elder, then the extra elements after `count` are ignored.
    ///
    /// The default threshold_size for the `SecretKeySet` is set to be `supermajority(elder_count)-1`
    /// This can be overridden by using `sk_threshold_size` since some tests need low thresholds.
    pub(crate) fn sap(
        mut self,
        prefix: Prefix,
        elder_count: usize,
        adult_count: usize,
        elder_age_pattern: Option<&[u8]>,
        sk_threshold_size: Option<usize>,
    ) -> TestNetworkBuilder<R> {
        let mut elder_age_pattern = elder_age_pattern;
        // provide default age pattern if nothing is provided
        if elder_age_pattern.is_none() {
            elder_age_pattern = Some(&[50, 45, 40, 35, 30, 25, 20]);
        }

        let (sap, nodes, sk_set, comm_rx) = self.build_sap(
            prefix,
            elder_count,
            adult_count,
            elder_age_pattern,
            sk_threshold_size,
        );

        self.sections.push((sap, nodes, sk_set));
        self.receivers.extend(comm_rx.into_iter());
        self
    }

    /// Provide pre-built SAPs to be used inside the `TestNetwork`
    /// Note: The `SocketAddr` will be replaced for the nodes. Else use `.sap_with_members()`
    /// if you want to preserve the `SocketAddr`
    pub(crate) fn sap_pre_built(
        mut self,
        sap: &SectionAuthorityProvider,
        node_infos: &[MyNodeInfo],
        secret_key_set: &SecretKeySet,
    ) -> TestNetworkBuilder<R> {
        let handle = Handle::current();
        let _ = handle.enter();

        let mut nodes = Vec::new();
        for node in node_infos {
            let (tx, rx) = mpsc::channel(TEST_EVENT_CHANNEL_SIZE);
            let socket_addr: SocketAddr = (Ipv4Addr::LOCALHOST, 0).into();
            let comm = futures::executor::block_on(Comm::new(socket_addr, Default::default(), tx))
                .expect("failed to create comm");
            let mut node = node.clone();
            node.addr = comm.socket_addr();

            // check MemberType
            let memb_type = if sap.elders_set().contains(&node.peer()) {
                TestMemberType::Elder
            } else {
                TestMemberType::Adult
            };
            // insert the commRx
            let _ = self.receivers.insert(node.public_key(), Some(rx));
            nodes.push((node, comm, memb_type));
        }
        self.sections
            .push((sap.clone(), nodes, secret_key_set.clone()));
        self
    }

    /// Create a SAP with the provided list of members. This is useful if you want to just change
    /// some the members of a previous SAP.
    ///
    /// Use `TestNetwork::build_node_infos()` to build the nodes;
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
        if !elder_keys.iter().all(|k| member_keys.contains(k)) {
            panic!("some elders are not part of the members list");
        }

        let elder_count = elders.len();
        let elders_iter = elders.iter().map(|(node, _)| MyNodeInfo::peer(node));
        let members_iter = members
            .iter()
            .map(|(node, _)| NodeState::joined(node.peer(), None));
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
                let _ = self.receivers.insert(info.public_key(), None);
                (info, comm, t)
            })
            .collect::<Vec<(MyNodeInfo, Comm, TestMemberType)>>();
        self.sections.push((sap, nodes, sk_set));
        self
    }

    /// Builds the `TestNetwork` struct.
    ///
    /// Will fill in the gaps in prefixes left by the user. For e.g., if the user has provided just
    /// Prefix(100), the ancestors P() -> P(1) -> P(10) will be automatically created.
    /// And if n_churns = 2 , then we will get SAP1() -> SAP2() -> SAP1(1) -> SAP2(1) -> SAP1(10)
    /// -> SAP2 (10) -> user_SAP(100)
    pub(crate) fn build(mut self) -> TestNetwork {
        // initially we will have only the user provided saps; hence get the missing prefixes and
        // the user provided prefixes.
        let (missing_prefixes, max_prefixes) = {
            let user_prefixes: BTreeSet<Prefix> =
                self.sections.iter().map(|(sap, ..)| sap.prefix()).collect();

            // missing_prefixes are used to construct saps
            let mut missing_prefixes = BTreeSet::new();
            // max_prefixes are used to construct the `SectionTree`
            let mut min_prefixes = BTreeSet::new();

            for prefix in user_prefixes.iter() {
                // get all the missing ancestor
                prefix
                    .ancestors()
                    .filter(|anc| !user_prefixes.contains(anc))
                    .for_each(|missing_anc| {
                        let _ = missing_prefixes.insert(missing_anc);
                    });

                // a prefix is min, if it is part of the ancestor_list of any prefix
                prefix.ancestors().for_each(|anc| {
                    if user_prefixes.contains(&anc) {
                        _ = min_prefixes.insert(anc);
                    }
                });
            }

            // max_prefixes are the larget user provided prefixes
            let max_prefixes = user_prefixes
                .into_iter()
                .filter(|pre| !min_prefixes.contains(pre))
                .collect::<BTreeSet<_>>();
            (missing_prefixes, max_prefixes)
        };

        // insert the user provided saps
        let mut sections = BTreeMap::new();
        let mut node_infos = BTreeMap::new();
        let mut sk_shares = BTreeMap::new();
        for (sap, infos, sk_set) in self.sections.iter() {
            let sap = TestKeys::get_section_signed(&sk_set.secret_key(), sap.clone());
            let prefix = sap.prefix();

            // store the sk_shares of the elders
            for (idx, (elder, ..)) in infos
                .iter()
                .enumerate()
                .filter(|(_, (.., t))| matches!(t, TestMemberType::Elder))
            {
                let sk_share = TestKeys::get_section_key_share(sk_set, idx);
                match sk_shares.entry(elder.public_key()) {
                    Entry::Vacant(entry) => {
                        let _ = entry.insert(vec![sk_share.clone()]);
                    }
                    Entry::Occupied(mut entry) => entry.get_mut().push(sk_share.clone()),
                }
            }

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
            let n_churns = self.n_churns;
            for _ in 0..n_churns {
                // infos are sorted by age
                let (sap, infos, sk, comm_rx) = self.build_sap(
                    prefix,
                    elder_count(),
                    0,
                    Some(&[50, 45, 40, 35, 30, 25, 20]),
                    Some(supermajority(elder_count())),
                );
                let sap = TestKeys::get_section_signed(&sk.secret_key(), sap);
                // the CommRx for the user provided SAPs prior to calling `build`. Hence we just
                // need to insert the ones that we get now.
                self.receivers.extend(comm_rx.into_iter());
                s.push((sap, sk));
                n_i.push(infos);
            }
            let _ = node_infos.insert(prefix, n_i);
            let _ = sections.insert(prefix, s);
        }

        let section_tree = Self::build_section_tree(&sections, max_prefixes);
        TestNetwork {
            sections,
            section_tree,
            nodes: node_infos,
            sk_shares,
            receivers: self.receivers,
        }
    }

    /// Helper to build the `SectionTree`
    fn build_section_tree(
        sections: &BTreeMap<Prefix, Vec<(SectionSigned<SectionAuthorityProvider>, SecretKeySet)>>,
        max_prefixes: BTreeSet<Prefix>,
    ) -> SectionTree {
        let gen_prefix = &sections
            .get(&Prefix::default())
            .expect("Genesis section is absent. Provide atleast a single SAP");
        let gen_sk = gen_prefix[0].1.secret_key();

        let mut section_tree = SectionTree::new(gen_sk.public_key());
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
                // if we have the default prefix, then we have not inserted anything yet. So the first churn
                // should be inserted.
                let parent = sections
                    .get(max_prefix)
                    .expect("sections should contain the prefix")
                    .first()
                    .expect("should contain atleast one sap")
                    .1
                    .secret_key();
                (*max_prefix, parent)
            } else {
                // if max_prefix is not the default prefix, then it means that the user has
                // provided something greater than Prefix() and hence find ancestor that we have
                // not inserted yet.
                let first_unique_prefix = max_prefix
                    .ancestors()
                    .chain(iter::once(*max_prefix))
                    .find(|anc| !completed.contains(anc))
                    .expect("Ancestors starts from genesis, so it should always return something");
                // completed_till is the smallest prefix that we have inserted from the ancestor list
                // of our max_prefix. If the prefix has n_churns, the last key from the last chrun is
                // considered as the parent
                let completed_till = first_unique_prefix.popped();
                let parent = sections
                    .get(&completed_till)
                    .expect("sections should contain the prefix")
                    .last()
                    .expect("Should contain atleast one SAP")
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
                // each anc can have multiple chruns
                for (sap, sk) in sections
                    .get(&anc)
                    .expect("The ancestor {anc:?} should be present")
                {
                    // to skip the first iteration of Prefix() since we have used that to create the
                    // SectionTree
                    if !genesis_section_skipped && anc == Prefix::default() {
                        genesis_section_skipped = true;
                        continue;
                    }
                    let sk = sk.secret_key();
                    let sig = TestKeys::sign(&parent, &sk.public_key());
                    let mut proof_chain = SectionsDAG::new(parent.public_key());
                    proof_chain
                        .insert(&parent.public_key(), sk.public_key(), sig)
                        .expect("should not fail");
                    let update =
                        TestSectionTree::get_section_tree_update(sap, &proof_chain, &parent);
                    let _ = section_tree
                        .update(update)
                        .expect("Failed to update section_tree");
                    parent = sk;
                    let _ = completed.insert(anc);
                }
            }
        }

        section_tree
    }

    fn build_sap(
        &mut self,
        prefix: Prefix,
        elder_count: usize,
        adult_count: usize,
        elder_age_pattern: Option<&[u8]>,
        sk_threshold_size: Option<usize>,
    ) -> (
        SectionAuthorityProvider,
        Vec<(MyNodeInfo, Comm, TestMemberType)>,
        SecretKeySet,
        TestCommRx,
    ) {
        let (elders, adults, comm_rx) =
            TestNetwork::gen_node_infos(&prefix, elder_count, adult_count, elder_age_pattern);
        let elders_for_sap = elders.iter().map(|(node, _)| MyNodeInfo::peer(node));
        let members = adults
            .iter()
            .map(|(node, _)| MyNodeInfo::peer(node))
            .chain(elders_for_sap.clone())
            .map(|peer| NodeState::joined(peer, None));
        let sk_set = gen_sk_set(&mut self.rng, elder_count, sk_threshold_size);
        let sap =
            SectionAuthorityProvider::new(elders_for_sap, prefix, members, sk_set.public_keys(), 0);

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
    // All the SectionKeyShares for a given elder node (if it is an elder more than one section)
    sk_shares: BTreeMap<PublicKey, Vec<SectionKeyShare>>,
    // The mpsc receiver for each node. Will be moved out once retrived
    receivers: TestCommRx,
}

impl TestNetwork {
    /// Get elder/adult `MyNode` instances for a given `Prefix`. The elder_count and adult_count
    /// should be <= the actual count specified in the SAP. Also return the `SectionKeyShare` for
    /// elder nodes.
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get
    /// a specific SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_nodes(
        &self,
        prefix: Prefix,
        elder_count: usize,
        adult_count: usize,
        churn_idx: Option<usize>,
    ) -> Vec<(MyNode, Option<SectionKeyShare>)> {
        let nodes = self._get_node_infos(prefix, churn_idx);
        let section = self._get_section_details(prefix, churn_idx);

        if elder_count > section.0.elder_count() {
            panic!("elder_count should be <= {}", section.0.elder_count());
        }
        let section_adult_count = section.0.members().count() - section.0.elder_count();
        if adult_count > section_adult_count {
            panic!("adult_count should be <= {}", section_adult_count);
        }

        let network_knowledge = self._get_network_knowledge(&section.0, &section.1);

        let mut my_nodes = Vec::new();
        let nodes_iter = {
            let elder_iter = nodes
                .iter()
                .filter(|(.., t)| matches!(t, TestMemberType::Elder))
                .enumerate()
                .map(|(idx, node)| {
                    let sk_share = TestKeys::get_section_key_share(&section.1, idx);
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
            let my_node =
                self._get_node(prefix, churn_idx, &network_knowledge, info, comm, &sk_share);
            my_nodes.push((my_node, sk_share));
        }

        my_nodes
    }

    /// Get the `MyNode` instances and the for a given `Prefix`. Optionally returns the `SectionKeyShare` if
    /// it's an elder
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get
    /// a specific SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_node_by_key(
        &self,
        prefix: Prefix,
        node_pk: PublicKey,
        churn_idx: Option<usize>,
    ) -> (MyNode, Option<SectionKeyShare>) {
        let nodes = self._get_node_infos(prefix, churn_idx);
        let node_idx = nodes
            .iter()
            .position(|(info, ..)| info.public_key() == node_pk)
            .expect("The node with the given pk is not present for the given prefix/churn");
        let node = &nodes[node_idx];

        let section = self._get_section_details(prefix, churn_idx);
        let network_knowledge = self._get_network_knowledge(&section.0, &section.1);

        let sk_share = if matches!(node.2, TestMemberType::Elder) {
            Some(TestKeys::get_section_key_share(&section.1, node_idx))
        } else {
            None
        };

        let node = self._get_node(
            prefix,
            churn_idx,
            &network_knowledge,
            &node.0,
            &node.1,
            &sk_share,
        );
        (node, sk_share)
    }

    /// Get elder/adult `Dispatcher<MyNode>` instances for a given `Prefix`. The elder_count and adult_count
    /// should be <= the actual count specified in the SAP. Also return the `SectionKeyShare` for
    /// elder nodes.
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get a specific
    /// SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_dispatchers(
        &self,
        prefix: Prefix,
        elder_count: usize,
        adult_count: usize,
        churn_idx: Option<usize>,
    ) -> Vec<(Dispatcher, Option<SectionKeyShare>)> {
        self.get_nodes(prefix, elder_count, adult_count, churn_idx)
            .into_iter()
            .map(|(node, sk_share)| (Dispatcher::new(Arc::new(RwLock::new(node))), sk_share))
            .collect()
    }

    /// Retrieve the `mspc::Receiver` for a given node. The receiver will be moved out and will be set to None
    /// since it does not implement Clone.
    ///
    /// Will panic if called more than once for a single node or if the `node_pk` is not part of the `TestNetwork`
    pub(crate) fn get_comm_rx(&mut self, node_pk: PublicKey) -> Receiver<MsgFromPeer> {
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

    /// Get elder/adult `Peer` for a given `Prefix`. The elder_count and adult_count
    /// should be <= the actual count specified in the SAP. Also return the `SectionKeyShare` for
    /// elder nodes.
    ///
    /// If the Prefix contains multiple churn events (multiple SAPs), provide the churn_idx to get a specific
    /// SAP, else the latest SAP for the prefix is used.
    pub(crate) fn get_peers(
        &self,
        prefix: Prefix,
        elder_count: usize,
        adult_count: usize,
        churn_idx: Option<usize>,
    ) -> Vec<Peer> {
        let nodes = self._get_node_infos(prefix, churn_idx);
        let section = self._get_section_details(prefix, churn_idx);

        if elder_count > section.0.elder_count() {
            panic!("elder_count should be <= {}", section.0.elder_count());
        }
        let section_adult_count = section.0.members().count() - section.0.elder_count();
        if adult_count > section_adult_count {
            panic!("adult_count should be <= {}", section_adult_count);
        }

        let elder_iter = nodes.iter().take(elder_count).map(|(node, ..)| node.peer());
        let adult_iter = nodes
            .iter()
            .skip(section.0.elder_count())
            .map(|(node, ..)| node.peer())
            .take(adult_count);
        elder_iter.chain(adult_iter).collect()
    }

    /// Create set of elder, adults nodes
    ///
    /// Optionally provide `age_pattern` to create elders with specific ages.
    /// If None = elder's age is set to `MIN_ADULT_AGE`
    /// If age_pattern.len() == elder, then apply the respective ages to each node
    /// If age_pattern.len() < elder, then the last element's value is taken as the age for the remaining nodes.
    /// If age_pattern.len() > elder, then the extra elements after `count` are ignored.
    #[allow(clippy::type_complexity)]
    pub(crate) fn gen_node_infos(
        prefix: &Prefix,
        elder: usize,
        adult: usize,
        elder_age_pattern: Option<&[u8]>,
    ) -> (Vec<(MyNodeInfo, Comm)>, Vec<(MyNodeInfo, Comm)>, TestCommRx) {
        let pattern = if let Some(user_pattern) = elder_age_pattern {
            if user_pattern.is_empty() {
                None
            } else if user_pattern.len() < elder {
                let last_element = user_pattern[user_pattern.len() - 1];
                let mut pattern = vec![last_element; elder - user_pattern.len()];
                pattern.extend_from_slice(user_pattern);
                Some(pattern)
            } else {
                Some(Vec::from(user_pattern))
            }
        } else {
            None
        };
        let mut comm_rx = BTreeMap::new();
        let elders = (0..elder)
            .map(|idx| {
                let age = if let Some(pattern) = &pattern {
                    pattern[idx]
                } else {
                    MIN_ADULT_AGE
                };
                let (node, comm, rx) = Self::gen_info(age, Some(*prefix));
                comm_rx.extend(rx.into_iter());
                (node, comm)
            })
            .collect();
        let adults = (0..adult)
            .map(|_| {
                let (node, comm, rx) = Self::gen_info(MIN_ADULT_AGE, Some(*prefix));
                comm_rx.extend(rx.into_iter());
                (node, comm)
            })
            .collect();
        (elders, adults, comm_rx)
    }

    /// Generate `MyNodeInfo` and `Comm`
    pub(crate) fn gen_info(age: u8, prefix: Option<Prefix>) -> (MyNodeInfo, Comm, TestCommRx) {
        let handle = Handle::current();
        let _ = handle.enter();
        let (tx, rx) = mpsc::channel(TEST_EVENT_CHANNEL_SIZE);
        let socket_addr: SocketAddr = (Ipv4Addr::LOCALHOST, 0).into();
        let comm = futures::executor::block_on(Comm::new(socket_addr, Default::default(), tx))
            .expect("failed  to create comm");
        let info = MyNodeInfo::new(
            gen_keypair(&prefix.unwrap_or_default().range_inclusive(), age),
            comm.socket_addr(),
        );
        let comm_rx = BTreeMap::from([(info.public_key(), Some(rx))]);
        (info, comm, comm_rx)
    }

    // Create a single `MyNode` instance
    fn _get_node(
        &self,
        prefix: Prefix,
        churn_idx: Option<usize>,
        network_knowledge: &NetworkKnowledge,
        info: &MyNodeInfo,
        comm: &Comm,
        sk_share: &Option<SectionKeyShare>,
    ) -> MyNode {
        // enter the current tokio runtime
        let handle = Handle::current();
        let _ = handle.enter();

        let (max_capacity, root_storage_dir) =
            create_test_max_capacity_and_root_storage().expect("Failed to create root storage");
        let mut my_node = futures::executor::block_on(MyNode::new(
            comm.clone(),
            info.keypair.clone(),
            network_knowledge.clone(),
            sk_share.clone(),
            event_channel::new(1).0,
            UsedSpace::new(max_capacity),
            root_storage_dir,
        ))
        .expect("Failed to create MyNode");

        // the node might've been an elder in any of the ancestor section of the current prefix
        // or the neighbouring prefix. So get the sk_shares.
        let mut elders_in_sections = BTreeSet::new();

        // check if it was an elder for any of the current prefixe's churned SAP;
        // Obtain the sk_share of the current churn as well since we will be overriding
        // my_node.section_keys_provider
        let sections = self._get_sections_details(prefix);
        let churn_idx = churn_idx.unwrap_or(sections.len() - 1);
        (0..churn_idx + 1).rev().for_each(|idx| {
            let sec = sections.get(idx).expect("invalid churn_idx");
            if sec
                .0
                .elders()
                .map(|peer| peer.name())
                .contains(&info.name())
            {
                let _ = elders_in_sections.insert(sec.0.section_key());
            }
        });

        // the node might've been an elder in any of the SAP of our prefix's ancestor. So get
        // them.
        prefix.ancestors().for_each(|anc| {
            let sections = self._get_sections_details(anc);
            for sec in sections {
                if sec
                    .0
                    .elders()
                    .map(|peer| peer.name())
                    .contains(&info.name())
                {
                    let _ = elders_in_sections.insert(sec.0.section_key());
                }
            }
        });

        // deal with all other prefixes. Requires us to get max_prefixes; can be obtained from
        // SectionTree::sections, but its private. So maybe have a extra field in our struct.
        // Todo: implement only if any test requires it

        // now that we have the details of the sections that the node is/was an elder of, get its sk_share
        if let Some(shares) = self.sk_shares.get(&info.public_key()) {
            let mut key_provider = SectionKeysProvider::new(None);
            shares
                .iter()
                .filter(|sk_share| {
                    elders_in_sections.contains(&sk_share.public_key_set.public_key())
                })
                .for_each(|sk_share| key_provider.insert(sk_share.clone()));
            my_node.section_keys_provider = key_provider;
        } else if !elders_in_sections.is_empty() {
            panic!("We should have some sk_shares")
        }
        my_node
    }
    fn _get_section_details(
        &self,
        prefix: Prefix,
        churn_idx: Option<usize>,
    ) -> &(SectionSigned<SectionAuthorityProvider>, SecretKeySet) {
        let section = self._get_sections_details(prefix);
        // select the last churn
        let churn_idx = churn_idx.unwrap_or(section.len() - 1);
        section
            .get(churn_idx)
            .expect("invalid churn idx: {churn_idx}")
    }

    fn _get_sections_details(
        &self,
        prefix: Prefix,
    ) -> &Vec<(SectionSigned<SectionAuthorityProvider>, SecretKeySet)> {
        self.sections
            .get(&prefix)
            .expect("section not found for {prefix:?}")
    }

    fn _get_node_infos(
        &self,
        prefix: Prefix,
        churn_idx: Option<usize>,
    ) -> &Vec<(MyNodeInfo, Comm, TestMemberType)> {
        let nodes = self._get_nodes_infos(prefix);
        let churn_idx = churn_idx.unwrap_or(nodes.len() - 1);
        nodes
            .get(churn_idx)
            .expect("invalid churn idx: {churn_idx}")
    }

    fn _get_nodes_infos(&self, prefix: Prefix) -> &Vec<Vec<(MyNodeInfo, Comm, TestMemberType)>> {
        self.nodes
            .get(&prefix)
            .expect("nodes not found for {prefix:?}")
    }
}
