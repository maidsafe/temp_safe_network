// Copyright 2016 MaidSafe.net limited.
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

use rand::{self, Rng};
use routing::{Authority, ImmutableData, XorName};
use rust_sodium::crypto::sign;
use utils;

macro_rules! assert_match {
    ($e:expr, $p:pat => $r:expr) => {
        match $e {
            $p => $r,
            ref x => panic!("Unexpected {:?} (expecting: {})", x, stringify!($p)),
        }
    };

    ($e:expr, $p:pat) => {
        assert_match!($e, $p => ())
    }
}

/// Toggle iterations for quick test environment variable
pub fn iterations() -> usize {
    use std::env;
    match env::var("QUICK_TEST") {
        Ok(_) => 4,
        Err(_) => 10,
    }
}

/// Creates random immutable data
pub fn random_immutable_data<R: Rng>(size: usize, rng: &mut R) -> ImmutableData {
    ImmutableData::new(rng.gen_iter().take(size).collect())
}

/// Generate random `Client` authority and return it together with its client key.
pub fn gen_client_authority() -> (Authority<XorName>, sign::PublicKey) {
    let (client_key, _) = sign::gen_keypair();

    let client = Authority::Client {
        client_key: client_key,
        peer_id: rand::random(),
        proxy_node_name: rand::random(),
    };

    (client, client_key)
}

/// Generate `ClientManager` authority for the client with the given client key.
pub fn gen_client_manager_authority(client_key: sign::PublicKey) -> Authority<XorName> {
    Authority::ClientManager(utils::client_name_from_key(&client_key))
}
