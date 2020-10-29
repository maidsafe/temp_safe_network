// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    common::ed_sk_from_hex,
    helpers::{parse_coins_amount, pk_to_hex},
    xorurl::XorUrlEncoder,
    xorurl::{SafeContentType, SafeDataType},
    Safe,
};
use crate::{Error, Result};
use rand::rngs::OsRng;
use sn_client::Client;
use sn_data_types::{Keypair, SecretKey};
use std::sync::Arc;
use xor_name::XorName;

impl Safe {
    // Generate a key pair without creating and/or storing a SafeKey on the network
    pub fn generate_random_ed_keypair(&self) -> Result<Keypair> {
        let mut rng = OsRng;
        let keypair = Keypair::new_ed25519(&mut rng);
        Ok(keypair)
    }

    // Create a SafeKey on the network, preloaded from another key and return its XOR-URL.
    pub async fn keys_create_and_preload_from_sk_string(
        &mut self,
        from: &str,
        preload_amount: Option<&str>,
    ) -> Result<(String, Keypair)> {
        let from_sk = match ed_sk_from_hex(&from) {
            Ok(sk) => sk,
            Err(_) => return Err(Error::InvalidInput(
                "The source of funds needs to be a secret key. The secret key provided is invalid"
                    .to_string(),
            )),
        };

        let keypair = Keypair::from(from_sk);
        let amount = parse_coins_amount(&preload_amount.unwrap_or_else(|| "0.0"))?;

        let (xorname, keypair) = {
            let our_new_keypair = Keypair::new_ed25519(&mut OsRng);

            let mut paying_client = Client::new(Some(keypair)).await?;
            paying_client
                .send_money(our_new_keypair.public_key(), amount)
                .await?;
            (XorName::from(our_new_keypair.public_key()), our_new_keypair)
        };

        let xorurl = XorUrlEncoder::encode_safekey(xorname, self.xorurl_base)?;
        Ok((xorurl, keypair))
    }

    #[cfg(feature = "simulated-payouts")]
    // Create a SafeKey on the network, allocates testcoins onto it, and return the SafeKey's XOR-URL
    pub async fn keys_create_preload_test_coins(
        &mut self,
        preload_amount: &str,
    ) -> Result<(String, Keypair)> {
        let amount = parse_coins_amount(preload_amount)?;
        let mut rng = OsRng;
        let keypair = Keypair::new_ed25519(&mut rng);
        self.safe_client
            .trigger_simulated_farming_payout(amount)
            .await?;
        let xorname = XorName::from(keypair.public_key());

        let xorurl = XorUrlEncoder::encode_safekey(xorname, self.xorurl_base)?;
        Ok((xorurl, keypair))
    }

    // Check SafeKey's balance from the network from a given SecretKey string
    pub async fn keys_balance_from_sk(&self, secret_key: Arc<SecretKey>) -> Result<String> {
        let secret_key = Arc::as_ref(&secret_key);

        let keypair = match secret_key {
            SecretKey::Bls(sk) => Keypair::from(sk),
            SecretKey::Ed25519(sk) => {
                let bytes = sk.to_bytes();
                let secret_key = ed25519_dalek::SecretKey::from_bytes(&bytes)
                    .map_err(|_| Error::Unexpected("Error parsing SecretKey bytes".to_string()))?;
                Keypair::from(secret_key)
            }
            _ => {
                return Err(Error::Unexpected(
                    "Cannot convert from BlsShare to a FullId".to_string(),
                ))
            }
        };

        let mut temp_client = Client::new(Some(keypair)).await?;
        let balance = temp_client.get_balance().await?;

        Ok(balance.to_string())
    }

    // Check SafeKey's balance from the network from a given XOR/NRS-URL and secret key string.
    // The difference between this and 'keys_balance_from_sk' function is that this will additionally
    // check that the XOR/NRS-URL corresponds to the public key derived from the provided secret key
    pub async fn keys_balance_from_url(
        &mut self,
        url: &str,
        secret_key: Arc<SecretKey>,
    ) -> Result<String> {
        self.validate_sk_for_url(secret_key.clone(), url).await?;
        self.keys_balance_from_sk(secret_key).await
    }

    // Check that the XOR/NRS-URL corresponds to the public key derived from the provided client id
    pub async fn validate_sk_for_url(
        &mut self,
        secret_key: Arc<SecretKey>,
        url: &str,
    ) -> Result<String> {
        let secret_key = Arc::as_ref(&secret_key);
        let keypair = match secret_key {
            SecretKey::Bls(sk) => Keypair::from(sk),
            SecretKey::Ed25519(sk) => {
                let bytes = sk.to_bytes();
                let secret_key = ed25519_dalek::SecretKey::from_bytes(&bytes)
                    .map_err(|_| Error::Unexpected("Error parsing SecretKey bytes".to_string()))?;
                Keypair::from(secret_key)
            }
            _ => {
                return Err(Error::InvalidInput(
                    "Cannot form a keypair from a BlsKeyShare at this time.".to_string(),
                ))
            }
        };

        let (xorurl_encoder, _) = self.parse_and_resolve_url(url).await?;
        let public_key = keypair.public_key();
        let derived_xorname = XorName::from(public_key);
        if xorurl_encoder.xorname() != derived_xorname {
            Err(Error::InvalidInput(
                "The URL doesn't correspond to the public key derived from the provided secret key"
                    .to_string(),
            ))
        } else {
            Ok(pk_to_hex(&public_key))
        }
    }

    /// # Transfer safecoins from one SafeKey to another, or to a Wallet
    ///
    /// Using a secret key you can send safecoins to a Wallet or to a SafeKey.
    ///
    /// ## Example
    /// ```
    /// # use sn_api::Safe;
    /// let mut safe = Safe::default();
    /// # async_std::task::block_on(async {
    /// #   safe.connect("", Some("fake-credentials")).await.unwrap();
    ///     let (key1_xorurl, keypair1) = safe.keys_create_preload_test_coins("14").await.unwrap();
    ///     let (key2_xorurl, keypair2) = safe.keys_create_preload_test_coins("1").await.unwrap();
    ///     let current_balance = safe.keys_balance_from_sk(keypair1.clone().unwrap().sk).await.unwrap();
    ///     assert_eq!("14.000000000", current_balance);
    ///
    ///     safe.keys_transfer( "10", Some(&keypair1.clone().unwrap().sk), &key2_xorurl, None ).await.unwrap();
    ///     let from_balance = safe.keys_balance_from_url( &key1_xorurl, &keypair1.unwrap().sk ).await.unwrap();
    ///     assert_eq!("4.000000000", from_balance);
    ///     let to_balance = safe.keys_balance_from_url( &key2_xorurl, &keypair2.unwrap().sk ).await.unwrap();
    ///     assert_eq!("11.000000000", to_balance);
    /// # });
    /// ```
    pub async fn keys_transfer(
        &mut self,
        amount: &str,
        from_sk_str: Option<&str>,
        to_url: &str,
    ) -> Result<()> {
        // Parse and validate the amount is a valid
        let amount_coins = parse_coins_amount(amount)?;

        // Let's check if the 'to_url' is a valid Wallet or a SafeKey URL
        let (to_xorurl_encoder, _) = self.parse_and_resolve_url(to_url).await?;
        let to_xorname = if to_xorurl_encoder.content_type() == SafeContentType::Wallet {
            let (to_balance, _) = self
                .wallet_get_default_balance(&to_xorurl_encoder.to_string())
                .await?;

            XorUrlEncoder::from_url(&to_balance.xorurl)?.xorname()
        } else if to_xorurl_encoder.content_type() == SafeContentType::Raw
            && to_xorurl_encoder.data_type() == SafeDataType::SafeKey
        {
            // TODO, retrieve the key
            to_xorurl_encoder.xorname()
        } else {
            return Err(Error::InvalidInput(format!(
                "The destination URL doesn't target a SafeKey or Wallet, target is: {:?} ({})",
                to_xorurl_encoder.content_type(),
                to_xorurl_encoder.data_type()
            )));
        };

        let from = match &from_sk_str {
            Some(sk) => Some(Keypair::from(ed_sk_from_hex(sk)?)),
            None => None,
        };

        // Finally, let's make the transfer
        match self
            .safe_client
            .safecoin_transfer_to_xorname(from, to_xorname, amount_coins)
            .await
        {
            Err(Error::InvalidAmount(_)) => Err(Error::InvalidAmount(format!(
                "The amount '{}' specified for the transfer is invalid",
                amount
            ))),
            Err(Error::NotEnoughBalance(_)) => {
                let msg = if from_sk_str.is_some() {
                    "Not enough balance for the transfer at provided source SafeKey".to_string()
                } else {
                    "Not enough balance for the transfer at Account's default SafeKey".to_string()
                };

                Err(Error::NotEnoughBalance(msg))
            }
            Err(other_error) => Err(Error::Unexpected(format!(
                "Unexpected error when attempting to transfer: {}",
                other_error
            ))),
            // TODO: return transfer id...?
            Ok(_tx) => Ok(()),
            // Ok(tx) => Ok(tx.id),
        }
    }
}

#[cfg(all(test, feature = "simulated-payouts"))]
mod tests {
    use super::*;
    use crate::api::app::test_helpers::{new_safe_instance, random_nrs_name};

    #[tokio::test]
    async fn test_keys_create_preload_test_coins() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_xorurl, _keypair) = safe.keys_create_preload_test_coins("12.23").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_create_and_preload_from_sk_string() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_xorurl, from_keypair) = safe.keys_create_preload_test_coins("543.2312").await?;

        let preload_amount = "1.800000000";
        let (_xorurl, keypair) = safe
            .keys_create_and_preload_from_sk_string(
                &from_keypair.secret_key()?.to_string(),
                Some(preload_amount),
            )
            .await?;
        let balance = safe
            .keys_balance_from_sk(Arc::new(keypair.secret_key()?))
            .await?;
        assert_eq!(balance, preload_amount);
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_create_preload_invalid_amounts() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        match safe.keys_create_preload_test_coins(".45").await {
            Err(err) => assert_eq!(
                err,
                Error::InvalidAmount(
                    "Invalid safecoins amount '.45' (Can\'t parse coin units)".to_string()
                )
            ),
            Ok(_) => {
                return Err(Error::Unexpected(
                    "Key with test-coins was created unexpectedly".to_string(),
                ))
            }
        };

        let (_xorurl, keypair) = safe.keys_create_preload_test_coins("12").await?;
        match safe
            .keys_create_and_preload_from_sk_string(
                &keypair.secret_key()?.to_string(),
                Some(".003"),
            )
            .await
        {
            Err(err) => assert_eq!(
                err,
                Error::InvalidAmount(
                    "Invalid safecoins amount '.003' (Can\'t parse coin units)".to_string()
                )
            ),
            Ok(_) => {
                return Err(Error::Unexpected(
                    "Key was created unexpectedly".to_string(),
                ))
            }
        };

        // test it fails with corrupted secret key
        let mut sk = keypair.secret_key()?.to_string();
        sk.replace_range(..6, "ababab");
        match safe
            .keys_create_and_preload_from_sk_string(&sk, Some(".003"))
            .await
        {
            Err(err) => assert_eq!(
                err,
                Error::InvalidAmount(
                    "Invalid safecoins amount '.003' (Can\'t parse coin units)".to_string()
                )
            ),
            Ok(_) => {
                return Err(Error::Unexpected(
                    "Key was created unexpectedly".to_string(),
                ))
            }
        };

        // test it fails to preload with more than available balance in source (which has only 12 coins)
        match safe
            .keys_create_and_preload_from_sk_string(&sk, Some("12.000000001"))
            .await
        {
            Err(err) => {
                assert_eq!(
                    err,
                    Error::NotEnoughBalance(
                        "Not enough balance at 'source' for the operation".to_string()
                    )
                );
                Ok(())
            }
            Ok(_) => Err(Error::Unexpected(
                "Key was created unexpectedly".to_string(),
            )),
        }
    }

    #[tokio::test]
    async fn test_keys_create_pk() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_xorurl, from_keypair) = safe.keys_create_preload_test_coins("1.1").await?;
        let (_xorurl, _keypair) = safe
            .keys_create_and_preload_from_sk_string(&from_keypair.secret_key()?.to_string(), None)
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_pk() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let preload_amount = "1.154200000";
        let (_xorurl, keypair) = safe.keys_create_preload_test_coins(preload_amount).await?;
        let current_balance = safe
            .keys_balance_from_sk(Arc::new(keypair.secret_key()?))
            .await?;
        assert_eq!(preload_amount, current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_xorurl() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let preload_amount = "0.243000000";
        let (xorurl, keypair) = safe.keys_create_preload_test_coins(preload_amount).await?;
        let current_balance = safe
            .keys_balance_from_url(&xorurl, Arc::new(keypair.secret_key()?))
            .await?;
        assert_eq!(preload_amount, current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_wrong_url() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_xorurl, keypair) = safe.keys_create_preload_test_coins("0").await?;

        let invalid_xorurl = "safe://this-is-not-a-valid-xor-url";
        let current_balance = safe
            .keys_balance_from_url(&invalid_xorurl, Arc::new(keypair.secret_key()?))
            .await;
        match current_balance {
            Err(Error::ContentNotFound(msg)) => {
                assert!(msg.contains(&format!("Content not found at {}", invalid_xorurl)));
                Ok(())
            }
            Err(err) => Err(Error::Unexpected(format!(
                "Error returned is not the expected: {:?}",
                err
            ))),
            Ok(balance) => Err(Error::Unexpected(format!(
                "Unexpected balance obtained: {:?}",
                balance
            ))),
        }
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_wrong_location() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let amount = "35312.000000000";
        let (xorurl, keypair) = safe.keys_create_preload_test_coins(amount).await?;

        let current_balance = safe
            .keys_balance_from_url(&xorurl, Arc::new(keypair.secret_key()?))
            .await?;
        assert_eq!(amount, current_balance);

        // let's use the XOR-URL of another SafeKey
        let (other_kp_xorurl, _) = safe.keys_create_preload_test_coins("0").await?;
        let current_balance = safe
            .keys_balance_from_url(&other_kp_xorurl, Arc::new(keypair.secret_key()?))
            .await;
        match current_balance {
            Err(Error::InvalidInput(msg)) => {
                assert!(msg.contains(
                "The URL doesn't correspond to the public key derived from the provided secret key"
            ));
                Ok(())
            }
            Err(err) => Err(Error::Unexpected(format!(
                "Error returned is not the expected: {:?}",
                err
            ))),
            Ok(balance) => Err(Error::Unexpected(format!(
                "Unexpected balance obtained: {:?}",
                balance
            ))),
        }
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_wrong_sk() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_xorurl, _kp) = safe.keys_create_preload_test_coins("0").await?;
        let bls_sk = threshold_crypto::SecretKey::random();
        let sk = SecretKey::Bls(threshold_crypto::serde_impl::SerdeSecret(bls_sk));
        let current_balance = safe.keys_balance_from_sk(Arc::new(sk)).await;
        match current_balance {
            Err(Error::ContentNotFound(msg)) => {
                assert!(msg.contains("No SafeKey found at specified location"));
                Ok(())
            }
            Err(err) => Err(Error::Unexpected(format!(
                "Error returned is not the expected: {:?}",
                err
            ))),
            Ok(balance) => Err(Error::Unexpected(format!(
                "Unexpected balance obtained: {:?}",
                balance
            ))),
        }
    }

    #[tokio::test]
    async fn test_keys_balance_pk() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let preload_amount = "1743.234";
        let (_xorurl, from_keypair) = safe.keys_create_preload_test_coins(preload_amount).await?;

        let amount = "1740.000000000";
        let (_xorurl, to_keypair) = safe
            .keys_create_and_preload_from_sk_string(
                &from_keypair.secret_key()?.to_string(),
                Some(amount),
            )
            .await?;

        let from_current_balance = safe
            .keys_balance_from_sk(Arc::new(from_keypair.secret_key()?))
            .await?;
        assert_eq!(
            "3.233999999", /*== 1743.234 - 1740 - 0.000000001 (creation cost) */
            from_current_balance
        );

        let to_current_balance = safe
            .keys_balance_from_sk(Arc::new(to_keypair.secret_key()?))
            .await?;
        assert_eq!(amount, to_current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_balance_xorname() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let preload_amount = "435.34";
        let (from_xorname, from_keypair) =
            safe.keys_create_preload_test_coins(preload_amount).await?;

        let amount = "35.300000000";
        let (to_xorname, to_keypair) = safe
            .keys_create_and_preload_from_sk_string(
                &from_keypair.secret_key()?.to_string(),
                Some(amount),
            )
            .await?;

        let from_current_balance = safe
            .keys_balance_from_url(&from_xorname, Arc::new(from_keypair.secret_key()?))
            .await?;
        assert_eq!(
            "400.039999999", /*== 435.34 - 35.3 - 0.000000001 (creation cost)*/
            from_current_balance
        );

        let to_current_balance = safe
            .keys_balance_from_url(&to_xorname, Arc::new(to_keypair.secret_key()?))
            .await?;
        assert_eq!(amount, to_current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_validate_sk_for_url() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, keypair) = safe.keys_create_preload_test_coins("23.22").await?;
        let pk = safe
            .validate_sk_for_url(Arc::new(keypair.secret_key()?), &xorurl)
            .await?;
        assert_eq!(pk, keypair.public_key().to_string());
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_transfer_from_zero_balance() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_from_safekey_xorurl, keypair1) = safe.keys_create_preload_test_coins("0.0").await?;
        let (to_safekey_xorurl, _keypair2) = safe.keys_create_preload_test_coins("0.5").await?;

        // test it fails to transfer with 0 balance at SafeKey in <from> argument
        match safe
            .keys_transfer(
                "0",
                Some(&keypair1.secret_key()?.to_string()),
                &to_safekey_xorurl,
            )
            .await
        {
            Err(Error::InvalidAmount(msg)) => {
                assert_eq!(
                    msg,
                    "The amount '0' specified for the transfer is invalid".to_string()
                );
                Ok(())
            }
            Err(err) => Err(Error::Unexpected(format!(
                "Error returned is not the expected: {:?}",
                err
            ))),
            Ok(_) => Err(Error::Unexpected(
                "Transfer succeeded unexpectedly".to_string(),
            )),
        }
    }

    #[tokio::test]
    async fn test_keys_transfer_diff_amounts() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (safekey1_xorurl, keypair1) = safe.keys_create_preload_test_coins("0.5").await?;
        let (safekey2_xorurl, keypair2) = safe.keys_create_preload_test_coins("100.5").await?;

        // test it fails to transfer more than current balance at SafeKey in <from> argument
        match safe
            .keys_transfer(
                "100.6",
                Some(&keypair1.secret_key()?.to_string()),
                &safekey2_xorurl,
            )
            .await
        {
            Err(Error::NotEnoughBalance(msg)) => assert_eq!(
                msg,
                "Not enough balance for the transfer at provided source SafeKey".to_string()
            ),
            Err(err) => {
                return Err(Error::Unexpected(format!(
                    "Error returned is not the expected: {:?}",
                    err
                )))
            }
            Ok(_) => {
                return Err(Error::Unexpected(
                    "Transfer succeeded unexpectedly".to_string(),
                ))
            }
        };

        // test it fails to transfer as it's a invalid/non-numeric amount
        match safe
            .keys_transfer(
                ".06",
                Some(&keypair1.secret_key()?.to_string()),
                &safekey2_xorurl,
            )
            .await
        {
            Err(Error::InvalidAmount(msg)) => assert_eq!(
                msg,
                "Invalid safecoins amount '.06' (Can\'t parse coin units)"
            ),
            Err(err) => {
                return Err(Error::Unexpected(format!(
                    "Error returned is not the expected: {:?}",
                    err
                )))
            }
            Ok(_) => {
                return Err(Error::Unexpected(
                    "Transfer succeeded unexpectedly".to_string(),
                ))
            }
        };

        // test it fails to transfer less than 1 nano coin
        match safe.keys_transfer(
                "0.0000000009",
                Some(&keypair2.secret_key()?.to_string()),
                &safekey2_xorurl,
            ).await {
                    Err(Error::InvalidAmount(msg)) => assert_eq!(msg, "Invalid safecoins amount '0.0000000009', the minimum possible amount is one nano coin (0.000000001)"),
                    Err(err) => return Err(Error::Unexpected(format!("Error returned is not the expected: {:?}", err))),
                    Ok(_) => return Err(Error::Unexpected("Transfer succeeded unexpectedly".to_string())),
            };

        // test successful transfer
        match safe
            .keys_transfer(
                "100.4",
                Some(&keypair2.secret_key()?.to_string()),
                &safekey1_xorurl,
            )
            .await
        {
            Err(msg) => Err(Error::Unexpected(format!(
                "Transfer was expected to succeed: {}",
                msg
            ))),
            Ok(_) => {
                let from_current_balance = safe
                    .keys_balance_from_sk(Arc::new(keypair2.secret_key()?))
                    .await?;
                assert_eq!("0.100000000", from_current_balance);
                let to_current_balance = safe
                    .keys_balance_from_sk(Arc::new(keypair1.secret_key()?))
                    .await?;
                assert_eq!("100.900000000", to_current_balance);
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_keys_transfer_to_wallet() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let to_wallet_xorurl = safe.wallet_create().await?;
        let (_, keypair1) = safe.keys_create_preload_test_coins("10.0").await?;
        safe.wallet_insert(
            &to_wallet_xorurl,
            Some("my-first-balance"),
            true, // set --default
            &keypair1.secret_key()?.to_string(),
        )
        .await?;

        let (_safekey_xorurl, keypair2) = safe.keys_create_preload_test_coins("4621.45").await?;

        // test successful transfer
        match safe
            .keys_transfer(
                "523.87",
                Some(&keypair2.secret_key()?.to_string()),
                &to_wallet_xorurl.clone(),
            )
            .await
        {
            Err(msg) => Err(Error::Unexpected(format!(
                "Transfer was expected to succeed: {}",
                msg
            ))),
            Ok(_) => {
                let from_current_balance = safe
                    .keys_balance_from_sk(Arc::new(keypair2.secret_key()?))
                    .await?;
                assert_eq!(
                    "4097.580000000", /* 4621.45 - 523.87 */
                    from_current_balance
                );
                let wallet_current_balance = safe.wallet_balance(&to_wallet_xorurl).await?;
                assert_eq!("533.870000000", wallet_current_balance);
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_keys_transfer_to_nrs_urls() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_from_safekey_xorurl, keypair1) = safe.keys_create_preload_test_coins("0.2").await?;
        let (to_safekey_xorurl, keypair2) = safe.keys_create_preload_test_coins("0.1").await?;

        let to_nrsurl = random_nrs_name();
        let _ = safe
            .nrs_map_container_create(&to_nrsurl, &to_safekey_xorurl, false, true, false)
            .await?;

        // test successful transfer
        match safe
            .keys_transfer("0.2", Some(&keypair1.secret_key()?.to_string()), &to_nrsurl)
            .await
        {
            Err(msg) => Err(Error::Unexpected(format!(
                "Transfer was expected to succeed: {}",
                msg
            ))),
            Ok(_) => {
                let from_current_balance = safe
                    .keys_balance_from_sk(Arc::new(keypair1.secret_key()?))
                    .await?;
                assert_eq!("0.000000000" /* 0.2 - 0.2 */, from_current_balance);
                let to_current_balance = safe
                    .keys_balance_from_sk(Arc::new(keypair2.secret_key()?))
                    .await?;
                assert_eq!("0.300000000" /* 0.1 + 0.2 */, to_current_balance);
                Ok(())
            }
        }
    }
}
