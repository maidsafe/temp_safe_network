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
    dbcs::DbcReason,
    messaging::data::{
        DataCmd, DataQuery, Error as NetworkDataError, QueryResponse, SpendQuery, SpentbookCmd,
    },
    types::{fees::FeeCiphers, SpentbookAddress},
};

use std::collections::{BTreeMap, BTreeSet};
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
    #[instrument(skip(self, tx, spent_proofs, spent_transactions), level = "debug")]
    pub async fn spend_dbc(
        &self,
        public_key: PublicKey,
        tx: DbcTransaction,
        reason: DbcReason,
        spent_proofs: BTreeSet<SpentProof>,
        spent_transactions: BTreeSet<DbcTransaction>,
        #[cfg(not(feature = "data-network"))] fee_ciphers: BTreeMap<XorName, FeeCiphers>,
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
                reason,
                spent_proofs: spent_proofs.clone(),
                spent_transactions: spent_transactions.clone(),
                network_knowledge,
                #[cfg(not(feature = "data-network"))]
                fee_ciphers: fee_ciphers.clone(),
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
    // Spend related reads
    //---------------------

    /// Return the set of spent proof shares if the provided DBC's public key is spent
    #[instrument(skip(self), level = "debug")]
    pub async fn spent_proof_shares(&self, public_key: PublicKey) -> Result<Vec<SpentProofShare>> {
        let address = SpentbookAddress::new(XorName::from_content(&public_key.to_bytes()));
        let query = DataQuery::Spentbook(SpendQuery::GetSpentProofShares(address));
        let response = self.send_query(query.clone()).await?;
        match response {
            QueryResponse::GetSpentProofShares(res) => {
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
    use crate::{
        api::send_tokens,
        utils::test_utils::{
            create_test_client_with, init_logger, read_genesis_dbc_from_first_node,
        },
        Client,
    };

    use sn_dbc::{rng, OwnerOnce, Token};
    use sn_interface::{messaging::data::Error as ErrorMsg, types::fees::SpendPriority};

    use eyre::{bail, Result};

    const ONE_BN_NANOS: u64 = 1_000_000_000;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_spentbook_spend_dbc() -> Result<()> {
        init_logger();
        let _outer_span = tracing::info_span!("test__spentbook_spend_dbc").entered();

        let (client, genesis_dbc) = get_genesis(false).await?;

        let mut rng = rng::thread_rng();
        let base_owner = sn_dbc::Owner::from_random_secret_key(&mut rng);
        let recipient = OwnerOnce::from_owner_base(base_owner, &mut rng);
        let half_amount = genesis_dbc
            .as_revealed_input_bearer()?
            .revealed_amount()
            .value()
            / 2;
        let recipients = vec![(Token::from_nano(half_amount), recipient)];

        // Send the tokens..
        let (_, change) = send_tokens(
            &client,
            vec![genesis_dbc],
            recipients,
            SpendPriority::Normal,
        )
        .await?;

        // We only assert that we have some change back
        // since we don't need/want to account for the fees here.
        assert!(change.is_some());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn spentbook_spend_spent_proof_with_invalid_pk_should_return_spentbook_error(
    ) -> Result<()> {
        init_logger();
        let _outer_span = tracing::info_span!(
            "test__spentbook_spend_spent_proof_with_invalid_pk_should_return_spentbook_error"
        )
        .entered();

        let (client, mut genesis_dbc) = get_genesis(false).await?;

        let mut rng = rng::thread_rng();
        let base_owner = sn_dbc::Owner::from_random_secret_key(&mut rng);
        let recipient = OwnerOnce::from_owner_base(base_owner, &mut rng);
        let half_amount = genesis_dbc
            .as_revealed_input_bearer()?
            .revealed_amount()
            .value()
            / 2;
        let recipients = vec![(Token::from_nano(half_amount), recipient)];

        // Insert the invalid pk to proofs.
        let invalid_pk = bls::SecretKey::random().public_key();
        genesis_dbc.inputs_spent_proofs = genesis_dbc
            .inputs_spent_proofs
            .into_iter()
            .map(|mut proof| {
                proof.spentbook_pub_key = invalid_pk;
                proof
            })
            .collect();

        // Send the tokens..
        let result = send_tokens(
            &client,
            vec![genesis_dbc],
            recipients,
            SpendPriority::Normal,
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

        let (client, genesis_dbc) = get_genesis(true).await?; // pass in true, for getting an invalid genesis

        let mut rng = rng::thread_rng();
        let base_owner = sn_dbc::Owner::from_random_secret_key(&mut rng);
        let recipient = OwnerOnce::from_owner_base(base_owner, &mut rng);
        let half_amount = genesis_dbc
            .as_revealed_input_bearer()?
            .revealed_amount()
            .value()
            / 2;
        let recipients = vec![(Token::from_nano(half_amount), recipient)];

        let genesis_dbc_owner_pk = genesis_dbc.owner_base().public_key();

        // Send the tokens..
        let result = send_tokens(
            &client,
            vec![genesis_dbc],
            recipients,
            SpendPriority::Normal,
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

        // The idea for this test case is to pass the wrong spent proofs and transactions for
        // the public key we're trying to spend. To do so, we reissue `output_dbc_1` from
        // `genesis_dbc`, then reissue `output_dbc_2` from `output_dbc_1`, then when we try to spend
        // `output_dbc_2`, we use the spent proofs/transactions from `genesis_dbc`. This should
        // not be permitted. The correct way would be to pass the spent proofs/transactions
        // from `output_dbc_1`, which was our input to `output_dbc_2`.

        let (client, genesis_dbc) = get_genesis(false).await?;

        let mut rng = rng::thread_rng();

        let recipient_1 = OwnerOnce::from_owner_base(client.dbc_owner().clone(), &mut rng);
        let dbc_id_1 = recipient_1.as_owner().public_key();

        // Send the tokens..
        let (outputs_1, _) = send_tokens(
            &client,
            vec![genesis_dbc.clone()],
            vec![(Token::from_nano(ONE_BN_NANOS), recipient_1)],
            SpendPriority::Normal,
        )
        .await?;

        let output_dbc_1 = match outputs_1
            .iter()
            .find(|(dbc, _, _)| dbc.public_key() == dbc_id_1)
        {
            Some((dbc, _, _)) => dbc.clone(),
            None => bail!("We expected to find the dbc we were looking for."),
        };

        // -> Next

        let recipient_2 = OwnerOnce::from_owner_base(client.dbc_owner().clone(), &mut rng);
        let dbc_id_2 = recipient_2.as_owner().public_key();

        // Send the tokens..
        let (outputs_2, _) = send_tokens(
            &client,
            vec![output_dbc_1],
            vec![(Token::from_nano(ONE_BN_NANOS / 2), recipient_2)],
            SpendPriority::Normal,
        )
        .await?;

        let mut output_dbc_2 = match outputs_2
            .iter()
            .find(|(dbc, _, _)| dbc.public_key() == dbc_id_2)
        {
            Some((dbc, _, _)) => dbc.clone(),
            None => bail!("We expected to find the dbc we were looking for."),
        };

        output_dbc_2.inputs_spent_proofs = genesis_dbc.inputs_spent_proofs.clone();
        output_dbc_2.inputs_spent_transactions = genesis_dbc.inputs_spent_transactions;

        let recipient_3 = OwnerOnce::from_owner_base(client.dbc_owner().clone(), &mut rng);

        // Send the tokens..
        let result = send_tokens(
            &client,
            vec![output_dbc_2],
            vec![(Token::from_nano(ONE_BN_NANOS / 4), recipient_3)],
            SpendPriority::Normal,
        )
        .await;

        match result {
            Ok(_) => bail!("We expected an error to be returned"),
            Err(crate::Error::CmdError {
                source: ErrorMsg::InvalidOperation(error_string),
                ..
            }) => {
                let correct_error_str =
                    format!("{:?}", sn_dbc::Error::MissingAmountForPubkey(dbc_id_2));
                assert!(
                    error_string.contains(&correct_error_str),
                    "A different SpentbookError error was expected for this case. What we got: {error_string:?}, expected: {correct_error_str:?}"
                );
                Ok(())
            }
            Err(error) => bail!("We expected a different error to be returned. Actual: {error:?}"),
        }
    }

    // returns a client which is the owner to the genesis dbc,
    // we can do this since our genesis dbc is currently generated as a bearer dbc, and stored locally
    // so we can fetch that owner key from the first node, and pass it to the client
    async fn get_genesis(invalid_genesis_dbc: bool) -> Result<(Client, sn_dbc::Dbc)> {
        init_logger();

        let genesis_dbc = if invalid_genesis_dbc {
            let sk_set = bls::SecretKeySet::random(0, &mut rand::thread_rng());
            sn_interface::dbcs::gen_genesis_dbc(&sk_set, &sk_set.secret_key())?
        } else {
            read_genesis_dbc_from_first_node()?
        };
        let dbc_owner = genesis_dbc.owner_base().clone();
        let client = create_test_client_with(None, Some(dbc_owner.clone()), None).await?;

        Ok((client, genesis_dbc))
    }
}
