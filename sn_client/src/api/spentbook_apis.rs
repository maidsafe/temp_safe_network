// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Client;

use crate::Error;
use sn_dbc::{KeyImage, RingCtTransaction, SpentProofShare};
use sn_interface::messaging::data::{
    DataCmd, DataQuery, QueryResponse, SpentbookCmd, SpentbookQuery,
};
use sn_interface::types::SpentbookAddress;

use xor_name::XorName;

impl Client {
    //----------------------
    // Write Operations
    //---------------------

    /// Spend a DBC's key image.
    #[instrument(skip(self, tx), level = "debug")]
    pub async fn spend_dbc(&self, key_image: KeyImage, tx: RingCtTransaction) -> Result<(), Error> {
        let cmd = SpentbookCmd::Spend { key_image, tx };
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
        let query = DataQuery::Spentbook(SpentbookQuery::SpentProofShares(address));
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
    use super::*;
    use crate::utils::test_utils::{create_test_client, init_logger};
    use eyre::Result;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_spentbook_spend_dbc() -> Result<()> {
        init_logger();

        let client = create_test_client().await?;

        let sk = bls::SecretKey::random();
        let key_image = KeyImage::from_bytes(sk.public_key().to_bytes())?;

        // Check the spentbook is empty for this key_image
        let spent_proof_shares = client.spent_proof_shares(key_image).await?;
        assert_eq!(spent_proof_shares.len(), 0);

        // Spend the key_image. TODO: provide a valid RingCtTransaction
        let tx = RingCtTransaction {
            mlsags: Vec::new(),
            outputs: Vec::new(),
        };
        client.spend_dbc(key_image, tx).await?;

        // Get spent proof shares for the key_image
        let spent_proof_shares = client.spent_proof_shares(key_image).await?;

        // TODO: we should have 'spent_proof_shares' client API to contact at least
        // a supermajority of Elders for writing and reading Spentbooks, this is why now we don't
        // obtain a supermajority of spent proof shares yet.
        assert!(spent_proof_shares.len() >= 2);

        Ok(())
    }
}
