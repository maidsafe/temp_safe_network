// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{JoinResponse, ResourceProofResponse, SystemMsg};
use crate::node::{
    api::cmds::Cmd,
    core::{Node, RESOURCE_PROOF_DATA_SIZE, RESOURCE_PROOF_DIFFICULTY},
    ed25519,
    network_knowledge::NodeState,
    Error, Result,
};
use crate::types::{log_markers::LogMarker, Peer};

use ed25519_dalek::Verifier;

// Resource signed
impl Node {
    pub(crate) async fn handle_resource_proof_response(
        &self,
        peer: Peer,
        resource_proof: ResourceProofResponse,
    ) -> Result<Vec<Cmd>> {
        let sig_is_verified =
            if let Ok(serialized) = bincode::serialize(&(peer.name(), &resource_proof.nonce)) {
                self.info
                    .read()
                    .await
                    .keypair
                    .public
                    .verify(&serialized, &resource_proof.nonce_signature)
                    .is_ok()
            } else {
                false
            };

        let is_valid = sig_is_verified
            && self.resource_proof.validate_all(
                &resource_proof.nonce,
                &resource_proof.data,
                resource_proof.solution,
            );

        if is_valid {
            // Propose Join with BRB membership consensus protocol
            let node_state = NodeState::joined(peer, None);
            Ok(vec![Cmd::HandleNewNodeOnline(node_state)])
        } else {
            debug!(
                "Ignoring JoinRequest from {} - invalid resource signed response",
                peer
            );
            Ok(vec![])
        }
    }

    pub(crate) async fn send_resource_proof_challenge(&self, peer: Peer) -> Result<Cmd> {
        let nonce: [u8; 32] = rand::random();
        let serialized =
            bincode::serialize(&(peer.name(), &nonce)).map_err(|_| Error::InvalidMessage)?;
        let response = SystemMsg::JoinResponse(Box::new(JoinResponse::ResourceChallenge {
            data_size: RESOURCE_PROOF_DATA_SIZE,
            difficulty: RESOURCE_PROOF_DIFFICULTY,
            nonce,
            nonce_signature: ed25519::sign(&serialized, &self.info.read().await.keypair),
        }));

        trace!("{}", LogMarker::SendResourceProofChallenge);
        self.send_direct_msg(peer, response, self.network_knowledge.section_key().await)
            .await
    }
}
