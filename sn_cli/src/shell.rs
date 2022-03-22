// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::cli;
use async_std::task;
use color_eyre::{eyre::eyre, Result};
use shrust::{Shell, ShellIO};
use sn_api::{Safe, SafeAuthdClient};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn shell_run() -> Result<()> {
    // We create a Safe instance which we''ll use for all commands till we exit.
    let safe = Safe::dry_runner(None);
    let (authd_cert_path, authd_notify_cert_path, authd_notify_key_path) = get_certificates()?;
    let sn_authd_client = SafeAuthdClient::new(
        None,
        &authd_cert_path,
        &authd_notify_cert_path,
        &authd_notify_key_path,
    );
    let mut shell = Shell::new((safe, sn_authd_client));

    shell.set_default(|io, _, cmd| {
        writeln!(
            io,
            "Command '{}' is unknown or not supported yet in interactive mode",
            cmd
        )?;
        writeln!(io, "Type 'help' for a list of currently supported top level commands")?;
        writeln!(io, "Pass '--help' flag to any top level command for a complete list of supported subcommands and arguments")?;
        Ok(())
    });
    shell.new_command(
        "cat",
        "Read data on the Safe Network",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("cat", args, safe, io),
    );
    shell.new_command(
        "config",
        "CLI config settings",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("config", args, safe, io),
    );
    shell.new_command(
        "dog",
        "Inspect data on the Safe Network providing only metadata information about the content",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("dog", args, safe, io),
    );
    shell.new_command(
        "files",
        "Manage files on the Safe Network",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("files", args, safe, io),
    );
    shell.new_command(
        "seq",
        "Manage Sequences on the Safe Network",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("seq", args, safe, io),
    );
    shell.new_command(
        "keypair",
        "Generate a key pair without creating and/or storing a SafeKey on the network",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("keypair", args, safe, io),
    );
    shell.new_command(
        "keys",
        "Manage keys on the Safe Network",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("keys", args, safe, io),
    );
    shell.new_command(
        "networks",
        "Switch between Safe networks",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("networks", args, safe, io),
    );
    shell.new_command(
        "nrs",
        "Manage public names on the Safe Network",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("nrs", args, safe, io),
    );
    shell.new_command(
        "setup",
        "Perform setup tasks",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("setup", args, safe, io),
    );
    shell.new_command(
        "update",
        "Update the application to the latest available version",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("update", args, safe, io),
    );
    shell.new_command(
        "node",
        "Commands to manage Safe Network Nodes",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("node", args, safe, io),
    );
    shell.new_command(
        "xorurl",
        "Obtain the XOR-URL of data without uploading it to the network, or decode XOR-URLs",
        0,
        |io, (safe, _sn_authd_client), args| call_cli("xorurl", args, safe, io),
    );

    println!();
    println!("Welcome to Safe CLI interactive shell!");
    println!("Type 'help' for a list of supported commands");
    println!("Pass '--help' flag to any top level command for a complete list of supported subcommands and arguments");
    println!("Type 'quit' to exit this shell. Enjoy it!");
    println!();

    // Run the shell loop to process user commands
    shell.run_loop(&mut ShellIO::default());

    Ok(())
}

fn call_cli(
    subcommand: &str,
    args: &[&str],
    safe: &mut Safe,
    io: &mut shrust::ShellIO,
) -> Result<(), shrust::ExecError> {
    // Let's create an args array to mimic the one we'd receive when passed to CLI
    let mut mimic_cli_args = vec!["safe", subcommand];
    mimic_cli_args.extend(args.iter());

    // We can now pass this args array to the CLI
    match task::block_on(cli::run_with(Some(&mimic_cli_args), safe)) {
        Ok(()) => Ok(()),
        Err(err) => {
            writeln!(io, "{}", err)?;
            Ok(())
        }
    }
}

/// Gets the paths of the certificate for authd and the certificate and private key for the
/// authd notification service. These are located at:
/// * ~/.safe/authd/authd_cert.der
/// * ~/.safe/authd/authd_notify_cert.der
/// * ~/.safe/authd/authd_notify_key.der
///
/// If the ~/.safe/authd directory doesn't exist, it will be created and self-signed certificates
/// will be generated.
fn get_certificates() -> Result<(PathBuf, PathBuf, PathBuf)> {
    let cert_base_path = dirs_next::home_dir()
        .ok_or_else(
            || eyre!("Failed to obtain home directory for authd certificates".to_string(),),
        )?
        .join(".safe")
        .join("authd");
    let authd_cert_path = cert_base_path.join("authd_cert.der");
    let authd_key_path = cert_base_path.join("authd_key.der");
    let authd_notify_cert_path = cert_base_path.join("authd_notify_cert.der");
    let authd_notify_key_path = cert_base_path.join("authd_notify_key.der");
    if !cert_base_path.exists() {
        std::fs::create_dir_all(cert_base_path)?;
        generate_certificate(&authd_cert_path, &authd_key_path)?;
        generate_certificate(&authd_notify_cert_path, &authd_notify_key_path)?;
    }

    Ok((
        authd_cert_path,
        authd_notify_cert_path,
        authd_notify_key_path,
    ))
}

fn generate_certificate(cert_path: &Path, key_path: &Path) -> Result<()> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).map_err(|err| {
        eyre!(format!(
            "Failed to generate self-signed certificate: {}",
            err
        ))
    })?;
    let key = cert.serialize_private_key_der();
    let cert = cert
        .serialize_der()
        .map_err(|err| eyre!(format!("Failed to serialise certificate: {}", err)))?;
    std::fs::write(&cert_path, &cert)
        .map_err(|err| eyre!(format!("Failed to write certificate: {}", err)))?;
    std::fs::write(&key_path, &key)
        .map_err(|err| eyre!(format!("Failed to write private key: {}", err)))?;
    Ok(())
}

// #[allow(dead_code)]
// fn prompt_to_allow_auth(auth_req: AuthReq) -> Option<bool> {
//     println!();
//     println!("A new application authorisation request was received:");
//     let req_id = auth_req.req_id;
//     pretty_print_auth_reqs(vec![auth_req], None);

//     println!("You can use 'auth allow'/'auth deny' commands to allow/deny the request respectively, e.g.: auth allow {}", req_id);
//     println!("Press Enter to continue");
//     let _ = stdout().flush();
//     None
// }
