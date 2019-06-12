// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! Testing Safecoin operations from the apps point of view.

use crate::test_utils::{create_app, create_app_by_req, create_random_auth_req};
use crate::{run, AppError};
use futures::Future;
use rand::{self, Rand};
use routing::XorName;
use safe_core::{Client, CoreError};
use safe_nd::{AppPermissions, Coins, Error};
use std::str::FromStr;
use tiny_keccak::sha3_256;

// Apps should not be able to request the wallet balance if they don't have
// explicit permissions.
// 1. Create a user account with the default coin balance.
// 2. Create an app and authorise it _without_ giving a permissions to access the wallet.
// 3. Try to get balance of the wallet from the app. This request must fail.
// 4. Try to transfer balance from wallet A to some other random wallet B. This request must fail.
// 5. Try to get an existing transaction from the app. This request must fail.
#[test]
fn coin_app_deny_permissions() {
    let app = create_app();

    unwrap!(run(&app, |client, _app_context| {
        let owner_wallet = XorName(sha3_256(&unwrap!(client.owner_key()).0));
        let c2 = client.clone();
        let c3 = client.clone();

        client
            .get_balance(owner_wallet)
            .then(move |res| {
                match res {
                    Err(CoreError::NewRoutingClientError(Error::AccessDenied)) => (),
                    res => panic!("Unexpected result: {:?}", res),
                }

                c2.transfer_coins(
                    XorName::rand(&mut rand::thread_rng()),
                    unwrap!(Coins::from_str("1.0")),
                    None,
                )
            })
            .then(move |res| {
                match res {
                    Err(CoreError::NewRoutingClientError(Error::AccessDenied)) => (),
                    res => panic!("Unexpected result: {:?}", res),
                }

                c3.get_transaction(owner_wallet, 1)
            })
            .then(move |res| {
                match res {
                    Err(CoreError::NewRoutingClientError(Error::AccessDenied)) => (),
                    res => panic!("Unexpected result: {:?}", res),
                }
                Ok::<_, AppError>(())
            })
    }));
}

// 1. Create a user account with the default coin balance.
// 2. Create an app B and authorise it with a permission to access the wallet.
// 3. GetBalance, TransferBalance, and GetTransaction requests should succeed.
#[test]
fn coin_app_allow_permissions() {
    // Create a recipient app
    let app = create_app();

    let wallet2 = unwrap!(run(&app, |client, _app_context| {
        XorName(sha3_256(&unwrap!(client.owner_key()).0))
    }));

    // Create an app that can access the owner's coin balance.
    let mut app_auth_req = create_random_auth_req();
    app_auth_req.app_permissions = AppPermissions {
        transfer_coins: true,
    };

    let app = unwrap!(create_app_by_req(&app_auth_req));

    // Test the basic coin operations.
    unwrap!(run(&app, |client, _app_context| {
        let owner_wallet = XorName(sha3_256(&unwrap!(client.owner_key()).0));
        let c2 = client.clone();
        let c3 = client.clone();

        client
            .get_balance(owner_wallet)
            .then(move |res| {
                match res {
                    Ok(balance) => dbg!(balance),
                    res => panic!("Unexpected result: {:?}", res),
                }

                c2.transfer_coins(wallet2, unwrap!(Coins::from_str("1.0")), Some(1))
            })
            .then(move |res| {
                match res {
                    Ok(()) => (),
                    res => panic!("Unexpected result: {:?}", res),
                }

                c3.get_transaction(owner_wallet, 1)
            })
            .then(move |res| {
                match res {
                    Ok(transaction) => dbg!(transaction),
                    res => panic!("Unexpected result: {:?}", res),
                }
                Ok::<_, AppError>(())
            })
    }));
}
