// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::helpers::sk_from_hex;
use super::{Error, ResultReturn};
use futures::{stream, Future, Stream};
use log::{debug, info};
use maidsafe_utilities::serialisation::deserialise;
use safe_authenticator::ipc::{decode_ipc_msg, update_container_perms};
use safe_authenticator::revocation::revoke_app as safe_authenticator_revoke_app;
use safe_authenticator::{
    access_container, app_auth::authenticate, config, errors::AuthError, run as auth_run_helper,
    Authenticator,
};
use safe_core::client::Client;
use safe_core::ipc::req::{
    AppExchangeInfo, AuthReq, ContainerPermissions, ContainersReq, IpcReq, ShareMDataReq,
};
use safe_core::ipc::resp::{AccessContainerEntry, IpcResp};
use safe_core::ipc::{access_container_enc_key, decode_msg, encode_msg, IpcError, IpcMsg};
use safe_core::utils::symmetric_decrypt;
use safe_core::{client as safe_core_client, CoreError};
use safe_nd::{MDataAddress, PublicKey};

// Type of the function/callback invoked for querying if an authorisation request shall be allowed.
// All the relevant information about the authorisation request is passed as args to the callback.
// type AuthAllowPrompt = dyn Fn(SafeAuthReqId, IpcReq) -> bool + std::marker::Send + std::marker::Sync;

pub type SafeAuthReq = IpcReq;
pub type SafeAuthReqId = u32;

#[derive(Debug)]
pub struct AuthedAppsList {
    pub app: AppExchangeInfo,
    pub perms: Vec<(String, ContainerPermissions)>,
}

// Authenticator API
#[derive(Default)]
pub struct SafeAuthenticator {
    safe_authenticator: Option<Authenticator>,
}

#[allow(dead_code)]
impl SafeAuthenticator {
    pub fn new() -> Self {
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
    /// let mut safe_auth = SafeAuthenticator::new();
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's secret and password:
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
    /// let mut safe_auth = SafeAuthenticator::new();
    /// let not_logged_in = safe_auth.log_in("non", "existant");
    /// match not_logged_in {
    ///    Ok(()) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("Failed to log in"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    ///```
    pub fn log_in(&mut self, secret: &str, password: &str) -> ResultReturn<()> {
        debug!("Attempting to log in...");
        match Authenticator::login(secret, password, || info!("Disconnected from network")) {
            Ok(auth) => {
                debug!("Logged-in successfully");
                self.safe_authenticator = Some(auth);
                Ok(())
            }
            Err(err) => Err(Error::AuthError(format!("Failed to log in: {:?}", err))),
        }
    }

    pub fn log_out(&mut self) -> ResultReturn<()> {
        debug!("Dropping logged in session...");
        self.safe_authenticator = None;
        Ok(())
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
    /// let mut safe_auth = SafeAuthenticator::new();
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
    /// If an account with same secret already exists,
    /// the function will return an error:
    /// ```ignore
    /// use safe_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new();
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's secret and password:
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
    pub fn create_acc(&mut self, sk: &str, secret: &str, password: &str) -> ResultReturn<()> {
        debug!("Attempting to create a SAFE account...");
        let secret_key = sk_from_hex(sk)?;

        match Authenticator::create_acc(secret, password, secret_key, || {
            // TODO: allow the caller to provide the callback function
            // eprintln!("{}", "Disconnected from network");
        }) {
            Ok(auth) => {
                debug!("Account just created successfully");
                self.safe_authenticator = Some(auth);
                Ok(())
            }
            Err(err) => Err(Error::AuthError(format!(
                "Failed to create an account: {:?}",
                err
            ))),
        }
    }

    pub fn decode_req(&self, req: &str) -> ResultReturn<(SafeAuthReqId, SafeAuthReq)> {
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
        .unwrap();

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
    /// let mut safe_auth = SafeAuthenticator::new();
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's secret and password:
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
    /// let mut safe_auth = SafeAuthenticator::new();
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing account's secret and password:
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
    pub fn authorise_app(
        &self,
        req: &str,
        //allow: &'static AuthAllowPrompt,
    ) -> ResultReturn<String> {
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
        .unwrap();
        match ipc_req {
            Ok(IpcMsg::Req {
                req: IpcReq::Auth(app_auth_req),
                req_id,
            }) => {
                info!("Request was recognised as a general app auth request");
                debug!("Decoded request (req_id={:?}): {:?}", req_id, app_auth_req);

                /*debug!("Checking if the authorisation shall be allowed...");
                if !allow(req_id, IpcReq::Auth(app_auth_req.clone())) {
                    debug!("Authorisation request was denied!");
                    return gen_auth_denied_response(req_id);
                }*/

                debug!("Allowed!. Attempting to authorise application...");
                gen_auth_response(safe_authenticator, req_id, app_auth_req)
            }
            Ok(IpcMsg::Req {
                req: IpcReq::Containers(cont_req),
                req_id,
            }) => {
                info!("Request was recognised as a containers auth request");
                debug!("Decoded request (req_id={:?}): {:?}", req_id, cont_req);

                /*debug!("Checking if the containers authorisation shall be allowed...");
                if !allow(req_id, IpcReq::Containers(cont_req.clone())) {
                    debug!("Authorisation request was denied!");
                    return gen_auth_denied_response(req_id);
                }*/

                debug!("Allowed!. Attempting to grant permissions to the containers...");
                gen_cont_auth_response(safe_authenticator, req_id, cont_req)
            }
            Ok(IpcMsg::Req {
                req: IpcReq::Unregistered(user_data),
                req_id,
            }) => {
                info!("Request was recognised as an unregistered auth request");
                debug!("Decoded request (req_id={:?}): {:?}", req_id, user_data);

                /*debug!("Checking if the authorisation shall be allowed...");
                if !allow(req_id, IpcReq::Unregistered(user_data)) {
                    debug!("Authorisation request was denied!");
                    return gen_auth_denied_response(req_id);
                }*/

                debug!("Allowed!");
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

                /*debug!("Checking if the authorisation to share a MD shall be allowed...");
                if !allow(req_id, IpcReq::ShareMData(share_mdata_req.clone())) {
                    debug!("Authorisation request was denied!");
                    return gen_auth_denied_response(req_id);
                }*/

                debug!("Allowed!. Attempting to grant permissions to the MD...");
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
    /// let mut safe_auth = SafeAuthenticator::new();
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
    pub fn authed_apps(&self) -> ResultReturn<Vec<AuthedAppsList>> {
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

                    let mut apps = Vec::new();
                    for app in auth_cfg.values() {
                        let key = access_container_enc_key(&app.info.id, &app.keys.enc_key, nonce)?;

                        // Empty entry means it has been deleted.
                        let entry = match entries.get(&key) {
                            Some(entry) if !entry.data.is_empty() => Some(entry),
                            _ => None,
                        };

                        let mut cont_perms = Vec::new();
                        if let Some(entry) = entry {
                            let plaintext = symmetric_decrypt(&entry.data, &app.keys.enc_key)?;
                            let app_access = deserialise::<AccessContainerEntry>(&plaintext)?;

                            for (key, (_mdata_info, perms)) in app_access.into_iter() {
                                cont_perms.push((key, perms));
                            }

                            apps.push(AuthedAppsList {
                                app: app.info.clone(),
                                perms: cont_perms,
                            });
                        }
                    }

                    debug!("Returning list of authorised applications: {:?}", apps);
                    Ok(apps)
                })
                .map_err(AuthError::from)
        })
        .unwrap();

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
    /// let mut safe_auth = SafeAuthenticator::new();
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
    /// let mut safe_auth = SafeAuthenticator::new();
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
    pub fn revoke_app(&self, app_id: &str) -> ResultReturn<()> {
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

// Helper to unwrap the Authenticator is it's logged in
fn get_safe_authenticator(
    safe_authenticator: &Option<Authenticator>,
) -> ResultReturn<&Authenticator> {
    safe_authenticator.as_ref().ok_or_else(|| {
        Error::AuthError("You need to log in to a SAFE Network account first".to_string())
    })
}

// Helper function to generate an app authorisation response
#[allow(dead_code)]
fn gen_auth_denied_response(req_id: SafeAuthReqId) -> ResultReturn<String> {
    debug!("Encoding auth denied response...");
    let resp = encode_msg(&IpcMsg::Resp {
        req_id,
        resp: IpcResp::Auth(Err(IpcError::AuthDenied)),
    })
    .unwrap();
    debug!("Returning auth response generated: {:?}", resp);

    Ok(resp)
}

// Helper function to generate an app authorisation response
fn gen_auth_response(
    authenticator: &Authenticator,
    req_id: SafeAuthReqId,
    auth_req: AuthReq,
) -> ResultReturn<String> {
    let auth_granted =
        auth_run_helper(authenticator, move |client| authenticate(client, auth_req)).unwrap();

    debug!("Encoding response... {:?}", auth_granted);
    let resp = encode_msg(&IpcMsg::Resp {
        req_id,
        resp: IpcResp::Auth(Ok(auth_granted)),
    })
    .unwrap();
    debug!("Returning auth response generated: {:?}", resp);

    Ok(resp)
}

// Helper function to generate a containers authorisation response
fn gen_cont_auth_response(
    authenticator: &Authenticator,
    req_id: SafeAuthReqId,
    cont_req: ContainersReq,
) -> ResultReturn<String> {
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
fn gen_unreg_auth_response(req_id: SafeAuthReqId) -> ResultReturn<String> {
    let bootstrap_cfg = safe_core_client::bootstrap_config().unwrap();

    debug!("Encoding response... {:?}", bootstrap_cfg);
    let resp = encode_msg(&IpcMsg::Resp {
        req_id,
        resp: IpcResp::Unregistered(Ok(bootstrap_cfg)),
    })
    .unwrap();

    debug!("Returning unregistered auth response generated: {:?}", resp);
    Ok(resp)
}

// Helper function to generate an authorisation response for sharing MD
fn gen_shared_md_auth_response(
    authenticator: &Authenticator,
    req_id: SafeAuthReqId,
    share_mdata_req: ShareMDataReq,
) -> ResultReturn<String> {
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
