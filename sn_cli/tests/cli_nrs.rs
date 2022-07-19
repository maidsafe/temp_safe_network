// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use color_eyre::{eyre::eyre, Result};
use predicates::prelude::*;
use sn_api::SafeUrl;
use sn_cmd_test_utilities::util::{get_random_string, safe_cmd, upload_path};

///
/// `nrs register` subcommand
///
/// Note: these CLI tests will *not* verify that NRS has linked to the correct content. For
/// example, if you create an NRS link to <something>/README.md, we're not going to perform a `cat`
/// to make sure README.md has been linked to, and not, say, test.md. This should be covered by
/// other tests in the system that cover the `nrs_associate` API. It's reasonable to make the
/// assumption in these tests that `nrs_associate` links to the correct content.

#[test]
fn nrs_register_should_register_a_topname() -> Result<()> {
    let topname = get_random_string();
    safe_cmd(["nrs", "register", &topname], Some(0))?
        .assert()
        .stdout(predicate::str::contains(format!(
            "New NRS Map created for \"safe://{}\"",
            topname
        )));
    Ok(())
}

#[test]
fn nrs_register_should_register_a_topname_with_a_versioned_content_link() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let topname = get_random_string();
    safe_cmd(
        ["nrs", "register", &topname, "--link", &url.to_string()],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(format!(
        "New NRS Map created for \"safe://{}\"",
        topname
    )))
    .stdout(predicate::str::contains(format!(
        "The entry points to {}",
        url
    )));
    Ok(())
}

#[test]
fn nrs_register_should_register_a_topname_with_an_immutable_content_link() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (_files_container_xor, processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let test_md_entry = processed_files
        .iter()
        .last()
        .ok_or_else(|| eyre!("list of processed files unexpectedly empty"))?;
    let test_md_file_link = test_md_entry
        .1
        .to_owned()
        .link()
        .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?;
    let url = SafeUrl::from_url(test_md_file_link)?;
    println!("processed_files = {:?}", processed_files);

    let topname = get_random_string();
    safe_cmd(
        ["nrs", "register", &topname, "--link", &url.to_string()],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(format!(
        "New NRS Map created for \"safe://{}\"",
        topname
    )))
    .stdout(predicate::str::contains(format!(
        "The entry points to {}",
        url
    )));
    Ok(())
}

#[test]
fn nrs_register_should_return_an_error_if_a_subname_is_specified() -> Result<()> {
    let name = format!("a.{}", get_random_string());
    safe_cmd(["nrs", "register", &name], Some(1))?
        .assert()
        .stderr(predicate::str::contains(
            "The register command can only register a topname, \
            it cannot add subnames.",
        ))
        .stderr(predicate::str::contains(
            "Please use the nrs add command with the --register-top-name \
            argument to register a topname and add a subname at the same time.",
        ))
        .stderr(predicate::str::contains(
            "Alternatively, register the topname first with the register command, \
            then use the add command to add the subname.",
        ));
    Ok(())
}

#[test]
fn nrs_register_should_return_an_error_if_link_to_versioned_content_has_no_version() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");
    url.set_content_version(None);

    let topname = get_random_string();
    safe_cmd(
        ["nrs", "register", &topname, "--link", &url.to_string()],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The destination you're trying to link to is versionable content. \
            When linking to versionable content, you must supply a version hash on the url. \
            The requested topname was not registered.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the version hash appended to the link. \
            The link should have the form safe://<url>?v=<versionhash>.",
    ));
    Ok(())
}

#[test]
fn nrs_register_should_return_an_error_if_the_topname_already_exists() -> Result<()> {
    let topname = get_random_string();
    safe_cmd(["nrs", "register", &topname], Some(0))?;
    safe_cmd(["nrs", "register", &topname], Some(1))?
        .assert()
        .stderr(predicate::str::contains(format!(
            "Could not register topname {}. That name is already taken.",
            topname
        )))
        .stderr(predicate::str::contains(
            "Try the command again with a different name.",
        ));
    Ok(())
}

///
/// `nrs add` subcommand
///
/// Note: these tests will also not verify that correct content has been linked to, with the
/// exception of one.

#[test]
fn nrs_add_should_add_a_subname_to_versioned_content() -> Result<()> {
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let test_md_file = tmp_data_path.child("test.md");
    let (files_container_xor, _processed_files, _) = upload_path(&test_md_file, false)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let test_name = get_random_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(["nrs", "register", &test_name], Some(0))?;
    safe_cmd(
        ["nrs", "add", &public_name, "--link", &url.to_string()],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains("Existing NRS Map updated"))
    .stdout(predicate::str::contains("+"))
    .stdout(predicate::str::contains(public_name))
    .stdout(predicate::str::contains(url.to_string()));

    Ok(())
}

#[test]
fn nrs_add_should_add_a_subname_to_immutable_content() -> Result<()> {
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let test_md_file = tmp_data_path.child("test.md");
    let (_, processed_files, _) = upload_path(&test_md_file, false)?;
    let test_md_entry = processed_files
        .iter()
        .last()
        .ok_or_else(|| eyre!("list of processed files unexpectedly empty"))?;
    let test_md_file_link = test_md_entry
        .1
        .to_owned()
        .link()
        .ok_or_else(|| eyre!("Missing xorurl link of uploaded test file"))?;
    let url = SafeUrl::from_url(test_md_file_link)?;

    let test_name = get_random_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(["nrs", "register", &test_name], Some(0))?;
    safe_cmd(
        ["nrs", "add", &public_name, "--link", &url.to_string()],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains("Existing NRS Map updated"))
    .stdout(predicate::str::contains("+"))
    .stdout(predicate::str::contains(public_name))
    .stdout(predicate::str::contains(url.to_string()));

    Ok(())
}

#[test]
fn nrs_add_should_add_a_subname_and_set_it_as_the_default_for_the_topname() -> Result<()> {
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let test_md_file = tmp_data_path.child("test.md");
    let (files_container_xor, _processed_files, _) = upload_path(&test_md_file, false)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let topname = get_random_string();
    let public_name = format!("test.{}", &topname);
    safe_cmd(["nrs", "register", &topname], Some(0))?;
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &url.to_string(),
            "--default",
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains("Existing NRS Map updated"))
    .stdout(predicate::str::contains(format!(
        "This link was also set as the default location for {}",
        topname
    )))
    .stdout(predicate::str::contains("+"))
    .stdout(predicate::str::contains(&public_name))
    .stdout(predicate::str::contains(url.to_string()));

    // In this particular test, we will verify the content linked to, because it's the
    // responsibility of the CLI to make sure the correct link is passed to associate the topname
    // with that link.
    let subname_output = safe_cmd(["cat", &public_name], Some(0))?;
    let topname_output = safe_cmd(["cat", &topname], Some(0))?;
    assert_eq!(subname_output, topname_output);

    Ok(())
}

#[test]
fn nrs_add_should_add_a_subname_and_a_new_topname() -> Result<()> {
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let test_md_file = tmp_data_path.child("test.md");
    let (files_container_xor, _processed_files, _) = upload_path(&test_md_file, false)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let test_name = get_random_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &url.to_string(),
            "--register-top-name",
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains("New NRS Map created"))
    .stdout(predicate::str::contains("+"))
    .stdout(predicate::str::contains(public_name))
    .stdout(predicate::str::contains(url.to_string()));

    Ok(())
}

#[test]
fn nrs_add_should_add_a_subname_and_behave_idempotently_for_existing_topname() -> Result<()> {
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let test_md_file = tmp_data_path.child("test.md");
    let (files_container_xor, _processed_files, _) = upload_path(&test_md_file, false)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let test_name = get_random_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(["nrs", "register", &test_name], Some(0))?;
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &url.to_string(),
            "--register-top-name",
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains("Existing NRS Map updated"))
    .stdout(predicate::str::contains("+"))
    .stdout(predicate::str::contains(public_name))
    .stdout(predicate::str::contains(url.to_string()));

    Ok(())
}

#[test]
fn nrs_add_should_update_an_existing_subname() -> Result<()> {
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let test_md_file = tmp_data_path.child("test.md");
    let (files_container_xor, _processed_files, _) = upload_path(&test_md_file, false)?;
    let mut test_md_url = SafeUrl::from_url(&files_container_xor)?;
    test_md_url.set_path("test.md");
    let mut another_md_url = SafeUrl::from_url(&files_container_xor)?;
    another_md_url.set_path("another.md");

    let test_name = get_random_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &test_md_url.to_string(),
            "--register-top-name",
        ],
        Some(0),
    )?;
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &another_md_url.to_string(),
        ],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains("Existing NRS Map updated"))
    .stdout(predicate::str::contains("+"))
    .stdout(predicate::str::contains(public_name))
    .stdout(predicate::str::contains(another_md_url.to_string()));

    Ok(())
}

#[test]
fn nrs_add_should_return_an_error_if_link_to_versioned_content_has_no_version() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");
    url.set_content_version(None);

    let topname = get_random_string();
    let public_name = format!("test.{}", &topname);
    safe_cmd(["nrs", "register", &topname], Some(0))?;
    safe_cmd(
        ["nrs", "add", &public_name, "--link", &url.to_string()],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The destination you're trying to link to is versionable content. \
            When linking to versionable content, you must supply a version hash on the url. \
            The requested topname was not registered.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the version hash appended to the link. \
            The link should have the form safe://<url>?v=<versionhash>.",
    ));
    Ok(())
}

#[test]
fn nrs_add_with_register_top_name_should_return_an_error_if_link_to_versioned_content_has_no_version(
) -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");
    url.set_content_version(None);

    let topname = get_random_string();
    let public_name = format!("test.{}", &topname);
    safe_cmd(["nrs", "register", &topname], Some(0))?;
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &url.to_string(),
            "--register-top-name",
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The destination you're trying to link to is versionable content. \
            When linking to versionable content, you must supply a version hash on the url. \
            The requested topname was not registered.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the version hash appended to the link. \
            The link should have the form safe://<url>?v=<versionhash>.",
    ));
    Ok(())
}

#[test]
fn nrs_add_with_default_should_return_an_error_if_link_to_versioned_content_has_no_version(
) -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");
    url.set_content_version(None);

    let topname = get_random_string();
    let public_name = format!("test.{}", &topname);
    safe_cmd(["nrs", "register", &topname], Some(0))?;
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &url.to_string(),
            "--default",
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The destination you're trying to link to is versionable content. \
            When linking to versionable content, you must supply a version hash on the url. \
            The requested topname was not registered.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the version hash appended to the link. \
            The link should have the form safe://<url>?v=<versionhash>.",
    ));
    Ok(())
}

///
/// `nrs remove` subcommand
///
#[test]
fn nrs_remove_should_remove_a_subname() -> Result<()> {
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let test_md_file = tmp_data_path.child("test.md");
    let (files_container_xor, _processed_files, _) = upload_path(&test_md_file, false)?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let test_name = get_random_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &url.to_string(),
            "--register-top-name",
        ],
        Some(0),
    )?;
    safe_cmd(["nrs", "remove", &public_name], Some(0))?
        .assert()
        .stdout(predicate::str::contains("NRS Map updated"))
        .stdout(predicate::str::contains("-"))
        .stdout(predicate::str::contains(public_name));

    Ok(())
}

#[test]
fn nrs_remove_should_return_an_error_for_a_non_existent_topname() -> Result<()> {
    let topname = get_random_string();
    let public_name = format!("test.{}", &topname);
    safe_cmd(["nrs", "remove", &public_name], Some(1))?
        .assert()
        .stderr(predicate::str::contains(format!(
            "Failed to remove {}.",
            public_name
        )))
        .stderr(predicate::str::contains(format!(
            "The topname {} is likely not registered in Safe NRS",
            topname
        )))
        .stderr(predicate::str::contains(format!(
            "Try the command again or verify that {} is a registered topname.",
            topname
        )));
    Ok(())
}
