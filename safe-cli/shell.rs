// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::operations::{auth_daemon::*, safe_net::*};
use crate::APP_ID;
use safe_api::{AuthReq, Safe, SafeAuthdClient};
use shrust::{Shell, ShellIO};
use std::io::{stdout, Write};

const AUTH_REQS_NOTIFS_ENDPOINT: &str = "https://localhost:33001";

pub fn shell_run() -> Result<(), String> {
    let safe = Safe::new(None);
    let safe_authd_client = SafeAuthdClient::new(None);
    let mut shell = Shell::new((safe, safe_authd_client));
    shell.set_default(|io, _, cmd| {
        writeln!(
            io,
            "Command '{}' is unknown or not supported yet in interactive mode",
            cmd
        )?;
        writeln!(io, "Type 'help' for a list of currently supported commands")?;
        Ok(())
    });
    shell.new_command_noargs(
        "auth",
        "Authorise the CLI using a remote Authenticator daemon, or interact with it using subcommands",
        |io, (safe, _)| match authorise_cli(safe, None) {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command_noargs(
        "auth-create",
        "Send request to a remote Authenticator daemon to create a new SAFE account",
        |io, (safe, safe_authd_client)| match authd_create(safe, safe_authd_client, None, true) {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command_noargs(
        "auth-status",
        "Send request to a remote Authenticator daemon to obtain an status report",
        |io, (_, safe_authd_client)| match authd_status(safe_authd_client) {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command_noargs(
        "auth-login",
        "Send request to a remote Authenticator daemon to login to a SAFE account",
        |io, (_, safe_authd_client)| match authd_login(safe_authd_client) {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command_noargs("auth-logout", "Send request to a remote Authenticator daemon to logout from currently logged in SAFE account", |io, (_, safe_authd_client)| {
        match authd_logout(safe_authd_client) {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        }
    });
    shell.new_command_noargs(
        "auth-clear",
        "Clear SAFE CLI authorisation credentials from local file",
        |io, (_, _)| match clear_credentials() {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command_noargs(
        "auth-apps",
        "Send request to a remote Authenticator daemon to retrieve the list of the authorised applications",
        |io, (_, safe_authd_client)| match authd_apps(safe_authd_client) {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command(
        "auth-revoke",
        "Send request to a remote Authenticator daemon to revoke permissions from a previously authorised application",
        1, |io, (_, safe_authd_client), args| {
            let app_id = args[0];
            match authd_revoke(safe_authd_client, app_id.to_string()) {
                Ok(()) => Ok(()),
                Err(err) => {
                    writeln!(io, "{}", err)?;
                    Ok(())
                }
            }
        }
    );
    shell.new_command_noargs(
        "auth-reqs",
        "Send request to a remote Authenticator daemon to retrieve the list of the pending authorisation requests",
        |io, (_, safe_authd_client)| match authd_auth_reqs(safe_authd_client) {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command(
        "auth-allow",
        "Send request to a remote Authenticator daemon to allow an authorisation request",
        1,
        |io, (_, safe_authd_client), args| {
            let req_id = args[0].to_string().parse::<u32>()?;
            match authd_allow(safe_authd_client, req_id) {
                Ok(()) => Ok(()),
                Err(err) => {
                    writeln!(io, "{}", err)?;
                    Ok(())
                }
            }
        },
    );
    shell.new_command(
        "auth-deny",
        "Send request to a remote Authenticator daemon to deny an authorisation request",
        1,
        |io, (_, safe_authd_client), args| {
            let req_id = args[0].to_string().parse::<u32>()?;
            match authd_deny(safe_authd_client, req_id) {
                Ok(()) => Ok(()),
                Err(err) => {
                    writeln!(io, "{}", err)?;
                    Ok(())
                }
            }
        },
    );

    shell.new_command(
        "auth-subscribe",
        "Send request to a remote Authenticator daemon to subscribe to receive authorisation requests notifications",
        0,
        |io, (_, safe_authd_client), args| {
            let endpoint = if args.is_empty() {
                AUTH_REQS_NOTIFS_ENDPOINT.to_string()
            } else {
                args[0].to_string()
            };

            match authd_subscribe(safe_authd_client, endpoint, APP_ID, &prompt_to_allow_auth) {
                Ok(()) => {
                    writeln!(io, "Keep this shell session open to receive the notifications")?;
                    Ok(())
                },
                Err(err) => {
                    writeln!(io, "{}", err)?;
                    Ok(())
                }
            }
        },
    );
    shell.new_command(
        "auth-unsubscribe",
        "Send request to a remote Authenticator daemon to unsubscribe from authorisation requests notifications",
        0,
        |io, (_, safe_authd_client), args| {
            let endpoint = if args.is_empty() {
                AUTH_REQS_NOTIFS_ENDPOINT.to_string()
            } else {
                args[0].to_string()
            };

            match authd_unsubscribe(safe_authd_client, endpoint) {
                Ok(()) => Ok(()),
                Err(err) => {
                    writeln!(io, "{}", err)?;
                    Ok(())
                }
            }
        },
    );
    shell.new_command_noargs(
        "auth-start",
        "Starts the Authenticator daemon if it's not running already",
        |io, (_, safe_authd_client)| match authd_start(safe_authd_client) {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command_noargs(
        "auth-stop",
        "Stops the Authenticator daemon if it's running",
        |io, (_, safe_authd_client)| match authd_stop(safe_authd_client) {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command_noargs(
        "auth-restart",
        "Restarts the Authenticator daemon if it's running already",
        |io, (_, safe_authd_client)| match authd_restart(safe_authd_client) {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );

    println!();
    println!("Welcome to SAFE CLI interactive shell!");
    println!("Type 'help' for a list of supported commands");
    println!("Type 'quit' to exit this shell. Enjoy it!");
    println!();

    // Run the shell loop to process user commands
    shell.run_loop(&mut ShellIO::default());

    Ok(())
}

fn prompt_to_allow_auth(auth_req: AuthReq) -> Option<bool> {
    println!();
    println!("A new application authorisation request was received:");
    pretty_print_auth_reqs(vec![auth_req], None);

    println!("To allow/deny the request, use the auth-allow/auth-deny commands respectively, e.g.: auth-allow <request id>");
    println!("Press Enter to continue");
    let _ = stdout().flush();
    None
}
