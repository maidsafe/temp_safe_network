// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use color_eyre::{eyre::bail, Result};
use regex::Regex;
use std::{
    fs::File,
    io::prelude::*,
    io::BufReader,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};
use tokio::time::{sleep, Duration};
use walkdir::WalkDir;

use safenode::safe_node_client::SafeNodeClient;
use safenode::{NodeInfoRequest, SectionMembersRequest};
use xor_name::{XorName, XOR_NAME_LEN};

// this would include code generated from .proto file
#[allow(unused_qualifications, clippy::unwrap_used)]
mod safenode {
    tonic::include_proto!("safenode");
}

const LOG_FILENAME_PREFIX: &str = "safenode.log";

#[derive(Debug)]
struct NodeInfo {
    pid: u32,
    name: String,
    addr: SocketAddr,
    log_path: PathBuf,
}

pub async fn run(logs_path: &Path, node_count: u32, nodes_launch_interval: u64) -> Result<()> {
    sleep(Duration::from_millis(nodes_launch_interval)).await;
    println!();
    println!("======== Verifying nodes ========");

    let expected_node_count = node_count as usize + 1; // we'll also check genesis node
    println!(
        "Checking nodes log files to verify all ({expected_node_count}) nodes \
        have joined. Logs path: {}",
        logs_path.display()
    );
    let mut nodes = nodes_info_from_logs(logs_path)?;

    println!("Number of nodes: {}", nodes.len());
    assert_eq!(
        expected_node_count,
        nodes.len(),
        "Unexpected number of joined nodes. Expected {}, we have {}",
        expected_node_count,
        nodes.len()
    );

    println!("All nodes have joined. Nodes IPs and names:");
    nodes.sort_by(|a, b| a.log_path.cmp(&b.log_path));
    for node in &nodes {
        println!(
            "{:>16} -> {} @ {}",
            node.addr,
            node.name,
            node.log_path.display()
        );
    }
    println!();

    // let's check all nodes have the same knowledge of the section's members
    let mut num_of_elders = 0;
    for node in &nodes {
        println!(
            "Checking name and network knowledge of node at {}, PID: {}, name: {}",
            node.addr, node.pid, node.name
        );

        let (name, is_elder, known_members) = {
            // assume their RPC interface is at the subsequent port number
            let mut addr = node.addr;
            addr.set_port(addr.port() + 1);
            send_rpc_query_to_node(addr).await?
        };

        if is_elder {
            num_of_elders += 1;
        }

        assert_eq!(
            node.name,
            format!("{name:?}"),
            "Node at {} reported a mismatching name: {name:?}",
            node.addr,
        );

        assert_eq!(
            known_members.len(),
            expected_node_count,
            "Node {} is aware of {} section member/s, expected: {expected_node_count}. Known members: {known_members:?}",
            node.name,
            known_members.len(),
        );

        let any_unknown_member = known_members.iter().find(|(name, _addr)| {
            nodes.iter().all(
                |node| node.name != format!("{name:?}"), /*&& &node.addr == addr*/
            )
        });
        if let Some((name, addr)) = any_unknown_member {
            bail!(
                "At least one section member known to node {} is not expected: \
                name = {name:?}, addr = {addr}",
                node.name,
            );
        }

        println!(
            "Name and network knowledge of node at {} is the expected!",
            node.addr
        );
        println!();
    }

    println!("Number of Elders: {num_of_elders}");
    let expected_num_elders = std::cmp::min(7, expected_node_count);
    assert_eq!(
        num_of_elders, expected_num_elders,
        "Mismatching number of Elders, found {num_of_elders}, expected {expected_num_elders}"
    );
    Ok(())
}

// Parse node logs files and extract info for each of them
fn nodes_info_from_logs(path: &Path) -> Result<Vec<NodeInfo>> {
    let mut nodes = Vec::<NodeInfo>::new();
    let re = Regex::new(
        r"Node PID: (\d+),.*name: (.{6}\(\d{8}\)..),.*connection info: ((127\.0\.0\.1|0\.0\.0\.0):\d{5})",
    )?;

    let log_files = WalkDir::new(path).into_iter().filter_map(|entry| {
        entry.ok().and_then(|f| {
            if f.file_type().is_file() {
                Some(f.into_path())
            } else {
                None
            }
        })
    });

    for file_path in log_files {
        let file_name = if let Some(name) = file_path.file_name().and_then(|s| s.to_str()) {
            name
        } else {
            println!("Failed to obtain filename from {}", file_path.display());
            continue;
        };

        if file_name.starts_with(LOG_FILENAME_PREFIX) {
            let file = File::open(&file_path)?;
            let lines = BufReader::new(file).lines();
            lines.filter_map(|item| item.ok()).for_each(|line| {
                if let Some(cap) = re.captures_iter(&line).next() {
                    let mut addr: SocketAddr = cap[3].parse().unwrap();
                    if addr.ip() == IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) {
                        addr.set_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
                    }
                    nodes.push(NodeInfo {
                        pid: cap[1].parse().unwrap(),
                        name: cap[2].to_string(),
                        addr,
                        log_path: file_path.parent().unwrap().to_path_buf(),
                    });
                }
            });
        }
    }

    Ok(nodes)
}

// Send RPC request to the node at the provided address,
// querying for its own name and section members it is aware of.
async fn send_rpc_query_to_node(
    addr: SocketAddr,
) -> Result<(XorName, bool, Vec<(XorName, SocketAddr)>)> {
    let url = format!("http://{addr}");
    println!("Connecting to node's RPC service at {url} ...");
    let mut client = SafeNodeClient::connect(url).await?;

    let request = tonic::Request::new(NodeInfoRequest {});
    let response = client.node_info(request).await?;
    let is_elder = response.get_ref().is_elder;
    let name = xorname_from_bytes(&response.get_ref().node_name);

    let request = tonic::Request::new(SectionMembersRequest {});
    let response = client.section_members(request).await?;
    let section_members = response
        .get_ref()
        .section_members
        .iter()
        .map(|p| (xorname_from_bytes(&p.node_name), p.addr.parse().unwrap()))
        .collect();

    Ok((name, is_elder, section_members))
}

fn xorname_from_bytes(bytes: &[u8]) -> XorName {
    let mut xorname = [0u8; XOR_NAME_LEN];
    bytes.iter().enumerate().for_each(|(i, b)| xorname[i] = *b);
    XorName(xorname)
}
