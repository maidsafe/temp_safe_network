// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::MessageId;
use crate::node::{
    network::Network, node_api::interaction::push_state, node_api::role::ElderRole,
    node_ops::NodeDuties, Error, Node, Result,
};
use crate::routing::{Prefix, XorName};
use crate::types::PublicKey;
use std::collections::BTreeSet;

impl Node {
    /// Called on split reported from routing layer.
    pub(crate) async fn begin_split_as_newbie(&self, our_key: PublicKey) -> Result<()> {
        let section_key = self.network_api.section_public_key().await?;
        if our_key != section_key {
            return Err(Error::Logic(format!(
                "Some failure.. our_key: {}, section_key: {}",
                our_key, section_key
            )));
        }

        self.level_up().await
    }

    /// Called on split reported from routing layer.
    pub(crate) async fn begin_split_as_oldie(
        elder: &ElderRole,
        network_api: &Network,
        our_prefix: Prefix,
        our_key: PublicKey,
        sibling_key: PublicKey,
        our_new_elders: BTreeSet<XorName>,
        their_new_elders: BTreeSet<XorName>,
    ) -> Result<NodeDuties> {
        let sibling_prefix = our_prefix.sibling();
        let mut ops = vec![];

        // replicate state to our new elders
        let msg_id = MessageId::combine(&[our_prefix.name().0, XorName::from(our_key).0]);
        ops.push(push_state(elder, our_prefix, msg_id, our_new_elders).await?);

        // replicate state to our neighbour's new elders
        let msg_id = MessageId::combine(&[sibling_prefix.name().0, XorName::from(sibling_key).0]);
        ops.push(push_state(elder, sibling_prefix, msg_id, their_new_elders).await?);

        let our_adults = network_api.our_adults().await;
        // drop metadata state
        elder
            .meta_data
            .write()
            .await
            .retain_members_only(our_adults)
            .await?;

        Ok(ops)
    }
}
