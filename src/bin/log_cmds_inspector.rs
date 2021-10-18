use eyre::Result;
use regex::Regex;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead},
    path::Path,
};
use structopt::{clap::AppSettings::ColoredHelp, StructOpt};

#[derive(StructOpt, Debug)]
/// Inspect Safe Network local testnet logs
#[structopt(global_settings(&[ColoredHelp]))]
struct CmdArgs {
    /// subcommands
    #[structopt(subcommand)]
    pub cmd: SubCommands,
    /// Path to the testnet logs folder, e.g. ~/.safe/node/local-test-network
    pub logs_path: String,
}

#[derive(StructOpt, Debug)]
enum SubCommands {
    /// Generate a report of commands and corresponding sub-commands
    // TODO: make the cmd-id optional, to report all commands
    Commands {
        /// ID of the command to obtain a report for, e.g. 924678512
        cmd_id: String,
    },
    /// Generate a report of commands dispatched to process incoming messages
    // TODO: make the msg-id optional, to report all messages
    Messages {
        /// ID of the message to obtain a report for, e.g. c971..cfb5
        msg_id: String,
    },
    /// Generate a report of commands dispatched, and which were started but not completed
    CompletedCmds,
}

fn main() -> Result<()> {
    let args = CmdArgs::from_args();
    let report = inspect_log_files(&args)?;

    println!();
    match args.cmd {
        SubCommands::Commands { cmd_id } => {
            println!(
                "*** REPORT: The following commands were found for cmd id {} ***",
                cmd_id
            );
            for (cmd_id, log_entries) in report.iter() {
                println!("==> Log entries for sub-command {}:", cmd_id);
                for log_entry in log_entries.iter() {
                    println!("{}", log_entry);
                }
                println!();
            }
        }
        SubCommands::Messages { msg_id } => {
            println!(
                "*** REPORT: The following commands were found for msg id {} ***",
                msg_id
            );
            for (cmd_id, log_entries) in report.iter() {
                println!("==> Log entries for sub-command {}:", cmd_id);
                for log_entry in log_entries.iter() {
                    println!("{}", log_entry);
                }
                println!();
            }
        }
        SubCommands::CompletedCmds => {
            if report.is_empty() {
                println!("** No errors detected in any of the logs scanned! **");
            } else {
                println!("*** REPORT: The following issues were detected ***");
                for (logfile, log_entries) in report.iter() {
                    println!("Commands not completed in log {}:", logfile);
                    for log_entry in log_entries.iter() {
                        println!("{}", log_entry);
                    }
                }
            }
        }
    }

    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Default)]
// Maps from cmd_id to log entry
struct ScannedCommands {
    pub spawned: BTreeMap<String, BTreeMap<String, String>>,
    pub started: BTreeMap<String, BTreeMap<String, String>>,
    pub succeeded: BTreeMap<String, BTreeMap<String, String>>,
    pub failed: BTreeMap<String, BTreeMap<String, String>>,
    pub handle_msg: BTreeMap<String, String>,
    // Map from msg_id to cmd_ids
    pub msg_id_to_cmds: BTreeMap<String, Vec<String>>,
}

fn inspect_log_files(args: &CmdArgs) -> Result<BTreeMap<String, Vec<String>>> {
    println!("Inspecting testnet logs folder: {}", args.logs_path);
    let logs_dir = std::fs::read_dir(args.logs_path.clone())?;

    let mut report = BTreeMap::<String, Vec<String>>::new();

    // Set of regex to scan trace logs we are interested in
    let regex_cmd_spawned =
        Regex::new(r".*CommandHandleSpawned.*cmd_id=([^\s-]*)($|.*MessageId\((.*)\)$)")?;
    let regex_cmd_start = Regex::new(r".*CommandHandleStart.*cmd_id=(.*)$")?;
    let regex_cmd_end = Regex::new(r".*CommandHandleEnd.*cmd_id=(.*)$")?;
    let regex_cmd_error = Regex::new(r".*CommandHandleError.*cmd_id=(.*)$")?;
    let regex_handle_msg = Regex::new(r".*DispatchHandleMsgCmd.*msg_id=MessageId\((.*)\)$")?;
    let regex_node_is_elder = Regex::new(r".*elder=true.*")?;

    // Iterate over each of the testnet nodes log files
    for node_log_path in logs_dir {
        let node_log_filepath = node_log_path?.path();
        println!();
        println!("-------------------------");
        println!("Inspecting node's folder: {}", node_log_filepath.display());
        println!("-------------------------");
        let logs_paths = std::fs::read_dir(&node_log_filepath)?;

        let mut commands = ScannedCommands::default();

        let mut is_elder = false;

        // Inspect each of the log files for current node's log
        for path in logs_paths {
            let log_file_path = path.unwrap().path();
            match log_file_path.file_name() {
                Some(file_name) if file_name.to_str().unwrap().starts_with("sn_node.log.") => {
                    println!("Inspecting log file: {}", log_file_path.display());
                }
                _ => {
                    // Ignoring this file which is not a log file
                    continue;
                }
            }

            // Apply all regex to this log entry to keep track what commands where
            // spawned/started/succeeded/failed by this node
            match read_lines(&log_file_path) {
                Ok(lines) => {
                    for log_entry in lines.flatten() {
                        if regex_node_is_elder.is_match(&log_entry) {
                            is_elder = true;
                        }

                        if let Some(cap) = regex_cmd_spawned.captures_iter(&log_entry).next() {
                            let cmd_id = cap[1].to_string();
                            let root_cmd_id = get_root_cmd_id(&cmd_id);
                            commands
                                .spawned
                                .entry(root_cmd_id.clone())
                                .or_insert_with(BTreeMap::new)
                                .insert(cmd_id.clone(), log_entry.clone());

                            if let Some(msg_id) = cap.get(3) {
                                commands
                                    .msg_id_to_cmds
                                    .entry(msg_id.as_str().to_string())
                                    .or_insert_with(Vec::new)
                                    .push(root_cmd_id);
                            }
                        } else if let Some(cap) = regex_cmd_start.captures_iter(&log_entry).next() {
                            let cmd_id = cap[1].to_string();
                            let root_cmd_id = get_root_cmd_id(&cmd_id);
                            commands
                                .started
                                .entry(root_cmd_id)
                                .or_insert_with(BTreeMap::new)
                                .insert(cmd_id, log_entry.clone());
                        } else if let Some(cap) = regex_cmd_end.captures_iter(&log_entry).next() {
                            let cmd_id = cap[1].to_string();
                            let root_cmd_id = get_root_cmd_id(&cmd_id);
                            commands
                                .succeeded
                                .entry(root_cmd_id)
                                .or_insert_with(BTreeMap::new)
                                .insert(cmd_id, log_entry.clone());
                        } else if let Some(cap) = regex_cmd_error.captures_iter(&log_entry).next() {
                            let cmd_id = cap[1].to_string();
                            let root_cmd_id = get_root_cmd_id(&cmd_id);
                            commands
                                .failed
                                .entry(root_cmd_id)
                                .or_insert_with(BTreeMap::new)
                                .insert(cmd_id, log_entry.clone());
                        } else if let Some(cap) = regex_handle_msg.captures_iter(&log_entry).next()
                        {
                            let msg_id = cap[1].to_string();
                            commands.handle_msg.insert(msg_id, log_entry);
                        }
                    }
                }
                Err(err) => println!("ERROR: {:?}", err),
            }
        }

        println!("Node is Elder?: {}", is_elder);
        println!(
            "Commands Spawned: {}, Started: {}, Succeeded: {}, Failed: {}",
            commands.spawned.len(),
            commands.started.len(),
            commands.succeeded.len(),
            commands.failed.len()
        );

        match args.cmd {
            SubCommands::Commands { ref cmd_id } => {
                populate_commands_tree(&commands, &mut report, cmd_id);
            }
            SubCommands::Messages { ref msg_id } => {
                populate_commands_tree_for_msgs(&commands, &mut report, msg_id);
            }
            SubCommands::CompletedCmds => {
                check_completed_cmds(&commands, &mut report, &node_log_filepath);
            }
        }
    }

    Ok(report)
}

// Try to find inconsistencies among the commands,
// trying to find those which were started but not completed
fn check_completed_cmds(
    commands: &ScannedCommands,
    report: &mut BTreeMap<String, Vec<String>>,
    node_log_filepath: &Path,
) {
    let mut cmds_with_error = 0;
    let mut cmds_not_completed = 0;
    let mut cmds_with_end = 0;
    for (cmd_id, log_entry) in commands.spawned.iter().flat_map(|(_, v)| v.iter()) {
        let root_cmd_id = get_root_cmd_id(cmd_id);
        if commands
            .started
            .get(&root_cmd_id)
            .map(|subcmds| subcmds.get(cmd_id))
            .is_none()
        {
            println!(
                "Command with id {} spawned but not started: {}",
                cmd_id, log_entry
            );
            cmds_not_completed += 1;
            let logfile = node_log_filepath.display().to_string();
            report
                .entry(logfile)
                .or_insert_with(Vec::new)
                .push(log_entry.clone());
        } else {
            // Command spwned and started, let's see if it completed...
            if commands
                .succeeded
                .get(&root_cmd_id)
                .map(|subcmds| subcmds.get(cmd_id))
                .is_none()
            {
                // it didn't finish succesfully, did it fail?....
                if commands
                    .failed
                    .get(&root_cmd_id)
                    .map(|subcmds| subcmds.get(cmd_id))
                    .is_none()
                {
                    cmds_with_error += 1;
                } else {
                    println!(
                        "Command with id {} spawned and started, but not completed: {}",
                        cmd_id, log_entry
                    );
                    cmds_not_completed += 1;
                    let logfile = node_log_filepath.display().to_string();
                    report
                        .entry(logfile)
                        .or_insert_with(Vec::new)
                        .push(log_entry.clone());
                }
            } else {
                // command completed
                cmds_with_end += 1;
            }
        }
    }

    println!(
        "Commands handled which Failed: {}, Succeeded: {}, not Completed: {}",
        cmds_with_error, cmds_with_end, cmds_not_completed
    );
    println!(
        "Incoming messages handled: {}",
        commands.msg_id_to_cmds.len()
    );

    if cmds_not_completed > 0 {
        println!(
            "\n!!! ERROR !!!: Some command/s were not completed in log: {}",
            node_log_filepath.display()
        );
    }
}

// Populate the report with the list of commands/sub-commands correlated to the provided cmd id.
fn populate_commands_tree(
    commands: &ScannedCommands,
    report: &mut BTreeMap<String, Vec<String>>,
    cmd_id: &str,
) {
    println!("Looking for commands spawned from cmd id {}", cmd_id);
    let root_cmd_id = get_root_cmd_id(cmd_id);
    if let Some(matching_cmds) = commands.spawned.get(&root_cmd_id) {
        matching_cmds.iter().for_each(|(cmd_id, log_entry)| {
            report
                .entry(cmd_id.clone())
                .or_insert_with(Vec::new)
                .push(log_entry.clone())
        });
    }

    if let Some(matching_cmds) = commands.started.get(&root_cmd_id) {
        matching_cmds.iter().for_each(|(cmd_id, log_entry)| {
            report
                .entry(cmd_id.clone())
                .or_insert_with(Vec::new)
                .push(log_entry.clone())
        });
    }

    if let Some(matching_cmds) = commands.succeeded.get(&root_cmd_id) {
        matching_cmds.iter().for_each(|(cmd_id, log_entry)| {
            report
                .entry(cmd_id.clone())
                .or_insert_with(Vec::new)
                .push(log_entry.clone())
        });
    }

    if let Some(matching_cmds) = commands.failed.get(&root_cmd_id) {
        matching_cmds.iter().for_each(|(cmd_id, log_entry)| {
            report
                .entry(cmd_id.clone())
                .or_insert_with(Vec::new)
                .push(log_entry.clone())
        });
    }
}

// Populate the report with the list of message ids and their correlated commands.
fn populate_commands_tree_for_msgs(
    commands: &ScannedCommands,
    report: &mut BTreeMap<String, Vec<String>>,
    msg_id: &str,
) {
    println!("Looking for commands spawned from msg id {}", msg_id);
    commands.handle_msg.get(msg_id).map(|log_entry| {
        commands.msg_id_to_cmds.get(msg_id).map(|ids| {
            ids.iter().for_each(|root_cmd_id| {
                report
                    .entry(root_cmd_id.clone())
                    .or_insert_with(Vec::new)
                    .push(log_entry.clone());

                populate_commands_tree(commands, report, root_cmd_id);
            })
        })
    });
}

// Given a command id, return the root id, e.g. the root cmd id of 'abc.1.0.2' is 'a.b.c'.
fn get_root_cmd_id(cmd_id: &str) -> String {
    let mut root_cmd_id = cmd_id.to_string();
    root_cmd_id.truncate(cmd_id.find('.').unwrap_or(cmd_id.len()));
    root_cmd_id
}
