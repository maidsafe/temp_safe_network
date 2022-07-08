// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;

use crate::Error;

use sn_dbc::{KeyImage, RingCtTransaction, SpentProof, SpentProofShare};
use sn_interface::{
    messaging::data::{DataCmd, DataQueryVariant, QueryResponse, SpentbookCmd, SpentbookQuery},
    types::SpentbookAddress,
};

use xor_name::XorName;

impl Client {
    //----------------------
    // Write Operations
    //---------------------

    /// Spend a DBC's key image.
    #[instrument(skip(self), level = "debug")]
    pub async fn spend_dbc(
        &self,
        key_image: KeyImage,
        tx: RingCtTransaction,
        spent_proofs: Vec<SpentProof>,
        spent_transactions: Vec<RingCtTransaction>,
    ) -> Result<(), Error> {
        let cmd = SpentbookCmd::Spend {
            key_image,
            tx,
            spent_proofs,
            spent_transactions,
        };
        self.send_cmd(DataCmd::Spentbook(cmd)).await
    }

    //----------------------
    // Read Spentbook
    //---------------------

    /// Return the set of spent proof shares if the provided DBC's key image is spent
    #[instrument(skip(self), level = "debug")]
    pub async fn spent_proof_shares(
        &self,
        key_image: KeyImage,
    ) -> Result<Vec<SpentProofShare>, Error> {
        let address = SpentbookAddress::new(XorName::from_content(&key_image.to_bytes()));
        let query = DataQueryVariant::Spentbook(SpentbookQuery::SpentProofShares(address));
        let query_result = self.send_query(query).await?;
        match query_result.response {
            QueryResponse::SpentProofShares((res, op_id)) => {
                res.map_err(|err| Error::ErrorMsg { source: err, op_id })
            }
            _ => Err(Error::ReceivedUnexpectedEvent),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::test_utils::{
        create_test_client_with, init_logger, read_genesis_dbc_from_first_node,
    };
    use eyre::Result;
    use sn_dbc::{rng, OwnerOnce, TransactionBuilder};

    #[tokio::test(flavor = "multi_thread")]
    async fn test_spentbook_spend_dbc() -> Result<()> {
        init_logger();

        let genesis_dbc = read_genesis_dbc_from_first_node()?;
        let dbc_owner = genesis_dbc.owner_base().clone();
        let client = create_test_client_with(None, Some(dbc_owner.clone()), None).await?;

        let genesis_key_image = genesis_dbc.key_image_bearer()?;

        // Obtain the number of current spent shares for this key_image, note this test
        // could have been more than once thus the genesis DBC could have been spent already.
        // TODO: in this test we allow double spents for now, once we have our Spentbook
        // to prevent double spents we'll need to adapt this test.
        let previously_spent_shares = client.spent_proof_shares(genesis_key_image).await?;

        let output_owner = OwnerOnce::from_owner_base(dbc_owner, &mut rng::thread_rng());
        let dbc_builder = TransactionBuilder::default()
            .set_decoys_per_input(0)
            .set_require_all_decoys(false)
            .add_input_dbc_bearer(&genesis_dbc)?;

        let inputs_amount_sum = dbc_builder.inputs_amount_sum();
        let dbc_builder = dbc_builder
            .add_output_by_amount(inputs_amount_sum, output_owner)
            .build(&mut rng::thread_rng())?;

        assert_eq!(dbc_builder.inputs().len(), 1);
        let (key_image, tx) = &dbc_builder.inputs()[0];
        assert_eq!(&genesis_key_image, key_image);

        let spent_proofs = genesis_dbc.spent_proofs.into_iter().collect::<Vec<_>>();
        let spent_transactions = genesis_dbc
            .spent_transactions
            .into_iter()
            .collect::<Vec<_>>();

        // Spend the key_image.
        client
            .spend_dbc(*key_image, tx.clone(), spent_proofs, spent_transactions)
            .await?;

        // Get spent proof shares for the key_image
        let spent_proof_shares = client.spent_proof_shares(*key_image).await?;

        // We should have 'spent_proof_shares' client API to contact at least
        // a supermajority of Elders for writing and reading Spentbooks, this is why
        // should obtain a supermajority of spent proof shares.
        // Note we just check we received more spent proof shares than there already were
        // in the spent book before, since we are temporarily allowing double spents in this test.
        assert!(spent_proof_shares.len() >= 5 + previously_spent_shares.len());

        Ok(())
    }
}
