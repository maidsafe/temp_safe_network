// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{
    api::ipc::{
        req::{AuthReq, IpcReq},
        resp::{AuthGranted, IpcResp},
        IpcMsg,
    },
    Error, Result, SafeAuthReq,
};

use hmac::Hmac;
use log::{debug, info, trace};
use rand::rngs::{OsRng, StdRng};
use rand_core::SeedableRng;
use sha3::Sha3_256;
use sn_client::{
    client::{bootstrap_config, Client},
    ClientError,
};
use sn_data_types::{
    Error::NoSuchEntry, Keypair, MapAction, MapAddress, MapEntryActions, MapPermissionSet,
    MapSeqEntryActions, MapValue, Money,
};
use std::{collections::BTreeMap, sync::Arc};
use tiny_keccak::{sha3_256, sha3_512};
use xor_name::{XorName, XOR_NAME_LEN};

const SHA3_512_HASH_LEN: usize = 64;

// Type tag value used for the Map which holds the Safe's content on the network.
const SAFE_TYPE_TAG: u64 = 1_300;

// Number of testcoins (in nano) for any new keypair when simulated-payouts is enabled.
const DEFAULT_TEST_COINS_AMOUNT: u64 = 777_000_000_000;

/// Derive Password, Keyword and PIN (in order).
pub fn derive_secrets(acc_passphrase: &[u8], acc_password: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let passphrase_hash = sha3_512(acc_passphrase);

    // what is the PIN for here?
    let pin = sha3_512(&passphrase_hash[SHA3_512_HASH_LEN / 2..]).to_vec();
    let keyword = passphrase_hash.to_vec();
    let password = sha3_512(acc_password).to_vec();

    (password, keyword, pin)
}

/// Create a new Ed25519 keypair from seed
fn create_ed25519_keypair_from_seed(seeder: &[u8]) -> Keypair {
    let seed = sha3_256(seeder);
    let mut rng = StdRng::from_seed(seed);
    Keypair::new_ed25519(&mut rng)
}

/// Create a new BLS keypair from seed
#[allow(dead_code)]
fn create_bls_keypair_from_seed(seeder: &[u8]) -> Keypair {
    let seed = sha3_256(seeder);
    let mut rng = StdRng::from_seed(seed);

    Keypair::new_bls(&mut rng)
}

/// Perform all derivations and seeding to deterministically obtain location and Keypair from input
pub fn derive_location_and_keypair(
    passphrase: &str,
    password: &str,
) -> Result<(XorName, Arc<Keypair>)> {
    let (password, keyword, salt) = derive_secrets(passphrase.as_bytes(), password.as_bytes());

    let map_data_location = generate_network_address(&keyword, &salt)?;

    let mut seed = password;
    seed.extend(salt.iter());
    let keypair = Arc::new(create_ed25519_keypair_from_seed(&seed));

    Ok((map_data_location, keypair))
}

// /// use password based crypto
// fn derive_key(output: &mut [u8], input: &[u8], user_salt: &[u8]) {
//     const ITERATIONS: usize = 10000;

//     let salt = sha3_256(user_salt);
//     pbkdf2::pbkdf2::<Hmac<Sha3_256>>(input, &salt, ITERATIONS, output)
// }

/// Generates User's Identity for the network using supplied credentials in
/// a deterministic way.  This is similar to the username in various places.
pub fn generate_network_address(keyword: &[u8], pin: &[u8]) -> Result<XorName> {
    let mut id = XorName([0; XOR_NAME_LEN]);

    const ITERATIONS: usize = 10000;

    let _salt = sha3_256(pin);
    pbkdf2::pbkdf2::<Hmac<Sha3_256>>(keyword, &pin, ITERATIONS, &mut id.0[..]);

    // Self::derive_key(&mut id.0[..], keyword, pin);

    Ok(id)
}

// Authenticator API
#[derive(Default)]
pub struct SafeAuthenticator {
    // We keep the client instantiated with the derived keypair, along
    // with the address of the Map which holds its Safe on the network.
    safe: Option<(Client, MapAddress)>,
}

impl SafeAuthenticator {
    pub fn new(config_dir_path: Option<&str>) -> Self {
        if let Some(path) = config_dir_path {
            sn_client::config_handler::set_config_dir_path(path);
        }

        Self { safe: None }
    }

    /// # Create Safe
    /// Creates a new Safe on the Network.
    /// Returns an error if a Safe exists or if there was some
    /// problem during the creation process.
    /// If the Safe is successfully created it keeps the logged in session (discarding a previous session)
    ///
    /// Note: This does _not_ perform any strength checks on the
    /// strings used to create the Safe.
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
    /// let acc_created = safe_auth.create(sk, my_secret, my_password).await;
    /// match acc_created {
    ///    Ok(()) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    /// # });
    ///```
    ///
    /// ## Error Example
    /// If a Safe with same passphrase already exists,
    /// the function will return an error:
    /// ```ignore
    /// use sn_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing Safe's passphrase and password:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # async_std::task::block_on(async {
    /// # safe_auth.create(sk, my_secret, my_password).await.unwrap();
    /// let acc_not_created = safe_auth.create(sk, my_secret, my_password).await;
    /// match acc_not_created {
    ///    Ok(_) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("Failed to create a Safe"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    /// # });
    ///```
    pub async fn create(&mut self, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to create a Safe from provided passphrase and password.");

        let (location, keypair) = derive_location_and_keypair(passphrase, password)?;
        let data_owner = keypair.public_key();

        debug!("Creating Safe to be owned by PublicKey: {:?}", data_owner);

        let mut client = Client::new(Some(keypair), None).await?;
        trace!("Client instantiated properly!");

        // check if client data already exists
        // TODO: Use a more reliable test for existing data...
        let existing_balance = client.get_balance().await?;

        if existing_balance != Money::from_nano(0) {
            return Err(Error::AuthenticatorError(
                "Client data already exists".to_string(),
            ));
        }

        client
            .trigger_simulated_farming_payout(Money::from_nano(DEFAULT_TEST_COINS_AMOUNT))
            .await?;

        // Create Map data to store the list of keypairs generated for
        // each of the user's applications.
        let permission_set = MapPermissionSet::new()
            .allow(MapAction::Read)
            .allow(MapAction::Insert)
            .allow(MapAction::Update)
            .allow(MapAction::Delete)
            .allow(MapAction::ManagePermissions);

        let mut permission_map = BTreeMap::new();
        permission_map.insert(data_owner, permission_set);

        // TODO: encrypt content
        let map_address = client
            .store_seq_map(
                location,
                SAFE_TYPE_TAG,
                data_owner,
                None,
                Some(permission_map),
            )
            .await
            .map_err(|err| {
                Error::AuthenticatorError(format!("Failed to store Safe on a Map: {}", err))
            })?;
        debug!("Map stored successfully for new Safe at: {:?}", map_address);

        self.safe = Some((client, map_address));
        Ok(())
    }

    /// # Unlock
    ///
    /// Unlock a Safe already created on the network using the `Authenticator` daemon.
    ///
    /// ## Example
    /// ```ignore
    /// use sn_api::SafeAuthenticator;
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # fn random_str() -> String { (0..4).map(|_| rand::random::<char>()).collect() }
    /// /// Using an already existing Safe's passphrase and password:
    /// let my_secret = "mysecretstring";
    /// let my_password = "mypassword";
    /// # let my_secret = &(random_str());
    /// # let my_password = &(random_str());
    /// # let sk = "83c055c5efdc483bd967adba5c1769daee0a17bc5fa2b6e129cd6b596c217617";
    /// # async_std::task::block_on(async {
    /// # safe_auth.create(sk, my_secret, my_password).await.unwrap();
    /// let logged_in = safe_auth.unlock(my_secret, my_password).await;
    /// match logged_in {
    ///    Ok(()) => assert!(true), // This should pass
    ///    Err(_) => assert!(false)
    /// }
    /// # });
    ///```
    ///
    /// ## Error Example
    /// If the Safe does not exist, the function will return an appropriate error:
    ///```ignore
    /// use sn_api::{SafeAuthenticator, Error};
    /// let mut safe_auth = SafeAuthenticator::new(None);
    /// # async_std::task::block_on(async {
    /// let not_logged_in = safe_auth.unlock("non", "existant").await;
    /// match not_logged_in {
    ///    Ok(()) => assert!(false), // This should not pass
    ///    Err(Error::AuthError(message)) => {
    ///         assert!(message.contains("Failed to log in"));
    ///    }
    ///    Err(_) => assert!(false), // This should not pass
    /// }
    /// # });
    ///```
    pub async fn unlock(&mut self, passphrase: &str, password: &str) -> Result<()> {
        debug!("Attempting to unlock a Safe...");

        let (location, keypair) = derive_location_and_keypair(passphrase, password)?;

        debug!(
            "Unlocking Safe owned by PublicKey: {:?}",
            keypair.public_key()
        );

        let client = Client::new(Some(keypair), None).await?;
        trace!("Client instantiated properly!");

        let map_address = MapAddress::Seq {
            name: location,
            tag: SAFE_TYPE_TAG,
        };

        // Attempt to retrieve Map to make sure it actually exists
        let _ = client.get_map(map_address).await?;
        debug!("Safe unlocked successfully!");

        self.safe = Some((client, map_address));
        Ok(())
    }

    pub fn lock(&mut self) -> Result<()> {
        debug!("Locking Safe...");
        self.safe = None;
        Ok(())
    }

    pub fn is_a_safe_unlocked(&self) -> bool {
        let is_a_safe_unlocked = self.safe.is_some();
        debug!(
            "Is there a Safe currently unlocked?: {}",
            is_a_safe_unlocked
        );
        is_a_safe_unlocked
    }

    pub async fn decode_req(&self, req: &str) -> Result<SafeAuthReq> {
        match IpcMsg::from_string(req) {
            Ok(IpcMsg::Req(IpcReq::Auth(app_auth_req))) => {
                debug!("Auth request string decoded: {:?}", app_auth_req);
                Ok(SafeAuthReq::Auth(app_auth_req))
            }
            Ok(other) => Err(Error::AuthError(format!(
                "Failed to decode string as an authorisation request, it's a: '{:?}'",
                other
            ))),
            Err(error) => Err(Error::AuthenticatorError(format!(
                "Failed to decode request: {:?}",
                error
            ))),
        }
    }

    // TODO: update terminology around apps auth here
    pub async fn revoke_app(&self, _y: &str) -> Result<()> {
        unimplemented!()
    }

    /// Decode requests and trigger application authorisation against the current client
    pub async fn authorise_app(&self, req: &str) -> Result<String> {
        let ipc_req = IpcMsg::from_string(req).map_err(|err| {
            Error::AuthenticatorError(format!("Failed to decode authorisation request: {:?}", err))
        })?;

        debug!("Auth request string decoded: {:?}", ipc_req);

        match ipc_req {
            IpcMsg::Req(IpcReq::Auth(app_auth_req)) => {
                info!("Request was recognised as a general app auth request");
                debug!("Decoded request: {:?}", app_auth_req);
                self.gen_auth_response(app_auth_req).await
            }
            IpcMsg::Req(IpcReq::Unregistered(user_data)) => {
                info!("Request was recognised as an unregistered auth request");
                debug!("Decoded request: {:?}", user_data);

                self.gen_unreg_auth_response()
            }
            IpcMsg::Resp { .. } | IpcMsg::Err(..) => Err(Error::AuthError(
                "The request was not recognised as a valid auth request".to_string(),
            )),
        }
    }

    /// Authenticate an app request.
    ///
    /// First, this function searches for an app info in the Safe.
    /// If the app is found, then the `AuthGranted` struct is returned based on that information.
    /// If the app is not found in the Safe, then it will be authenticated.
    pub async fn authenticate(&self, auth_req: AuthReq) -> Result<AuthGranted> {
        if let Some((client, map_address)) = &self.safe {
            let app_id = auth_req.app_id.as_bytes().to_vec();
            let keypair = match client.get_map_value(*map_address, app_id.clone()).await {
                Ok(value) => {
                    // This app already has its own keypair

                    // TODO: support for scenario when app was previously revoked,
                    // in which case we should generate a new keypair

                    let keypair_bytes = match value {
                        MapValue::Seq(seq_value) => seq_value.data,
                        MapValue::Unseq(data) => data,
                    };
                    let keypair_str = String::from_utf8(keypair_bytes).map_err(|_err| {
                        Error::AuthError(
                            "The Safe contains an invalid keypair associated to this app"
                                .to_string(),
                        )
                    })?;
                    let keypair: Arc<Keypair> =
                        Arc::new(serde_json::from_str(&keypair_str).map_err(|_err| {
                            Error::AuthError(
                                "The Safe contains an invalid keypair associated to this app"
                                    .to_string(),
                            )
                        })?);

                    debug!(
                        "Keypair for the app being authorised ('{}') retrieved from the Safe: {}",
                        auth_req.app_id,
                        keypair.public_key()
                    );

                    keypair
                }
                Err(ClientError::DataError(NoSuchEntry)) => {
                    // This is the first time this app is being authorised,
                    // thus let's generate a keypair for it
                    let mut rng = OsRng;
                    let keypair = Keypair::new_ed25519(&mut rng);

                    let keypair_str = serde_json::to_string(&keypair).map_err(|err| {
                        Error::AuthError(format!(
                            "Failed to serialised keypair to store it in the Safe: {}",
                            err
                        ))
                    })?;

                    let keypair = Arc::new(keypair);
                    debug!(
                        "New keypair generated for app ('{}') being authorised: {}",
                        auth_req.app_id,
                        keypair.public_key()
                    );

                    // Allocate some test coins
                    // TODO: we may want to allow different options here, either accept
                    // a proof of payment from the requester, transfer from the Safe's balance,
                    // or simply allocate testcoins as it's now.
                    let mut tmp_client = Client::new(Some(keypair.clone()), None).await?;
                    tmp_client
                        .trigger_simulated_farming_payout(Money::from_nano(
                            DEFAULT_TEST_COINS_AMOUNT,
                        ))
                        .await?;

                    // Store the keypair in the Safe, mapped to the app id
                    let map_actions =
                        MapSeqEntryActions::new().ins(app_id, keypair_str.as_bytes().to_vec(), 0);

                    client
                        .edit_map_entries(*map_address, MapEntryActions::Seq(map_actions))
                        .await?;

                    keypair
                }
                Err(err) => {
                    return Err(Error::AuthError(format!(
                        "Failed to retrieve keypair from the Safe: {}",
                        err
                    )))
                }
            };

            Ok(AuthGranted {
                app_keypair: keypair,
                bootstrap_config: bootstrap_config()?,
            })
        } else {
            Err(Error::AuthenticatorError(
                "No Safe is currently unlocked".to_string(),
            ))
        }
    }

    // Helper function to generate an app authorisation response
    async fn gen_auth_response(&self, auth_req: AuthReq) -> Result<String> {
        let auth_granted = self.authenticate(auth_req).await.map_err(|err| {
            Error::AuthenticatorError(format!(
                "Failed to authorise application on the network: {}",
                err
            ))
        })?;

        debug!("Encoding response with auth credentials auth granted...");
        let resp = serde_json::to_string(&IpcMsg::Resp(IpcResp::Auth(Ok(auth_granted)))).map_err(
            |err| Error::AuthenticatorError(format!("Failed to encode response: {:?}", err)),
        )?;

        debug!("Returning auth response generated");

        Ok(resp)
    }

    // Helper function to generate an unregistered authorisation response
    fn gen_unreg_auth_response(&self) -> Result<String> {
        let bootstrap_cfg = bootstrap_config().map_err(|err| {
            Error::AuthenticatorError(format!(
                "Failed to obtain bootstrap info for response: {}",
                err
            ))
        })?;

        debug!("Encoding response... {:?}", bootstrap_cfg);
        let resp = serde_json::to_string(&IpcMsg::Resp(IpcResp::Unregistered(Ok(bootstrap_cfg))))
            .map_err(|err| {
            Error::AuthenticatorError(format!("Failed to encode response: {:?}", err))
        })?;

        debug!("Returning unregistered auth response generated: {:?}", resp);
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use sn_data_types::PublicKey;

    #[test]
    fn get_deterministic_pk_from_known_seed() -> Result<()> {
        let seed = b"bacon";
        let pk = create_ed25519_keypair_from_seed(seed).public_key();

        let public_key_bytes: [u8; ed25519_dalek::PUBLIC_KEY_LENGTH] = [
            239, 124, 31, 157, 76, 101, 124, 119, 164, 143, 80, 234, 249, 84, 0, 22, 91, 128, 67,
            92, 39, 182, 197, 184, 83, 44, 41, 127, 78, 175, 205, 198,
        ];

        let ed_pk = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)
            .map_err(|_| Error::Unexpected("Cannot deserialise expected key".to_string()))?;
        let expected_pk = PublicKey::from(ed_pk);

        assert_eq!(pk, expected_pk);

        Ok(())
    }

    proptest! {
        #[test]
        fn proptest_always_get_same_info_from_from_phrase_and_pw(s in "\\PC*", p in "\\PC*") {
            let (location, keypair) = derive_location_and_keypair(&s, &p).expect("could not derive location/keypair");
            let (location_again, keypair_again) = derive_location_and_keypair(&s, &p).expect("could not derive location/keypair");
            prop_assert_eq!(location, location_again);
            prop_assert_eq!(keypair, keypair_again);
        }
    }
}
