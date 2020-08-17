// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Network;
use safe_core::client::exported_tests;
use std::sync::Once;

static mut NETWORK: Network = Network { vaults: Vec::new() };
static START: Once = Once::new();

#[allow(unsafe_code)]
fn start_network() {
    START.call_once(|| unsafe {
        NETWORK = futures::executor::block_on(Network::new(7));
    });
}

#[tokio::test]
pub async fn pub_blob_test() {
    start_network();
    assert!(exported_tests::pub_blob_test().await.is_ok());
}

#[tokio::test]
async fn unpub_blob_test() {
    start_network();
    assert!(exported_tests::unpub_blob_test().await.is_ok());
}

#[tokio::test]
pub async fn unseq_map_test() {
    start_network();
    assert!(exported_tests::unseq_map_test().await.is_ok())
}

#[tokio::test]
pub async fn seq_map_test() {
    start_network();
    assert!(exported_tests::seq_map_test().await.is_ok());
}

#[tokio::test]
pub async fn del_seq_map_test() {
    start_network();
    assert!(exported_tests::del_seq_map_test().await.is_ok());
}

#[tokio::test]
pub async fn del_unseq_map_test() {
    assert!(exported_tests::del_unseq_map_test().await.is_ok());
}

#[tokio::test]
#[ignore]
async fn money_permissions() {
    exported_tests::money_permissions().await;
}

#[tokio::test]
async fn random_clients() {
    exported_tests::random_clients().await;
}

#[tokio::test]
async fn money_balance_transfer() {
    exported_tests::money_balance_transfer().await;
}

#[tokio::test]
pub async fn del_unseq_map_permission_test() {
    assert!(exported_tests::del_unseq_map_permission_test()
        .await
        .is_ok());
}

#[tokio::test]
pub async fn map_cannot_initially_put_data_with_another_owner_than_current_client() {
    assert!(
        exported_tests::map_cannot_initially_put_data_with_another_owner_than_current_client()
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn map_can_modify_permissions_test() {
    assert!(exported_tests::map_can_modify_permissions_test()
        .await
        .is_ok());
}

#[tokio::test]
pub async fn map_mutations_test() {
    assert!(exported_tests::map_mutations_test().await.is_ok());
}

#[tokio::test]
pub async fn blob_deletions_should_cost_put_price() {
    assert!(exported_tests::blob_deletions_should_cost_put_price()
        .await
        .is_ok());
}

#[tokio::test]
pub async fn map_deletions_should_cost_put_price() {
    assert!(exported_tests::map_deletions_should_cost_put_price()
        .await
        .is_ok());
}

#[tokio::test]
async fn sequence_deletions_should_cost_put_price() {
    assert!(exported_tests::sequence_deletions_should_cost_put_price()
        .await
        .is_ok());
}

/// Sequence data tests ///

#[tokio::test]
pub async fn sequence_basics_test() {
    assert!(exported_tests::sequence_basics_test().await.is_ok());
}

#[tokio::test]
pub async fn sequence_private_permissions_test() {
    assert!(exported_tests::sequence_private_permissions_test()
        .await
        .is_ok());
}

#[tokio::test]
pub async fn sequence_pub_permissions_test() {
    assert!(exported_tests::sequence_pub_permissions_test()
        .await
        .is_ok());
}

#[tokio::test]
pub async fn sequence_append_test() {
    assert!(exported_tests::sequence_append_test().await.is_ok());
}

#[tokio::test]
pub async fn sequence_owner_test() {
    assert!(exported_tests::sequence_owner_test().await.is_ok());
}

#[tokio::test]
pub async fn sequence_can_delete_private_test() {
    assert!(exported_tests::sequence_can_delete_private_test()
        .await
        .is_ok());
}

#[tokio::test]
pub async fn sequence_cannot_delete_public_test() {
    assert!(exported_tests::sequence_cannot_delete_public_test()
        .await
        .is_ok());
}
