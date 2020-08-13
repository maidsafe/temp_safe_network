use super::Network;
use safe_core::client::tests;

#[tokio::test]
pub async fn pub_blob_test() {
    let _network = Network::new(7).await;
    assert!(tests::pub_blob_test().await.is_ok());
}

#[tokio::test]
async fn unpub_blob_test() {
    assert!(tests::unpub_blob_test().await.is_ok());
}

#[tokio::test]
pub async fn unseq_map_test() {
    assert!(tests::unseq_map_test().await.is_ok())
}

#[tokio::test]
pub async fn seq_map_test() {
    assert!(tests::seq_map_test().await.is_ok());
}

#[tokio::test]
pub async fn del_seq_map_test() {
    assert!(tests::del_seq_map_test().await.is_ok());
}

#[tokio::test]
pub async fn del_unseq_map_test() {
    assert!(tests::del_unseq_map_test().await.is_ok());
}

#[tokio::test]
#[ignore]
async fn money_permissions() {
    tests::money_permissions().await;
}

#[tokio::test]
async fn random_clients() {
    tests::random_clients().await;
}

#[tokio::test]
async fn money_balance_transfer() {
    tests::money_balance_transfer().await;
}

#[tokio::test]
pub async fn del_unseq_map_permission_test() {
    assert!(tests::del_unseq_map_permission_test().await.is_ok());
}

#[tokio::test]
pub async fn map_cannot_initially_put_data_with_another_owner_than_current_client() {
    assert!(
        tests::map_cannot_initially_put_data_with_another_owner_than_current_client()
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn map_can_modify_permissions_test() {
    assert!(tests::map_can_modify_permissions_test().await.is_ok());
}

#[tokio::test]
pub async fn map_mutations_test() {
    assert!(tests::map_mutations_test().await.is_ok());
}

#[tokio::test]
pub async fn blob_deletions_should_cost_put_price() {
    assert!(tests::blob_deletions_should_cost_put_price().await.is_ok());
}

#[tokio::test]
pub async fn map_deletions_should_cost_put_price() {
    assert!(tests::map_deletions_should_cost_put_price().await.is_ok());
}

#[tokio::test]
async fn sequence_deletions_should_cost_put_price() {
    assert!(tests::sequence_deletions_should_cost_put_price()
        .await
        .is_ok());
}

/// Sequence data tests ///

#[tokio::test]
pub async fn sequence_basics_test() {
    assert!(tests::sequence_basics_test().await.is_ok());
}

#[tokio::test]
pub async fn sequence_private_permissions_test() {
    assert!(tests::sequence_private_permissions_test().await.is_ok());
}

#[tokio::test]
pub async fn sequence_pub_permissions_test() {
    assert!(tests::sequence_pub_permissions_test().await.is_ok());
}

#[tokio::test]
pub async fn sequence_append_test() {
    assert!(tests::sequence_append_test().await.is_ok());
}

#[tokio::test]
pub async fn sequence_owner_test() {
    assert!(tests::sequence_owner_test().await.is_ok());
}

#[tokio::test]
pub async fn sequence_can_delete_private_test() {
    assert!(tests::sequence_can_delete_private_test().await.is_ok());
}

#[tokio::test]
pub async fn sequence_cannot_delete_public_test() {
    assert!(tests::sequence_cannot_delete_public_test().await.is_ok());
}
