// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;

use crate::{Error, Result};

use sn_dbc::{DbcTransaction, PublicKey, SpentProof, SpentProofShare};
use sn_interface::{
    messaging::data::{
        DataCmd, DataQuery, Error as NetworkDataError, QueryResponse, SpentbookCmd, SpentbookQuery,
    },
    types::SpentbookAddress,
};

use std::collections::BTreeSet;
use xor_name::XorName;

// Maximum number of attempts when retrying a spend DBC operation with updated network knowledge.
const MAX_SPEND_DBC_ATTEMPS: u8 = 5;

impl Client {
    //----------------------
    // Write Operations
    //---------------------

    /// Spend a DBC's public key.
    ///
    /// It's possible that the section processing the spend request will not be aware of the
    /// section keys used to sign the spent proofs. If this is the case, the network will return a
    /// particular error and we will retry. There are several retries because there could be
    /// several keys the section is not aware of, but it only returns back the first one it
    /// encounters.
    ///
    /// When the request is resubmitted, it gets sent along with a proof chain and a signed SAP
    /// that the section can use to update itself.
    #[instrument(skip(self, tx, spent_proofs), level = "debug")]
    pub async fn spend_dbc(
        &self,
        public_key: PublicKey,
        tx: DbcTransaction,
        spent_proofs: BTreeSet<SpentProof>,
        spent_transactions: BTreeSet<DbcTransaction>,
    ) -> Result<()> {
        let mut network_knowledge = None;
        let mut attempts = 1;

        debug!(
            "Attempting DBC spend request. Will reattempt if spent proof was signed \
            with a section key that is unknown to the processing section."
        );
        loop {
            let cmd = SpentbookCmd::Spend {
                public_key,
                tx: tx.clone(),
                spent_proofs: spent_proofs.clone(),
                spent_transactions: spent_transactions.clone(),
                network_knowledge,
            };

            let result = self.send_cmd(DataCmd::Spentbook(cmd)).await;

            if let Err(Error::CmdError {
                source: NetworkDataError::SpentProofUnknownSectionKey(unknown_section_key),
                ..
            }) = result
            {
                debug!(
                    "Encountered unknown section key during spend request. \
                        Will obtain updated network knowledge and retry. \
                        Attempts made: {attempts}"
                );
                if attempts >= MAX_SPEND_DBC_ATTEMPS {
                    error!("DBC spend request failed after {attempts} attempts");
                    return Err(Error::DbcSpendRetryAttemptsExceeded {
                        attempts,
                        public_key,
                    });
                }
                let network = self.session.network.read().await;
                let (proof_chain, _) = network
                    .get_sections_dag()
                    .single_branch_dag_for_key(&unknown_section_key)
                    .map_err(|_| Error::SectionsDagKeyNotFound(unknown_section_key))?;
                let signed_sap = network
                    .get_signed_by_key(&unknown_section_key)
                    .ok_or(Error::SignedSapNotFound(unknown_section_key))?;

                network_knowledge = Some((proof_chain, signed_sap.clone()));
                attempts += 1;
            } else {
                return result;
            }
        }
    }

    //----------------------
    // Read Spentbook
    //---------------------

    /// Return the set of spent proof shares if the provided DBC's public key is spent
    #[instrument(skip(self), level = "debug")]
    pub async fn spent_proof_shares(&self, public_key: PublicKey) -> Result<Vec<SpentProofShare>> {
        let address = SpentbookAddress::new(XorName::from_content(&public_key.to_bytes()));
        let query = DataQuery::Spentbook(SpentbookQuery::SpentProofShares(address));
        let response = self.send_query(query.clone()).await?;
        match response {
            QueryResponse::SpentProofShares(res) => {
                res.map_err(|err| Error::ErrorMsg { source: err })
            }
            other => Err(Error::UnexpectedQueryResponse {
                query,
                response: other,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::test_utils::{
        create_test_client_with, init_logger, read_genesis_dbc_from_first_node,
    };
    use crate::Client;

    use sn_dbc::{rng, DbcTransaction, Hash, OwnerOnce, TransactionBuilder};
    use sn_interface::messaging::data::Error as ErrorMsg;

    use eyre::{bail, Result};
    use std::collections::{BTreeSet, HashSet};
    use tokio::time::Duration;

    const MAX_ATTEMPTS: u8 = 5;
    const SLEEP_DURATION: Duration = Duration::from_secs(3);

    // Number of attempts to make trying to spend inputs when reissuing DBCs
    // As the spend and query cmds are cascaded closely, there is high chance
    // that the first two query attempts could both be failed.
    // Hence the max number of attempts set to a higher value.
    const NUM_OF_DBC_REISSUE_ATTEMPTS: u8 = 5;

    async fn verify_spent_proof_share(
        public_key: bls::PublicKey,
        tx: DbcTransaction,
        client: &Client,
    ) -> Result<()> {
        // The query could be too close to the spend which make adult only accumulated
        // part of shares. To avoid assertion faiure, more attempts are needed.
        let mut attempts = 0;
        loop {
            attempts += 1;

            // Get spent proof shares for the public_key.
            let spent_proof_shares = client.spent_proof_shares(public_key).await?;

            // Note this test could have been run more than once thus the genesis DBC
            // could have been spent a few times already, so we filter
            // the SpentProofShares that belong to the TX we just spent in this run.
            // TODO: once we have our Spentbook which prevents double spents
            // we shouldnt't need this filtering.
            let num_of_spent_proof_shares = spent_proof_shares
                .iter()
                .filter(|proof| proof.content.transaction_hash == Hash::from(tx.hash()))
                .count();

            if (5..=7).contains(&num_of_spent_proof_shares) {
                break Ok(());
            } else if attempts == MAX_ATTEMPTS {
                bail!(
                    "Failed to obtained enough spent proof shares after {} attempts, {} retrieved in last attempt",
                    MAX_ATTEMPTS, num_of_spent_proof_shares
                );
            }

            tokio::time::sleep(SLEEP_DURATION).await;
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_spentbook_spend_dbc() -> Result<()> {
        init_logger();
        let _outer_span = tracing::info_span!("test__spentbook_spend_dbc").entered();

        let (
            client,
            SpendDetails {
                public_key,
                genesis_dbc,
                tx,
            },
        ) = setup(false).await?;

        // Spend the public_key.
        client
            .spend_dbc(
                public_key,
                tx.clone(),
                genesis_dbc.spent_proofs,
                genesis_dbc.spent_transactions,
            )
            .await?;

        verify_spent_proof_share(public_key, tx, &client).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn spentbook_spend_spent_proof_with_invalid_pk_should_return_spentbook_error(
    ) -> Result<()> {
        init_logger();
        let _outer_span = tracing::info_span!(
            "test__spentbook_spend_spent_proof_with_invalid_pk_should_return_spentbook_error"
        )
        .entered();

        let (
            client,
            SpendDetails {
                public_key,
                genesis_dbc,
                tx,
            },
        ) = setup(false).await?;

        // insert the invalid pk to proofs
        let invalid_pk = bls::SecretKey::random().public_key();
        let invalid_spent_proofs = genesis_dbc
            .spent_proofs
            .into_iter()
            .map(|mut proof| {
                proof.spentbook_pub_key = invalid_pk;
                proof
            })
            .collect();

        // Try spend the public_key.
        let result = client
            .spend_dbc(
                public_key,
                tx.clone(),
                invalid_spent_proofs,
                genesis_dbc.spent_transactions,
            )
            .await;

        match result {
            Ok(_) => bail!("We expected an error to be returned"),
            Err(crate::Error::CmdError {
                source: ErrorMsg::InvalidOperation(error_string),
                ..
            }) => {
                let correct_error_str =
                    format!("SpentbookError(\"Spent proof signature {invalid_pk:?} is invalid\"");
                assert!(
                    error_string.contains(&correct_error_str),
                    "A different SpentbookError error was expected for this case. What we got: {error_string:?}"
                );
                Ok(())
            }
            Err(error) => bail!("We expected a different error to be returned. Actual: {error:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn spentbook_spend_spent_proof_with_key_not_in_section_chain_should_return_cmd_error_response(
    ) -> Result<()> {
        init_logger();
        let _outer_span = tracing::info_span!("test__spentbook_spend_spent_proof_with_key_not_in_section_chain_should_return_cmd_error_response").entered();

        let (
            client,
            SpendDetails {
                public_key,
                genesis_dbc,
                tx,
            },
        ) = setup(true).await?; // pass in true, for getting an invalid genesis

        let genesis_dbc_owner_pk = genesis_dbc.owner_base().public_key();

        // Try spend the public_key.
        let result = client
            .spend_dbc(
                public_key,
                tx.clone(),
                genesis_dbc.spent_proofs,
                genesis_dbc.spent_transactions,
            )
            .await;

        match result {
            Ok(_) => bail!("We expected an error to be returned"),
            Err(crate::Error::SectionsDagKeyNotFound(section_key)) => {
                assert_eq!(
                    section_key, genesis_dbc_owner_pk,
                    "We expected {genesis_dbc_owner_pk:?} in the error but got {section_key:?}"
                );
                Ok(())
            }
            Err(error) => bail!("We expected a different error to be returned. Actual: {error:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn spentbook_spend_spent_proofs_do_not_relate_to_input_dbcs_should_return_spentbook_error(
    ) -> Result<()> {
        init_logger();
        let _outer_span = tracing::info_span!("test__spentbook_spend_spent_proofs_do_not_relate_to_input_dbcs_should_return_spentbook_error").entered();

        let (client, SpendDetails { genesis_dbc, .. }) = setup(false).await?;

        // The idea for this test case is to pass the wrong spent proofs and transactions for
        // the public key we're trying to spend. To do so, we reissue `output_dbc_1` from
        // `genesis_dbc`, then reissue `output_dbc_2` from `output_dbc_1`, then when we try to spend
        // `output_dbc_2`, we use the spent proofs/transactions from `genesis_dbc`. This should
        // not be permitted. The correct way would be to pass the spent proofs/transactions
        // from `output_dbc_1`, which was our input to `output_dbc_2`.

        let spend_amount_1 = 10;
        let recipient_owneronce_1 =
            OwnerOnce::from_owner_base(client.dbc_owner().clone(), &mut rng::thread_rng());
        let outputs_1 = vec![(
            sn_dbc::Token::from_nano(spend_amount_1),
            recipient_owneronce_1,
        )];
        let (output_dbcs_1, _change_dbc_1) = reissue_dbcs(
            &client,
            vec![genesis_dbc.clone()],
            outputs_1,
            sn_dbc::Token::from_nano(sn_interface::dbcs::GENESIS_DBC_AMOUNT - spend_amount_1),
        )
        .await?;

        let (output_dbc_1, _output_owneronce_1, _amount_secrects_1) = output_dbcs_1[0].clone();

        let spend_amount_2 = 5;
        let recipient_owneronce_2 =
            OwnerOnce::from_owner_base(client.dbc_owner().clone(), &mut rng::thread_rng());
        let outputs_2 = vec![(
            sn_dbc::Token::from_nano(spend_amount_2),
            recipient_owneronce_2,
        )];
        let (output_dbcs_2, _change_dbc_2) = reissue_dbcs(
            &client,
            vec![output_dbc_1],
            outputs_2,
            sn_dbc::Token::from_nano(spend_amount_1 - spend_amount_2),
        )
        .await?;

        let (output_dbc_2, output_owneronce_2, _amount_secrects_2) = output_dbcs_2[0].clone();

        // Try spend the dbc.
        let pk = genesis_dbc
            .public_key_bearer()
            .expect("unexpectedly failed to get DBC public key");
        let result = client
            .spend_dbc(
                output_owneronce_2.as_owner().public_key(),
                output_dbc_2.transaction.clone(),
                genesis_dbc.spent_proofs.clone(),
                genesis_dbc.spent_transactions,
            )
            .await;

        match result {
            Ok(_) => bail!("We expected an error to be returned"),
            Err(crate::Error::CmdError {
                source: ErrorMsg::InvalidOperation(error_string),
                ..
            }) => {
                let correct_error_str =
                    format!("{:?}", sn_dbc::Error::MissingCommitmentForPubkey(pk));
                assert!(
                    error_string.contains(&correct_error_str),
                    "A different SpentbookError error was expected for this case. What we got: {error_string:?}, expected: {correct_error_str:?}"
                );
                Ok(())
            }
            Err(error) => bail!("We expected a different error to be returned. Actual: {error:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn spentbook_spend_with_random_public_key_should_return_spentbook_error() -> Result<()> {
        init_logger();
        let _outer_span = tracing::info_span!(
            "test__spentbook_spend_with_random_public_key_should_return_spentbook_error"
        )
        .entered();

        let (
            client,
            SpendDetails {
                genesis_dbc, tx, ..
            },
        ) = setup(false).await?;

        // generate the random public key
        let random_public_key = bls::SecretKey::random().public_key();

        // Try spend the random_public_key.
        let result = client
            .spend_dbc(
                random_public_key,
                tx.clone(),
                genesis_dbc.spent_proofs.clone(),
                genesis_dbc.spent_transactions,
            )
            .await;

        match result {
            Ok(_) => bail!("We expected an error to be returned"),
            Err(crate::Error::CmdError {
                source: ErrorMsg::InvalidOperation(error_string),
                ..
            }) => {
                let correct_error_str =
                    format!("SpentbookError(\"There are no commitments for the given public key {random_public_key:?}\"");
                assert!(
                    error_string.contains(&correct_error_str),
                    "A different SpentbookError error was expected for this case. What we got: {error_string:?}"
                );
                Ok(())
            }
            Err(error) => bail!("We expected a different error to be returned. Actual: {error:?}"),
        }
    }

    struct SpendDetails {
        genesis_dbc: sn_dbc::Dbc,
        tx: DbcTransaction,
        public_key: sn_dbc::PublicKey,
    }

    // returns a client which is the owner to the genesis dbc,
    // we can do this since our genesis dbc is currently generated as a bearer dbc, and stored locally
    // so we can fetch that owner key from the first node, and pass it to the client
    async fn setup(invalid_genesis_dbc: bool) -> Result<(Client, SpendDetails)> {
        init_logger();

        let genesis_dbc = if invalid_genesis_dbc {
            let sk_set = bls::SecretKeySet::random(0, &mut rand::thread_rng());
            sn_interface::dbcs::gen_genesis_dbc(&sk_set, &sk_set.secret_key())?
        } else {
            read_genesis_dbc_from_first_node()?
        };
        let dbc_owner = genesis_dbc.owner_base().clone();
        let client = create_test_client_with(None, Some(dbc_owner.clone()), None).await?;

        let genesis_public_key = genesis_dbc.public_key_bearer()?;

        let output_owner = OwnerOnce::from_owner_base(dbc_owner, &mut rng::thread_rng());
        let dbc_builder = TransactionBuilder::default().add_input_dbc_bearer(&genesis_dbc)?;

        let inputs_amount_sum = dbc_builder.inputs_amount_sum();
        let dbc_builder = dbc_builder
            .add_output_by_amount(inputs_amount_sum, output_owner)
            .build(rng::thread_rng())?;

        assert_eq!(dbc_builder.inputs().len(), 1);
        let (public_key, tx) = dbc_builder.inputs()[0].clone();
        assert_eq!(genesis_public_key, public_key);

        Ok((
            client,
            SpendDetails {
                genesis_dbc,
                tx,
                public_key,
            },
        ))
    }

    // Reissue DBCs and log the spent input DBCs on the network. Return the output DBC and the
    // change DBC if there is one.
    async fn reissue_dbcs(
        client: &Client,
        input_dbcs: Vec<sn_dbc::Dbc>,
        outputs: Vec<(sn_dbc::Token, OwnerOnce)>,
        change_amount: sn_dbc::Token,
    ) -> Result<(
        Vec<(sn_dbc::Dbc, OwnerOnce, sn_dbc::AmountSecrets)>,
        Option<sn_dbc::Dbc>,
    )> {
        // TODO: enable the use of decoys
        let mut tx_builder = TransactionBuilder::default()
            .add_inputs_dbc_bearer(input_dbcs.iter())?
            .add_outputs_by_amount(outputs.into_iter().map(|(token, owner)| (token, owner)));

        let change_owneronce =
            OwnerOnce::from_owner_base(client.dbc_owner().clone(), &mut rng::thread_rng());
        if change_amount.as_nano() > 0 {
            tx_builder = tx_builder.add_output_by_amount(change_amount, change_owneronce.clone());
        }

        let spent_proofs: BTreeSet<sn_dbc::SpentProof> = input_dbcs
            .iter()
            .flat_map(|dbc| dbc.spent_proofs.clone())
            .collect();

        let spent_transactions: BTreeSet<DbcTransaction> = input_dbcs
            .iter()
            .flat_map(|dbc| dbc.spent_transactions.clone())
            .collect();

        let proof_key_verifier = SpentProofKeyVerifier { client };

        // Let's build the output DBCs
        let mut dbc_builder = tx_builder.build(rng::thread_rng())?;

        // Spend all the input DBCs, collecting the spent proof shares for each of them
        for (public_key, tx) in dbc_builder.inputs() {
            let tx_hash = Hash::from(tx.hash());
            // TODO: spend DBCs concurrently spawning tasks
            let mut attempts = 0;
            loop {
                attempts += 1;
                client
                    .spend_dbc(
                        public_key,
                        tx.clone(),
                        spent_proofs.clone(),
                        spent_transactions.clone(),
                    )
                    .await?;

                let spent_proof_shares = client.spent_proof_shares(public_key).await?;

                // TODO: we temporarilly filter the spent proof shares which correspond to the TX we
                // are spending now. This is because current implementation of Spentbook allows
                // double spents, so we may be retrieving spent proof shares for others spent TXs.
                let shares_for_current_tx: HashSet<sn_dbc::SpentProofShare> = spent_proof_shares
                    .into_iter()
                    .filter(|proof_share| proof_share.content.transaction_hash == tx_hash)
                    .collect();

                match verify_spent_proof_shares_for_tx(
                    public_key,
                    tx_hash,
                    shares_for_current_tx.iter(),
                    &proof_key_verifier,
                ) {
                    Ok(()) => {
                        dbc_builder = dbc_builder
                            .add_spent_proof_shares(shares_for_current_tx.into_iter())
                            .add_spent_transaction(tx);

                        break;
                    }
                    Err(err) if attempts == NUM_OF_DBC_REISSUE_ATTEMPTS => {
                        bail!(format!(
                            "Failed to spend input, {} proof shares obtained from spentbook: {}",
                            shares_for_current_tx.len(),
                            err
                        ))
                        // return Err(crate::Error::DbcSpendRetryAttemptsExceeded {
                        //     attempts,
                        //     public_key,
                        // });
                    }
                    Err(_) => {}
                }
            }
        }

        // Perform verifications of input TX and spentproofs,
        // as well as building the output DBCs.
        let mut output_dbcs = dbc_builder.build(&proof_key_verifier)?;

        let mut change_dbc = None;
        output_dbcs.retain(|(dbc, owneronce, _)| {
            if owneronce == &change_owneronce && change_amount.as_nano() > 0 {
                change_dbc = Some(dbc.clone());
                false
            } else {
                true
            }
        });

        Ok((output_dbcs, change_dbc))
    }

    // Private helper to verify if a set of spent proof shares are valid for a given public_key and TX
    fn verify_spent_proof_shares_for_tx<'a>(
        public_key: sn_dbc::PublicKey,
        tx_hash: Hash,
        proof_shares: impl Iterator<Item = &'a sn_dbc::SpentProofShare>,
        proof_key_verifier: &SpentProofKeyVerifier,
    ) -> Result<()> {
        sn_dbc::SpentProof::try_from_proof_shares(public_key, tx_hash, proof_shares)
            .and_then(|spent_proof| spent_proof.verify(tx_hash, proof_key_verifier))?;

        Ok(())
    }

    /// Verifier required by test to check a SpentProof
    /// is signed by known sections keys.
    struct SpentProofKeyVerifier<'a> {
        client: &'a Client,
    }

    impl sn_dbc::SpentProofKeyVerifier for SpentProofKeyVerifier<'_> {
        type Error = crate::Error;

        // Called by test when it needs to verify a SpentProof is signed by a known key,
        // we check if the key is any of the network sections keys we are aware of
        fn verify_known_key(&self, key: &sn_dbc::PublicKey) -> crate::Result<()> {
            if !futures::executor::block_on(self.client.is_known_section_key(key)) {
                Err(crate::Error::SectionsDagKeyNotFound(*key))
            } else {
                Ok(())
            }
        }
    }
}
