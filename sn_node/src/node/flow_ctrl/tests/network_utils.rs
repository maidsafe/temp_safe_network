use crate::comm::Comm;
use crate::node::{
    cfg::create_test_max_capacity_and_root_storage,
    flow_ctrl::{
        dispatcher::Dispatcher,
        event_channel,
        event_channel::{EventReceiver, EventSender},
    },
    relocation_check, ChurnId, MyNode,
};
use crate::storage::UsedSpace;
use bls::Signature;
use ed25519_dalek::Keypair;
use eyre::{bail, eyre, Context, Result};
use sn_consensus::Decision;
use sn_interface::{
    elder_count,
    messaging::{system::NodeState as NodeStateMsg, SectionTreeUpdate},
    network_knowledge::{
        test_utils::*, MyNodeInfo, NetworkKnowledge, NodeState, SectionAuthorityProvider,
        SectionKeyShare, SectionTree, SectionsDAG, MIN_ADULT_AGE,
    },
    types::{keys::ed25519, Peer, SecretKeySet},
};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use xor_name::Prefix;

pub(crate) static TEST_EVENT_CHANNEL_SIZE: usize = 20;

/// Utility for constructing a Node with a mock network section.
///
/// The purpose is to reduce test setup verbosity when unit testing things like message handlers.
pub(crate) struct TestNodeBuilder {
    pub(crate) prefix: Prefix,
    pub(crate) elder_count: usize,
    pub(crate) adult_count: usize,
    pub(crate) section_sk_threshold: usize,
    pub(crate) data_copy_count: usize,
    pub(crate) node_event_sender: EventSender,
    pub(crate) section: Option<NetworkKnowledge>,
    pub(crate) first_node: Option<MyNodeInfo>,
    pub(crate) genesis_sk_set: Option<bls::SecretKeySet>,
    pub(crate) sap: Option<SectionAuthorityProvider>,
    pub(crate) custom_peer: Option<Peer>,
    pub(crate) other_section_keys: Option<Vec<bls::SecretKey>>,
    pub(crate) parent_section_tree: Option<SectionTree>,
}

impl TestNodeBuilder {
    /// Create an instance of the builder.
    ///
    /// At the minimum a prefix for the section and an elder count value are required.
    ///
    /// The default adult count will be 0, and the threshold value for generating the secret key
    /// part of the section key will be the supermajority minus 1.
    ///
    /// To supply different values for these, use the `adult_count` and `section_sk_threshold`
    /// functions to set them.
    pub(crate) fn new(prefix: Prefix, elder_count: usize) -> TestNodeBuilder {
        let supermajority = 1 + elder_count * 2 / 3;
        let (event_sender, _) = event_channel::new(TEST_EVENT_CHANNEL_SIZE);
        Self {
            prefix,
            elder_count,
            adult_count: 0,
            section_sk_threshold: supermajority - 1,
            data_copy_count: 4,
            node_event_sender: event_sender,
            section: None,
            first_node: None,
            genesis_sk_set: None,
            sap: None,
            custom_peer: None,
            other_section_keys: None,
            parent_section_tree: None,
        }
    }

    /// Provide a the genesis key set for the section to be created with.
    pub(crate) fn genesis_sk_set(mut self, sk_set: bls::SecretKeySet) -> TestNodeBuilder {
        self.genesis_sk_set = Some(sk_set);
        self
    }

    /// Provide other keys for the section chain.
    ///
    /// This list should *not* include the genesis key.
    pub(crate) fn other_section_keys(mut self, other_keys: Vec<bls::SecretKey>) -> TestNodeBuilder {
        self.other_section_keys = Some(other_keys);
        self
    }

    /// Provide the parent section tree.
    ///
    /// Use this when creating section that's supposed to be related to another one.
    pub(crate) fn parent_section_tree(
        mut self,
        parent_section_tree: SectionTree,
    ) -> TestNodeBuilder {
        self.parent_section_tree = Some(parent_section_tree);
        self
    }

    /// Set the number of adults for the section.
    ///
    /// The default is 0.
    ///
    /// The total members for the section will be `elder_count` + `adult_count`.
    pub(crate) fn adult_count(mut self, count: usize) -> TestNodeBuilder {
        self.adult_count = count;
        self
    }

    /// Specify the threshold used when generating the secret key part of the section key.
    ///
    /// The default is the supermajority of the elder count, minus one.
    ///
    /// It will sometimes be necessary to set this value to 0.
    pub(crate) fn section_sk_threshold(mut self, threshold: usize) -> TestNodeBuilder {
        self.section_sk_threshold = threshold;
        self
    }

    /// Specify the number of times data should be replicated.
    ///
    /// The default value is 4.
    ///
    /// Handling certain messages can result in commands being issued to replicate data across
    /// adults. The number of times it is replicated is controlled by the `SN_DATA_COPY_COUNT`
    /// variable. This variable will be set to the `count` supplied.
    ///
    /// In your test, it may be desirable to control the amount of commands that would be
    /// generated.
    pub(crate) fn data_copy_count(mut self, count: usize) -> TestNodeBuilder {
        self.data_copy_count = count;
        self
    }

    /// Specify a custom event sender.
    ///
    /// The event sender and receiver is a pair. If you need to access the receiver in the test,
    /// create the pair in the test setup and then pass the sender in here and then access the
    /// receiver as needed.
    pub(crate) fn event_sender(mut self, event_sender: EventSender) -> TestNodeBuilder {
        self.node_event_sender = event_sender;
        self
    }

    /// Specify a custom section for the node to be built.
    ///
    /// If a custom section is provided, the secret key set for the section and the first node info
    /// must be provided.
    ///
    /// The node info provides the keypair and address to be used for initialising the node, and the
    /// secret key set is used for generating the section key share.
    pub(crate) fn section(
        mut self,
        section: NetworkKnowledge,
        sk_set: bls::SecretKeySet,
        first_node: MyNodeInfo,
    ) -> TestNodeBuilder {
        self.section = Some(section);
        self.genesis_sk_set = Some(sk_set);
        self.first_node = Some(first_node);
        self
    }

    /// Specify a single custom peer in the section.
    ///
    /// This may have a different age than other members in the section.
    pub(crate) fn custom_peer(mut self, peer: Peer) -> TestNodeBuilder {
        self.custom_peer = Some(peer);
        self
    }

    /// Build a mock network section using the values provided.
    ///
    /// This is to avoid creating another node and dispatcher when they are not needed. It's
    /// simpler to have two separate functions rather than `build` returning options and so on.
    ///
    /// Note that this function is *not* compatible with the use of a custom section.
    pub(crate) async fn build_section(
        self,
    ) -> Result<(NetworkKnowledge, bls::SecretKeySet, SectionKeyShare)> {
        let section_key_set = if let Some(ref section_keys) = self.other_section_keys {
            let last_key = section_keys
                .last()
                .ok_or_else(|| eyre!("The section keys list must be populated"))?;
            bls::SecretKeySet::from_bytes(last_key.to_bytes().to_vec())?
        } else if let Some(ref genesis_sk_set) = self.genesis_sk_set {
            genesis_sk_set.clone()
        } else {
            bls::SecretKeySet::random(self.section_sk_threshold, &mut rand::thread_rng())
        };

        let (sap, _) = random_sap_with_key(
            self.prefix,
            self.elder_count,
            self.adult_count,
            &section_key_set,
        );
        let genesis_key_set = if let Some(ref genesis_sk_set) = self.genesis_sk_set {
            genesis_sk_set.clone()
        } else {
            bls::SecretKeySet::random(self.section_sk_threshold, &mut rand::thread_rng())
        };
        let (section, section_key_share) = create_section(
            &genesis_key_set,
            &sap,
            self.other_section_keys,
            self.parent_section_tree,
        )?;
        Ok((section, section_key_set, section_key_share))
    }

    /// Build a `Node` with mock network section using the values provided.
    ///
    /// A `Dispatcher`, `NetworkKnowledge`, `Peer` and `SecretKeySet` are returned for use in the
    /// test.
    ///
    /// The dispatcher is for sending a message that will invoke the handler you wish to test, the
    /// peer supplies a location for the message, and the secret key set can be used, e.g., when
    /// generating a DBC for the test case. Some tests also require access to things like the SAP,
    /// which can be provided via the section (NetworkKnowledge).
    ///
    /// A node will be created with a mock section and it will be wrapped inside the dispatcher.
    pub(crate) async fn build(
        self,
    ) -> Result<(Dispatcher, NetworkKnowledge, Peer, bls::SecretKeySet)> {
        std::env::set_var("SN_DATA_COPY_COUNT", self.data_copy_count.to_string());
        let (mut section, section_key_share, keypair, peer, sk_set) = if let Some(custom_section) =
            self.section
        {
            let first_node = self.first_node.ok_or_else(|| {
                eyre!("The first node must be provided when providing a custom section")
            })?;
            let sk_set = self.genesis_sk_set.ok_or_else(|| {
                eyre!("The secret key set must be supplied when providing a custom section")
            })?;
            let section_key_share = create_section_key_share(&sk_set, 0);
            (
                custom_section,
                section_key_share,
                first_node.keypair.clone(),
                first_node.peer(),
                sk_set,
            )
        } else {
            let (sap, mut nodes, sk_set) = if let Some(sk_set) = self.genesis_sk_set {
                let (sap, nodes) =
                    random_sap_with_key(self.prefix, self.elder_count, self.adult_count, &sk_set);
                (sap, nodes, sk_set)
            } else {
                random_sap(
                    self.prefix,
                    self.elder_count,
                    self.adult_count,
                    Some(self.section_sk_threshold),
                )
            };
            let (section, section_key_share) = create_section(
                &sk_set,
                &sap,
                self.other_section_keys,
                self.parent_section_tree,
            )?;
            let node = nodes.remove(0);
            let keypair = node.keypair.clone();
            (section, section_key_share, keypair, node.peer(), sk_set)
        };

        if let Some(custom_peer) = self.custom_peer {
            let node_state = NodeState::joined(custom_peer, None);
            let node_state = section_signed(&sk_set.secret_key(), node_state)?;
            let _updated = section.update_member(node_state);
        }

        let (max_capacity, root_storage_dir) = create_test_max_capacity_and_root_storage()?;
        let comm = create_comm().await?;
        let node = MyNode::new(
            comm.socket_addr(),
            keypair,
            section.clone(),
            Some(section_key_share),
            self.node_event_sender,
            UsedSpace::new(max_capacity),
            root_storage_dir,
        )
        .await?;
        let node = Arc::new(RwLock::new(node));
        let dispatcher = Dispatcher::new(node, comm);
        Ok((dispatcher, section, peer, sk_set))
    }
}

pub(crate) fn create_section_with_key(
    prefix: Prefix,
    sk_set: &SecretKeySet,
) -> Result<(NetworkKnowledge, SectionAuthorityProvider)> {
    let (sap, _) = random_sap_with_key(prefix, elder_count(), 0, sk_set);
    let (section, _) = create_section(sk_set, &sap, None, None)?;
    Ok((section, sap))
}

/// Creates a section where all elders and adults are marked as joined members.
///
/// Can be used for tests requiring adults to be members of the section, e.g., when you expect
/// replication to occur after handling a message.
pub(crate) fn create_section(
    genesis_sk_set: &bls::SecretKeySet,
    sap: &SectionAuthorityProvider,
    other_keys: Option<Vec<bls::SecretKey>>,
    parent_section_tree: Option<SectionTree>,
) -> Result<(NetworkKnowledge, SectionKeyShare)> {
    let (mut section, section_key_share) =
        do_create_section(sap, genesis_sk_set, other_keys, parent_section_tree)?;
    for ns in sap.members() {
        let auth_ns = section_signed(&genesis_sk_set.secret_key(), ns.clone())?;
        let _updated = section.update_member(auth_ns);
    }
    Ok((section, section_key_share))
}

/// Creates a section where only elders are marked as joined members.
///
/// Some tests require the condition where only the elders were marked as joined members.
pub(crate) fn create_section_with_elders(
    sk_set: &SecretKeySet,
    sap: &SectionAuthorityProvider,
) -> Result<(NetworkKnowledge, SectionKeyShare)> {
    let (mut section, section_key_share) = do_create_section(sap, sk_set, None, None)?;
    for peer in sap.elders() {
        let node_state = NodeState::joined(*peer, None);
        let node_state = section_signed(sk_set.secret_key(), node_state)?;
        let _updated = section.update_member(node_state);
    }
    Ok((section, section_key_share))
}

pub(crate) fn create_section_key_share(
    sk_set: &bls::SecretKeySet,
    index: usize,
) -> SectionKeyShare {
    SectionKeyShare {
        public_key_set: sk_set.public_keys(),
        index,
        secret_key_share: sk_set.secret_key_share(index),
    }
}

pub(crate) fn create_section_auth() -> (SectionAuthorityProvider, Vec<MyNodeInfo>, bls::SecretKeySet)
{
    let (section_auth, elders, secret_key_set) =
        random_sap(Prefix::default(), elder_count(), 0, None);
    (section_auth, elders, secret_key_set)
}

pub(crate) fn create_peer(age: u8) -> Peer {
    let name = ed25519::gen_name_with_age(age);
    Peer::new(name, gen_addr())
}

pub(crate) fn create_peer_in_prefix(prefix: &Prefix, age: u8) -> Peer {
    let name = ed25519::gen_name_with_age(age);
    Peer::new(prefix.substituted_in(name), gen_addr())
}

pub(crate) fn gen_info(age: u8, prefix: Option<Prefix>) -> MyNodeInfo {
    MyNodeInfo::new(
        ed25519::gen_keypair(&prefix.unwrap_or_default().range_inclusive(), age),
        gen_addr(),
    )
}

pub(crate) async fn create_comm() -> Result<Comm> {
    let (tx, _rx) = mpsc::channel(TEST_EVENT_CHANNEL_SIZE);
    Ok(Comm::new((Ipv4Addr::LOCALHOST, 0).into(), Default::default(), tx).await?)
}

/// Create a `Proposal::Online` whose agreement handling triggers relocation of a node with the
/// given age.
///
/// NOTE: recommended to call this with low `age` (4 or 5), otherwise it might take very long time
/// to complete because it needs to generate a signature with the number of trailing zeroes equal
/// to (or greater that) `age`.
pub(crate) fn create_relocation_trigger(
    sk_set: &bls::SecretKeySet,
    age: u8,
) -> Result<Decision<NodeStateMsg>> {
    loop {
        let node_state =
            NodeState::joined(create_peer(MIN_ADULT_AGE), Some(xor_name::rand::random())).to_msg();
        let decision = section_decision(sk_set, node_state.clone())?;

        let sig: Signature = decision.proposals[&node_state].clone();
        let churn_id = ChurnId(sig.to_bytes());

        if relocation_check(age, &churn_id) && !relocation_check(age + 1, &churn_id) {
            return Ok(decision);
        }
    }
}

///
/// Private helpers
///

fn do_create_section(
    section_auth: &SectionAuthorityProvider,
    genesis_ks: &bls::SecretKeySet,
    other_section_keys: Option<Vec<bls::SecretKey>>,
    parent_section_tree: Option<SectionTree>,
) -> Result<(NetworkKnowledge, SectionKeyShare)> {
    let (section_chain, last_sk, share_index) = if let Some(other_section_keys) = other_section_keys
    {
        let section_chain = make_section_chain(&genesis_ks.secret_key(), &other_section_keys)?;
        let last_key = other_section_keys
            .last()
            .ok_or_else(|| eyre!("The section keys list must be populated"))?;
        let share_index = other_section_keys.len() - 1;
        (section_chain, last_key.clone(), share_index)
    } else {
        let section_chain = SectionsDAG::new(genesis_ks.public_keys().public_key());
        (section_chain, genesis_ks.secret_key(), 0)
    };

    let signed_sap = section_signed(&last_sk, section_auth.clone())?;
    let section_tree_update = SectionTreeUpdate::new(signed_sap, section_chain);
    let section_tree = if let Some(parent_section_tree) = parent_section_tree {
        parent_section_tree
    } else {
        SectionTree::new(genesis_ks.public_keys().public_key())
    };
    let section = NetworkKnowledge::new(section_tree, section_tree_update)?;

    let sks = bls::SecretKeySet::from_bytes(last_sk.to_bytes().to_vec())?;
    let section_key_share = create_section_key_share(&sks, share_index);
    Ok((section, section_key_share))
}

fn make_section_chain(
    genesis_key: &bls::SecretKey,
    other_keys: &Vec<bls::SecretKey>,
) -> Result<SectionsDAG> {
    let mut section_chain = SectionsDAG::new(genesis_key.public_key());
    let mut parent = genesis_key.clone();
    for key in other_keys {
        let sig = parent.sign(key.public_key().to_bytes());
        section_chain.insert(&parent.public_key(), key.public_key(), sig)?;
        parent = key.clone();
    }
    Ok(section_chain)
}
