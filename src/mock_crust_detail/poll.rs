// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use super::test_client::TestClient;
use super::test_node::TestNode;
use fake_clock::FakeClock;
use routing::test_consts::{ACK_TIMEOUT_SECS, CONNECTED_PEER_TIMEOUT_SECS, RATE_EXCEED_RETRY_MS};

// Maximum number of times to try and poll in a loop.  This is several orders higher than the
// anticipated upper limit for any test, and if hit is likely to indicate an infinite loop.
const MAX_POLL_CALLS: usize = 1000;

/// Empty event queue of nodes provided
pub fn nodes(nodes: &mut [TestNode]) -> usize {
    nodes_and_clients(nodes, &mut [], false)
}

/// Empty event queue of nodes and the client provided
pub fn nodes_and_client(nodes: &mut [TestNode], client: &mut TestClient) -> usize {
    nodes_and_clients(nodes, ref_slice_mut(client), false)
}

/// Empty event queue of nodes and clients provided
pub fn nodes_and_clients(
    nodes: &mut [TestNode],
    clients: &mut [TestClient],
    parallel_poll: bool,
) -> usize {
    let mut count: usize = 0;
    let mut fired_rate_exceeded_timeout = false;
    loop {
        nodes[0].handle.deliver_messages();
        let prev_count = count;

        for node in nodes.iter_mut() {
            if parallel_poll {
                if node.poll_once() {
                    count += 1;
                }
            } else {
                count += node.poll();
            }
        }

        for client in clients.iter_mut() {
            if parallel_poll {
                if client.poll_once() {
                    count += 1;
                }
            } else {
                count += client.poll();
            }
        }

        if prev_count == count {
            if !fired_rate_exceeded_timeout {
                fired_rate_exceeded_timeout = true;
                FakeClock::advance_time(RATE_EXCEED_RETRY_MS + 1);
            } else {
                break;
            }
        } else {
            fired_rate_exceeded_timeout = false;
        }
    }

    count
}

/// Empty event queue of nodes and client, until there are no unacknowledged messages left.
pub fn nodes_and_client_with_resend(nodes: &mut [TestNode], client: &mut TestClient) -> usize {
    nodes_and_clients_with_resend(nodes, ref_slice_mut(client))
}

/// Empty event queue of nodes and clients, until there are no unacknowledged messages left.
pub fn nodes_and_clients_with_resend(nodes: &mut [TestNode], clients: &mut [TestClient]) -> usize {
    let mut fired_connected_peer_timeout = false;
    let mut total_count = 0;

    for _ in 0..MAX_POLL_CALLS {
        let count = nodes_and_clients(nodes, clients, false);
        if count > 0 {
            // Once each route is polled, advance time to trigger the following route.
            FakeClock::advance_time(ACK_TIMEOUT_SECS * 1000 + 1);
            total_count += count;
        } else if !fired_connected_peer_timeout {
            // When all routes are polled, advance time to purge any invalid peers.
            FakeClock::advance_time(CONNECTED_PEER_TIMEOUT_SECS * 1000 + 1);
            fired_connected_peer_timeout = true;
        } else {
            return total_count;
        }
    }
    panic!("Polling has been called {} times.", MAX_POLL_CALLS);
}

// Converts a reference to `A` into a slice of length 1 (without copying).
#[allow(unsafe_code)]
fn ref_slice_mut<A>(s: &mut A) -> &mut [A] {
    use std::slice;
    unsafe { slice::from_raw_parts_mut(s, 1) }
}
