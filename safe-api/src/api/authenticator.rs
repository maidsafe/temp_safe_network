// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::sk_from_hex;
use super::{Error, Result};
use bincode::deserialize;
use futures::{stream, Future, Stream};
use log::{debug, info};
use safe_authenticator::ipc::{decode_ipc_msg, update_container_perms};
use safe_authenticator::revocation::revoke_app as safe_authenticator_revoke_app;
use safe_authenticator::{
    access_container, app_auth::authenticate, config, errors::AuthError, run as auth_run_helper,
    Authenticator,
};
use safe_core::client::Client;
use safe_core::ipc::req::{AuthReq, ContainerPermissions, ContainersReq, IpcReq, ShareMDataReq};
use safe_core::ipc::resp::{AccessContainerEntry, IpcResp};
use safe_core::ipc::{access_container_enc_key, decode_msg, encode_msg, IpcError, IpcMsg};
use safe_core::utils::symmetric_decrypt;
use safe_core::{client as safe_core_client, CoreError};
use safe_nd::{AppPermissions, Error as SndError, MDataAddress, PublicKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Type of the function/callback invoked for querying if an authorisation request shall be allowed.
// All the relevant information about the authorisation request is passed as args to the callback.
// type AuthAllowPrompt = dyn Fn(SafeAuthReqId, IpcReq) -> bool + std::marker::Send + std::marker::Sync;

pub type SafeAuthReq = IpcReq;
pub type SafeAuthReqId = u32;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthedApp {
    /// The App ID. It must be unique.
    pub id: String,
    /// The application friendly-name.
    pub name: String,
    /// The application provider/vendor (e.g. MaidSafe)
    pub vendor: String,
    /// Permissions granted, e.g. allowing to work with the user's coin balance.
    pub app_permissions: AppPermissions,
    /// Permissions granted to the app for named containers
    // TODO: ContainerPermissions will/shall be refactored to expose a struct defined in this crate
    pub containers: HashMap<String, ContainerPermissions>,
    /// If the app was given a dedicated named container for itself
    pub own_container: bool,
}

// Type of the list of authorised applications in a SAFE account
pub type AuthedAppsList = Vec<AuthedApp>;

// Authenticator API
#[derive(Default)]
pub struct SafeAuthenticator {
    safe_authenticator: Option<Authenticator>,
}

#[allow(dead_code)]
impl SafeAuthenticator {
    pub fn new(config_dir_path: Option<&str>) -> Self {
        if let Some(path) = config_dir_path {
            safe_core::config_handler::set_config_dir_path(path);
        }

        Self {
            safe_authenticator: None,
        }
    }

    /// # Log in
    ///
    /// Using an account already created, you can log in to
    /// the SAFE Network using the `Authenticator` daemon.
    ///
    /// ## Example
    /// ```ignore
    /// use safe_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's passphrase and password:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # safe_auth.create_acc(sk, my_secret, my_password).unwrap();
    /// let logged_in = safe_auth.log_in(my_secret, my_password);
    /// match logged_in {
    ///    Ok(()) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    ///```
    ///
    /// ## Error Example
    /// If the account does not exist, the function will return an appropriate error:
    ///```
    /// use safe_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// let not_logged_in = safe_auth.log_in("non", "existant");
    /// match not_logged_in {
    ///    Ok(()) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("Failed to log in"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    ///```
    pub fn log_in(&mut self, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to log in...");
        match Authenticator::login(passphrase, password, || info!("Disconnected from network")) {
            Ok(auth) => {
                debug!("Logged-in successfully");
                self.safe_authenticator = Some(auth);
                Ok(())
            }
            Err(err) => {
                let msg = match err {
                    AuthError::SndError(SndError::NoSuchLoginPacket) => {
                        "no SAFE account found with the passphrase provided".to_string()
                    }
                    AuthError::CoreError(CoreError::SymmetricDecipherFailure) => {
                        "unable to login with the password provided".to_string()
                    }
                    other => other.to_string(),
                };
                Err(Error::AuthError(format!("Failed to log in: {}", msg)))
            }
        }
    }

    pub fn log_out(&mut self) -> Result<()> {
        debug!("Dropping logged in session...");
        self.safe_authenticator = None;
        Ok(())
    }

    pub fn is_logged_in(&self) -> bool {
        let is_logged_in = self.safe_authenticator.is_some();
        debug!("Is logged in? {}", is_logged_in);
        is_logged_in
    }

    /// # Create Account
    /// Creates a new account on the SAFE Network.
    /// Returns an error if an account exists or if there was some
    /// problem during the account creation process.
    /// If the account is successfully created it keeps the logged in session (discarding a previous session)
    ///
    /// Note: This does _not_ perform any strength checks on the
    /// strings used to create the account.
    ///
    /// ## Example
    /// ```ignore
    /// use safe_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// let acc_created = safe_auth.create_acc(sk, my_secret, my_password);
    /// match acc_created {
    ///    Ok(()) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    ///```
    ///
    /// ## Error Example
    /// If an account with same passphrase already exists,
    /// the function will return an error:
    /// ```ignore
    /// use safe_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's passphrase and password:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # safe_auth.create_acc(sk, my_secret, my_password).unwrap();
    /// let acc_not_created = safe_auth.create_acc(sk, my_secret, my_password);
    /// match acc_not_created {
    ///    Ok(_) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("Failed to create an account"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    ///```
    pub fn create_acc(&mut self, sk: &str, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to create a SAFE account...");
        let secret_key = sk_from_hex(sk)?;

        match Authenticator::create_acc(passphrase, password, secret_key, || {
            // TODO: allow the caller to provide the callback function
            // eprintln!("{}", "Disconnected from network");
        }) {
            Ok(auth) => {
                debug!("Account just created successfully");
                self.safe_authenticator = Some(auth);
                Ok(())
            }
            Err(err) => {
                let msg = match err {
                    AuthError::SndError(SndError::LoginPacketExists) => {
                        "a SAFE account already exists with the passphrase provided".to_string()
                    }
                    other => other.to_string(),
                };
                Err(Error::AuthError(format!(
                    "Failed to create an account: {}",
                    msg
                )))
            }
        }
    }

    pub fn decode_req(&self, req: &str) -> Result<(SafeAuthReqId, SafeAuthReq)> {
        let safe_authenticator = get_safe_authenticator(&self.safe_authenticator)?;

        let req_msg = match decode_msg(req) {
            Ok(msg) => msg,
            Err(err) => {
                return Err(Error::AuthError(format!(
                    "Failed to decode the auth request string: {:?}",
                    err
                )));
            }
        };
        debug!("Auth request string decoded: {:?}", req_msg);

        let ipc_req = auth_run_helper(safe_authenticator, move |client| {
            decode_ipc_msg(client, req_msg)
        })
        .map_err(|err| Error::AuthenticatorError(format!("Failed to decode request: {}", err)))?;

        match ipc_req {
            Ok(IpcMsg::Req {
                req: IpcReq::Auth(app_auth_req),
                req_id,
            }) => Ok((req_id, SafeAuthReq::Auth(app_auth_req))),
            other => Err(Error::AuthError(format!(
                "Failed to decode string as an authorisation request, it's a: '{:?}'",
                other
            ))),
        }
    }

    /// # Authorise an application
    ///
    /// Using an account already created, you can log in to
    /// the SAFE Network and authorise an application.
    ///
    /// ## Example
    /// ```ignore
    /// use safe_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's passphrase and password:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # safe_auth.create_acc(sk, my_secret, my_password).unwrap();
    /// let auth_req = "bAAAAAAEXVK4SGAAAAAABAAAAAAAAAAAANZSXILTNMFUWI43BMZSS4Y3MNEAAQAAAAAAAAAAAKNAUMRJAINGESEAAAAAAAAAAABGWC2LEKNQWMZJONZSXIICMORSAAAIBAAAAAAAAAAAAOAAAAAAAAAAAL5YHKYTMNFRQCAAAAAAAAAAAAAAAAAAB";
    /// safe_auth.log_in(my_secret, my_password).unwrap();
    /// let auth_response = safe_auth.authorise_app(auth_req/*, &|_| true*/);
    /// match auth_response {
    ///    Ok(_) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    ///```
    /// ## Error Example
    /// ```ignore
    /// use safe_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's passphrase and password:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # safe_auth.create_acc(sk, my_secret, my_password).unwrap();
    /// /// Using an invalid auth request string
    /// let auth_req = "invalid-auth-req-string";
    /// safe_auth.log_in(my_secret, my_password).unwrap();
    /// let auth_response = safe_auth.authorise_app(auth_req/*, &|_| true*/);
    /// match auth_response {
    ///    Ok(_) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("EncodeDecodeError"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    ///```
    pub fn authorise_app(&self, req: &str) -> Result<String> {
        let safe_authenticator = get_safe_authenticator(&self.safe_authenticator)?;

        let req_msg = match decode_msg(req) {
            Ok(msg) => msg,
            Err(err) => {
                return Err(Error::AuthError(format!(
                    "Failed to decode the auth request string: {:?}",
                    err
                )));
            }
        };
        debug!("Auth request string decoded: {:?}", req_msg);

        let ipc_req = auth_run_helper(safe_authenticator, move |client| {
            decode_ipc_msg(client, req_msg)
        })
        .map_err(|err| Error::AuthenticatorError(format!("Failed to decode request: {}", err)))?;

        match ipc_req {
            Ok(IpcMsg::Req {
                req: IpcReq::Auth(app_auth_req),
                req_id,
            }) => {
                info!("Request was recognised as a general app auth request");
                debug!("Decoded request (req_id={:?}): {:?}", req_id, app_auth_req);
                gen_auth_response(safe_authenticator, req_id, app_auth_req)
            }
            Ok(IpcMsg::Req {
                req: IpcReq::Containers(cont_req),
                req_id,
            }) => {
                info!("Request was recognised as a containers auth request");
                debug!("Decoded request (req_id={:?}): {:?}", req_id, cont_req);
                gen_cont_auth_response(safe_authenticator, req_id, cont_req)
            }
            Ok(IpcMsg::Req {
                req: IpcReq::Unregistered(user_data),
                req_id,
            }) => {
                info!("Request was recognised as an unregistered auth request");
                debug!("Decoded request (req_id={:?}): {:?}", req_id, user_data);
                gen_unreg_auth_response(req_id)
            }
            Ok(IpcMsg::Req {
                req: IpcReq::ShareMData(share_mdata_req),
                req_id,
            }) => {
                info!("Request was recognised as a share MD auth request");
                debug!(
                    "Decoded request (req_id={:?}): {:?}",
                    req_id, share_mdata_req
                );
                gen_shared_md_auth_response(safe_authenticator, req_id, share_mdata_req)
            }
            Err((error_code, description, _err)) => Err(Error::AuthError(format!(
                "Failed decoding the auth request: {} - {:?}",
                error_code, description
            ))),
            Ok(IpcMsg::Resp { .. }) | Ok(IpcMsg::Revoked { .. }) | Ok(IpcMsg::Err(..)) => {
                Err(Error::AuthError(
                    "The request was not recognised as a valid auth request".to_string(),
                ))
            }
        }
    }

    /// # Get the list of applications authorised by this account
    ///
    /// Using an account already created, you can log in to
    /// the SAFE Network and get the list of all the applications that have
    /// been authorised so far.
    ///
    /// ## Example
    /// ```ignore
    /// use safe_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account which has been used
    /// /// to authorise some application already:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # safe_auth.create_acc(sk, my_secret, my_password).unwrap();
    /// safe_auth.log_in(my_secret, my_password).unwrap();
    /// # let auth_req = "bAAAAAAEXVK4SGAAAAAABAAAAAAAAAAAANZSXILTNMFUWI43BMZSS4Y3MNEAAQAAAAAAAAAAAKNAUMRJAINGESEAAAAAAAAAAABGWC2LEKNQWMZJONZSXIICMORSAAAIBAAAAAAAAAAAAOAAAAAAAAAAAL5YHKYTMNFRQCAAAAAAAAAAAAAAAAAAB";
    /// # safe_auth.authorise_app(auth_req/*, &|_| true*/).unwrap();
    /// /// Get the list of authorised apps
    /// let authed_apps = safe_auth.authed_apps();
    /// match authed_apps {
    ///    Ok(_) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    ///```
    pub fn authed_apps(&self) -> Result<AuthedAppsList> {
        let safe_authenticator = get_safe_authenticator(&self.safe_authenticator)?;

        debug!("Attempting to fetch list of authorised apps...");
        let authed_apps = auth_run_helper(safe_authenticator, move |client| {
            let c2 = client.clone();
            let c3 = client.clone();
            config::list_apps(client)
                .map(move |(_, auth_cfg)| (c2.access_container(), auth_cfg))
                .and_then(move |(access_container, auth_cfg)| {
                    c3.list_seq_mdata_entries(access_container.name(), access_container.type_tag())
                        .map_err(From::from)
                        .map(move |entries| (access_container, entries, auth_cfg))
                })
                .and_then(move |(access_container, entries, auth_cfg)| {
                    let nonce = access_container.nonce().ok_or_else(|| {
                        AuthError::from("No nonce on access container's MDataInfo")
                    })?;

                    let mut authed_apps_list = AuthedAppsList::new();
                    for app in auth_cfg.values() {
                        let key = access_container_enc_key(&app.info.id, &app.keys.enc_key, nonce)?;

                        // Empty entry means it has been deleted.
                        let entry = match entries.get(&key) {
                            Some(entry) if !entry.data.is_empty() => Some(entry),
                            _ => None,
                        };

                        if let Some(entry) = entry {
                            let plaintext = symmetric_decrypt(&entry.data, &app.keys.enc_key)?;
                            let app_access = deserialize::<AccessContainerEntry>(&plaintext)?;

                            let mut containers = HashMap::new();
                            for (container_name, (_mdata_info, permission_set)) in app_access {
                                let _ = containers.insert(container_name, permission_set);
                            }

                            authed_apps_list.push(AuthedApp {
                                id: app.info.id.clone(),
                                name: app.info.name.clone(),
                                vendor: app.info.vendor.clone(),
                                app_permissions: AppPermissions {
                                    //TODO: retrieve the app permissions
                                    transfer_coins: true,
                                    perform_mutations: true,
                                    get_balance: true,
                                },
                                containers,
                                own_container: false, //TODO: retrieve the own container flag
                            });
                        }
                    }

                    debug!(
                        "Returning list of authorised applications: {:?}",
                        authed_apps_list
                    );
                    Ok(authed_apps_list)
                })
                .map_err(AuthError::from)
        })
        .map_err(|err| {
            Error::AuthenticatorError(format!(
                "Failed to obtain list of authorised applications: {}",
                err
            ))
        })?;

        Ok(authed_apps)
    }

    /// # Revoke all permissions from an application
    ///
    /// Using an account already created, you can log in to
    /// the SAFE Network and revoke all permissions previously granted to an
    /// application by providing its ID.
    ///
    /// ## Example
    /// ```ignore
    /// use safe_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account which has been used
    /// /// to authorise some application already:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # safe_auth.create_acc(sk, my_secret, my_password).unwrap();
    /// safe_auth.log_in(my_secret, my_password).unwrap();
    /// # let auth_req = "bAAAAAAEXVK4SGAAAAAABAAAAAAAAAAAANZSXILTNMFUWI43BMZSS4Y3MNEAAQAAAAAAAAAAAKNAUMRJAINGESEAAAAAAAAAAABGWC2LEKNQWMZJONZSXIICMORSAAAIBAAAAAAAAAAAAOAAAAAAAAAAAL5YHKYTMNFRQCAAAAAAAAAAAAAAAAAAB";
    /// # safe_auth.authorise_app(auth_req/*, &|_| true*/).unwrap();
    /// /// Revoke all permissions from app with ID `net.maidsafe.cli`
    /// let revoked = safe_auth.revoke_app("net.maidsafe.cli");
    /// match revoked {
    ///    Ok(_) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    /// ```
    ///
    /// ## Error Example
    /// ```ignore
    /// use safe_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account which has been used
    /// /// to authorise some application already:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # safe_auth.create_acc(sk, my_secret, my_password).unwrap();
    /// safe_auth.log_in(my_secret, my_password).unwrap();
    /// /// Try to revoke permissions with an incorrect app ID
    /// let revoked = safe_auth.revoke_app("invalid-app-id");
    /// match revoked {
    ///    Ok(_) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("UnknownApp"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    ///```
    pub fn revoke_app(&self, app_id: &str) -> Result<()> {
        let safe_authenticator = get_safe_authenticator(&self.safe_authenticator)?;
        let id = app_id.to_string();
        auth_run_helper(safe_authenticator, |client| {
            safe_authenticator_revoke_app(client, &id).and_then(move |_| {
                debug!("Application sucessfully revoked: {}", id);
                Ok(())
            })
        })
        .map_err(|err| Error::AuthError(format!("Failed to revoke permissions: {}", err)))
    }
}

// Helper to unwrap the Authenticator if it's logged in
fn get_safe_authenticator(safe_authenticator: &Option<Authenticator>) -> Result<&Authenticator> {
    safe_authenticator.as_ref().ok_or_else(|| {
        Error::AuthError("You need to log in to a SAFE Network account first".to_string())
    })
}

// Helper function to generate an app authorisation response
#[allow(dead_code)]
fn gen_auth_denied_response(req_id: SafeAuthReqId) -> Result<String> {
    debug!("Encoding auth denied response...");
    let resp = encode_msg(&IpcMsg::Resp {
        req_id,
        resp: IpcResp::Auth(Err(IpcError::AuthDenied)),
    })
    .map_err(|err| Error::AuthenticatorError(format!("Failed to encode response: {:?}", err)))?;

    debug!("Returning auth response generated: {:?}", resp);

    Ok(resp)
}

// Helper function to generate an app authorisation response
fn gen_auth_response(
    authenticator: &Authenticator,
    req_id: SafeAuthReqId,
    auth_req: AuthReq,
) -> Result<String> {
    let auth_granted = auth_run_helper(authenticator, move |client| authenticate(client, auth_req))
        .map_err(|err| {
            Error::AuthenticatorError(format!(
                "Failed to authorise application on the network: {}",
                err
            ))
        })?;

    debug!("Encoding response... {:?}", auth_granted);
    let resp = encode_msg(&IpcMsg::Resp {
        req_id,
        resp: IpcResp::Auth(Ok(auth_granted)),
    })
    .map_err(|err| Error::AuthenticatorError(format!("Failed to encode response: {:?}", err)))?;

    debug!("Returning auth response generated: {:?}", resp);

    Ok(resp)
}

// Helper function to generate a containers authorisation response
fn gen_cont_auth_response(
    authenticator: &Authenticator,
    req_id: SafeAuthReqId,
    cont_req: ContainersReq,
) -> Result<String> {
    let permissions = cont_req.containers.clone();
    let app_id = cont_req.app.id.clone();

    auth_run_helper(authenticator, move |client| {
        let c2 = client.clone();
        let c3 = client.clone();
        let c4 = client.clone();

        config::get_app(client, &app_id)
            .and_then(move |app| {
                let sign_pk = PublicKey::from(app.keys.bls_pk);
                update_container_perms(&c2, permissions, sign_pk).map(move |perms| (app, perms))
            })
            .and_then(move |(app, mut perms)| {
                let app_keys = app.keys;
                access_container::fetch_entry(&c3, &app_id, app_keys.clone()).then(move |res| {
                    let version = match res {
                        // Updating an existing entry
                        Ok((version, Some(mut existing_perms))) => {
                            for (key, val) in perms {
                                let _ = existing_perms.insert(key, val);
                            }
                            perms = existing_perms;
                            version + 1
                        }

                        // Adding a new access container entry
                        Ok((_, None))
                        | Err(AuthError::CoreError(CoreError::DataError(
                            safe_nd::Error::NoSuchEntry,
                        ))) => 0,

                        // Error has occurred while trying to get an
                        // existing entry
                        Err(e) => return Err(e),
                    };
                    Ok((version, app_id, app_keys, perms))
                })
            })
            .and_then(move |(version, app_id, app_keys, perms)| {
                access_container::put_entry(&c4, &app_id, &app_keys, &perms, version)
            })
            .and_then(move |_| {
                debug!("Encoding response...");
                let resp = encode_msg(&IpcMsg::Resp {
                    req_id,
                    resp: IpcResp::Containers(Ok(())),
                })?;

                debug!("Returning containers auth response generated: {:?}", resp);
                Ok(resp)
            })
            .map_err(AuthError::from)
    })
    .map_err(|err| Error::AuthError(format!("Failed to generate response: {}", err)))
}

// Helper function to generate an unregistered authorisation response
fn gen_unreg_auth_response(req_id: SafeAuthReqId) -> Result<String> {
    let bootstrap_cfg = safe_core_client::bootstrap_config().map_err(|err| {
        Error::AuthenticatorError(format!(
            "Failed to obtain bootstrap info for response: {}",
            err
        ))
    })?;

    debug!("Encoding response... {:?}", bootstrap_cfg);
    let resp = encode_msg(&IpcMsg::Resp {
        req_id,
        resp: IpcResp::Unregistered(Ok(bootstrap_cfg)),
    })
    .map_err(|err| Error::AuthenticatorError(format!("Failed to encode response: {:?}", err)))?;

    debug!("Returning unregistered auth response generated: {:?}", resp);
    Ok(resp)
}

// Helper function to generate an authorisation response for sharing MD
fn gen_shared_md_auth_response(
    authenticator: &Authenticator,
    req_id: SafeAuthReqId,
    share_mdata_req: ShareMDataReq,
) -> Result<String> {
    auth_run_helper(authenticator, move |client| {
        let client_cloned0 = client.clone();
        let client_cloned1 = client.clone();
        config::get_app(client, &share_mdata_req.app.id).and_then(move |app_info| {
            let user = PublicKey::from(app_info.keys.bls_pk);
            let num_mdata = share_mdata_req.mdata.len();
            stream::iter_ok(share_mdata_req.mdata.into_iter())
                .map(move |mdata| {
                    client_cloned0
                        .get_seq_mdata_shell(mdata.name, mdata.type_tag)
                        .map(|md| (md.version(), mdata))
                })
                .buffer_unordered(num_mdata)
                .map(move |(version, mdata)| {
                    let address = MDataAddress::Seq {
                        name: mdata.name,
                        tag: mdata.type_tag,
                    };
                    client_cloned1.set_mdata_user_permissions(
                        address,
                        user,
                        mdata.perms,
                        version + 1,
                    )
                })
                .buffer_unordered(num_mdata)
                .map_err(AuthError::from)
                .for_each(|()| Ok(()))
                .and_then(move |()| {
                    debug!("Encoding response...");
                    let resp = encode_msg(&IpcMsg::Resp {
                        req_id,
                        resp: IpcResp::ShareMData(Ok(())),
                    })?;

                    debug!("Returning shared MD auth response generated: {:?}", resp);
                    Ok(resp)
                })
        })
    })
    .map_err(|err| Error::AuthError(format!("Failed to generate response: {}", err)))
}
