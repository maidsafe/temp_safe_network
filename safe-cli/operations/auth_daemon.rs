// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use prettytable::Table;
use rpassword;
use safe_api::{AuthAllowPrompt, Safe, SafeAuthdClient};

pub fn authd_start(safe_authd: &SafeAuthdClient) -> Result<(), String> {
    let authd_path = get_authd_bin_path();
    safe_authd
        .start(Some(&authd_path))
        .map_err(|err| err.to_string())
}

pub fn authd_stop(safe_authd: &SafeAuthdClient) -> Result<(), String> {
    let authd_path = get_authd_bin_path();
    safe_authd
        .stop(Some(&authd_path))
        .map_err(|err| err.to_string())
}

pub fn authd_restart(safe_authd: &SafeAuthdClient) -> Result<(), String> {
    let authd_path = get_authd_bin_path();
    safe_authd
        .restart(Some(&authd_path))
        .map_err(|err| err.to_string())
}

pub fn authd_create(
    safe: &mut Safe,
    safe_authd: &SafeAuthdClient,
    sk: Option<String>,
    test_coins: bool,
) -> Result<(), String> {
    let secret = prompt_sensitive(None, "Secret:")?;
    let password = prompt_sensitive(None, "Password:")?;
    if test_coins {
        // We then generate a SafeKey with test-coins to use it for the account creation
        println!("Creating a SafeKey with test-coins...");
        let (_xorurl, key_pair) = safe.keys_create_preload_test_coins("1000.11")?;
        let kp = key_pair.ok_or("Faild to obtain the secret key of the newly created SafeKey")?;
        println!("Sending account creation request to authd...");
        safe_authd.create_acc(&kp.sk, &secret, &password)?;
        println!("Account was created successfully!");
        println!("SafeKey created and preloaded with test-coins. Owner key pair generated:");
        println!("Public Key = {}", kp.pk);
        println!("Secret Key = {}", kp.sk);
    } else {
        let sk = prompt_sensitive(sk, "Enter SafeKey's secret key to pay with:")?;
        println!("Sending account creation request to authd...");
        safe_authd.create_acc(&sk, &secret, &password)?;
        println!("Account was created successfully!");
    };
    Ok(())
}

pub fn authd_login(safe_authd: &mut SafeAuthdClient) -> Result<(), String> {
    let secret = prompt_sensitive(None, "Secret:")?;
    let password = prompt_sensitive(None, "Password:")?;
    println!("Sending login action request to authd...");
    safe_authd.log_in(&secret, &password)?;
    println!("Logged in successfully");
    Ok(())
}

pub fn authd_logout(safe_authd: &mut SafeAuthdClient) -> Result<(), String> {
    println!("Sending logout action request to authd...");
    safe_authd.log_out()?;
    println!("Logged out successfully");
    Ok(())
}

pub fn authd_apps(safe_authd: &SafeAuthdClient) -> Result<(), String> {
    println!("Requesting list of authorised apps from authd...");
    let authed_apps = safe_authd.authed_apps()?;
    pretty_print_authed_apps(authed_apps);

    Ok(())
}

pub fn authd_revoke(safe_authd: &SafeAuthdClient, app_id: String) -> Result<(), String> {
    println!("Sending application revocation request to authd...");
    safe_authd.revoke_app(&app_id)?;
    println!("Application revoked successfully");
    Ok(())
}

pub fn authd_auth_reqs(safe_authd: &SafeAuthdClient) -> Result<(), String> {
    println!("Requesting list of pending authorisation requests from authd...");
    let auth_reqs = safe_authd.auth_reqs()?;
    //pretty_print_authed_apps(auth_reqs);
    println!("{}", auth_reqs);
    Ok(())
}

pub fn authd_allow(safe_authd: &SafeAuthdClient, req_id: u32) -> Result<(), String> {
    println!("Sending request to authd to allow an authorisation request...");
    safe_authd.allow(req_id)?;
    println!("Authorisation request was allowed successfully");
    Ok(())
}

pub fn authd_deny(safe_authd: &SafeAuthdClient, req_id: u32) -> Result<(), String> {
    println!("Sending request to authd to deny an authorisation request...");
    safe_authd.deny(req_id)?;
    println!("Authorisation request was denied successfully");
    Ok(())
}

pub fn authd_subscribe(
    safe_authd: &mut SafeAuthdClient,
    notifs_endpoint: String,
    auth_allow_prompt: &'static AuthAllowPrompt,
) -> Result<(), String> {
    println!("Sending request to subscribe...");
    safe_authd.subscribe(&notifs_endpoint, auth_allow_prompt)?;
    println!("Subscribed successfully");
    Ok(())
}

#[allow(dead_code)]
pub fn authd_subscribe_url(
    safe_authd: &SafeAuthdClient,
    notifs_endpoint: String,
) -> Result<(), String> {
    println!("Sending request to subscribe URL...");
    safe_authd.subscribe_url(&notifs_endpoint)?;
    println!("URL subscribed successfully");
    Ok(())
}

pub fn authd_unsubscribe(
    safe_authd: &SafeAuthdClient,
    notifs_endpoint: String,
) -> Result<(), String> {
    println!("Sending request to unsubscribe...");
    safe_authd.unsubscribe(&notifs_endpoint)?;
    println!("Unsubscribed successfully");
    Ok(())
}

pub fn pretty_print_authed_apps(authed_apps: /*Vec<AuthedAppsList>*/ String) {
    let mut table = Table::new();
    table.add_row(row![bFg->"Authorised Applications"]);
    /*table.add_row(row![bFg->"Id", bFg->"Name", bFg->"Vendor", bFg->"Permissions"]);
    table.add_row(row![]);
    let all_app_iterator = authed_apps.iter();
    for app_info in all_app_iterator {
        let mut row = String::from("");
        for (cont, perms) in app_info.perms.iter() {
            row += &format!("{}: {:?}\n", cont, perms);
        }
        table.add_row(row![
            app_info.app.id,
            app_info.app.name,
            // app_info.app.scope || "",
            app_info.app.vendor,
            row,
        ]);
    }*/
    table.printstd();
    println!("{}", authed_apps);
}

// Private helpers

// TODO: use a different crate than rpassword as it has problems with some Windows shells including PowerShell
fn prompt_sensitive(arg: Option<String>, msg: &str) -> Result<String, String> {
    if let Some(str) = arg {
        Ok(str)
    } else {
        rpassword::read_password_from_tty(Some(msg))
            .map_err(|err| format!("Failed reading string from input: {}", err))
    }
}

fn get_authd_bin_path() -> String {
    let target_dir = match std::env::var("CARGO_TARGET_DIR") {
        Ok(target_dir) => target_dir,
        Err(_) => "target".to_string(),
    };

    if cfg!(debug_assertions) {
        format!("{}{}", target_dir, "/debug/safe-authd")
    } else {
        format!("{}{}", target_dir, "/release/safe-authd")
    }
}
