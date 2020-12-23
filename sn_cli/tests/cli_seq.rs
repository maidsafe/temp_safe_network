// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

extern crate sn_cmd_test_utilities;

#[macro_use]
extern crate duct;

use anyhow::{anyhow, Result};
use assert_cmd::prelude::*;
use predicates::prelude::*;
use sn_cmd_test_utilities::{
    get_random_nrs_string, parse_cat_seq_output, parse_seq_store_output, xorurl_encoder_from, CLI,
    SAFE_PROTOCOL,
};
use std::{env, process::Command};

const PRETTY_FILES_CREATION_RESPONSE: &str = "Public Sequence stored at: ";
const PRETTY_FILES_PRIVATE_CREATION_RESPONSE: &str = "Private Sequence stored at: ";

#[test]
fn calling_safe_seq_store_pretty() -> Result<()> {
    let random_content: String = (0..10).map(|_| rand::random::<char>()).collect();
    let mut cmd = Command::cargo_bin(CLI).map_err(|e| anyhow!(e.to_string()))?;
    cmd.args(&vec!["seq", "store", &random_content])
        .assert()
        .stdout(predicate::str::contains(PRETTY_FILES_CREATION_RESPONSE))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(1))
        .success();

    // run same command but now with --private flag
    cmd.args(&vec!["--private"])
        .assert()
        .stdout(predicate::str::contains(
            PRETTY_FILES_PRIVATE_CREATION_RESPONSE,
        ))
        .stdout(predicate::str::contains(SAFE_PROTOCOL).count(1))
        .success();
    Ok(())
}

#[test]
fn calling_safe_seq_store_and_cat() -> Result<()> {
    let content = "first item";
    let seq_store = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "seq",
        "store",
        content,
        "--json"
    )
    .read()
    .map_err(|e| anyhow!(e.to_string()))?;

    let seq_url = parse_seq_store_output(&seq_store);

    let seq_cat = cmd!(env!("CARGO_BIN_EXE_safe"), "cat", seq_url, "--json")
        .read()
        .map_err(|e| anyhow!(e.to_string()))?;

    let (_url, data) = parse_cat_seq_output(&seq_cat);

    assert_eq!(data, content.as_bytes());
    Ok(())
}

#[test]
fn calling_safe_seq_store_priv_and_cat() -> Result<()> {
    let content = "first item";
    let seq_store = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "seq",
        "store",
        content,
        "--private",
        "--json"
    )
    .read()
    .map_err(|e| anyhow!(e.to_string()))?;

    let seq_url = parse_seq_store_output(&seq_store);

    let seq_cat = cmd!(env!("CARGO_BIN_EXE_safe"), "cat", seq_url, "--json")
        .read()
        .map_err(|e| anyhow!(e.to_string()))?;

    let (_url, data) = parse_cat_seq_output(&seq_cat);

    assert_eq!(data, content.as_bytes());
    Ok(())
}

#[test]
fn calling_safe_seq_append() -> Result<()> {
    let content_v0 = "first item";
    let seq_store = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "seq",
        "store",
        content_v0,
        "--json"
    )
    .read()
    .map_err(|e| anyhow!(e.to_string()))?;

    let seq_url = parse_seq_store_output(&seq_store);
    let mut xorurl_encoder = xorurl_encoder_from(&seq_url)?;

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
    .map_err(|e| anyhow!(e.to_string()))?;

    let seq_cat = cmd!(env!("CARGO_BIN_EXE_safe"), "cat", &seq_url, "--json")
        .read()
        .map_err(|e| anyhow!(e.to_string()))?;

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
    .map_err(|e| anyhow!(e.to_string()))?;

    let (_url, data) = parse_cat_seq_output(&seq_cat);
    assert_eq!(data, content_v0.as_bytes());
    Ok(())
}

#[test]
fn calling_safe_seq_priv_append() -> Result<()> {
    let content_v0 = "first item";
    let seq_store = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "seq",
        "store",
        content_v0,
        "--private",
        "--json"
    )
    .read()
    .map_err(|e| anyhow!(e.to_string()))?;

    let seq_url = parse_seq_store_output(&seq_store);
    let mut xorurl_encoder = xorurl_encoder_from(&seq_url)?;

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
    .map_err(|e| anyhow!(e.to_string()))?;

    let seq_cat = cmd!(env!("CARGO_BIN_EXE_safe"), "cat", &seq_url, "--json")
        .read()
        .map_err(|e| anyhow!(e.to_string()))?;

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
    .map_err(|e| anyhow!(e.to_string()))?;

    let (_url, data) = parse_cat_seq_output(&seq_cat);
    assert_eq!(data, content_v0.as_bytes());
    Ok(())
}

#[test]
fn calling_seq_store_and_fetch_with_nrsurl() -> Result<()> {
    let content = "first item";
    let seq_store = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "seq",
        "store",
        content,
        "--json"
    )
    .read()
    .map_err(|e| anyhow!(e.to_string()))?;

    let seq_url = parse_seq_store_output(&seq_store);

    let mut xorurl_encoder = xorurl_encoder_from(&seq_url)?;
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
    .map_err(|e| anyhow!(e.to_string()))?;

    let cat_nrsurl_v1 = cmd!(env!("CARGO_BIN_EXE_safe"), "cat", &nrsurl, "--json")
        .read()
        .map_err(|e| anyhow!(e.to_string()))?;
    let (xorurl, data) = parse_cat_seq_output(&cat_nrsurl_v1);
    assert_eq!(xorurl, nrsurl);
    assert_eq!(data, content.as_bytes());
    Ok(())
}
