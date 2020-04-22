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
use safe_api::{fetch::SafeData, Safe};
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
    for (i, ref c) in content.iter().enumerate() {
        println!();
        println!("== URL resolution step {} ==", i + 1);
        match c {
            SafeData::NrsMapContainer {
                public_name,
                xorurl,
                xorname,
                type_tag,
                version,
                nrs_map,
                data_type,
                resolved_from,
            } => {
                if resolved_from != xorurl {
                    println!("Resolved from: {}", resolved_from);
                }
                println!("= NRS Map Container =");
                println!("PublicName: \"{}\"", public_name);
                println!("XOR-URL: {}", xorurl);
                println!("Version: {}", version);
                println!("Type tag: {}", type_tag);
                println!("XOR name: 0x{}", xorname_to_hex(xorname));
                println!("Native data type: {}", data_type);

                let mut table = Table::new();
                table.add_row(
                    row![bFg->"NRS name/subname", bFg->"Created", bFg->"Modified", bFg->"Link"],
                );

                let summary = nrs_map.get_map_summary();
                summary.iter().for_each(|(name, rdf_info)| {
                    table.add_row(row![
                        format!("{}{}", name, public_name),
                        rdf_info["created"],
                        rdf_info["modified"],
                        rdf_info["link"],
                    ]);
                });
                table.printstd();
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
                if OutputFmt::Pretty == output_fmt {
                    if resolved_from != xorurl {
                        println!("Resolved from: {}", resolved_from);
                    }
                    println!("= FilesContainer =");
                    println!("XOR-URL: {}", xorurl);
                    println!("Version: {}", version);
                    println!("Type tag: {}", type_tag);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: {}", data_type);
                // print_resolved_from(100, resolved_from);
                //} else if resolved_from.is_some() {
                //    println!("{}", serialise_output(&(url, content), output_fmt));
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
                media_type,
                resolved_from,
                ..
            } => {
                if OutputFmt::Pretty == output_fmt {
                    if resolved_from != xorurl {
                        println!("Resolved from: {}", resolved_from);
                    }
                    println!("= File =");
                    println!("XOR-URL: {}", xorurl);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: ImmutableData (published)");
                    println!(
                        "Media type: {}",
                        media_type.clone().unwrap_or_else(|| "Unknown".to_string())
                    );
                // print_resolved_from(100, resolved_from);
                //} else if resolved_from.is_some() {
                //    println!("{}", serialise_output(&(url, content), output_fmt));
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
                    if resolved_from != xorurl {
                        println!("Resolved from: {}", resolved_from);
                    }
                    println!("= Wallet =");
                    println!("XOR-URL: {}", xorurl);
                    println!("Type tag: {}", type_tag);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: {}", data_type);
                // print_resolved_from(100, resolved_from);
                //} else if resolved_from.is_some() {
                //    println!("{}", serialise_output(&(url, content), output_fmt));
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
                    if resolved_from != xorurl {
                        println!("Resolved from: {}", resolved_from);
                    }
                    println!("= SafeKey =");
                    println!("XOR-URL: {}", xorurl);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: SafeKey");
                // print_resolved_from(100, resolved_from);
                //} else if resolved_from.is_some() {
                //    println!("{}", serialise_output(&(url, content), output_fmt));
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
    }

    println!();
    Ok(())
}
