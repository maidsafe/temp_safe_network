// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::PublicKey;
use bls::PublicKeySet;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use xor_name::{Prefix, XorName};

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct SectionElders {
    ///
    pub prefix: Prefix,
    ///
    pub names: BTreeSet<XorName>,
    ///
    pub key_set: PublicKeySet,
}

impl SectionElders {
    /// The BLS public key
    pub fn key(&self) -> bls::PublicKey {
        self.key_set.public_key()
    }

    /// The BLS based name
    pub fn name(&self) -> XorName {
        PublicKey::Bls(self.key()).into()
    }

    /// The prefix based address
    pub fn address(&self) -> XorName {
        self.prefix.name()
    }
}
