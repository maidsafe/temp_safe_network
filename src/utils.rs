// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use maidsafe_utilities;
use routing::{Authority, MutableData, Value, XorName};
use rust_sodium::crypto::hash::sha256;
use rust_sodium::crypto::sign;

/// Extract client key (a public singing key) from the authority.
///
/// # Panics
///
/// Panics when the authority is not `Client`.
pub fn client_key(authority: &Authority<XorName>) -> &sign::PublicKey {
    if let Authority::Client { ref client_key, .. } = *authority {
        client_key
    } else {
        unreachable!("Logic error")
    }
}

/// Extract client name (a `XorName`) from the authority.
///
/// # Panics
///
/// Panics when the authority is not `Client` or `ClientManager`.
pub fn client_name(authority: &Authority<XorName>) -> XorName {
    match *authority {
        Authority::Client { ref client_key, .. } => client_name_from_key(client_key),
        Authority::ClientManager(name) => name,
        _ => unreachable!("Logic error"),
    }
}

pub fn client_name_from_key(key: &sign::PublicKey) -> XorName {
    XorName(sha256::hash(&key[..]).0)
}

pub fn mdata_shell_hash(data: &MutableData) -> u64 {
    let shell = (*data.name(),
                 data.tag(),
                 data.version(),
                 data.owners().clone(),
                 data.permissions().clone());
    maidsafe_utilities::big_endian_sip_hash(&shell)
}

pub fn mdata_value_hash(value: &Value) -> u64 {
    maidsafe_utilities::big_endian_sip_hash(&value)
}
