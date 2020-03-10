// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    helpers::{get_from_arg_or_stdin, serialise_output, xorname_to_hex},
    OutputFmt,
};
use log::debug;
use prettytable::Table;
use safe_api::{fetch::NrsMapContainerInfo, fetch::SafeData, Safe};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct DogCommands {
    /// The safe:// location to inspect
    location: Option<String>,
}

pub async fn dog_commander(
    cmd: DogCommands,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    let url = get_from_arg_or_stdin(cmd.location, None)?;
    debug!("Running dog for: {:?}", &url);

    let content = safe.inspect(&url).await?;
    match &content {
        SafeData::FilesContainer {
            xorurl,
            version,
            type_tag,
            xorname,
            data_type,
            resolved_from,
            ..
        } => {
            if OutputFmt::Pretty == output_fmt {
                println!("Native data type: {}", data_type);
                println!("Version: {}", version);
                println!("Type tag: {}", type_tag);
                println!("XOR name: 0x{}", xorname_to_hex(xorname));
                println!("XOR-URL: {}", xorurl);
                print_resolved_from(100, resolved_from);
            } else if resolved_from.is_some() {
                println!("{}", serialise_output(&(url, content), output_fmt));
            } else {
                let jsonv = serde_json::json!([
                    url,
                    {
                        "data_type": data_type,
                        "version": version,
                        "type_tag": type_tag,
                        "xorname": xorname_to_hex(xorname)
                    }
                ]);
                println!("{}", serialise_output(&jsonv, output_fmt));
            }
        }
        SafeData::PublishedImmutableData {
            xorurl,
            xorname,
            resolved_from,
            media_type,
            ..
        } => {
            if OutputFmt::Pretty == output_fmt {
                println!("Native data type: ImmutableData (published)");
                println!("XOR name: 0x{}", xorname_to_hex(xorname));
                println!("XOR-URL: {}", xorurl);
                println!(
                    "Media type: {}",
                    media_type.clone().unwrap_or_else(|| "Unknown".to_string())
                );
                print_resolved_from(100, resolved_from);
            } else if resolved_from.is_some() {
                println!("{}", serialise_output(&(url, content), output_fmt));
            } else {
                let jsonv = serde_json::json!([
                    url,
                    {
                        "data_type": "PublishedImmutableData",
                        "media_type": media_type.clone().unwrap_or_else(|| "Unknown".to_string()),
                        "xorname": xorname_to_hex(xorname)
                    }
                ]);
                println!("{}", serialise_output(&jsonv, output_fmt));
            }
        }
        SafeData::Wallet {
            xorurl,
            xorname,
            type_tag,
            data_type,
            resolved_from,
            ..
        } => {
            if OutputFmt::Pretty == output_fmt {
                println!("Native data type: {}", data_type);
                println!("Type tag: {}", type_tag);
                println!("XOR name: 0x{}", xorname_to_hex(xorname));
                println!("XOR-URL: {}", xorurl);
                print_resolved_from(100, resolved_from);
            } else if resolved_from.is_some() {
                println!("{}", serialise_output(&(url, content), output_fmt));
            } else {
                let jsonv = serde_json::json!([
                    url,
                    {
                        "data_type": data_type,
                        "type_type": type_tag,
                        "xorname": xorname_to_hex(xorname)
                    }
                ]);
                println!("{}", serialise_output(&jsonv, output_fmt));
            }
        }
        SafeData::SafeKey {
            xorurl,
            xorname,
            resolved_from,
        } => {
            if OutputFmt::Pretty == output_fmt {
                println!("Native data type: SafeKey");
                println!("XOR name: 0x{}", xorname_to_hex(xorname));
                println!("XOR-URL: {}", xorurl);
                print_resolved_from(100, resolved_from);
            } else if resolved_from.is_some() {
                println!("{}", serialise_output(&(url, content), output_fmt));
            } else {
                let jsonv = serde_json::json!([
                    url,
                    {
                        "data_type": "SafeKey",
                        "xorname": xorname_to_hex(xorname)
                    }
                ]);
                println!("{}", serialise_output(&jsonv, output_fmt));
            }
        }
    }

    Ok(())
}

pub fn print_resolved_from(info_level: u8, resolved_from: &Option<NrsMapContainerInfo>) {
    if info_level > 1 {
        if let Some(nrs_map_container) = resolved_from {
            // print out the resolved_from info since it's --info level 2
            println!();
            println!("Resolved using NRS Map:");
            println!("PublicName: \"{}\"", nrs_map_container.public_name);
            println!("Container XOR-URL: {}", nrs_map_container.xorurl);
            println!("Native data type: {}", nrs_map_container.data_type);
            println!("Type tag: {}", nrs_map_container.type_tag);
            println!("XOR name: 0x{}", xorname_to_hex(&nrs_map_container.xorname));
            println!("Version: {}", nrs_map_container.version);

            if info_level > 2 {
                let mut table = Table::new();
                table.add_row(
                    row![bFg->"NRS name/subname", bFg->"Created", bFg->"Modified", bFg->"Link"],
                );

                let summary = nrs_map_container.nrs_map.get_map_summary();
                summary.iter().for_each(|(name, rdf_info)| {
                    table.add_row(row![
                        format!("{}{}", name, nrs_map_container.public_name),
                        rdf_info["created"],
                        rdf_info["modified"],
                        rdf_info["link"],
                    ]);
                });
                table.printstd();
            }
        }
    }
}
