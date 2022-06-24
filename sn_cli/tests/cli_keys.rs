// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use color_eyre::{eyre::eyre, Result};
use sn_cmd_test_utilities::util::{parse_keys_create_output, safe_cmd_stdout};

#[test]
fn keys_create_should_output_public_and_secret_key() -> Result<()> {
    let output = safe_cmd_stdout(["keys", "create"], Some(0))?;
    let lines: Vec<&str> = output.split('\n').collect();
    let (_, pk_hex) = lines[0]
        .split_once(':')
        .ok_or_else(|| eyre!("the output should contain a public key in hex"))?;
    let (_, sk_hex) = lines[1]
        .split_once(':')
        .ok_or_else(|| eyre!("the output should contain a secret key in hex"))?;
    let pk_hex = pk_hex.trim();
    let sk_hex = sk_hex.trim();

    // Test the key strings are parsable, then make sure they are an actual pair, since it would be
    // possible for the CLI to print out strings from different pairs.
    let _ = bls::PublicKey::from_hex(pk_hex)?;
    let sk = bls::SecretKey::from_hex(sk_hex)?;
    assert_eq!(pk_hex, sk.public_key().to_hex());

    Ok(())
}

#[test]
fn keys_create_with_json_output_should_output_keys_and_url() -> Result<()> {
    let output = safe_cmd_stdout(["keys", "create", "--json"], Some(0))?;
    let (pk_hex, sk_hex) = parse_keys_create_output(&output)?;
    let _ = bls::PublicKey::from_hex(&pk_hex)?;
    let sk = bls::SecretKey::from_hex(&sk_hex)?;
    assert_eq!(pk_hex, sk.public_key().to_hex());
    Ok(())
}

#[test]
fn keys_show_should_output_public_key() -> Result<()> {
    let output = safe_cmd_stdout(["keys", "show"], Some(0))?;
    let lines: Vec<&str> = output.split('\n').collect();
    let (_, pk_hex) = lines[1]
        .split_once(':')
        .ok_or_else(|| eyre!("the output should contain a public key in hex"))?;
    let pk_hex = pk_hex.trim();

    let _ = bls::PublicKey::from_hex(pk_hex)?;
    Ok(())
}

#[test]
fn keys_show_with_show_sk_should_output_public_and_secret_key() -> Result<()> {
    let output = safe_cmd_stdout(["keys", "show", "--show-sk"], Some(0))?;
    let lines: Vec<&str> = output.split('\n').collect();
    let (_, pk_hex) = lines[1]
        .split_once(':')
        .ok_or_else(|| eyre!("the output should contain a public key in hex"))?;
    let (_, sk_hex) = lines[2]
        .split_once(':')
        .ok_or_else(|| eyre!("the output should contain a secret key in hex"))?;
    let pk_hex = pk_hex.trim();
    let sk_hex = sk_hex.trim();

    let _ = bls::PublicKey::from_hex(pk_hex)?;
    let sk = bls::SecretKey::from_hex(sk_hex)?;
    assert_eq!(pk_hex, sk.public_key().to_hex());
    Ok(())
}

#[test]
fn keys_show_with_json_output_should_output_public_and_secret_key() -> Result<()> {
    let output = safe_cmd_stdout(["keys", "show", "--json"], Some(0))?;
    let (pk_hex, sk_hex) = parse_keys_create_output(&output)?;

    let _ = bls::PublicKey::from_hex(&pk_hex)?;
    let sk = bls::SecretKey::from_hex(&sk_hex)?;
    assert_eq!(pk_hex, sk.public_key().to_hex());
    Ok(())
}
