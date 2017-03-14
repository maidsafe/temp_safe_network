// Copyright 2017 MaidSafe.net limited.
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

use super::*;
use maidsafe_utilities::serialisation::deserialise;
use rand::{self, Rng};
use routing::{EntryActions, Request, Response};
use std::env;
use test_utils;

const CHUNK_STORE_CAPACITY: u64 = 1024;
const CHUNK_STORE_DIR: &'static str = "test_safe_vault_chunk_store";

const TEST_TAG: u64 = 12345678;

#[test]
fn idata_basics() {
    let (client, client_key) = test_utils::gen_client_authority();
    let client_manager = test_utils::gen_client_manager_authority(client_key);

    let data = test_utils::gen_random_immutable_data(10, &mut rand::thread_rng());
    let nae_manager = Authority::NaeManager(*data.name());

    let mut node = RoutingNode::new();
    let mut dm = create_data_manager();

    // Get non-existent data fails.
    let msg_id = MessageId::new();
    unwrap!(dm.handle_get_idata(&mut node, client, nae_manager, *data.name(), msg_id));

    let message = unwrap!(node.sent_responses.remove(&msg_id));
    assert_match!(
            message.response,
            Response::GetIData { res: Err(ClientError::NoSuchData), .. });

    // Put immutable data sends refresh to the NAE manager.
    let msg_id = MessageId::new();
    unwrap!(dm.handle_put_idata(&mut node, client_manager, nae_manager, data.clone(), msg_id));

    let message = unwrap!(node.sent_requests.remove(&msg_id));
    let refresh = assert_match!(message.request, Request::Refresh(payload, _) => payload);
    assert_eq!(message.src, nae_manager);
    assert_eq!(message.dst, nae_manager);

    // Simulate receiving the refresh. This should result in the data being
    // put into the chunk store.
    unwrap!(dm.handle_group_refresh(&mut node, &refresh));

    // Get the data back and assert its the same data we put in originally.
    let msg_id = MessageId::new();
    unwrap!(dm.handle_get_idata(&mut node, client, nae_manager, *data.name(), msg_id));

    let message = unwrap!(node.sent_responses.remove(&msg_id));
    let retrieved_data =
        assert_match!(message.response, Response::GetIData { res: Ok(data), .. } => data);
    assert_eq!(retrieved_data, data);
}

#[test]
fn mdata_basics() {
    let mut rng = rand::thread_rng();

    let (client, client_key) = test_utils::gen_client_authority();
    let client_manager = test_utils::gen_client_manager_authority(client_key);

    let data = test_utils::gen_empty_mutable_data(TEST_TAG, client_key, &mut rng);
    let data_name = *data.name();
    let nae_manager = Authority::NaeManager(data_name);

    let mut node = RoutingNode::new();
    let mut dm = create_data_manager();

    // Attempt to list entries of non-existent data fails.
    let msg_id = MessageId::new();
    unwrap!(dm.handle_list_mdata_entries(&mut node,
                                             client,
                                             nae_manager,
                                             data_name,
                                             TEST_TAG,
                                             msg_id));
    let message = unwrap!(node.sent_responses.remove(&msg_id));
    assert_match!(
            message.response,
            Response::ListMDataEntries { res: Err(ClientError::NoSuchData), .. });

    // Put mutable data sends refresh to the NAE manager.
    let msg_id = MessageId::new();
    unwrap!(dm.handle_put_mdata(&mut node,
                                    client_manager,
                                    nae_manager,
                                    data,
                                    msg_id,
                                    client_key));

    let message = unwrap!(node.sent_requests.remove(&msg_id));
    let refresh = assert_match!(message.request, Request::Refresh(payload, _) => payload);

    // Simulate receiving the refresh. This should result in the data being
    // put into the chunk store.
    unwrap!(dm.handle_group_refresh(&mut node, &refresh));

    let message = unwrap!(node.sent_responses.remove(&msg_id));
    assert_match!(message.response, Response::PutMData { res: Ok(()), .. });

    // Now list the data entries - should successfuly respond with empty list.
    let msg_id = MessageId::new();
    unwrap!(dm.handle_list_mdata_entries(&mut node,
                                             client,
                                             nae_manager,
                                             data_name,
                                             TEST_TAG,
                                             msg_id));

    let message = unwrap!(node.sent_responses.remove(&msg_id));
    let entries = assert_match!(
            message.response,
            Response::ListMDataEntries { res: Ok(entries), .. } => entries);
    assert!(entries.is_empty());
}

#[test]
fn mdata_mutations() {
    let mut rng = rand::thread_rng();

    let (client, client_key) = test_utils::gen_client_authority();

    let data = test_utils::gen_empty_mutable_data(TEST_TAG, client_key, &mut rng);
    let data_name = *data.name();
    let nae_manager = Authority::NaeManager(data_name);

    let mut node = RoutingNode::new();
    let mut dm = create_data_manager();

    // Put the data.
    dm.put_into_chunk_store(data);

    // Initially, the entries should be empty.
    let msg_id = MessageId::new();
    unwrap!(dm.handle_list_mdata_entries(&mut node,
                                             client,
                                             nae_manager,
                                             data_name,
                                             TEST_TAG,
                                             msg_id));

    let message = unwrap!(node.sent_responses.remove(&msg_id));
    let entries = assert_match!(
            message.response,
            Response::ListMDataEntries { res: Ok(entries), .. } => entries);
    assert!(entries.is_empty());

    // Mutate the entries and simulate refresh.
    let key_0 = test_utils::gen_random_vec(10, &mut rng);
    let value_0 = test_utils::gen_random_vec(10, &mut rng);

    let key_1 = test_utils::gen_random_vec(10, &mut rng);
    let value_1 = test_utils::gen_random_vec(10, &mut rng);

    let actions = EntryActions::new()
        .ins(key_0.clone(), value_0.clone(), 0)
        .ins(key_1.clone(), value_1.clone(), 0)
        .into();
    let msg_id = MessageId::new();
    unwrap!(dm.handle_mutate_mdata_entries(&mut node,
                                               client,
                                               nae_manager,
                                               data_name,
                                               TEST_TAG,
                                               actions,
                                               msg_id,
                                               client_key));

    let message = unwrap!(node.sent_requests.remove(&msg_id));
    let refresh = assert_match!(message.request, Request::Refresh(payload, _) => payload);
    unwrap!(dm.handle_group_refresh(&mut node, &refresh));

    let message = unwrap!(node.sent_responses.remove(&msg_id));
    assert_match!(message.response, Response::MutateMDataEntries { res: Ok(()), .. });

    // The data should now contain the previously inserted two entries.
    let msg_id = MessageId::new();
    unwrap!(dm.handle_list_mdata_entries(&mut node,
                                             client,
                                             nae_manager,
                                             data_name,
                                             TEST_TAG,
                                             msg_id));

    let message = unwrap!(node.sent_responses.remove(&msg_id));
    let entries = assert_match!(
            message.response,
            Response::ListMDataEntries { res: Ok(entries), .. } => entries);
    assert_eq!(entries.len(), 2);
    let retrieved_value_0 = unwrap!(entries.get(&key_0));
    let retrieved_value_1 = unwrap!(entries.get(&key_1));

    assert_eq!(retrieved_value_0.content, value_0);
    assert_eq!(retrieved_value_1.content, value_1);
}

#[test]
fn mdata_change_owner() {
    let mut rng = rand::thread_rng();

    let (client_0, client_0_key) = test_utils::gen_client_authority();

    let data = test_utils::gen_empty_mutable_data(TEST_TAG, client_0_key, &mut rng);
    let data_name = *data.name();
    let nae_manager = Authority::NaeManager(data_name);

    let mut node = RoutingNode::new();
    let mut dm = create_data_manager();

    // Put the data.
    dm.put_into_chunk_store(data);

    let (client_1, _) = test_utils::gen_client_authority();
    let (_, client_2_key) = test_utils::gen_client_authority();

    // Attempt to change the owner by a non-owner fails.
    let mut new_owners = BTreeSet::new();
    let _ = new_owners.insert(client_2_key);

    let msg_id = MessageId::new();
    unwrap!(dm.handle_change_mdata_owner(&mut node,
                                             client_1,
                                             nae_manager,
                                             data_name,
                                             TEST_TAG,
                                             new_owners.clone(),
                                             1,
                                             msg_id));
    let message = unwrap!(node.sent_responses.remove(&msg_id));
    assert_match!(
            message.response,
            Response::ChangeMDataOwner { res: Err(ClientError::AccessDenied), .. });

    // Changing the owner by the current owner succeeds.
    let msg_id = MessageId::new();
    unwrap!(dm.handle_change_mdata_owner(&mut node,
                                             client_0,
                                             nae_manager,
                                             data_name,
                                             TEST_TAG,
                                             new_owners,
                                             1,
                                             msg_id));
    let message = unwrap!(node.sent_requests.remove(&msg_id));
    let refresh = assert_match!(message.request, Request::Refresh(payload, _) => payload);
    unwrap!(dm.handle_group_refresh(&mut node, &refresh));

    let message = unwrap!(node.sent_responses.remove(&msg_id));
    assert_match!(message.response, Response::ChangeMDataOwner { res: Ok(()), .. });
}

#[test]
fn churn_idata() {
    let mut rng = rand::thread_rng();

    // N0 - this node has the data initially
    let mut n0_node = RoutingNode::new();
    let mut n0_dm = create_data_manager();
    let n0_name = unwrap!(n0_node.name());

    // N1 - a new node that has joined the network.
    let mut n1_node = RoutingNode::new();
    let mut n1_dm = create_data_manager();
    let n1_name = unwrap!(n1_node.name());

    // Other nodes in the group
    let node_names: Vec<_> = rng.gen_iter().take(GROUP_SIZE - 2).collect();

    n0_node.add_to_routing_table(n1_name);
    n1_node.add_to_routing_table(n0_name);

    for name in &node_names {
        n0_node.add_to_routing_table(*name);
        n1_node.add_to_routing_table(*name);
    }

    let data = test_utils::gen_random_immutable_data(10, &mut rng);
    n0_dm.put_into_chunk_store(data.clone());

    // N0 receives node added event. It should send refresh containing all the
    // fragments it holds to the new node.
    let rt = n0_node.routing_table().clone();
    n0_dm.handle_node_added(&mut n0_node, &n1_name, &rt);

    assert_eq!(n0_node.sent_requests.len(), 1);
    let (_, message) = unwrap!(n0_node.sent_requests.drain().next());
    assert_eq!(message.src, Authority::ManagedNode(n0_name));
    assert_eq!(message.dst, Authority::ManagedNode(n1_name));
    let refresh_payload = assert_match!(message.request,
                                        Request::Refresh(payload, _) => payload);

    let fragments: Vec<FragmentInfo> = unwrap!(deserialise(&refresh_payload));
    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0], FragmentInfo::ImmutableData(*data.name()));

    // N1 receives the refresh message from P0. The message should not accumulate yet.
    unwrap!(n1_dm.handle_refresh(&mut n1_node, n0_name, &refresh_payload));
    assert!(n1_node.sent_requests.is_empty());

    // N1 receives the refresh from at least QUORUM other nodes. The message should now accumulate.
    for node_name in node_names.iter().take(ACCUMULATOR_QUORUM - 1) {
        unwrap!(n1_dm.handle_refresh(&mut n1_node, *node_name, &refresh_payload));
    }

    assert_eq!(n1_node.sent_requests.len(), 1);
    let (msg_id, message) = unwrap!(n1_node.sent_requests.drain().next());
    let name = assert_match!(message.request,
                             Request::GetIData { name, .. } => name);
    let dst = assert_match!(message.dst, Authority::ManagedNode(name) => name);
    assert_eq!(name, *data.name());

    // One of the nodes receives the above GetIData requests and sends the
    // response. We gloss over that here, as it's not the focus of the test.

    // N1 receives response to the get request. It should put the data into the chunk store.
    unwrap!(n1_dm.handle_get_idata_success(&mut n1_node, dst, data.clone(), msg_id));
    assert!(n1_dm.get_from_chunk_store(&DataId::Immutable(*data.name())).is_some());

    // TODO: test also that if N1 receives get_idata failure, it retries the
    // request with another holder.
}

fn create_data_manager() -> DataManager {
    let suffix: u64 = rand::random();
    let dir = format!("{}_{}", CHUNK_STORE_DIR, suffix);

    unwrap!(DataManager::new(env::temp_dir().join(dir), CHUNK_STORE_CAPACITY))
}
