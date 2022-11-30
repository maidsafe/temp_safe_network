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

use itertools::Either;
use serde::{Deserialize, Serialize};

/// A Proposal about the state of the section
/// This can be a result of seeing a node go offline, changes to section info etc.
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
    /// After Handover consensus, the elders inform the candidates that they are promoted (elder->candidates)
    /// Contains the candidate's SAP signed by the current elder's section key
    NewSections {
        sap1: SectionSigned<SectionAuthorityProvider>,
        sap2: SectionSigned<SectionAuthorityProvider>,
    },
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
    ) -> Result<Either<SectionSigShare, (SectionSigShare, SectionSigShare)>> {
        match &self.as_signable_bytes()? {
            Either::Left(bytes) => Ok(Either::Left(SectionSigShare::new(
                public_key_set,
                index,
                secret_key_share,
                bytes,
            ))),
            Either::Right((bytes_1, bytes_2)) => {
                let sig_1 =
                    SectionSigShare::new(public_key_set.clone(), index, secret_key_share, bytes_1);
                let sig_2 = SectionSigShare::new(public_key_set, index, secret_key_share, bytes_2);
                Ok(Either::Right((sig_1, sig_2)))
            }
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn as_signable_bytes(&self) -> Result<Either<Vec<u8>, (Vec<u8>, Vec<u8>)>> {
        let bytes = match self {
            Self::VoteNodeOffline(node_state) => bincode::serialize(node_state),
            Self::RequestHandover(sap) => bincode::serialize(sap),
            Self::NewElders(signed_sap) => bincode::serialize(&signed_sap.sig.public_key), // the pub key of the new elders
            Self::NewSections { sap1, sap2 } => {
                // the pub key of the new elders
                let sap1_sig = bincode::serialize(&sap1.sig.public_key).map_err(|err| {
                    Error::Serialisation(format!(
                        "Couldn't serialise the Proposal '{:?}': {:?}",
                        self, err
                    ))
                })?;
                let sap2_sig = bincode::serialize(&sap2.sig.public_key).map_err(|err| {
                    Error::Serialisation(format!(
                        "Couldn't serialise the Proposal '{:?}': {:?}",
                        self, err
                    ))
                })?;
                return Ok(Either::Right((sap1_sig, sap2_sig)));
            }
            Self::JoinsAllowed(joins_allowed) => bincode::serialize(&joins_allowed),
        }
        .map_err(|err| {
            Error::Serialisation(format!(
                "Couldn't serialise the Proposal '{:?}': {:?}",
                self, err
            ))
        })?;
        Ok(Either::Left(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{TestKeys, TestSapBuilder};

    use eyre::Result;
    use serde::Serialize;
    use std::fmt::Debug;
    use xor_name::Prefix;

    #[test]
    fn serialize_for_signing() -> Result<()> {
        // Proposal::RequestHandover
        let (sap, ..) = TestSapBuilder::new(Prefix::default())
            .elder_count(4)
            .build();
        let proposal = Proposal::RequestHandover(sap.clone());
        verify_serialize_for_signing(&proposal, &sap)?;

        // Proposal::NewElders
        let new_sk = bls::SecretKey::random();
        let new_pk = new_sk.public_key();
        let signed_sap = TestKeys::get_section_signed(&new_sk, sap);
        let proposal = Proposal::NewElders(signed_sap);
        verify_serialize_for_signing(&proposal, &new_pk)?;

        Ok(())
    }

    // Verify that `SignableView(proposal)` serializes the same as `should_serialize_as`.
    fn verify_serialize_for_signing<T>(proposal: &Proposal, should_serialize_as: &T) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let actual = match proposal.as_signable_bytes()? {
            itertools::Either::Left(bytes) => bytes,
            itertools::Either::Right(_) => {
                eyre::bail!("Invalid expectations! Wrong proposal used!")
            }
        };
        let expected = bincode::serialize(should_serialize_as)?;

        assert_eq!(
            actual, expected,
            "expected SignableView({:?}) to serialize same as {:?}, but didn't",
            proposal, should_serialize_as
        );

        Ok(())
    }
}
