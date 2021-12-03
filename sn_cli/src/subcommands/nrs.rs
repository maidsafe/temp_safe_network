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
use color_eyre::{eyre::eyre, eyre::WrapErr, Help, Result};
use prettytable::{format::FormatBuilder, Table};
use sn_api::Error::{InvalidInput, NrsNameAlreadyExists, UnversionedContentError};
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
        } => {
            run_create_subcommand(name, link, safe, dry_run, output_fmt).await
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
                "".to_string(),
                &url,
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
                "".to_string(),
                &url,
                ("-", &name, ""),
            );
            Ok(())
        }
    }
}

async fn run_create_subcommand(
    name: String,
    link: Option<String>,
    safe: &mut Safe,
    dry_run: bool,
    output_fmt: OutputFmt,
) -> Result<()> {
    if let Some(ref link) = link {
        validate_target_link(link, &name)?;
    }
    match safe.nrs_create(&name, dry_run).await {
        Ok(topname_url) => {
            let (nrs_url, summary) =
                get_new_nrs_url_for_topname(&name, safe, topname_url, link, dry_run).await?;
            print_summary(
                output_fmt,
                &format!(
                    "New NRS Map created for \"safe://{}\"",
                    name.replace("safe://", "")
                ),
                summary,
                &nrs_url,
                ("+", &name, &nrs_url.to_string()),
            );
            Ok(())
        }
        Err(error) => match error {
            InvalidInput(_) => Err(eyre!(error)
                .wrap_err(
                    "The create command can only create a topname, it cannot create subnames.",
                )
                .suggestion(
                    "Please use the nrs add command with the --create-top-name \
                        argument to create a topname and add a subname at the same time.",
                )
                .suggestion(
                    "Alternatively, create the topname first with the create command, \
                        then use the add command to create the subname.",
                )),
            NrsNameAlreadyExists(_) => Err(eyre!(error)
                .wrap_err(format!(
                    "Could not create topname {}. That name is already taken.",
                    name
                ))
                .suggestion("Try the command again with a different name.")),
            _ => Err(eyre!(error)),
        },
    }
}

/// Determine if the link is a valid XorUrl *before* creating the topname.
///
/// Otherwise the user receives an error even though the topname was actually created, which is a
/// potentially confusing experience: they may think the topname wasn't created.
fn validate_target_link(link: &str, name: &str) -> Result<()> {
    Url::from_url(link)
        .wrap_err(format!(
            "Could not create topname {}. The supplied link was not a valid XorUrl.",
            name
        ))
        .suggestion("Run the command again with a valid XorUrl for the --link argument.")?;
    Ok(())
}

/// Get the new NRS URL that's going to be displayed to the user.
///
/// If no target link has been supplied, the URL is just going to be the one returned from the
/// topname creation; otherwise, associate the link with the newly created topname, and return the
/// URL generated from the association.
async fn get_new_nrs_url_for_topname(
    name: &str,
    safe: &mut Safe,
    topname_url: Url,
    link: Option<String>,
    dry_run: bool,
) -> Result<(Url, String)> {
    if let Some(link) = link {
        let url = Url::from_url(&link)?;
        match safe.nrs_associate(name, &url, dry_run).await {
            Ok(new_url) => return Ok((new_url, format!("The entry points to {}", link))),
            Err(error) => match error {
                UnversionedContentError(_) => {
                    return Err(eyre!(error).wrap_err(
                        "The destination you're trying to link to is versionable content. \
                            When linking to versionable content, you must supply a version hash on the \
                            XorUrl. The requested topname was not created.",
                    ).suggestion(
                        "Please run the command again with the version hash appended to the link. \
                            The link should have the form safe://<xorurl>?v=<versionhash>.",
                    ));
                }
                _ => {
                    return Err(eyre!(error));
                }
            },
        }
    }
    Ok((topname_url, "".to_string()))
}

fn print_summary(
    output_fmt: OutputFmt,
    header: &str,
    summary: String,
    nrs_url: &Url,
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
        println!("{}", header);
        if !summary.is_empty() {
            println!("{}", summary);
        }
        table.printstd();
    } else {
        println!(
            "{}",
            serialise_output(&(nrs_url, processed_entry), output_fmt)
        );
    }
}
