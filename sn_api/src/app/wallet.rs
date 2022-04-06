// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::safeurl::{ContentType, SafeUrl, XorUrl};
use crate::{Error, Result, Safe};
use bytes::Bytes;
use log::debug;
use safe_network::types::Token;
pub use sn_dbc::Dbc;
use std::collections::{BTreeMap, BTreeSet};

// Type tag used for the Wallet
const WALLET_TYPE_TAG: u64 = 1_000;

/// Set of spendable DBC's mapped to their friendly name
/// as defined/chosen by the user when depositing DBC's into a Wallet.
pub type WalletSpendableDbcs = BTreeMap<String, Dbc>;

impl Safe {
    /// Create an empty Wallet on Safe and return its XOR-URL
    pub async fn wallet_create(&self) -> Result<XorUrl> {
        // A Wallet is stored on a Private Register
        let xorurl = self
            .multimap_create(None, WALLET_TYPE_TAG, /*private = */ true)
            .await?;

        let mut safeurl = SafeUrl::from_url(&xorurl)?;
        safeurl.set_content_type(ContentType::Wallet)?;

        Ok(safeurl.to_string())
    }

    /// Deposit a DBC into a Wallet to make it a spendable balance. It will be mapped to
    /// a friendly name if provided, otherwise the hash of the DBC content will be used by default.
    /// Returns the friendly name set to it.
    pub async fn wallet_deposit(
        &self,
        wallet_url: &str,
        spendable_name: Option<&str>,
        dbc: &Dbc,
    ) -> Result<String> {
        if !dbc.is_bearer() {
            return Err(Error::InvalidInput("Only bearer DBC's are supported at this point by the Wallet. Please deposit a bearer DBC's.".to_string()));
        }

        let spendable_name = match spendable_name {
            Some(name) => name.to_string(),
            None => hex::encode(dbc.hash()),
        };

        let dbc_bytes = Bytes::from(rmp_serde::to_vec_named(&dbc).map_err(|err| {
            Error::Serialisation(format!(
                "Failed to serialise DBC to insert it into the Wallet: {:?}",
                err
            ))
        })?);

        let safeurl = self.parse_and_resolve_url(wallet_url).await?.to_string();
        let dbc_xorurl = self.store_private_bytes(dbc_bytes, None).await?;

        let entry = (spendable_name.clone().into_bytes(), dbc_xorurl.into_bytes());
        let _entry_hash = self
            .multimap_insert(
                &safeurl,
                entry,
                BTreeSet::default(), // TODO: provide root to replace if DBC already exists ??
            )
            .await?;

        debug!(
            "A spendable DBC deposited into Wallet at {}, with name: {}",
            safeurl, spendable_name
        );

        Ok(spendable_name)
    }

    /// Fetch a Wallet from a Url performing all type of URL resolution required.
    /// Return the set of spendable DBCs found in the Wallet.
    pub async fn wallet_get(&self, url: &str) -> Result<WalletSpendableDbcs> {
        let safeurl = self.parse_and_resolve_url(url).await?;
        debug!("Wallet URL was parsed and resolved to: {}", safeurl);
        self.fetch_wallet(&safeurl).await
    }

    /// Fetch a Wallet from a SafeUrl without performing any type of URL resolution
    pub(crate) async fn fetch_wallet(&self, safeurl: &SafeUrl) -> Result<WalletSpendableDbcs> {
        let entries = match self.fetch_multimap(safeurl).await {
            Ok(entries) => entries,
            Err(Error::AccessDenied(_)) => {
                return Err(Error::AccessDenied(format!(
                    "Couldn't read Wallet at \"{}\"",
                    safeurl
                )))
            }
            Err(Error::ContentNotFound(_)) => {
                return Err(Error::ContentNotFound(format!(
                    "No Wallet found at {}",
                    safeurl
                )))
            }
            Err(err) => {
                return Err(Error::ContentError(format!(
                    "Failed to read balances from Wallet: {}",
                    err
                )))
            }
        };

        let mut balances = WalletSpendableDbcs::default();
        for (_entry_hash, (key, value)) in entries.iter() {
            let xorurl_str = std::str::from_utf8(value)?;
            let dbc_xorurl = SafeUrl::from_xorurl(xorurl_str)?;
            let dbc_bytes = self.fetch_data(&dbc_xorurl, None).await?;

            let dbc: Dbc = rmp_serde::from_slice(&dbc_bytes).map_err(|err| {
                Error::ContentError(format!(
                    "Couldn't deserialise DBC stored in the Wallet at {}: {:?}",
                    safeurl, err
                ))
            })?;

            let spendable_name = String::from_utf8_lossy(key).to_string();
            balances.insert(spendable_name, dbc);
        }

        Ok(balances)
    }

    /// Check the total balance of a Wallet found at a given XOR-URL
    pub async fn wallet_balance(&self, wallet_url: &str) -> Result<String> {
        debug!("Finding total Wallet balance for: {}", wallet_url);

        // Let's get the list of balances from the Wallet
        let balances = self.wallet_get(wallet_url).await?;
        debug!("Spendable balances to check: {:?}", balances);

        // Iterate through the DBCs adding up the amounts
        let mut total_balance = Token::from_nano(0);
        for (name, dbc) in balances.iter() {
            debug!("Checking spendable balance named: {}", name);

            let balance = match dbc.amount_secrets_bearer() {
                Ok(amount_secrets) => Token::from_nano(amount_secrets.amount()),
                Err(err) => {
                    debug!("Ignoring amount from DBC found in Wallet due to error in revealing secret amount: {:?}", err);
                    continue;
                }
            };
            debug!("Amount in spendable balance '{}': {}", name, balance);

            match total_balance.checked_add(balance) {
                None => {
                    return Err(Error::ContentError(format!(
                        "Failed to calculate total balance due to overflow when adding {} to {}",
                        balance, total_balance
                    )))
                }
                Some(new_total_balance) => total_balance = new_total_balance,
            }
        }

        Ok(total_balance.to_string())
    }

    /// Reissue a DBC from a Wallet returning the output DBC, and automatically depositing
    /// the change DBC into the source wallet.
    pub async fn wallet_reissue(
        &self,
        _wallet_url: &str,
        _amount: &str,
        _recipient: &str,
    ) -> Result<Dbc> {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app::test_helpers::new_safe_instance, retry_loop, retry_loop_for_pattern};
    use anyhow::{anyhow, Result};

    // FIXME: allow to set an amount and sk to generate a DBC with,
    // instead of deserialising a static DBC (this is all taken from sn_dbc examples for now)
    fn new_dbc() -> Result<Dbc> {
        let dbc_hex = "c4a1953dfa419234b8ab9a60dc0e6fc073a1576cf34a45345218b95e445fdfe7cae2cc4050352891483e93e9f6dc311766ead4835324c04a222ed4aace21410625747877cb83eb7b8b59e196c3ff607e097aecb647415024716856f44e5dd4ae47406706769a8f1498d11ecdbd58f7e83b24c3e498425b86a0b2a07d5277322ed2112a3a853a0c36c0e49fa6ce5543a763b8f891d8d98d5c0c509e62408b354eb18b21c417e7b51b043d52bfc2ed6b444b1b7b00f7e5a359ac712dab253404970000000000000001fb0aa04f1d01b1bdb5e033db74c8fa8ce0afa6a98992ffd5a6639bde4fc772c1bd546d885fd355619bd1a226c6cc743f40c939d39634f6bcc15a1a590e40d597d7c9a08e2346cbe7c06e1a514abedc93000000000000000138aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e18707ea919914faca624361a4cf56c46c5af7311712a4ce792b2c4b3d0dd6fa7e0e5739d453d82f9cd7957421539b6ce3df184007606ccc21ec98fcde61af1ff83a9a1836eb943a8951b5dc368662efdfcccfbefff5cc368353ebb6ea06217dfc317a28312d0436d55d8c210fcb1bb35295dcbe4b1f43e932c630f99b7629f83725efa04b8b5de60ad9c59e6f6ca02f865fcaa78f39f6245122ef72a9a594ece893bdcad0ed9c6d4ef70dbde8e0b6c674eaebb054dc2bae1057718f09d7272c74c322c200f360ac1e593cb4cd9386fff1a680b6334a5c553f25aeb8d5a78b2137e3beb3c686b1141ccfe6303b08882cf564cadab4d29bca09c7c9544ed427003a91d7cd080343cd4cc87e7ba621b0bad37e384428439d0bb0ece88bde3ee3018adb116f40b633c38418a34d6100baa97d84f76d09ede201b47e36a163e8d59f8724188b3f993d73d6ee05f98e665c7368012c9e10f5ca94dabf7b11c71816039a93e0d0ca221b4d69d1c3ce0afc416b8c05e8b49870eba331310eedfb4b866269b99cc1d0ab8ee3e8a3b574fe2bbbd100b882868cfffe28db4ae1ef9345e7bc94d3babdb3abe3708b7913d69c2a9d766f176093c7809358a5876d8aa36cafd671b87a8ff2a8edb14ad93d584ce65c867ac523a91164e2a6320fa22efa817915959a98e4af659c305b5a2f209341caed64a7e22d5e38811d3ad33cfb5252286b126061691e2b47d2f83e15131a0456562449812e0c4590b5ce2556ef9e07113f55aab3c518c74b8cf3703536de6fc83e775d5203b343af00322512fa97584c4522b8715b48b2d428dec09c2a87a5d2577fb9ea0a895fcd2b86a1d251e30b39e50a10596ddef7e61ab01b624b66fbd1528314628d483999ea0d457a625498817bc9b0215808e77d7f544fcd115d80b520d426990093797183e3bd8b70ce540f20045432f7ee915bf993fc1ddfeb2377b2accad1444174cafc74b6eb78fe764bca0d203b05231581f1cec51cc8d9c8557fc3743db85e126570964526c1c36fabedc8f7f7f9cb871895759777311242957e7ffafdcadde45381fb3ebaa89971361667b59c7d6b858be1aeb0eb7ba7dd643c3cb2ca63f9689ea6d3dcfd9c1503e5e6f264c9b2243a11bcf019be5c43902dd850fda28c9d88144d19a6df48a7672ae33b962ee24cbcc9d826f32e9b9ffbf4ea28932027a5b1ff6338142c68ba3353376735244e07dc49a1f92ca55b625a167dd298cbbdb3a3855b8636fb27178b72015b40789257e72f3e452860756392ec9b2173cb4c8f551193b5d6275c937de915379100000000000003a096535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe88000000000000000138aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e1875ec7f615f506c5b54a6f2c581b53054eb27f064c3216970f98165341f1f5e25e7508f4d421c3dba24ff7adbe61d61da910f652f9d986fe9649d2937a0f631c5c8d1a922817b5637b263ac7642066e81df5a8c4e8ecb51168fd2349c71534d4ab0000000000000001bd546d885fd355619bd1a226c6cc743f40c939d39634f6bcc15a1a590e40d597d7c9a08e2346cbe7c06e1a514abedc933dfe1a98a93b51d4c1da98fb7f71c82689cb90a92b4b882d2b7ebbfd63f167e82be854194397cba8f0a87a5fcd07848220c26efbccdc78006f7abb47d90d0f5600000000000000010454acf22d354960e1234aa7878e260bfbc16960378064edecba8edcf7245e860000000000000001a460961fa01de9af3e25b9fa2de0eae07fab57a77d008e32eed003fc9b0fab7c88d31227205b67c5869bfa190787a806a72fcf51b7215073a5fa096a33d633a38c1cbc6acb0e9352fa6ada19f2c940c47e47255f0eb1388be73eac4c63e9678586e57f4967288d544910477648557437c78fee95ed7ef617e040dddedcf87f4c447127fa3efdc4bf00000000000000285434cdc90dcf805084231785b0601ef76669a3850767f2e425ca9f823d6dabbe2da9cb85de80e0ff2bbd95e1302a5aa640f8a05e740ba15e12cc1653cf1fb3b07503b97f77e0564498040a9d77f02858dd0f084c98d7880ff61259b068734b08b6274864828e02dbb33cb912b9ae1f48964a9a8db13db72f9e702b9a60da2a6f1b8febd9e851bd678b191121f05b3994c34edb90f6cd72c9b4999fa51209b859bb9f14223e620cb1cc11137cc45f66fa000000000000002047adb7e64b0c19ed1203da940e2154497d81ab0964688819e37542352bb05a0107feb99b49f0234c9eaa95106453d0b2c16551530000000000000000000000000000000000000000000000000000000000000000";

        let mut dbc_str = hex::decode(dbc_hex)
            .map_err(|err| anyhow!("Couldn't hex-decode test DBC: {:?}", err))?;

        dbc_str.reverse();
        bincode::deserialize(&dbc_str)
            .map_err(|err| anyhow!("Couldn't deserialise test DBC: {:?}", err))
    }

    #[tokio::test]
    async fn test_wallet_create() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;
        assert!(wallet_xorurl.starts_with("safe://"));
        let _ = retry_loop!(safe.fetch(&wallet_xorurl, None));

        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!("0.000000000", current_balance);
        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_deposit_and_balance() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let _ = retry_loop!(safe.fetch(&wallet_xorurl, None));

        // TODO: deposit the first DBC with 12.23 amount, and a second one with 1.53
        let dbc1 = new_dbc()?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc1)
            .await?;

        let _ = retry_loop_for_pattern!(safe.wallet_balance(&wallet_xorurl), Ok(balance) if balance == &Token::from_nano(u64::MAX).to_string())?;

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_get() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let _ = retry_loop!(safe.fetch(&wallet_xorurl, None));

        let dbc1 = new_dbc()?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc1)
            .await?;

        let wallet_balances = retry_loop_for_pattern!(safe.wallet_get(&wallet_xorurl), Ok(balances) if balances.len() == 1)?;

        let dbc_read = wallet_balances
            .get("my-first-dbc")
            .ok_or_else(|| anyhow!("Couldn't read DBC from fetched wallet"))?;

        assert_eq!(dbc_read.owner_base(), dbc1.owner_base());

        let balance = dbc_read
            .amount_secrets_bearer()
            .map_err(|err| anyhow!("Couldn't read balance from DBC fetched: {:?}", err))?;
        assert_eq!(balance.amount(), u64::MAX);

        Ok(())
    }
}
