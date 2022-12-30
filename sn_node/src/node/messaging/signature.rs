// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{MyNode, Result};
use sn_interface::messaging::system::SectionSigShare;
use sn_interface::network_knowledge::SectionKeyShare;

impl MyNode {
    /// Sign a piece of data with a given key_share
    pub(crate) fn sign_with_key_share<T: AsRef<[u8]>>(
        data: T,
        key_share: &SectionKeyShare,
    ) -> SectionSigShare {
        SectionSigShare::new(
            key_share.public_key_set.clone(),
            key_share.index,
            &key_share.secret_key_share,
            data.as_ref(),
        )
    }

    /// Sign a piece of data with our current section sig share
    /// Fails if we are not an elder or if we are missing the key
    pub(crate) fn sign_with_section_key_share<T: AsRef<[u8]>>(
        &self,
        data: T,
    ) -> Result<SectionSigShare> {
        let section_key = self.network_knowledge.section_key();
        let key_share = self.section_keys_provider.key_share(&section_key)?;
        Ok(MyNode::sign_with_key_share(data, &key_share))
    }
}
