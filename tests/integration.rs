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
use rand::Rng;
use safe_nd::{
    AData, ADataAddress, ADataIndex, ADataIndices, ADataOwner, AppPermissions, AppendOnlyData,
    Coins, Error as NdError, IData, IDataAddress, LoginPacket, PubImmutableData,
    PubSeqAppendOnlyData, PubUnseqAppendOnlyData, PublicKey, Request, Response, Result as NdResult,
    SeqAppendOnly, UnpubImmutableData, UnpubSeqAppendOnlyData, UnpubUnseqAppendOnlyData,
    UnseqAppendOnly, XorName,
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

#[test]
fn put_append_only_data() {
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

    let owner = ADataOwner {
        public_key: *client.public_id().public_key(),
        data_index: 0,
        permissions_index: 0,
    };

    // Seq
    let adata_name: XorName = env.rng().gen();
    let tag = 100;
    let mut adata = PubSeqAppendOnlyData::new(adata_name, tag);
    unwrap!(adata.append_owner(owner.clone(), 0));
    unwrap!(adata.append(vec![(b"more".to_vec(), b"data".to_vec())], 0));
    let adata = AData::PubSeq(adata);
    let pub_seq_adata_address = *adata.address();
    common::perform_mutation(&mut env, &mut client, Request::PutAData(adata));

    // Unseq
    let adata_name: XorName = env.rng().gen();
    let tag = 101;
    let mut adata = PubUnseqAppendOnlyData::new(adata_name, tag);
    unwrap!(adata.append_owner(owner.clone(), 0));
    unwrap!(adata.append(vec![(b"more".to_vec(), b"data".to_vec())]));
    let adata = AData::PubUnseq(adata);
    let pub_unseq_adata_address = *adata.address();
    common::perform_mutation(&mut env, &mut client, Request::PutAData(adata));

    // Unpub Seq
    let adata_name: XorName = env.rng().gen();
    let tag = 102;
    let mut adata = UnpubSeqAppendOnlyData::new(adata_name, tag);
    unwrap!(adata.append_owner(owner.clone(), 0));
    unwrap!(adata.append(vec![(b"more".to_vec(), b"data".to_vec())], 0));
    let adata = AData::UnpubSeq(adata);
    let unpub_seq_adata_address = *adata.address();
    common::perform_mutation(&mut env, &mut client, Request::PutAData(adata));

    // Unpub Unseq
    let adata_name: XorName = env.rng().gen();
    let tag = 103;
    let mut adata = UnpubUnseqAppendOnlyData::new(adata_name, tag);
    unwrap!(adata.append_owner(owner.clone(), 0));
    unwrap!(adata.append(vec![(b"more".to_vec(), b"data".to_vec())]));
    let adata = AData::UnpubUnseq(adata);
    let unpub_unseq_adata_address = *adata.address();
    common::perform_mutation(&mut env, &mut client, Request::PutAData(adata));

    // TODO - get the data to verify

    // Delete the data
    common::send_request_expect_err(
        &mut env,
        &mut client,
        Request::DeleteAData(pub_seq_adata_address),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::send_request_expect_err(
        &mut env,
        &mut client,
        Request::DeleteAData(pub_unseq_adata_address),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::DeleteAData(unpub_seq_adata_address),
    );
    common::perform_mutation(
        &mut env,
        &mut client,
        Request::DeleteAData(unpub_unseq_adata_address),
    );

    // Delete again to test if it's gone
    common::send_request_expect_err(
        &mut env,
        &mut client,
        Request::DeleteAData(unpub_seq_adata_address),
        Response::Mutation(Err(NdError::NoSuchData)),
    );
    common::send_request_expect_err(
        &mut env,
        &mut client,
        Request::DeleteAData(unpub_unseq_adata_address),
        Response::Mutation(Err(NdError::NoSuchData)),
    );
}

#[test]
fn append_only_data_get_operations() {
    let mut env = Environment::new();
    let mut client = env.new_connected_client();

    // Create an append-only data
    let name: XorName = env.rng().gen();
    let tag = 100;
    let mut data = PubSeqAppendOnlyData::new(name, tag);

    let owner = ADataOwner {
        public_key: *client.public_id().public_key(),
        data_index: 0,
        permissions_index: 0,
    };

    unwrap!(data.append_owner(owner, 0));
    unwrap!(data.append(
        vec![
            (b"one".to_vec(), b"foo".to_vec()),
            (b"two".to_vec(), b"bar".to_vec()),
        ],
        0,
    ));

    let data = AData::PubSeq(data);

    common::perform_mutation(&mut env, &mut client, Request::PutAData(data.clone()));

    // GetAData (failure - non-existing data)
    let invalid_name: XorName = env.rng().gen();
    let invalid_address = ADataAddress::PubSeq {
        name: invalid_name,
        tag,
    };

    common::send_request_expect_err(
        &mut env,
        &mut client,
        Request::GetAData(invalid_address),
        Response::GetAData(Err(NdError::NoSuchData)),
    );

    // GetAData (success)
    let message_id = client.send_request(Request::GetAData(*data.address()));
    env.poll();

    match client.expect_response(message_id) {
        Response::GetAData(Ok(got)) => assert_eq!(got, data),
        x => unexpected!(x),
    }

    // GetADataShell
    let index = 0;
    let message_id = client.send_request(Request::GetADataShell {
        address: *data.address(),
        data_index: ADataIndex::FromStart(index),
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetADataShell(Ok(got)) => assert_eq!(got, unwrap!(data.shell(index))),
        x => unexpected!(x),
    }

    // GetADataRange
    let mut test_range = |start, end, expected_result| {
        let message_id = client.send_request(Request::GetADataRange {
            address: *data.address(),
            range: (start, end),
        });
        env.poll();

        match client.expect_response(message_id) {
            Response::GetADataRange(got) => assert_eq!(got, expected_result),
            x => unexpected!(x),
        }
    };

    test_range(
        ADataIndex::FromStart(0),
        ADataIndex::FromStart(0),
        Ok(vec![]),
    );
    test_range(
        ADataIndex::FromStart(0),
        ADataIndex::FromStart(1),
        Ok(vec![(b"one".to_vec(), b"foo".to_vec())]),
    );
    test_range(
        ADataIndex::FromStart(1),
        ADataIndex::FromStart(2),
        Ok(vec![(b"two".to_vec(), b"bar".to_vec())]),
    );
    test_range(
        ADataIndex::FromEnd(1),
        ADataIndex::FromEnd(0),
        Ok(vec![(b"two".to_vec(), b"bar".to_vec())]),
    );
    test_range(
        ADataIndex::FromStart(0),
        ADataIndex::FromEnd(0),
        Ok(vec![
            (b"one".to_vec(), b"foo".to_vec()),
            (b"two".to_vec(), b"bar".to_vec()),
        ]),
    );
    // FIXME: uncomment once safe-nd stops panicking on this.
    // test_range(
    //     ADataIndex::FromStart(0),
    //     ADataIndex::FromStart(3),
    //     Err(NdError::NoSuchEntry),
    // );

    // GetADataIndices
    let expected = ADataIndices::new(2, 1, 0);
    let message_id = client.send_request(Request::GetADataIndices(*data.address()));
    env.poll();

    match client.expect_response(message_id) {
        Response::GetADataIndices(Ok(got)) => assert_eq!(got, expected),
        x => unexpected!(x),
    }

    // GetADataLastEntry
    let expected = (b"two".to_vec(), b"bar".to_vec());
    let message_id = client.send_request(Request::GetADataLastEntry(*data.address()));
    env.poll();

    match client.expect_response(message_id) {
        Response::GetADataLastEntry(Ok(got)) => assert_eq!(got, expected),
        x => unexpected!(x),
    }

    // GetADataValue
    let message_id = client.send_request(Request::GetADataValue {
        address: *data.address(),
        key: b"one".to_vec(),
    });
    env.poll();

    match client.expect_response(message_id) {
        Response::GetADataValue(Ok(got)) => assert_eq!(got, b"foo"),
        x => unexpected!(x),
    }
}

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
    common::send_request_expect_err(
        &mut env,
        &mut client,
        Request::GetIData(IDataAddress::Pub(address)),
        Response::GetIData(Err(NdError::NoSuchData)),
    );

    // Try to get non-existing unpublished immutable data while having no balance
    common::send_request_expect_err(
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
    common::send_request_expect_err(
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
    common::send_request_expect_err(
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
    common::send_request_expect_err(
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
    common::send_request_expect_err(
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
    common::send_request_expect_err(
        &mut env,
        &mut client,
        Request::DeleteUnpubIData(IDataAddress::Pub(address)),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );

    // Try to delete non-existing unpublished data while not having a balance
    common::send_request_expect_err(
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
    common::send_request_expect_err(
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
    common::send_request_expect_err(
        &mut env,
        &mut client_a,
        Request::DeleteUnpubIData(IDataAddress::Pub(pub_idata_address)),
        Response::Mutation(Err(NdError::InvalidOperation)),
    );

    // Try to delete published data by raw XorName
    common::send_request_expect_err(
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
    common::send_request_expect_err(
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
    common::send_request_expect_err(
        &mut env,
        &mut client_a,
        Request::DeleteUnpubIData(IDataAddress::Unpub(unpub_idata_address)),
        Response::Mutation(Err(NdError::NoSuchData)),
    )
}

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
    common::send_request_expect_err(&mut env, &mut owner, make_ins_request(1), no_such_balance);
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
    common::send_request_expect_err(
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
