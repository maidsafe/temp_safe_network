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

#![cfg(test)]

use core::CoreError;
use core::utility::{self, test_utils};
use futures::Future;
use routing::client_errors::{GetError, MutationError};
use rust_sodium::crypto::box_;
use maidsafe_utilities::log;
use nfs::NfsError;
use super::{DnsError, operations};

#[test]
fn register_and_delete_dns_internal_error_recovery() {
    unwrap!(log::init(true));

    test_utils::register_and_run(|client| {
        let client2 = client.clone();
        let client3 = client.clone();
        let client4 = client.clone();
        let client5 = client.clone();
        let client6 = client.clone();
        let client7 = client.clone();
        let client8 = client.clone();
        let client9 = client.clone();
        let client10 = client.clone();

        let dns_name = unwrap!(utility::generate_random_string(10));
        let dns_name2 = dns_name.clone();
        let dns_name3 = dns_name.clone();
        let dns_name4 = dns_name.clone();
        let dns_name5 = dns_name.clone();
        let dns_name6 = dns_name.clone();
        let dns_name7 = dns_name.clone();

        let messaging_keypair = box_::gen_keypair();
        let messaging_keypair2 = messaging_keypair.clone();
        let messaging_keypair3 = messaging_keypair.clone();
        let messaging_keypair4 = messaging_keypair.clone();

        let owners = vec![unwrap!(client.public_signing_key()).clone()];
        let owners2 = owners.clone();
        let owners3 = owners.clone();
        let owners4 = owners.clone();

        let secret_signing_key = unwrap!(client.secret_signing_key()).clone();
        let secret_signing_key2 = secret_signing_key.clone();
        let secret_signing_key3 = secret_signing_key.clone();
        let secret_signing_key4 = secret_signing_key.clone();
        let secret_signing_key5 = secret_signing_key.clone();
        let secret_signing_key6 = secret_signing_key.clone();
        let secret_signing_key7 = secret_signing_key.clone();

        // Limit of `Some(2)` would prevent the mutation to happen. We want one
        // `Mutation` exactly at this point
        client.set_network_limits(Some(3));

        info!("Fail to register the name");
        operations::register_dns(client,
                                 dns_name,
                                 messaging_keypair.0,
                                 messaging_keypair.1,
                                 &[],
                                 owners,
                                 secret_signing_key,
                                 None)
            .then(move |result| {
                match result {
                    Err(DnsError::NfsError(NfsError::CoreError(CoreError::MutationFailure {
                        reason: MutationError::NetworkOther(ref s), ..
                    }))) if s == "Max operations exhausted" => (),
                    Ok(()) => panic!("Operation unexpectedly succeeded"),
                    Err(err) => panic!("{:?}", err),
                }

                // Remove artificial network failure
                client2.set_network_limits(None);

                info!("Now try and delete. It should fail because the registration failed.");
                operations::delete_dns(&client2,
                                       dns_name2,
                                       secret_signing_key2)
            })
            .then(move |result| {
                match result {
                    Err(DnsError::DnsRecordNotFound) => (),
                    Ok(()) => panic!("Operation unexpectedly succeeded"),
                    Err(err) => panic!("{:?}", err),
                }

                info!("List of registered names should be empty");
                operations::get_all_registered_names(&client3)
            })
            .then(move |result| {
                assert!(unwrap!(result).is_empty());

                info!("Register for real this time.");
                operations::register_dns(&client4,
                                         dns_name3,
                                         messaging_keypair2.0,
                                         messaging_keypair2.1,
                                         &[],
                                         owners2,
                                         secret_signing_key3,
                                         None)
            })
            .then(move |result| {
                unwrap!(result);

                info!("Delete with simulated failure");
                client5.set_network_limits(Some(5));

                operations::delete_dns(&client5,
                                       dns_name4,
                                       secret_signing_key4)
            })
            .then(move |result| {
                match result {
                    Err(DnsError::NfsError(NfsError::CoreError(CoreError::GetFailure {
                        reason: GetError::NetworkOther(ref s), ..
                    }))) if s == "Max operations exhausted" => (),
                    Ok(()) => panic!("Operation unexpectedly succeeded"),
                    Err(err) => panic!("{:?}", err),
                }

                // Remove artificial network failure
                client6.set_network_limits(None);

                info!("Fail to register because it's already registered");
                operations::register_dns(&client6,
                                         dns_name5,
                                         messaging_keypair3.0,
                                         messaging_keypair3.1,
                                         &[],
                                         owners3,
                                         secret_signing_key5,
                                         None)
            })
            .then(move |result| {
                match result {
                    Err(DnsError::DnsNameAlreadyRegistered) => (),
                    Ok(()) => panic!("Operation unexpectedly succeeded"),
                    Err(err) => panic!("{:?}", err),
                }

                info!("List of registered names should contain record");
                operations::get_all_registered_names(&client7)
            })
            .then(move |result| {
                let names = unwrap!(result);
                assert_eq!(&names, &[dns_name6.clone()]);

                info!("Delete should succeed");
                operations::delete_dns(&client8, dns_name6, secret_signing_key6)
            })
            .then(move |result| {
                unwrap!(result);

                info!("List of registered names should be empty");
                operations::get_all_registered_names(&client9)
            })
            .then(move |result| {
                let names = unwrap!(result);
                assert!(names.is_empty());

                info!("Register for real again.");
                operations::register_dns(&client10,
                                         dns_name7,
                                         messaging_keypair4.0,
                                         messaging_keypair4.1,
                                         &[],
                                         owners4,
                                         secret_signing_key7,
                                         None)
            })
            .map_err(|err| panic!("{:?}", err))
    })
}
