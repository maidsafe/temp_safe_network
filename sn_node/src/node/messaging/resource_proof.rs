// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{
    flow_ctrl::cmds::Cmd, messaging::Peers, Error, Node, Result, RESOURCE_PROOF_DATA_SIZE,
    RESOURCE_PROOF_DIFFICULTY,
};

use sn_interface::{
    messaging::system::{JoinResponse, ResourceProof, SystemMsg},
    types::{keys::ed25519, log_markers::LogMarker, Peer},
};

use ed25519_dalek::Verifier;
use xor_name::XorName;

// Resource signed
impl Node {
    pub(crate) fn validate_resource_proof(
        &self,
        peer_name: &XorName,
        proof: ResourceProof,
    ) -> bool {
        let serialized = if let Ok(serialized) = bincode::serialize(&(peer_name, &proof.nonce)) {
            serialized
        } else {
            return false;
        };

        if self
            .keypair
            .public
            .verify(&serialized, &proof.nonce_signature)
            .is_err()
        {
            return false;
        }

        self.resource_proof
            .validate_all(&proof.nonce, &proof.data, proof.solution)
    }

    pub(crate) fn send_resource_proof_challenge(&self, peer: Peer) -> Result<Cmd> {
        let nonce: [u8; 32] = rand::random();
        let serialized =
            bincode::serialize(&(peer.name(), &nonce)).map_err(|_| Error::InvalidMessage)?;
        let response = SystemMsg::JoinResponse(Box::new(JoinResponse::ResourceChallenge {
            data_size: RESOURCE_PROOF_DATA_SIZE,
            difficulty: RESOURCE_PROOF_DIFFICULTY,
            nonce,
            nonce_signature: ed25519::sign(&serialized, &self.keypair),
        }));
        trace!("{}", LogMarker::SendResourceProofChallenge);
        Ok(self.send_system(response, Peers::Single(peer)))
    }
}
