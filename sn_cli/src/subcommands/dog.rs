// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#![cfg(feature = "data-network")]

use super::{
    helpers::{
        get_from_arg_or_stdin, get_target_url, print_nrs_map, serialise_output, xorname_to_hex,
    },
    OutputFmt,
};
use clap::Args;
use color_eyre::{eyre::eyre, Help, Result};
use sn_api::{
    resolver::{ContentType, SafeData},
    Safe, SafeUrl, XorName,
};
use tracing::debug;

#[derive(Args, Debug)]
pub struct DogCommands {
    /// The safe:// location to inspect
    location: Option<String>,
    /// Query all the data replicas matching the given indexes to check they hold a
    /// copy of the content. E.g. -r0 -r2 will query replicas at index 0 and 2.
    /// The network sorts nodes (data replicas) in a section by comparing the data's name with
    /// nodes' names, it's a measure of "closeness to the data" in XOR-namespace. The index of a
    /// node in such list is what it's referred as a data replica index. At least the
    /// first 'data_copy_count' nodes should hold this data, more may well hold it too.
    #[clap(short = 'r', long = "replicas")]
    replicas: Vec<usize>,
}

pub async fn dog_commander(cmd: DogCommands, output_fmt: OutputFmt, safe: &Safe) -> Result<()> {
    let link = get_from_arg_or_stdin(cmd.location, None)?;
    let url = get_target_url(&link)?;
    debug!("Running dog for: {url}");

    let mut replicas_indexes = cmd.replicas;
    replicas_indexes.sort();

    let resolved_content = safe.inspect(&url.to_string()).await?;
    if OutputFmt::Pretty != output_fmt {
        let replicas_report =
            if let Some(xorurl) = resolved_content.last().map(|content| content.xorurl()) {
                gen_replicas_report(safe, &xorurl, &replicas_indexes).await?
            } else {
                vec![]
            };

        println!(
            "{}",
            serialise_output(
                &(url.to_string(), resolved_content, replicas_report),
                output_fmt
            )
        );
    } else {
        let num_of_resolutions = resolved_content.len();
        for (i, ref content) in resolved_content.iter().enumerate() {
            println!();
            println!("== URL resolution step {} ==", i + 1);
            match content {
                SafeData::NrsMapContainer {
                    xorurl,
                    xorname,
                    type_tag,
                    nrs_map,
                    data_type,
                    ..
                } => {
                    println!("= NRS Map Container =");
                    println!("XOR-URL: {xorurl}");
                    println!("Type tag: {type_tag}");
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: {data_type}");
                    let mut safeurl = SafeUrl::from_url(xorurl)?;
                    safeurl.set_content_type(ContentType::Raw)?;
                    println!("Native data XOR-URL: {safeurl}");
                    print_nrs_map(nrs_map);
                }
                SafeData::NrsEntry {
                    xorurl,
                    public_name,
                    data_type,
                    resolves_into,
                    resolved_from,
                    version,
                } => {
                    println!("Resolved from: {resolved_from}");
                    println!("= NrsEntry =");
                    println!("Public name: {public_name}");
                    println!("Target XOR-URL: {xorurl}");
                    println!("Target native data type: {data_type}");
                    println!("Resolves into: {resolves_into}");
                    println!(
                        "Version: {}",
                        version.map_or("none".to_string(), |v| v.to_string())
                    );
                }
                SafeData::FilesContainer {
                    xorurl,
                    xorname,
                    type_tag,
                    version,
                    data_type,
                    resolved_from,
                    ..
                } => {
                    println!("Resolved from: {resolved_from}");
                    println!("= FilesContainer =");
                    println!("XOR-URL: {xorurl}");
                    println!(
                        "Version: {}",
                        version.map_or("none".to_string(), |v| v.to_string())
                    );
                    println!("Type tag: {type_tag}");
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: {data_type}");
                    let mut safeurl = SafeUrl::from_url(xorurl)?;
                    safeurl.set_content_type(ContentType::Raw)?;
                    println!("Native data XOR-URL: {safeurl}");
                }
                SafeData::PublicFile {
                    xorurl,
                    xorname,
                    media_type,
                    resolved_from,
                    ..
                } => {
                    println!("Resolved from: {resolved_from}");
                    println!("= File =");
                    println!("XOR-URL: {xorurl}");
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: PublicFile");
                    println!(
                        "Media type: {}",
                        media_type.clone().unwrap_or_else(|| "Unknown".to_string())
                    );
                }
                SafeData::SafeKey { .. } => {
                    println!("The SafeKey data type is not supported at the moment");
                }
                SafeData::Multimap {
                    xorurl,
                    resolved_from,
                    xorname,
                    type_tag,
                    ..
                } => {
                    let safeurl = SafeUrl::from_xorurl(xorurl)?;
                    println!("Resolved from: {resolved_from}");
                    if safeurl.content_type() == ContentType::Wallet {
                        println!("= Wallet =");
                    } else {
                        println!("= Multimap =");
                    }
                    println!("XOR-URL: {xorurl}");
                    println!("Type tag: {type_tag}");
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: Register");
                }
                SafeData::Register {
                    xorurl,
                    resolved_from,
                    xorname,
                    type_tag,
                    ..
                } => {
                    println!("Resolved from: {resolved_from}");
                    println!("= Register =");
                    println!("XOR-URL: {xorurl}");
                    println!("Type tag: {type_tag}");
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: Register");
                }
            }

            // If this is the last resolution step, and a set of replicas indexes was provided,
            // then query all data replicas and print out a report.
            if !replicas_indexes.is_empty() && i == num_of_resolutions - 1 {
                println!();
                println!("== Checking data replicas of resolved content ==");
                let xorurl = content.xorurl();
                println!("XOR-URL: {xorurl}");
                let replicas_report = gen_replicas_report(safe, &xorurl, &replicas_indexes).await?;

                println!("Replicas indexes queried: {replicas_indexes:?}");
                println!("Content composed of {} chunk/s:", replicas_report.len());
                for (chunk_name, outcomes) in replicas_report {
                    println!("= Chunk at XOR name 0x{} =", xorname_to_hex(&chunk_name));
                    for (replica_index, outcome) in outcomes {
                        if outcome.is_empty() {
                            println!("Replica #{replica_index}: Ok!");
                        } else {
                            println!("Replica #{replica_index}: {outcome}");
                        }
                    }
                    println!();
                }
            }
        }
        println!();
    }

    Ok(())
}

// Query data replicas and collect the outcomes
async fn gen_replicas_report(
    safe: &Safe,
    xorurl: &str,
    replicas_indexes: &[usize],
) -> Result<Vec<(XorName, Vec<(usize, String)>)>> {
    // If a set of replicas indexes was provided, let's obtain data from replicas
    if replicas_indexes.is_empty() {
        return Ok(vec![]);
    }

    let replicated_content = safe
        .check_replicas(xorurl, replicas_indexes)
        .await
        .map_err(|err| {
            eyre!(err)
                .wrap_err(format!(
                    "Could not check data replicas for content at {xorurl}."
                ))
                .suggestion("Try the command again with an appropriate Url.")
        })?;

    let report = replicated_content
        .into_iter()
        .map(|content| {
            let outcomes: Vec<(usize, String)> = content
                .outcomes
                .into_iter()
                .map(|(replica_index, outcome)| {
                    if let Err(err) = outcome {
                        (replica_index, format!("{err}"))
                    } else {
                        (replica_index, "".to_string())
                    }
                })
                .collect();

            (content.name, outcomes)
        })
        .collect();

    Ok(report)
}
