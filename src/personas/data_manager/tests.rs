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
use maidsafe_utilities::serialisation::{deserialise, serialise};
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

    let data = test_utils::gen_immutable_data(10, &mut rand::thread_rng());
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

    let data = test_utils::gen_mutable_data(TEST_TAG, 0, client_key, &mut rng);
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
    let client_manager = test_utils::gen_client_manager_authority(client_key);

    let data = test_utils::gen_mutable_data(TEST_TAG, 0, client_key, &mut rng);
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
    let key_0 = test_utils::gen_vec(10, &mut rng);
    let value_0 = test_utils::gen_vec(10, &mut rng);

    let key_1 = test_utils::gen_vec(10, &mut rng);
    let value_1 = test_utils::gen_vec(10, &mut rng);

    let actions = EntryActions::new()
        .ins(key_0.clone(), value_0.clone(), 0)
        .ins(key_1.clone(), value_1.clone(), 0)
        .into();
    let msg_id = MessageId::new();
    unwrap!(dm.handle_mutate_mdata_entries(&mut node,
                                           client_manager,
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

    let (_, client_key_0) = test_utils::gen_client_authority();
    let client_manager_0 = test_utils::gen_client_manager_authority(client_key_0);

    let data = test_utils::gen_mutable_data(TEST_TAG, 0, client_key_0, &mut rng);
    let data_name = *data.name();
    let nae_manager = Authority::NaeManager(data_name);

    let mut node = RoutingNode::new();
    let mut dm = create_data_manager();

    // Put the data.
    dm.put_into_chunk_store(data);

    let (_, client_key_1) = test_utils::gen_client_authority();
    let client_manager_1 = test_utils::gen_client_manager_authority(client_key_1);

    let (_, client_key_2) = test_utils::gen_client_authority();

    // Attempt to change the owner by a non-owner fails.
    let mut new_owners = BTreeSet::new();
    let _ = new_owners.insert(client_key_2);

    let msg_id = MessageId::new();
    unwrap!(dm.handle_change_mdata_owner(&mut node,
                                         client_manager_1,
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
                                         client_manager_0,
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
fn handle_node_added() {
    let mut rng = rand::thread_rng();

    let mut node = RoutingNode::new();
    let mut dm = create_data_manager();

    let new_node_name = rand::random();
    node.add_to_routing_table(new_node_name);

    let (_, client_key) = test_utils::gen_client_authority();
    let data0 = test_utils::gen_immutable_data(10, &mut rng);
    let data1 = test_utils::gen_mutable_data(TEST_TAG, 2, client_key, &mut rng);

    dm.put_into_chunk_store(data0.clone());
    dm.put_into_chunk_store(data1.clone());

    // Node receives NodeAdded event. It should send all the fragments that it has
    // to the new node in a Refresh message.
    let rt = node.routing_table().clone();
    dm.handle_node_added(&mut node, &new_node_name, &rt);

    assert_eq!(node.sent_requests.len(), 1);

    let (_, message) = unwrap!(node.sent_requests.drain().next());

    assert_eq!(message.src, Authority::ManagedNode(unwrap!(node.name())));
    assert_eq!(message.dst, Authority::ManagedNode(new_node_name));

    let payload = assert_match!(message.request, Request::Refresh(payload, _) => payload);
    let fragments: Vec<FragmentInfo> = unwrap!(deserialise(&payload));

    assert_eq!(fragments.len(), 4);

    let check = fragments.iter().any(|fragment| match *fragment {
        FragmentInfo::ImmutableData(ref name) if name == data0.name() => true,
        _ => false,
    });
    assert!(check);

    let check = fragments.iter().any(|fragment| match *fragment {
        FragmentInfo::MutableDataShell { name, tag, version, .. } if name == *data1.name() &&
                                                                     tag == data1.tag() &&
                                                                     version == data1.version() => {
            true
        }
        _ => false,
    });
    assert!(check);

    for entry_key in data1.keys() {
        let check = fragments.iter().any(|fragment| match *fragment {
            FragmentInfo::MutableDataEntry { name, tag, ref key, .. } if name == *data1.name() &&
                                                                         tag == data1.tag() &&
                                                                         *key == *entry_key => true,
            _ => false,
        });
        assert!(check);
    }

    // TODO: test also that the refresh contains fragments that the node doesn't
    // yet have, but needs.
}

// Test how immutable data is replicated from nodes that hold it to a newly
// joined node during churn.
//
// 1) New node X joins a group that holds an immutable data.
// 2) Existing nodes from the group send refresh to X with the info about the data.
// 3) X accumualtes the refresh and send request to retrieve the data
// 4) X receives response failure. Retries the request.
// 5) X receives response with wrong data. Retries the request.
// 6) X receives good response. It puts the data into the chunk store and sends
//    no more requests.
#[test]
fn idata_with_churn() {
    let mut rng = rand::thread_rng();

    let (mut new_node, mut new_dm, old_node_names) = setup_churn(&mut rng);

    let data = test_utils::gen_immutable_data(10, &mut rng);

    let fragment = FragmentInfo::ImmutableData(*data.name());
    let refresh_payload = unwrap!(serialise(&vec![fragment]));

    // New node receives the refresh message from one of the old nodes. The message
    // should not accumulate yet.
    unwrap!(new_dm.handle_refresh(&mut new_node, old_node_names[0], &refresh_payload));
    assert!(new_node.sent_requests.is_empty());

    // New node receives the refresh from at least QUORUM other nodes. The message
    // should now accumulate.
    for name in old_node_names.iter().skip(1).take(ACCUMULATOR_QUORUM - 1) {
        unwrap!(new_dm.handle_refresh(&mut new_node, *name, &refresh_payload));
    }

    // Helper function to verify the node sent the get request for a data with
    // the given name. Returns the message id and the destination authority name
    // of the request.
    fn verify_get_idata_request_sent(node: &mut RoutingNode,
                                     data_name: &XorName)
                                     -> (MessageId, XorName) {
        assert_eq!(node.sent_requests.len(), 1);
        let (msg_id, message) = unwrap!(node.sent_requests.drain().next());
        let name = assert_match!(message.request,
                                 Request::GetIData { name, .. } => name);
        assert_eq!(name, *data_name);
        let dst = assert_match!(message.dst, Authority::ManagedNode(name) => name);

        (msg_id, dst)
    }

    let (msg_id, dst) = verify_get_idata_request_sent(&mut new_node, data.name());

    // One of the nodes receives the above GetIData requests and sends the
    // response. We gloss over that here, as it's not the focus of the test.

    // Simulate failure of the GetIData request. New node should retry the request with
    // another holder.
    unwrap!(new_dm.handle_get_idata_failure(&mut new_node, dst, msg_id));
    let (msg_id, dst) = verify_get_idata_request_sent(&mut new_node, data.name());

    // Again, we gloss over the request handling and response sending here.

    // Simulate malicious node sending wrong data. New node should throw it away and
    // send another request to another holder.
    let bad_data = test_utils::gen_immutable_data(10, &mut rng);
    let bad_data_name = *bad_data.name();
    unwrap!(new_dm.handle_get_idata_success(&mut new_node, dst, bad_data, msg_id));
    assert!(new_dm.get_from_chunk_store(&DataId::Immutable(bad_data_name)).is_none());
    let (msg_id, dst) = verify_get_idata_request_sent(&mut new_node, data.name());

    // ...

    // New node now receives successful response. It should put the data into the chunk store.
    unwrap!(new_dm.handle_get_idata_success(&mut new_node, dst, data.clone(), msg_id));
    assert!(new_dm.get_from_chunk_store(&DataId::Immutable(*data.name())).is_some());

    // New node should not send any more requests to the other holders, because it already
    // has everything it needs.
    assert!(new_node.sent_requests.is_empty());
}

// Test how mutable data with some entries is replicated from nodes that hold it
// to a newly joined node during churn.
//
// 1. New node X joins the group that holds the data.
// 2. Nodes from the group send refresh message to X. The message consist of multiple
//    fragments - one for the shell of the data, and one for each entry.
// 3. X receives the refresh and sends requests for all the fragments of the data.
// 4. X receives success respones to the requests. First the shell, the the entries.
// 5. X puts the complete data into the chunk store and sends no more requests.
#[test]
fn mdata_with_churn() {
    let mut rng = rand::thread_rng();

    let (_, client_key) = test_utils::gen_client_authority();
    let data = test_utils::gen_mutable_data(TEST_TAG, 2, client_key, &mut rng);

    let (mut new_node, mut new_dm) = setup_mdata_refresh(&data, &mut rng);

    // New node should sent one request for the shell and one request for each entry.
    assert_eq!(new_node.sent_requests.len(), 3);
    let (shell_msg_id, shell_dst) = take_get_mdata_shell_request(&mut new_node);
    let entry_messages = take_get_mdata_value_requests(&mut new_node);
    assert_eq!(entry_messages.len(), 2);

    // New node receives responses for the above requests. It should put the complete
    // data (shell + entries) into the chunk store.
    unwrap!(new_dm.handle_get_mdata_shell_success(&mut new_node,
                                                  shell_dst,
                                                  data.shell(),
                                                  shell_msg_id));

    for (msg_id, dst, key) in entry_messages {
        let value = unwrap!(data.get(&key)).clone();
        unwrap!(new_dm.handle_get_mdata_value_success(&mut new_node,
                                                      dst,
                                                      value,
                                                      msg_id));
    }

    let stored_data = assert_match!(
        new_dm.get_from_chunk_store(&DataId::mutable(&data)),
        Some(Data::Mutable(data)) => data);
    assert_eq!(stored_data, data);

    // NEW NODE should not send any more requests, because it already has everything it needs.
    assert!(new_node.sent_requests.is_empty());
}

// Same as `mdata_with_churn` except now X receives response failure.
//
// 1. X receives response fialure
// 2. X sends the requests again, to a different node from the group.
#[test]
fn mdata_with_churn_with_response_failure() {
    let mut rng = rand::thread_rng();

    let (_, client_key) = test_utils::gen_client_authority();
    let data = test_utils::gen_mutable_data(TEST_TAG, 1, client_key, &mut rng);

    let (mut new_node, mut new_dm) = setup_mdata_refresh(&data, &mut rng);

    let (shell_msg_id, shell_dst0) = take_get_mdata_shell_request(&mut new_node);
    let (entry_msg_id, entry_dst0, _) = unwrap!(take_get_mdata_value_requests(&mut new_node).pop());

    // Simulate receiving failure response. The node should retry the request with
    // different holder.
    unwrap!(new_dm.handle_get_mdata_shell_failure(&mut new_node,
                                                  shell_dst0,
                                                  shell_msg_id));
    assert!(new_dm.get_from_chunk_store(&DataId::mutable(&data)).is_none());

    let (_, shell_dst1) = take_get_mdata_shell_request(&mut new_node);
    assert!(shell_dst0 != shell_dst1);

    // Simulate receiving failure for value request too.
    unwrap!(new_dm.handle_get_mdata_value_failure(&mut new_node,
                                                  entry_dst0,
                                                  entry_msg_id));

    let (_, entry_dst1, _) = unwrap!(take_get_mdata_value_requests(&mut new_node).pop());
    assert!(entry_dst0 != entry_dst1);
}

// Same as `mdata_with_churn` except now X receives invalid fragment.
//
// 1. X receives the response with invalid fragment.
// 2. X sends the requests again, to a different node from the group.
#[test]
fn mdata_with_churn_with_hash_mismatch() {
    let mut rng = rand::thread_rng();

    let (_, client_key) = test_utils::gen_client_authority();
    let data = test_utils::gen_mutable_data(TEST_TAG, 1, client_key, &mut rng);
    let bad_data = test_utils::gen_mutable_data(TEST_TAG, 1, client_key, &mut rng);

    let (mut new_node, mut new_dm) = setup_mdata_refresh(&data, &mut rng);

    let (shell_msg_id, shell_dst0) = take_get_mdata_shell_request(&mut new_node);
    let (entry_msg_id, entry_dst0, _) = unwrap!(take_get_mdata_value_requests(&mut new_node).pop());

    // Simulate malicious node sending wrong data. The node should reject it and retry the request
    // with other holder.
    unwrap!(new_dm.handle_get_mdata_shell_success(&mut new_node,
                                                  shell_dst0,
                                                  bad_data.shell(),
                                                  shell_msg_id));
    assert!(new_dm.get_from_chunk_store(&DataId::mutable(&bad_data)).is_none());

    let (_, shell_dst1) = take_get_mdata_shell_request(&mut new_node);
    assert!(shell_dst0 != shell_dst1);

    let bad_value = unwrap!(bad_data.values().into_iter().next()).clone();
    unwrap!(new_dm.handle_get_mdata_value_success(&mut new_node,
                                                  entry_dst0,
                                                  bad_value,
                                                  entry_msg_id));

    let (_, entry_dst1, _) = unwrap!(take_get_mdata_value_requests(&mut new_node).pop());
    assert!(entry_dst0 != entry_dst1);
}

// Same as `mdata_with_churn`, except now X receives the entry fragments before
// the shell fragments.
//
// 1. X receives response with the entry fragment.
// 2. X receives response with the shell fragment.
// 3. X puts the data into the chunk store and sends no more requests.
#[test]
fn mdata_with_churn_with_entries_arriving_before_shell() {
    let mut rng = rand::thread_rng();

    let (_, client_key) = test_utils::gen_client_authority();
    let data = test_utils::gen_mutable_data(TEST_TAG, 1, client_key, &mut rng);
    let value = unwrap!(data.values().into_iter().cloned().next());

    let (mut new_node, mut new_dm) = setup_mdata_refresh(&data, &mut rng);

    let (shell_msg_id, shell_dst) = take_get_mdata_shell_request(&mut new_node);
    let (entry_msg_id, entry_dst, _) = unwrap!(take_get_mdata_value_requests(&mut new_node).pop());

    // First the entry arrives.
    unwrap!(new_dm.handle_get_mdata_value_success(&mut new_node,
                                                  entry_dst,
                                                  value,
                                                  entry_msg_id));

    // Then the shell arrives.
    unwrap!(new_dm.handle_get_mdata_shell_success(&mut new_node,
                                                  shell_dst,
                                                  data.shell(),
                                                  shell_msg_id));

    let stored_data = assert_match!(new_dm.get_from_chunk_store(&DataId::mutable(&data)),
                                    Some(Data::Mutable(data)) => data);
    assert_eq!(stored_data, data);

    // The node should not send any more requests, because it already has everything it needs.
    assert!(new_node.sent_requests.is_empty());
}

#[test]
fn mdata_parallel_mutations() {
    let mut rng = rand::thread_rng();

    let mut node = RoutingNode::new();
    let mut dm = create_data_manager();

    let (_, client_key_0) = test_utils::gen_client_authority();
    let client_manager_0 = test_utils::gen_client_manager_authority(client_key_0);

    let (_, client_key_1) = test_utils::gen_client_authority();
    let client_manager_1 = test_utils::gen_client_manager_authority(client_key_1);

    let data = test_utils::gen_mutable_data(TEST_TAG, 0, client_key_0, &mut rng);
    dm.put_into_chunk_store(data.clone());
    let nae_manager = Authority::NaeManager(*data.name());

    // Issue two mutations in parallel. Only the first one should result in group
    // refresh being sent.
    let actions = EntryActions::new().ins(b"key0".to_vec(), b"value0".to_vec(), 0).into();
    let msg_id_0 = MessageId::new();
    unwrap!(dm.handle_mutate_mdata_entries(&mut node,
                                           client_manager_0,
                                           nae_manager,
                                           *data.name(),
                                           data.tag(),
                                           actions,
                                           msg_id_0,
                                           client_key_0));

    let actions = EntryActions::new().ins(b"key1".to_vec(), b"value1".to_vec(), 0).into();
    let msg_id_1 = MessageId::new();
    unwrap!(dm.handle_mutate_mdata_entries(&mut node,
                                           client_manager_1,
                                           nae_manager,
                                           *data.name(),
                                           data.tag(),
                                           actions,
                                           msg_id_1,
                                           client_key_1));

    let message = unwrap!(node.sent_requests.remove(&msg_id_0));
    let payload = assert_match!(message.request, Request::Refresh(payload, _) => payload);

    assert!(!node.sent_requests.contains_key(&msg_id_1));

    // After receiving the group refresh, the first mutation succeeds and the second
    // one is rejected.
    unwrap!(dm.handle_group_refresh(&mut node, &payload));

    let message = unwrap!(node.sent_responses.remove(&msg_id_0));
    assert_match!(message.response, Response::MutateMDataEntries { res: Ok(()), .. });

    let message = unwrap!(node.sent_responses.remove(&msg_id_1));
    assert_match!(message.response, Response::MutateMDataEntries { res: Err(_), .. });
}

fn create_data_manager() -> DataManager {
    let suffix: u64 = rand::random();
    let dir = format!("{}_{}", CHUNK_STORE_DIR, suffix);

    unwrap!(DataManager::new(env::temp_dir().join(dir), CHUNK_STORE_CAPACITY))
}

// Create and setup all the objects necessary for churn-related tests.
// Returns:
//   - new node (RoutingNode + DataManager),
//   - names of the rest of the nodes in the group.
fn setup_churn<R: Rng>(rng: &mut R) -> (RoutingNode, DataManager, Vec<XorName>) {
    let mut new_node = RoutingNode::new();
    let new_dm = create_data_manager();

    let old_node_names: Vec<_> = rng.gen_iter().take(GROUP_SIZE - 1).collect();

    for name in &old_node_names {
        new_node.add_to_routing_table(*name);
    }

    (new_node, new_dm, old_node_names)
}

// Create and setup all the objects necessary to test mutable data handling during
// churn:
//   - Create new node
//   - Simulate it receiving refresh messages containing fragments of the given
//     mutable data. The messages accumulate.
//   - Returns the new RoutingNode and DataManager.
fn setup_mdata_refresh<R: Rng>(data: &MutableData, rng: &mut R) -> (RoutingNode, DataManager) {
    let (mut new_node, mut new_dm, old_node_names) = setup_churn(rng);

    let fragments = FragmentInfo::mutable_data(&data);
    let refresh_payload = unwrap!(serialise(&fragments));

    // Node receives the refresh messages from at least QUORUM other nodes and it accumulates.
    for name in old_node_names.iter().take(ACCUMULATOR_QUORUM) {
        unwrap!(new_dm.handle_refresh(&mut new_node, *name, &refresh_payload));
    }

    (new_node, new_dm)
}

// Removes GetMDataShell request from the list of sent requests and returns its message id and
// destination authority name.
fn take_get_mdata_shell_request(node: &mut RoutingNode) -> (MessageId, XorName) {
    let result = node.sent_requests
        .iter()
        .filter_map(|(msg_id, message)| match (&message.request, message.dst) {
            (&Request::GetMDataShell { .. }, Authority::ManagedNode(dst)) => Some((*msg_id, dst)),
            _ => None,
        })
        .next();
    let (msg_id, dst) = unwrap!(result);
    let _ = node.sent_requests.remove(&msg_id);
    (msg_id, dst)
}

// Removes GetMDataValue requests from the list of sent requests and retuns their
// entry keys, message ids and destination authority names.
fn take_get_mdata_value_requests(node: &mut RoutingNode) -> Vec<(MessageId, XorName, Vec<u8>)> {
    let result: Vec<_> = node.sent_requests
        .iter()
        .filter_map(|(msg_id, message)| match (&message.request, message.dst) {
            (&Request::GetMDataValue { ref key, .. }, Authority::ManagedNode(dst)) => {
                Some((*msg_id, dst, key.clone()))
            }
            _ => None,
        })
        .collect();

    for &(msg_id, _, _) in &result {
        let _ = node.sent_requests.remove(&msg_id);
    }

    result
}
