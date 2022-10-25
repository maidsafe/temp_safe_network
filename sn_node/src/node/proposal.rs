// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::Result;

use sn_interface::messaging::system::Proposal;

pub(crate) fn as_signable_bytes(proposal: &Proposal) -> Result<Vec<u8>> {
    Ok(match proposal {
        Proposal::VoteNodeOffline(node_state) => bincode::serialize(node_state),
        Proposal::SectionInfo(sap) => bincode::serialize(sap),
        Proposal::NewElders(info) => bincode::serialize(&info.sig.public_key), // the pub key of the new elders
        Proposal::JoinsAllowed(joins_allowed) => bincode::serialize(&joins_allowed),
    }?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sn_interface::network_knowledge::test_utils::random_sap;

    use eyre::Result;
    use serde::Serialize;
    use std::fmt::Debug;
    use xor_name::Prefix;

    #[test]
    fn serialize_for_signing() -> Result<()> {
        // Proposal::SectionInfo
        let (section_auth, _, _) = random_sap(Prefix::default(), 4, 0, None);
        let proposal = Proposal::SectionInfo(section_auth.clone());
        verify_serialize_for_signing(&proposal, &section_auth)?;

        // Proposal::NewElders
        let new_sk = bls::SecretKey::random();
        let new_pk = new_sk.public_key();
        let section_signed_auth =
            sn_interface::network_knowledge::test_utils::section_signed(&new_sk, section_auth)?;
        let proposal = Proposal::NewElders(section_signed_auth);
        verify_serialize_for_signing(&proposal, &new_pk)?;

        Ok(())
    }

    // Verify that `SignableView(proposal)` serializes the same as `should_serialize_as`.
    fn verify_serialize_for_signing<T>(proposal: &Proposal, should_serialize_as: &T) -> Result<()>
    where
        T: Serialize + Debug,
    {
        let actual = as_signable_bytes(proposal)?;
        let expected = bincode::serialize(should_serialize_as)?;

        assert_eq!(
            actual, expected,
            "expected SignableView({:?}) to serialize same as {:?}, but didn't",
            proposal, should_serialize_as
        );

        Ok(())
    }
}
