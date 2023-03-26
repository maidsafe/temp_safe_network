// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    cfg::keypair_storage::{
        get_reward_secret_key, store_network_keypair, store_new_reward_keypair,
    },
    flow_ctrl::{fault_detection::FaultsCmd, CmdCtrl, FlowCtrl},
    logging::log_system_details,
    CmdChannel, Config, Error, MyNode, NodeContext, NodeEventsChannel, Result,
    STANDARD_CHANNEL_SIZE,
};
use crate::UsedSpace;

use sn_comms::Comm;
use sn_interface::{
    network_knowledge::{NetworkKnowledge, SectionTree, MIN_ADULT_AGE},
    types::{keys::ed25519, log_markers::LogMarker},
};

use std::{path::Path, sync::Arc, time::Duration};
use tokio::{fs, sync::mpsc};
use xor_name::Prefix;

// Filename for storing the content of the genesis DBC.
// The Genesis DBC is generated and owned by the genesis PK of the network's section chain,
// i.e. the very first section key in the chain.
// The first node mints the genesis DBC (as a bearer DBC) and stores it in a file
// named `genesis_dbc`, located at it's configured root dir.
// In current implementation the amount owned by the Genesis DBC is
// set to GENESIS_DBC_AMOUNT (currently 4,525,524,120 * 10^9) individual units.
const GENESIS_DBC_FILENAME: &str = "genesis_dbc";

/// A reference to data structures to receive node's events,
/// and submit commands to be processed by it.
#[allow(missing_debug_implementations)]
pub struct NodeRef {
    /// Sender which can be used to add a Cmd to the Node's CmdQueue
    pub cmd_channel: CmdChannel,
    /// Channel to subscribe in order to receive node events
    pub events_channel: NodeEventsChannel,
    /// Snapshot of node's state
    pub context: NodeContext,
}

/// Start a new node.
pub async fn new_node(config: &Config, join_retry_timeout: Duration) -> Result<NodeRef> {
    let root_dir_buf = config.root_dir()?;
    let root_dir = root_dir_buf.as_path();
    fs::create_dir_all(root_dir).await?;

    let reward_secret_key = match get_reward_secret_key(root_dir).await? {
        Some(secret_key) => secret_key,
        None => {
            let secret_key = bls::SecretKey::random();
            store_new_reward_keypair(root_dir, &secret_key).await?;
            secret_key
        }
    };

    let used_space = UsedSpace::new(config.min_capacity(), config.max_capacity());

    start_node(
        config,
        used_space,
        root_dir,
        reward_secret_key,
        join_retry_timeout,
    )
    .await
}

// Private helper to create a new node using the given config and bootstraps it to the network.
async fn start_node(
    config: &Config,
    used_space: UsedSpace,
    root_storage_dir: &Path,
    reward_secret_key: bls::SecretKey,
    join_retry_timeout: Duration,
) -> Result<NodeRef> {
    let (fault_cmds_sender, fault_cmds_receiver) =
        mpsc::channel::<FaultsCmd>(STANDARD_CHANNEL_SIZE);

    let events_channel = NodeEventsChannel::default();
    let (comm, incoming_msg_receiver) = Comm::new(config.local_addr(), config.first())?;

    let node = if config.first().is_some() {
        start_genesis_node(
            comm,
            used_space,
            root_storage_dir,
            reward_secret_key,
            fault_cmds_sender.clone(),
            events_channel.clone(),
        )
        .await?
    } else {
        let node = start_normal_node(
            config,
            comm,
            used_space,
            root_storage_dir,
            reward_secret_key,
            fault_cmds_sender.clone(),
            events_channel.clone(),
        )
        .await?;
        info!("Node {:?} join has been accepted.", node.name());
        node
    };

    let node_name = node.name();
    let context = node.context();

    let (cmd_ctrl, data_replication_receiver) = CmdCtrl::new();
    let cmd_channel = FlowCtrl::start(
        node,
        cmd_ctrl,
        join_retry_timeout,
        incoming_msg_receiver,
        data_replication_receiver,
        (fault_cmds_sender, fault_cmds_receiver),
    )
    .await?;

    let root_dir_buf = config.root_dir()?;
    let root_dir = root_dir_buf.as_path();

    // Network keypair may have to be changed due to naming criteria or network requirements.
    let keypair_as_bytes = context.keypair.to_bytes();
    store_network_keypair(root_dir, keypair_as_bytes).await?;

    let our_pid = std::process::id();
    let node_prefix = context.network_knowledge.prefix();
    let node_age = context.info.age();
    let our_conn_info = context.info.addr;
    let info_msg = format!(
        "Node PID: {our_pid:?}, prefix: {node_prefix:?}, name: {node_name:?}, \
        age: {node_age}, connection info: {our_conn_info}",
    );
    println!("{info_msg}");
    info!("{info_msg}");

    log_system_details(node_prefix);

    Ok(NodeRef {
        cmd_channel,
        events_channel,
        context,
    })
}

async fn start_genesis_node(
    comm: Comm,
    used_space: UsedSpace,
    root_storage_dir: &Path,
    reward_secret_key: bls::SecretKey,
    fault_cmds_sender: mpsc::Sender<FaultsCmd>,
    node_events_sender: NodeEventsChannel,
) -> Result<MyNode> {
    // Genesis node having a fix age of 255.
    let keypair = ed25519::gen_keypair(&Prefix::default().range_inclusive(), 255);
    let node_name = ed25519::name(&keypair.public);

    info!(
        "{} Starting a new network as the genesis node (PID: {}).",
        node_name,
        std::process::id()
    );

    // Generate the genesis key, this will be the first key in the sections chain,
    // as well as the owner of the genesis DBC minted by this first node of the network.
    let genesis_sk_set = bls::SecretKeySet::random(0, &mut rand::thread_rng());
    let (node, genesis_dbc) = MyNode::first_node(
        comm,
        keypair,
        reward_secret_key,
        used_space.clone(),
        root_storage_dir.to_path_buf(),
        genesis_sk_set,
        fault_cmds_sender,
        node_events_sender,
    )?;

    // Write the genesis DBC to disk
    let path = root_storage_dir.join(GENESIS_DBC_FILENAME);
    fs::write(path, genesis_dbc.to_hex()?).await?;

    info!("{}", LogMarker::PromotedToElder);

    let genesis_key = node.network_knowledge.genesis_key();
    info!(
        "{} Genesis node started!. Genesis key {:?}, hex: {}",
        node_name,
        genesis_key,
        hex::encode(genesis_key.to_bytes())
    );

    Ok(node)
}

#[allow(clippy::too_many_arguments)]
async fn start_normal_node(
    config: &Config,
    comm: Comm,
    used_space: UsedSpace,
    root_storage_dir: &Path,
    reward_secret_key: bls::SecretKey,
    fault_cmds_sender: mpsc::Sender<FaultsCmd>,
    node_events_sender: NodeEventsChannel,
) -> Result<MyNode> {
    let keypair = ed25519::gen_keypair(&Prefix::default().range_inclusive(), MIN_ADULT_AGE);
    let node_name = ed25519::name(&keypair.public);
    info!("{} Bootstrapping as a new node.", node_name);
    let section_tree_path = config.network_contacts_file().ok_or_else(|| {
        Error::Configuration("Could not obtain network contacts file path".to_string())
    })?;
    let section_tree = SectionTree::from_disk(&section_tree_path).await?;
    let sap = section_tree.get_signed_by_name(&node_name)?;
    let network_knowledge = NetworkKnowledge::new(sap.prefix(), section_tree.clone())?;

    info!(
        "{} Starting a new node (PID: {}) with socket: {}, network's genesis key: {:?}",
        node_name,
        std::process::id(),
        comm.socket_addr(),
        section_tree.genesis_key()
    );

    let node = MyNode::new(
        comm,
        Arc::new(keypair),
        reward_secret_key,
        network_knowledge,
        None,
        used_space.clone(),
        root_storage_dir.to_path_buf(),
        fault_cmds_sender,
        node_events_sender,
    )?;

    info!("Node {} started.", node.info().name());

    Ok(node)
}
