// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::flow_ctrl::cmds::Cmd;
use crate::node::{MyNode, Result};
use sn_interface::messaging::system::SectionSigned;
use sn_interface::messaging::{system::SectionSigShare, MsgId};
use sn_interface::types::Peer;
use sn_interface::SectionAuthorityProvider;

impl MyNode {
    pub(crate) fn handle_handover_promotion(
        &mut self,
        msg_id: MsgId,
        sap: SectionSigned<SectionAuthorityProvider>,
        sig_share: SectionSigShare,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        trace!("Handling handover promotion message {msg_id:?} by {sender:?} with sap: {sap:?}");
        let our_prefix = self.network_knowledge.prefix();
        let sig_share_pk = &sig_share.public_key_set.public_key();

        // Proposal from other sections shall be ignored.
        if !our_prefix.matches(&sender.name()) {
            trace!("Ignore promotion message {msg_id:?} from other section");
            return Ok(vec![]);
        }
        // Let's now verify the section key in the msg authority is trusted
        // based on our current knowledge of the network and sections chains.
        if !self.network_knowledge.has_chain_key(sig_share_pk) {
            warn!("Ignore promotion message {msg_id:?} with untrusted sig share");
            return Ok(vec![]);
        }

        // try aggregate
        let serialize_err = |e| {
            error!(
                "Failed to serialize pubkey while handling handover promotion message {msg_id:?}"
            );
            e
        };
        let serialised_pk = bincode::serialize(&sap.sig.public_key).map_err(serialize_err)?;
        match self
            .elder_promotion_aggregator
            .try_aggregate(&serialised_pk, sig_share)
        {
            Ok(Some(sig)) => {
                trace!("Promotion message {msg_id:?} successfully aggregated");
                Ok(vec![Cmd::HandleNewEldersAgreement {
                    new_elders: sap,
                    sig,
                }])
            }
            Ok(None) => {
                trace!("Promotion message {msg_id:?} acknowledged, waiting for more...");
                Ok(vec![])
            }
            Err(err) => {
                error!("Failed to aggregate promotion message {msg_id:?} from {sender}: {err:?}");
                Ok(vec![])
            }
        }
    }

    pub(crate) fn handle_section_split_promotion(
        &mut self,
        msg_id: MsgId,
        sap1: SectionSigned<SectionAuthorityProvider>,
        sig_share1: SectionSigShare,
        sap2: SectionSigned<SectionAuthorityProvider>,
        sig_share2: SectionSigShare,
        sender: Peer,
    ) -> Result<Vec<Cmd>> {
        trace!("Handling section split promotion message {msg_id:?} by {sender:?} with saps: {sap1:?} {sap2:?}");
        let our_prefix = self.network_knowledge.prefix();
        let sig_share_pk1 = &sig_share1.public_key_set.public_key();
        let sig_share_pk2 = &sig_share2.public_key_set.public_key();

        // Proposal from other sections shall be ignored.
        if !our_prefix.matches(&sender.name()) {
            trace!("Ignore promotion message {msg_id:?} from other section");
            return Ok(vec![]);
        }
        // Let's now verify the section key in the msg authority is trusted
        // based on our current knowledge of the network and sections chains.
        if !self.network_knowledge.has_chain_key(sig_share_pk1)
            || !self.network_knowledge.has_chain_key(sig_share_pk2)
        {
            warn!("Ignore promotion message {msg_id:?} with untrusted sig share");
            return Ok(vec![]);
        }

        // try aggregate
        let serialize_err = |e| {
            error!("Failed to serialize pubkey while handling split promotion message {msg_id:?}");
            e
        };
        let serialised_pk1 = bincode::serialize(&sap1.sig.public_key).map_err(serialize_err)?;
        let serialised_pk2 = bincode::serialize(&sap2.sig.public_key).map_err(serialize_err)?;
        let res1 = self
            .elder_promotion_aggregator
            .try_aggregate(&serialised_pk1, sig_share1);
        let res2 = self
            .elder_promotion_aggregator
            .try_aggregate(&serialised_pk2, sig_share2);

        match (res1, res2) {
            (Ok(Some(sig1)), Ok(Some(sig2))) => {
                trace!("Promotion message {msg_id:?} successfully aggregated");
                Ok(vec![Cmd::HandleNewSectionsAgreement {
                    sap1,
                    sig1,
                    sap2,
                    sig2,
                }])
            }
            (Ok(None), Ok(None)) => {
                trace!("Promotion message {msg_id:?} acknowledged, waiting for more...");
                Ok(vec![])
            }
            (_, Err(err)) | (Err(err), _) => {
                error!("Failed to aggregate promotion message {msg_id:?} from {sender}: {err:?}");
                Ok(vec![])
            }
            _ => {
                warn!("Unexpected aggregation result aggregate promotion message {msg_id:?}: one sig is aggregated while the other is not. This should not happen.");
                Ok(vec![])
            }
        }
    }
}
