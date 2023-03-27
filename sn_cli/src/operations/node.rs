// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::subcommands::{
    helpers::{serialise_output, xorname_to_hex},
    OutputFmt,
};

use color_eyre::Result;
use comfy_table::Table;
use futures_util::StreamExt;
use std::{net::SocketAddr, time::Duration};
use tonic::Request;
use xor_name::{XorName, XOR_NAME_LEN};

use safenode::safe_node_client::SafeNodeClient;
use safenode::{
    NodeEventsRequest, NodeInfoRequest, RestartRequest, SectionMembersRequest, StopRequest,
    UpdateRequest,
};

// this would include code generated from .proto file
#[allow(unused_qualifications, clippy::unwrap_used)]
mod safenode {
    tonic::include_proto!("safenode");
}

pub async fn node_info(addr: SocketAddr, output_fmt: OutputFmt) -> Result<()> {
    let endpoint = format!("http://{addr}");
    let mut client = SafeNodeClient::connect(endpoint.clone()).await?;
    let response = client.node_info(Request::new(NodeInfoRequest {})).await?;
    let node_info = response.get_ref();
    let name = xorname_from_bytes(&node_info.node_name);

    if OutputFmt::Pretty == output_fmt {
        println!("Node info received:");
        println!("RPC endpoint: {endpoint}");
        println!("Node name: {name:?}");
        println!("Is Elder: {}", node_info.is_elder);
        println!("Logs dir: {}", node_info.log_dir);
        println!("Binary version: {}", node_info.bin_version);
    } else {
        println!(
            "{}",
            serialise_output(
                &(
                    endpoint,
                    xorname_to_hex(&name),
                    node_info.is_elder,
                    &node_info.log_dir,
                    &node_info.bin_version
                ),
                output_fmt
            )
        );
    }

    Ok(())
}

pub async fn section_members(addr: SocketAddr, output_fmt: OutputFmt) -> Result<()> {
    let mut client = SafeNodeClient::connect(format!("http://{addr}")).await?;
    let response = client
        .section_members(Request::new(SectionMembersRequest {}))
        .await?;
    let members = response.get_ref().section_members.iter().map(|member| {
        (
            xorname_from_bytes(&member.node_name),
            member.is_elder,
            member.addr.clone(),
        )
    });

    if OutputFmt::Pretty == output_fmt {
        let members_len = members.len();
        let mut table = Table::new();
        table.add_row(&vec!["Node name", "Is Elder?", "Address"]);
        for (name, is_elder, addr) in members {
            table.add_row(&vec![format!("{name:?}"), is_elder.to_string(), addr]);
        }

        println!("The node is currently aware of {members_len} section members:",);
        println!("{table}");
    } else {
        let members_vec: Vec<_> = members
            .map(|(name, is_elder, addr)| (xorname_to_hex(&name), is_elder, addr))
            .collect();
        println!("{}", serialise_output(&members_vec, output_fmt));
    }

    Ok(())
}

pub async fn node_events(addr: SocketAddr, output_fmt: OutputFmt) -> Result<()> {
    let mut client = SafeNodeClient::connect(format!("http://{addr}")).await?;
    let response = client
        .node_events(Request::new(NodeEventsRequest {}))
        .await?;

    println!("Listening to node events... (press Ctrl+C to exit)");
    let mut stream = response.into_inner();
    while let Some(Ok(e)) = stream.next().await {
        if OutputFmt::Pretty == output_fmt {
            println!("New event received: {}", e.event);
        } else {
            println!("{}", e.event);
        }
    }

    Ok(())
}

pub async fn node_restart(addr: SocketAddr, delay_millis: u64) -> Result<()> {
    let mut client = SafeNodeClient::connect(format!("http://{addr}")).await?;
    let _response = client
        .restart(Request::new(RestartRequest { delay_millis }))
        .await?;
    println!(
        "Node successfully received the request to restart in {:?}",
        Duration::from_millis(delay_millis)
    );
    Ok(())
}

pub async fn node_stop(addr: SocketAddr, delay_millis: u64) -> Result<()> {
    let mut client = SafeNodeClient::connect(format!("http://{addr}")).await?;
    let _response = client
        .stop(Request::new(StopRequest { delay_millis }))
        .await?;
    println!(
        "Node successfully received the request to stop in {:?}",
        Duration::from_millis(delay_millis)
    );
    Ok(())
}

pub async fn node_update(addr: SocketAddr, delay_millis: u64) -> Result<()> {
    let mut client = SafeNodeClient::connect(format!("http://{addr}")).await?;
    let _response = client
        .update(Request::new(UpdateRequest { delay_millis }))
        .await?;
    println!(
        "Node successfully received the request to try to update in {:?}",
        Duration::from_millis(delay_millis)
    );
    Ok(())
}

fn xorname_from_bytes(bytes: &[u8]) -> XorName {
    let mut xorname = [0u8; XOR_NAME_LEN];
    bytes.iter().enumerate().for_each(|(i, b)| xorname[i] = *b);
    XorName(xorname)
}
