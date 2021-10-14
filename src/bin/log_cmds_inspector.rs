use eyre::{eyre, Result};
use regex::Regex;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufRead},
    path::Path,
};

fn main() -> Result<()> {
    let logs_path = std::env::args().nth(1).ok_or_else(|| {
        eyre!(
        "Please provide the path to the testnet logs folder, e.g. ~/.safe/node/local-test-network"
    )
    })?;

    // Container to build a summary of issues detected
    let logs_with_issues = inspect_log_files(logs_path)?;

    println!();
    if logs_with_issues.is_empty() {
        println!("** No errors detected in any of the logs scanned! **");
    } else {
        println!("** The following issues were detected **");
        for (logfile, log_entries) in logs_with_issues.iter() {
            println!("Commands not completed in log {}:", logfile);
            for log_entry in log_entries.iter() {
                println!("{}", log_entry);
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

fn inspect_log_files(logs_path: String) -> Result<BTreeMap<String, Vec<String>>> {
    println!("Inspecting testnet logs folder: {}", logs_path);
    let logs_dir = std::fs::read_dir(logs_path)?;

    let mut issues = BTreeMap::<String, Vec<String>>::new();

    // Set of regex to scan trace logs we are interested in
    let regex_cmd_spawned = Regex::new(r".*CommandHandleSpawned.*cmd_id=([^\s-]*).*")?;
    let regex_cmd_start = Regex::new(r".*CommandHandleStart.*cmd_id=(.*)$")?;
    let regex_cmd_end = Regex::new(r".*CommandHandleEnd.*cmd_id=(.*)$")?;
    let regex_cmd_error = Regex::new(r".*CommandHandleError.*cmd_id=(.*)$")?;
    let regex_node_is_elder = Regex::new(r".*elder=true.*")?;

    // Iterate over each of the testnet nodes log files
    for node_log_path in logs_dir {
        let node_log_filepath = node_log_path?.path();
        println!();
        println!("-------------------------");
        println!("Inspecting node's folder: {}", node_log_filepath.display());
        println!("-------------------------");
        let logs_paths = std::fs::read_dir(&node_log_filepath)?;

        // Maps from cmd_id to log entry
        let mut cmd_spawned = BTreeMap::<String, String>::new();
        let mut cmd_start = BTreeMap::<String, String>::new();
        let mut cmd_end = BTreeMap::<String, String>::new();
        let mut cmd_failed = BTreeMap::<String, String>::new();

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
            // spawned/started/ended/failed by this node
            match read_lines(&log_file_path) {
                Ok(lines) => {
                    for log_entry in lines.flatten() {
                        if regex_node_is_elder.is_match(&log_entry) {
                            is_elder = true;
                        }

                        if let Some(cap) = regex_cmd_spawned.captures_iter(&log_entry).next() {
                            cmd_spawned.insert(cap[1].to_string(), log_entry);
                        } else if let Some(cap) = regex_cmd_start.captures_iter(&log_entry).next() {
                            cmd_start.insert(cap[1].to_string(), log_entry);
                        } else if let Some(cap) = regex_cmd_end.captures_iter(&log_entry).next() {
                            cmd_end.insert(cap[1].to_string(), log_entry);
                        } else if let Some(cap) = regex_cmd_error.captures_iter(&log_entry).next() {
                            cmd_failed.insert(cap[1].to_string(), log_entry);
                        }
                    }
                }
                Err(err) => println!("ERROR: {:?}", err),
            }
        }

        println!("Node is Elder?: {}", is_elder);
        println!(
            "Commands Spawned: {}, Started: {}, Succeeded: {}, Failed: {}",
            cmd_spawned.len(),
            cmd_start.len(),
            cmd_end.len(),
            cmd_failed.len()
        );

        // We can now try to find inconsistencies among the commands,
        // trying to find those which were started but not completed
        let mut cmds_with_error = 0;
        let mut cmds_not_completed = 0;
        let mut cmds_with_end = 0;
        for (cmd_id, log_entry) in cmd_spawned.iter() {
            if cmd_start.get(cmd_id).is_none() {
                println!(
                    "Command with id {} spawned but not started: {}",
                    cmd_id, log_entry
                );
                cmds_not_completed += 1;
                let logfile = node_log_filepath.display().to_string();
                issues
                    .entry(logfile)
                    .or_insert_with(Vec::new)
                    .push(log_entry.clone());
            } else {
                // Command spwned and started, let's see if it completed...
                if cmd_end.get(cmd_id).is_none() {
                    // it didn't finish succesfully, did it fail?....
                    if cmd_failed.get(cmd_id).is_some() {
                        cmds_with_error += 1;
                    } else {
                        println!(
                            "Command with id {} spawned and started, but not completed: {}",
                            cmd_id, log_entry
                        );
                        cmds_not_completed += 1;
                        let logfile = node_log_filepath.display().to_string();
                        issues
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

        if cmds_not_completed > 0 {
            println!(
                "\n!!! ERROR !!!: Some command/s were not completed in log: {}",
                node_log_filepath.display()
            );
        }
    }

    Ok(issues)
}
