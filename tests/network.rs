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

// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md

use rand::Rng;
use rand::distributions::{IndependentSample, Range};
use routing::{ImmutableData, MutableData};
use routing::mock_crust::{self, Network};
use safe_vault::{Config, GROUP_SIZE, test_utils};
use safe_vault::mock_crust_detail::{poll, test_node};
use safe_vault::mock_crust_detail::test_client::TestClient;

#[test]
fn fill_network() {
    let network = Network::new(GROUP_SIZE, None);
    let mut rng = network.new_rng();

    let config = Config {
        wallet_address: None,
        max_capacity: Some(2000),
        chunk_store_root: None,
    };
    // Use 8 nodes to avoid the case where four target nodes are full: In that case neither the
    // PutSuccess nor the PutFailure accumulates and client.put_and_verify() would hang.
    let mut nodes = test_node::create_nodes(&network, 8, Some(config), true);
    let crust_config = mock_crust::Config::with_contacts(&[nodes[0].endpoint()]);
    let mut client = TestClient::new(&network, Some(crust_config));
    let full_id = client.full_id().clone();

    client.ensure_connected(&mut nodes);
    client.create_account(&mut nodes);

    loop {
        let (result, data_id) = if rng.gen() {
            let data = test_utils::gen_immutable_data(100, &mut rng);
            let data_id = data.debug_id();
            let result = client.put_idata_response(data, &mut nodes);
            (result, data_id)
        } else {
            let owner = *full_id.public_id().signing_public_key();
            let data = test_utils::gen_mutable_data(rng.gen(), 20, owner, &mut rng);
            let data_id = data.debug_id();
            let result = client.put_mdata_response(data, &mut nodes);
            (result, data_id)
        };

        match result {
            Ok(()) => trace!("Stored {}", data_id),
            Err(error) => {
                trace!("Failed storing {}, reason: {:?}", data_id, error);
                break;
            }
        }
    }


    for _ in 0..10 {
        let index = Range::new(1, nodes.len()).ind_sample(&mut rng);
        trace!("Adding node with bootstrap node {}.", index);
        test_node::add_node(&network, &mut nodes, index, true);
        let _ = poll::poll_and_resend_unacknowledged(&mut nodes, &mut client);

        let data = test_utils::gen_immutable_data(100, &mut rng);
        let data_id = data.debug_id();

        match client.put_idata_response(data, &mut nodes) {
            Ok(()) => {
                    trace!("Stored {}", data_id);
                    return;
                }
            Err(error) => {
                trace!("Failed storing {}, reason: {:?}", data_id, error);
            }
        }
    }

    panic!("Failed to put again after adding nodes.");
}

trait DebugId {
    fn debug_id(&self) -> String;
}

impl DebugId for ImmutableData {
    fn debug_id(&self) -> String {
        format!("immutable chunk (name: {:?})", self.name())
    }
}

impl DebugId for MutableData {
    fn debug_id(&self) -> String {
        format!("mutable chunk (name: {:?}, tag: {})",
                self.name(),
                self.tag())
    }
}
