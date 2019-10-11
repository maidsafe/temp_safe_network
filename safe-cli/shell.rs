// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::operations::auth::*;
use safe_api::{Safe, SafeAuthdClient};
use shrust::{Shell, ShellIO};
use std::io::{stdin, stdout, Write};

pub fn shell_run() -> Result<(), String> {
    let safe = Safe::new("");
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
        "Authorise the SAFE CLI using a remote Authenticator daemon",
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

    shell.new_command_noargs(
        "auth-subscribe",
        "Send request to a remote Authenticator daemon to subscribe to receive authorisation requests notifications",
        |io, (_, safe_authd_client)| {
            let endpoint = "https://localhost:33001".to_string(); // args[0].to_string();
            match authd_subscribe(safe_authd_client, endpoint, &prompt_to_allow_auth) {
                Ok(()) => Ok(()),
                Err(err) => {
                    writeln!(io, "{}", err)?;
                    Ok(())
                }
            }
        },
    );
    shell.new_command_noargs(
        "auth-unsubscribe",
        "Send request to a remote Authenticator daemon to unsubscribe from authorisation requests notifications",
        |io, (_, safe_authd_client)| {
            let endpoint = "https://localhost:33001".to_string(); // args[0].to_string();
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
        "start-authd",
        "Starts the Authenticator daemon if it's not running already",
        |io, (_, _)| match authd_run_cmd("start") {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command_noargs(
        "stop-authd",
        "Stops the Authenticator daemon if it's running",
        |io, (_, _)| match authd_run_cmd("stop") {
            Ok(()) => Ok(()),
            Err(err) => {
                writeln!(io, "{}", err)?;
                Ok(())
            }
        },
    );
    shell.new_command_noargs(
        "restart-authd",
        "Restarts the Authenticator daemon if it's running already",
        |io, (_, _)| match authd_run_cmd("restart") {
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
    Ok(shell.run_loop(&mut ShellIO::default()))
}

fn prompt_to_allow_auth(app_id: &str) -> bool {
    /*    match req {
            IpcReq::Auth(app_auth_req) => {
                println!("The following application authorisation request was received:");
                let mut table = Table::new();
                table
                    .add_row(row![bFg->"Id", bFg->"Name", bFg->"Vendor", bFg->"Permissions requested"]);
                table.add_row(row![
                    app_auth_req.app.id,
                    app_auth_req.app.name,
                    // app_auth_req.app.scope || "",
                    app_auth_req.app.vendor,
                    format!(
                        "Own container: {}\nDefault containers: {:?}",
                        app_auth_req.app_container, app_auth_req.containers
                    ),
                ]);
                table.printstd();
            }
            IpcReq::Containers(cont_req) => {
                println!("The following authorisation request for containers was received:");
                println!("{:?}", cont_req);
                let mut table = Table::new();
                table
                    .add_row(row![bFg->"Id", bFg->"Name", bFg->"Vendor", bFg->"Permissions requested"]);
                table.add_row(row![
                    cont_req.app.id,
                    cont_req.app.name,
                    // cont_req.app.scope || "",
                    cont_req.app.vendor,
                    format!("{:?}", cont_req.containers)
                ]);
                table.printstd();
            }
            IpcReq::ShareMData(share_mdata_req) => {
                println!("The following authorisation request to share a MutableData was received:");
                let mut row = String::from("");
                for mdata in share_mdata_req.mdata.iter() {
                    row += &format!("Type tag: {}\nXoR name: {:?}", mdata.type_tag, mdata.name);
                    let insert_perm = if mdata.perms.is_allowed(MDataAction::Insert) {
                        " Insert"
                    } else {
                        ""
                    };
                    let update_perm = if mdata.perms.is_allowed(MDataAction::Update) {
                        " Update"
                    } else {
                        ""
                    };
                    let delete_perm = if mdata.perms.is_allowed(MDataAction::Delete) {
                        " Delete"
                    } else {
                        ""
                    };
                    let manage_perm = if mdata.perms.is_allowed(MDataAction::ManagePermissions) {
                        " ManagePermissions"
                    } else {
                        ""
                    };
                    row += &format!(
                        "\nPermissions:{}{}{}{}\n\n",
                        insert_perm, update_perm, delete_perm, manage_perm
                    );
                }
                let mut table = Table::new();
                table.add_row(row![
                    bFg->"Id",
                    bFg->"Name",
                    bFg->"Vendor",
                    bFg->"MutableData's requested to share"
                ]);
                table.add_row(row![
                    share_mdata_req.app.id,
                    share_mdata_req.app.name,
                    // share_mdata_req.app.scope || "",
                    share_mdata_req.app.vendor,
                    row
                ]);
                table.printstd();
            }
            IpcReq::Unregistered(_) => {
                // we simply allow unregistered authorisation requests
                return true;
            }
        };
    */
    println!("The following application authorisation request was received:");
    println!("App ID: {}", app_id);

    let mut prompt = String::new();
    print!("Allow authorisation? [y/N]: ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut prompt)
        .expect("Did not enter a correct string. Authorisation will be denied.");
    if let Some('\n') = prompt.chars().next_back() {
        prompt.pop();
    }
    if let Some('\r') = prompt.chars().next_back() {
        prompt.pop();
    }

    if prompt.to_lowercase() == "y" {
        println!("Authorisation will be allowed...");
        true
    } else {
        println!("Authorisation will be denied...");
        false
    }
}
