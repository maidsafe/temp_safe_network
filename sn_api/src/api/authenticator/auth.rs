// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::common::sk_from_hex;
use crate::{AuthedApp, AuthedAppsList, Error, Result, SafeAuthReq, SafeAuthReqId};
use bincode::deserialize;
use log::{debug, info};
use safe_authenticator::{
    access_container, access_container::update_container_perms, app_auth::authenticate, config,
    errors::AuthError, ipc::decode_ipc_msg,
    revocation::revoke_app as safe_authenticator_revoke_app, Authenticator,
};
use safe_core::{
    client as safe_core_client,
    client::Client,
    core_structs::{access_container_enc_key, AccessContainerEntry},
    ipc::{
        decode_msg, encode_msg,
        req::{AuthReq, ContainersReq, IpcReq, ShareMDataReq},
        resp::IpcResp,
        IpcMsg,
    },
    utils::symmetric_decrypt,
    CoreError,
};
use sn_data_types::{AppPermissions, ClientFullId, Error as SndError, MDataAddress};
use std::collections::HashMap;

// Authenticator API
#[derive(Default)]
pub struct SafeAuthenticator {
    safe_authenticator: Option<Authenticator>,
}

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
    /// use sn_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's passphrase and password:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # async_std::task::block_on(async {
    /// # safe_auth.create_acc(sk, my_secret, my_password).await.unwrap();
    /// let logged_in = safe_auth.log_in(my_secret, my_password).await;
    /// match logged_in {
    ///    Ok(()) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    /// # });
    ///```
    ///
    /// ## Error Example
    /// If the account does not exist, the function will return an appropriate error:
    ///```ignore
    /// use sn_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # async_std::task::block_on(async {
    /// let not_logged_in = safe_auth.log_in("non", "existant").await;
    /// match not_logged_in {
    ///    Ok(()) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("Failed to log in"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    /// # });
    ///```
    pub async fn log_in(&mut self, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to log in...");
        match Authenticator::login(passphrase, password, || info!("Disconnected from network"))
            .await
        {
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
                        "unable to log in with the password provided".to_string()
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
    /// use sn_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # async_std::task::block_on(async {
    /// let acc_created = safe_auth.create_acc(sk, my_secret, my_password).await;
    /// match acc_created {
    ///    Ok(()) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    /// # });
    ///```
    ///
    /// ## Error Example
    /// If an account with same passphrase already exists,
    /// the function will return an error:
    /// ```ignore
    /// use sn_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's passphrase and password:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # async_std::task::block_on(async {
    /// # safe_auth.create_acc(sk, my_secret, my_password).await.unwrap();
    /// let acc_not_created = safe_auth.create_acc(sk, my_secret, my_password).await;
    /// match acc_not_created {
    ///    Ok(_) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("Failed to create an account"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    /// # });
    ///```
    pub async fn create_acc(&mut self, sk: &str, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to create a SAFE account...");
        let secret_key = sk_from_hex(sk)?;

        match Authenticator::create_client_with_acc(
            passphrase,
            password,
            ClientFullId::from(secret_key),
            || {
                // TODO: allow the caller to provide the callback function
                // eprintln!("{}", "Disconnected from network");
            },
        )
        .await
        {
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

    pub async fn decode_req(&self, req: &str) -> Result<(SafeAuthReqId, SafeAuthReq)> {
        let client = &get_safe_authenticator(&self.safe_authenticator)?.client;

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

        let ipc_req = decode_ipc_msg(client, req_msg).await.map_err(|err| {
            Error::AuthenticatorError(format!("Failed to decode request: {}", err))
        })?;

        match ipc_req {
            Ok(IpcMsg::Req {
                request: IpcReq::Auth(app_auth_req),
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
    /// use sn_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's passphrase and password:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # async_std::task::block_on(async {
    /// # safe_auth.create_acc(sk, my_secret, my_password).await.unwrap();
    /// let auth_req = "bAAAAAAEXVK4SGAAAAAABAAAAAAAAAAAANZSXILTNMFUWI43BMZSS4Y3MNEAAQAAAAAAAAAAAKNAUMRJAINGESEAAAAAAAAAAABGWC2LEKNQWMZJONZSXIICMORSAAAIBAAAAAAAAAAAAOAAAAAAAAAAAL5YHKYTMNFRQCAAAAAAAAAAAAAAAAAAB";
    /// safe_auth.log_in(my_secret, my_password).await.unwrap();
    /// let auth_response = safe_auth.authorise_app(auth_req/*, &|_| true*/).await;
    /// match auth_response {
    ///    Ok(_) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    /// # });
    ///```
    /// ## Error Example
    /// ```ignore
    /// use sn_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's passphrase and password:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # async_std::task::block_on(async {
    /// # safe_auth.create_acc(sk, my_secret, my_password).await.unwrap();
    /// /// Using an invalid auth request string
    /// let auth_req = "invalid-auth-req-string";
    /// safe_auth.log_in(my_secret, my_password).await.unwrap();
    /// let auth_response = safe_auth.authorise_app(auth_req/*, &|_| true*/).await;
    /// match auth_response {
    ///    Ok(_) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("EncodeDecodeError"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    /// # });
    ///```
    pub async fn authorise_app(&self, req: &str) -> Result<String> {
        let safe_authenticator = get_safe_authenticator(&self.safe_authenticator)?;
        let client = &safe_authenticator.client;
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

        let ipc_req = decode_ipc_msg(client, req_msg).await.map_err(|err| {
            Error::AuthenticatorError(format!("Failed to decode request: {}", err))
        })?;

        match ipc_req {
            Ok(IpcMsg::Req {
                request: IpcReq::Auth(app_auth_req),
                req_id,
            }) => {
                info!("Request was recognised as a general app auth request");
                debug!("Decoded request (req_id={:?}): {:?}", req_id, app_auth_req);
                gen_auth_response(safe_authenticator, req_id, app_auth_req).await
            }
            Ok(IpcMsg::Req {
                request: IpcReq::Containers(cont_req),
                req_id,
            }) => {
                info!("Request was recognised as a containers auth request");
                debug!("Decoded request (req_id={:?}): {:?}", req_id, cont_req);
                gen_cont_auth_response(safe_authenticator, req_id, cont_req).await
            }
            Ok(IpcMsg::Req {
                request: IpcReq::Unregistered(user_data),
                req_id,
            }) => {
                info!("Request was recognised as an unregistered auth request");
                debug!("Decoded request (req_id={:?}): {:?}", req_id, user_data);
                gen_unreg_auth_response(req_id)
            }
            Ok(IpcMsg::Req {
                request: IpcReq::ShareMData(share_mdata_req),
                req_id,
            }) => {
                info!("Request was recognised as a share MD auth request");
                debug!(
                    "Decoded request (req_id={:?}): {:?}",
                    req_id, share_mdata_req
                );
                gen_shared_md_auth_response(safe_authenticator, req_id, share_mdata_req).await
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
    /// use sn_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account which has been
    /// /// already used to authorise some application:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # async_std::task::block_on(async {
    /// # safe_auth.create_acc(sk, my_secret, my_password).await.unwrap();
    /// safe_auth.log_in(my_secret, my_password).await.unwrap();
    /// # let auth_req = "bAAAAAAEXVK4SGAAAAAABAAAAAAAAAAAANZSXILTNMFUWI43BMZSS4Y3MNEAAQAAAAAAAAAAAKNAUMRJAINGESEAAAAAAAAAAABGWC2LEKNQWMZJONZSXIICMORSAAAIBAAAAAAAAAAAAOAAAAAAAAAAAL5YHKYTMNFRQCAAAAAAAAAAAAAAAAAAB";
    /// # safe_auth.authorise_app(auth_req/*, &|_| true*/).await.unwrap();
    /// /// Get the list of authorised apps
    /// let authed_apps = safe_auth.authed_apps().await;
    /// match authed_apps {
    ///    Ok(_) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    /// # });
    ///```
    pub async fn authed_apps(&self) -> Result<AuthedAppsList> {
        let client = &get_safe_authenticator(&self.safe_authenticator)?.client;

        debug!("Attempting to fetch list of authorised apps...");

        let (_, auth_cfg) = config::list_apps(client).await.map_err(|err| {
            Error::AuthenticatorError(format!(
                "Failed to obtain list of authorised applications: {}",
                err
            ))
        })?;
        let access_container = client.access_container().await;
        let entries = client
            .list_seq_mdata_entries(access_container.name(), access_container.type_tag())
            .await
            .map_err(|err| {
                Error::AuthenticatorError(format!("Failed to read access container: {}", err))
            })?;

        let nonce = access_container.nonce().ok_or_else(|| {
            Error::AuthenticatorError("No nonce on access container's MDataInfo".to_string())
        })?;

        let mut authed_apps = AuthedAppsList::new();
        for app in auth_cfg.values() {
            let key = access_container_enc_key(&app.info.id, &app.keys.enc_key, nonce).map_err(
                |err| {
                    Error::AuthenticatorError(format!(
                        "Failed to generate an acceess container encryption key: {}",
                        err
                    ))
                },
            )?;

            // Empty entry means it has been deleted.
            match entries.get(&key) {
                Some(entry) if !entry.data.is_empty() => {
                    let plaintext =
                        symmetric_decrypt(&entry.data, &app.keys.enc_key).map_err(|err| {
                            Error::AuthenticatorError(format!(
                                "Failed to obtain list of authorised applications: {}",
                                err
                            ))
                        })?;
                    let app_access =
                        deserialize::<AccessContainerEntry>(&plaintext).map_err(|err| {
                            Error::AuthenticatorError(format!(
                                "Failed to deserialise access container entry: {}",
                                err
                            ))
                        })?;

                    let mut containers = HashMap::new();
                    for (container_name, (_mdata_info, permission_set)) in app_access {
                        let _ = containers.insert(container_name, permission_set);
                    }

                    authed_apps.push(AuthedApp {
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
                _ => {}
            }
        }

        debug!(
            "Returning list of authorised applications: {:?}",
            authed_apps
        );

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
    /// use sn_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account which has been
    /// /// already used to authorise some application:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # async_std::task::block_on(async {
    /// # safe_auth.create_acc(sk, my_secret, my_password).await.unwrap();
    /// safe_auth.log_in(my_secret, my_password).await.unwrap();
    /// # let auth_req = "bAAAAAAEXVK4SGAAAAAABAAAAAAAAAAAANZSXILTNMFUWI43BMZSS4Y3MNEAAQAAAAAAAAAAAKNAUMRJAINGESEAAAAAAAAAAABGWC2LEKNQWMZJONZSXIICMORSAAAIBAAAAAAAAAAAAOAAAAAAAAAAAL5YHKYTMNFRQCAAAAAAAAAAAAAAAAAAB";
    /// # safe_auth.authorise_app(auth_req/*, &|_| true*/).await.unwrap();
    /// /// Revoke all permissions from app with ID `net.maidsafe.cli`
    /// let revoked = safe_auth.revoke_app("net.maidsafe.cli").await;
    /// match revoked {
    ///    Ok(_) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    /// # });
    /// ```
    ///
    /// ## Error Example
    /// ```ignore
    /// use sn_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account which has been
    /// /// already used to authorise some application:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # async_std::task::block_on(async {
    /// # safe_auth.create_acc(sk, my_secret, my_password).await.unwrap();
    /// safe_auth.log_in(my_secret, my_password).await.unwrap();
    /// /// Try to revoke permissions with an incorrect app ID
    /// let revoked = safe_auth.revoke_app("invalid-app-id").await;
    /// match revoked {
    ///    Ok(_) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("UnknownApp"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    /// # });
    ///```
    pub async fn revoke_app(&self, app_id: &str) -> Result<()> {
        let client = &get_safe_authenticator(&self.safe_authenticator)?.client;
        let id = app_id.to_string();
        safe_authenticator_revoke_app(client, &id)
            .await
            .map_err(|err| Error::AuthError(format!("Failed to revoke permissions: {}", err)))?;

        debug!("Application sucessfully revoked: {}", id);
        Ok(())
    }
}

// Helper to unwrap the Authenticator if it's logged in
fn get_safe_authenticator(safe_authenticator: &Option<Authenticator>) -> Result<&Authenticator> {
    safe_authenticator.as_ref().ok_or_else(|| {
        Error::AuthError("You need to log in to a SAFE Network account first".to_string())
    })
}

// Helper function to generate an app authorisation response
async fn gen_auth_response(
    authenticator: &Authenticator,
    req_id: SafeAuthReqId,
    auth_req: AuthReq,
) -> Result<String> {
    let auth_granted = authenticate(&authenticator.client, auth_req)
        .await
        .map_err(|err| {
            Error::AuthenticatorError(format!(
                "Failed to authorise application on the network: {}",
                err
            ))
        })?;

    debug!("Encoding response with auth credentials auth granted...");
    let resp = encode_msg(&IpcMsg::Resp {
        req_id,
        response: IpcResp::Auth(Ok(auth_granted)),
    })
    .map_err(|err| Error::AuthenticatorError(format!("Failed to encode response: {:?}", err)))?;

    debug!("Returning auth response generated");

    Ok(resp)
}

// Helper function to generate a containers authorisation response
async fn gen_cont_auth_response(
    authenticator: &Authenticator,
    req_id: SafeAuthReqId,
    cont_req: ContainersReq,
) -> Result<String> {
    let permissions = cont_req.containers.clone();
    let app_id = cont_req.app.id;
    let client = &authenticator.client;

    let app = config::get_app(client, &app_id)
        .await
        .map_err(|err| Error::AuthError(format!("Failed to generate response: {}", err)))?;

    let sign_pk = app.keys.public_key();
    let mut perms = update_container_perms(&client, permissions, sign_pk)
        .await
        .map_err(|err| Error::AuthError(format!("Failed to update permissions: {}", err)))?;

    let app_keys = app.keys;
    let version =
        match access_container::fetch_entry(client.clone(), app_id.clone(), app_keys.clone()).await
        {
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
            | Err(AuthError::CoreError(CoreError::DataError(sn_data_types::Error::NoSuchEntry))) => 0,

            // Error has occurred while trying to get an
            // existing entry
            Err(e) => return Err(Error::AuthError(format!("{}", e))),
        };

    access_container::put_entry(&client, &app_id, &app_keys, &perms, version)
        .await
        .map_err(|err| {
            Error::AuthError(format!(
                "Failed to write permissions in access container: {}",
                err
            ))
        })?;

    debug!("Encoding response...");
    let resp = encode_msg(&IpcMsg::Resp {
        req_id,
        response: IpcResp::Containers(Ok(())),
    })
    .map_err(|err| Error::AuthError(format!("Failed to encode response: {:?}", err)))?;

    debug!("Returning containers auth response generated: {:?}", resp);
    Ok(resp)
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
        response: IpcResp::Unregistered(Ok(bootstrap_cfg)),
    })
    .map_err(|err| Error::AuthenticatorError(format!("Failed to encode response: {:?}", err)))?;

    debug!("Returning unregistered auth response generated: {:?}", resp);
    Ok(resp)
}

// Helper function to generate an authorisation response for sharing MD
async fn gen_shared_md_auth_response(
    authenticator: &Authenticator,
    req_id: SafeAuthReqId,
    share_mdata_req: ShareMDataReq,
) -> Result<String> {
    let client = &authenticator.client;

    let app_info = config::get_app(client, &share_mdata_req.app.id)
        .await
        .map_err(|err| {
            Error::AuthenticatorError(format!("Failed to find application by its id: {}", err))
        })?;

    let user = app_info.keys.public_key();
    for mdata in share_mdata_req.mdata.iter() {
        let md = client
            .get_seq_mdata_shell(mdata.name, mdata.type_tag)
            .await
            .map_err(|err| {
                Error::AuthenticatorError(format!("Failed to obtain MData shell: {}", err))
            })?;

        let version = md.version();
        let address = MDataAddress::Seq {
            name: mdata.name,
            tag: mdata.type_tag,
        };
        client
            .set_mdata_user_permissions(address, user, mdata.perms.clone(), version + 1)
            .await
            .map_err(|err| Error::AuthError(format!("Failed set user permissions: {}", err)))?;
    }

    debug!("Encoding response...");
    let resp = encode_msg(&IpcMsg::Resp {
        req_id,
        response: IpcResp::ShareMData(Ok(())),
    })
    .map_err(|err| Error::AuthenticatorError(format!("Failed to encode response: {:?}", err)))?;

    debug!("Returning shared MD auth response generated: {:?}", resp);
    Ok(resp)
}
