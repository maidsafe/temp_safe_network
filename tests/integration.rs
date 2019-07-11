// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// TODO: make these tests work without mock too.
#![cfg(feature = "mock")]
#![forbid(
    exceeding_bitshifts,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types,
    warnings
)]
#![deny(
    bad_style,
    deprecated,
    improper_ctypes,
    missing_docs,
    non_shorthand_field_patterns,
    overflowing_literals,
    plugin_as_library,
    stable_features,
    unconditional_recursion,
    unknown_lints,
    unsafe_code,
    unused,
    unused_allocation,
    unused_attributes,
    unused_comparisons,
    unused_features,
    unused_parens,
    while_true
)]
#![warn(
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
#![allow(
    box_pointers,
    missing_copy_implementations,
    missing_debug_implementations,
    variant_size_differences
)]

#[macro_use]
mod common;

use self::common::{Environment, TestClientTrait};
use maplit::btreemap;
use rand::{distributions::Standard, Rng};
use safe_nd::{
    AData, ADataAddress, ADataAppend, ADataEntry, ADataIndex, ADataOwner, ADataPubPermissionSet,
    ADataPubPermissions, ADataUnpubPermissionSet, ADataUnpubPermissions, ADataUser, AppPermissions,
    AppendOnlyData, Coins, EntryError, Error as NdError, IData, IDataAddress, LoginPacket, MData,
    MDataAddress, MDataSeqEntryActions, MDataUnseqEntryActions, MDataValue, PubImmutableData,
    PubSeqAppendOnlyData, PubUnseqAppendOnlyData, PublicKey, Request, Response, Result as NdResult,
    SeqAppendOnly, SeqMutableData, UnpubImmutableData, UnpubSeqAppendOnlyData,
    UnpubUnseqAppendOnlyData, UnseqAppendOnly, UnseqMutableData, XorName,
};
use safe_vault::COST_OF_PUT;
use std::collections::BTreeMap;
use unwrap::unwrap;

#[test]
fn client_connects() {
    let mut env = Environment::new();
    let client = env.new_connected_client();
    let _app = env.new_connected_app(client.public_id().clone());
}

////////////////////////////////////////////////////////////////////////////////
//
// Login packets
//
////////////////////////////////////////////////////////////////////////////////

#[test]
fn login_packets() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    let login_packet_data = vec![0; 32];
    let login_packet_locator: XorName = env.rng().gen();

    // Try to get a login packet that does not exist yet.
    let message_id = client.send_request(Request::GetLoginPacket(login_packet_locator));
    env.poll();
    match client.expect_response(message_id) {
        Response::GetLoginPacket(Err(NdError::NoSuchLoginPacket)) => (),
        x => unexpected!(x),
    }

    // Create a new login packet.
    let login_packet = unwrap!(LoginPacket::new(
        login_packet_locator,
        *client.public_id().public_key(),
        login_packet_data.clone(),
        client.sign(&login_packet_data),
    ));

    common::perform_mutation(
        &mut env,
        &mut client,
        Request::CreateLoginPacket(login_packet.clone()),
    );

    // Try to get the login packet data and signature.
    let message_id = client.send_request(Request::GetLoginPacket(login_packet_locator));
    env.poll();
    match client.expect_response(message_id) {
        Response::GetLoginPacket(Ok((data, sig))) => {
            assert_eq!(data, login_packet_data);

            match client.public_id().public_key().verify(&sig, &data) {
                Ok(()) => (),
                x => unexpected!(x),
            }
        }
        x => unexpected!(x),
    }

    // Putting login packet to the same address should fail.
    let message_id = client.send_request(Request::CreateLoginPacket(login_packet));
    env.poll();
    match client.expect_response(message_id) {
        Response::Mutation(Err(NdError::LoginPacketExists)) => (),
        x => unexpected!(x),
    }

    // Getting login packet from non-owning client should fail.
    {
        let mut client = env.new_connected_client();
        let message_id = client.send_request(Request::GetLoginPacket(login_packet_locator));
        env.poll();
        match client.expect_response(message_id) {
            Response::GetLoginPacket(Err(NdError::AccessDenied)) => (),
            x => unexpected!(x),
        }
    }
}

#[test]
fn update_login_packet() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    let login_packet_data = vec![0; 32];
    let login_packet_locator: XorName = env.rng().gen();

    // Create a new login packet.
    let login_packet = unwrap!(LoginPacket::new(
        login_packet_locator,
        *client.public_id().public_key(),
        login_packet_data.clone(),
        client.sign(&login_packet_data),
    ));

    common::perform_mutation(
        &mut env,
        &mut client,
        Request::CreateLoginPacket(login_packet.clone()),
    );

    // Update the login packet data.
    let new_login_packet_data = vec![1; 32];
    let client_public_key = *client.public_id().public_key();
    let signature = client.sign(&new_login_packet_data);
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::UpdateLoginPacket(unwrap!(LoginPacket::new(
            login_packet_locator,
            client_public_key,
            new_login_packet_data.clone(),
            signature,
        ))),
    );

    // Try to get the login packet data and signature.
    let message_id = client.send_request(Request::GetLoginPacket(login_packet_locator));
    env.poll();

    match client.expect_response(message_id) {
        Response::GetLoginPacket(Ok((data, sig))) => {
            assert_eq!(data, new_login_packet_data);
            unwrap!(client.public_id().public_key().verify(&sig, &data));
        }
        x => unexpected!(x),
    }

    // Updating login packet from non-owning client should fail.
    {
        let mut client = env.new_connected_client();

        let message_id = client.send_request(Request::UpdateLoginPacket(login_packet.clone()));
        env.poll();

        match client.expect_response(message_id) {
            Response::Mutation(Err(NdError::AccessDenied)) => (),
            x => unexpected!(x),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//
// Coins
//
////////////////////////////////////////////////////////////////////////////////

#[test]
fn coin_operations() {
    let mut env = Environment::new();

    let mut client_a = env.new_connected_client();
    let mut client_b = env.new_connected_client();

    let balance = common::get_balance(&mut env, &mut client_a);
    assert_eq!(balance, unwrap!(Coins::from_nano(0)));

    // Create A's balance
    let public_key = *client_a.public_id().public_key();
    let message_id = client_a.send_request(Request::CreateBalance {
        new_balance_owner: public_key,
        amount: unwrap!(Coins::from_nano(10)),
        transaction_id: 0,
    });
    env.poll();

    match client_a.expect_response(message_id) {
        Response::Transaction(Ok(transaction)) => {
            assert_eq!(transaction.id, 0);
            assert_eq!(transaction.amount, unwrap!(Coins::from_nano(10)))
        }
        x => unexpected!(x),
    }

    let balance = common::get_balance(&mut env, &mut client_a);
    assert_eq!(balance, unwrap!(Coins::from_nano(10)));

    // Create B's balance
    let message_id = client_a.send_request(Request::CreateBalance {
        new_balance_owner: *client_b.public_id().public_key(),
        amount: unwrap!(Coins::from_nano(1)),
        transaction_id: 0,
    });
    env.poll();

    match client_a.expect_response(message_id) {
        Response::Transaction(Ok(transaction)) => {
            assert_eq!(transaction.id, 0);
            assert_eq!(transaction.amount, unwrap!(Coins::from_nano(1)))
        }
        x => unexpected!(x),
    }

    let balance_a = common::get_balance(&mut env, &mut client_a);
    let balance_b = common::get_balance(&mut env, &mut client_b);
    assert_eq!(balance_a, unwrap!(Coins::from_nano(9)));
    assert_eq!(balance_b, unwrap!(Coins::from_nano(1)));

    // Transfer coins from A to B
    let message_id = client_a.send_request(Request::TransferCoins {
        destination: *client_b.public_id().name(),
        amount: unwrap!(Coins::from_nano(2)),
        transaction_id: 1,
    });
    env.poll();

    match client_a.expect_response(message_id) {
        Response::Transaction(Ok(transaction)) => {
            assert_eq!(transaction.id, 1);
            assert_eq!(transaction.amount, unwrap!(Coins::from_nano(2)))
        }
        x => unexpected!(x),
    }

    let balance_a = common::get_balance(&mut env, &mut client_a);
    let balance_b = common::get_balance(&mut env, &mut client_b);
    assert_eq!(balance_a, unwrap!(Coins::from_nano(7)));
    assert_eq!(balance_b, unwrap!(Coins::from_nano(3)));
}

////////////////////////////////////////////////////////////////////////////////
//
// Append-only data
//
////////////////////////////////////////////////////////////////////////////////

#[test]
fn put_append_only_data() {
    let mut env = Environment::new();
    let mut client_a = env.new_connected_client();
    let mut client_b = env.new_connected_client();

    let owner_a = ADataOwner {
        public_key: *client_a.public_id().public_key(),
        entries_index: 0,
        permissions_index: 0,
    };

    // Published sequential data
    let pub_seq_adata_name: XorName = env.rng().gen();
    let mut pub_seq_adata = PubSeqAppendOnlyData::new(pub_seq_adata_name, 100);
    unwrap!(pub_seq_adata.append_owner(owner_a, 0));
    unwrap!(pub_seq_adata.append(
        vec![ADataEntry {
            key: b"one".to_vec(),
            value: b"pub sec".to_vec()
        }],
        0
    ));
    unwrap!(pub_seq_adata.append(
        vec![ADataEntry {
            key: b"two".to_vec(),
            value: b"pub sec".to_vec()
        }],
        1
    ));
    let pub_seq_adata = AData::PubSeq(pub_seq_adata);

    // Published unsequential data
    let pub_unseq_adata_name: XorName = env.rng().gen();
    let mut pub_unseq_adata = PubUnseqAppendOnlyData::new(pub_unseq_adata_name, 100);
    unwrap!(pub_unseq_adata.append_owner(owner_a, 0));
    unwrap!(pub_unseq_adata.append(vec![ADataEntry {
        key: b"one".to_vec(),
        value: b"pub unsec".to_vec()
    }]));
    unwrap!(pub_unseq_adata.append(vec![ADataEntry {
        key: b"two".to_vec(),
        value: b"pub unsec".to_vec()
    }]));
    let pub_unseq_adata = AData::PubUnseq(pub_unseq_adata);

    // Unpublished sequential
    let unpub_seq_adata_name: XorName = env.rng().gen();
    let mut unpub_seq_adata = UnpubSeqAppendOnlyData::new(unpub_seq_adata_name, 100);
    unwrap!(unpub_seq_adata.append_owner(owner_a, 0));
    unwrap!(unpub_seq_adata.append(
        vec![ADataEntry {
            key: b"one".to_vec(),
            value: b"unpub sec".to_vec()
        }],
        0
    ));
    unwrap!(unpub_seq_adata.append(
        vec![ADataEntry {
            key: b"two".to_vec(),
            value: b"unpub sec".to_vec()
        }],
        1
    ));
    let unpub_seq_adata = AData::UnpubSeq(unpub_seq_adata);

    // Unpublished unsequential data
    let unpub_unseq_adata_name: XorName = env.rng().gen();
    let mut unpub_unseq_adata = UnpubUnseqAppendOnlyData::new(unpub_unseq_adata_name, 100);
    unwrap!(unpub_unseq_adata.append_owner(owner_a, 0));
    unwrap!(unpub_unseq_adata.append(vec![ADataEntry {
        key: b"one".to_vec(),
        value: b"unpub unsec".to_vec()
    }]));
    unwrap!(unpub_unseq_adata.append(vec![ADataEntry {
        key: b"two".to_vec(),
        value: b"unpub unsec".to_vec()
    }]));
    let unpub_unseq_adata = AData::UnpubUnseq(unpub_unseq_adata);

    // TODO - Enable this once we're passed phase 1
    // First try to put some data without any associated balance.
    if false {
        common::send_request_expect_response(
            &mut env,
            &mut client_a,
            Request::PutAData(pub_seq_adata.clone()),
            Response::Mutation(Err(NdError::AccessDenied)),
        );
        common::send_request_expect_response(
            &mut env,
            &mut client_a,
            Request::PutAData(pub_unseq_adata.clone()),
            Response::Mutation(Err(NdError::AccessDenied)),
        );
        common::send_request_expect_response(
            &mut env,
            &mut client_a,
            Request::PutAData(unpub_seq_adata.clone()),
            Response::Mutation(Err(NdError::AccessDenied)),
        );
        common::send_request_expect_response(
            &mut env,
            &mut client_a,
            Request::PutAData(unpub_unseq_adata.clone()),
            Response::Mutation(Err(NdError::AccessDenied)),
        );
    }

    let start_nano = 1_000_000_000_000;
    let new_balance_owner = *client_a.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client_a,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    // Check that client B cannot put A's data
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::PutAData(pub_seq_adata.clone()),
        Response::Mutation(Err(NdError::InvalidOwners)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::PutAData(pub_unseq_adata.clone()),
        Response::Mutation(Err(NdError::InvalidOwners)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::PutAData(unpub_seq_adata.clone()),
        Response::Mutation(Err(NdError::InvalidOwners)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::PutAData(unpub_unseq_adata.clone()),
        Response::Mutation(Err(NdError::InvalidOwners)),
    );

    // Put, this time with a balance and the correct owner
    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::PutAData(pub_seq_adata.clone()),
    );
    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::PutAData(pub_unseq_adata.clone()),
    );
    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::PutAData(unpub_seq_adata.clone()),
    );
    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::PutAData(unpub_unseq_adata.clone()),
    );

    // Get the data to verify
    let message_id_1 = client_a.send_request(Request::GetAData(*pub_seq_adata.address()));
    let message_id_2 = client_a.send_request(Request::GetAData(*pub_unseq_adata.address()));
    let message_id_3 = client_a.send_request(Request::GetAData(*unpub_seq_adata.address()));
    let message_id_4 = client_a.send_request(Request::GetAData(*unpub_unseq_adata.address()));
    env.poll();
    match client_a.expect_response(message_id_1) {
        Response::GetAData(Ok(got)) => assert_eq!(got, pub_seq_adata),
        x => unexpected!(x),
    }
    match client_a.expect_response(message_id_2) {
        Response::GetAData(Ok(got)) => assert_eq!(got, pub_unseq_adata),
        x => unexpected!(x),
    }
    match client_a.expect_response(message_id_3) {
        Response::GetAData(Ok(got)) => assert_eq!(got, unpub_seq_adata),
        x => unexpected!(x),
    }
    match client_a.expect_response(message_id_4) {
        Response::GetAData(Ok(got)) => assert_eq!(got, unpub_unseq_adata),
        x => unexpected!(x),
    }

    // Verify that B cannot delete A's data
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::DeleteAData(*pub_seq_adata.address()),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::DeleteAData(*pub_unseq_adata.address()),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::DeleteAData(*unpub_seq_adata.address()),
        Response::Mutation(Err(NdError::AccessDenied)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::DeleteAData(*unpub_unseq_adata.address()),
        Response::Mutation(Err(NdError::AccessDenied)),
    );

    // Delete the data
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::DeleteAData(*pub_seq_adata.address()),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::DeleteAData(*pub_unseq_adata.address()),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::DeleteAData(*unpub_seq_adata.address()),
    );
    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::DeleteAData(*unpub_unseq_adata.address()),
    );

    // Delete again to test if it's gone
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::DeleteAData(*unpub_seq_adata.address()),
        Response::Mutation(Err(NdError::NoSuchData)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::DeleteAData(*unpub_unseq_adata.address()),
        Response::Mutation(Err(NdError::NoSuchData)),
    );
}

#[test]
fn append_only_data_delete_data_doesnt_exist() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    let name: XorName = env.rng().gen();
    let tag = 100;

    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::DeleteAData(*AData::PubSeq(PubSeqAppendOnlyData::new(name, tag)).address()),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::DeleteAData(*AData::PubUnseq(PubUnseqAppendOnlyData::new(name, tag)).address()),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::DeleteAData(*AData::UnpubSeq(UnpubSeqAppendOnlyData::new(name, tag)).address()),
        Response::Mutation(Err(NdError::AccessDenied)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::DeleteAData(
            *AData::UnpubUnseq(UnpubUnseqAppendOnlyData::new(name, tag)).address(),
        ),
        Response::Mutation(Err(NdError::AccessDenied)),
    );

    let start_nano = 1_000_000_000_000;
    let new_balance_owner = *client.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::DeleteAData(*AData::PubSeq(PubSeqAppendOnlyData::new(name, tag)).address()),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::DeleteAData(*AData::PubUnseq(PubUnseqAppendOnlyData::new(name, tag)).address()),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::DeleteAData(*AData::UnpubSeq(UnpubSeqAppendOnlyData::new(name, tag)).address()),
        Response::Mutation(Err(NdError::NoSuchData)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::DeleteAData(
            *AData::UnpubUnseq(UnpubUnseqAppendOnlyData::new(name, tag)).address(),
        ),
        Response::Mutation(Err(NdError::NoSuchData)),
    );
}

#[test]
fn get_pub_append_only_data() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    let mut data = PubSeqAppendOnlyData::new(env.rng().gen(), 100);

    let owner = ADataOwner {
        public_key: *client.public_id().public_key(),
        entries_index: 0,
        permissions_index: 0,
    };
    unwrap!(data.append_owner(owner, 0));

    let data = AData::PubSeq(data);
    let address = *data.address();
    common::perform_mutation(&mut env, &mut client, Request::PutAData(data.clone()));

    // Success
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetAData(address),
        Response::GetAData(Ok(data.clone())),
    );

    // Failure - non-existing data
    let invalid_name: XorName = env.rng().gen();
    let invalid_address = ADataAddress::PubSeq {
        name: invalid_name,
        tag: 100,
    };

    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetAData(invalid_address),
        Response::GetAData(Err(NdError::NoSuchData)),
    );

    // Published data is gettable by non-owners too
    let mut other_client = env.new_connected_client();
    common::send_request_expect_response(
        &mut env,
        &mut other_client,
        Request::GetAData(address),
        Response::GetAData(Ok(data)),
    );
}

#[test]
fn get_unpub_append_only_data() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    let new_balance_owner = *client.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(0)),
            transaction_id: 0,
        },
    );

    let mut data = UnpubSeqAppendOnlyData::new(env.rng().gen(), 100);

    let owner = ADataOwner {
        public_key: *client.public_id().public_key(),
        entries_index: 0,
        permissions_index: 0,
    };
    unwrap!(data.append_owner(owner, 0));

    let data = AData::UnpubSeq(data);
    let address = *data.address();
    common::perform_mutation(&mut env, &mut client, Request::PutAData(data.clone()));

    // Success
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetAData(address),
        Response::GetAData(Ok(data.clone())),
    );

    // Failure - non-existing data
    let invalid_name: XorName = env.rng().gen();
    let invalid_address = ADataAddress::UnpubSeq {
        name: invalid_name,
        tag: 100,
    };

    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetAData(invalid_address),
        Response::GetAData(Err(NdError::NoSuchData)),
    );

    // Failure - get by non-owner not allowed
    let mut other_client = env.new_connected_client();
    let new_balance_owner = *other_client.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut other_client,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(0)),
            transaction_id: 0,
        },
    );

    common::send_request_expect_response(
        &mut env,
        &mut other_client,
        Request::GetAData(address),
        Response::GetAData(Err(NdError::InvalidPermissions)),
    );
}

#[test]
fn append_only_data_get_entries() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    let mut data = PubSeqAppendOnlyData::new(env.rng().gen(), 100);

    let owner = ADataOwner {
        public_key: *client.public_id().public_key(),
        entries_index: 0,
        permissions_index: 0,
    };

    unwrap!(data.append_owner(owner, 0));
    unwrap!(data.append(
        vec![
            ADataEntry {
                key: b"one".to_vec(),
                value: b"foo".to_vec()
            },
            ADataEntry {
                key: b"two".to_vec(),
                value: b"bar".to_vec()
            },
        ],
        0,
    ));

    let data = AData::PubSeq(data);
    let address = *data.address();
    common::perform_mutation(&mut env, &mut client, Request::PutAData(data.clone()));

    // GetADataRange
    let mut range_scenario = |start, end, expected_result| {
        common::send_request_expect_response(
            &mut env,
            &mut client,
            Request::GetADataRange {
                address,
                range: (start, end),
            },
            Response::GetADataRange(expected_result),
        )
    };

    range_scenario(
        ADataIndex::FromStart(0),
        ADataIndex::FromStart(0),
        Ok(vec![]),
    );
    range_scenario(
        ADataIndex::FromStart(0),
        ADataIndex::FromStart(1),
        Ok(vec![ADataEntry {
            key: b"one".to_vec(),
            value: b"foo".to_vec(),
        }]),
    );
    range_scenario(
        ADataIndex::FromStart(1),
        ADataIndex::FromStart(2),
        Ok(vec![ADataEntry {
            key: b"two".to_vec(),
            value: b"bar".to_vec(),
        }]),
    );
    range_scenario(
        ADataIndex::FromEnd(1),
        ADataIndex::FromEnd(0),
        Ok(vec![ADataEntry {
            key: b"two".to_vec(),
            value: b"bar".to_vec(),
        }]),
    );
    range_scenario(
        ADataIndex::FromStart(0),
        ADataIndex::FromEnd(0),
        Ok(vec![
            ADataEntry {
                key: b"one".to_vec(),
                value: b"foo".to_vec(),
            },
            ADataEntry {
                key: b"two".to_vec(),
                value: b"bar".to_vec(),
            },
        ]),
    );
    range_scenario(
        ADataIndex::FromStart(0),
        ADataIndex::FromStart(3),
        Err(NdError::NoSuchEntry),
    );

    // GetADataLastEntry
    let expected = ADataEntry {
        key: b"two".to_vec(),
        value: b"bar".to_vec(),
    };
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetADataLastEntry(address),
        Response::GetADataLastEntry(Ok(expected)),
    );

    // GetADataValue
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetADataValue {
            address,
            key: b"one".to_vec(),
        },
        Response::GetADataValue(Ok(b"foo".to_vec())),
    );
}

#[test]
fn append_only_data_get_owners() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    let name: XorName = env.rng().gen();
    let tag = 100;
    let mut data = PubSeqAppendOnlyData::new(name, tag);

    let owner_0 = ADataOwner {
        public_key: common::gen_public_key(env.rng()),
        entries_index: 0,
        permissions_index: 0,
    };
    let owner_1 = ADataOwner {
        public_key: common::gen_public_key(env.rng()),
        entries_index: 0,
        permissions_index: 0,
    };
    let owner_2 = ADataOwner {
        public_key: *client.public_id().public_key(),
        entries_index: 1,
        permissions_index: 0,
    };

    unwrap!(data.append_owner(owner_0, 0));
    unwrap!(data.append_owner(owner_1, 1));

    unwrap!(data.append(
        vec![ADataEntry {
            key: b"one".to_vec(),
            value: b"foo".to_vec()
        }],
        0
    ));
    unwrap!(data.append_owner(owner_2, 2));

    let address = *data.address();
    common::perform_mutation(&mut env, &mut client, Request::PutAData(data.into()));

    let mut scenario = |owners_index, expected_result| {
        common::send_request_expect_response(
            &mut env,
            &mut client,
            Request::GetADataOwners {
                address,
                owners_index,
            },
            Response::GetADataOwners(expected_result),
        );
    };

    scenario(ADataIndex::FromStart(0), Ok(owner_0));
    scenario(ADataIndex::FromStart(1), Ok(owner_1));
    scenario(ADataIndex::FromStart(2), Ok(owner_2));
    scenario(ADataIndex::FromStart(3), Err(NdError::InvalidOwners));

    scenario(ADataIndex::FromEnd(0), Err(NdError::InvalidOwners));
    scenario(ADataIndex::FromEnd(1), Ok(owner_2));
    scenario(ADataIndex::FromEnd(2), Ok(owner_1));
    scenario(ADataIndex::FromEnd(3), Ok(owner_0));
    scenario(ADataIndex::FromEnd(4), Err(NdError::InvalidOwners));
}

#[test]
fn pub_append_only_data_get_permissions() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    let name: XorName = env.rng().gen();
    let tag = 100;
    let mut data = PubSeqAppendOnlyData::new(name, tag);

    let owner = ADataOwner {
        public_key: *client.public_id().public_key(),
        entries_index: 0,
        permissions_index: 0,
    };

    unwrap!(data.append_owner(owner, 0));

    let perms_0 = ADataPubPermissions {
        permissions: btreemap![ADataUser::Anyone => ADataPubPermissionSet::new(true, false)],
        entries_index: 0,
        owners_index: 1,
    };
    unwrap!(data.append_permissions(perms_0.clone(), 0));

    let public_key = common::gen_public_key(env.rng());
    let perms_1 = ADataPubPermissions {
        permissions: btreemap![
            ADataUser::Anyone => ADataPubPermissionSet::new(false, false),
            ADataUser::Key(public_key) => ADataPubPermissionSet::new(true, false)
        ],
        entries_index: 0,
        owners_index: 1,
    };
    unwrap!(data.append_permissions(perms_1.clone(), 1));

    let address = *data.address();
    common::perform_mutation(&mut env, &mut client, Request::PutAData(data.into()));

    // GetPubADataUserPermissions
    let mut scenario = |permissions_index, user, expected_response| {
        common::send_request_expect_response(
            &mut env,
            &mut client,
            Request::GetPubADataUserPermissions {
                address,
                permissions_index,
                user,
            },
            Response::GetPubADataUserPermissions(expected_response),
        );
    };

    scenario(
        ADataIndex::FromStart(0),
        ADataUser::Anyone,
        Ok(ADataPubPermissionSet::new(true, false)),
    );
    scenario(
        ADataIndex::FromStart(0),
        ADataUser::Key(public_key),
        Err(NdError::NoSuchEntry),
    );
    scenario(
        ADataIndex::FromStart(1),
        ADataUser::Anyone,
        Ok(ADataPubPermissionSet::new(false, false)),
    );
    scenario(
        ADataIndex::FromStart(1),
        ADataUser::Key(public_key),
        Ok(ADataPubPermissionSet::new(true, false)),
    );
    scenario(
        ADataIndex::FromStart(2),
        ADataUser::Anyone,
        Err(NdError::NoSuchEntry),
    );

    scenario(
        ADataIndex::FromEnd(1),
        ADataUser::Anyone,
        Ok(ADataPubPermissionSet::new(false, false)),
    );
    scenario(
        ADataIndex::FromEnd(2),
        ADataUser::Anyone,
        Ok(ADataPubPermissionSet::new(true, false)),
    );
    scenario(
        ADataIndex::FromEnd(3),
        ADataUser::Anyone,
        Err(NdError::NoSuchEntry),
    );

    // GetUnpubADataUserPermissions (failure - incorrect data kind)
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetUnpubADataUserPermissions {
            address,
            permissions_index: ADataIndex::FromStart(1),
            public_key,
        },
        Response::GetUnpubADataUserPermissions(Err(NdError::NoSuchData)),
    );

    // GetADataPermissions
    let mut scenario = |permissions_index, expected_result| {
        common::send_request_expect_response(
            &mut env,
            &mut client,
            Request::GetADataPermissions {
                address,
                permissions_index,
            },
            Response::GetPubADataPermissionAtIndex(expected_result),
        );
    };

    scenario(ADataIndex::FromStart(0), Ok(perms_0));
    scenario(ADataIndex::FromStart(1), Ok(perms_1));
    scenario(ADataIndex::FromStart(2), Err(NdError::NoSuchEntry));
}

#[test]
fn unpub_append_only_data_get_permissions() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    let start_nano = 1_000_000_000_000;
    let new_balance_owner = *client.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    let name: XorName = env.rng().gen();
    let tag = 100;
    let mut data = UnpubSeqAppendOnlyData::new(name, tag);

    let owner = ADataOwner {
        public_key: *client.public_id().public_key(),
        entries_index: 0,
        permissions_index: 0,
    };

    unwrap!(data.append_owner(owner, 0));

    let public_key_0 = common::gen_public_key(env.rng());
    let public_key_1 = common::gen_public_key(env.rng());

    let perms_0 = ADataUnpubPermissions {
        permissions: btreemap![
            public_key_0 => ADataUnpubPermissionSet::new(true, true, false)
        ],
        entries_index: 0,
        owners_index: 1,
    };
    unwrap!(data.append_permissions(perms_0.clone(), 0));

    let perms_1 = ADataUnpubPermissions {
        permissions: btreemap![
            public_key_0 => ADataUnpubPermissionSet::new(true, false, false),
            public_key_1 => ADataUnpubPermissionSet::new(true, true, true)
        ],
        entries_index: 0,
        owners_index: 1,
    };
    unwrap!(data.append_permissions(perms_1.clone(), 1));

    let address = *data.address();
    common::perform_mutation(&mut env, &mut client, Request::PutAData(data.into()));

    // GetUnpubADataUserPermissions
    let mut scenario = |permissions_index, public_key, expected_response| {
        common::send_request_expect_response(
            &mut env,
            &mut client,
            Request::GetUnpubADataUserPermissions {
                address,
                permissions_index,
                public_key,
            },
            Response::GetUnpubADataUserPermissions(expected_response),
        );
    };

    scenario(
        ADataIndex::FromStart(0),
        public_key_0,
        Ok(ADataUnpubPermissionSet::new(true, true, false)),
    );
    scenario(
        ADataIndex::FromStart(0),
        public_key_1,
        Err(NdError::NoSuchEntry),
    );
    scenario(
        ADataIndex::FromStart(1),
        public_key_0,
        Ok(ADataUnpubPermissionSet::new(true, false, false)),
    );
    scenario(
        ADataIndex::FromStart(1),
        public_key_1,
        Ok(ADataUnpubPermissionSet::new(true, true, true)),
    );
    scenario(
        ADataIndex::FromStart(2),
        public_key_0,
        Err(NdError::NoSuchEntry),
    );

    scenario(
        ADataIndex::FromEnd(1),
        public_key_0,
        Ok(ADataUnpubPermissionSet::new(true, false, false)),
    );
    scenario(
        ADataIndex::FromEnd(2),
        public_key_0,
        Ok(ADataUnpubPermissionSet::new(true, true, false)),
    );
    scenario(
        ADataIndex::FromEnd(3),
        public_key_0,
        Err(NdError::NoSuchEntry),
    );

    // GetPubADataUserPermissions (failure - incorrect data kind)
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetPubADataUserPermissions {
            address,
            permissions_index: ADataIndex::FromStart(1),
            user: ADataUser::Key(public_key_0),
        },
        Response::GetPubADataUserPermissions(Err(NdError::NoSuchData)),
    );

    // GetADataPermissions
    let mut scenario = |permissions_index, expected_result| {
        common::send_request_expect_response(
            &mut env,
            &mut client,
            Request::GetADataPermissions {
                address,
                permissions_index,
            },
            Response::GetUnpubADataPermissionAtIndex(expected_result),
        );
    };

    scenario(ADataIndex::FromStart(0), Ok(perms_0));
    scenario(ADataIndex::FromStart(1), Ok(perms_1));
    scenario(ADataIndex::FromStart(2), Err(NdError::NoSuchEntry));
}

#[test]
fn pub_append_only_data_put_permissions() {
    let mut env = Environment::new();
    let mut client_a = env.new_connected_client();
    let mut client_b = env.new_connected_client();

    let public_key_a = *client_a.public_id().public_key();
    let public_key_b = *client_b.public_id().public_key();

    let start_nano = 1_000_000_000_000;
    common::perform_transaction(
        &mut env,
        &mut client_a,
        Request::CreateBalance {
            new_balance_owner: public_key_a,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );
    common::perform_transaction(
        &mut env,
        &mut client_b,
        Request::CreateBalance {
            new_balance_owner: public_key_b,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    let name: XorName = env.rng().gen();
    let tag = 100;
    let mut data = PubSeqAppendOnlyData::new(name, tag);

    let owner = ADataOwner {
        public_key: *client_a.public_id().public_key(),
        entries_index: 0,
        permissions_index: 0,
    };

    unwrap!(data.append_owner(owner, 0));

    // Client A can manage permissions, but not B
    let perms_0 = ADataPubPermissions {
        permissions: btreemap![ADataUser::Key(public_key_a) => ADataPubPermissionSet::new(true, true)],
        entries_index: 0,
        owners_index: 1,
    };
    unwrap!(data.append_permissions(perms_0.clone(), 0));

    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::PutAData(AData::PubSeq(data.clone())),
    );

    // Before
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::GetADataPermissions {
            address: *data.address(),
            permissions_index: ADataIndex::FromStart(0),
        },
        Response::GetPubADataPermissionAtIndex(Ok(perms_0)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::GetADataPermissions {
            address: *data.address(),
            permissions_index: ADataIndex::FromStart(1),
        },
        Response::GetPubADataPermissionAtIndex(Err(NdError::NoSuchEntry)),
    );

    let perms_1 = ADataPubPermissions {
        permissions: btreemap![
            ADataUser::Key(public_key_b) => ADataPubPermissionSet::new(true, true)
        ],
        entries_index: 0,
        owners_index: 1,
    };

    // Only client A has permissions to add permissions
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::AddPubADataPermissions {
            address: *data.address(),
            permissions: perms_1.clone(),
            permissions_idx: 1,
        },
        // TODO: InvalidPermissions because client B doesn't have any key avail. We should consider
        // changing this behaviour to AccessDenied.
        Response::Mutation(Err(NdError::InvalidPermissions)),
    );

    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::AddPubADataPermissions {
            address: *data.address(),
            permissions: perms_1.clone(),
            permissions_idx: 1,
        },
    );

    // Check that the permissions have been updated
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::GetADataPermissions {
            address: *data.address(),
            permissions_index: ADataIndex::FromStart(1),
        },
        Response::GetPubADataPermissionAtIndex(Ok(perms_1)),
    );
}

#[test]
fn unpub_append_only_data_put_permissions() {
    let mut env = Environment::new();
    let mut client_a = env.new_connected_client();
    let mut client_b = env.new_connected_client();

    let public_key_a = *client_a.public_id().public_key();
    let public_key_b = *client_b.public_id().public_key();

    let start_nano = 1_000_000_000_000;
    common::perform_transaction(
        &mut env,
        &mut client_a,
        Request::CreateBalance {
            new_balance_owner: public_key_a,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );
    common::perform_transaction(
        &mut env,
        &mut client_b,
        Request::CreateBalance {
            new_balance_owner: public_key_b,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    let name: XorName = env.rng().gen();
    let tag = 100;
    let mut data = UnpubSeqAppendOnlyData::new(name, tag);

    let owner = ADataOwner {
        public_key: *client_a.public_id().public_key(),
        entries_index: 0,
        permissions_index: 0,
    };

    unwrap!(data.append_owner(owner, 0));

    // Client A can manage permissions, but not B
    let perms_0 = ADataUnpubPermissions {
        permissions: btreemap![public_key_a => ADataUnpubPermissionSet::new(true, true, true)],
        entries_index: 0,
        owners_index: 1,
    };
    unwrap!(data.append_permissions(perms_0.clone(), 0));

    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::PutAData(AData::UnpubSeq(data.clone())),
    );

    // Before
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::GetADataPermissions {
            address: *data.address(),
            permissions_index: ADataIndex::FromStart(0),
        },
        Response::GetUnpubADataPermissionAtIndex(Ok(perms_0)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::GetADataPermissions {
            address: *data.address(),
            permissions_index: ADataIndex::FromStart(1),
        },
        Response::GetUnpubADataPermissionAtIndex(Err(NdError::NoSuchEntry)),
    );

    let perms_1 = ADataUnpubPermissions {
        permissions: btreemap![
            public_key_b => ADataUnpubPermissionSet::new(true, true, true)
        ],
        entries_index: 0,
        owners_index: 1,
    };

    // Only client A has permissions to add permissions
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::AddUnpubADataPermissions {
            address: *data.address(),
            permissions: perms_1.clone(),
            permissions_idx: 1,
        },
        // TODO: InvalidPermissions because client B doesn't have any key avail. We should consider
        // changing this behaviour to AccessDenied.
        Response::Mutation(Err(NdError::InvalidPermissions)),
    );

    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::AddUnpubADataPermissions {
            address: *data.address(),
            permissions: perms_1.clone(),
            permissions_idx: 1,
        },
    );

    // Check that the permissions have been updated
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::GetADataPermissions {
            address: *data.address(),
            permissions_index: ADataIndex::FromStart(1),
        },
        Response::GetUnpubADataPermissionAtIndex(Ok(perms_1)),
    );
}

#[test]
fn append_only_data_put_owners() {
    let mut env = Environment::new();
    let mut client_a = env.new_connected_client();
    let mut client_b = env.new_connected_client();

    let public_key_a = *client_a.public_id().public_key();
    let public_key_b = *client_b.public_id().public_key();

    let start_nano = 1_000_000_000_000;
    common::perform_transaction(
        &mut env,
        &mut client_a,
        Request::CreateBalance {
            new_balance_owner: public_key_a,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );
    common::perform_transaction(
        &mut env,
        &mut client_b,
        Request::CreateBalance {
            new_balance_owner: public_key_b,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    let name: XorName = env.rng().gen();
    let tag = 100;
    let mut data = PubSeqAppendOnlyData::new(name, tag);

    let owner_0 = ADataOwner {
        public_key: public_key_a,
        entries_index: 0,
        permissions_index: 0,
    };
    unwrap!(data.append_owner(owner_0, 0));

    let perms_0 = ADataPubPermissions {
        permissions: btreemap![ADataUser::Key(public_key_a) => ADataPubPermissionSet::new(true, true)],
        entries_index: 0,
        owners_index: 1,
    };

    unwrap!(data.append_permissions(perms_0.clone(), 0));
    unwrap!(data.append(
        vec![ADataEntry {
            key: b"one".to_vec(),
            value: b"foo".to_vec()
        }],
        0
    ));
    unwrap!(data.append(
        vec![ADataEntry {
            key: b"two".to_vec(),
            value: b"foo".to_vec()
        }],
        1
    ));

    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::PutAData(data.clone().into()),
    );

    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::GetADataOwners {
            address: *data.address(),
            owners_index: ADataIndex::FromStart(0),
        },
        Response::GetADataOwners(Ok(owner_0)),
    );
    // Neither A or B can get the owners with index 1 (it doesn't exist)
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::GetADataOwners {
            address: *data.address(),
            owners_index: ADataIndex::FromStart(1),
        },
        Response::GetADataOwners(Err(NdError::InvalidOwners)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::GetADataOwners {
            address: *data.address(),
            owners_index: ADataIndex::FromStart(1),
        },
        Response::GetADataOwners(Err(NdError::InvalidOwners)),
    );

    // Set the new owner, change from A -> B
    let owner_1 = ADataOwner {
        public_key: public_key_b,
        entries_index: 2,
        permissions_index: 1,
    };

    // B can't set the new owner, but A can
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::SetADataOwner {
            address: *data.address(),
            owner: owner_1,
            owners_idx: 1,
        },
        // TODO - InvalidPermissions because client B doesn't have their key registered. Maybe we
        //        should consider changing this.
        Response::Mutation(Err(NdError::InvalidPermissions)),
    );
    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::SetADataOwner {
            address: *data.address(),
            owner: owner_1,
            owners_idx: 1,
        },
    );

    // Check the new owner
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::GetADataOwners {
            address: *data.address(),
            owners_index: ADataIndex::FromStart(1),
        },
        Response::GetADataOwners(Ok(owner_1)),
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::GetADataOwners {
            address: *data.address(),
            owners_index: ADataIndex::FromStart(1),
        },
        Response::GetADataOwners(Ok(owner_1)),
    );
}

#[test]
fn append_only_data_append_seq() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();
    let public_key = *client.public_id().public_key();

    let start_nano = 1_000_000_000_000;
    common::perform_transaction(
        &mut env,
        &mut client,
        Request::CreateBalance {
            new_balance_owner: public_key,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    let name: XorName = env.rng().gen();
    let tag = 100;
    let mut data = PubSeqAppendOnlyData::new(name, tag);

    let owner_0 = ADataOwner {
        public_key,
        entries_index: 0,
        permissions_index: 0,
    };
    unwrap!(data.append_owner(owner_0, 0));

    let perms_0 = ADataPubPermissions {
        permissions: btreemap![ADataUser::Anyone => ADataPubPermissionSet::new(true, true)],
        entries_index: 0,
        owners_index: 1,
    };

    unwrap!(data.append_permissions(perms_0.clone(), 0));
    unwrap!(data.append(
        vec![ADataEntry {
            key: b"one".to_vec(),
            value: b"foo".to_vec()
        }],
        0
    ));
    unwrap!(data.append(
        vec![ADataEntry {
            key: b"two".to_vec(),
            value: b"foo".to_vec()
        }],
        1
    ));

    common::perform_mutation(
        &mut env,
        &mut client,
        Request::PutAData(data.clone().into()),
    );

    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetADataLastEntry(*data.address()),
        Response::GetADataLastEntry(Ok(ADataEntry {
            key: b"two".to_vec(),
            value: b"foo".to_vec(),
        })),
    );

    let appended_values = ADataEntry {
        key: b"three".to_vec(),
        value: b"bar".to_vec(),
    };
    let append = ADataAppend {
        address: *data.address(),
        values: vec![appended_values.clone()],
    };
    // First try an invalid append
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::AppendUnseq(append.clone()),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::AppendSeq { append, index: 2 },
    );

    // Check the result
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetADataLastEntry(*data.address()),
        Response::GetADataLastEntry(Ok(appended_values)),
    );
}

#[test]
fn append_only_data_append_unseq() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();
    let public_key = *client.public_id().public_key();

    let start_nano = 1_000_000_000_000;
    common::perform_transaction(
        &mut env,
        &mut client,
        Request::CreateBalance {
            new_balance_owner: public_key,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    let name: XorName = env.rng().gen();
    let tag = 100;
    let mut data = PubUnseqAppendOnlyData::new(name, tag);

    let owner_0 = ADataOwner {
        public_key,
        entries_index: 0,
        permissions_index: 0,
    };
    unwrap!(data.append_owner(owner_0, 0));

    let perms_0 = ADataPubPermissions {
        permissions: btreemap![ADataUser::Anyone => ADataPubPermissionSet::new(true, true)],
        entries_index: 0,
        owners_index: 1,
    };

    unwrap!(data.append_permissions(perms_0.clone(), 0));
    unwrap!(data.append(vec![ADataEntry {
        key: b"one".to_vec(),
        value: b"foo".to_vec()
    }]));
    unwrap!(data.append(vec![ADataEntry {
        key: b"two".to_vec(),
        value: b"foo".to_vec()
    }]));

    common::perform_mutation(
        &mut env,
        &mut client,
        Request::PutAData(data.clone().into()),
    );

    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetADataLastEntry(*data.address()),
        Response::GetADataLastEntry(Ok(ADataEntry {
            key: b"two".to_vec(),
            value: b"foo".to_vec(),
        })),
    );

    let appended_values = ADataEntry {
        key: b"three".to_vec(),
        value: b"bar".to_vec(),
    };
    let append = ADataAppend {
        address: *data.address(),
        values: vec![appended_values.clone()],
    };

    // First try an invalid append
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::AppendSeq {
            append: append.clone(),
            index: 2,
        },
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::perform_mutation(&mut env, &mut client, Request::AppendUnseq(append));

    // Check the result
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetADataLastEntry(*data.address()),
        Response::GetADataLastEntry(Ok(appended_values)),
    );
}

////////////////////////////////////////////////////////////////////////////////
//
// Immutable data
//
////////////////////////////////////////////////////////////////////////////////

#[test]
fn put_immutable_data() {
    let mut env = Environment::new();

    let mut client_a = env.new_connected_client();
    let mut client_b = env.new_connected_client();

    let mut raw_data = vec![0u8; 1024];
    env.rng().fill(raw_data.as_mut_slice());
    let pub_idata = IData::Pub(PubImmutableData::new(raw_data.clone()));
    let unpub_idata = IData::Unpub(UnpubImmutableData::new(
        raw_data,
        *client_b.public_id().public_key(),
    ));

    // TODO - enable this once we're passed phase 1.
    if false {
        // Put should fail when the client has no associated balance.
        let message_id_1 = client_a.send_request(Request::PutIData(pub_idata.clone()));
        let message_id_2 = client_b.send_request(Request::PutIData(unpub_idata.clone()));
        env.poll();

        match client_a.expect_response(message_id_1) {
            Response::Mutation(Err(NdError::InsufficientBalance)) => (),
            x => unexpected!(x),
        }
        match client_b.expect_response(message_id_2) {
            Response::Mutation(Err(NdError::InsufficientBalance)) => (),
            x => unexpected!(x),
        }
    }

    // Create balances.  Client A starts with 2000 safecoins and spends 1000 to initialise
    // Client B's balance.
    let start_nano = 1_000_000_000_000;
    let message_id_1 = client_a.send_request(Request::CreateBalance {
        new_balance_owner: *client_a.public_id().public_key(),
        amount: unwrap!(Coins::from_nano(2 * start_nano)),
        transaction_id: 0,
    });
    let message_id_2 = client_a.send_request(Request::CreateBalance {
        new_balance_owner: *client_b.public_id().public_key(),
        amount: unwrap!(Coins::from_nano(start_nano)),
        transaction_id: 0,
    });
    env.poll();

    for message_id in &[message_id_1, message_id_2] {
        match client_a.expect_response(*message_id) {
            Response::Transaction(Ok(_)) => (),
            x => unexpected!(x),
        }
    }

    // Check client A can't Put an UnpubIData where B is the owner.
    let unpub_req = Request::PutIData(unpub_idata.clone());
    let mut message_id_1 = client_a.send_request(unpub_req.clone());
    env.poll();
    match client_a.expect_response(message_id_1) {
        Response::Mutation(Err(NdError::InvalidOwners)) => (),
        x => unexpected!(x),
    }
    let mut balance_a = common::get_balance(&mut env, &mut client_a);
    let mut expected_balance = unwrap!(Coins::from_nano(start_nano));
    assert_eq!(expected_balance, balance_a);

    for _ in &[0, 1] {
        // Check they can both Put valid data.
        let pub_req = Request::PutIData(pub_idata.clone());
        message_id_1 = client_a.send_request(pub_req);
        let mut message_id_2 = client_b.send_request(unpub_req.clone());
        env.poll();

        match client_a.expect_response(message_id_1) {
            Response::Mutation(Ok(())) => (),
            x => unexpected!(x),
        }
        match client_b.expect_response(message_id_2) {
            Response::Mutation(Ok(())) => (),
            x => unexpected!(x),
        }
        balance_a = common::get_balance(&mut env, &mut client_a);
        let balance_b = common::get_balance(&mut env, &mut client_b);
        expected_balance = unwrap!(expected_balance.checked_sub(*COST_OF_PUT));
        assert_eq!(expected_balance, balance_a);
        assert_eq!(expected_balance, balance_b);

        // Check the data is retrievable.
        message_id_1 = client_a.send_request(Request::GetIData(*pub_idata.address()));
        message_id_2 = client_b.send_request(Request::GetIData(*unpub_idata.address()));
        env.poll();

        match client_a.expect_response(message_id_1) {
            Response::GetIData(Ok(received)) => assert_eq!(pub_idata, received),
            x => unexpected!(x),
        }
        match client_b.expect_response(message_id_2) {
            Response::GetIData(Ok(received)) => assert_eq!(unpub_idata, received),
            x => unexpected!(x),
        }
    }
}

#[test]
fn get_immutable_data_that_doesnt_exist() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    // Try to get non-existing published immutable data
    let address: XorName = env.rng().gen();
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetIData(IDataAddress::Pub(address)),
        Response::GetIData(Err(NdError::NoSuchData)),
    );

    // Try to get non-existing unpublished immutable data while having no balance
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetIData(IDataAddress::Unpub(address)),
        Response::GetIData(Err(NdError::AccessDenied)),
    );

    // Try to get non-existing unpublished immutable data while having balance
    let start_nano = 1_000_000_000_000;
    let new_balance_owner = *client.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetIData(IDataAddress::Unpub(address)),
        Response::GetIData(Err(NdError::NoSuchData)),
    );
}

#[test]
fn get_immutable_data_from_other_owner() {
    let mut env = Environment::new();

    let mut client_a = env.new_connected_client();
    let mut client_b = env.new_connected_client();

    let start_nano = 1_000_000_000_000;
    let new_balance_owner = *client_a.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client_a,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    let start_nano = 1_000_000_000_000;
    let new_balance_owner = *client_b.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client_b,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    // Client A uploads published data that Client B can fetch
    let raw_data = vec![1, 2, 3];
    let pub_idata = IData::Pub(PubImmutableData::new(raw_data.clone()));
    let pub_idata_address = *pub_idata.address();
    common::perform_mutation(&mut env, &mut client_a, Request::PutIData(pub_idata));
    assert_eq!(
        common::get_idata(&mut env, &mut client_a, pub_idata_address,),
        raw_data
    );
    assert_eq!(
        common::get_idata(&mut env, &mut client_b, pub_idata_address,),
        raw_data
    );

    // Client A uploads unpublished data that Client B can't fetch
    let raw_data = vec![42];
    let owner = client_a.public_id().public_key();
    let unpub_idata = IData::Unpub(UnpubImmutableData::new(raw_data.clone(), *owner));
    let unpub_idata_address = *unpub_idata.address();
    common::perform_mutation(&mut env, &mut client_a, Request::PutIData(unpub_idata));
    assert_eq!(
        common::get_idata(&mut env, &mut client_a, unpub_idata_address,),
        raw_data
    );
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::GetIData(unpub_idata_address),
        Response::GetIData(Err(NdError::AccessDenied)),
    );
}

#[test]
fn put_pub_and_get_unpub_immutable_data_at_same_xor_name() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    // Create balance.
    let start_nano = 1_000_000_000_000;
    let new_balance_owner = *client.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    // Put and verify some published immutable data
    let raw_data = vec![1, 2, 3];
    let pub_idata = IData::Pub(PubImmutableData::new(raw_data.clone()));
    let pub_idata_address: XorName = *pub_idata.address().name();
    common::perform_mutation(&mut env, &mut client, Request::PutIData(pub_idata));
    assert_eq!(
        common::get_idata(&mut env, &mut client, IDataAddress::Pub(pub_idata_address)),
        raw_data
    );

    // Get some unpublished immutable data from the same address
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetIData(IDataAddress::Unpub(pub_idata_address)),
        Response::GetIData(Err(NdError::NoSuchData)),
    );
}

#[test]
fn put_unpub_and_get_pub_immutable_data_at_same_xor_name() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    // Create balances.
    let start_nano = 1_000_000_000_000;
    let new_balance_owner = *client.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    // Put and verify some unpub immutable data
    let raw_data = vec![1, 2, 3];
    let owner = client.public_id().public_key();
    let unpub_idata = IData::Unpub(UnpubImmutableData::new(raw_data.clone(), *owner));
    let unpub_idata_address: XorName = *unpub_idata.address().name();
    common::perform_mutation(&mut env, &mut client, Request::PutIData(unpub_idata));
    assert_eq!(
        common::get_idata(
            &mut env,
            &mut client,
            IDataAddress::Unpub(unpub_idata_address)
        ),
        raw_data
    );

    // Get some published immutable data from the same address
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetIData(IDataAddress::Pub(unpub_idata_address)),
        Response::GetIData(Err(NdError::NoSuchData)),
    );
}

#[test]
fn delete_immutable_data_that_doesnt_exist() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    // Try to delete non-existing published idata while not having a balance
    let address: XorName = env.rng().gen();
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::DeleteUnpubIData(IDataAddress::Pub(address)),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );

    // Try to delete non-existing unpublished data while not having a balance
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetIData(IDataAddress::Unpub(address)),
        Response::GetIData(Err(NdError::AccessDenied)),
    );

    // Try to delete non-existing unpublished data
    let start_nano = 1_000_000_000_000;
    let new_balance_owner = *client.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );
    common::send_request_expect_response(
        &mut env,
        &mut client,
        Request::GetIData(IDataAddress::Unpub(address)),
        Response::GetIData(Err(NdError::NoSuchData)),
    );
}

#[test]
fn delete_immutable_data() {
    let mut env = Environment::new();
    let mut client_a = env.new_connected_client();
    let mut client_b = env.new_connected_client();

    let start_nano = 1_000_000_000_000;
    let new_balance_owner = *client_a.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut client_a,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(start_nano)),
            transaction_id: 0,
        },
    );

    let raw_data = vec![1, 2, 3];
    let pub_idata = IData::Pub(PubImmutableData::new(raw_data.clone()));
    let pub_idata_address: XorName = *pub_idata.address().name();
    common::perform_mutation(&mut env, &mut client_a, Request::PutIData(pub_idata));

    // Try to delete published data by constructing inconsistent Request
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::DeleteUnpubIData(IDataAddress::Pub(pub_idata_address)),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );

    // Try to delete published data by raw XorName
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::DeleteUnpubIData(IDataAddress::Unpub(pub_idata_address)),
        Response::Mutation(Err(NdError::NoSuchData)),
    );

    let raw_data = vec![42];
    let owner = client_a.public_id().public_key();
    let unpub_idata = IData::Unpub(UnpubImmutableData::new(raw_data.clone(), *owner));
    let unpub_idata_address: XorName = *unpub_idata.address().name();
    common::perform_mutation(&mut env, &mut client_a, Request::PutIData(unpub_idata));

    // Delete unpublished data without being the owner
    common::send_request_expect_response(
        &mut env,
        &mut client_b,
        Request::DeleteUnpubIData(IDataAddress::Unpub(unpub_idata_address)),
        Response::Mutation(Err(NdError::AccessDenied)),
    );

    // Delete unpublished data without having the balance
    common::perform_mutation(
        &mut env,
        &mut client_a,
        Request::DeleteUnpubIData(IDataAddress::Unpub(unpub_idata_address)),
    );

    // Delete unpublished data again
    common::send_request_expect_response(
        &mut env,
        &mut client_a,
        Request::DeleteUnpubIData(IDataAddress::Unpub(unpub_idata_address)),
        Response::Mutation(Err(NdError::NoSuchData)),
    )
}

////////////////////////////////////////////////////////////////////////////////
//
// Auth keys
//
////////////////////////////////////////////////////////////////////////////////

#[test]
fn auth_keys() {
    type KeysResult = NdResult<(BTreeMap<PublicKey, AppPermissions>, u64)>;
    fn list_keys<T: TestClientTrait>(env: &mut Environment, client: &mut T, expected: KeysResult) {
        let message_id = client.send_request(Request::ListAuthKeysAndVersion);
        env.poll();
        match client.expect_response(message_id) {
            Response::ListAuthKeysAndVersion(result) => assert_eq!(expected, result),
            x => unexpected!(x),
        }
    }

    let mut env = Environment::new();
    let mut owner = env.new_connected_client();
    let mut app = env.new_connected_app(owner.public_id().clone());

    // Try to insert and then list authorised keys using a client with no balance.  Each should
    // return `NoSuchBalance`.
    let permissions = AppPermissions {
        transfer_coins: true,
    };
    let app_public_key = *app.public_id().public_key();
    let make_ins_request = |version| Request::InsAuthKey {
        key: app_public_key,
        version,
        permissions,
    };

    let no_such_balance = Response::Mutation(Err(NdError::NoSuchBalance));
    common::send_request_expect_response(
        &mut env,
        &mut owner,
        make_ins_request(1),
        no_such_balance,
    );
    list_keys(&mut env, &mut owner, Err(NdError::NoSuchBalance));

    // Create a balance for the owner and check that listing authorised keys returns an empty
    // collection.
    let new_balance_owner = *owner.public_id().public_key();
    common::perform_transaction(
        &mut env,
        &mut owner,
        Request::CreateBalance {
            new_balance_owner,
            amount: unwrap!(Coins::from_nano(1_000_000_000_000)),
            transaction_id: 0,
        },
    );
    let mut expected_map = BTreeMap::new();
    list_keys(&mut env, &mut owner, Ok((expected_map.clone(), 0)));

    // Insert then list the app.
    let _ = expected_map.insert(*app.public_id().public_key(), permissions);
    common::perform_mutation(&mut env, &mut owner, make_ins_request(1));
    list_keys(&mut env, &mut owner, Ok((expected_map.clone(), 1)));

    // Check the app isn't allowed to get a listing of authorised keys, nor insert, nor delete any.
    // No response should be returned to any of these requests.
    let _ = app.send_request(Request::ListAuthKeysAndVersion);
    let _ = app.send_request(make_ins_request(2));
    let del_auth_key_request = Request::DelAuthKey {
        key: *app.public_id().public_key(),
        version: 2,
    };
    let _ = app.send_request(del_auth_key_request.clone());
    env.poll();
    app.expect_no_new_message();

    // Remove the app, then list the keys.
    common::perform_mutation(&mut env, &mut owner, del_auth_key_request);
    list_keys(&mut env, &mut owner, Ok((BTreeMap::new(), 2)));

    // Try to insert using an invalid version number.
    let invalid_successor = Response::Mutation(Err(NdError::InvalidSuccessor(2)));
    common::send_request_expect_response(
        &mut env,
        &mut owner,
        make_ins_request(100),
        invalid_successor,
    );
    list_keys(&mut env, &mut owner, Ok((BTreeMap::new(), 2)));

    // Insert again and list again.
    common::perform_mutation(&mut env, &mut owner, make_ins_request(3));
    list_keys(&mut env, &mut owner, Ok((expected_map, 3)));
}

////////////////////////////////////////////////////////////////////////////////
//
// Mutable data
//
////////////////////////////////////////////////////////////////////////////////

#[test]
fn put_seq_mutable_data() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    // Try to put sequenced Mutable Data
    let name: XorName = env.rng().gen();
    let tag = 100;
    let mdata = SeqMutableData::new(name, tag, *client.public_id().public_key());
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::PutMData(MData::Seq(mdata.clone())),
    );

    // Get Mutable Data and verify it's been stored correctly.
    let message_id = client.send_request(Request::GetMData(MDataAddress::Seq { name, tag }));
    env.poll();

    match client.expect_response(message_id) {
        Response::GetMData(Ok(MData::Seq(data))) => assert_eq!(data, mdata),
        x => unexpected!(x),
    }
}

#[test]
fn put_unseq_mutable_data() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    // Try to put unsequenced Mutable Data
    let name: XorName = env.rng().gen();
    let tag = 100;
    let mdata = UnseqMutableData::new(name, tag, *client.public_id().public_key());
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::PutMData(MData::Unseq(mdata.clone())),
    );

    // Get Mutable Data and verify it's been stored correctly.
    let message_id = client.send_request(Request::GetMData(MDataAddress::Unseq { name, tag }));
    env.poll();

    match client.expect_response(message_id) {
        Response::GetMData(Ok(MData::Unseq(data))) => assert_eq!(data, mdata),
        x => unexpected!(x),
    }
}

#[test]
fn read_seq_mutable_data() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    // Try to put sequenced Mutable Data with several entries.
    let entries: BTreeMap<_, _> = (1..4)
        .map(|_| {
            let key = env.rng().sample_iter(&Standard).take(8).collect();
            let data = env.rng().sample_iter(&Standard).take(8).collect();
            (key, MDataValue { data, version: 0 })
        })
        .collect();

    let name: XorName = env.rng().gen();
    let tag = 100;
    let mdata = SeqMutableData::new_with_data(
        name,
        tag,
        entries.clone(),
        Default::default(),
        *client.public_id().public_key(),
    );
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::PutMData(MData::Seq(mdata.clone())),
    );

    // Get version.
    let message_id = client.send_request(Request::GetMDataVersion(MDataAddress::Seq { name, tag }));
    env.poll();

    match client.expect_response(message_id) {
        Response::GetMDataVersion(Ok(version)) => assert_eq!(version, 0),
        x => unexpected!(x),
    }

    // Get keys.
    let message_id = client.send_request(Request::ListMDataKeys(MDataAddress::Seq { name, tag }));
    env.poll();

    match client.expect_response(message_id) {
        Response::ListMDataKeys(Ok(keys)) => assert_eq!(keys, entries.keys().cloned().collect()),
        x => unexpected!(x),
    }

    // Get values.
    let message_id = client.send_request(Request::ListMDataValues(MDataAddress::Seq { name, tag }));
    env.poll();

    match client.expect_response(message_id) {
        Response::ListSeqMDataValues(Ok(values)) => {
            assert_eq!(values, entries.values().cloned().collect::<Vec<_>>())
        }
        x => unexpected!(x),
    }

    // Get entries.
    let message_id =
        client.send_request(Request::ListMDataEntries(MDataAddress::Seq { name, tag }));
    env.poll();

    match client.expect_response(message_id) {
        Response::ListSeqMDataEntries(Ok(fetched_entries)) => assert_eq!(fetched_entries, entries),
        x => unexpected!(x),
    }

    // Get a value by key.
    let key = unwrap!(entries.keys().cloned().nth(0));
    let message_id = client.send_request(Request::GetMDataValue {
        address: MDataAddress::Seq { name, tag },
        key: key.clone(),
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetSeqMDataValue(Ok(val)) => assert_eq!(val, entries[&key]),
        x => unexpected!(x),
    }
}

#[test]
fn mutate_seq_mutable_data() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    // Try to put sequenced Mutable Data.
    let name: XorName = env.rng().gen();
    let tag = 100;
    let mdata = SeqMutableData::new(name, tag, *client.public_id().public_key());
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::PutMData(MData::Seq(mdata.clone())),
    );

    // Get a non-existant value by key.
    let message_id = client.send_request(Request::GetMDataValue {
        address: MDataAddress::Seq { name, tag },
        key: vec![0],
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetSeqMDataValue(Err(NdError::NoSuchEntry)) => (),
        x => unexpected!(x),
    }

    // Insert new values.
    let actions = MDataSeqEntryActions::new()
        .ins(vec![0], vec![1], 0)
        .ins(vec![1], vec![1], 0);
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::MutateSeqMDataEntries {
            address: MDataAddress::Seq { name, tag },
            actions,
        },
    );

    // Get an existing value by key.
    let message_id = client.send_request(Request::GetMDataValue {
        address: MDataAddress::Seq { name, tag },
        key: vec![0],
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetSeqMDataValue(Ok(val)) => assert_eq!(
            val,
            MDataValue {
                data: vec![1],
                version: 0
            }
        ),
        x => unexpected!(x),
    }

    // Update and delete entries.
    let actions = MDataSeqEntryActions::new()
        .update(vec![0], vec![2], 1)
        .del(vec![1], 1);
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::MutateSeqMDataEntries {
            address: MDataAddress::Seq { name, tag },
            actions,
        },
    );

    // Get an existing value by key.
    let message_id = client.send_request(Request::GetMDataValue {
        address: MDataAddress::Seq { name, tag },
        key: vec![0],
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetSeqMDataValue(Ok(val)) => assert_eq!(
            val,
            MDataValue {
                data: vec![2],
                version: 1
            }
        ),
        x => unexpected!(x),
    }

    // Deleted key should not exist now.
    let message_id = client.send_request(Request::GetMDataValue {
        address: MDataAddress::Seq { name, tag },
        key: vec![1],
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetSeqMDataValue(Err(NdError::NoSuchEntry)) => (),
        x => unexpected!(x),
    }

    // Try an invalid update request.
    let actions = MDataSeqEntryActions::new().update(vec![0], vec![3], 0);
    let message_id = client.send_request(Request::MutateSeqMDataEntries {
        address: MDataAddress::Seq { name, tag },
        actions,
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::Mutation(Err(NdError::InvalidEntryActions(invalid_actions))) => {
            let mut expected_invalid_actions = BTreeMap::new();
            let _ = expected_invalid_actions.insert(vec![0], EntryError::InvalidSuccessor(1));

            assert_eq!(invalid_actions, expected_invalid_actions);
        }
        x => unexpected!(x),
    }
}

#[test]
fn mutate_unseq_mutable_data() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    // Try to put unsequenced Mutable Data.
    let name: XorName = env.rng().gen();
    let tag = 100;
    let mdata = UnseqMutableData::new(name, tag, *client.public_id().public_key());
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::PutMData(MData::Unseq(mdata.clone())),
    );

    // Get a non-existant value by key.
    let message_id = client.send_request(Request::GetMDataValue {
        address: MDataAddress::Unseq { name, tag },
        key: vec![0],
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetUnseqMDataValue(Err(NdError::NoSuchEntry)) => (),
        x => unexpected!(x),
    }

    // Insert new values.
    let actions = MDataUnseqEntryActions::new()
        .ins(vec![0], vec![1])
        .ins(vec![1], vec![1]);
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::MutateUnseqMDataEntries {
            address: MDataAddress::Unseq { name, tag },
            actions,
        },
    );

    // Get an existing value by key.
    let message_id = client.send_request(Request::GetMDataValue {
        address: MDataAddress::Unseq { name, tag },
        key: vec![0],
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetUnseqMDataValue(Ok(val)) => assert_eq!(val, vec![1],),
        x => unexpected!(x),
    }

    // Update and delete entries.
    let actions = MDataUnseqEntryActions::new()
        .update(vec![0], vec![2])
        .del(vec![1]);
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::MutateUnseqMDataEntries {
            address: MDataAddress::Unseq { name, tag },
            actions,
        },
    );

    // Get an existing value by key.
    let message_id = client.send_request(Request::GetMDataValue {
        address: MDataAddress::Unseq { name, tag },
        key: vec![0],
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetUnseqMDataValue(Ok(val)) => assert_eq!(val, vec![2]),
        x => unexpected!(x),
    }

    // Deleted key should not exist now.
    let message_id = client.send_request(Request::GetMDataValue {
        address: MDataAddress::Unseq { name, tag },
        key: vec![1],
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetUnseqMDataValue(Err(NdError::NoSuchEntry)) => (),
        x => unexpected!(x),
    }
}
