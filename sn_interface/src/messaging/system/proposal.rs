// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    messaging::system::{SectionSigShare, SectionSigned},
    messaging::{Error, Result},
    network_knowledge::{NodeState, SectionAuthorityProvider},
};

use serde::{Deserialize, Serialize};

/// A Proposal about the state of the section
/// This can be a result of seeing a node come online, go offline, changes to section info etc.
/// Anything where we need section authority before action can be taken
/// Proposals sent by elders or elder candidates, to elders or elder candidates
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Proposal {
    /// Proposal to remove a node from our section (elder->elder)
    VoteNodeOffline(NodeState),
    /// After DKG, elder candidates request handover with this Proposal to the current elders
    /// by submitting their new SAP (candidates->elder)
    RequestHandover(SectionAuthorityProvider),
    /// After Handover consensus, the elders inform the candidates that they are promoted (elder->candidates)
    /// Contains the candidate's SAP signed by the current elder's section key
    NewElders(SectionSigned<SectionAuthorityProvider>),
    /// Proposal to change whether new nodes are allowed to join our section (elder->elder)
    JoinsAllowed(bool),
}

impl Proposal {
    /// Create `SigShare` for this proposal.
    pub fn sign_with_key_share(
        &self,
        public_key_set: bls::PublicKeySet,
        index: usize,
        secret_key_share: &bls::SecretKeyShare,
    ) -> Result<SectionSigShare> {
        Ok(SectionSigShare::new(
            public_key_set,
            index,
            secret_key_share,
            &self.as_signable_bytes()?,
        ))
    }

    pub fn as_signable_bytes(&self) -> Result<Vec<u8>> {
        let bytes = match self {
            Self::VoteNodeOffline(node_state) => bincode::serialize(node_state),
            Self::RequestHandover(sap) => bincode::serialize(sap),
            Self::NewElders(info) => bincode::serialize(&info.sig.public_key), // the pub key of the new elders
            Self::JoinsAllowed(joins_allowed) => bincode::serialize(&joins_allowed),
        }
        .map_err(|err| {
            Error::Serialisation(format!(
                "Couldn't serialise the Proposal '{:?}': {:?}",
                self, err
            ))
        })?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network_knowledge::test_utils::random_sap;

    use eyre::Result;
    use serde::Serialize;
    use std::fmt::Debug;
    use xor_name::Prefix;

    #[test]
    fn serialize_for_signing() -> Result<()> {
        // Proposal::RequestHandover
        let (section_auth, _, _) = random_sap(Prefix::default(), 4, 0, None);
        let proposal = Proposal::RequestHandover(section_auth.clone());
        verify_serialize_for_signing(&proposal, &section_auth)?;

        // Proposal::NewElders
        let new_sk = bls::SecretKey::random();
        let new_pk = new_sk.public_key();
        let section_signed_auth =
            crate::network_knowledge::test_utils::section_signed(&new_sk, section_auth)?;
        let proposal = Proposal::NewElders(section_signed_auth);
        verify_serialize_for_signing(&proposal, &new_pk)?;

        Ok(())
    }

    // Verify that `SignableView(proposal)` serializes the same as `should_serialize_as`.
    fn verify_serialize_for_signing<T>(proposal: &Proposal, should_serialize_as: &T) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let actual = proposal.as_signable_bytes()?;
        let expected = bincode::serialize(should_serialize_as)?;

        assert_eq!(
            actual, expected,
            "expected SignableView({:?}) to serialize same as {:?}, but didn't",
            proposal, should_serialize_as
        );

        Ok(())
    }
}
