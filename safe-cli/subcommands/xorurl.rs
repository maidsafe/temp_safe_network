// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    helpers::{gen_processed_files_table, get_from_arg_or_stdin, serialise_output, xorname_to_hex},
    OutputFmt,
};
use crate::operations::safe_net::connect;
use async_std::task;
use safe_api::{xorurl::XorUrlEncoder, Safe};
use structopt::StructOpt;

// Defines subcommands of 'xorurl'
#[derive(StructOpt, Debug)]
pub enum XorurlSubCommands {
    #[structopt(name = "decode")]
    /// Decode a XOR-URL extracting all the information encoded it in
    Decode {
        /// The XOR-URL to decode
        xorurl: Option<String>,
    },
}

pub fn xorurl_commander(
    cmd: Option<XorurlSubCommands>,
    location: Option<String>,
    recursive: bool,
    output_fmt: OutputFmt,
    mut safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        Some(XorurlSubCommands::Decode { xorurl }) => {
            let url = get_from_arg_or_stdin(xorurl, Some("...awaiting XOR-URL from stdin"))?;
            let xorurl_encoder = XorUrlEncoder::from_url(&url)?;
            if OutputFmt::Pretty == output_fmt {
                println!("Information decoded from XOR-URL: {}", url);
                println!("Xorname: {}", xorname_to_hex(&xorurl_encoder.xorname()));
                println!("Type tag: {}", xorurl_encoder.type_tag());
                println!("Native data type: {}", xorurl_encoder.data_type());
                let path = if xorurl_encoder.path().is_empty() {
                    "none"
                } else {
                    xorurl_encoder.path()
                };
                println!("Path: {}", path);
                println!("Sub names: {:?}", xorurl_encoder.sub_names());
                println!(
                    "Content version: {}",
                    xorurl_encoder
                        .content_version()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "latest".to_string())
                );
            } else {
                println!("{}", serialise_output(&xorurl_encoder, output_fmt));
            }
        }
        None => {
            connect(&mut safe)?;
            let location =
                get_from_arg_or_stdin(location, Some("...awaiting location path from stdin"))?;

            // Do a dry-run on the location
            let (_version, processed_files, _files_map) =
                task::block_on(safe.files_container_create(&location, None, recursive, true))?;

            // Now let's just print out a list of the xorurls
            if OutputFmt::Pretty == output_fmt {
                if processed_files.is_empty() {
                    println!("No files were processed");
                } else {
                    let (table, success_count) = gen_processed_files_table(&processed_files, false);
                    println!("{} file/s processed:", success_count);
                    table.printstd();
                }
            } else {
                let mut list = Vec::<(String, String)>::new();
                for (file_name, (_change, link)) in processed_files {
                    list.push((file_name, link));
                }
                println!("{}", serialise_output(&list, output_fmt));
            }
        }
    }
    Ok(())
}
