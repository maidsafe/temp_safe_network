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
use sn_dbc::{rng, Owner, OwnerOnce, RingCtTransaction, SpentProof, TransactionBuilder};
use sn_interface::types::Token;
use std::collections::{BTreeMap, BTreeSet};

const WALLET_TYPE_TAG: u64 = 1_000;

/// Set of spendable DBCs mapped to their friendly name as defined/chosen by the user when
/// depositing DBCs into a wallet.
pub type WalletSpendableDbcs = BTreeMap<String, (Dbc, EntryHash)>;

impl Safe {
    /// Create an empty wallet and return its XOR-URL.
    ///
    /// A wallet is stored on a private register.
    pub async fn wallet_create(&self) -> Result<XorUrl> {
        let xorurl = self.multimap_create(None, WALLET_TYPE_TAG).await?;

        let mut safeurl = SafeUrl::from_url(&xorurl)?;
        safeurl.set_content_type(ContentType::Wallet)?;

        Ok(safeurl.to_string())
    }

    /// Deposit a DBC in a wallet to make it a spendable balance.
    ///
    /// A name can optionally be specified for the deposit. If it isn't, the hash of the DBC
    /// content will be used.
    ///
    /// Returns the name that was set.
    pub async fn wallet_deposit(
        &self,
        wallet_url: &str,
        spendable_name: Option<&str>,
        dbc: &Dbc,
        secret_key: Option<bls::SecretKey>,
    ) -> Result<String> {
        // TODO: check the input DBCs were spent and all other sort of verifications,
        // perhaps all optional, and we may want a separate API to also do these verifications
        // for the user to perform them without depositing the DBC into a wallet.
        let dbc_to_deposit = if dbc.is_bearer() {
            if secret_key.is_some() {
                return Err(Error::DbcDepositError(
                    "A secret key should not be supplied when depositing a bearer DBC".to_string(),
                ));
            }
            dbc.clone()
        } else if let Some(sk) = secret_key {
            let mut owned_dbc = dbc.clone();
            owned_dbc.to_bearer(&sk).map_err(|e| {
                if e.to_string()
                    .contains("supplied secret key does not match the public key")
                {
                    Error::DbcDepositInvalidSecretKey
                } else {
                    Error::DbcDepositError(e.to_string())
                }
            })?;
            owned_dbc
        } else {
            return Err(Error::DbcDepositError(
                "A secret key must be provided to deposit an owned DBC".to_string(),
            ));
        };

        let safeurl = self.parse_and_resolve_url(wallet_url).await?;

        let spendable_name = match spendable_name {
            Some(name) => name.to_string(),
            None => hex::encode(dbc.hash()),
        };

        self.insert_dbc_into_wallet(&safeurl, &dbc_to_deposit, spendable_name.clone())
            .await?;

        debug!(
            "A spendable DBC deposited into wallet at {}, with name: {}",
            safeurl, spendable_name
        );

        Ok(spendable_name)
    }

    /// Fetch a wallet from a Url performing all type of URL resolution required.
    /// Return the set of spendable DBCs found in the wallet.
    pub async fn wallet_get(&self, wallet_url: &str) -> Result<WalletSpendableDbcs> {
        let safeurl = self.parse_and_resolve_url(wallet_url).await?;
        debug!("Wallet URL was parsed and resolved to: {}", safeurl);
        self.fetch_wallet(&safeurl).await
    }

    /// Fetch a wallet from a SafeUrl without performing any type of URL resolution
    pub(crate) async fn fetch_wallet(&self, safeurl: &SafeUrl) -> Result<WalletSpendableDbcs> {
        let entries = match self.fetch_multimap(safeurl).await {
            Ok(entries) => entries,
            Err(Error::AccessDenied(_)) => {
                return Err(Error::AccessDenied(format!(
                    "Couldn't read wallet found at \"{}\"",
                    safeurl
                )))
            }
            Err(Error::ContentNotFound(_)) => {
                return Err(Error::ContentNotFound(format!(
                    "No wallet found at {}",
                    safeurl
                )))
            }
            Err(err) => {
                return Err(Error::ContentError(format!(
                    "Failed to read balances from wallet: {}",
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
                    warn!("Ignoring entry found in wallet since it cannot be deserialised as a valid DBC: {:?}", err);
                    continue;
                }
            };

            let spendable_name = std::str::from_utf8(key)?.to_string();
            balances.insert(spendable_name, (dbc, *entry_hash));
        }

        Ok(balances)
    }

    /// Check the total balance of a wallet found at a given XOR-URL
    pub async fn wallet_balance(&self, wallet_url: &str) -> Result<Token> {
        debug!("Finding total wallet balance for: {}", wallet_url);

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
                    warn!("Ignoring amount from DBC found in wallet due to error in revealing secret amount: {:?}", err);
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

    /// Reissue a DBC from a wallet and return the output DBC.
    ///
    /// If you pass `None` for the `owner_public_key` argument, the output DBC will be a bearer. If
    /// the public key is specified, the output DBC will be owned by the person in possession of the
    /// secret key corresponding to the public key.
    ///
    /// If there is change from the transaction, the change DBC will be deposited in the source
    /// wallet.
    ///
    /// Spent DBCs are marked as removed from the source wallet, but since all entries are kept in
    /// the history, they can still be retrieved if desired by the user.
    pub async fn wallet_reissue(
        &self,
        wallet_url: &str,
        amount: &str,
        owner_public_key: Option<bls::PublicKey>,
    ) -> Result<Dbc> {
        debug!(
            "Reissuing DBC from wallet at {} for an amount of {} tokens",
            wallet_url, amount
        );

        let output_amount = parse_tokens_amount(amount)?;
        if output_amount.as_nano() == 0 {
            return Err(Error::InvalidAmount(
                "Output amount to reissue needs to be larger than zero (0).".to_string(),
            ));
        }

        let safeurl = self.parse_and_resolve_url(wallet_url).await?;
        let spendable_dbcs = self.fetch_wallet(&safeurl).await?;

        // We'll combine one or more input DBCs and reissue:
        // - one output DBC for the recipient,
        // - and a second DBC for the change, which will be stored in the source wallet.
        let mut input_dbcs_to_spend = Vec::<Dbc>::new();
        let mut input_dbcs_entries_hash = BTreeSet::<EntryHash>::new();
        let mut total_input_amount = 0;
        let mut change_amount = output_amount;
        for (name, (dbc, entry_hash)) in spendable_dbcs.into_iter() {
            let dbc_balance = match dbc.amount_secrets_bearer() {
                Ok(amount_secrets) => Token::from_nano(amount_secrets.amount()),
                Err(err) => {
                    warn!("Ignoring input DBC found in wallet (entry: {}) due to error in revealing secret amount: {:?}", name, err);
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
            .reissue_dbcs(
                input_dbcs_to_spend,
                output_amount,
                change_amount,
                owner_public_key,
            )
            .await?;

        if let Some(change_dbc) = change_dbc {
            self.insert_dbc_into_wallet(&safeurl, &change_dbc, "change-dbc".to_string())
                .await?;
        }

        // (virtually) remove input DBCs in the source wallet
        self.multimap_remove(&safeurl.to_string(), input_dbcs_entries_hash)
            .await?;

        Ok(output_dbc)
    }

    ///
    /// Private helpers
    ///

    /// Insert a DBC into the wallet's underlying `Multimap`.
    async fn insert_dbc_into_wallet(
        &self,
        safeurl: &SafeUrl,
        dbc: &Dbc,
        spendable_name: String,
    ) -> Result<()> {
        if !dbc.is_bearer() {
            return Err(Error::InvalidInput("Only bearer DBC's are supported at this point by the wallet. Please deposit a bearer DBC's.".to_string()));
        }

        let dbc_bytes = Bytes::from(rmp_serde::to_vec_named(dbc).map_err(|err| {
            Error::Serialisation(format!(
                "Failed to serialise DBC to insert it into the wallet: {:?}",
                err
            ))
        })?);

        let dbc_xorurl = self.store_bytes(dbc_bytes, None).await?;

        let entry = (spendable_name.into_bytes(), dbc_xorurl.into_bytes());
        let _entry_hash = self
            .multimap_insert(&safeurl.to_string(), entry, BTreeSet::default())
            .await?;

        Ok(())
    }

    /// Reissue DBCs and log the spent input DBCs on the network. Return the output DBC and the
    /// change DBC if there is one.
    async fn reissue_dbcs(
        &self,
        input_dbcs: Vec<Dbc>,
        output_amount: Token,
        change_amount: Token,
        public_key: Option<bls::PublicKey>,
    ) -> Result<(Dbc, Option<Dbc>)> {
        let output_owner = if let Some(pk) = public_key {
            let owner = Owner::from(pk);
            OwnerOnce::from_owner_base(owner, &mut rng::thread_rng())
        } else {
            let owner = Owner::from_random_secret_key(&mut rng::thread_rng());
            OwnerOnce::from_owner_base(owner, &mut rng::thread_rng())
        };

        // TODO: enable the use of decoys
        let mut tx_builder = TransactionBuilder::default()
            .set_decoys_per_input(0)
            .set_require_all_decoys(false)
            .add_inputs_dbc_bearer(input_dbcs.iter())?
            .add_output_by_amount(output_amount.as_nano(), output_owner);

        let client = self.get_safe_client()?;
        let change_owneronce =
            OwnerOnce::from_owner_base(client.dbc_owner(), &mut rng::thread_rng());
        if change_amount.as_nano() > 0 {
            tx_builder =
                tx_builder.add_output_by_amount(change_amount.as_nano(), change_owneronce.clone());
        }

        let mut dbc_builder = tx_builder.build(&mut rng::thread_rng())?;

        // Build the output DBCs
        // Spend all the input DBCs, collecting the spent proof shares for each of them
        let client = self.get_safe_client()?;

        let spent_proofs: Vec<SpentProof> = input_dbcs
            .iter()
            .flat_map(|dbc| dbc.spent_proofs.clone())
            .collect();

        let spent_transactions: Vec<RingCtTransaction> = input_dbcs
            .iter()
            .flat_map(|dbc| dbc.spent_transactions.clone())
            .collect();

        for (keyimage, tx) in dbc_builder.inputs() {
            // TODO: spend DBCs concurrently spawning tasks
            client
                .spend_dbc(
                    keyimage,
                    tx.clone(),
                    spent_proofs.clone(),
                    spent_transactions.clone(),
                )
                .await?;
            let spent_proof_shares = client.spent_proof_shares(keyimage).await?;

            dbc_builder = dbc_builder
                .add_spent_proof_shares(spent_proof_shares.into_iter())
                .add_spent_transaction(tx);
        }

        // TODO: Perform the verification of the transaction and spentproofs for input DBCs,
        // as well as building the output DBCs.
        let dbcs = dbc_builder.build_without_verifying()?;

        let mut output_dbc = None;
        let mut change_dbc = None;
        for (dbc, owneronce, _) in dbcs {
            if change_owneronce == owneronce && change_amount.as_nano() > 0 {
                change_dbc = Some(dbc);
            } else {
                output_dbc = Some(dbc);
            }
        }

        match output_dbc {
            None => Err(Error::DbcReissueError(
                "Unexpectedly failed to generate output DBC. No balance were spent from the wallet.".to_string(),
            )),
            Some(dbc) => Ok((dbc, change_dbc)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_helpers::{
        new_read_only_safe_instance, new_safe_instance, new_safe_instance_with_dbc_owner,
    };
    use anyhow::{anyhow, Result};
    use sn_dbc::Owner;

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
    async fn test_wallet_deposit_with_bearer_dbc() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-dbc"), &dbc, None)
            .await?;

        let wallet_balances = safe.wallet_get(&wallet_xorurl).await?;
        assert_eq!(wallet_balances.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_deposit_with_name() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-dbc"), &dbc, None)
            .await?;

        let wallet_balances = safe.wallet_get(&wallet_xorurl).await?;
        assert!(wallet_balances.contains_key("my-dbc"));

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_deposit_with_no_name() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        let name = safe
            .wallet_deposit(&wallet_xorurl, None, &dbc, None)
            .await?;

        let wallet_balances = safe.wallet_get(&wallet_xorurl).await?;
        assert!(wallet_balances.contains_key(&name));

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_deposit_with_owned_dbc() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;
        let pk = bls::PublicKey::from_hex(
            "aa12ba9055367b2274c38073af13ace42310e1a13a948d73f7dee09d10bdabec4629082a1321d41e123212c47e0908e5",
        )?;
        let sk = bls::SecretKey::from_hex(
            "18f5b51fafeaa74b50f2324c2c721e6facf524e3b8dbd0e67e5e4a794e64d84e",
        )?;

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-dbc"), &dbc, None)
            .await?;
        let owned_dbc = safe
            .wallet_reissue(&wallet_xorurl, "2.35", Some(pk))
            .await?;
        safe.wallet_deposit(
            &wallet_xorurl,
            Some("owned-dbc"),
            &owned_dbc,
            Some(sk.clone()),
        )
        .await?;

        let owner = Owner::from(sk);
        let balances = safe.wallet_get(&wallet_xorurl).await?;
        let (owned_dbc, _) = balances
            .get("owned-dbc")
            .ok_or_else(|| anyhow!("Couldn't read DBC from wallet"))?;
        assert_eq!(*owned_dbc.owner_base(), owner);

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_deposit_with_owned_dbc_without_providing_secret_key() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;
        let pk = bls::PublicKey::from_hex(
            "aa12ba9055367b2274c38073af13ace42310e1a13a948d73f7dee09d10bdabec4629082a1321d41e123212c47e0908e5",
        )?;

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-dbc"), &dbc, None)
            .await?;
        let owned_dbc = safe
            .wallet_reissue(&wallet_xorurl, "2.35", Some(pk))
            .await?;
        let result = safe
            .wallet_deposit(&wallet_xorurl, Some("owned-dbc"), &owned_dbc, None)
            .await;
        match result {
            Ok(_) => Err(anyhow!(
                "This test case should result in an error".to_string()
            )),
            Err(Error::DbcDepositError(e)) => {
                assert_eq!(e, "A secret key must be provided to deposit an owned DBC");
                Ok(())
            }
            Err(_) => Err(anyhow!("This test should use a DbcDepositError".to_string())),
        }
    }

    #[tokio::test]
    async fn test_wallet_deposit_with_owned_dbc_with_invalid_secret_key() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;
        let sk = bls::SecretKey::random();
        let sk2 = bls::SecretKey::random();
        let pk = sk.public_key();

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-dbc"), &dbc, None)
            .await?;
        let owned_dbc = safe
            .wallet_reissue(&wallet_xorurl, "2.35", Some(pk))
            .await?;
        let result = safe
            .wallet_deposit(&wallet_xorurl, Some("owned-dbc"), &owned_dbc, Some(sk2))
            .await;
        match result {
            Ok(_) => Err(anyhow!(
                "This test case should result in an error".to_string()
            )),
            Err(Error::DbcDepositInvalidSecretKey) => Ok(()),
            Err(_) => Err(anyhow!(
                "This test should use a DbcDepositInvalidSecretKey error".to_string()
            )),
        }
    }

    #[tokio::test]
    async fn test_wallet_deposit_with_bearer_dbc_and_secret_key() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;
        let sk = bls::SecretKey::from_hex(
            "18f5b51fafeaa74b50f2324c2c721e6facf524e3b8dbd0e67e5e4a794e64d84e",
        )?;

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        let result = safe
            .wallet_deposit(&wallet_xorurl, Some("my-dbc"), &dbc, Some(sk))
            .await;
        match result {
            Ok(_) => Err(anyhow!(
                "This test case should result in an error".to_string()
            )),
            Err(Error::DbcDepositError(e)) => {
                assert_eq!(
                    e,
                    "A secret key should not be supplied when depositing a bearer DBC"
                );
                Ok(())
            }
            Err(_) => Err(anyhow!("This test should use a DbcDepositError".to_string())),
        }
    }

    #[tokio::test]
    async fn test_wallet_reissue_with_deposited_owned_dbc() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;
        let wallet2_xorurl = safe.wallet_create().await?;
        let pk = bls::PublicKey::from_hex(
            "aa12ba9055367b2274c38073af13ace42310e1a13a948d73f7dee09d10bdabec4629082a1321d41e123212c47e0908e5",
        )?;
        let sk = bls::SecretKey::from_hex(
            "18f5b51fafeaa74b50f2324c2c721e6facf524e3b8dbd0e67e5e4a794e64d84e",
        )?;

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-dbc"), &dbc, None)
            .await?;
        let owned_dbc = safe
            .wallet_reissue(&wallet_xorurl, "2.35", Some(pk))
            .await?;
        // Deposit the owned DBC in another wallet because it's easier to ensure this owned DBC
        // will be used as an input in the next reissue rather than having to be precise about
        // balances.
        safe.wallet_deposit(
            &wallet2_xorurl,
            Some("owned-dbc"),
            &owned_dbc,
            Some(sk.clone()),
        )
        .await?;

        let result = safe.wallet_reissue(&wallet2_xorurl, "2", None).await;
        match result {
            Ok(_) => {
                // For this case, we just want to make sure the reissue went through without an
                // error, which means the owned DBC was used as an input. There are other test
                // cases that verify balances are correct and so on, we don't need to do that again
                // here.
                Ok(())
            }
            Err(e) => Err(anyhow!(e)),
        }
    }

    #[tokio::test]
    async fn test_wallet_balance() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        // We deposit the first DBC with 12.23 amount
        let dbc1 = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc1, None)
            .await?;

        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::from_nano(12_230_000_000));

        // ...and a second DBC with 1.53
        let dbc2 = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-second-dbc"), &dbc2, None)
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
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc1, None)
            .await?;
        let dbc2 = new_dbc(DBC_WITH_MAX)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-second-dbc"), &dbc2, None)
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
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc1, None)
            .await?;
        let dbc2 = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-second-dbc"), &dbc2, None)
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

    /// Ignoring until we implement encryption support again.
    #[ignore]
    #[tokio::test]
    async fn test_wallet_get_not_owned_wallet() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc, None)
            .await?;

        // test it fails to get a not owned wallet
        let read_only_safe = new_read_only_safe_instance().await?;
        match read_only_safe.wallet_get(&wallet_xorurl).await {
            Err(Error::AccessDenied(msg)) => {
                assert_eq!(
                    msg,
                    format!("Couldn't read wallet found at \"{}\"", wallet_xorurl)
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
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc, None)
            .await?;

        // We insert an entry (to its underlying data type, i.e. the Multimap) which is
        // not a valid serialised DBC, thus making part of its content incompatible/corrupted.
        let corrupted_dbc_xorurl = safe.store_bytes(Bytes::from_static(b"bla"), None).await?;
        let entry = (b"corrupted-dbc".to_vec(), corrupted_dbc_xorurl.into_bytes());
        safe.multimap_insert(&wallet_xorurl, entry, BTreeSet::default())
            .await?;

        // Now check the Wallet can still be read and the corrupted entry is ignored
        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::from_nano(1_530_000_000));

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_reissue_with_multiple_input_dbcs() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("deposited-dbc-1"), &dbc, None)
            .await?;
        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("deposited-dbc-2"), &dbc, None)
            .await?;

        let output_dbc = safe.wallet_reissue(&wallet_xorurl, "2.35", None).await?;

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
    async fn test_wallet_reissue_with_single_input_dbc() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("deposited-dbc-1"), &dbc, None)
            .await?;

        let output_dbc = safe.wallet_reissue(&wallet_xorurl, "1", None).await?;

        let output_balance = output_dbc
            .amount_secrets_bearer()
            .map_err(|err| anyhow!("Couldn't read balance from output DBC: {:?}", err))?;
        assert_eq!(output_balance.amount(), 1_000_000_000);

        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::from_nano(530_000_000));

        let wallet_balances = safe.wallet_get(&wallet_xorurl).await?;

        assert_eq!(wallet_balances.len(), 1);

        let (change_dbc_read, _) = wallet_balances
            .get("change-dbc")
            .ok_or_else(|| anyhow!("Couldn't read change DBC from fetched wallet"))?;
        let change = change_dbc_read
            .amount_secrets_bearer()
            .map_err(|err| anyhow!("Couldn't read balance from change DBC fetched: {:?}", err))?;
        assert_eq!(change.amount(), 530_000_000);

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_reissue_with_persistent_dbc_owner() -> Result<()> {
        let (safe, dbc_owner) = new_safe_instance_with_dbc_owner(
            "3917ad935714cf1e71b9b5e2831684811e83acc6c10f030031fe886292152e83",
        )
        .await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("deposited-dbc-1"), &dbc, None)
            .await?;

        let _ = safe.wallet_reissue(&wallet_xorurl, "1", None).await?;
        let wallet_balances = safe.wallet_get(&wallet_xorurl).await?;

        let (change_dbc_read, _) = wallet_balances
            .get("change-dbc")
            .ok_or_else(|| anyhow!("Couldn't read change DBC from fetched wallet"))?;
        assert_eq!(*change_dbc_read.owner_base(), dbc_owner);

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_reissue_with_owned_dbc() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("deposited-dbc-1"), &dbc, None)
            .await?;

        let pk = bls::PublicKey::from_hex(
            "a9e3531c4d5ce128410fe0a8d3963e492daf8a5854174f7a3d67cb1b2e4c80d7\
            1be56533298a533a2b0712a4a006f648",
        )?;
        let owner = Owner::from(pk);
        let output_dbc = safe.wallet_reissue(&wallet_xorurl, "1", Some(pk)).await?;

        // We have verified transaction details in other tests. In this test, we're just concerned
        // with the owner being assigned correctly.
        assert_eq!(owner, *output_dbc.owner_base());

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_not_enough_balance() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_1_530_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("deposited-dbc"), &dbc, None)
            .await?;

        match safe.wallet_reissue(&wallet_xorurl, "2.55", None).await {
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

        match safe.wallet_reissue(&wallet_xorurl, "0", None).await {
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
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc, None)
            .await?;

        // We insert an entry (to its underlying data type, i.e. the Multimap) which is
        // not a valid serialised DBC, thus making part of its content incompatible/corrupted.
        let corrupted_dbc_xorurl = safe.store_bytes(Bytes::from_static(b"bla"), None).await?;
        let entry = (b"corrupted-dbc".to_vec(), corrupted_dbc_xorurl.into_bytes());
        safe.multimap_insert(&wallet_xorurl, entry, BTreeSet::default())
            .await?;

        // Now check we can still reissue from the wallet and the corrupted entry is ignored
        let _ = safe.wallet_reissue(&wallet_xorurl, "0.4", None).await?;
        let current_balance = safe.wallet_balance(&wallet_xorurl).await?;
        assert_eq!(current_balance, Token::from_nano(1_130_000_000));

        Ok(())
    }

    #[tokio::test]
    async fn test_wallet_reissue_all_balance() -> Result<()> {
        let safe = new_safe_instance().await?;
        let wallet_xorurl = safe.wallet_create().await?;

        let dbc = new_dbc(DBC_WITH_12_230_000_000)?;
        safe.wallet_deposit(&wallet_xorurl, Some("my-first-dbc"), &dbc, None)
            .await?;

        // Now check thaat after reissuing with the total balance,
        // there is no change deposited in the wallet, i.e. wallet is empty with 0 balance
        let _ = safe.wallet_reissue(&wallet_xorurl, "12.23", None).await?;

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
        safe.wallet_deposit(&wallet1_xorurl, Some("deposited-dbc"), &dbc, None)
            .await?;

        let output_dbc = safe.wallet_reissue(&wallet1_xorurl, "0.25", None).await?;

        safe.wallet_deposit(&wallet2_xorurl, Some("reissued-dbc"), &output_dbc, None)
            .await?;

        let balance = safe.wallet_balance(&wallet2_xorurl).await?;
        assert_eq!(balance, Token::from_nano(250000000));

        Ok(())
    }

    const DBC_WITH_MAX: &str = "38aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e18707ea919914faca624361a4cf56c46c5af7311712a4ce792b2c4b3d0dd6fa7e0e5739d453d82f9cd7957421539b6ce3df184007606ccc21ec98fcde61af1ff83a9a1836eb943a8951b5dc368662efdfcccfbefff5cc368353ebb6ea06217dfc317a28312d0436d55d8c210fcb1bb35295dcbe4b1f43e932c630f99b7629f83725efa04b8b5de60ad9c59e6f6ca02f865fcaa78f39f6245122ef72a9a594ece893bdcad0ed9c6d4ef70dbde8e0b6c674eaebb054dc2bae1057718f09d7272c74c322c200f360ac1e593cb4cd9386fff1a680b6334a5c553f25aeb8d5a78b2137e3beb3c686b1141ccfe6303b08882cf564cadab4d29bca09c7c9544ed427003a91d7cd080343cd4cc87e7ba621b0bad37e384428439d0bb0ece88bde3ee3018adb116f40b633c38418a34d6100baa97d84f76d09ede201b47e36a163e8d59f8724188b3f993d73d6ee05f98e665c7368012c9e10f5ca94dabf7b11c71816039a93e0d0ca221b4d69d1c3ce0afc416b8c05e8b49870eba331310eedfb4b866269b99cc1d0ab8ee3e8a3b574fe2bbbd100b882868cfffe28db4ae1ef9345e7bc94d3babdb3abe3708b7913d69c2a9d766f176093c7809358a5876d8aa36cafd671b87a8ff2a8edb14ad93d584ce65c867ac523a91164e2a6320fa22efa817915959a98e4af659c305b5a2f209341caed64a7e22d5e38811d3ad33cfb5252286b126061691e2b47d2f83e15131a0456562449812e0c4590b5ce2556ef9e07113f55aab3c518c74b8cf3703536de6fc83e775d5203b343af00322512fa97584c4522b8715b48b2d428dec09c2a87a5d2577fb9ea0a895fcd2b86a1d251e30b39e50a10596ddef7e61ab01b624b66fbd1528314628d483999ea0d457a625498817bc9b0215808e77d7f544fcd115d80b520d426990093797183e3bd8b70ce540f20045432f7ee915bf993fc1ddfeb2377b2accad1444174cafc74b6eb78fe764bca0d203b05231581f1cec51cc8d9c8557fc3743db85e126570964526c1c36fabedc8f7f7f9cb871895759777311242957e7ffafdcadde45381fb3ebaa89971361667b59c7d6b858be1aeb0eb7ba7dd643c3cb2ca63f9689ea6d3dcfd9c1503e5e6f264c9b2243a11bcf019be5c43902dd850fda28c9d88144d19a6df48a7672ae33b962ee24cbcc9d826f32e9b9ffbf4ea28932027a5b1ff6338142c68ba3353376735244e07dc49a1f92ca55b625a167dd298cbbdb3a3855b8636fb27178b72015b40789257e72f3e452860756392ec9b2173cb4c8f551193b5d6275c937de915379100000000000003a096535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe88000000000000000138aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e1875ec7f615f506c5b54a6f2c581b53054eb27f064c3216970f98165341f1f5e25e7508f4d421c3dba24ff7adbe61d61da910f652f9d986fe9649d2937a0f631c5c8d1a922817b5637b263ac7642066e81df5a8c4e8ecb51168fd2349c71534d4ab0000000000000001bd546d885fd355619bd1a226c6cc743f40c939d39634f6bcc15a1a590e40d597d7c9a08e2346cbe7c06e1a514abedc933dfe1a98a93b51d4c1da98fb7f71c82689cb90a92b4b882d2b7ebbfd63f167e82be854194397cba8f0a87a5fcd07848220c26efbccdc78006f7abb47d90d0f5600000000000000010454acf22d354960e1234aa7878e260bfbc16960378064edecba8edcf7245e860000000000000001000000000000000132dd1edfdd2d7f584f9a882ba078e6fb5879a84e53c1f1dae0062e69a3307665b2773e00f271ee271cb1fff854de7e199684bf5ab0bb8fe4ef9221aaadc18713afd9363a7771ab659c1d3210e77240018d8cd8df379a90612900914216b86f8c1b40aa44f2c4c949bc78449a4a56088212bec11ecc3805aff1a4026d3faa4b59d17dc2c6cd4debbbb971872d4198bab463b8f891d8d98d5c0c509e62408b354eb18b21c417e7b51b043d52bfc2ed6b444b1b7b00f7e5a359ac712dab253404970000000000000001fb0aa04f1d01b1bdb5e033db74c8fa8ce0afa6a98992ffd5a6639bde4fc772c1bd546d885fd355619bd1a226c6cc743f40c939d39634f6bcc15a1a590e40d597d7c9a08e2346cbe7c06e1a514abedc93000000000000000138aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e18707ea919914faca624361a4cf56c46c5af7311712a4ce792b2c4b3d0dd6fa7e0e5739d453d82f9cd7957421539b6ce3df184007606ccc21ec98fcde61af1ff83a9a1836eb943a8951b5dc368662efdfcccfbefff5cc368353ebb6ea06217dfc317a28312d0436d55d8c210fcb1bb35295dcbe4b1f43e932c630f99b7629f83725efa04b8b5de60ad9c59e6f6ca02f865fcaa78f39f6245122ef72a9a594ece893bdcad0ed9c6d4ef70dbde8e0b6c674eaebb054dc2bae1057718f09d7272c74c322c200f360ac1e593cb4cd9386fff1a680b6334a5c553f25aeb8d5a78b2137e3beb3c686b1141ccfe6303b08882cf564cadab4d29bca09c7c9544ed427003a91d7cd080343cd4cc87e7ba621b0bad37e384428439d0bb0ece88bde3ee3018adb116f40b633c38418a34d6100baa97d84f76d09ede201b47e36a163e8d59f8724188b3f993d73d6ee05f98e665c7368012c9e10f5ca94dabf7b11c71816039a93e0d0ca221b4d69d1c3ce0afc416b8c05e8b49870eba331310eedfb4b866269b99cc1d0ab8ee3e8a3b574fe2bbbd100b882868cfffe28db4ae1ef9345e7bc94d3babdb3abe3708b7913d69c2a9d766f176093c7809358a5876d8aa36cafd671b87a8ff2a8edb14ad93d584ce65c867ac523a91164e2a6320fa22efa817915959a98e4af659c305b5a2f209341caed64a7e22d5e38811d3ad33cfb5252286b126061691e2b47d2f83e15131a0456562449812e0c4590b5ce2556ef9e07113f55aab3c518c74b8cf3703536de6fc83e775d5203b343af00322512fa97584c4522b8715b48b2d428dec09c2a87a5d2577fb9ea0a895fcd2b86a1d251e30b39e50a10596ddef7e61ab01b624b66fbd1528314628d483999ea0d457a625498817bc9b0215808e77d7f544fcd115d80b520d426990093797183e3bd8b70ce540f20045432f7ee915bf993fc1ddfeb2377b2accad1444174cafc74b6eb78fe764bca0d203b05231581f1cec51cc8d9c8557fc3743db85e126570964526c1c36fabedc8f7f7f9cb871895759777311242957e7ffafdcadde45381fb3ebaa89971361667b59c7d6b858be1aeb0eb7ba7dd643c3cb2ca63f9689ea6d3dcfd9c1503e5e6f264c9b2243a11bcf019be5c43902dd850fda28c9d88144d19a6df48a7672ae33b962ee24cbcc9d826f32e9b9ffbf4ea28932027a5b1ff6338142c68ba3353376735244e07dc49a1f92ca55b625a167dd298cbbdb3a3855b8636fb27178b72015b40789257e72f3e452860756392ec9b2173cb4c8f551193b5d6275c937de915379100000000000003a096535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe88000000000000000138aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e1875ec7f615f506c5b54a6f2c581b53054eb27f064c3216970f98165341f1f5e25e7508f4d421c3dba24ff7adbe61d61da910f652f9d986fe9649d2937a0f631c5c8d1a922817b5637b263ac7642066e81df5a8c4e8ecb51168fd2349c71534d4ab0000000000000001bd546d885fd355619bd1a226c6cc743f40c939d39634f6bcc15a1a590e40d597d7c9a08e2346cbe7c06e1a514abedc933dfe1a98a93b51d4c1da98fb7f71c82689cb90a92b4b882d2b7ebbfd63f167e82be854194397cba8f0a87a5fcd07848220c26efbccdc78006f7abb47d90d0f5600000000000000010454acf22d354960e1234aa7878e260bfbc16960378064edecba8edcf7245e86000000000000000136ae9da857dea71fdb4f000b81134898c25e6d4b54faceb08bf189c033c2c77486926ccfb32d8493b768395bb53b1a11d9aab58ce4d44b0ee9c5262aae352ba10c399cdc40190c6979d88b24c9e7c5d2207dfe00d182d7cb9718fedb12f894a5eaff1f79c85de783d03625f485b2f016958008fbacb83ff173ed2c13884e6c6c7c28272baa6dae50000000000000002824ac824164239da4fe37315ebaeb79a3200e6f975588760c44a41a7ed3d228101099928f754cac7d07328767a9c031a8c7fc854ed016530c3461bb0c0b0a939a3d2943780faa6d730cc0918a9c1d19228ae41ae4517d40ff7fc371f8c2fa0919cb8d55c251c5ce2649bee18c2d83e81b024d1db22fa2b561ce6d9c49f11396ed4075c5577e6b8727fc09b1e4f7c172a617b7b33e020749bec26d255afa3e702a3c200b316076d0fc913657682e65ff500000000000000020270c372504454a1d33e91e2f8aeb5080c8c2f202d1f2e7f5ad7762bca65a9d8f6605e572228f7c657fe368dfb84cd3a4c16551530000000000000000000000000000000000000000000000000000000000000000";

    const DBC_WITH_12_230_000_000: &str = "ee787589d84b6a09c0230310c9d661177c6efd4ed3b079f3baff803dfe4e7a03816175f292d53f4999f7836ce43114954a40af5585871ff5fa106eb5a857b441c2caf8b4fa8f2d69fd36782be853f48d5e2dfafc9f4dcf5eda0c53b39b44030d7f6c96f2307c759b323035a50d94ff27dd92026c7620d30318cfeba2873c65622eac23674eb82f481738f09d4e82868a00c0a7e751a4ce8e03bc0979417f6588307970085a7537633d7616d21549b416c2fb1d6e0a31bc7767022d502984857664cfa9da97fd01b7619401cabfc0e1b43a2e0f47e84c929d31182214788609da5b5c2e1a9ef26717616d63592f8b93de51f6d1c3d3284a91ffa9c69d0702f6b7cf90aecad7d07b063bd6933801b5152574aee9513bc1d91a798d1d2687266f96c736b52f381583b55a958c9790ce7696fb1e58f792126ae8a69f24103a1e2767d7fbf1da4e9ceb2dfb676e39a21a232b1921c5a156e551d7d66d2c3f59c664b575a49cc6db4cc6e46fc46b40beab0b9239d55a4e2f5931b2978a199ab4e429a9c29f6c6e765341db4c9bb22cbd5837a4b6c488f7d8c130c785f42a8ada556763c48f8885156e106e18e14b32efb074b4d921e1f16f605ecd7263a27734d2658ebcf6e08c70db76159d6227142c2b2b1f5c629ef75efd1648c96e8f84b51a985fd27bd737b93b9d55263d3fa460794bb990095966724a4c1acd1f9d1dff4f1f7e39e63003e80659785062c5ed7e37921f6acea34859f77994811c90e152ba2e80457cd3599ed64da1f5da06d4664cc85f4428611fa2673ea25487cd4b965bf9cdec0d55c9bc45924d7b04298c4b3756ae6cff29082bd1b6f58cb690520559b246b0ede184c951b60023ade9d3660e465cbb4d7021cab1dfe6ad2351e887eb938be624b7ac84215d0965e67e9d9e36d6d773fc1e7aa8f68f2f66cfee16b8d826ca50e79576f146039284c78959006329810339dbd6f411b5d350f127da5bd0d9d0ad8045b32097a8bc45f9885f2940a6482119cb26d5107a482419a4307bc989983d79620c385286f66805865a8a68b09259dbda4e6d4ec907d44cf0285415cc362c6eb260eabd634f562bb836a03167c2e0faf7bb268dd37eb24d8e4de8a9172d2946d85121a07564e02db54b8d71b1f7d94d7da5b53b6b0abc86a8555a119c946ce4d24501c97567523caf078bc6858274695dfcfc8505f0439d423b73177eaa3fd47e73cfa92a0b0a3629ba6475368701c7df62264cd313b2430f9291f4f3936a9c0e704b1da31c0991d933c00d643e3195412738b0874a7e769c2a1393f0b380aa5a15d2b1e0cb2c29f5d733f0365b12fe4a4303a4bc3fccbd3e5c98c444b96ed922a200af35d6ac1f5b285485338800000000000003a01d56a59164605850ca63ecd9d68a781b071833a5bfe981b28bdf96f7363641ec5e596f34605a502ead75135c5f020ea4890817dd8463365d6b00faa71e2ecb00ab7a32d9b0198695dc35c13ac48969873ebc26403dce0a7414de4519774bc1b51b153a7779cb29a7d722553406ab078dd8f4ca602f39be2bf4aacdef93c51d87233c372e62a7f204e05c73aa9d1dcec57e076347288f4facb30cc521033da0041dabd2752299e85545f52eba607b32f7d224b0262ffd24d430a758ce2898f38ff795d193d54ec5af2f934400b72d548485a285568c47bb0d02a15f1a1e39f1c92c9d45c32d5b1ebcc25c0bddd51f7de3b518bf20dd035ea944e7dc97007565a1a096fc386685e0cf98a6de5e7fa2ad6807a44c060ebd438a54192706348e2caba3fc831b2ca960e601405c3bf69f27a561bda1e54166531683f05af3cd2982db50efa915a1b891ce2e9c6fd5c9b9d553c6987d3a7084a383e65b1fc57bf60491eff48db81213f9d528d004b8e3b72ea5a411c214fec52132e53f8cd2425fea9548f77cdbb1421d078ea25f92094923a94b1157016eed6c0f9cc7d97f3b76de22b105a1bcc378991127585a389d710c2b2caf5bb169e089c810671f03e94b74b769a473d277cc3194f7964bb7403cd993fbf01db3d50c98594ca8e425834444959b26b427ed255f55306db5204c6093b66629ab4b83b99ef4316e5795d0999a3017e102c99d57beac28fb791ea877dda9894ef32dd9db1f3f52960d0e7ad01b86d903c3b67c6e868ee75935eebe988689f8c6f8552a6db3cbdfa857c2fa9b2e88d68a11243da67627de3d78500e40d0aa0086b512d2465d5f1d7c36586e6a69119445e4c2def64c2e84c9bcb20267766f08cf2572c6ee01deb82c6282e384d7adb95f8dd1f395ad62fc03c7206e0c381237ff86ab493dacf7c4d73b01597b675d52979ef58b1e8e454e1302bc6d662c8979aa6e3872e9d8714b1546de055e07a072048233f2671732c67afb75f5b6b537d2de02bdc0bf0d6b68d084448fff75b13ce4d8adb56ebd2e02e583331e018e85a4b880979acc2f680e11b1e1a397c645157b750d97dc47b50ce0692d2c3d4aff634d81d67a14d2b8d403f6dde097683e116cc5a192f8b4e184aff8fc57e5b236df978077f6f3d052d8207e6aa27b0c06e562af543258c358a9b60e14bb78505933cc5b2beddb9f2bba54d08eb1bd48b9a03955a3519654f91c0aba98f686168570cd11cf4f10da3e2690b388327775313965deb4f764d3387645e85678b2ab5a584c1e6990072016cc4953af882bcb98c116085ca8db54a482724cb982e6350041722c8cef1bc97c7ba7d8bd87170fd4a45c7c5320a6a305f2c0369bf686de898cae9cc24a108423331baf5d1518122ce140ecd2749f54cff563ee7a0fd124ac4c24f8c1e5db057c943a4065bed5ec8800000000000003a068eecbebca0cce0f1de3a3d82dc1c9d757eb053f11f718bdbc5802d4e50ae602573b73b5f3a4a5bb8a02c6229109858800000000000000020d93917b310e05b6052ccb08d618b7f9ba840b44ef2248c6f3a167660dc0e2ca02a8577c436aaa2396206f01c9216c8f7aae0faee99f2b22bee78e790a3f7c7b9fe5dafe1e328c84ecd5ce1a8398528e01b7fe096d7e0af681b6e24e656f24a496535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe8800000000000000011ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b32d5b5cb4c4214606830338c1a941a7f8cc416a6a62fea720b9de6af2e3e51b22c166aa90ffefe7e3c51dcdbad8fa1377d86cfdb01baada35d620519d9f33bcd0000000000000001630a9a5331b8cc59a1902a4fdfc9da818730c22d29b61f3bc22c3eb3b8a16e03000000000000000100000000000000015f74debf101b87e707d9a77ed7a8a6b016303435dbbf076ad51d7e5f005bdd0dbe0d820d8341b93f9244dac38e12d60794d5618189cf6277a014abadfd794728e5ac043408982030ed04f320593f6bf3f097a3297d23745f17b460a2504f77ab65e38eb4bcab5bd8d8059641aa65188f6d2412dd0ae4676aa28114500eea0b5eae8919efc0f2d57a166e33ca3ada06b738aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e1870000000000000001b6d944cd99586d869ea8ad5fdac2f3c8aab7d5aec1e01cd7336d95cba8fb396b1ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b0000000000000001ee787589d84b6a09c0230310c9d661177c6efd4ed3b079f3baff803dfe4e7a03816175f292d53f4999f7836ce43114954a40af5585871ff5fa106eb5a857b441c2caf8b4fa8f2d69fd36782be853f48d5e2dfafc9f4dcf5eda0c53b39b44030d7f6c96f2307c759b323035a50d94ff27dd92026c7620d30318cfeba2873c65622eac23674eb82f481738f09d4e82868a00c0a7e751a4ce8e03bc0979417f6588307970085a7537633d7616d21549b416c2fb1d6e0a31bc7767022d502984857664cfa9da97fd01b7619401cabfc0e1b43a2e0f47e84c929d31182214788609da5b5c2e1a9ef26717616d63592f8b93de51f6d1c3d3284a91ffa9c69d0702f6b7cf90aecad7d07b063bd6933801b5152574aee9513bc1d91a798d1d2687266f96c736b52f381583b55a958c9790ce7696fb1e58f792126ae8a69f24103a1e2767d7fbf1da4e9ceb2dfb676e39a21a232b1921c5a156e551d7d66d2c3f59c664b575a49cc6db4cc6e46fc46b40beab0b9239d55a4e2f5931b2978a199ab4e429a9c29f6c6e765341db4c9bb22cbd5837a4b6c488f7d8c130c785f42a8ada556763c48f8885156e106e18e14b32efb074b4d921e1f16f605ecd7263a27734d2658ebcf6e08c70db76159d6227142c2b2b1f5c629ef75efd1648c96e8f84b51a985fd27bd737b93b9d55263d3fa460794bb990095966724a4c1acd1f9d1dff4f1f7e39e63003e80659785062c5ed7e37921f6acea34859f77994811c90e152ba2e80457cd3599ed64da1f5da06d4664cc85f4428611fa2673ea25487cd4b965bf9cdec0d55c9bc45924d7b04298c4b3756ae6cff29082bd1b6f58cb690520559b246b0ede184c951b60023ade9d3660e465cbb4d7021cab1dfe6ad2351e887eb938be624b7ac84215d0965e67e9d9e36d6d773fc1e7aa8f68f2f66cfee16b8d826ca50e79576f146039284c78959006329810339dbd6f411b5d350f127da5bd0d9d0ad8045b32097a8bc45f9885f2940a6482119cb26d5107a482419a4307bc989983d79620c385286f66805865a8a68b09259dbda4e6d4ec907d44cf0285415cc362c6eb260eabd634f562bb836a03167c2e0faf7bb268dd37eb24d8e4de8a9172d2946d85121a07564e02db54b8d71b1f7d94d7da5b53b6b0abc86a8555a119c946ce4d24501c97567523caf078bc6858274695dfcfc8505f0439d423b73177eaa3fd47e73cfa92a0b0a3629ba6475368701c7df62264cd313b2430f9291f4f3936a9c0e704b1da31c0991d933c00d643e3195412738b0874a7e769c2a1393f0b380aa5a15d2b1e0cb2c29f5d733f0365b12fe4a4303a4bc3fccbd3e5c98c444b96ed922a200af35d6ac1f5b285485338800000000000003a01d56a59164605850ca63ecd9d68a781b071833a5bfe981b28bdf96f7363641ec5e596f34605a502ead75135c5f020ea4890817dd8463365d6b00faa71e2ecb00ab7a32d9b0198695dc35c13ac48969873ebc26403dce0a7414de4519774bc1b51b153a7779cb29a7d722553406ab078dd8f4ca602f39be2bf4aacdef93c51d87233c372e62a7f204e05c73aa9d1dcec57e076347288f4facb30cc521033da0041dabd2752299e85545f52eba607b32f7d224b0262ffd24d430a758ce2898f38ff795d193d54ec5af2f934400b72d548485a285568c47bb0d02a15f1a1e39f1c92c9d45c32d5b1ebcc25c0bddd51f7de3b518bf20dd035ea944e7dc97007565a1a096fc386685e0cf98a6de5e7fa2ad6807a44c060ebd438a54192706348e2caba3fc831b2ca960e601405c3bf69f27a561bda1e54166531683f05af3cd2982db50efa915a1b891ce2e9c6fd5c9b9d553c6987d3a7084a383e65b1fc57bf60491eff48db81213f9d528d004b8e3b72ea5a411c214fec52132e53f8cd2425fea9548f77cdbb1421d078ea25f92094923a94b1157016eed6c0f9cc7d97f3b76de22b105a1bcc378991127585a389d710c2b2caf5bb169e089c810671f03e94b74b769a473d277cc3194f7964bb7403cd993fbf01db3d50c98594ca8e425834444959b26b427ed255f55306db5204c6093b66629ab4b83b99ef4316e5795d0999a3017e102c99d57beac28fb791ea877dda9894ef32dd9db1f3f52960d0e7ad01b86d903c3b67c6e868ee75935eebe988689f8c6f8552a6db3cbdfa857c2fa9b2e88d68a11243da67627de3d78500e40d0aa0086b512d2465d5f1d7c36586e6a69119445e4c2def64c2e84c9bcb20267766f08cf2572c6ee01deb82c6282e384d7adb95f8dd1f395ad62fc03c7206e0c381237ff86ab493dacf7c4d73b01597b675d52979ef58b1e8e454e1302bc6d662c8979aa6e3872e9d8714b1546de055e07a072048233f2671732c67afb75f5b6b537d2de02bdc0bf0d6b68d084448fff75b13ce4d8adb56ebd2e02e583331e018e85a4b880979acc2f680e11b1e1a397c645157b750d97dc47b50ce0692d2c3d4aff634d81d67a14d2b8d403f6dde097683e116cc5a192f8b4e184aff8fc57e5b236df978077f6f3d052d8207e6aa27b0c06e562af543258c358a9b60e14bb78505933cc5b2beddb9f2bba54d08eb1bd48b9a03955a3519654f91c0aba98f686168570cd11cf4f10da3e2690b388327775313965deb4f764d3387645e85678b2ab5a584c1e6990072016cc4953af882bcb98c116085ca8db54a482724cb982e6350041722c8cef1bc97c7ba7d8bd87170fd4a45c7c5320a6a305f2c0369bf686de898cae9cc24a108423331baf5d1518122ce140ecd2749f54cff563ee7a0fd124ac4c24f8c1e5db057c943a4065bed5ec8800000000000003a068eecbebca0cce0f1de3a3d82dc1c9d757eb053f11f718bdbc5802d4e50ae602573b73b5f3a4a5bb8a02c6229109858800000000000000020d93917b310e05b6052ccb08d618b7f9ba840b44ef2248c6f3a167660dc0e2ca02a8577c436aaa2396206f01c9216c8f7aae0faee99f2b22bee78e790a3f7c7b9fe5dafe1e328c84ecd5ce1a8398528e01b7fe096d7e0af681b6e24e656f24a496535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe8800000000000000011ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b32d5b5cb4c4214606830338c1a941a7f8cc416a6a62fea720b9de6af2e3e51b22c166aa90ffefe7e3c51dcdbad8fa1377d86cfdb01baada35d620519d9f33bcd0000000000000001630a9a5331b8cc59a1902a4fdfc9da818730c22d29b61f3bc22c3eb3b8a16e03000000000000000167d1153fd164ca0ea3c1e2e89133ff09560d00de7d279eb39fe474b68da1e1336ffade2e011700148ce4cd6e352a160d07421e69077962a102d1ca6230637e4e8592ce345e86e0fcc51b62f87b33c441fa618448e206762851585a24db5815b1a15b0e8912db828956a303f91fa82edac8b02df064b2ed111926cdfc437aede140f2497eed1aaf8c00000000000000280f6a383f7677d9adc41a11f716550ebf32074fbea7aaf02ce85aa5a96d1ddf85e2a8b1c640767e7d952aae6c04a42999e25e455e894b8b8abc03c417f0d382963f47465306ceeada07416c279b43c2c167c2a0ca9adfcd37078dfa383a829402f667f82f426c5c4bc731baef134cb0f973f977065698345d7ac748098022f18fc8449add772c9c978993c48055b6fba9e37308c9edd1010952846610da2df9bdec54d85c2d43e7aaf65ca83c5441380400000000000000208c198334c661452d01fc3fb19f31319537f790457f80e3bd2e0a8bcb84cdb6d266490c43c98a59dfc3a8aff74e673e89299fa246893bc7d67b6ea6f1b4a7de9e531354ec832dd73dd36494da0d21bd2800000000";

    const DBC_WITH_1_530_000_000: &str = "5b27e8998542c6ae461c20bbb764da84b16721c795fa5ec73db3d109a68dcdded655d1c1ed7d2106ac1d12558049bab64581076215747dbbff95397a32a3d3848ceb318cf6dd5b371a2e2e910e0697972fb69d93e07de0d4387c3e4dfa2d59bdf91debc36b3bc8c45c3fa390e9bbb492ba54cdaca5bd94544a56f8d209b8876fa3e5eeef1e9d624a4b65c2627983dfbc3ef0f2cb1b815c3748052525fb7bdab933a5fdfc39d7dac1f657bd63f3c64d9e7601e031455e5b49479aa82c87c6cd944bba03423f7099c695593a94247b64a5bb32eccc0ad9fdbb89fb278d415a382761a130301e29d5673635b459b7932f2454d2e64e0489adc4a037e0b5bd6f9793fe52c8fba9405d0ef7eed48a296f9e070ec6961484490788bf629f2151bddd6097f63dd53274cd0df1693e96b8d3179619a05259fb25c7912520468a0abd1731535bdecf6b4f5497429dda47268d24f9f203eebac6978739a5d0d91358f84bb1f64712c83e8ed825fd1beaa06d63189fdfada90df84705f959681b4b34d58f8843dbe98bff97f87a3df4d235859c75b4642dec1566cd196f01d9665aa24597131c3c36bf5893a27136fd6fbb1c5b14c435c6914e9340b7ea5c522c834137c9b9eae762481905a04514e4ced0f048daedf7298f9ae16588f799e527963c9c7be9c89505652b62d0cf035a24ff6ab4fcaa41e9e19b217750ca2a2e6a23e14c4b54511dac6823a4e90ff077c447c941ffe75d6ddbc91939a7dbd6e0d98b01c1b0a8cb9bef1675e22939d113f23e4f245003e82051c5e4d6a37cfb87e4baf3e185f9fc4d11a7df03a191e9689eb9c07aa9ad831789577bc12446c65c29ebc7b16c022bad0e2fb1b96ba77785c6fcf60babd7c559445a42396e0f515efdf44f6058b7dfdad0345b748c4ed5ed3bd6b44e1056d54a35d05c1227db3dd194c64b30c6555622351ace8198bfcc47b57e7f7b3699032746711350a17e74207613b4395e58b892e0f1eb258ccffbc9d44f520216496fd8999d432a9a9825854e71dfe242da51f6ed909431ac766ca6e421318337425d5cb0a8794c2fa062a6575615049ad9359b493b72b51e7d5c54e23e521fd82698deffe4d9968120be1921e8288930d15d298abe9527757aecc35d87e7ec98f9ce68193e966e32274e0dd4be7dc4eda480fb9ed581053f51451e4fcd44ec292c42f9c23d40016409a097100674745fcda19603a9d4bcb3990e641ade096d7db2edc340fb0e63eca1d4a7a5e4fd2f6d9bba29f5fb69cb2053d403982672689126045e946fedd54ad97b71f9c908e40e91aa430fc12b42dcd3cd46ccac89e45828ec770bdb7963332e4afca6b2e79fd5ce4ab681cbf214e2d1b5f97f30ef1b379400000000000003a022f7884d31afa7d6290ac827610f8d1b3ac85b23d772aad211432020dc6ef7fb3b16caff655e8033f23a5a02e70a14853a153d1f310223795eedd09b7174755e42cf9ab2450e1dd5e913e5939017a2e288d521d9abfbdced62f2139aaa9bd1a030f012e75822f069d9efb8b06594c2ab3cf51631b8451bfe1289e39ae3263ca965d7dd887ca70a8d4a24fa740ea6737ef0989e55298c08c861ddea293c2112f154cc68813af354ca1e376b87b6b95504b7373fe142fddbc9bd649236bef035d1f75c14fe2b50b580ad993df6a4e5e6b855ecfa165e660beefe0b1160f2b8aebea8458cb3b4bffa99b49ec364ce39dacc8adbc388fe5c519c3428ff33e8a3238c7c17bbd748c7412432b6ca1937d02991b094e8f1b96f30df799bae9a9959d922543100fb32eb35f206ab6f875e1058b6aaaeb57e4e878fcf01d8d700c5ce619cc0c856635d1ebba1c0f031eee8a7fc4a5587a3c1acea24f6c5bbf67e8a366098091452e3412cf42360840395016ebfc882d49a9b5986e59995521b14c0645d586f4ca9afd58b90680f9f7dcf304156aeee1bf0a07517d7f12d5e1124d7fd14429214598a15b6096bac99fa59d8b8243a228953eba44f4e4df22a369f52a72da4b91fae2fb0803074f453f5f8c7ab586caeac5805f2891de74a3c5cfadc4b5bf425ae802f067f6a5a54f7564309e74fb938ca81610f88cde01aaa999306474fb49ca31994619c1c76351487efef558f8c8dd8140c9de805f40c9c8b8b152b55a5b0c61b5ff26d194f39aec49659790a8b761c837b6acb377e790798b2ff1fede7d460c146b62159c25ed50a5f58a683941668e8b849065668ecf4e380165f3029e64686d8b1f0f8ec53d4914da9612805272153be3e3e10855e7b81a4914c90ab770d72859a442055b0a3c143a435f45758ba5650da7e85cbe1f99fc04a17f993b2d092163a22ec397b01a7e37dddee80fc81b1a5244c5b4e4d06d28dd7e0e09ab2892f0b902fbc085d715a9380ac1181817975c7fa1a0df62997d2c69270f9aa2e6215c55fd92143d064119d1b283e58c46b5ebaf36ca9f1a99e10b4952423b705fea91fb697d40e5caeb1f6ca2f164dea7bd9571bf810e8aba02ce522319bfb1eacf38dc1dfb953b47833191bb695eef377f44be0753dd8abde493e413008a37ec5ebdce084d0e2cff8348f3e4208af9ed6605398f0ab2975a5d0547c67b59b9969991bcea289370e4989c749b31e8469c76f643e2cdc5bf07e7f0e4b34374cd29a305933a482667782ce7d94041fabc74c5acd5868ea15824eeb89ade841b541ad49c28a4fc10183ed1118b5db3ba997c5635fa0e4f7ed0580280e8184e3ae743f286c9d7a048a64c0dd262c80769915b66169bf7a1f1c52bbcbb508d8bcfcd9b0c392e01a638f30cb7def3c74ed3bd5de7e794a71630d8055fa10ba858f8200000000000003a07f7a21e7168519441c7c08df4df085df9016351e4f001d6bc5df9bff21e570470a34546aa7b733fdcd338c547bae02810000000000000002f5435dd67154de0c0d36d338d1282a35bfb31ccefca42a4786aac034115fc76fe6a39deb556ee1cb58682b8216ce989235f556ccfbfb57b54f658262c8b2748b32e076d5e81cd8837f2b86900ca7b27f586955006b54b10ed304ee7c4fd8318996535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe8800000000000000011ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b548303a73305fc9c94eada44863144d84b6f9088a6280ff1b54cf540dc6c4ded457121351f13df5ffd94cb9764079dede3220daca7d85f61d5092eaca480b7b500000000000000013f80a95efa16af78eceb4d770980e57c9f2ef4939db4fd6233e71179b281c30c0000000000000001000000000000000140bf4b37d26b0d348e8b6971ae02f1b8f4c45464d730cff075dba8dc16b3ebe83bdd0967eca3217ec4a66448829271052ede75124f072c1428102aa508a9cd83b4ee63f52d19764068efe798e3d96ae8004e2496ed626daa8fc3277c1b2aeb981f6d0adfbd03542f93053fa4056d0ce36018ebb704258d130a797471ce3a0d1802c05a7f578a1bf4851975549b2bf0b038aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e1870000000000000001fa3523c0c909f6104727a3148f3469aedf9626c9421c030dd52e65dc057914211ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b00000000000000015b27e8998542c6ae461c20bbb764da84b16721c795fa5ec73db3d109a68dcdded655d1c1ed7d2106ac1d12558049bab64581076215747dbbff95397a32a3d3848ceb318cf6dd5b371a2e2e910e0697972fb69d93e07de0d4387c3e4dfa2d59bdf91debc36b3bc8c45c3fa390e9bbb492ba54cdaca5bd94544a56f8d209b8876fa3e5eeef1e9d624a4b65c2627983dfbc3ef0f2cb1b815c3748052525fb7bdab933a5fdfc39d7dac1f657bd63f3c64d9e7601e031455e5b49479aa82c87c6cd944bba03423f7099c695593a94247b64a5bb32eccc0ad9fdbb89fb278d415a382761a130301e29d5673635b459b7932f2454d2e64e0489adc4a037e0b5bd6f9793fe52c8fba9405d0ef7eed48a296f9e070ec6961484490788bf629f2151bddd6097f63dd53274cd0df1693e96b8d3179619a05259fb25c7912520468a0abd1731535bdecf6b4f5497429dda47268d24f9f203eebac6978739a5d0d91358f84bb1f64712c83e8ed825fd1beaa06d63189fdfada90df84705f959681b4b34d58f8843dbe98bff97f87a3df4d235859c75b4642dec1566cd196f01d9665aa24597131c3c36bf5893a27136fd6fbb1c5b14c435c6914e9340b7ea5c522c834137c9b9eae762481905a04514e4ced0f048daedf7298f9ae16588f799e527963c9c7be9c89505652b62d0cf035a24ff6ab4fcaa41e9e19b217750ca2a2e6a23e14c4b54511dac6823a4e90ff077c447c941ffe75d6ddbc91939a7dbd6e0d98b01c1b0a8cb9bef1675e22939d113f23e4f245003e82051c5e4d6a37cfb87e4baf3e185f9fc4d11a7df03a191e9689eb9c07aa9ad831789577bc12446c65c29ebc7b16c022bad0e2fb1b96ba77785c6fcf60babd7c559445a42396e0f515efdf44f6058b7dfdad0345b748c4ed5ed3bd6b44e1056d54a35d05c1227db3dd194c64b30c6555622351ace8198bfcc47b57e7f7b3699032746711350a17e74207613b4395e58b892e0f1eb258ccffbc9d44f520216496fd8999d432a9a9825854e71dfe242da51f6ed909431ac766ca6e421318337425d5cb0a8794c2fa062a6575615049ad9359b493b72b51e7d5c54e23e521fd82698deffe4d9968120be1921e8288930d15d298abe9527757aecc35d87e7ec98f9ce68193e966e32274e0dd4be7dc4eda480fb9ed581053f51451e4fcd44ec292c42f9c23d40016409a097100674745fcda19603a9d4bcb3990e641ade096d7db2edc340fb0e63eca1d4a7a5e4fd2f6d9bba29f5fb69cb2053d403982672689126045e946fedd54ad97b71f9c908e40e91aa430fc12b42dcd3cd46ccac89e45828ec770bdb7963332e4afca6b2e79fd5ce4ab681cbf214e2d1b5f97f30ef1b379400000000000003a022f7884d31afa7d6290ac827610f8d1b3ac85b23d772aad211432020dc6ef7fb3b16caff655e8033f23a5a02e70a14853a153d1f310223795eedd09b7174755e42cf9ab2450e1dd5e913e5939017a2e288d521d9abfbdced62f2139aaa9bd1a030f012e75822f069d9efb8b06594c2ab3cf51631b8451bfe1289e39ae3263ca965d7dd887ca70a8d4a24fa740ea6737ef0989e55298c08c861ddea293c2112f154cc68813af354ca1e376b87b6b95504b7373fe142fddbc9bd649236bef035d1f75c14fe2b50b580ad993df6a4e5e6b855ecfa165e660beefe0b1160f2b8aebea8458cb3b4bffa99b49ec364ce39dacc8adbc388fe5c519c3428ff33e8a3238c7c17bbd748c7412432b6ca1937d02991b094e8f1b96f30df799bae9a9959d922543100fb32eb35f206ab6f875e1058b6aaaeb57e4e878fcf01d8d700c5ce619cc0c856635d1ebba1c0f031eee8a7fc4a5587a3c1acea24f6c5bbf67e8a366098091452e3412cf42360840395016ebfc882d49a9b5986e59995521b14c0645d586f4ca9afd58b90680f9f7dcf304156aeee1bf0a07517d7f12d5e1124d7fd14429214598a15b6096bac99fa59d8b8243a228953eba44f4e4df22a369f52a72da4b91fae2fb0803074f453f5f8c7ab586caeac5805f2891de74a3c5cfadc4b5bf425ae802f067f6a5a54f7564309e74fb938ca81610f88cde01aaa999306474fb49ca31994619c1c76351487efef558f8c8dd8140c9de805f40c9c8b8b152b55a5b0c61b5ff26d194f39aec49659790a8b761c837b6acb377e790798b2ff1fede7d460c146b62159c25ed50a5f58a683941668e8b849065668ecf4e380165f3029e64686d8b1f0f8ec53d4914da9612805272153be3e3e10855e7b81a4914c90ab770d72859a442055b0a3c143a435f45758ba5650da7e85cbe1f99fc04a17f993b2d092163a22ec397b01a7e37dddee80fc81b1a5244c5b4e4d06d28dd7e0e09ab2892f0b902fbc085d715a9380ac1181817975c7fa1a0df62997d2c69270f9aa2e6215c55fd92143d064119d1b283e58c46b5ebaf36ca9f1a99e10b4952423b705fea91fb697d40e5caeb1f6ca2f164dea7bd9571bf810e8aba02ce522319bfb1eacf38dc1dfb953b47833191bb695eef377f44be0753dd8abde493e413008a37ec5ebdce084d0e2cff8348f3e4208af9ed6605398f0ab2975a5d0547c67b59b9969991bcea289370e4989c749b31e8469c76f643e2cdc5bf07e7f0e4b34374cd29a305933a482667782ce7d94041fabc74c5acd5868ea15824eeb89ade841b541ad49c28a4fc10183ed1118b5db3ba997c5635fa0e4f7ed0580280e8184e3ae743f286c9d7a048a64c0dd262c80769915b66169bf7a1f1c52bbcbb508d8bcfcd9b0c392e01a638f30cb7def3c74ed3bd5de7e794a71630d8055fa10ba858f8200000000000003a07f7a21e7168519441c7c08df4df085df9016351e4f001d6bc5df9bff21e570470a34546aa7b733fdcd338c547bae02810000000000000002f5435dd67154de0c0d36d338d1282a35bfb31ccefca42a4786aac034115fc76fe6a39deb556ee1cb58682b8216ce989235f556ccfbfb57b54f658262c8b2748b32e076d5e81cd8837f2b86900ca7b27f586955006b54b10ed304ee7c4fd8318996535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe8800000000000000011ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b548303a73305fc9c94eada44863144d84b6f9088a6280ff1b54cf540dc6c4ded457121351f13df5ffd94cb9764079dede3220daca7d85f61d5092eaca480b7b500000000000000013f80a95efa16af78eceb4d770980e57c9f2ef4939db4fd6233e71179b281c30c000000000000000117b4bfcd5437771b00ce5d6fc6f604576e621cd12754539817b1ec4a6923780443e64fff60cd8adbeaca4f1d80e0f50ecdd1a86fb6a266f6eac665ba26afa828bb5badd9844262740d78bd453057a075e5beb72a437f6a6937c941cc7bb9318c52963d0c8fd7eac17088fbafa7a0de5c5703582dadd2df0ad059e859b5c38865edbe24f57ad896eb0000000000000028c931b6d85bc74c955eba7b2da84c3972aaf8131d412dcdef7b127ec7a867d45336d27907b4408369c38f1552a6ec3e840c7bd599224f0278cb63f5f02421dd9fb9ce203f9d818bd164bd3ef114ea1c80072bae0c8d2809c9ead4b4fe6744940a1b15241bd165f974c729fc16599dd5013bbfef692d01a8ff8a1f0b7ac8f03efea80de71961d60e9c2325adeaddcbe28686de4a78b5a713a39e2f786a9e6f638ec8c8d1dbc6ca8f599a114fbc1201e1810000000000000020f763c2828f215f20407616362011799e83c511791fb78a18db13c7f785b83e15659c158be1e6837ce88162954f1c9c892bb869ec22db0154809cc9e2c9ddeada3c47928783051f20f6b56f12127fdb6900000000";
}
