// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::system::{Proposal as ProposalMsg, SectionAuth};
use crate::node::{dkg::SigShare, network_knowledge::SectionAuthorityProvider, Result};

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Proposal {
    SectionInfo(SectionAuthorityProvider),
    NewElders(SectionAuth<SectionAuthorityProvider>),
}

impl Proposal {
    /// Create SigShare for this proposal.
    pub(crate) fn sign_with_key_share(
        &self,
        public_key_set: bls::PublicKeySet,
        index: usize,
        secret_key_share: &bls::SecretKeyShare,
    ) -> Result<SigShare> {
        Ok(SigShare::new(
            public_key_set,
            index,
            secret_key_share,
            &self.as_signable_bytes()?,
        ))
    }

    pub(crate) fn as_signable_bytes(&self) -> Result<Vec<u8>> {
        Ok(match self {
            Self::SectionInfo(info) => bincode::serialize(info),
            Self::NewElders(info) => bincode::serialize(&info.sig.public_key),
        }?)
    }
}

// Add conversion methods to/from `messaging::...::Proposal`
// We prefer this over `From<...>` to make it easier to read the conversion.

impl Proposal {
    pub(crate) fn into_msg(self) -> ProposalMsg {
        match self {
            Self::SectionInfo(sap) => ProposalMsg::SectionInfo(sap.to_msg()),
            Self::NewElders(sap) => ProposalMsg::NewElders(sap.into_authed_msg()),
        }
    }
}

impl ProposalMsg {
    pub(crate) fn into_state(self) -> Proposal {
        match self {
            Self::SectionInfo(sap) => Proposal::SectionInfo(sap.into_state()),
            Self::NewElders(sap) => Proposal::NewElders(sap.into_authed_state()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{dkg, network_knowledge};
    use eyre::Result;
    use serde::Serialize;
    use std::fmt::Debug;
    use xor_name::Prefix;

    #[test]
    fn serialize_for_signing() -> Result<()> {
        // Proposal::SectionInfo
        let (section_auth, _, _) =
            network_knowledge::test_utils::gen_section_authority_provider(Prefix::default(), 4);
        let proposal = Proposal::SectionInfo(section_auth.clone());
        verify_serialize_for_signing(&proposal, &section_auth)?;

        // Proposal::NewElders
        let new_sk = bls::SecretKey::random();
        let new_pk = new_sk.public_key();
        let section_signed_auth = dkg::test_utils::section_signed(&new_sk, section_auth)?;
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
