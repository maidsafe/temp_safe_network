// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::helpers::{gen_processed_files_table, get_from_arg_or_stdin, serialise_output};
use super::OutputFmt;
use safe_api::Safe;
use structopt::StructOpt;

// Defines subcommands of 'xorurl'
#[derive(StructOpt, Debug)]
pub struct XorurlSubCommands {
    /// The source file/folder local path
    location: Option<String>,
    /// Recursively crawl folders and files found in the location
    #[structopt(short = "r", long = "recursive")]
    recursive: bool,
}

pub fn xorurl_commander(
    cmd: XorurlSubCommands,
    output_fmt: OutputFmt,
    safe: &mut Safe,
) -> Result<(), String> {
    let location =
        get_from_arg_or_stdin(cmd.location, Some("...awaiting location path from stdin"))?;

    // Do a dry-run on the location
    let (_version, processed_files, _files_map) =
        safe.files_container_create(&location, None, cmd.recursive, true)?;

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
    Ok(())
}
