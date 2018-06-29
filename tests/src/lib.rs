// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Integration tests for Safe Client Libs.

#![cfg(test)]
// For explanation of lint checks, run `rustc -W help` or see
// https://github.
// com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(
    exceeding_bitshifts, mutable_transmutes, no_mangle_const_items, unknown_crate_types, warnings
)]
#![deny(
    bad_style, deprecated, improper_ctypes, missing_docs, non_shorthand_field_patterns,
    overflowing_literals, plugin_as_library, private_no_mangle_fns, private_no_mangle_statics,
    stable_features, unconditional_recursion, unknown_lints, unused, unused_allocation,
    unused_attributes, unused_comparisons, unused_features, unused_parens, while_true
)]
#![warn(
    trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
    unused_qualifications, unused_results
)]
#![allow(
    box_pointers, missing_copy_implementations, missing_debug_implementations,
    variant_size_differences
)]
#![cfg_attr(
    feature = "cargo-clippy",
    deny(clippy, unicode_not_nfc, wrong_pub_self_convention, option_unwrap_used)
)]
#![cfg_attr(feature = "cargo-clippy", allow(implicit_hasher, too_many_arguments, use_debug))]

extern crate ffi_utils;
extern crate safe_app;
extern crate safe_authenticator;
#[macro_use]
extern crate safe_core;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate unwrap;

mod real_network;

// Tests for unregistered clients.
// 1. Have a registered client PUT something on the network.
// 2. Try to set the access container as unregistered - this should fail.
// 3. Try to set the config root directory as unregistered - this should fail.
#[test]
fn unregistered_client() {
    let orig_data = ImmutableData::new(unwrap!(utils::generate_random_vector(30)));

    // Registered Client PUTs something onto the network
    {
        let orig_data = orig_data.clone();
        random_client(|client| client.put_idata(orig_data));
    }

    // Unregistered Client should be able to retrieve the data
    setup_client(
        |el_h, core_tx, net_tx| Client::unregistered(el_h, core_tx, net_tx, None),
        move |client| {
            let client2 = client.clone();
            let client3 = client.clone();

            client
                .get_idata(*orig_data.name())
                .then(move |res| {
                    let data = unwrap!(res);
                    assert_eq!(data, orig_data);
                    let dir = unwrap!(MDataInfo::random_private(DIR_TAG));
                    client2.set_access_container(dir)
                })
                .then(move |res| {
                    let e = match res {
                        Ok(_) => {
                            panic!("Unregistered client should not be allowed to set user root dir")
                        }
                        Err(e) => e,
                    };
                    match e {
                        CoreError::OperationForbidden => (),
                        _ => panic!("Unexpected {:?}", e),
                    }

                    let dir = unwrap!(MDataInfo::random_private(DIR_TAG));
                    client3.set_config_root_dir(dir)
                })
                .then(|res| {
                    let e = match res {
                        Ok(_) => panic!(
                            "Unregistered client should not be allowed to set config root \
                             dir"
                        ),
                        Err(e) => e,
                    };
                    match e {
                        CoreError::OperationForbidden => (),
                        _ => panic!("Unexpected {:?}", e),
                    }
                    finish()
                })
        },
    );
}
