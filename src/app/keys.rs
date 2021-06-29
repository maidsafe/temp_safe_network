// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::Safe;
use crate::{Error, Result};
use hex::encode;
use rand::rngs::OsRng;
use safe_network::types::{Keypair, SecretKey};
use xor_name::XorName;

impl Safe {
    // Generate a key pair
    pub fn generate_random_ed_keypair(&self) -> Keypair {
        let mut rng = OsRng;
        Keypair::new_ed25519(&mut rng)
    }

    // Check that the XOR/NRS-URL corresponds to the public key derived from the provided client id
    pub async fn validate_sk_for_url(&self, secret_key: &SecretKey, url: &str) -> Result<String> {
        let derived_xorname = match secret_key {
            SecretKey::Ed25519(sk) => {
                let pk: ed25519_dalek::PublicKey = sk.into();
                XorName(pk.to_bytes())
            }
            _ => {
                return Err(Error::InvalidInput(
                    "Cannot form a keypair from a BlsKeyShare at this time.".to_string(),
                ))
            }
        };

        let (safeurl, _) = self.parse_and_resolve_url(url).await?;
        if safeurl.xorname() != derived_xorname {
            Err(Error::InvalidInput(
                "The URL doesn't correspond to the public key derived from the provided secret key"
                    .to_string(),
            ))
        } else {
            Ok(encode(&derived_xorname))
        }
    }
}

#[cfg(all(test, feature = "testings"))]
mod tests {
    use super::*;
    use crate::{
        app::test_helpers::{new_safe_instance, random_nrs_name},
        common::sk_to_hex,
        retry_loop,
    };
    use anyhow::{anyhow, bail, Result};

    #[tokio::test]
    async fn test_keys_create_preload_test_coins() -> Result<()> {
        let safe = new_safe_instance().await?;
        let _ = safe.keys_create_preload_test_coins("12.23").await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_create_and_preload_from_sk_string() -> Result<()> {
        let safe = new_safe_instance().await?;
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
        let safe = new_safe_instance().await?;
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
        let safe = new_safe_instance().await?;
        let (_, from_keypair) = safe.keys_create_preload_test_coins("1.1").await?;
        let from_sk_hex = sk_to_hex(from_keypair.secret_key()?);
        let _ = safe
            .keys_create_and_preload_from_sk_string(&from_sk_hex, "0.1")
            .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_pk() -> Result<()> {
        let safe = new_safe_instance().await?;
        let preload_amount = "1.154200000";
        let (_, keypair) = safe.keys_create_preload_test_coins(preload_amount).await?;
        let current_balance = safe.keys_balance_from_sk(keypair.secret_key()?).await?;
        assert_eq!(preload_amount, current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_test_coins_balance_xorurl() -> Result<()> {
        let safe = new_safe_instance().await?;
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
        let safe = new_safe_instance().await?;
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
        let safe = new_safe_instance().await?;
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
        let safe = new_safe_instance().await?;
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
        let safe = new_safe_instance().await?;
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
        let safe = new_safe_instance().await?;
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
        let safe = new_safe_instance().await?;
        let (xorurl, keypair) = safe.keys_create_preload_test_coins("23.22").await?;
        let pk = safe
            .validate_sk_for_url(&keypair.secret_key()?, &xorurl)
            .await?;
        assert_eq!(pk, encode(keypair.public_key().to_bytes()));
        Ok(())
    }

    #[tokio::test]
    async fn test_keys_transfer_from_zero_balance() -> Result<()> {
        let safe = new_safe_instance().await?;
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
        let safe = new_safe_instance().await?;
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
}
