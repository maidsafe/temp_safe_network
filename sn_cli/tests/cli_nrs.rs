// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use color_eyre::{eyre::eyre, Result};
use predicates::prelude::*;
use sn_api::{resolver::ContentType, DataAddress, Scope, Url, XorUrlBase};
use sn_cmd_test_utilities::util::{
    get_random_nrs_string, parse_nrs_create_output, safe_cmd, safe_cmd_stdout, safeurl_from,
    upload_path, upload_test_folder, CLI, SAFE_PROTOCOL,
};
use std::process::Command;
use xor_name::XorName;

fn gen_fake_target() -> Result<String> {
    let xorname = XorName(*b"12345678901234567890123456789012");
    let address = DataAddress::bytes(xorname, Scope::Public);
    Url::encode(
        address,
        None,
        0x00a5_3cde,
        ContentType::Raw,
        None,
        None,
        None,
        None,
        None,
        XorUrlBase::Base32,
    )
    .map_err(|e| eyre!("Failed to encode URL: {}", e))
}

///
/// `nrs create` subcommand
///
/// Note: these CLI tests will *not* verify that NRS has linked to the correct content. For
/// example, if you create an NRS link to <something>/README.md, we're not going to perform a `cat`
/// to make sure README.md has been linked to, and not, say, test.md. This should be covered by
/// other tests in the system that cover the `nrs_associate` API. It's reasonable to make the
/// assumption in these tests that `nrs_associate` links to the correct content.

#[test]
fn nrs_create_should_create_a_topname() -> Result<()> {
    let topname = get_random_nrs_string();
    safe_cmd(["nrs", "create", &topname], Some(0))?
        .assert()
        .stdout(predicate::str::contains(format!(
            "New NRS Map created for \"safe://{}\"",
            topname
        )));
    Ok(())
}

#[test]
fn nrs_create_should_create_a_topname_with_a_versioned_content_link() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let mut url = Url::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let topname = get_random_nrs_string();
    safe_cmd(
        ["nrs", "create", &topname, "--link", &url.to_string()],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(format!(
        "New NRS Map created for \"safe://{}\"",
        topname
    )))
    .stdout(predicate::str::contains(format!(
        "The entry points to {}",
        url.to_string()
    )));
    Ok(())
}

#[test]
fn nrs_create_should_create_a_topname_with_an_immutable_content_link() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (_files_container_xor, processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let test_md_entry = processed_files.iter().last().unwrap();
    let test_md_blob_link = test_md_entry.1.to_owned().1;
    let url = Url::from_url(&test_md_blob_link)?;
    println!("processed_files = {:?}", processed_files);

    let topname = get_random_nrs_string();
    safe_cmd(
        ["nrs", "create", &topname, "--link", &url.to_string()],
        Some(0),
    )?
    .assert()
    .stdout(predicate::str::contains(format!(
        "New NRS Map created for \"safe://{}\"",
        topname
    )))
    .stdout(predicate::str::contains(format!(
        "The entry points to {}",
        url.to_string()
    )));
    Ok(())
}

#[test]
fn nrs_create_should_return_an_error_if_a_subname_is_specified() -> Result<()> {
    let name = format!("a.{}", get_random_nrs_string());
    safe_cmd(["nrs", "create", &name], Some(1))?
        .assert()
        .stderr(predicate::str::contains(
            "The create command can only create a topname, \
            it cannot create subnames.",
        ))
        .stderr(predicate::str::contains(
            "Please use the nrs add command with the --create-top-name \
            argument to create a topname and add a subname at the same time.",
        ))
        .stderr(predicate::str::contains(
            "Alternatively, create the topname first with the create command, \
            then use the add command to create the subname.",
        ));
    Ok(())
}

#[test]
fn nrs_create_should_return_an_error_if_an_invalid_link_is_specified() -> Result<()> {
    let topname = get_random_nrs_string();
    safe_cmd(["nrs", "create", &topname, "--link", "invalid"], Some(1))?
        .assert()
        .stderr(predicate::str::contains(
            "The supplied link was not a valid XorUrl.",
        ))
        .stderr(predicate::str::contains(
            "Run the command again with a valid XorUrl for the --link argument.",
        ));
    Ok(())
}

#[test]
fn nrs_create_should_return_an_error_if_link_to_versioned_content_has_no_version() -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let mut url = Url::from_url(&files_container_xor)?;
    url.set_path("test.md");
    url.set_content_version(None);

    let topname = get_random_nrs_string();
    safe_cmd(
        ["nrs", "create", &topname, "--link", &url.to_string()],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The destination you're trying to link to is versionable content. \
            When linking to versionable content, you must supply a version hash on the XorUrl. \
            The requested topname was not created.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the version hash appended to the link. \
            The link should have the form safe://<xorurl>?v=<versionhash>.",
    ));
    Ok(())
}

#[test]
fn nrs_create_should_return_an_error_if_the_topname_already_exists() -> Result<()> {
    let topname = get_random_nrs_string();
    safe_cmd(["nrs", "create", &topname], Some(0))?;
    safe_cmd(["nrs", "create", &topname], Some(1))?
        .assert()
        .stderr(predicate::str::contains(format!(
            "Could not create topname {}. That name is already taken.",
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
    let mut url = Url::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let test_name = get_random_nrs_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(["nrs", "create", &test_name], Some(0))?;
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
    let test_md_entry = processed_files.iter().last().unwrap();
    let test_md_blob_link = test_md_entry.1.to_owned().1;
    let url = Url::from_url(&test_md_blob_link)?;

    let test_name = get_random_nrs_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(["nrs", "create", &test_name], Some(0))?;
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
    let mut url = Url::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let topname = get_random_nrs_string();
    let public_name = format!("test.{}", &topname);
    safe_cmd(["nrs", "create", &topname], Some(0))?;
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
    let mut url = Url::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let test_name = get_random_nrs_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &url.to_string(),
            "--create-top-name",
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
    let mut url = Url::from_url(&files_container_xor)?;
    url.set_path("test.md");

    let test_name = get_random_nrs_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(["nrs", "create", &test_name], Some(0))?;
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &url.to_string(),
            "--create-top-name",
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
    let mut test_md_url = Url::from_url(&files_container_xor)?;
    test_md_url.set_path("test.md");
    let mut another_md_url = Url::from_url(&files_container_xor)?;
    another_md_url.set_path("another.md");

    let test_name = get_random_nrs_string();
    let public_name = format!("test.{}", &test_name);
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &test_md_url.to_string(),
            "--create-top-name",
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
    let mut url = Url::from_url(&files_container_xor)?;
    url.set_path("test.md");
    url.set_content_version(None);

    let topname = get_random_nrs_string();
    let public_name = format!("test.{}", &topname);
    safe_cmd(["nrs", "create", &topname], Some(0))?;
    safe_cmd(
        ["nrs", "add", &public_name, "--link", &url.to_string()],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The destination you're trying to link to is versionable content. \
            When linking to versionable content, you must supply a version hash on the XorUrl. \
            The requested topname was not created.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the version hash appended to the link. \
            The link should have the form safe://<xorurl>?v=<versionhash>.",
    ));
    Ok(())
}

#[test]
fn nrs_add_with_create_top_name_should_return_an_error_if_link_to_versioned_content_has_no_version(
) -> Result<()> {
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;
    let mut url = Url::from_url(&files_container_xor)?;
    url.set_path("test.md");
    url.set_content_version(None);

    let topname = get_random_nrs_string();
    let public_name = format!("test.{}", &topname);
    safe_cmd(["nrs", "create", &topname], Some(0))?;
    safe_cmd(
        [
            "nrs",
            "add",
            &public_name,
            "--link",
            &url.to_string(),
            "--create-top-name",
        ],
        Some(1),
    )?
    .assert()
    .stderr(predicate::str::contains(
        "The destination you're trying to link to is versionable content. \
            When linking to versionable content, you must supply a version hash on the XorUrl. \
            The requested topname was not created.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the version hash appended to the link. \
            The link should have the form safe://<xorurl>?v=<versionhash>.",
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
    let mut url = Url::from_url(&files_container_xor)?;
    url.set_path("test.md");
    url.set_content_version(None);

    let topname = get_random_nrs_string();
    let public_name = format!("test.{}", &topname);
    safe_cmd(["nrs", "create", &topname], Some(0))?;
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
            When linking to versionable content, you must supply a version hash on the XorUrl. \
            The requested topname was not created.",
    ))
    .stderr(predicate::str::contains(
        "Please run the command again with the version hash appended to the link. \
            The link should have the form safe://<xorurl>?v=<versionhash>.",
    ));
    Ok(())
}

#[test]
fn nrs_add_should_return_an_error_if_an_invalid_link_is_specified() -> Result<()> {
    let topname = get_random_nrs_string();
    let public_name = format!("test.{}", &topname);
    safe_cmd(["nrs", "create", &topname], Some(0))?;
    safe_cmd(["nrs", "add", &public_name, "--link", "invalid"], Some(1))?
        .assert()
        .stderr(predicate::str::contains(
            "The supplied link was not a valid XorUrl.",
        ))
        .stderr(predicate::str::contains(
            "Run the command again with a valid XorUrl for the --link argument.",
        ));
    Ok(())
}

#[test]
fn calling_safe_nrs_put_folder_and_fetch() -> Result<()> {
    let (container_xorurl, _map) = upload_test_folder(true)?;
    let container_url = Url::from_url(&container_xorurl)?;
    let mut nrs_url = Url::from_nrsurl(&format!("safe://{}", &get_random_nrs_string()))?;

    let output = safe_cmd_stdout(
        [
            "nrs",
            "create",
            nrs_url.public_name(),
            "-l",
            &container_url.to_string(),
            "--json",
        ],
        Some(0),
    )?;

    assert!(output.contains("safe://"));
    assert!(output.contains('+'));
    assert!(output.contains(&nrs_url.public_name()));

    let (nrs_map_xorurl, _change_map) = parse_nrs_create_output(&output);
    let version = container_url.content_version().unwrap();

    let output = safe_cmd_stdout(["cat", &nrs_map_xorurl.to_string()], Some(0))?;

    assert!(output.contains("safe://"));
    assert!(output.contains("another.md"));
    assert!(output.contains(&format!(
        "Files of FilesContainer (version {})",
        version.to_string()
    )));

    nrs_url.set_path("another.md");

    let output = safe_cmd_stdout(["cat", &nrs_url.to_string()], Some(0))?;

    assert_eq!(output, "exists");

    Ok(())
}

#[test]
#[ignore = "nrs top name invalid because it contains url parts"]
fn calling_safe_nrs_put_no_top_default_fetch() -> Result<()> {
    let nrs_name = get_random_nrs_string();
    let test_name1 = format!("a.b.c.{}", nrs_name);
    let test_name2 = format!("b.c.{}", nrs_name);

    let (container_xorurl, _map) = upload_test_folder(true)?;
    let mut safeurl = safeurl_from(&container_xorurl)?;
    safeurl.set_path("/test.md");
    let link = safeurl.to_string();
    safe_cmd(
        ["nrs", "create", &test_name1, "-l", &link, "--json"],
        Some(0),
    )?;

    let cat_of_new_url = safe_cmd_stdout(["cat", &test_name1], Some(0))?;
    assert_eq!(cat_of_new_url, "hello tests!");

    safeurl.set_path("/another.md");
    let link2 = safeurl.to_string();
    safe_cmd(["nrs", "add", &test_name2, "-l", &link2, "--json"], Some(0))?;

    let cat_of_new_url = safe_cmd_stdout(["cat", &test_name2], Some(0))?;
    assert_eq!(cat_of_new_url, "exists");
    Ok(())
}

#[test]
#[ignore = "nrs top name invalid because it contains url parts"]
fn calling_safe_nrs_put_folder_and_fetch_from_subname() -> Result<()> {
    let (container_xorurl, _map) = upload_test_folder(true)?;
    let container_xorurl = Url::from_url(&container_xorurl)?;
    let mut nrs_url = Url::from_nrsurl(&format!("safe://subname.{}", &get_random_nrs_string()))?;

    let output = safe_cmd_stdout(
        [
            "nrs",
            "create",
            nrs_url.public_name(),
            "-l",
            &container_xorurl.to_string(),
            "--json",
        ],
        Some(0),
    )?;

    assert!(output.contains("safe://"));
    assert!(output.contains("subname"));
    assert!(output.contains('+'));
    assert!(output.contains(&nrs_url.public_name()));

    let (nrs_map_xorurl, _change_map) = parse_nrs_create_output(&output);
    let version = container_xorurl.content_version().unwrap();

    let output = safe_cmd_stdout(["cat", &nrs_map_xorurl.to_string()], Some(0))?;

    assert!(output.contains("safe://"));
    assert!(output.contains("another.md"));
    assert!(output.contains(&format!(
        "Files of FilesContainer (version {})",
        version.to_string()
    )));

    nrs_url.set_path("another.md");

    let output = safe_cmd_stdout(["cat", &nrs_url.to_string()], Some(0))?;

    assert_eq!(output, "exists");

    Ok(())
}

#[test]
#[ignore = "nrs top name invalid because it contains url parts"]
fn calling_safe_nrs_put_and_retrieve_many_subnames() -> Result<()> {
    let (container_xorurl, _map) = upload_test_folder(true)?;
    let mut nrs_url = Url::from_nrsurl(&format!("safe://a.b.{}", &get_random_nrs_string()))?;

    let output = safe_cmd_stdout(
        [
            "nrs",
            "create",
            nrs_url.public_name(),
            "-l",
            &container_xorurl,
            "--json",
        ],
        Some(0),
    )?;

    assert!(output.contains("safe://"));
    assert!(output.contains("a.b"));
    assert!(output.contains('+'));
    assert!(output.contains(&nrs_url.public_name()));

    let (nrs_map_xorurl, _change_map) = parse_nrs_create_output(&output);
    let url = Url::from_url(&container_xorurl)?;
    let version = url.content_version().unwrap();
    let output = safe_cmd_stdout(["cat", &nrs_map_xorurl.to_string()], Some(0))?;

    assert!(output.contains("safe://"));
    assert!(output.contains("another.md"));
    assert!(output.contains(&format!(
        "Files of FilesContainer (version {})",
        version.to_string()
    )));

    nrs_url.set_path("another.md");
    let output = safe_cmd_stdout(["cat", &nrs_url.to_string()], Some(0))?;
    assert_eq!(output, "exists");
    Ok(())
}

#[test]
#[ignore = "nrs top name invalid because it contains url parts"]
fn calling_safe_nrs_put_and_add_new_subnames_set_default_and_retrieve() -> Result<()> {
    let (_container_xorurl, file_map) = upload_test_folder(true)?;

    let test_name = get_random_nrs_string();
    let test_name_w_sub = format!("a.b.{}", &test_name);
    let test_name_w_new_sub = format!("x.b.{}", &test_name);

    let (_a_sign, another_md_xor) = &file_map["../resources/testdata/another.md"];
    let (_t_sign, test_md_xor) = &file_map["../resources/testdata/test.md"];

    let cat_of_another_raw = safe_cmd_stdout(["cat", another_md_xor], Some(0))?;
    assert_eq!(cat_of_another_raw, "exists");

    safe_cmd(
        [
            "nrs",
            "create",
            &test_name_w_sub,
            "-l",
            another_md_xor,
            "--json",
        ],
        Some(0),
    )?;

    let cat_of_sub_one = safe_cmd_stdout(["cat", &test_name_w_sub], Some(0))?;
    assert_eq!(cat_of_sub_one, "exists");

    let first_default = safe_cmd_stdout(["cat", &test_name], Some(0))?;
    assert_eq!(first_default, "exists");

    safe_cmd(
        [
            "nrs",
            "add",
            &test_name_w_new_sub,
            "-l",
            test_md_xor,
            "--json",
            "--default",
        ],
        Some(0),
    )?;

    let new_nrs_creation_cat = safe_cmd_stdout(["cat", &test_name_w_new_sub], Some(0))?;
    assert_eq!(new_nrs_creation_cat, "hello tests!");

    let new_default = safe_cmd_stdout(["cat", &test_name], Some(0))?;
    assert_eq!(new_default, "hello tests!");
    Ok(())
}

#[test]
#[ignore = "nrs top name invalid because it contains url parts"]
fn calling_safe_nrs_put_and_add_new_subnames_remove_one_and_retrieve() -> Result<()> {
    let (_container_xorurl, file_map) = upload_test_folder(true)?;

    let test_name = get_random_nrs_string();
    let test_name_w_sub = format!("a.b.{}", &test_name);
    let test_name_w_new_sub = format!("x.b.{}", &test_name);

    let (_a_sign, another_md_xor) = &file_map["../resources/testdata/another.md"];
    let (_t_sign, test_md_xor) = &file_map["../resources/testdata/test.md"];

    let cat_of_another_raw = safe_cmd_stdout(["cat", another_md_xor], Some(0))?;
    assert_eq!(cat_of_another_raw, "exists");

    safe_cmd(
        [
            "nrs",
            "create",
            &test_name_w_sub,
            "-l",
            another_md_xor,
            "--json",
        ],
        Some(0),
    )?;
    safe_cmd(
        [
            "nrs",
            "add",
            &test_name_w_new_sub,
            "-l",
            test_md_xor,
            "--json",
            "--default",
        ],
        Some(0),
    )?;
    safe_cmd(["nrs", "remove", &test_name_w_sub, "--json"], Some(0))?;

    let new_nrs_creation_cat = safe_cmd_stdout(["cat", &test_name_w_new_sub], Some(0))?;
    assert_eq!(new_nrs_creation_cat, "hello tests!");

    let new_default = safe_cmd_stdout(["cat", &test_name], Some(0))?;
    assert_eq!(new_default, "hello tests!");
    Ok(())
}

#[test]
#[ignore = "nrs top name invalid because it contains url parts"]
fn calling_safe_nrs_put_and_add_new_subnames_remove_one_and_so_fail_to_retrieve() -> Result<()> {
    let (_container_xorurl, file_map) = upload_test_folder(true)?;

    let test_name = get_random_nrs_string();
    let test_name_w_sub = format!("a.b.{}", &test_name);
    let test_name_w_new_sub = format!("x.b.{}", &test_name);

    let (_a_sign, another_md_xor) = &file_map["../resources/testdata/another.md"];
    let (_t_sign, test_md_xor) = &file_map["../resources/testdata/test.md"];

    let cat_of_another_raw = safe_cmd_stdout(["cat", another_md_xor], Some(0))?;
    assert_eq!(cat_of_another_raw, "exists");

    safe_cmd(
        [
            "nrs",
            "create",
            &test_name_w_sub,
            "-l",
            another_md_xor,
            "--json",
        ],
        Some(0),
    )?;
    safe_cmd(
        [
            "nrs",
            "add",
            &test_name_w_new_sub,
            "-l",
            test_md_xor,
            "--json",
        ],
        Some(0),
    )?;

    let new_nrs_creation_cat = safe_cmd_stdout(["cat", &test_name_w_new_sub], Some(0))?;
    assert_eq!(new_nrs_creation_cat, "hello tests!");

    let safe_default = safe_cmd_stdout(["cat", &test_name], Some(0))?;
    assert_eq!(safe_default, "exists");

    let remove_one_nrs = safe_cmd_stdout(["nrs", "remove", &test_name_w_sub, "--json"], Some(0))?;
    assert!(remove_one_nrs.contains('-'));
    assert!(remove_one_nrs.contains(&test_name_w_sub));

    let mut invalid_cat = Command::cargo_bin(CLI).map_err(|e| eyre!(e.to_string()))?;
    invalid_cat
        .args(&vec!["cat", &test_name_w_sub])
        .assert()
        .stderr(predicate::str::contains(
            "Sub name not found in NRS Map Container",
        ))
        .failure();
    Ok(())
}
