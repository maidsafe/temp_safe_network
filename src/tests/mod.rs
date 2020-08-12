use crate::{Command, Config, Node, network::Network};
use crossbeam_channel::Sender;
use flexi_logger::{DeferredNow, Logger};
use log::{self, Record};
use quic_p2p::Config as NetworkConfig;
use routing::{Node as Routing, NodeConfig as RoutingConfig};
use std::cell::RefCell;
use std::io::Write;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread::{self, JoinHandle};

mod client;

struct InProcNetwork {
    vaults: Vec<(Sender<Command>, JoinHandle<()>)>,
}

fn init_logging(log_dir: Option<&PathBuf>) {
    // Custom formatter for logs
    let do_format = move |writer: &mut dyn Write, clock: &mut DeferredNow, record: &Record| {
        write!(
            writer,
            "{} {} [{}:{}] {}",
            record.level(),
            clock.now().to_rfc3339(),
            record.file().unwrap_or_default(),
            record.line().unwrap_or_default(),
            record.args()
        )
    };

    let logger = Logger::with_env().format(do_format).suppress_timestamp();

    let _ = if let Some(log_dir) = log_dir {
        logger.log_to_file().directory(log_dir)
    } else {
        logger
    }
    .start()
    .expect("Error when initialising logger");
}

impl InProcNetwork {
    pub fn new(no_of_vaults: usize) -> Self {
        let path = std::path::Path::new("vaults");
        std::fs::create_dir_all(&path).expect("Cannot create vaults directory");
        init_logging(Some(&path.into()));
        let mut vaults = Vec::new();
        let mut genesis_info: SocketAddr = "127.0.0.1:12000".parse().unwrap();
        let mut node_config = Config::default();
        node_config.set_flag("local", 1);
        node_config.listen_on_loopback();
        let (command_tx, command_rx) = crossbeam_channel::bounded(1);
        let mut genesis_config = node_config.clone();
        let handle = thread::spawn(move || {
            genesis_config.set_flag("first", 1);
            let path = path.join("genesis-vault");
            genesis_config.set_log_dir(&path);
            genesis_config.set_root_dir(&path);
            genesis_config.listen_on_loopback();

            let mut routing_config = RoutingConfig::default();
            routing_config.first = genesis_config.is_first();
            routing_config.transport_config = genesis_config.network_config().clone();

            let (routing, routing_rx, client_rx) = Routing::new(routing_config);
            let routing_layer = Rc::new(RefCell::new(routing));
            let routing = Network::new(routing_layer.clone());
            let receiver = crate::Receiver::new(routing_rx, client_rx, command_rx, routing_layer.clone());
            let mut node = Node::new(
                receiver,
                routing.clone(),
                &genesis_config,
                rand::thread_rng(),
            )
            .expect("Unable to start vault Node");
            node.run();
        });
        vaults.push((command_tx, handle));
        for i in 1..no_of_vaults {
            thread::sleep(std::time::Duration::from_secs(30));
            let (command_tx, command_rx) = crossbeam_channel::bounded(1);
            let mut vault_config = node_config.clone();
            let handle = thread::spawn(move || {
                let vault_path = path.join(format!("vault-{}", i));
                println!("Starting new vault: {:?}", &vault_path);
                vault_config.set_log_dir(&vault_path);
                vault_config.set_root_dir(&vault_path);
                
                let mut network_config = NetworkConfig::default();
                // println!("{:?}", &genesis_info);
                let _ = network_config
                    .hard_coded_contacts
                    .insert(genesis_info.clone());
                vault_config.set_network_config(network_config);
                vault_config.listen_on_loopback();

                let mut routing_config = RoutingConfig::default();
                routing_config.transport_config = vault_config.network_config().clone();
                // print!("{:?}", &routing_config.transport_config);

                let (routing, routing_rx, client_rx) = Routing::new(routing_config);
                let routing_layer = Rc::new(RefCell::new(routing));
                let routing = Network::new(routing_layer.clone());
                let receiver =
                    crate::Receiver::new(routing_rx, client_rx, command_rx, routing_layer.clone());
                let mut node = Node::new(receiver, routing, &vault_config, rand::thread_rng())
                    .expect("Unable to start vault Node");
                node.run();
            });
            vaults.push((command_tx, handle));
        }
        Self { vaults }
    }

    pub fn size(&self) -> usize {
        self.vaults.len()
    }
}

#[test]
fn start_network() {
    let network = Network::new(7);
    loop {
        
    }
}
