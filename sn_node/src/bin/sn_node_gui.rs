use std::{net::SocketAddr, path::PathBuf, time::Duration};

use color_eyre::{Section, SectionExt};
use eframe::{
    egui::{self, TextEdit, Ui},
    emath::Vec2,
};
use eyre::{eyre, Result, WrapErr};
use file_rotate::{compression::Compression, suffix::AppendCount, ContentLimit};
use sn_node::{
    node::{
        add_connection_info, project_dirs, set_connection_info, Config, Error, EventStream, NodeApi,
    },
    LogFormatter,
};
use tokio::{
    runtime::{Handle, Runtime},
    time::sleep,
};
use tracing::{self, error, info, trace, warn, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

fn playground_root_dir() -> PathBuf {
    let mut root_dir = project_dirs().unwrap();
    root_dir.push("playground");
    root_dir
}

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    tracing_subscriber::fmt::init();
    if playground_root_dir().exists() {
        info!("Clearing out old playground {:?}", playground_root_dir());
        std::fs::remove_dir_all(playground_root_dir()).unwrap();
    }

    // let filter = match EnvFilter::try_from_env("RUST_LOG") {
    //     Ok(filter) => filter,
    //     // If we have an error (ie RUST_LOG not set or otherwise), we check the verbosity flags
    //     Err(_) => {
    //         // we manually determine level filter instead of using tracing EnvFilter.
    //         let level_filter = Level::TRACE;

    //         let module_filter = format!("{}={}", MODULE_NAME, level_filter)
    //             .parse()
    //             .wrap_err("BUG: invalid module filter constructed")?;
    //         EnvFilter::from_default_env().add_directive(module_filter)
    //     }
    // };

    // tracing_subscriber::fmt()
    //     .with_thread_names(true)
    //     .with_ansi(false)
    //     .with_env_filter(filter)
    //     .with_target(false)
    //     .event_format(LogFormatter::default())
    //     .init();

    let mut options = eframe::NativeOptions::default();
    options.initial_window_size = Some(Vec2::new(800.0, 600.0));
    eframe::run_native(
        "Safe Network Gui",
        options,
        Box::new(|_cc| Box::new(NetworkGui::default())),
    )
}

const MODULE_NAME: &str = "sn_node";
const BOOTSTRAP_RETRY_TIME_SEC: u64 = 10;

struct Node {
    api: NodeApi,
}

#[derive(Default)]
struct NodeSupervisor {
    config: Config,
    genesis_key_input: String,
    bootstrap_node_input: String,
    node: Option<Node>,
}

impl NodeSupervisor {
    fn new(config: Config) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    fn ui(&mut self, ui: &mut Ui) {
        ui.label("Config");
        ui.separator();
        egui::Grid::new("Config")
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("genesis key");
                if let Some(key) = self.config.genesis_key.clone() {
                    ui.label(format!("{}..", &key[..7.min(key.len())]));
                    if ui.button("unset").clicked() {
                        self.config.genesis_key = None
                    }
                } else {
                    ui.add(
                        TextEdit::singleline(&mut self.genesis_key_input)
                            .code_editor()
                            .hint_text("unset"),
                    );
                    if ui.button("set").clicked() {
                        self.config.genesis_key = Some(std::mem::take(&mut self.genesis_key_input));
                    }
                }

                ui.end_row();
                ui.label("first");
                ui.label(format!("{}", self.config.first));
                ui.checkbox(&mut self.config.first, "")
                    .on_hover_text("Check if this is the genesis node.");

                ui.end_row();
                ui.label("bootstrap nodes");
                ui.group(|ui| {
                    for addr in self.config.hard_coded_contacts.clone() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{:?}", addr));
                            if ui.button("delete").clicked() {
                                self.config.hard_coded_contacts.remove(&addr);
                            }
                        });
                    }
                });

                ui.text_edit_singleline(&mut self.bootstrap_node_input);
                if ui.button("add").clicked() {
                    if let Ok(addr) = self.bootstrap_node_input.parse::<SocketAddr>() {
                        self.config.hard_coded_contacts.insert(addr);
                        self.bootstrap_node_input.clear();
                    }
                }
            });
        ui.separator();
        ui.vertical(|ui| {
            ui.label("bootstrap nodes");
            for addr in self.config.hard_coded_contacts.clone() {
                ui.horizontal(|ui| {
                    ui.label(format!("{:?}", addr));
                    if ui.button("delete").clicked() {
                        self.config.hard_coded_contacts.remove(&addr);
                    }
                });
            }
            ui.text_edit_singleline(&mut self.bootstrap_node_input);
            if ui.button("add bootstrap node").clicked() {
                if let Ok(addr) = self.bootstrap_node_input.parse::<SocketAddr>() {
                    self.config.hard_coded_contacts.insert(addr);
                    self.bootstrap_node_input.clear();
                }
            }
        });

        ui.group(|ui| {
            ui.label("Node Actions");
            if ui.button("Start").clicked() {
                let bootstrap_retry_duration = Duration::from_secs(BOOTSTRAP_RETRY_TIME_SEC);
                let res = Handle::current()
                    .block_on(NodeApi::new(&self.config, bootstrap_retry_duration));
                match res {
                    Ok((api, mut event_stream)) => {
                        println!("Started");
                        tokio::task::spawn(async move {
                            println!("Started worker thread");
                            while let Some(event) = event_stream.next().await {
                                println!("Routing event! {:?}", event);
                            }
                            println!("Exiting worker thread");
                        });

                        self.node = Some(Node { api });
                    }
                    Err(err) => println!("Error Starting {err:#?}"),
                }
            }
        });

        if let Some(node) = self.node.as_mut() {
            ui.group(|ui| {
                ui.label("Node State");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("genesis key");
                    let genesis_key = Handle::current().block_on(node.api.genesis_key());
                    let encoded_key = hex::encode(genesis_key.to_bytes());
                    if ui
                        .button(format!("{}..", &encoded_key[..7.min(encoded_key.len())]))
                        .clicked()
                    {
                        ui.output().copied_text = encoded_key;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("connection info");
                    let node_addr = Handle::current().block_on(node.api.our_connection_info());
                    let formatted_node_addr = format!("{:?}", node_addr);
                    if ui.button(&formatted_node_addr).clicked() {
                        ui.output().copied_text = formatted_node_addr;
                    }
                });
            });
        } else {
            ui.label("Node Not Started Yet");
        }
    }

    async fn run_node() -> Result<()> {
        let config = Config::new().await?;

        // ==============
        // Set up logging
        // ==============

        {
            let filter = match EnvFilter::try_from_env("RUST_LOG") {
                Ok(filter) => filter,
                // If we have an error (ie RUST_LOG not set or otherwise), we check the verbosity flags
                Err(_) => {
                    // we manually determine level filter instead of using tracing EnvFilter.
                    let level_filter = config.verbose();
                    let module_filter = format!("{}={}", MODULE_NAME, level_filter)
                        .parse()
                        .wrap_err("BUG: invalid module filter constructed")?;
                    EnvFilter::from_default_env().add_directive(module_filter)
                }
            };

            println!("Starting logging to stdout");

            tracing_subscriber::fmt()
                .with_thread_names(true)
                .with_ansi(false)
                .with_env_filter(filter)
                .with_target(false)
                .event_format(LogFormatter::default())
                .init();
        }

        let message = format!("Running node v{}", env!("CARGO_PKG_VERSION"));
        info!("\n\n{}\n{}", message, "=".repeat(message.len()));

        let log = format!("The network is not accepting nodes right now. Retrying after {BOOTSTRAP_RETRY_TIME_SEC} seconds");

        let bootstrap_retry_duration = Duration::from_secs(BOOTSTRAP_RETRY_TIME_SEC);
        let (node, mut event_stream) = loop {
            match NodeApi::new(&config, bootstrap_retry_duration).await {
                Ok(result) => break result,
                Err(Error::CannotConnectEndpoint(qp2p::EndpointError::Upnp(error))) => {
                    return Err(error).suggestion(
                    "You can disable port forwarding by supplying --skip-auto-port-forwarding. Without port\n\
                    forwarding, your machine must be publicly reachable by the given\n\
                    --public-addr. If your machine is not publicly reachable, you may have to\n\
                    adjust your router settings to either:\n\
                    \n\
                    - Resolve the error (e.g. by enabling UPnP).\n\
                    - Manually configure port forwarding, such that your machine is publicly \
                      reachable, and supplying that address with --public-addr."
                        .header("Disable port forwarding or change your router settings"),
                );
                }
                Err(Error::TryJoinLater) => {
                    println!("{}", log);
                    info!("{}", log);
                }
                Err(Error::NodeNotReachable(addr)) => {
                    let err_msg = format!(
                    "Unfortunately we are unable to establish a connection to your machine ({}) either through a \
                    public IP address, or via IGD on your router. Please ensure that IGD is enabled on your router - \
                    if it is and you are still unable to add your node to the testnet, then skip adding a node for this \
                    testnet iteration. You can still use the testnet as a client, uploading and downloading content, etc. \
                    https://safenetforum.org/",
                    addr
                );
                    println!("{}", err_msg);
                    error!("{}", err_msg);
                }
                Err(Error::JoinTimeout) => {
                    let message = format!("Encountered a timeout while trying to join the network. Retrying after {BOOTSTRAP_RETRY_TIME_SEC} seconds.");
                    println!("{}", &message);
                    error!("{}", &message);
                }
                Err(e) => {
                    let log_path = if let Some(path) = config.log_dir() {
                        format!("{}", path.display())
                    } else {
                        "unknown".to_string()
                    };

                    return Err(e).wrap_err(format!(
                    "Cannot start node (log path: {}). If this is the first node on the network pass the local \
                    address to be used using --first", log_path)
                );
                }
            }
            sleep(bootstrap_retry_duration).await;
        };

        let our_conn_info = node.our_connection_info().await;

        if config.is_first() {
            let genesis_key = node.genesis_key().await;
            set_connection_info(genesis_key, our_conn_info)
                .await
                .unwrap_or_else(|err| {
                    error!("Unable to write our connection info to disk: {:?}", err);
                });
        } else {
            add_connection_info(our_conn_info)
                .await
                .unwrap_or_else(|err| {
                    error!("Unable to add our connection info to disk: {:?}", err);
                });
        }

        // This just keeps the node going as long as routing goes
        while let Some(event) = event_stream.next().await {
            trace!("Routing event! {:?}", event);
        }

        Ok(())
    }
}

#[derive(Default)]
struct NetworkGui {
    nodes: Vec<NodeSupervisor>,
}

impl eframe::App for NetworkGui {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Safe Network Playground");
        });

        for (i, node) in self.nodes.iter_mut().enumerate() {
            egui::Window::new(&format!("Node - {i}"))
                .collapsible(true)
                .resizable(true)
                .default_width(280.0)
                .show(ctx, |ui| node.ui(ui));
        }

        egui::Window::new("Actions")
            .collapsible(true)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    if ui.button("Spawn Node").clicked() {
                        println!("Spawning node");
                        let mut root_dir = playground_root_dir();
                        root_dir.push(format!("node-{}", self.nodes.len()));
                        let config = Config {
                            root_dir: Some(root_dir),
                            local_addr: Some("127.0.0.1:0".parse().unwrap()),
                            verbose: 4,
                            ..Config::default()
                        };
                        self.nodes.push(NodeSupervisor::new(config));
                    }
                });
            });
    }
}
