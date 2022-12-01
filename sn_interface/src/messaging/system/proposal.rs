// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    messaging::system::SectionSigShare,
    messaging::{Error, Result},
    network_knowledge::{NodeState, SapCandidate, SectionAuthorityProvider},
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
    HandoverCompleted(SapCandidate),
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
            Self::HandoverCompleted(SapCandidate::ElderHandover(signed_sap)) => {
                // the pub key of the new elders
                bincode::serialize(&signed_sap.sig.public_key)
            }
            Self::HandoverCompleted(SapCandidate::SectionSplit(sap1, sap2)) => {
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
    use itertools::Either;
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
        verify_serialize_for_signing(&proposal, Either::Left(&sap))?;

        // Proposal::HandoverCompleted, SapCandidate::ElderHandover
        let new_sk = bls::SecretKey::random();
        let new_pk = new_sk.public_key();
        let signed_sap = TestKeys::get_section_signed(&new_sk, sap);
        let candidate = SapCandidate::ElderHandover(signed_sap.clone());
        let proposal = Proposal::HandoverCompleted(candidate);
        verify_serialize_for_signing(&proposal, Either::Left(&new_pk))?;

        // Proposal::HandoverCompleted, SapCandidate::SectionSplit
        let new_sk_2 = bls::SecretKey::random();
        let new_pk_2 = new_sk_2.public_key();
        let (sap2, ..) = TestSapBuilder::new(Prefix::default())
            .elder_count(4)
            .build();
        let signed_sap2 = TestKeys::get_section_signed(&new_sk_2, sap2);
        let candidate = SapCandidate::SectionSplit(signed_sap, signed_sap2);
        let proposal = Proposal::HandoverCompleted(candidate);
        verify_serialize_for_signing(&proposal, Either::Right((&new_pk, &new_pk_2)))?;

        Ok(())
    }

    // Verify that `SignableView(proposal)` serializes the same as `should_serialize_as`.
    fn verify_serialize_for_signing<T>(
        proposal: &Proposal,
        should_serialize_as: Either<&T, (&T, &T)>,
    ) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let actual = proposal.as_signable_bytes()?;
        let expected = match should_serialize_as {
            Either::Left(item) => Either::Left(bincode::serialize(item)?),
            Either::Right((item1, item2)) => {
                Either::Right((bincode::serialize(item1)?, bincode::serialize(item2)?))
            }
        };

        assert_eq!(
            actual, expected,
            "expected SignableView({:?}) to serialize same as {:?}, but didn't",
            proposal, should_serialize_as
        );

        Ok(())
    }
}
