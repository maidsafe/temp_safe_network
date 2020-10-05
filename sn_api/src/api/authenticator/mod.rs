// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.


use crate::{Error, Result, SafeAuthReq, SafeAuthReqId};

use log::{debug, trace, info};
// use safe_authenticator::{
//     access_container, access_container::update_container_perms, app_auth::authenticate, config,
//     errors::AuthError, ipc::decode_ipc_msg,
//     revocation::revoke_app as safe_authenticator_revoke_app, Authenticator,
// };
use rand::rngs::StdRng;
use rand::Rng;
use rand_core::SeedableRng;

use crate::api::ipc::{
    decode_msg, encode_msg,
    req::{AuthReq, IpcReq},
    resp::IpcResp,
    IpcError, IpcMsg,
};
use tiny_keccak::{sha3_256, sha3_512};

use hmac::Hmac;
use serde::{Deserialize, Serialize};
use sha3::Sha3_256;
use sn_client::{
    client as sn_client_client,
    client::Client,
    // core_structs::{access_container_enc_key, AccessContainerEntry},
    utils::symmetric_decrypt,
    ClientError,
};
use sn_data_types::{ClientFullId};
// extern crate ed25519_dalek;

use ed25519_dalek::{Keypair as Ed25519Keypair, SecretKey as Ed25519SecretKey};
// use ed25519_dalek::Signature;


use xor_name::{XorName, XOR_NAME_LEN};

const SHA3_512_HASH_LEN: usize = 64;

/// Derive Password, Keyword and PIN (in order).
pub fn derive_secrets(acc_passphrase: &[u8], acc_password: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let passphrase_hash = sha3_512(acc_passphrase);

    // what is the PIN for here?
    let pin = sha3_512(&passphrase_hash[SHA3_512_HASH_LEN / 2..]).to_vec();
    let keyword = passphrase_hash.to_vec();
    let password = sha3_512(acc_password).to_vec();

    (password, keyword, pin)
}

/// Create a new full id from seed
fn create_full_id_from_seed(seeder: &[u8]) -> ClientFullId {
    let seed = sha3_256(&seeder);
    let mut rng = StdRng::from_seed(seed);
    ClientFullId::new_bls(&mut rng)
}

/// Create a new BLS sk from seed
fn create_bls_sk_from_seed(seeder: &[u8]) -> threshold_crypto::SecretKey {
    let seed = sha3_256(&seeder);
    let mut rng = StdRng::from_seed(seed);

    let bls_secret_key: threshold_crypto::SecretKey = rng.gen();

    bls_secret_key
}

fn create_ed25519_sk_from_seed(seeder: &[u8]) -> Ed25519SecretKey {
    let seed = sha3_256(&seeder);
    let mut rng = StdRng::from_seed(seed);

    let sk= Ed25519Keypair::generate(&mut rng);

    sk.secret
}

pub fn get_sk_from_input(passphrase: &str, password: &str) -> threshold_crypto::SecretKey {

     // TODO: Q what is the need for this third secret?
     let (password, keyword, salt) = derive_secrets(passphrase.as_bytes(), password.as_bytes());

     // TODO properly derive an Map location
     let _map_data_location = generate_network_address(&keyword, &salt);     

     // TODO: use a combo of derived inputs for seed here.
     let mut seed = password.clone();
     seed.extend(salt.iter());
     let sk = create_bls_sk_from_seed(&seed);

     sk
}

// /// use password based crypto
// fn derive_key(output: &mut [u8], input: &[u8], user_salt: &[u8]) {
//     const ITERATIONS: usize = 10000;

//     let salt = sha3_256(user_salt);
//     pbkdf2::pbkdf2::<Hmac<Sha3_256>>(input, &salt, ITERATIONS, output)
// }

// /// Generates User's Identity for the network using supplied credentials in
// /// a deterministic way.  This is similar to the username in various places.
pub fn generate_network_address(keyword: &[u8], pin: &[u8]) -> Result<XorName> {
    let mut id = XorName([0; XOR_NAME_LEN]);

    const ITERATIONS: usize = 10000;

    let _salt = sha3_256(pin);
    pbkdf2::pbkdf2::<Hmac<Sha3_256>>(keyword, &pin, ITERATIONS, &mut id.0[..]);

    // Self::derive_key(&mut id.0[..], keyword, pin);

    Ok(id)
}

/// "Account...."
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Account {
    /// The User Account Keys.
    // pub maid_keys: ClientKeys,
    /// The user's access container.
    // pub access_container: MapInfo,
    /// The user's configuration directory.
    // pub config_root: MapInfo,
    /// Set to `true` when all root and standard containers
    /// have been created successfully. `false` signifies that
    /// previous attempt might have failed - check on login.
    pub root_dirs_created: bool,
}

// Authenticator API
#[derive(Default)]
pub struct SafeAuthenticator {
    authenticator_client: Option<Client>,
}

impl SafeAuthenticator {
    pub fn new(config_dir_path: Option<&str>) -> Self {
        if let Some(path) = config_dir_path {
            sn_client::config_handler::set_config_dir_path(path);
        }

        Self {
            authenticator_client: None,
        }
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
    pub async fn create_acc(&mut self, sk: threshold_crypto::SecretKey) -> Result<()> {
        debug!("Attempting to create a Safe account from provided password and passphrase.");
        // let secret_key = sk_from_hex(sk)?;

        // TODO derive SK from passphrase etc. Put data storage blob on network.

     
        trace!("Creating an account...");

        // TODO: Q what is the need for this third secret?
        // let (password, keyword, salt) = derive_secrets(passphrase.as_bytes(), password.as_bytes());

        // TODO properly derive an Map location
        // let _map_data_location = generate_network_address(&keyword, &salt);     

        // TODO: use a combo of derived inputs for seed here.
        // let mut seed = password.clone();
        // seed.extend(salt.iter());
        // let sk = create_bls_sk_from_seed(&seed);

        let auth_client = Client::new(Some(sk)).await?;

        self.authenticator_client = Some(auth_client);

        // TODO: actually create and put Map data to be used in storage of apps.

        Ok(())
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
    pub async fn log_in(&mut self, sk: threshold_crypto::SecretKey) -> Result<()> {
        debug!("Attempting to log in...");


        // // TODO: Q what is the need for this third secret?
        // let (password, keyword, salt) = derive_secrets(passphrase.as_bytes(), password.as_bytes());

        // // TODO properly derive an Map location
        // let _map_data_location = generate_network_address(&keyword, &salt);


        // // TODO: use a combo of derived inputs for seed here.
        // let mut seed = password.clone();
        // seed.extend(salt.iter());

        // // unimplemented!();
        // let sk = create_bls_sk_from_seed(&seed);


        // let auth_client = Client::new(Some(sk)).await?;
        // self.authenticator_client = Some(auth_client);

        info!("secret key derived successfully, and Safe Client is connected");
        // TODO: retrieve any data needed
        Ok(())
    }

    pub fn log_out(&mut self) -> Result<()> {
        debug!("Dropping logged in session...");
        self.authenticator_client = None;
        Ok(())
    }

    pub fn is_logged_in(&self) -> bool {
        let is_logged_in = self.authenticator_client.is_some();
        debug!("Is logged in? {}", is_logged_in);
        is_logged_in
    }

    pub async fn decode_req(&self, req: &str) -> Result<(SafeAuthReqId, SafeAuthReq)> {
        let _client = &self.authenticator_client;

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

        let ipc_req = decode_ipc_msg(req_msg).await.map_err(|err| {
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

    // TODO: update terminology around apps auth here
    pub async fn revoke_app(&self, y: &String) -> Result<()> {
        unimplemented!()
    }

    pub async fn authorise_app(&self, x: &str) -> Result<String> {
        unimplemented!()
    }

    // /// Authenticate an app request.
    // ///
    // /// First, this function searches for an app info in the access container.
    // /// If the app is found, then the `AuthGranted` struct is returned based on that information.
    // /// If the app is not found in the access container, then it will be authenticated.
    // pub async fn authenticate(
    //     // client: &AuthClient,
    //     auth_req: AuthReq,
    // ) -> Result<AuthGranted, AuthError> {
    //     let app_id = auth_req.app.id.clone();
    //     let permissions = auth_req.containers.clone();
    //     let AuthReq {
    //         app_container,
    //         app_permissions,
    //         ..
    //     } = auth_req;

    //     // let (apps_version, mut apps) = config::list_apps(client).await?;
    //     // check_revocation(client, app_id.clone()).await?;

    //     // let app_state = app_state(&client, &apps, &app_id).await?;

    //     // Determine an app state. If it's revoked we can reuse existing
    //     // keys stored in the config. And if it is authorised, we just
    //     // return the app info from the config.
    //     // let (app, app_state, app_id) = match app_state {
    //     //     AppState::NotAuthenticated => {
    //             let public_id = client.public_id().await;
    //             // Safe to unwrap as the auth client will have a client public id.
    //             let keys = AppKeys::new(unwrap!(public_id.client_public_id()).clone());
    //             let app = AppInfo {
    //                 info: auth_req.app,
    //                 keys,
    //             };
    //             let _ = config::insert_app(
    //                 &client,
    //                 apps,
    //                 config::next_version(apps_version),
    //                 app.clone(),
    //             )
    //             .await?;
    //             // (app, app_state, app_id)
    //     //     }
    //     //     AppState::Authenticated | AppState::Revoked => {
    //     //         let app_entry_name = sha3_256(app_id.as_bytes());
    //     //         if let Some(app) = apps.remove(&app_entry_name) {
    //     //             (app, app_state, app_id)
    //     //         } else {
    //     //             return Err(AuthError::from(
    //     //                 "Logical error - couldn't find a revoked app in config",
    //     //             ));
    //     //         }
    //     //     }
    //     // };

    //     match app_state {
    //         AppState::Authenticated => {
    //             // Return info of the already registered app
    //             authenticated_app(&client, app, app_id, app_container, app_permissions).await
    //         }
    //         AppState::NotAuthenticated | AppState::Revoked => {
    //             // Register a new app or restore a previously registered app
    //             authenticate_new_app(&client, app, app_container, app_permissions, permissions).await
    //         }
    //     }
    // }
}

// Helper to unwrap the Authenticator if it's logged in
// fn get_authenticator_client(authenticator_client: &Option<Authenticator>) -> Result<&SafeAuthenticator> {
//     authenticator_client.as_ref().ok_or_else(|| {
//         Error::AuthError("You need to log in to a SAFE Network account first".to_string())
//     })
// }

// Helper function to generate an app authorisation response
// async fn gen_auth_response(
//     authenticator: &Client,
//     req_id: SafeAuthReqId,
//     auth_req: AuthReq,
// ) -> Result<String> {
//     let auth_granted = authenticate(&authenticator, auth_req)
//         .await
//         .map_err(|err| {
//             Error::AuthenticatorError(format!(
//                 "Failed to authorise application on the network: {}",
//                 err
//             ))
//         })?;

//     debug!("Encoding response with auth credentials auth granted...");
//     let resp = encode_msg(&IpcMsg::Resp {
//         req_id,
//         response: IpcResp::Auth(Ok(auth_granted)),
//     })
//     .map_err(|err| Error::AuthenticatorError(format!("Failed to encode response: {:?}", err)))?;

//     debug!("Returning auth response generated");

//     Ok(resp)
// }

/// Decodes a given encoded IPC message and returns either an `IpcMsg` struct or
/// an error code + description & an encoded `IpcMsg::Resp` in case of an error
#[allow(clippy::type_complexity)]
pub async fn decode_ipc_msg(
    // client: &Client,
    msg: IpcMsg,
) -> Result<Result<IpcMsg>> {
    match msg {
        IpcMsg::Req {
            request: IpcReq::Auth(auth_req),
            req_id,
        } => {
            // Ok status should be returned for all app states (including
            // Revoked and Authenticated).
            Ok(Ok(IpcMsg::Req {
                req_id,
                request: IpcReq::Auth(auth_req),
            }))
        }
        IpcMsg::Req {
            request: IpcReq::Unregistered(extra_data),
            req_id,
        } => Ok(Ok(IpcMsg::Req {
            req_id,
            request: IpcReq::Unregistered(extra_data),
        })),
        // IpcMsg::Req {
        //     request: IpcReq::ShareMap(share_map_req),
        //     req_id,
        // } => Ok(Ok(IpcMsg::Req {
        //     req_id,
        //     request: IpcReq::ShareMap(share_map_req),
        // })),
        // IpcMsg::Req {
        //     request: IpcReq::Containers(cont_req),
        //     req_id,
        // } => {
        //     trace!("Handling IpcReq::Containers({:?})", cont_req);

        //     let app_id = cont_req.app.id.clone();
        //     let c2 = client.clone();

        //     let (_config_version, config) = config::list_apps(client).await?;
        //     let app_state = app_state(&c2, &config, &app_id).await?;
        //     match app_state {
        //         AppState::Authenticated => Ok(Ok(IpcMsg::Req {
        //             req_id,
        //             request: IpcReq::Containers(cont_req),
        //         })),
        //         AppState::Revoked | AppState::NotAuthenticated => {
        //             // App is not authenticated
        //             let error_code = sn_client::ffi::error_codes::ERR_UNKNOWN_APP;
        //             let description = AuthError::from(IpcError::UnknownApp).to_string();

        //             let response = IpcMsg::Resp {
        //                 response: IpcResp::Auth(Err(IpcError::UnknownApp)),
        //                 req_id,
        //             };
        //             let encoded_response = encode_response(&response)?;

        //             Ok(Err((error_code, description, encoded_response)))
        //         }
        //     }
        // }
        IpcMsg::Resp { .. } | IpcMsg::Revoked { .. } | IpcMsg::Err(..) => Err(Error::AuthError(
            "Invalid Authenticator IPC Message".to_string(),
        )),
    }
}

// /// Encode `IpcMsg` into a `CString`, using base32 encoding.
// pub fn encode_response(msg: &IpcMsg) -> Result<CString, IpcError> {
//     let response = ipc::encode_msg(msg)?;
//     Ok(CString::new(response).map_err(StringError::from)?)
// }
