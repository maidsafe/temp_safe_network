// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::NodeMsgAuthority;
use crate::types::keys::ed25519::{self};
use bls::PublicKey as BlsPublicKey;
use std::collections::BTreeSet;
use xor_name::XorName;

pub trait NodeMsgAuthorityUtils {
    fn src_location(&self) -> (XorName, bls::PublicKey);

    fn name(&self) -> XorName;

    // Verify if the section key of the NodeMsgAuthority can be trusted
    // based on a set of known keys.
    fn verify_src_section_key_is_known(&self, known_keys: &BTreeSet<BlsPublicKey>) -> bool;
}

impl NodeMsgAuthorityUtils for NodeMsgAuthority {
    fn src_location(&self) -> (XorName, bls::PublicKey) {
        match self {
            Self::Node(node_auth) => (ed25519::name(&node_auth.node_ed_pk), node_auth.section_pk),
            Self::BlsShare(bls_share_auth) => (bls_share_auth.src_name, bls_share_auth.section_pk),
            Self::Section(section_auth) => (section_auth.src_name, section_auth.sig.public_key),
        }
    }

    fn name(&self) -> XorName {
        match self {
            Self::Node(node_auth) => ed25519::name(&node_auth.node_ed_pk),
            Self::BlsShare(bls_share_auth) => bls_share_auth.src_name,
            Self::Section(section_auth) => section_auth.src_name,
        }
    }

    // Verify if it's a section/bls-share signed authority,
    // and if can be trusted based on a set of known keys.
    fn verify_src_section_key_is_known(&self, known_keys: &BTreeSet<BlsPublicKey>) -> bool {
        let section_pk = match &self {
            Self::Node(_) => return true,
            Self::BlsShare(bls_share_auth) => &bls_share_auth.section_pk,
            Self::Section(section_auth) => &section_auth.sig.public_key,
        };

        known_keys.contains(section_pk)
    }
}
