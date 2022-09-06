use crate::comm::Comm;
use crate::node::{
    cfg::create_test_max_capacity_and_root_storage,
    flow_ctrl::{
        dispatcher::Dispatcher,
        event_channel,
        event_channel::{EventReceiver, EventSender},
    },
    relocation_check, ChurnId, Node,
};
use crate::storage::UsedSpace;
use bls::Signature;
use ed25519_dalek::Keypair;
use eyre::{bail, eyre, Context, Result};
use sn_consensus::Decision;
use sn_interface::network_knowledge::SectionTree;
use sn_interface::{
    elder_count,
    messaging::{system::NodeState as NodeStateMsg, SectionTreeUpdate},
    network_knowledge::{
        test_utils::*, NetworkKnowledge, NodeInfo, NodeState, SectionAuthorityProvider,
        SectionKeyShare, SectionsDAG, MIN_ADULT_AGE,
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
    pub(crate) first_node: Option<NodeInfo>,
    pub(crate) sk_set: Option<SecretKeySet>,
    pub(crate) sap: Option<SectionAuthorityProvider>,
    pub(crate) custom_peer: Option<Peer>,
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
            sk_set: None,
            sap: None,
            custom_peer: None,
        }
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
        sk_set: SecretKeySet,
        first_node: NodeInfo,
    ) -> TestNodeBuilder {
        self.section = Some(section);
        self.sk_set = Some(sk_set);
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
    pub(crate) async fn build(self) -> Result<(Dispatcher, NetworkKnowledge, Peer, SecretKeySet)> {
        std::env::set_var("SN_DATA_COPY_COUNT", self.data_copy_count.to_string());
        let (section, section_key_share, keypair, peer, sk_set) =
            if let Some(custom_section) = self.section {
                let first_node = self.first_node.ok_or_else(|| {
                    eyre!("The first node must be provided when providing a custom section")
                })?;
                let sk_set = self.sk_set.ok_or_else(|| {
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
                let (sap, mut nodes, sk_set) = random_sap(
                    self.prefix,
                    self.elder_count,
                    self.adult_count,
                    Some(self.section_sk_threshold),
                );
                let (section, section_key_share) = create_section(&sk_set, &sap)?;
                let node = nodes.remove(0);
                let keypair = node.keypair.clone();
                (section, section_key_share, keypair, node.peer(), sk_set)
            };

        if let Some(custom_peer) = self.custom_peer {
            let node_state = NodeState::joined(custom_peer, None);
            let node_state = section_signed(sk_set.secret_key(), node_state)?;
            let _updated = section.update_member(node_state);
        }

        let (max_capacity, root_storage_dir) = create_test_max_capacity_and_root_storage()?;
        let comm = create_comm().await?;
        let node = Node::new(
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

pub(crate) fn create_section_with_random_sap(
    prefix: Prefix,
) -> Result<(NetworkKnowledge, SectionAuthorityProvider, SecretKeySet)> {
    let (sap, _, sk_set) = random_sap(prefix, elder_count(), 0, Some(0));
    let (section, _) = create_section(&sk_set, &sap)?;
    Ok((section, sap, sk_set))
}

/// Creates a section where all elders and adults are marked as joined members.
///
/// Can be used for tests requiring adults to be members of the section, e.g., when you expect
/// replication to occur after handling a message.
pub(crate) fn create_section(
    sk_set: &SecretKeySet,
    section_auth: &SectionAuthorityProvider,
) -> Result<(NetworkKnowledge, SectionKeyShare)> {
    let genesis_key = sk_set.public_keys().public_key();
    let section_tree_update = {
        let section_chain = SectionsDAG::new(genesis_key);
        let signed_sap = section_signed(sk_set.secret_key(), section_auth.clone())?;
        SectionTreeUpdate::new(signed_sap, section_chain)
    };
    let section = NetworkKnowledge::new(SectionTree::new(genesis_key), section_tree_update)?;

    for ns in section_auth.members() {
        let auth_ns = section_signed(sk_set.secret_key(), ns.clone())?;
        let _updated = section.update_member(auth_ns);
    }

    let section_key_share = create_section_key_share(sk_set, 0);

    Ok((section, section_key_share))
}

/// Creates a section where only elders are marked as joined members.
///
/// Some tests require the condition where only the elders were marked as joined members.
pub(crate) fn create_section_with_elders(
    sk_set: &SecretKeySet,
    section_auth: &SectionAuthorityProvider,
) -> Result<(NetworkKnowledge, SectionKeyShare)> {
    let genesis_key = sk_set.public_keys().public_key();
    let section_tree_update = {
        let section_chain = SectionsDAG::new(genesis_key);
        let signed_sap = section_signed(sk_set.secret_key(), section_auth.clone())?;
        SectionTreeUpdate::new(signed_sap, section_chain)
    };

    let section = NetworkKnowledge::new(SectionTree::new(genesis_key), section_tree_update)?;

    for peer in section_auth.elders() {
        let node_state = NodeState::joined(*peer, None);
        let node_state = section_signed(sk_set.secret_key(), node_state)?;
        let _updated = section.update_member(node_state);
    }

    let section_key_share = create_section_key_share(sk_set, 0);

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

pub(crate) fn create_section_auth() -> (SectionAuthorityProvider, Vec<NodeInfo>, SecretKeySet) {
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

pub(crate) fn gen_info(age: u8, prefix: Option<Prefix>) -> NodeInfo {
    NodeInfo::new(
        ed25519::gen_keypair(&prefix.unwrap_or_default().range_inclusive(), age),
        gen_addr(),
    )
}

pub(crate) async fn create_comm() -> Result<Comm> {
    let (tx, _rx) = mpsc::channel(TEST_EVENT_CHANNEL_SIZE);
    Ok(Comm::first_node((Ipv4Addr::LOCALHOST, 0).into(), Default::default(), tx).await?)
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
