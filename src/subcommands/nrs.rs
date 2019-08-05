// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::get_from_arg_or_stdin;
use super::OutputFmt;
use prettytable::{format::FormatBuilder, Table};
use safe_cli::{Safe, XorUrl};
use std::collections::BTreeMap;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum NrsSubCommands {
    #[structopt(name = "add")]
    /// Add a subname to an existing NRS name, or updates its link if it already exists
    Add {
        /// The name to add (or update if it already exists)
        name: String,
        /// The safe:// URL to map this to. Usually a FilesContainer for a website
        #[structopt(short = "l", long = "link")]
        link: Option<String>,
        /// Set the sub name as default for this public name
        #[structopt(long = "default")]
        default: bool,
        /// If --default is set, the default is set using a direct link to the final destination that was provided with `--link`, rather than a link to the sub name being added (which is the default behaviour if this flag is not passed)
        #[structopt(short = "t", long = "direct")]
        direct_link: bool,
    },
    #[structopt(name = "create")]
    /// Create a new public name
    Create {
        /// The name to give site, eg 'safenetwork'
        name: String,
        /// The safe:// URL to map this to. Usually a FilesContainer for a website
        #[structopt(short = "l", long = "link")]
        link: Option<String>,
        /// The default is set but using a direct link to the final destination that was provided with `--link`, rather than a link to the sub name being created (which is the default behaviour if this flag is not passed)
        #[structopt(short = "t", long = "direct")]
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
    cmd: Option<NrsSubCommands>,
    output_fmt: OutputFmt,
    dry_run: bool,
    safe: &mut Safe,
) -> Result<(), String> {
    match cmd {
        Some(NrsSubCommands::Create {
            name,
            link,
            direct_link,
        }) => {
            // TODO: Where do we store/reference these? add it to the Root container,
            // sanitize name / spacing etc., validate destination?
            let link = get_from_arg_or_stdin(link, Some("...awaiting link URL from stdin"))?;

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
        Some(NrsSubCommands::Add {
            name,
            link,
            default,
            direct_link,
        }) => {
            let link = get_from_arg_or_stdin(link, Some("...awaiting link URL from stdin"))?;
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
        Some(NrsSubCommands::Remove { name }) => {
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
        None => Err("Missing keys sub-command. Use --help for details.".to_string()),
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
            serde_json::to_string(&(xorurl, processed_entries))
                .unwrap_or_else(|_| "Failed to serialise output to json".to_string())
        );
    }
}
