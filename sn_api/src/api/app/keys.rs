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
    helpers::{parse_coins_amount, pk_from_hex, pk_to_hex},
    xorurl::XorUrlEncoder,
    xorurl::{SafeContentType, SafeDataType},
    Safe,
};
use crate::{Error, Result};
use rand::rngs::OsRng;
use sn_data_types::{Keypair, SecretKey};
use xor_name::XorName;

impl Safe {
    // Generate a key pair without creating and/or storing a SafeKey on the network
    pub fn generate_random_ed_keypair(&self) -> Keypair {
        let mut rng = OsRng;
        Keypair::new_ed25519(&mut rng)
    }

    // Create a SafeKey on the network, allocates token from current client's key onto it,
    // and return the SafeKey's XOR-URL
    pub async fn keys_create_and_preload(
        &mut self,
        preload_amount: &str,
    ) -> Result<(String, Keypair)> {
        let amount = parse_coins_amount(preload_amount)?;
        let new_keypair = self.generate_random_ed_keypair();

        // let's make the transfer
        let _ = self
            .safe_client
            .safecoin_transfer_to_pk(None, new_keypair.public_key(), amount)
            .await?;

        let xorname = XorName::from(new_keypair.public_key());
        let xorurl = XorUrlEncoder::encode_safekey(xorname, self.xorurl_base)?;

        Ok((xorurl, new_keypair))
    }

    // Create a SafeKey on the network, preloaded from another key and return its XOR-URL.
    pub async fn keys_create_and_preload_from_sk_string(
        &mut self,
        from: &str,
        preload_amount: &str,
    ) -> Result<(String, Keypair)> {
        let from_sk = match ed_sk_from_hex(&from) {
            Ok(sk) => sk,
            Err(_) => return Err(Error::InvalidInput(
                "The source of funds needs to be an Ed25519 secret key. The secret key provided is invalid"
                    .to_string(),
            )),
        };

        let from_keypair = Keypair::from(from_sk);
        let amount = parse_coins_amount(&preload_amount)?;
        let new_keypair = self.generate_random_ed_keypair();

        // let's make the transfer
        let _ = self
            .safe_client
            .safecoin_transfer_to_pk(Some(from_keypair), new_keypair.public_key(), amount)
            .await?;

        let xorname = XorName::from(new_keypair.public_key());
        let xorurl = XorUrlEncoder::encode_safekey(xorname, self.xorurl_base)?;

        Ok((xorurl, new_keypair))
    }

    #[cfg(feature = "simulated-payouts")]
    // Create a SafeKey on the network, allocates testcoins onto it, and return the SafeKey's XOR-URL
    pub async fn keys_create_preload_test_coins(
        &mut self,
        preload_amount: &str,
    ) -> Result<(String, Keypair)> {
        let amount = parse_coins_amount(preload_amount)?;
        let keypair = self.generate_random_ed_keypair();
        self.safe_client
            .trigger_simulated_farming_payout(amount, Some(keypair.clone()))
            .await?;

        let xorname = XorName::from(keypair.public_key());
        let xorurl = XorUrlEncoder::encode_safekey(xorname, self.xorurl_base)?;

        Ok((xorurl, keypair))
    }

    // Check SafeKey's balance from the network from a given SecretKey string
    pub async fn keys_balance_from_sk(&self, secret_key: SecretKey) -> Result<String> {
        let keypair = match secret_key {
            SecretKey::Ed25519(sk) => {
                let bytes = sk.to_bytes();
                let secret_key = ed25519_dalek::SecretKey::from_bytes(&bytes).map_err(|err| {
                    Error::InvalidInput(format!("Error parsing SecretKey bytes: {}", err))
                })?;
                Keypair::from(secret_key)
            }
            SecretKey::BlsShare(_) => {
                return Err(Error::InvalidInput(
                    "Cannot convert from BlsShare key to a Keypair".to_string(),
                ))
            }
        };

        let balance = self.safe_client.read_balance_from_keypair(keypair).await?;

        Ok(balance.to_string())
    }

    // Check SafeKey's balance from the network from a given XOR/NRS-URL and secret key string.
    // The difference between this and 'keys_balance_from_sk' function is that this will additionally
    // check that the XOR/NRS-URL corresponds to the public key derived from the provided secret key
    pub async fn keys_balance_from_url(
        &mut self,
        url: &str,
        secret_key: SecretKey,
    ) -> Result<String> {
        self.validate_sk_for_url(&secret_key, url).await?;
        self.keys_balance_from_sk(secret_key).await
    }

    // Check that the XOR/NRS-URL corresponds to the public key derived from the provided client id
    pub async fn validate_sk_for_url(
        &mut self,
        secret_key: &SecretKey,
        url: &str,
    ) -> Result<String> {
        let keypair = match secret_key {
            SecretKey::Ed25519(sk) => {
                let bytes = sk.to_bytes();
                let secret_key = ed25519_dalek::SecretKey::from_bytes(&bytes).map_err(|err| {
                    Error::InvalidInput(format!("Error parsing SecretKey bytes: {}", err))
                })?;
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

    /// # Transfer safecoins from one SafeKey to another, to a Wallet, or to a PublicKey
    ///
    /// Using a secret key you can send safecoins to a SafeKey, Wallet, or PublicKey.
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
        to: &str,
    ) -> Result<u64> {
        // Parse and validate the amount is a valid
        let amount_coins = parse_coins_amount(amount)?;

        let from = match &from_sk_str {
            Some(sk) => Some(Keypair::from(ed_sk_from_hex(sk)?)),
            None => None,
        };

        let result = if to.starts_with("safe://") {
            // Let's check if the 'to' is a valid Wallet or a SafeKey URL
            let (to_xorurl_encoder, _) = self.parse_and_resolve_url(to).await.map_err(|_| {
                Error::InvalidInput(format!("Failed to parse the 'to' URL: {}", to))
            })?;

            let to_xorname = if to_xorurl_encoder.content_type() == SafeContentType::Wallet {
                let (to_balance, _) = self
                    .wallet_get_default_balance(&to_xorurl_encoder.to_string())
                    .await?;

                XorUrlEncoder::from_url(&to_balance.xorurl)?.xorname()
            } else if to_xorurl_encoder.content_type() == SafeContentType::Raw
                && to_xorurl_encoder.data_type() == SafeDataType::SafeKey
            {
                to_xorurl_encoder.xorname()
            } else {
                return Err(Error::InvalidInput(format!(
                    "The destination URL doesn't target a SafeKey or Wallet, target is: {:?} ({})",
                    to_xorurl_encoder.content_type(),
                    to_xorurl_encoder.data_type()
                )));
            };

            // Finally, let's make the transfer
            self.safe_client
                .safecoin_transfer_to_xorname(from, to_xorname, amount_coins)
                .await
        } else {
            // ...let's assume the 'to' is a PublicKey then
            let to_pk = pk_from_hex(to)?;

            // and let's make the transfer
            self.safe_client
                .safecoin_transfer_to_pk(from, to_pk, amount_coins)
                .await
        };

        match result {
            Err(Error::NotEnoughBalance(_)) => {
                let msg = if from_sk_str.is_some() {
                    "Not enough balance for the transfer at provided source SafeKey".to_string()
                } else {
                    "Not enough balance for the transfer".to_string()
                };

                Err(Error::NotEnoughBalance(msg))
            }
            Err(other_error) => Err(other_error),
            Ok(id) => Ok(id),
        }
    }
}

#[cfg(all(test, feature = "simulated-payouts"))]
mod tests {
    use super::*;
    use crate::{
        api::{
            app::test_helpers::{new_safe_instance, random_nrs_name},
            common::sk_to_hex,
        },
        retry_loop,
    };
    use anyhow::{anyhow, bail, Result};

    #[tokio::test]
    async fn test_keys_create_preload_test_coins() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let _ = safe.keys_create_preload_test_coins("12.23").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_create_and_preload_from_sk_string() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_, from_keypair) = safe.keys_create_preload_test_coins("543.2312").await?;
        let from_sk_hex = sk_to_hex(from_keypair.secret_key()?);

        let preload_amount = "1.800000000";
        let (_, keypair) = safe
            .keys_create_and_preload_from_sk_string(&from_sk_hex, preload_amount)
            .await?;
        let balance = safe.keys_balance_from_sk(keypair.secret_key()?).await?;
        assert_eq!(balance, preload_amount);
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_create_preload_invalid_amounts() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        match safe.keys_create_preload_test_coins(".45").await {
            Ok(_) => {
                bail!("Key with test-coins was created unexpectedly".to_string(),)
            }
            Err(Error::InvalidAmount(msg)) => assert_eq!(
                msg,
                "Invalid safecoins amount '.45' (Can\'t parse token units)".to_string()
            ),
            other => bail!("Error returned is not the expected one: {:?}", other),
        };

        let (_, keypair) = safe.keys_create_preload_test_coins("12").await?;
        let mut sk_hex = sk_to_hex(keypair.secret_key()?);
        match safe
            .keys_create_and_preload_from_sk_string(&sk_hex, ".003")
            .await
        {
            Ok(_) => {
                bail!("Key was created unexpectedly".to_string(),)
            }
            Err(Error::InvalidAmount(msg)) => assert_eq!(
                msg,
                "Invalid safecoins amount '.003' (Can\'t parse token units)".to_string()
            ),
            other => bail!("Error returned is not the expected one: {:?}", other),
        };

        // test it fails with corrupted secret key
        sk_hex.replace_range(..6, "ababab");
        match safe
            .keys_create_and_preload_from_sk_string(&sk_hex, ".003")
            .await
        {
            Ok(_) => {
                bail!("Key was created unexpectedly".to_string(),)
            }
            Err(Error::InvalidAmount(msg)) => assert_eq!(
                msg,
                "Invalid safecoins amount '.003' (Can\'t parse token units)".to_string()
            ),
            other => bail!("Error returned is not the expected one: {:?}", other),
        };

        // test it fails to preload with more than available balance in source (which has only 12 coins)
        let amount = "12.000000001";
        match safe
            .keys_create_and_preload_from_sk_string(&sk_hex, amount)
            .await
        {
            Ok(_) => Err(anyhow!("Key was created unexpectedly".to_string(),)),
            Err(Error::NotEnoughBalance(msg)) => {
                assert_eq!(
                    msg,
                    format!(
                        "Not enough balance at 'source' for the operation: {}",
                        amount
                    )
                );
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn test_keys_create_pk() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_, from_keypair) = safe.keys_create_preload_test_coins("1.1").await?;
        let from_sk_hex = sk_to_hex(from_keypair.secret_key()?);
        let _ = safe
            .keys_create_and_preload_from_sk_string(&from_sk_hex, "0.1")
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_pk() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let preload_amount = "1.154200000";
        let (_, keypair) = safe.keys_create_preload_test_coins(preload_amount).await?;
        let current_balance = safe.keys_balance_from_sk(keypair.secret_key()?).await?;
        assert_eq!(preload_amount, current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_xorurl() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let preload_amount = "0.243000000";
        let (xorurl, keypair) = safe.keys_create_preload_test_coins(preload_amount).await?;
        let current_balance = safe
            .keys_balance_from_url(&xorurl, keypair.secret_key()?)
            .await?;
        assert_eq!(preload_amount, current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_wrong_url() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_, keypair) = safe.keys_create_preload_test_coins("0").await?;

        let invalid_xorurl = "safe://this-is-not-a-valid-xor-url";
        let current_balance = safe
            .keys_balance_from_url(&invalid_xorurl, keypair.secret_key()?)
            .await;
        match current_balance {
            Err(Error::ContentNotFound(msg)) => {
                assert!(msg.contains(&format!("Content not found at {}", invalid_xorurl)));
                Ok(())
            }
            Err(err) => Err(anyhow!("Error returned is not the expected: {:?}", err)),
            Ok(balance) => Err(anyhow!("Unexpected balance obtained: {:?}", balance)),
        }
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_wrong_location() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let amount = "35312.000000000";
        let (xorurl, keypair) = safe.keys_create_preload_test_coins(amount).await?;

        let current_balance = safe
            .keys_balance_from_url(&xorurl, keypair.secret_key()?)
            .await?;
        assert_eq!(amount, current_balance);

        // let's use the XOR-URL of another SafeKey
        let (other_kp_xorurl, _) = safe.keys_create_preload_test_coins("0").await?;
        let current_balance = safe
            .keys_balance_from_url(&other_kp_xorurl, keypair.secret_key()?)
            .await;
        match current_balance {
            Err(Error::InvalidInput(msg)) => {
                assert!(msg.contains(
                "The URL doesn't correspond to the public key derived from the provided secret key"
            ));
                Ok(())
            }
            Err(err) => Err(anyhow!("Error returned is not the expected: {:?}", err)),
            Ok(balance) => Err(anyhow!("Unexpected balance obtained: {:?}", balance)),
        }
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_wrong_sk() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, _) = safe.keys_create_preload_test_coins("0").await?;

        let mut rng = OsRng;
        let sk = Keypair::new_ed25519(&mut rng).secret_key()?;
        let current_balance = safe.keys_balance_from_url(&xorurl, sk).await;
        match current_balance {
            Err(Error::InvalidInput(msg)) => {
                assert_eq!(msg, "The URL doesn't correspond to the public key derived from the provided secret key");
                Ok(())
            }
            Err(err) => Err(anyhow!("Error returned is not the expected: {:?}", err)),
            Ok(balance) => Err(anyhow!("Unexpected balance obtained: {:?}", balance)),
        }
    }

    #[tokio::test]
    async fn test_keys_balance_pk() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let preload_amount = "1743.234";
        let (_, from_keypair) = safe.keys_create_preload_test_coins(preload_amount).await?;
        let from_sk_hex = sk_to_hex(from_keypair.secret_key()?);

        let amount = "1740.000000000";
        let (_, to_keypair) = safe
            .keys_create_and_preload_from_sk_string(&from_sk_hex, amount)
            .await?;

        let from_current_balance = safe
            .keys_balance_from_sk(from_keypair.secret_key()?)
            .await?;
        assert_eq!(
            "3.234000000", /*== 1743.234 - 1740 */
            from_current_balance
        );

        let to_current_balance = safe.keys_balance_from_sk(to_keypair.secret_key()?).await?;
        assert_eq!(amount, to_current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_balance_xorname() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let preload_amount = "435.34";
        let (from_xorname, from_keypair) =
            safe.keys_create_preload_test_coins(preload_amount).await?;
        let from_sk_hex = sk_to_hex(from_keypair.secret_key()?);

        let amount = "35.300000000";
        let (to_xorname, to_keypair) = safe
            .keys_create_and_preload_from_sk_string(&from_sk_hex, amount)
            .await?;

        let from_current_balance = safe
            .keys_balance_from_url(&from_xorname, from_keypair.secret_key()?)
            .await?;
        assert_eq!(
            "400.040000000", /*== 435.34 - 35.3*/
            from_current_balance
        );

        let to_current_balance = safe
            .keys_balance_from_url(&to_xorname, to_keypair.secret_key()?)
            .await?;
        assert_eq!(amount, to_current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_validate_sk_for_url() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (xorurl, keypair) = safe.keys_create_preload_test_coins("23.22").await?;
        let pk = safe
            .validate_sk_for_url(&keypair.secret_key()?, &xorurl)
            .await?;
        assert_eq!(pk, pk_to_hex(&keypair.public_key()));
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_transfer_from_zero_balance() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (_, keypair1) = safe.keys_create_preload_test_coins("0.0").await?;
        let from_sk1_hex = sk_to_hex(keypair1.secret_key()?);
        let (to_safekey_xorurl, _keypair2) = safe.keys_create_preload_test_coins("0.5").await?;

        // test it fails to transfer with 0 balance at SafeKey in <from> argument
        match safe
            .keys_transfer("0", Some(&from_sk1_hex), &to_safekey_xorurl)
            .await
        {
            Err(Error::InvalidAmount(msg)) => {
                assert!(msg.contains("Cannot send zero-value transfers"));
                Ok(())
            }
            Err(err) => Err(anyhow!("Error returned is not the expected: {:?}", err)),
            Ok(_) => Err(anyhow!("Transfer succeeded unexpectedly".to_string(),)),
        }
    }

    #[tokio::test]
    async fn test_keys_transfer_diff_amounts() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (safekey1_xorurl, keypair1) = safe.keys_create_preload_test_coins("0.5").await?;
        let from_sk1_hex = sk_to_hex(keypair1.secret_key()?);

        let (safekey2_xorurl, keypair2) = safe.keys_create_preload_test_coins("100.5").await?;
        let from_sk2_hex = sk_to_hex(keypair2.secret_key()?);

        // test it fails to transfer more than current balance at SafeKey in <from> argument
        match safe
            .keys_transfer("100.6", Some(&from_sk1_hex), &safekey2_xorurl)
            .await
        {
            Err(Error::NotEnoughBalance(msg)) => assert_eq!(
                msg,
                "Not enough balance for the transfer at provided source SafeKey".to_string()
            ),
            Err(err) => {
                bail!("Error returned is not the expected: {:?}", err)
            }
            Ok(_) => {
                bail!("Transfer succeeded unexpectedly".to_string(),)
            }
        };

        // test it fails to transfer as it's a invalid/non-numeric amount
        match safe
            .keys_transfer(".06", Some(&from_sk2_hex), &safekey2_xorurl)
            .await
        {
            Err(Error::InvalidAmount(msg)) => assert_eq!(
                msg,
                "Invalid safecoins amount '.06' (Can\'t parse token units)"
            ),
            Err(err) => {
                bail!("Error returned is not the expected: {:?}", err)
            }
            Ok(_) => {
                bail!("Transfer succeeded unexpectedly".to_string(),)
            }
        };

        // test it fails to transfer less than 1 nano coin
        match safe.keys_transfer(
                "0.0000000009",
                Some(&from_sk2_hex),
                &safekey2_xorurl,
            ).await {
                    Err(Error::InvalidAmount(msg)) => assert_eq!(msg, "Invalid safecoins amount '0.0000000009', the minimum possible amount is one nano coin (0.000000001)"),
                    Err(err) => bail!("Error returned is not the expected: {:?}", err),
                    Ok(_) => bail!("Transfer succeeded unexpectedly".to_string()),
            };

        // test successful transfer
        match safe
            .keys_transfer("100.4", Some(&from_sk2_hex), &safekey1_xorurl)
            .await
        {
            Err(msg) => Err(anyhow!("Transfer was expected to succeed: {}", msg)),
            Ok(_) => {
                let from_current_balance =
                    safe.keys_balance_from_sk(keypair2.secret_key()?).await?;
                assert_eq!("0.100000000", from_current_balance);
                let to_current_balance = safe.keys_balance_from_sk(keypair1.secret_key()?).await?;
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
        let sk1_hex = sk_to_hex(keypair1.secret_key()?);
        safe.wallet_insert(
            &to_wallet_xorurl,
            Some("my-first-balance"),
            true, // set --default
            &sk1_hex,
        )
        .await?;

        let (_, keypair2) = safe.keys_create_preload_test_coins("4621.45").await?;
        let sk2_hex = sk_to_hex(keypair2.secret_key()?);

        // test successful transfer
        match safe
            .keys_transfer("523.87", Some(&sk2_hex), &to_wallet_xorurl.clone())
            .await
        {
            Err(msg) => Err(anyhow!("Transfer was expected to succeed: {}", msg)),
            Ok(_) => {
                let from_current_balance =
                    safe.keys_balance_from_sk(keypair2.secret_key()?).await?;
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

        let (_, keypair1) = safe.keys_create_preload_test_coins("0.2").await?;
        let from_sk1_hex = sk_to_hex(keypair1.secret_key()?);

        let (to_safekey_xorurl, keypair2) = safe.keys_create_preload_test_coins("0.1").await?;

        let to_nrs_name = random_nrs_name();
        let (xorurl, _, _) = safe
            .nrs_map_container_create(&to_nrs_name, &to_safekey_xorurl, false, true, false)
            .await?;
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        // test successful transfer
        let _ = safe
            .keys_transfer(
                "0.2",
                Some(&from_sk1_hex),
                &format!("safe://{}", to_nrs_name),
            )
            .await?;

        let from_current_balance = safe.keys_balance_from_sk(keypair1.secret_key()?).await?;
        assert_eq!("0.000000000" /* 0.2 - 0.2 */, from_current_balance);

        let to_current_balance = safe.keys_balance_from_sk(keypair2.secret_key()?).await?;
        assert_eq!("0.300000000" /* 0.1 + 0.2 */, to_current_balance);

        Ok(())
    }

    #[tokio::test]
    async fn test_keys_transfer_to_pk() -> Result<()> {
        let mut safe = new_safe_instance().await?;

        let (_, keypair1) = safe.keys_create_preload_test_coins("0.136").await?;
        let from_sk1_hex = sk_to_hex(keypair1.secret_key()?);

        let (_, keypair2) = safe.keys_create_preload_test_coins("0.73").await?;
        let to_pk2_hex = pk_to_hex(&keypair2.public_key());

        let _ = safe
            .keys_transfer("0.111", Some(&from_sk1_hex), &to_pk2_hex)
            .await?;

        let from_current_balance = safe.keys_balance_from_sk(keypair1.secret_key()?).await?;
        assert_eq!("0.025000000" /* 0.136 - 0.111 */, from_current_balance);

        let to_current_balance = safe.keys_balance_from_sk(keypair2.secret_key()?).await?;
        assert_eq!("0.841000000" /* 0.73 + 0.111 */, to_current_balance);

        Ok(())
    }
}
