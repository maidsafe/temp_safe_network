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
}
