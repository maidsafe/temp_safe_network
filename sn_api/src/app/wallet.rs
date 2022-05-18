// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub use sn_dbc::Dbc;

use super::helpers::parse_tokens_amount;
use super::register::EntryHash;
use crate::safeurl::{ContentType, SafeUrl, XorUrl};
use crate::{Error, Result, Safe};
use bytes::Bytes;
use log::{debug, warn};
use sn_dbc::{rng, Owner, OwnerOnce, TransactionBuilder};
use sn_interface::types::Token;
use std::collections::{BTreeMap, BTreeSet};

// Type tag used for the Wallet
const WALLET_TYPE_TAG: u64 = 1_000;

/// Set of spendable DBC's mapped to their friendly name
/// as defined/chosen by the user when depositing DBC's into a Wallet.
pub type WalletSpendableDbcs = BTreeMap<String, (Dbc, EntryHash)>;

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

        // TODO: check the input DBCs were spent and all other sort of verifications,
        // perhaps all optional, and we may want a separate API to also do these verifications
        // for the user to perform them without depositing the DBC into a Wallet.

        let safeurl = self.parse_and_resolve_url(wallet_url).await?;

        let spendable_name = match spendable_name {
            Some(name) => name.to_string(),
            None => hex::encode(dbc.hash()),
        };

        self.insert_dbc_into_wallet(&safeurl, dbc, spendable_name.clone())
            .await?;

        debug!(
            "A spendable DBC deposited into Wallet at {}, with name: {}",
            safeurl, spendable_name
        );

        Ok(spendable_name)
    }

    /// Fetch a Wallet from a Url performing all type of URL resolution required.
    /// Return the set of spendable DBCs found in the Wallet.
    pub async fn wallet_get(&self, wallet_url: &str) -> Result<WalletSpendableDbcs> {
        let safeurl = self.parse_and_resolve_url(wallet_url).await?;
        debug!("Wallet URL was parsed and resolved to: {}", safeurl);
        self.fetch_wallet(&safeurl).await
    }

    /// Fetch a Wallet from a SafeUrl without performing any type of URL resolution
    pub(crate) async fn fetch_wallet(&self, safeurl: &SafeUrl) -> Result<WalletSpendableDbcs> {
        let entries = match self.fetch_multimap(safeurl).await {
            Ok(entries) => entries,
            Err(Error::AccessDenied(_)) => {
                return Err(Error::AccessDenied(format!(
                    "Couldn't read Wallet found at \"{}\"",
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
        for (entry_hash, (key, value)) in entries.iter() {
            let xorurl_str = std::str::from_utf8(value)?;
            let dbc_xorurl = SafeUrl::from_xorurl(xorurl_str)?;
            let dbc_bytes = self.fetch_data(&dbc_xorurl, None).await?;

            let dbc: Dbc = match rmp_serde::from_slice(&dbc_bytes) {
                Ok(dbc) => dbc,
                Err(err) => {
                    warn!("Ignoring entry found in Wallet since it cannot be deserialised as a valid DBC: {:?}", err);
                    continue;
                }
            };

            let spendable_name = std::str::from_utf8(key)?.to_string();
            balances.insert(spendable_name, (dbc, *entry_hash));
        }

        Ok(balances)
    }

    /// Check the total balance of a Wallet found at a given XOR-URL
    pub async fn wallet_balance(&self, wallet_url: &str) -> Result<Token> {
        debug!("Finding total Wallet balance for: {}", wallet_url);

        // Let's get the list of balances from the Wallet
        let balances = self.wallet_get(wallet_url).await?;
        debug!("Spendable balances to check: {:?}", balances);

        // Iterate through the DBCs adding up the amounts
        let mut total_balance = Token::from_nano(0);
        for (name, (dbc, _)) in balances.iter() {
            debug!("Checking spendable balance named: {}", name);

            let balance = match dbc.amount_secrets_bearer() {
                Ok(amount_secrets) => Token::from_nano(amount_secrets.amount()),
                Err(err) => {
                    warn!("Ignoring amount from DBC found in Wallet due to error in revealing secret amount: {:?}", err);
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

        Ok(total_balance)
    }

    /// Reissue a Bearer-DBC from a Wallet returning the output DBC, and automatically depositing
    /// the change DBC into the source wallet.
    /// Spent DBCs are marked as removed from the source Wallet, but since all entries are kept in
    /// the history of the Wallet, they can still be retrieved eventually if desired by the user.
    pub async fn wallet_reissue(&self, wallet_url: &str, amount: &str) -> Result<Dbc> {
        debug!(
            "Reissuing Bearer-DBCs from Wallet at {}, for an amount of {} tokens",
            wallet_url, amount
        );

        // Parse and validate the output amount is valid
        let output_amount = parse_tokens_amount(amount)?;
        if output_amount.as_nano() == 0 {
            return Err(Error::InvalidAmount(
                "Output amount to reissue needs to be larger than zero (0).".to_string(),
            ));
        }

        // Resolve Wallet URL and obtain the list of spendable DBCs that can be used in this Tx
        let safeurl = self.parse_and_resolve_url(wallet_url).await?;
        let spendable_dbcs = self.fetch_wallet(&safeurl).await?;

        // We'll combine one or more input DBCs and reissue:
        // - one output DBC for the recipient,
        // - and a second DBC for the change, which will be stored in the source Wallet.
        let mut input_dbcs_to_spend = Vec::<Dbc>::new();
        let mut input_dbcs_entries_hash = BTreeSet::<EntryHash>::new();
        let mut total_input_amount = 0;
        let mut change_amount = output_amount;
        for (name, (dbc, entry_hash)) in spendable_dbcs.into_iter() {
            let dbc_balance = match dbc.amount_secrets_bearer() {
                Ok(amount_secrets) => Token::from_nano(amount_secrets.amount()),
                Err(err) => {
                    warn!("Ignoring input DBC found in Wallet (entry: {}) due to error in revealing secret amount: {:?}", name, err);
                    continue;
                }
            };

            // Add this DBC as input to be spent
            input_dbcs_to_spend.push(dbc);
            input_dbcs_entries_hash.insert(entry_hash);
            total_input_amount += dbc_balance.as_nano();

            // If we've already combined input DBCs for the total output amount, then stop
            match change_amount.checked_sub(dbc_balance) {
                Some(pending_output) => {
                    change_amount = pending_output;
                    if change_amount.as_nano() == 0 {
                        break;
                    }
                }
                None => {
                    change_amount =
                        Token::from_nano(dbc_balance.as_nano() - change_amount.as_nano());
                    break;
                }
            }
        }

        // Make sure total input amount gathered with input DBCs are enough for the output amount
        if total_input_amount < output_amount.as_nano() {
            return Err(Error::NotEnoughBalance(
                Token::from_nano(total_input_amount).to_string(),
            ));
        }

        // We can now reissue the output DBCs
        let (output_dbc, change_dbc) = self
            .reissue_dbcs(input_dbcs_to_spend, output_amount, change_amount)
            .await?;

        if let Some(change_dbc) = change_dbc {
            self.insert_dbc_into_wallet(&safeurl, &change_dbc, "change-dbc".to_string())
                .await?;
        }

        // (virtually) remove input DBCs in the source Wallet
        self.multimap_remove(&safeurl.to_string(), input_dbcs_entries_hash)
            .await?;

        Ok(output_dbc)
    }

    /// Private helper to insert a DBC into the Wallet's underlying Multimap
    async fn insert_dbc_into_wallet(
        &self,
        safeurl: &SafeUrl,
        dbc: &Dbc,
        spendable_name: String,
    ) -> Result<()> {
        if !dbc.is_bearer() {
            return Err(Error::InvalidInput("Only bearer DBC's are supported at this point by the Wallet. Please deposit a bearer DBC's.".to_string()));
        }

        let dbc_bytes = Bytes::from(rmp_serde::to_vec_named(dbc).map_err(|err| {
            Error::Serialisation(format!(
                "Failed to serialise DBC to insert it into the Wallet: {:?}",
                err
            ))
        })?);

        let dbc_xorurl = self.store_private_bytes(dbc_bytes, None).await?;

        let entry = (spendable_name.into_bytes(), dbc_xorurl.into_bytes());
        let _entry_hash = self
            .multimap_insert(&safeurl.to_string(), entry, BTreeSet::default())
            .await?;

        Ok(())
    }

    /// Private helper to reissue DBCs using the sn_dbc API,
    /// and logging the spent input DBCs on the network.
    /// Return the output DBC, and the change DBC if there is one.
    async fn reissue_dbcs(
        &self,
        input_dbcs: Vec<Dbc>,
        output_amount: Token,
        change_amount: Token,
    ) -> Result<(Dbc, Option<Dbc>)> {
        // TODO: support for non-bearer DBCs, and allow to provide a recipient's pk
        let client = self.get_safe_client()?;
        let owneronce = OwnerOnce::from_owner_base(client.dbc_owner(), &mut rng::thread_rng());

        // TODO: enable the use ot decoys
        let mut tx_builder = TransactionBuilder::default()
            .set_decoys_per_input(0)
            .set_require_all_decoys(false)
            .add_inputs_dbc_bearer(input_dbcs.iter())?
            .add_output_by_amount(output_amount.as_nano(), owneronce);

        // If there is a change, issue the change DBC
        let change_owner = Owner::from_random_secret_key(&mut rng::thread_rng());
        let change_owneronce = OwnerOnce::from_owner_base(change_owner, &mut rng::thread_rng());
        if change_amount.as_nano() > 0 {
            tx_builder =
                tx_builder.add_output_by_amount(change_amount.as_nano(), change_owneronce.clone());
        }

        let dbc_builder = tx_builder.build(&mut rng::thread_rng())?;

        // Build the output DBCs
        /*
        // TODO: spend all the input DBCs, collecting the spent proof for each of them:
        //   let _spent_proof_share = self.spend_dbc(input_dbcs).await?;
        //   dbc_builder = dbc_builder.add_spent_proof_share(spent_proof_share);
        //
        // TODO: perform the verification of the transaction and spentproofs before building DBCs.
         */

        let dbcs = dbc_builder.build_without_verifying()?;

        let mut output_dbc = None;
        let mut change_dbc = None;
        for (dbc, owneronce, _) in dbcs {
            // If there is a change DBC store it in the source Wallet
            if change_owneronce == owneronce && change_amount.as_nano() > 0 {
                change_dbc = Some(dbc);
            } else {
                output_dbc = Some(dbc);
            }
        }

        match output_dbc {
            None => Err(Error::DbcReissueError(
                "Unexpectedly failed to generate output DBC. No balance were spent from the Wallet.".to_string(),
            )),
            Some(dbc) => Ok((dbc, change_dbc)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_helpers::{new_read_only_safe_instance, new_safe_instance};
    use anyhow::{anyhow, Result};

    // TODO: allow to set an amount and SK to generate a DBC with,
    // instead of deserialising a hard-coded serialised DBC.
    // Hard-coded serialised DBCs are all generated with sn_dbc mint-repl example for now.
    fn new_dbc(dbc_hex: &str) -> Result<Dbc> {
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

        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::zero());

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_deposit() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-dbc"), &dbc)
            .await?;

        let wallet_balances = safe.wallet_get(&wallet_xorurl).await?;
        assert_eq!(wallet_balances.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_balance() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        // We deposit the first DBC with 12.23 amount
        let dbc1 = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc1)
            .await?;

        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::from_nano(12_230_000_000));

        // ...and a second DBC with 1.53
        let dbc2 = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-second-dbc"), &dbc2)
            .await?;

        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::from_nano(13_760_000_000));

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_balance_overflow() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc1 = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc1)
            .await?;
        let dbc2 = new_dbc(DBC_WITH_MAX)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-second-dbc"), &dbc2)
            .await?;

        match safe.wallet_balance(&wallet_xorurl).await {
            Err(Error::ContentError(msg)) => {
                assert_eq!(
                    msg,
                    format!(
                        "Failed to calculate total balance due to overflow when adding {} to 12.230000000",
                        Token::from_nano(u64::MAX)
                    )
                );
                Ok(())
            }
            Err(err) => Err(anyhow!("Error returned is not the expected: {:?}", err)),
            Ok(balance) => Err(anyhow!("Wallet balance obtained unexpectedly: {}", balance)),
        }
    }

    #[tokio::test]
    async fn test_wallet_get() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc1 = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc1)
            .await?;
        let dbc2 = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-second-dbc"), &dbc2)
            .await?;

        let wallet_balances = safe.wallet_get(&wallet_xorurl).await?;

        let (dbc1_read, _) = wallet_balances
            .get("my-first-dbc")
            .ok_or_else(|| anyhow!("Couldn't read first DBC from fetched wallet"))?;
        assert_eq!(dbc1_read.owner_base(), dbc1.owner_base());
        let balance1 = dbc1_read
            .amount_secrets_bearer()
            .map_err(|err| anyhow!("Couldn't read balance from first DBC fetched: {:?}", err))?;
        assert_eq!(balance1.amount(), 1_530_000_000);

        let (dbc2_read, _) = wallet_balances
            .get("my-second-dbc")
            .ok_or_else(|| anyhow!("Couldn't read second DBC from fetched wallet"))?;
        assert_eq!(dbc2_read.owner_base(), dbc2.owner_base());
        let balance2 = dbc2_read
            .amount_secrets_bearer()
            .map_err(|err| anyhow!("Couldn't read balance from second DBC fetched: {:?}", err))?;
        assert_eq!(balance2.amount(), 12_230_000_000);

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_get_not_owned_wallet() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc)
            .await?;

        // test it fails to get a not owned wallet
        let read_only_safe = new_read_only_safe_instance().await?;
        match read_only_safe.wallet_get(&wallet_xorurl).await {
            Err(Error::AccessDenied(msg)) => {
                assert_eq!(
                    msg,
                    format!("Couldn't read Wallet found at \"{}\"", wallet_xorurl)
                );
                Ok(())
            }
            Err(err) => Err(anyhow!("Error returned is not the expected: {:?}", err)),
            Ok(_) => Err(anyhow!("Wallet get succeeded unexpectedly".to_string())),
        }
    }

    #[tokio::test]
    async fn test_wallet_get_non_compatible_content() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc)
            .await?;

        // We insert an entry (to its underlying data type, i.e. the Multimap) which is
        // not a valid serialised DBC, thus making part of its content incompatible/corrupted.
        let corrupted_dbc_xorurl = safe
            .store_private_bytes(Bytes::from_static(b"bla"), None)
            .await?;
        let entry = (b"corrupted-dbc".to_vec(), corrupted_dbc_xorurl.into_bytes());
        safe.multimap_insert(&wallet_xorurl, entry, BTreeSet::default())
            .await?;

        // Now check the Wallet can still be read and the corrupted entry is ignored
        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::from_nano(1_530_000_000));

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_reissue() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("deposited-dbc-1"), &dbc)
            .await?;
        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("deposited-dbc-2"), &dbc)
            .await?;

        let output_dbc = safe.wallet_reissue(&wallet_xorurl, "2.35").await?;

        let output_balance = output_dbc
            .amount_secrets_bearer()
            .map_err(|err| anyhow!("Couldn't read balance from output DBC: {:?}", err))?;
        assert_eq!(output_balance.amount(), 2_350_000_000);

        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::from_nano(11_410_000_000));

        let wallet_balances = safe.wallet_get(&wallet_xorurl).await?;

        assert_eq!(wallet_balances.len(), 1);

        let (change_dbc_read, _) = wallet_balances
            .get("change-dbc")
            .ok_or_else(|| anyhow!("Couldn't read change DBC from fetched wallet"))?;
        let change = change_dbc_read
            .amount_secrets_bearer()
            .map_err(|err| anyhow!("Couldn't read balance from change DBC fetched: {:?}", err))?;
        assert_eq!(change.amount(), 11_410_000_000);

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_not_enough_balance() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("deposited-dbc"), &dbc)
            .await?;

        match safe.wallet_reissue(&wallet_xorurl, "2.55").await {
            Err(Error::NotEnoughBalance(msg)) => {
                assert_eq!(msg, "1.530000000");
                Ok(())
            }
            Err(err) => Err(anyhow!("Error returned is not the expected: {:?}", err)),
            Ok(_) => Err(anyhow!("Wallet reissue succeeded unexpectedly".to_string())),
        }
    }

    #[tokio::test]
    async fn test_wallet_reissue_invalid_amount() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        match safe.wallet_reissue(&wallet_xorurl, "0").await {
            Err(Error::InvalidAmount(msg)) => {
                assert_eq!(
                    msg,
                    "Output amount to reissue needs to be larger than zero (0)."
                );
                Ok(())
            }
            Err(err) => Err(anyhow!("Error returned is not the expected: {:?}", err)),
            Ok(_) => Err(anyhow!("Wallet reissue succeeded unexpectedly".to_string())),
        }
    }

    #[tokio::test]
    async fn test_wallet_reissue_with_non_compatible_content() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc)
            .await?;

        // We insert an entry (to its underlying data type, i.e. the Multimap) which is
        // not a valid serialised DBC, thus making part of its content incompatible/corrupted.
        let corrupted_dbc_xorurl = safe
            .store_private_bytes(Bytes::from_static(b"bla"), None)
            .await?;
        let entry = (b"corrupted-dbc".to_vec(), corrupted_dbc_xorurl.into_bytes());
        safe.multimap_insert(&wallet_xorurl, entry, BTreeSet::default())
            .await?;

        // Now check we can still reissue from the Wallet and the corrupted entry is ignored
        let _ = safe.wallet_reissue(&wallet_xorurl, "0.4").await?;
        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::from_nano(1_130_000_000));

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_reissue_all_balance() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc)
            .await?;

        // Now check thaat after reissuing with the total balance,
        // there is no change deposited in the Wallet, i.e. Wallet is empty with 0 balance
        let _ = safe.wallet_reissue(&wallet_xorurl, "12.23").await?;

        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::zero());

        let wallet_balances = safe.wallet_get(&wallet_xorurl).await?;
        assert!(wallet_balances.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_deposit_reissued_dbc() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet1_xorurl = safe.wallet_create().await?;
        let wallet2_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet1_xorurl, Some("deposited-dbc"), &dbc)
            .await?;

        let output_dbc = safe.wallet_reissue(&wallet1_xorurl, "0.25").await?;

        safe.wallet_deposit(&wallet2_xorurl, Some("reissued-dbc"), &output_dbc)
            .await?;

        let balance = safe.wallet_balance(&wallet2_xorurl).await?;
        assert_eq!(balance, Token::from_nano(250000000));

        Ok(())
    }

    const DBC_WITH_MAX: &str = "c4a1953dfa419234b8ab9a60dc0e6fc073a1576cf34a45345218b95e445fdfe7cae2cc4050352891483e93e9f6dc311766ead4835324c04a222ed4aace21410625747877cb83eb7b8b59e196c3ff607e097aecb647415024716856f44e5dd4ae47406706769a8f1498d11ecdbd58f7e83b24c3e498425b86a0b2a07d5277322ed2112a3a853a0c36c0e49fa6ce5543a763b8f891d8d98d5c0c509e62408b354eb18b21c417e7b51b043d52bfc2ed6b444b1b7b00f7e5a359ac712dab253404970000000000000001fb0aa04f1d01b1bdb5e033db74c8fa8ce0afa6a98992ffd5a6639bde4fc772c1bd546d885fd355619bd1a226c6cc743f40c939d39634f6bcc15a1a590e40d597d7c9a08e2346cbe7c06e1a514abedc93000000000000000138aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e18707ea919914faca624361a4cf56c46c5af7311712a4ce792b2c4b3d0dd6fa7e0e5739d453d82f9cd7957421539b6ce3df184007606ccc21ec98fcde61af1ff83a9a1836eb943a8951b5dc368662efdfcccfbefff5cc368353ebb6ea06217dfc317a28312d0436d55d8c210fcb1bb35295dcbe4b1f43e932c630f99b7629f83725efa04b8b5de60ad9c59e6f6ca02f865fcaa78f39f6245122ef72a9a594ece893bdcad0ed9c6d4ef70dbde8e0b6c674eaebb054dc2bae1057718f09d7272c74c322c200f360ac1e593cb4cd9386fff1a680b6334a5c553f25aeb8d5a78b2137e3beb3c686b1141ccfe6303b08882cf564cadab4d29bca09c7c9544ed427003a91d7cd080343cd4cc87e7ba621b0bad37e384428439d0bb0ece88bde3ee3018adb116f40b633c38418a34d6100baa97d84f76d09ede201b47e36a163e8d59f8724188b3f993d73d6ee05f98e665c7368012c9e10f5ca94dabf7b11c71816039a93e0d0ca221b4d69d1c3ce0afc416b8c05e8b49870eba331310eedfb4b866269b99cc1d0ab8ee3e8a3b574fe2bbbd100b882868cfffe28db4ae1ef9345e7bc94d3babdb3abe3708b7913d69c2a9d766f176093c7809358a5876d8aa36cafd671b87a8ff2a8edb14ad93d584ce65c867ac523a91164e2a6320fa22efa817915959a98e4af659c305b5a2f209341caed64a7e22d5e38811d3ad33cfb5252286b126061691e2b47d2f83e15131a0456562449812e0c4590b5ce2556ef9e07113f55aab3c518c74b8cf3703536de6fc83e775d5203b343af00322512fa97584c4522b8715b48b2d428dec09c2a87a5d2577fb9ea0a895fcd2b86a1d251e30b39e50a10596ddef7e61ab01b624b66fbd1528314628d483999ea0d457a625498817bc9b0215808e77d7f544fcd115d80b520d426990093797183e3bd8b70ce540f20045432f7ee915bf993fc1ddfeb2377b2accad1444174cafc74b6eb78fe764bca0d203b05231581f1cec51cc8d9c8557fc3743db85e126570964526c1c36fabedc8f7f7f9cb871895759777311242957e7ffafdcadde45381fb3ebaa89971361667b59c7d6b858be1aeb0eb7ba7dd643c3cb2ca63f9689ea6d3dcfd9c1503e5e6f264c9b2243a11bcf019be5c43902dd850fda28c9d88144d19a6df48a7672ae33b962ee24cbcc9d826f32e9b9ffbf4ea28932027a5b1ff6338142c68ba3353376735244e07dc49a1f92ca55b625a167dd298cbbdb3a3855b8636fb27178b72015b40789257e72f3e452860756392ec9b2173cb4c8f551193b5d6275c937de915379100000000000003a096535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe88000000000000000138aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e1875ec7f615f506c5b54a6f2c581b53054eb27f064c3216970f98165341f1f5e25e7508f4d421c3dba24ff7adbe61d61da910f652f9d986fe9649d2937a0f631c5c8d1a922817b5637b263ac7642066e81df5a8c4e8ecb51168fd2349c71534d4ab0000000000000001bd546d885fd355619bd1a226c6cc743f40c939d39634f6bcc15a1a590e40d597d7c9a08e2346cbe7c06e1a514abedc933dfe1a98a93b51d4c1da98fb7f71c82689cb90a92b4b882d2b7ebbfd63f167e82be854194397cba8f0a87a5fcd07848220c26efbccdc78006f7abb47d90d0f5600000000000000010454acf22d354960e1234aa7878e260bfbc16960378064edecba8edcf7245e860000000000000001a460961fa01de9af3e25b9fa2de0eae07fab57a77d008e32eed003fc9b0fab7c88d31227205b67c5869bfa190787a806a72fcf51b7215073a5fa096a33d633a38c1cbc6acb0e9352fa6ada19f2c940c47e47255f0eb1388be73eac4c63e9678586e57f4967288d544910477648557437c78fee95ed7ef617e040dddedcf87f4c447127fa3efdc4bf00000000000000285434cdc90dcf805084231785b0601ef76669a3850767f2e425ca9f823d6dabbe2da9cb85de80e0ff2bbd95e1302a5aa640f8a05e740ba15e12cc1653cf1fb3b07503b97f77e0564498040a9d77f02858dd0f084c98d7880ff61259b068734b08b6274864828e02dbb33cb912b9ae1f48964a9a8db13db72f9e702b9a60da2a6f1b8febd9e851bd678b191121f05b3994c34edb90f6cd72c9b4999fa51209b859bb9f14223e620cb1cc11137cc45f66fa000000000000002047adb7e64b0c19ed1203da940e2154497d81ab0964688819e37542352bb05a0107feb99b49f0234c9eaa95106453d0b2c16551530000000000000000000000000000000000000000000000000000000000000000";

    const DBC_WITH_12_230_000_000: &str = "2fce3f70d7ad38d48d81f3f67c11ea10388a95efff66ce444f3274fb261c119a839e3de7a4c97b3b2b12fde3af09990ed20f48b3a23d79ca9b39cf0b6f492b142c03d3c0ee34d087fa3b9989cab7f7fb4e2328587330ce226277b8062881b3a231b11365712ad3a82ece13af3699fb171f16001af12522a92d1cf6d788a919bf7aa7bc1cd70f0eac4f96967bdd9cb38338aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e1870000000000000001c428455eb4b5485f9465049a3a1b519a89b50ec3bef0c4ebbf45b7198cc348321ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b00000000000000016caf52b6919fa145146588809f6c0d2a0b70053c9501a52730bbfba00de94dfe72f14801d29d5b2d93c8b626d443498d34bf24d89d5012a6067ccb301919272553dcdf50cbf95240e47a3c7e967ba98e70b2a97e866c810a0ba806bb96a1b8308eb9b22b29b44ea5d33ae9e9c057261b09e21456123981952224dbf9cadfd50cfb02e0dc57ce55156f844c3a2c704a5be432d2444765dd15beb7fca57a20508f73df1043f8e2e5d8a56a2fb17fe475f2cca88c8d23eae27f3d172e3d6fad45c1b3232dc10ee46a7016df417935865a99b3761bffdefea6373584f50258ab9d60cddfc80ed76c9454bcaa8083f58e6c53d3095e06b02638eccd668031cf232b87ea9474b4166849a34ffbc805310520c8b19bafa7424df714becb9ce02d6d705b51b168a0b806798da550a0807fc1a8a9122908dbbd7e4838e65a9c9bdbc1fbf3e11a79187444274a828618e4debbcc3cee1ea0d1a157f21b8624bccc8eb58c91a60d84deb16ac42241ace0f3174c3abbca0d3331990e7c6db07ff24c8537060ea76348f96af31927c49c84a4e2c4cb8cdd95f86c0fcc18cb1d6c2a519ced3ad2e9903061dd3f3c8e02fafd969f53f52274faddb0b4f5aef19dc0e62277cbcda3e3ac39cfa4b709efbbdab65440a42e628fff051c83acd583a5e3bbfb2eb4807c5aa2303c7b52f1d539899572b674bd8fb4a529354e8087067a4e374e92b855a7ec523295867f310f7cea5d48bb0295a1c3ea47b6c4229f000d0c83ae22eb449656c25083a5fc99f99a23cf25fc2020941ffb392b3c8788c25b687b21610d35e3a5f12974b20778144d6930772381a694be5605f21142ca2121a7358e377fd20d50ff00ae3550aa2622ac8d8a642c41aec117b74c22732b297dbb4e4637595bb5b7d5a69414c7822e55efb9c40ddbc2a293532ee49c705e9bb2f3fff3f2dcce8c72b90c486b3a750ee594fd2b04c2dd9553e02a226001956bed24790349d243ca5d51167c4a348d19e42bf2cabc72c0510b8217e0ce5997b763fd11a881c8757f565e3ca3766fc5c3e011ad40d634666859a3a5fdbab6fb627c5abd1d80894424c5a1b0de74cfdf5eb17b9c8bb873f4515adc3e9d62d76f4f5bbb0ef453e369ad5ffa853ab6661f2bf242d91d098f5c59ceb4c1151b47f4cd5ee9dc98daca45898168f4ff69fb230d01d7660d0718cc1b6facbb7f8e9a73cb6a8b93aba7dddc450628588fbb916641abaab4a228bc2584950e34f86a8154b4f5773d07a4ad70b9c2fdff3d16933011664e8de0298c61cd2da7511b8a0cd63acf47156bd7f667929fce5b7257582f336683abf609ad7a4b9dca4395849d81da230d13822d01224d49f4d249d9881d2e876335824514448300000000000003a00af4916a22e0177947732d2d58db7e0325d004f1de5718b3275ae009845fa53ce6948ddd9623e445c8433d30c162c9a22b05a80d417ac2efc2bf684e091bc26a81bda1ec9253ed4657c2440710623741007bdb90e9d94121babea7484d25a49432e71b55250be2a5c8ac7716ec2c61218cca98458ae648e11d31c185d20e1eb21300e7b360f1553a128ff97d54c1510a26178421859bef1dae17e4cf496acc39b20059e48b5c94c7ab244dbfc368d12868af7cac467d8c18e8cb5b60c22b7e9151e013dd28b813254e65a73c24c0efae47b29cdc3a34286dabb6fd09d9ad8b121655394d272934013773667eae726cdbab0b42cfc2c0bd988ecacafead6f179046fa3a4f5fb8ca7a790e5685f262c0e770c20a700fdab00aeac3a8634d84102f750996b5323356ad1b3199702c2d23a64279726f35e0eb630f91794dda905093995d6b457782792564be257a191f2bdeedd01e76051f7aa740735b8c408c618b3b6edac93f7504d441855d89adb6e13ed2a60c81ac0a0b445a2d1447bc30f0f30376ab55995ad0b639c1dfddb3e29e8f8ddccb59e55d863117ca6bc1c8dbf42983ea889cd03b19ab7c76addddb6dbdd53856b92df8cc929f3d37eb27ba6ab68194fc55f9ca2943acd265538751335cdbf527675494d176282e59a9c8deb95645b5b2b8660456434701b182e08915dbb6de5bfacc5480e867a9f08c8fe2d209eaa44175912ce5fb782425e264587d8ad13a70eb83d0ac0b02095a8f6d029d588b80b4f7a96c73720344247eae9613a9ff6ca368dbd823c225304725a36aee0c1822fe554733bc074c50ea296ff6b60c8711f7ef216e1ff1b532b398ba13e6da8ebd4546dcf296a4c1f7b27da5ec6e771d23afc139af51d5e9b5f2be9f32a7ab949e86cb5158c0f61b5a31f81e9c9ddc8412fcdf971a7cc296e742bcae1d444928152841a927dd7985bb7ef06c1fcfa9a1b908e6b0f52a38321f325f4bff3af4064ce8266c93821bb81f9a0404ffff81520ff370ba1b831cae5db58cb943e59dac65a9092db9fb12b00424c49dc0b92356a5b2801c40b43f8d5262278b44eb980e5857397b6ec9570dde9ca80b5ab041bf81872f3f999182589771a75c8ef0e3b8603888dade9b1d449adaac0f08c7cddbe85df0871722ded236de4b7e5f999ad3f2c8e01cab184eaec5f2fa35b22586e7838e4dde010a37ff24927c8a4845794ee699ab54b57ba50e038008af5ea0bbaf3f4e627fe03809f973980b81ced08a602b7458158ab7684376535265761dc039bb13d01ff90e732b8df17d783338db830193034d48dca8ed42b077293db0aa1da1b14c0c4c0e76c7859b832a226464e2cad01aa41bf8fd1686933b55fd961ab336cdb697e851bf3b6c1142019475e0b2925d6727e15de30cbf7b342daf509778d2af676c53a3e962ddff846c310d768e00000000000003a058998e2c08c59208820948b33b051910a3156d3beff7984798c030f971904a9085c61447ded4a76575b5fcc40b5acd93b1d0b17ba74231d94f3df0f666e751067fdc9364a3a5d07ce0881d44fabf4cf0f478a2ec3d14a668e3e5f249d57138890a5db2846b458f82e77256f2f4b3f1412b0434831aad2b96fed8c9ed2e520a5c54ddfb9f72ee0932eea6ab7a58b2f14c5ef062717e9d1da9c7f213dd11bed65bd7693398c732e08b6fe96095c68e4d7e2d3378c3632e736358e88db56bad7329bfd83bac50c0649fcf4f8baf1f95a3b16a004a02b8d0a469b687ee7f91743232a46ae0eb27d0f45d18dcbc04aeb9595995802a4ddf42f2712248d38d5b7656916211ecc6bbfb4c62b6f9d3393061ead919662e53f356bf2327536dccedd7eee28b06514de978fcc965a3d6d507aba3b3af78e3ac52e3141e9fc1b51302616138f2e14ba340d1baea0b2d3d65e21fc94545f17e956292b4cfa8eaecdfa46344a88eb15f9e230a384caefaced24f6553f5ba7c7e41f9c9bbec71a9a5d8a33b994b6d1ce62a1334351057838c2fbe0aa3a204c17fb11e1970f36a8ef3a16d634890a98dee795c197ad586a5c0cf5a23e72d5931481ddf33a342c5bcc320965b98b46bfc2c51a106ce2dedc8f7196b1af0f30825854d5a34d7fc398aea4548e13d0289fe76ec71f4d9531e56d44d39562ca349fd9b62321e28e7d1c1c406199b5dd4841e12d1c649e9e3dd7db84c8499f61621c188fcf56da4915001bc82a8b8aaafa7001486df8dc215d02261640dfce04c79ba5d66ea5daec4fe8a7f2e39527c8c5ca8d8e05fe00e9f909d49a852a9c2ab96f185aa63ccd5e542b0355d8ee66b252a5a57ab0b30c704ea8055eea246d5700c0aadf23520a445e70b331b24fb8f8c43739403cba21d87a6a63e05cddfcf6934230c4f9420b796b70df0eb70f0c1a94c09c70b7e1bb6e2494fe4ae028c358a9048404e31e6191749b83f8cc942fc796459ce1669ea7c268d3c55cb2be9e8ed54400cd831c974fe59d63b043963e98330c7179038c3be277bcb8d9713234a48a7f2e84bdc41c6f822e7292a02407c9966834fc014727d1aa86ca08f39d5fb53b745bb16d2eb11f23af7955b1a5507ff670dc39d8bd929a2ac8ad2e72ed27d36c386223672d434474f9149d39079bd72977f573bee8686584eddcb1467c24c16a38b0e6f6a9c727f7cca2b7a7a86589fa507438ee0bdaa383ddb95ac2d3abe8d07fe06dc4980bbf6423d7496d4283f85cfc9dc79091eb8695d583ebf31a6aad857a6510e3bf268b634ec62ee7112be92a89633f394792332d8e0f5fcff45353bf8e55db172dae5d60dacd372c1e47f9ce8d497f7a18438fe6a3930c240b29ab1d91176ebd5934d23230b629f46f294f9926d4f33c5ba2c1cee78df2b3bb2d43c67423751cfd78ec652596dec00a06db900000000000003a0d28d2cf0c4c23a0ae40ab3a4a29ed7eaa11a3957bf5ee7b8e61c4f3858982f5fe8f40bb19cf728f76121c2cd51ab9a970000000000000003e8c9885501863a2462cf63cd2d4109746aeda1643d286fd6fb13014bef689dc66b34b6e5a6f83e88aead415c06737aaee5eb3c7fda86f635b3345c72329088cbae5eb375e33f880727993c4c01b736f93fc1520f3fa1b41a8c8eb7002dd6a1a996535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe8800000000000000011ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b158a686df05f9cb8c9f01f6499b27d21c499d31ad7d8274c4dff49192e73c184018a600ba8fc0e3734dd643706ae2af8922fffa4574a630845a4a91f0ce23518000000000000000125cf08c3ee49b669e5cfb0ad13de8e19691287addf6b5228a78fc769ce76c74e00000000000000018f4ba55337b209afca288eb85bfdd9d8919f20b31022d399f34a0170e4163a86dacac52a77e655366987ffdd726dd50bd51f2cc1b554fe2445b95cc78d176ac62f11e3c39cfb0fc1b3d02ad3b255eaba46d5d8587fe0faa7aae3e7d07aaabb9964eecbcba01e1dfcba56deb73c61fab162bdfbb3ce4bce16baa2e01c437b91dfc8213a62da4a4f5e00000000000000282697644c1845c35353295cdef403482ed7d11a8cab4e1d97817f61145a835c17f219c5ead6e1eec40b8fb085d256dd8020d97692739067230e15649334dbbc63bdfcc2d65a435ab30aadaee4494990ffddf25f5cabf451b691376deab52ade138932ac3b1fcab84cd6b9a6d88ee1d044bf495c5d1d08b3a448c5037d88d055dc481d2832a00c1d12349b1ed6ab62b6b718db76ee5066add05240cd94adb26c9afd87e674ddbc9f8d8578449cb7d6435f00000000000000204bcea50da17574d7dca87d2b13ccdd5754394d6d073c2ddfaee89a9a57b44e87efe52ff2a09ccdc8fea40d71af8be598d2c07cbc4a634060bbe2c11bcb537089130b30325c57b23492d9c0263fd01e5800000000";

    const DBC_WITH_1_530_000_000: &str = "2fce3f70d7ad38d48d81f3f67c11ea10388a95efff66ce444f3274fb261c119a839e3de7a4c97b3b2b12fde3af09990ed20f48b3a23d79ca9b39cf0b6f492b142c03d3c0ee34d087fa3b9989cab7f7fb4e2328587330ce226277b8062881b3a231b11365712ad3a82ece13af3699fb171f16001af12522a92d1cf6d788a919bf7aa7bc1cd70f0eac4f96967bdd9cb38338aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e1870000000000000001c428455eb4b5485f9465049a3a1b519a89b50ec3bef0c4ebbf45b7198cc348321ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b00000000000000016caf52b6919fa145146588809f6c0d2a0b70053c9501a52730bbfba00de94dfe72f14801d29d5b2d93c8b626d443498d34bf24d89d5012a6067ccb301919272553dcdf50cbf95240e47a3c7e967ba98e70b2a97e866c810a0ba806bb96a1b8308eb9b22b29b44ea5d33ae9e9c057261b09e21456123981952224dbf9cadfd50cfb02e0dc57ce55156f844c3a2c704a5be432d2444765dd15beb7fca57a20508f73df1043f8e2e5d8a56a2fb17fe475f2cca88c8d23eae27f3d172e3d6fad45c1b3232dc10ee46a7016df417935865a99b3761bffdefea6373584f50258ab9d60cddfc80ed76c9454bcaa8083f58e6c53d3095e06b02638eccd668031cf232b87ea9474b4166849a34ffbc805310520c8b19bafa7424df714becb9ce02d6d705b51b168a0b806798da550a0807fc1a8a9122908dbbd7e4838e65a9c9bdbc1fbf3e11a79187444274a828618e4debbcc3cee1ea0d1a157f21b8624bccc8eb58c91a60d84deb16ac42241ace0f3174c3abbca0d3331990e7c6db07ff24c8537060ea76348f96af31927c49c84a4e2c4cb8cdd95f86c0fcc18cb1d6c2a519ced3ad2e9903061dd3f3c8e02fafd969f53f52274faddb0b4f5aef19dc0e62277cbcda3e3ac39cfa4b709efbbdab65440a42e628fff051c83acd583a5e3bbfb2eb4807c5aa2303c7b52f1d539899572b674bd8fb4a529354e8087067a4e374e92b855a7ec523295867f310f7cea5d48bb0295a1c3ea47b6c4229f000d0c83ae22eb449656c25083a5fc99f99a23cf25fc2020941ffb392b3c8788c25b687b21610d35e3a5f12974b20778144d6930772381a694be5605f21142ca2121a7358e377fd20d50ff00ae3550aa2622ac8d8a642c41aec117b74c22732b297dbb4e4637595bb5b7d5a69414c7822e55efb9c40ddbc2a293532ee49c705e9bb2f3fff3f2dcce8c72b90c486b3a750ee594fd2b04c2dd9553e02a226001956bed24790349d243ca5d51167c4a348d19e42bf2cabc72c0510b8217e0ce5997b763fd11a881c8757f565e3ca3766fc5c3e011ad40d634666859a3a5fdbab6fb627c5abd1d80894424c5a1b0de74cfdf5eb17b9c8bb873f4515adc3e9d62d76f4f5bbb0ef453e369ad5ffa853ab6661f2bf242d91d098f5c59ceb4c1151b47f4cd5ee9dc98daca45898168f4ff69fb230d01d7660d0718cc1b6facbb7f8e9a73cb6a8b93aba7dddc450628588fbb916641abaab4a228bc2584950e34f86a8154b4f5773d07a4ad70b9c2fdff3d16933011664e8de0298c61cd2da7511b8a0cd63acf47156bd7f667929fce5b7257582f336683abf609ad7a4b9dca4395849d81da230d13822d01224d49f4d249d9881d2e876335824514448300000000000003a00af4916a22e0177947732d2d58db7e0325d004f1de5718b3275ae009845fa53ce6948ddd9623e445c8433d30c162c9a22b05a80d417ac2efc2bf684e091bc26a81bda1ec9253ed4657c2440710623741007bdb90e9d94121babea7484d25a49432e71b55250be2a5c8ac7716ec2c61218cca98458ae648e11d31c185d20e1eb21300e7b360f1553a128ff97d54c1510a26178421859bef1dae17e4cf496acc39b20059e48b5c94c7ab244dbfc368d12868af7cac467d8c18e8cb5b60c22b7e9151e013dd28b813254e65a73c24c0efae47b29cdc3a34286dabb6fd09d9ad8b121655394d272934013773667eae726cdbab0b42cfc2c0bd988ecacafead6f179046fa3a4f5fb8ca7a790e5685f262c0e770c20a700fdab00aeac3a8634d84102f750996b5323356ad1b3199702c2d23a64279726f35e0eb630f91794dda905093995d6b457782792564be257a191f2bdeedd01e76051f7aa740735b8c408c618b3b6edac93f7504d441855d89adb6e13ed2a60c81ac0a0b445a2d1447bc30f0f30376ab55995ad0b639c1dfddb3e29e8f8ddccb59e55d863117ca6bc1c8dbf42983ea889cd03b19ab7c76addddb6dbdd53856b92df8cc929f3d37eb27ba6ab68194fc55f9ca2943acd265538751335cdbf527675494d176282e59a9c8deb95645b5b2b8660456434701b182e08915dbb6de5bfacc5480e867a9f08c8fe2d209eaa44175912ce5fb782425e264587d8ad13a70eb83d0ac0b02095a8f6d029d588b80b4f7a96c73720344247eae9613a9ff6ca368dbd823c225304725a36aee0c1822fe554733bc074c50ea296ff6b60c8711f7ef216e1ff1b532b398ba13e6da8ebd4546dcf296a4c1f7b27da5ec6e771d23afc139af51d5e9b5f2be9f32a7ab949e86cb5158c0f61b5a31f81e9c9ddc8412fcdf971a7cc296e742bcae1d444928152841a927dd7985bb7ef06c1fcfa9a1b908e6b0f52a38321f325f4bff3af4064ce8266c93821bb81f9a0404ffff81520ff370ba1b831cae5db58cb943e59dac65a9092db9fb12b00424c49dc0b92356a5b2801c40b43f8d5262278b44eb980e5857397b6ec9570dde9ca80b5ab041bf81872f3f999182589771a75c8ef0e3b8603888dade9b1d449adaac0f08c7cddbe85df0871722ded236de4b7e5f999ad3f2c8e01cab184eaec5f2fa35b22586e7838e4dde010a37ff24927c8a4845794ee699ab54b57ba50e038008af5ea0bbaf3f4e627fe03809f973980b81ced08a602b7458158ab7684376535265761dc039bb13d01ff90e732b8df17d783338db830193034d48dca8ed42b077293db0aa1da1b14c0c4c0e76c7859b832a226464e2cad01aa41bf8fd1686933b55fd961ab336cdb697e851bf3b6c1142019475e0b2925d6727e15de30cbf7b342daf509778d2af676c53a3e962ddff846c310d768e00000000000003a058998e2c08c59208820948b33b051910a3156d3beff7984798c030f971904a9085c61447ded4a76575b5fcc40b5acd93b1d0b17ba74231d94f3df0f666e751067fdc9364a3a5d07ce0881d44fabf4cf0f478a2ec3d14a668e3e5f249d57138890a5db2846b458f82e77256f2f4b3f1412b0434831aad2b96fed8c9ed2e520a5c54ddfb9f72ee0932eea6ab7a58b2f14c5ef062717e9d1da9c7f213dd11bed65bd7693398c732e08b6fe96095c68e4d7e2d3378c3632e736358e88db56bad7329bfd83bac50c0649fcf4f8baf1f95a3b16a004a02b8d0a469b687ee7f91743232a46ae0eb27d0f45d18dcbc04aeb9595995802a4ddf42f2712248d38d5b7656916211ecc6bbfb4c62b6f9d3393061ead919662e53f356bf2327536dccedd7eee28b06514de978fcc965a3d6d507aba3b3af78e3ac52e3141e9fc1b51302616138f2e14ba340d1baea0b2d3d65e21fc94545f17e956292b4cfa8eaecdfa46344a88eb15f9e230a384caefaced24f6553f5ba7c7e41f9c9bbec71a9a5d8a33b994b6d1ce62a1334351057838c2fbe0aa3a204c17fb11e1970f36a8ef3a16d634890a98dee795c197ad586a5c0cf5a23e72d5931481ddf33a342c5bcc320965b98b46bfc2c51a106ce2dedc8f7196b1af0f30825854d5a34d7fc398aea4548e13d0289fe76ec71f4d9531e56d44d39562ca349fd9b62321e28e7d1c1c406199b5dd4841e12d1c649e9e3dd7db84c8499f61621c188fcf56da4915001bc82a8b8aaafa7001486df8dc215d02261640dfce04c79ba5d66ea5daec4fe8a7f2e39527c8c5ca8d8e05fe00e9f909d49a852a9c2ab96f185aa63ccd5e542b0355d8ee66b252a5a57ab0b30c704ea8055eea246d5700c0aadf23520a445e70b331b24fb8f8c43739403cba21d87a6a63e05cddfcf6934230c4f9420b796b70df0eb70f0c1a94c09c70b7e1bb6e2494fe4ae028c358a9048404e31e6191749b83f8cc942fc796459ce1669ea7c268d3c55cb2be9e8ed54400cd831c974fe59d63b043963e98330c7179038c3be277bcb8d9713234a48a7f2e84bdc41c6f822e7292a02407c9966834fc014727d1aa86ca08f39d5fb53b745bb16d2eb11f23af7955b1a5507ff670dc39d8bd929a2ac8ad2e72ed27d36c386223672d434474f9149d39079bd72977f573bee8686584eddcb1467c24c16a38b0e6f6a9c727f7cca2b7a7a86589fa507438ee0bdaa383ddb95ac2d3abe8d07fe06dc4980bbf6423d7496d4283f85cfc9dc79091eb8695d583ebf31a6aad857a6510e3bf268b634ec62ee7112be92a89633f394792332d8e0f5fcff45353bf8e55db172dae5d60dacd372c1e47f9ce8d497f7a18438fe6a3930c240b29ab1d91176ebd5934d23230b629f46f294f9926d4f33c5ba2c1cee78df2b3bb2d43c67423751cfd78ec652596dec00a06db900000000000003a0d28d2cf0c4c23a0ae40ab3a4a29ed7eaa11a3957bf5ee7b8e61c4f3858982f5fe8f40bb19cf728f76121c2cd51ab9a970000000000000003e8c9885501863a2462cf63cd2d4109746aeda1643d286fd6fb13014bef689dc66b34b6e5a6f83e88aead415c06737aaee5eb3c7fda86f635b3345c72329088cbae5eb375e33f880727993c4c01b736f93fc1520f3fa1b41a8c8eb7002dd6a1a996535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe8800000000000000011ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b158a686df05f9cb8c9f01f6499b27d21c499d31ad7d8274c4dff49192e73c184018a600ba8fc0e3734dd643706ae2af8922fffa4574a630845a4a91f0ce23518000000000000000125cf08c3ee49b669e5cfb0ad13de8e19691287addf6b5228a78fc769ce76c74e0000000000000001975d70bd51172f6374af4a6caef9cf309579710d8de8544924a4b3e1dea66268cbb03d16b627871f3953107611f586177d5513c8577e1f1d6fe32db8e15652c342c3962968a07e584544049e05cfcc50692e5dc7dbb8eaf492c6b8191083f1a363b6ab807e54f04f4a7d0dd03097354d733baa72c8c74fd4705afdca689209f566a27c010f60cc64000000000000002886ec356465a6f1f026d858b4379d94d26c85b519261f1aec7ef751dbb7047e0a0d4c5c554d6c0859d3f81ffc8c67d5923165c636f32ee6cf3aee685d84a38cd411a0007ab583cc18985830be0c634b132890055ab792751ac4154645f5fc0a00697d0b202f3bc1ed194875eb59dcacc7482ce06feb934e8409ba391e56537c02b599c54b173e9ac920d9f14af16cfa9649bd35c0d4b1425f35f9c2f49f7c441b9c02bc2eb97e318b76aa4dabc55a76f40000000000000020c0f8550862860085c4f90ac96b053dfa355be3de55a341c24ea20c3ae0050e4649bbf5aa9b06c678717d7582308bd68aafdf11a379bc34056f56c762e11d1340239b777df26c543ff1e61fd0066e842400000000";
}
