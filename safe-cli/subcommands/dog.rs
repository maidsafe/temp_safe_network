// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    helpers::{get_from_arg_or_stdin, print_nrs_map, serialise_output, xorname_to_hex},
    OutputFmt,
};
use log::debug;
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

    let resolved_content = safe.inspect(&url).await?;
    if OutputFmt::Pretty != output_fmt {
        println!("{}", serialise_output(&(url, resolved_content), output_fmt));
    } else {
        for (i, ref content) in resolved_content.iter().enumerate() {
            println!();
            println!("== URL resolution step {} ==", i + 1);
            match content {
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
                    println!("Resolved from: {}", resolved_from);
                    println!("= NRS Map Container =");
                    println!("PublicName: \"{}\"", public_name);
                    println!("XOR-URL: {}", xorurl);
                    println!("Version: {}", version);
                    println!("Type tag: {}", type_tag);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: {}", data_type);
                    print_nrs_map(&nrs_map, &public_name);
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
                    println!("Resolved from: {}", resolved_from);
                    println!("= FilesContainer =");
                    println!("XOR-URL: {}", xorurl);
                    println!("Version: {}", version);
                    println!("Type tag: {}", type_tag);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: {}", data_type);
                }
                SafeData::PublicImmutableData {
                    xorurl,
                    xorname,
                    media_type,
                    resolved_from,
                    ..
                } => {
                    println!("Resolved from: {}", resolved_from);
                    println!("= File =");
                    println!("XOR-URL: {}", xorurl);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: PublicImmutableData");
                    println!(
                        "Media type: {}",
                        media_type.clone().unwrap_or_else(|| "Unknown".to_string())
                    );
                }
                SafeData::Wallet {
                    xorurl,
                    xorname,
                    type_tag,
                    data_type,
                    resolved_from,
                    ..
                } => {
                    println!("Resolved from: {}", resolved_from);
                    println!("= Wallet =");
                    println!("XOR-URL: {}", xorurl);
                    println!("Type tag: {}", type_tag);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: {}", data_type);
                }
                SafeData::SafeKey {
                    xorurl,
                    xorname,
                    resolved_from,
                } => {
                    println!("Resolved from: {}", resolved_from);
                    println!("= SafeKey =");
                    println!("XOR-URL: {}", xorurl);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: SafeKey");
                }
                SafeData::PublicSequence {
                    xorurl,
                    xorname,
                    type_tag,
                    version,
                    resolved_from,
                    ..
                } => {
                    if resolved_from != xorurl {
                        println!("Resolved from: {}", resolved_from);
                    }
                    println!("= Sequence =");
                    println!("XOR-URL: {}", xorurl);
                    println!("Version: {}", version);
                    println!("Type tag: {}", type_tag);
                    println!("XOR name: 0x{}", xorname_to_hex(xorname));
                    println!("Native data type: PublicSequence");
                }
            }
        }
        println!();
    }

    Ok(())
}
