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
use color_eyre::{eyre::eyre, Result};
use prettytable::{format::FormatBuilder, Table};
use sn_api::{Safe, Url};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum NrsSubCommands {
    #[structopt(name = "add")]
    /// Add a subname to an existing NRS name, or update its link if it already exists
    Add {
        /// The name to add (or update if it already exists)
        name: String,
        /// The safe:// URL to map this to. Usually a FilesContainer for a website. This should be wrapped in double quotes on bash based systems.
        #[structopt(short = "l", long = "link")]
        link: Option<String>,
        /// Registers the topname for you if you didn't already register it with nrs create
        #[structopt(short = "y", long = "create-top-name")]
        create_top_name: bool,
        /// Set the link as default for the top level NRS name as well
        #[structopt(long = "default")]
        default: bool,
        /// If --default is set, the default name is set using a direct link to the final destination that was provided with `--link`, rather than a link to the sub name being added (which is the default behaviour if this flag is not passed)
        #[structopt(long = "direct")]
        direct_link: bool,
    },
    #[structopt(name = "create")]
    /// Create/Register a new top name
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

pub async fn nrs_commander(
    cmd: NrsSubCommands,
    output_fmt: OutputFmt,
    dry_run: bool,
    safe: &mut Safe,
) -> Result<()> {
    match cmd {
        NrsSubCommands::Create {
            name,
            link,
            ..
            // direct_link,
        } => {
            // TODO: Where do we store/reference these? add it to the Root container,
            // sanitize name / spacing etc., validate destination?

            // register nrs topname
            let url = safe.nrs_create(&name, dry_run).await?;

            // associate if a link is provided
            let (new_url, link_str) = match link {
                Some(l) => {
                    let link_url = Url::from_url(&l)?;
                    let new_url = safe.nrs_associate(&name, &link_url, dry_run).await?;
                    (new_url, l)
                },
                None => {
                    (url, "".to_string())
                },
            };

            if dry_run && OutputFmt::Pretty == output_fmt {
                notice_dry_run();
            }

            print_summary(
                output_fmt,
                &format!(
                    "New NRS Map for \"safe://{}\" created:",
                    name.replace("safe://", "")
                ),
                new_url,
                ("+", &name, &link_str),
            );
            Ok(())
        }
        NrsSubCommands::Add {
            name,
            link,
            create_top_name,
            ..
            // default,
            // direct_link,
        } => {
            let link = get_from_arg_or_stdin(link, Some("...awaiting link URL from stdin"))?;
            if dry_run && OutputFmt::Pretty == output_fmt {
                notice_dry_run();
            }

            let url = Url::from_url(&link)?;
            let (url, did_register_topname) = match create_top_name {
                true => safe.nrs_add(&name, &url, dry_run).await?,
                false => (safe.nrs_associate(&name, &url, dry_run).await?, false),
            };
            let version = url
                .content_version()
                .ok_or_else(|| eyre!("Content version not set for returned NRS Url"))?
                .to_string();
            let msg = if did_register_topname {
                format!("New NRS Map created (version {})", version)
            } else {
                format!("Existing NRS Map updated (version {})", version)
            };
            print_summary(
                output_fmt,
                &msg,
                url,
                ("+", &name, &link),
            );
            Ok(())
        }
        NrsSubCommands::Remove { name } => {
            if dry_run && OutputFmt::Pretty == output_fmt {
                notice_dry_run();
            }

            let url = safe.nrs_remove(&name, dry_run).await?;
            let version = url
                .content_version()
                .ok_or_else(|| eyre!("Content version not set for returned NRS Url"))?
                .to_string();
            print_summary(
                output_fmt,
                &format!("NRS Map updated (version {})", version),
                url,
                ("-", &name, ""),
            );
            Ok(())
        }
    }
}

fn print_summary(
    output_fmt: OutputFmt,
    header_msg: &str,
    xorurl: Url,
    processed_entry: (&str, &str, &str),
) {
    if OutputFmt::Pretty == output_fmt {
        let mut table = Table::new();
        let format = FormatBuilder::new()
            .column_separator(' ')
            .padding(0, 1)
            .build();
        table.set_format(format);

        let (change, top_name, url) = processed_entry;
        table.add_row(row![change, top_name, url]);
        println!("{}: \"{}\"", header_msg, xorurl);
        table.printstd();
    } else {
        println!(
            "{}",
            serialise_output(&(xorurl, processed_entry), output_fmt)
        );
    }
}
