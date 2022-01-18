// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    helpers::{get_from_arg_or_stdin, serialise_output},
    OutputFmt,
};
use color_eyre::{eyre::eyre, eyre::WrapErr, Help, Result};
use prettytable::{format::FormatBuilder, Table};
use sn_api::Error::{InvalidInput, NetDataError, NrsNameAlreadyExists, UnversionedContentError};
use sn_api::{Safe, SafeUrl};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum NrsSubCommands {
    #[structopt(name = "add")]
    /// Add a subname to a registered NRS name and link it to some content, or update an existing
    /// subname with a new link.
    Add {
        /// Specify the public name, which is the subname you wish to use, and the registered
        /// topname. For example, "new.topname". If the topname has not already been registered
        /// with the `nrs register` command, use the `--register-top-name` flag to register it here.
        public_name: String,
        /// The safe:// URL to link to. Usually a FilesContainer for a website. This should be
        /// wrapped in double quotes on bash based systems. A link must be provided for a subname.
        /// If you don't provide it with this argument, you will be prompted to provide it
        /// interactively.
        #[structopt(short = "l", long = "link")]
        link: Option<String>,
        /// Set this flag to register the topname if it hasn't already been registered.
        #[structopt(short = "y", long = "register-top-name")]
        register_top_name: bool,
        /// Set this flag to register this link as default for the topname when no subname is
        /// specified.
        #[structopt(long = "default")]
        default: bool,
    },
    #[structopt(name = "register")]
    /// Register a new top name in Safe NRS
    Register {
        /// The name of the new topname to register
        name: String,
        /// Optional safe:// URL to link the topname to. Usually a FilesContainer for a website.
        /// This should be wrapped in double quotes on bash based systems.
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
    safe: &mut Safe,
) -> Result<()> {
    match cmd {
        NrsSubCommands::Register { name, link } => {
            run_register_subcommand(name, link, safe, output_fmt).await
        }
        NrsSubCommands::Add {
            public_name: name,
            link,
            register_top_name,
            default,
        } => run_add_subcommand(name, link, register_top_name, default, safe, output_fmt).await,
        NrsSubCommands::Remove { name } => run_remove_subcommand(name, safe, output_fmt).await,
    }
}

async fn run_register_subcommand(
    name: String,
    link: Option<String>,
    safe: &mut Safe,
    output_fmt: OutputFmt,
) -> Result<()> {
    if let Some(ref link) = link {
        validate_target_link(link)?;
    }

    match safe.nrs_create(&name).await {
        Ok(topname_url) => {
            let (nrs_url, summary) =
                get_new_nrs_url_for_topname(&name, safe, topname_url, link).await?;
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
                    "The register command can only register a topname, it cannot add subnames.",
                )
                .suggestion(
                    "Please use the nrs add command with the --register-top-name \
                        argument to register a topname and add a subname at the same time.",
                )
                .suggestion(
                    "Alternatively, register the topname first with the register command, \
                        then use the add command to add the subname.",
                )),
            NrsNameAlreadyExists(_) => Err(eyre!(error)
                .wrap_err(format!(
                    "Could not register topname {}. That name is already taken.",
                    name
                ))
                .suggestion("Try the command again with a different name.")),
            _ => Err(eyre!(error)),
        },
    }
}

async fn run_add_subcommand(
    name: String,
    link: Option<String>,
    register_top_name: bool,
    default: bool,
    safe: &mut Safe,
    output_fmt: OutputFmt,
) -> Result<()> {
    let link = get_from_arg_or_stdin(link, Some("...awaiting link URL from stdin"))?;
    validate_target_link(&link)?;
    let link_url = SafeUrl::from_url(&link)?;
    let (url, topname_was_registered) = if register_top_name {
        add_public_name_for_url(&name, safe, &link_url).await?
    } else {
        (
            associate_url_with_public_name(&name, safe, &link_url).await?,
            false,
        )
    };

    let mut summary_header = String::new();
    if topname_was_registered {
        summary_header.push_str("New NRS Map created. ");
    } else {
        summary_header.push_str("Existing NRS Map updated. ");
    }
    let version = url
        .content_version()
        .ok_or_else(|| eyre!("Content version not set for returned NRS SafeUrl"))?
        .to_string();
    summary_header.push_str(&format!("Now at version {}. ", version));

    if default {
        let topname = get_topname_from_public_name(&name)?;
        associate_url_with_public_name(&topname, safe, &link_url).await?;
        summary_header.push_str(&format!(
            "This link was also set as the default location for {}.",
            topname
        ));
    }
    print_summary(
        output_fmt,
        &summary_header,
        "".to_string(),
        &url,
        ("+", &name, &link),
    );
    Ok(())
}

async fn run_remove_subcommand(name: String, safe: &mut Safe, output_fmt: OutputFmt) -> Result<()> {
    match safe.nrs_remove(&name).await {
        Ok(url) => {
            let version = url
                .content_version()
                .ok_or_else(|| eyre!("Content version not set for returned NRS SafeUrl"))?
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
        Err(error) => match error {
            // This is the type of error returned when you supply a topname that doesn't exist.
            // Although obviously, this error could occur due to a general connectivity issue,
            // which is why the error message advises that the topname is "likely" not registered.
            NetDataError(_) => {
                let topname = get_topname_from_public_name(&name)?;
                Err(eyre!(error)
                    .wrap_err(format!(
                        "Failed to remove {}. The topname {} is likely not registered in Safe NRS.",
                        name, topname
                    ))
                    .suggestion(format!(
                        "Try the command again or verify that {} is a registered topname.",
                        topname
                    )))
            }
            _ => Err(eyre!(error)),
        },
    }
}

/// Determine if the link is a valid url *before* creating the topname.
///
/// Otherwise the user receives an error even though the topname was actually registerd, which is a
/// potentially confusing experience: they may think the topname wasn't registerd.
fn validate_target_link(link: &str) -> Result<()> {
    SafeUrl::from_url(link)
        .wrap_err("The supplied link was not a valid url.")
        .suggestion("Run the command again with a valid url for the --link argument.")?;
    Ok(())
}

/// Get the new NRS URL that's going to be displayed to the user.
///
/// If no target link has been supplied, the URL is just going to be the one returned from the
/// topname creation; otherwise, associate the link with the newly registered topname, and return the
/// URL generated from the association.
async fn get_new_nrs_url_for_topname(
    name: &str,
    safe: &mut Safe,
    topname_url: SafeUrl,
    link: Option<String>,
) -> Result<(SafeUrl, String)> {
    if let Some(link) = link {
        let url = SafeUrl::from_url(&link)?;
        let new_url = associate_url_with_public_name(name, safe, &url).await?;
        return Ok((new_url, format!("The entry points to {}", link)));
    }
    Ok((topname_url, "".to_string()))
}

async fn associate_url_with_public_name(
    public_name: &str,
    safe: &mut Safe,
    url: &SafeUrl,
) -> Result<SafeUrl> {
    match safe.nrs_associate(public_name, url).await {
        Ok(new_url) => Ok(new_url),
        Err(error) => match error {
            UnversionedContentError(_) => Err(eyre!(error)
                .wrap_err(
                    "The destination you're trying to link to is versionable content. \
                        When linking to versionable content, you must supply a version hash on the \
                        url. The requested topname was not registered.",
                )
                .suggestion(
                    "Please run the command again with the version hash appended to the link. \
                            The link should have the form safe://<url>?v=<versionhash>.",
                )),
            _ => Err(eyre!(error)),
        },
    }
}

async fn add_public_name_for_url(
    public_name: &str,
    safe: &mut Safe,
    url: &SafeUrl,
) -> Result<(SafeUrl, bool)> {
    match safe.nrs_add(public_name, url).await {
        Ok((new_url, topname_was_registered)) => Ok((new_url, topname_was_registered)),
        Err(error) => match error {
            UnversionedContentError(_) => Err(eyre!(error)
                .wrap_err(
                    "The destination you're trying to link to is versionable content. \
                        When linking to versionable content, you must supply a version hash on the \
                        url. The requested topname was not registered.",
                )
                .suggestion(
                    "Please run the command again with the version hash appended to the link. \
                            The link should have the form safe://<url>?v=<versionhash>.",
                )),
            _ => Err(eyre!(error)),
        },
    }
}

fn print_summary(
    output_fmt: OutputFmt,
    header: &str,
    summary: String,
    nrs_url: &SafeUrl,
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
            println!("{}", summary.trim());
        }
        table.printstd();
    } else {
        println!(
            "{}",
            serialise_output(&(nrs_url, processed_entry), output_fmt)
        );
    }
}

fn get_topname_from_public_name(public_name: &str) -> Result<String> {
    let mut parts = public_name.split('.');
    let topname = parts
        .next_back()
        .ok_or_else(|| {
            eyre!(format!(
                "Could not parse topname from public name {}",
                public_name
            ))
        })?
        .to_string();
    Ok(topname)
}
