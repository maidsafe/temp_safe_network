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
// use safe_authenticator::{
//     access_container, access_container::update_container_perms, app_auth::authenticate, config,
//     errors::AuthError, ipc::decode_ipc_msg,
//     revocation::revoke_app as safe_authenticator_revoke_app, Authenticator,
// };
use safe_core::{
    client as safe_core_client,
    client::Client,
    core_structs::{access_container_enc_key, AccessContainerEntry},
    ipc::{
        decode_msg, encode_msg,
        req::{AuthReq, ContainersReq, IpcReq, ShareMapReq},
        resp::IpcResp,
        IpcMsg,
    },
    utils::symmetric_decrypt,
    CoreError,
};
use sn_data_types::{ClientFullId, Error as SndError};
use std::collections::HashMap;

// Authenticator API
#[derive(Default)]
pub struct SafeAuthenticator {
    safe_authenticator: Option<Client>,
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

        // aka: Get auth client sk.



        // match Authenticator::login(passphrase, password, || info!("Disconnected from network"))
        //     .await
        // {
        //     Ok(auth) => {
        //         debug!("Logged-in successfully");
        //         self.safe_authenticator = Some(auth);
        //         Ok(())
        //     }
        //     Err(err) => {
        //         let msg = match err {
        //             AuthError::SndError(SndError::NoSuchLoginPacket) => {
        //                 "no SAFE account found with the passphrase provided".to_string()
        //             }
        //             AuthError::CoreError(CoreError::SymmetricDecipherFailure) => {
        //                 "unable to log in with the password provided".to_string()
        //             }
        //             other => other.to_string(),
        //         };
        //         Err(Error::AuthError(format!("Failed to log in: {}", msg)))
        //     }
        // }

        Ok(())
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


        let salt = sha3_256(pin);
        pbkdf2::pbkdf2::<Hmac<Sha3_256>>(keyword, &pin, ITERATIONS, id.0[..])

        // Self::derive_key(&mut id.0[..], keyword, pin);

        Ok(id)
    }

    /// "Account...."
    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
    pub struct Account {
        /// The User Account Keys.
        pub maid_keys: ClientKeys,
        /// The user's access container.
        pub access_container: MapInfo,
        /// The user's configuration directory.
        pub config_root: MapInfo,
        /// Set to `true` when all root and standard containers
        /// have been created successfully. `false` signifies that
        /// previous attempt might have failed - check on login.
        pub root_dirs_created: bool,
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
    pub async fn create_acc(&mut self, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to create a Safe account from provided password and passphrase.");
        // let secret_key = sk_from_hex(sk)?;

        // TODO derive SK from passphrase etc. Put data storage blob on network.


 // This is a Gateway function to the Maidsafe network. This will help create a fresh acc for the
    // user in the SAFE-network.
    // #[allow(clippy::too_many_arguments)]
    // async fn registered_impl<F, R>(
    //     acc_locator: &[u8],
    //     acc_password: &[u8],
    //     client_id: ClientFullId,
    //     net_tx: NetworkTx,
    //     seed: Option<&mut R>,
    //     connection_manager_wrapper_fn: F,
    // ) -> Result<Self, AuthError>
    // where
    //     R: CryptoRng + SeedableRng + Rng,
    //     F: Fn(ConnectionManager) -> ConnectionManager,
    // {
        trace!("Creating an account...");

        // TODO: Q what is the need for this third secret?
        let (password, keyword, salt) = utils::derive_secrets(acc_locator, acc_password);

        // TODO properly derive an Map location
        let map_data_location = generate_network_address(keyword, salt);


        // let acc_locator = Account::generate_network_id(&keyword, &pin)?;
        // let user_cred = UserCred::new(password, pin);
        // let mut maid_keys = match seed {
        //     Some(seed) => ClientKeys::new(seed),
        //     None => ClientKeys::new(&mut thread_rng()),
        // };

        // maid_keys.client_id = client_id;

        // let client_safe_key = maid_keys.client_safe_key();

        // let acc = Account::new(maid_keys)?;
        // let acc_ciphertext = acc.encrypt(&user_cred.password, &user_cred.pin)?;

        // TODO: use a combo of derived inputs for seed here.

        let id_seed = password + salt;
        let full_id = create_full_id_from_seed(&id_seed);

        // let sig = transient_id.sign(&acc_ciphertext);
        let pk = transient_id.public_id().public_key();

        let client = Client::new(Some(full_id.keys))
        // let new_login_packet = LoginPacket::new(acc_locator, *transient_pk, acc_ciphertext, sig)?;

        // TODO:  Connect via the client.... (w/config etc)


        // // Create the connection manager
        // let mut connection_manager =
        //     attempt_bootstrap(&Config::new().quic_p2p, &net_tx, client_safe_key.clone()).await?;

        // connection_manager = connection_manager_wrapper_fn(connection_manager);

        // let response = req(
        //     &mut connection_manager,
        //     Request::LoginPacket(LoginPacketRequest::Create(new_login_packet)),
        //     &client_safe_key,
        // )
        // .await?;

        // TODO: Put the data.
        // match response {
        //     Response::Mutation(res) => res?,
        //     _ => return Err(AuthError::from("Unexpected response")),
        // };

    //     Ok(Self {
    //         inner: Arc::new(Mutex::new(Inner::new(
    //             connection_manager,
    //             Duration::from_secs(180), // FIXME //(REQUEST_TIMEOUT_SECS),
    //             net_tx,
    //         ))),
    //         auth_inner: Arc::new(Mutex::new(AuthInner {
    //             acc,
    //             acc_loc: acc_locator,
    //             user_cred,
    //         })),
    //     })
    // }











        // match Authenticator::create_client_with_acc(
        //     passphrase,
        //     password,
        //     ClientFullId::from(secret_key),
        //     || {
        //         // TODO: allow the caller to provide the callback function
        //         // eprintln!("{}", "Disconnected from network");
        //     },
        // )
        // .await
        // {
        //     Ok(auth) => {
        //         debug!("Account just created successfully");
        //         self.safe_authenticator = Some(auth);
        //         Ok(())
        //     }
        //     Err(err) => {
        //         let msg = match err {
        //             AuthError::SndError(SndError::LoginPacketExists) => {
        //                 "a SAFE account already exists with the passphrase provided".to_string()
        //             }
        //             other => other.to_string(),
        //         };
        //         Err(Error::AuthError(format!(
        //             "Failed to create an account: {}",
        //             msg
        //         )))
        //     }
        // }


        Ok(())
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




