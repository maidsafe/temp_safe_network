// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    helpers::{get_from_arg_or_stdin, notice_dry_run, serialise_output},
    OutputFmt,
};
use prettytable::{format::FormatBuilder, Table};
use safe_api::{xorurl::XorUrl, Safe};
use std::collections::BTreeMap;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum NrsSubCommands {
    #[structopt(name = "add")]
    /// Add a subname to an existing NRS name, or updates its link if it already exists
    Add {
        /// The name to add (or update if it already exists)
        name: String,
        /// The safe:// URL to map this to. Usually a FilesContainer for a website. This should be wrapped in double quotes on bash based systems.
        #[structopt(short = "l", long = "link")]
        link: Option<String>,
        /// Set the sub name as default for this public name
        #[structopt(long = "default")]
        default: bool,
        /// If --default is set, the default name is set using a direct link to the final destination that was provided with `--link`, rather than a link to the sub name being added (which is the default behaviour if this flag is not passed)
        #[structopt(long = "direct")]
        direct_link: bool,
    },
    #[structopt(name = "create")]
    /// Create a new public name
    Create {
        /// The name to give site, eg 'safenetwork'
        name: String,
        /// The safe:// URL to map this to. Usually a FilesContainer for a website. This should be wrapped in double quotes on bash based systems.
        #[structopt(short = "l", long = "link")]
        link: Option<String>,
        /// The default name is set using a direct link to the final destination that was provided with `--link`, rather than a link to the sub name being created (which is the default behaviour if this flag is not passed)
        #[structopt(long = "direct")]
        direct_link: bool,
    },
    #[structopt(name = "remove")]
    /// Remove a subname from an NRS name
    Remove {
        /// The name to remove
        name: String,
    },
}

pub fn nrs_commander(
    cmd: NrsSubCommands,
    output_fmt: OutputFmt,
    dry_run: bool,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        NrsSubCommands::Create {
            name,
            link,
            direct_link,
        } => {
            // TODO: Where do we store/reference these? add it to the Root container,
            // sanitize name / spacing etc., validate destination?
            let link = get_from_arg_or_stdin(link, Some("...awaiting link URL from stdin"))?;

            if dry_run && OutputFmt::Pretty == output_fmt {
                notice_dry_run();
            }

            // Set it as default too, so the top level NRS name is resolvable to same link
            let default = true;

            let (nrs_map_container_xorurl, processed_entries, _nrs_map) =
                safe.nrs_map_container_create(&name, &link, default, direct_link, dry_run)?;

            // Now let's just print out a summary
            print_summary(
                output_fmt,
                &format!(
                    "New NRS Map for \"safe://{}\" created at",
                    name.replace("safe://", "")
                ),
                nrs_map_container_xorurl,
                processed_entries,
            );

            Ok(())
        }
        NrsSubCommands::Add {
            name,
            link,
            default,
            direct_link,
        } => {
            let link = get_from_arg_or_stdin(link, Some("...awaiting link URL from stdin"))?;

            if dry_run && OutputFmt::Pretty == output_fmt {
                notice_dry_run();
            }

            let (version, xorurl, processed_entries, _nrs_map) =
                safe.nrs_map_container_add(&name, &link, default, direct_link, dry_run)?;

            // Now let's just print out the summary
            print_summary(
                output_fmt,
                &format!("NRS Map updated (version {})", version),
                xorurl,
                processed_entries,
            );

            Ok(())
        }
        NrsSubCommands::Remove { name } => {
            if dry_run && OutputFmt::Pretty == output_fmt {
                notice_dry_run();
            }

            let (version, xorurl, processed_entries, _nrs_map) =
                safe.nrs_map_container_remove(&name, dry_run)?;

            // Now let's just print out the summary
            print_summary(
                output_fmt,
                &format!("NRS Map updated (version {})", version),
                xorurl,
                processed_entries,
            );

            Ok(())
        }
    }
}

fn print_summary(
    output_fmt: OutputFmt,
    header_msg: &str,
    xorurl: XorUrl,
    processed_entries: BTreeMap<String, (String, String)>,
) {
    if OutputFmt::Pretty == output_fmt {
        let mut table = Table::new();
        let format = FormatBuilder::new()
            .column_separator(' ')
            .padding(0, 1)
            .build();
        table.set_format(format);

        for (public_name, (change, name_link)) in processed_entries.iter() {
            table.add_row(row![change, public_name, name_link]);
        }
        println!("{}: \"{}\"", header_msg, xorurl);
        table.printstd();
    } else {
        println!(
            "{}",
            serialise_output(&(xorurl, processed_entries), output_fmt)
        );
    }
}
