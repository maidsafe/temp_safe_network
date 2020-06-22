// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

extern crate safe_cmd_test_utilities;

#[macro_use]
extern crate duct;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use safe_api::xorurl::XorUrlEncoder;
use safe_cmd_test_utilities::{
    get_random_nrs_string, parse_cat_seq_output, parse_seq_store_output, CLI, SAFE_PROTOCOL,
};
use std::{env, process::Command};
use unwrap::unwrap;

const PRETTY_FILES_CREATION_RESPONSE: &str = "Sequence stored at: ";

#[test]
fn calling_safe_seq_store_pretty() {
    let random_content: String = (0..10).map(|_| rand::random::<char>()).collect();
    let mut cmd = Command::cargo_bin(CLI).unwrap();
    cmd.args(&vec!["seq", "store", &random_content])
        .assert()
        .stdout(predicate::str::contains(PRETTY_FILES_CREATION_RESPONSE))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(1))
        .success();
}

#[test]
fn calling_safe_seq_store_and_cat() {
    let content = "first item";
    let seq_store = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "seq",
        "store",
        content,
        "--json"
    )
    .read()
    .unwrap();

    let seq_url = parse_seq_store_output(&seq_store);

    let seq_cat = cmd!(env!("CARGO_BIN_EXE_safe"), "cat", seq_url, "--json")
        .read()
        .unwrap();

    let (_url, data) = parse_cat_seq_output(&seq_cat);

    assert_eq!(data, content.as_bytes());
}

#[test]
fn calling_safe_seq_append() {
    let content_v0 = "first item";
    let seq_store = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "seq",
        "store",
        content_v0,
        "--json"
    )
    .read()
    .unwrap();

    let seq_url = parse_seq_store_output(&seq_store);
    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&seq_url));

    let content_v1 = "second item";
    let _ = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "seq",
        "append",
        content_v1,
        &seq_url,
        "--json"
    )
    .read()
    .unwrap();

    let seq_cat = cmd!(env!("CARGO_BIN_EXE_safe"), "cat", &seq_url, "--json")
        .read()
        .unwrap();

    let (_url, data) = parse_cat_seq_output(&seq_cat);
    assert_eq!(data, content_v1.as_bytes());

    xorurl_encoder.set_content_version(Some(0));
    let seq_cat = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "cat",
        &xorurl_encoder.to_string(),
        "--json"
    )
    .read()
    .unwrap();

    let (_url, data) = parse_cat_seq_output(&seq_cat);
    assert_eq!(data, content_v0.as_bytes());
}

#[test]
fn calling_seq_store_and_fetch_with_nrsurl() {
    let content = "first item";
    let seq_store = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "seq",
        "store",
        content,
        "--json"
    )
    .read()
    .unwrap();

    let seq_url = parse_seq_store_output(&seq_store);

    let mut xorurl_encoder = unwrap!(XorUrlEncoder::from_url(&seq_url));
    xorurl_encoder.set_content_version(Some(0));
    let files_container_v0 = &xorurl_encoder.to_string();
    let nrsurl = format!("safe://{}", get_random_nrs_string());

    let _ = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "nrs",
        "create",
        &nrsurl,
        "-l",
        &files_container_v0,
    )
    .read()
    .unwrap();

    let cat_nrsurl_v1 = cmd!(env!("CARGO_BIN_EXE_safe"), "cat", &nrsurl, "--json")
        .read()
        .unwrap();
    let (xorurl, data) = parse_cat_seq_output(&cat_nrsurl_v1);
    assert_eq!(xorurl, nrsurl);
    assert_eq!(data, content.as_bytes());
}
