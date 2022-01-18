// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use assert_fs::prelude::*;
use color_eyre::{eyre::eyre, Result};
use predicates::prelude::*;
use sn_api::SafeUrl;
use sn_cmd_test_utilities::util::{
    can_write_symlinks, create_absolute_symlinks_directory, create_nrs_link, create_symlink,
    digest_file, get_random_nrs_string, safe_cmd, safe_cmd_at, safe_cmd_stdout, safeurl_from,
    str_to_sha3_256, sum_tree, test_symlinks_are_valid, upload_path, TEST_FOLDER,
};
use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// Synopsis
/// --------
/// Test:  safe files get <url> /tmp/testdata
///   src is a container url
///   dst exists, and is a directory
///   expected result: ../resources/testdata matches /tmp/testdata/testdata
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the specified path
/// on the local machine, when the files were uploaded without using a trailing slash.
///
/// Given a unique `tmp_data_path` directory is created
/// And the test data in the repository has been copied to `tmp_data_path`
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path --recursive` command
/// And a unique `dest` directory is created
/// And `src` is set to the files container xor address
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And due to the use of the trailing slash, `dest` should have a directory named `tmp_data_path`
/// And the directory tree of `dest/tmp_data_path` should match the test data directory in
/// the repository
#[test]
fn files_get_src_is_container_and_dst_is_dir() -> Result<()> {
    // Arrange
    let with_trailing_slash = false;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let src = &files_container_xor;
    let dst = assert_fs::TempDir::new()?;
    let dst = dst.path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    let mut dst_pb = PathBuf::from(dst);
    dst_pb.push(tmp_data_path.path().file_name().ok_or_else(|| {
        eyre!(
            "failed to read file name from path: {}",
            tmp_data_path.path().display()
        )
    })?);
    assert_eq!(
        sum_tree(TEST_FOLDER)?,
        sum_tree(&dst_pb.as_path().display().to_string())?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get <url> /tmp/testdata
///    src is a container url, uploaded with trailing slash
///    dst exists, and is an empty directory
///    expected result: ../resources/testdata matches /tmp/testdata
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the specified path
/// on the local machine, when the files were uploaded *with* a trailing slash.
///
/// Given a unique `tmp_data_path` directory is created
/// And the test data in the repository has been copied to `tmp_data_path`
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And a unique `dest` directory is created
/// And `src` is set to the files container xor address
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the directory tree of `dest` should match the test data directory in the repository
#[test]
fn files_get_src_is_container_trailing_and_dst_is_dir() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let src = &files_container_xor;
    let dst = assert_fs::TempDir::new()?;
    let dst = dst.path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(sum_tree(TEST_FOLDER)?, sum_tree(&dst)?);

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get <url> .
///    src is a container url, testdata put without slash.
///    dst is the current working directory.
///    expected result: ../resources/testdata matches ../resources/testdata
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the current
/// directory on the local machine.
///
/// Given a unique `tmp_data_path` directory is created
/// And the test data in the repository has been copied to `tmp_data_path`
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path --recursive` command
/// And `src` is set to the files container xor address
/// And a unique `dest` directory is created
/// And the current working directory is changed to `dest`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the directory tree of `dest` should match the test data directory in the repository
#[test]
fn files_get_src_is_container_and_dst_is_cwd() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let src = &files_container_xor;
    let dst = assert_fs::TempDir::new()?;
    let dst = dst.path().display().to_string();

    // Act
    safe_cmd_at(
        [
            "files",
            "get",
            src,
            ".",
            "--exists=overwrite",
            "--progress=none",
        ],
        &dst,
        Some(0),
    )?;

    // Assert
    assert_eq!(sum_tree(TEST_FOLDER)?, sum_tree(&dst)?);

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get <url>
///    src is a container url, testdata put without slash.
///    dst is unspecified.  (should default to the current working directory)
///    expected result: ../resources/testdata matches ../resources/testdata
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the current
/// directory on the local machine, when no destination is specified.
///
/// Given a unique `tmp_data_path` directory is created
/// And the test data in the repository has been copied to `tmp_data_path`
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path --recursive` command
/// And `src` is set to the files container xor address
/// And a unique `dest` directory is created
/// And the current working directory is changed to `dest`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the directory tree of `dest` should match the test data directory in the repository
#[test]
fn files_get_src_is_container_and_dst_is_unspecified() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let src = &files_container_xor;
    let dst = assert_fs::TempDir::new()?;
    let dst = dst.path().display().to_string();

    // Act
    safe_cmd_at(
        ["files", "get", src, "--exists=overwrite", "--progress=none"],
        &dst,
        Some(0),
    )?;

    // Assert
    assert_eq!(sum_tree(TEST_FOLDER)?, sum_tree(&dst)?);

    Ok(())
}

/// Synopsis
/// --------
/// From files_get.rs
///
/// It is an error/warning if the dst path attempts to use
/// an existing file as a directory. But other files should
/// still be written.  eg:
/// $ mkdir -p /tmp/a/b/c && touch /tmp/a/file.txt
/// $ mkdir /tmp/target && touch /tmp/target/b   (b is a file)
/// $ cp -r /tmp/a/* /tmp/target
///    cp: cannot overwrite non-directory '/tmp/target/b' with directory '/tmp/a/b'
/// $ ls -l /tmp/target/
///      total 0
///      -rw-rw-r-- 1 user user 0 Mar 31 14:38 b         (b still a file)
///      -rw-rw-r-- 1 user user 0 Mar 31 14:38 file.txt  (other file written)
///
/// note: cp returns exit code of 1 when this occurs, but sn_cli
///       presently returns 0.
///
/// ----------------
///
/// Test:  safe files get <url> /tmp/testdata
///    src is a container url, uploaded with trailing slash
///    dst exists, and is a directory
///    src contains 'subfolder', which is a directory
///    dst contains 'subfolder' (a file) and is otherwise empty.
///    expected result:
///        exit code = 0 and
///        /tmp/testdata/subfolder is still a file and
///        /tmp/testdata/test.md exists
///        /tmp/testdata/another.md exists
///        stderr contains: "Warning: cannot overwrite non-directory"
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the specified
/// destination path on the local machine, but the destination has a file with the same name as one
/// of the directories in the container.
///
/// Given a unique `tmp_data_path` directory is created
/// And the test data in the repository has been copied to `tmp_data_path`
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the files container xor address
/// And a unique `dest` directory is created
/// And a `dest/subfolder` file is created
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the process should exit with no error even though there was an existing file named
/// 'subfolder'
/// And the directory tree of `dest` should not match the test data directory in the repository
/// And the `dest/subfolder` file should still exist
/// And the `dest/subfolder` file should have the same content
/// And the `dest/test.md` file should have been downloaded by `files get`
/// And the `dest/another.md` file should have been downloaded by `files get`
#[test]
fn files_get_attempt_overwrite_sub_file_with_dir() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let subfolder_file = assert_fs::NamedTempFile::new("subfolder")?;
    subfolder_file.write_str("existing text file")?;
    let mut pb = PathBuf::from(subfolder_file.path());
    pb.pop();

    let src = &files_container_xor;
    let dst = pb.as_path().display().to_string();

    // Act
    let output = safe_cmd(
        [
            "files",
            "get",
            src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(output.status.code(), Some(0));
    assert_ne!(sum_tree(TEST_FOLDER)?, sum_tree(&dst)?);
    subfolder_file.assert(predicate::path::is_file());
    subfolder_file.assert(predicate::str::contains("existing text file"));

    assert!(Path::new(&dst).join("test.md").is_file());
    assert!(Path::new(&dst).join("another.md").is_file());

    // Disabled for now because the warning is disabled by sn_cli if TTY
    // not detected.  So it doesn't appear in our output.  Perhaps later
    // sn_cli adds a flag to override TTY detection.
    // assert!(String::from_utf8_lossy(&cmd_output.stderr).into_owned().contains("Warning: cannot overwrite non-directory"));

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get <nrs_url>
///    src is an nrs url, linked to a container url, testdata put without slash.
///    dst is unspecified.  (should default to the current working directory)
///    expected result: ../resources/testdata matches ../resources/testdata
///
/// Scenario: the `files get` command downloads a directory from the network to the specified
/// destination path on the local machine, where the source is an NRS name.
///
/// Details
/// -------
/// Given a unique `tmp_data_path` directory is created
/// And the test data in the repository has been copied to `tmp_data_path`
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And an NRS name is created that points to the uploaded files container
/// And `src` is set to the NRS name
/// And a unique `dest` directory is created
/// And the current working directory is changed to `dest`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the directory tree of `dest` should match the test data directory in the repository
#[test]
fn files_get_src_is_nrs_and_dst_is_unspecified() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let mut nrs_name = "NRS_NAME".to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| eyre!(e.to_string()))?
        .as_micros();
    nrs_name.push_str(&str_to_sha3_256(&format!("{}", now)));

    safe_cmd(
        ["nrs", "register", &nrs_name, "-l", &files_container_xor],
        Some(0),
    )?;

    let src = format!("safe://{}", &nrs_name);
    let dst = assert_fs::TempDir::new()?;
    let dst = dst.path().display().to_string();

    // Act
    safe_cmd_at(
        [
            "files",
            "get",
            &src,
            "--exists=overwrite",
            "--progress=none",
        ],
        &dst,
        Some(0),
    )?;

    // Assert
    assert_eq!(sum_tree(TEST_FOLDER)?, sum_tree(&dst)?);

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get <nrs_url>
///    src is an nrs url, linked to a container url, testdata put without slash.
///    dst is unspecified.  (should default to the current working directory)
///    expected result: ../resources/testdata matches ../resources/testdata
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the specified
/// destination path on the local machine, where the source is an NRS name.
///
/// Given a unique `tmp_data_path` directory is created
/// And the test data in the repository has been copied to `tmp_data_path`
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And an xor url is created to point to `files_container_xor/subfolder`
/// And an nrs name is created that points to the above xor url
/// And `src` is set to the nrs name
/// And a unique `dest` directory is created
/// And the current working directory is changed to `dest`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the contents of `dest/sub2.ms` should match the contents of sub2.md in the repository
/// testdata
#[test]
#[ignore = "investigate after sn_cli merge into workspace"]
fn files_get_src_is_nrs_with_path_and_dst_is_unspecified() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    tmp_data_dir.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_dir, with_trailing_slash)?;

    let mut e = SafeUrl::from_url(&files_container_xor)?;
    e.set_path("subfolder");
    let xor_url_with_path = e.to_string();

    let mut nrs_name = "NRS_NAME".to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| eyre!(e.to_string()))?
        .as_micros();
    nrs_name.push_str(&str_to_sha3_256(&format!("{}", now)));

    safe_cmd(
        ["nrs", "create", &nrs_name, "-l", &xor_url_with_path],
        Some(0),
    )?;

    let src = format!("safe://{}/sub2.md", &nrs_name);
    let dst = assert_fs::TempDir::new()?;
    let dst = dst.path().display().to_string();

    // Act
    safe_cmd_at(
        [
            "files",
            "get",
            &src,
            "--exists=overwrite",
            "--progress=none",
        ],
        &dst,
        Some(0),
    )?;

    // Assert
    assert_eq!(
        std::fs::read_to_string(Path::new(&format!("{}/sub2.md", dst)))?,
        std::fs::read_to_string(Path::new("testdata/subfolder/sub2.md"))?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get safe://subfolder /tmp/subfolder
///    src is a recursive nrs url
///       safe://subfolder  --> safe://testdata/subfolder
///       safe://testdata   --> safe://xorurl/testdata
///    dst exists
///    expected result: ../resources/testdata/subfolder matches /tmp/testdata/subfolder
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the specified
/// destination path on the local machine, where the source is an NRS name.
///
/// Given a unique `tmp_data_path` directory is created
/// And the test data in the repository has been copied to `tmp_data_path`
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And an xor url is created to point to `files_container_xor/tmp_data_path`
/// And an nrs link is created that points to the above xor url
/// And `src` is set to `nrs link/subfolder`
/// And a unique `dest` directory is created
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the nrs link pointing to the subfolder should have been downloaded to `dest`
/// And the contents of `dest/subfolder` should be the same as `testdata/subfolder`
#[test]
#[ignore = "investigate after sn_cli merge into workspace"]
fn files_get_src_is_nrs_recursive_and_dst_not_existing() -> Result<()> {
    // Arrange
    let with_trailing_slash = false;
    let tmp_data_dir = assert_fs::TempDir::new()?;
    tmp_data_dir.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_dir, with_trailing_slash)?;

    let container_folder_name = tmp_data_dir
        .path()
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| {
            eyre!(
                "failed to read file name from path: {}",
                tmp_data_dir.path().display()
            )
        })?;
    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_path(container_folder_name);

    let tmp_data_nrs = get_random_nrs_string();
    let tmp_data_nrs_url = create_nrs_link(&tmp_data_nrs, &url.to_string())?;
    let version = tmp_data_nrs_url
        .content_version()
        .ok_or_else(|| eyre!("failed to read content version from xorurl"))?;

    let src = format!("safe://{}/subfolder?v={}", tmp_data_nrs, version);
    let dst = assert_fs::TempDir::new()?;
    let dst = dst.path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(
        sum_tree("testdata/subfolder")?,
        sum_tree(&format!("{}/subfolder", dst))?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get "safe://.../dir with space/file with space" "/tmp/new file"
///    src is a file, directory and file both contain embedded spaces, not url encoded.
///    dst does not exist
///    expected result: /tmp/new file is written without error.
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the specified
/// destination path on the local machine, when the file and directory names have spaces in them.
///
/// Given a unique `tmp_data_path` directory is created
/// And a sub directory and file that contain spaces in their names are created in `tmp_data_path`
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `tmp_data_path` container xor url
/// And a unique `dest` directory is created
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the `tmp_data_path` and `dest` paths should have the same directory trees, with the files
/// and directories with spaces in the names.
#[test]
fn files_get_src_has_embedded_spaces_and_dst_also() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    let child = tmp_data_path.child("dir with space/file with space");
    child.write_str("some file content")?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let src = &files_container_xor;
    let dst = assert_fs::TempDir::new()?;
    let dst = dst.path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(
        sum_tree(&tmp_data_path.path().display().to_string())?,
        sum_tree(&dst)?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get "safe://.../dir%20with%20space/file%20with%20space" "/tmp/new file"
///    src is a file, directory and file both contain embedded spaces, url-encoded.
///    dst does not exist
///
///    expected result: /tmp/new file is written without error.
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the specified
/// destination path on the local machine, when the file and directory names have spaces in them.
/// The file will be retrieved using url encoded, embedded spaces.
///
/// Given a unique `tmp_data_path` directory is created
/// And a sub directory and file that contain spaces in their names are created in `tmp_data_path`
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `tmp_data_path/dir%20with%20space/file%20with%20space` container xor url
/// And a unique `dest` file path is specified
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the `tmp_data_path` and `dest` paths should have the same directory trees, with the files
/// and directories with spaces in the names.
#[test]
fn files_get_src_has_encoded_spaces_and_dst_also() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    let child = tmp_data_path.child("dir with space/file with space");
    child.write_str("some file content")?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let url = SafeUrl::from_url(&files_container_xor)?;
    let src = format!(
        "{}://{}/{}",
        url.scheme(),
        url.public_name(),
        "dir%20with%20space/file%20with%20space",
    );
    let dst = assert_fs::NamedTempFile::new("new file")?;

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst.path().display().to_string(),
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    dst.assert(predicate::path::is_file());
    dst.assert(predicate::str::contains("some file content"));

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get --exists=preserve <url> /tmp/testdata
///    src is a container url, uploaded with trailing slash
///    dst exists, and is a directory
///    dst contains only test.md, with 0 bytes.
///    expected result:
///        ../resources/testdata does not match /tmp/testdata/
///        /tmp/testdata still contains test.md with 0 bytes
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the specified
/// destination path on the local machine, when the destination has an existing file that has the
/// same file name as one of the files in the container being downloaded.
///
/// Given a unique `tmp_data_path` directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the files container xor address
/// And a unique `dest` file path is specified
/// And a markdown file is created at `dest/test.md`
///
/// When the `files get src dst --exists=preserve --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the `dest/test.md` markdown file should be preserved
/// And the `tmp_data_path` and `dest` paths should have different directory trees
/// And the other files in the container like `another.md` should be downloaded to `dest/another.md`
#[test]
fn files_get_exists_preserve() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let src = &files_container_xor;
    let test_md_file = assert_fs::NamedTempFile::new("test.md")?;
    test_md_file.write_str("some markdown content")?;
    let mut dst_pb = PathBuf::from(test_md_file.path());
    dst_pb.pop();
    let dst = dst_pb.as_path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            src,
            &dst,
            "--exists=preserve",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    test_md_file.assert(predicate::path::is_file());
    test_md_file.assert(predicate::str::contains("some markdown content"));
    assert_ne!(sum_tree(TEST_FOLDER)?, sum_tree(&dst)?);
    assert!(Path::new(&dst).join("another.md").is_file());

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get --exists=overwrite <url> /tmp/testdata
///    src is a container url, uploaded with trailing slash
///    dst exists, and is a directory
///    dst contains only test.md, with 0 bytes.
///    expected result:
///        ../resources/testdata matches /tmp/testdata
///
/// Details
/// -------
/// Scenario: the `files get` command downloads a directory from the network to the specified
/// destination path on the local machine, when the destination has an existing file that has the
/// same file name as one of the files in the container being downloaded.
///
/// Given a unique `tmp_data_path` directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the files container xor address
/// And a unique `dest` file path is specified
/// And a markdown file is created at `dest/test.md`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the `dest/test.md` markdown file should be overwritten
/// And the contents of `dest/test.md` should be the same as `tmp_data_path/test.md`
/// And the `tmp_data_path` and `dest` paths should have the same directory trees
#[test]
fn files_get_exists_overwrite() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let src = &files_container_xor;
    let test_md_file = assert_fs::NamedTempFile::new("test.md")?;
    test_md_file.write_str("some markdown content")?;
    let mut dst_pb = PathBuf::from(test_md_file.path());
    dst_pb.pop();
    let dst = dst_pb.as_path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(sum_tree(TEST_FOLDER)?, sum_tree(&dst)?);
    assert!(Path::new(&dst).join("another.md").is_file());
    assert_eq!(
        std::fs::read_to_string(Path::new(&format!("{}/test.md", dst)))?,
        "hello tests!"
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get <url>/path/is/invalid
///    src is a container url, uploaded without trailing slash
///    dst does not exist
///    src url contains a path, which is invalid, ie
///        no files in container match the path.
///    expected result:
///          command fails with exit code 1.
///          stderr contains string "Path not found"
///
/// Details
/// -------
/// Scenario: the `files get` command tries to download content from the network using an invalid
/// container path.
///
/// Given a unique `tmp_data_path` directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `safe://files_container_xor/path/is/invalid?v=0`
/// And a unique `dest` path is created and specified
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then the command should fail
/// And the the error output should contain "No data found for path '/path/is/invalid' on the
/// FilesContainer"
#[test]
fn files_get_src_path_is_invalid() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("/path/is/invalid");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let dst = dst.path().display().to_string();

    // Act
    let output = safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(1),
    )?;

    // Assert
    assert!(String::from_utf8_lossy(&output.stderr)
        .into_owned()
        .contains("no data found for path: /path/is/invalid/"));

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get <url> /tmp/foo/bar
///    src is a container url, uploaded without trailing slash
///    dst does not exist, /tmp/foo does not exist.
///    expected result:
///          command fails with exit code 1.
///          stderr contains string "No such directory:"
///
/// Details
/// -------
/// Scenario: the `files get` command tries to download content from the network using an invalid
/// destination path.
///
/// Given a unique `tmp_data_path` directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `files_container_xor`
/// And `dest` is set to `/non/existent/path`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then the command should fail
/// And the the error output should contain "No such directory"
#[test]
fn files_get_dst_parent_does_not_exist() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let src = &&files_container_xor;
    let dst = "/non/existent/path";

    // Act
    let output = safe_cmd(
        [
            "files",
            "get",
            src,
            dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(1),
    )?;

    // Assert
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr)
        .into_owned()
        .contains("No such directory:"));

    Ok(())
}

/*

// ----------------------------------------
// Path Matrix Tests
// ----------------------------------------

Path handling is a bit tricky.  The following matrix describes how unix
'cp' command handles various source/dest paths and states
(based on limited testing).

'safe files get' has been coded to use the same behavior, so we implement
a test case for each possibility.

-- source is a dir --

source     |source type| dst                      | dst exists | dst type | translated
---------------------------------------------------------------------------------------
testdata   | dir       | /tmp/testdata             | Y           | dir       | /tmp/testdata/testdata
testdata   | dir       | /tmp/testdata             | Y           | file      | error:  cannot overwrite non-directory '/tmp/testdata' with directory '../resources/testdata/'
testdata   | dir       | /tmp/testdata             | N           | --        | /tmp/testdata

testdata   | dir       | /tmp/newname              | Y           | dir       | /tmp/newname/testdata
testdata   | dir       | /tmp/newname              | Y           | file      | error:  cannot overwrite non-directory '/tmp/testdata' with directory '../resources/testdata/'
testdata   | dir       | /tmp/newname              | N           | --        | /tmp/newname

-- source is a file --

testdata   | file      | /tmp/testdata             | Y           | dir       | /tmp/testdata/testdata
testdata   | file      | /tmp/testdata             | Y           | file      | /tmp/testdata
testdata   | file      | /tmp/testdata             | N           | --        | /tmp/testdata

testdata   | file      | /tmp/newname              | Y           | dir       | /tmp/newname/testdata
testdata   | file      | /tmp/newname              | Y           | file      | /tmp/newname
testdata   | file      | /tmp/newname              | N           | --        | /tmp/newname

*/

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata /tmp/testdata
///    src is a dir
///    dst exists, and is a dir
///    expected result: ../resources/testdata matches /tmp/testdata/testdata
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the destination path
/// already exists as a directory.
///
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path/testdata` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path --recursive` command
/// And `src` is set to the `files_container_xor/testdata`
/// And a unique `dest` path is created and specified
/// And a directory is created at `dest/testdata`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the content should be downloaded to `dest/testdata/testdata`
/// And the `dest/testdata/testdata` tree should be the same as the testdata directory in the
/// repository.
#[test]
fn files_get_src_is_dir_and_dst_exists_as_dir() -> Result<()> {
    // Arrange
    let with_trailing_slash = false;
    let tmp_data_path = assert_fs::TempDir::new()?;
    let child = tmp_data_path.child("testdata");
    child.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) = upload_path(&child, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("testdata/");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let child = dst.child("testdata");
    child.create_dir_all()?;
    let dst = format!("{}/testdata", child.path().display());

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(sum_tree(TEST_FOLDER)?, sum_tree(&dst)?);

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata /tmp/testdata
///    src is a dir
///    dst exists, and is a file
///    expected result:
///        exit code = 1 and
///        stderr contains: "[Error] FileSystemError - cannot overwrite non-directory"
///
/// Details
/// -------
/// Scenario: the `files get` command tries to download content from the network when the
/// destination path is a file that already exists.
///
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path/testdata` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path --recursive` command
/// And `src` is set to the `files_container_xor/testdata`
/// And a unique `dest` directory is created
/// And a file is created at `dest/testdata`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then the command should fail
#[test]
fn files_get_src_is_dir_and_dst_exists_as_file() -> Result<()> {
    // Arrange
    let with_trailing_slash = false;
    let tmp_data_path = assert_fs::TempDir::new()?;
    let child = tmp_data_path.child("testdata");
    child.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) = upload_path(&child, with_trailing_slash)?;

    let url = safeurl_from(&files_container_xor)?;
    let src = format!(
        "{}://{}/{}?v=0",
        url.scheme(),
        url.public_name(),
        "testdata/",
    );
    let dst = assert_fs::TempDir::new()?;
    let child = dst.child("testdata");
    child.write_str("some file content")?;
    let dst = format!("{}/testdata", child.path().display());

    // Act
    let output = safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(1),
    )?;

    // Assert
    // So this command does fail as expected, but currently the error message isn't very helpful:
    // "No such directory". We should definitely adjust that to something more accurate.
    assert_eq!(output.status.code(), Some(1));

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata /tmp/testdata
///    src is a dir
///    dst does not exist
///    expected result: ../resources/testdata matches /tmp/testdata
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the destination path
/// has no `testdata` subdirectory.
///
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path/testdata` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path --recursive` command
/// And `src` is set to the `files_container_xor/testdata`
/// And a unique `dest` path is created and specified
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the `dest/testdata` tree should be the same as the testdata directory in the
/// repository.
#[test]
fn files_get_src_is_dir_and_dst_not_existing() -> Result<()> {
    // Arrange
    let with_trailing_slash = false;
    let tmp_data_path = assert_fs::TempDir::new()?;
    let child = tmp_data_path.child("testdata");
    child.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) = upload_path(&child, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("testdata/");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let dst = format!("{}/testdata", dst.path().display());

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(sum_tree(TEST_FOLDER)?, sum_tree(&dst)?);

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata /tmp/newname
///    src is a dir
///    dst exists, and is a dir
///    expected result: ../resources/testdata matches /tmp/newname/testdata
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the destination path
/// has no `testdata` subdirectory.
///
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path/testdata` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path --recursive` command
/// And `src` is set to the `files_container_xor/testdata`
/// And a unique `dest` path is created and specified
/// And a subdirectory is created at `dest/newname`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then it should download `src` to `dest`
/// And the `dest/testdata` tree should be the same as the testdata directory in the
/// repository.
#[test]
fn files_get_src_is_dir_and_dst_exists_as_newname_dir() -> Result<()> {
    // Arrange
    let with_trailing_slash = false;
    let tmp_data_path = assert_fs::TempDir::new()?;
    let child = tmp_data_path.child("testdata");
    child.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) = upload_path(&child, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("testdata/");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let child = dst.child("newname");
    child.create_dir_all()?;
    let dst = format!("{}/testdata", child.path().display());

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(sum_tree("../resources/testdata")?, sum_tree(&dst)?);

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata /tmp/newname
///    src is a dir
///    dst exists, and is a file
///    expected result:
///        exit code = 1 and
///        stderr contains: "[Error] FileSystemError - cannot overwrite non-directory"
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the destination path
/// has no `testdata` subdirectory.
///
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path/testdata` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path --recursive` command
/// And `src` is set to the `files_container_xor/testdata`
/// And a unique `dest` path is created and specified
/// And a file is created at `dest/newname`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then the command should fail
/// And the error message should contain "cannot overwrite non-directory"
#[test]
fn files_get_src_is_dir_and_dst_exists_as_newname_file() -> Result<()> {
    // Arrange
    let with_trailing_slash = false;
    let tmp_data_path = assert_fs::TempDir::new()?;
    let child = tmp_data_path.child("testdata");
    child.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) = upload_path(&child, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("testdata/");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let child = dst.child("newname");
    child.write_str("some file contents")?;
    let dst = child.path().display().to_string();

    // Act
    let output = safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(1),
    )?;

    // Assert
    assert!(String::from_utf8_lossy(&output.stderr)
        .into_owned()
        .contains("cannot overwrite non-directory"));
    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata/noextension /tmp/noextension
///    src is a file
///    dst exists, and is a dir
///    expected result: ../resources/testdata/noextension matches /tmp/noextension/noextension
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the source is a file
/// and the destination is a directory.
///
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `files_container_xor/noextension`
/// And a unique `dest` path is created and specified
/// And a dst is set to `dest/noextension`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then `src` should be downloaded to `dest/noextension` (this fully expands to
/// `dest/noextension/noextension`)
/// And the dst file should be the same as the noextension file in the repository testdata
#[test]
fn files_get_src_is_file_and_dst_exists_as_dir() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("noextension");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let child = dst.child("noextension");
    child.create_dir_all()?;
    let dst = child.path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(
        digest_file("../resources/testdata/noextension")?,
        digest_file(&format!("{}/noextension", dst))?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata/noextension /tmp/noextension
///    src is a file
///    dst exists, and is a file
///    expected result: ../resources/testdata/noextension matches /tmp/noextension
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the source is a file
/// and the destination is an existing file.
///
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `files_container_xor/noextension`
/// And a unique `dest` path is created and specified
/// And a file is written to `dest/noextension`
/// And a dst is set to `dest/noextension`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then `src` should be downloaded to `dest/noextension`
/// And the existing `dest/noextension` should be overwritten with `noextension` from the files
/// container
/// And `dest/noextension` should have the same content as `noextension` in the testdata in the
/// repository.
#[test]
fn files_get_src_is_file_and_dst_exists_as_file() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("noextension");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let child = dst.child("noextension");
    child.write_str("noextension is an existing file with some content")?;
    let dst = child.path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(
        digest_file("../resources/testdata/noextension")?,
        digest_file(&dst)?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata/noextension /tmp/noextension
///    src is a file
///    dst does not exist
///    expected result: ../resources/testdata/noextension matches /tmp/noextension
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the source is a file
/// and the destination is an existing file.
///
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `files_container_xor/noextension`
/// And a unique `dest` path is created and specified
/// And a dst is set to `dest/noextension`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then `src` should be downloaded to `dest/noextension`
/// And `dest/noextension` should have the same content as `noextension` in the testdata in the
/// repository.
#[test]
fn files_get_src_is_file_and_dst_not_existing() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("noextension");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let dst = dst.path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(
        digest_file("../resources/testdata/noextension")?,
        digest_file(&format!("{}/noextension", dst))?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata/noextension /tmp/newname
///    src is a file
///    dst exists, and is a dir with new name.
///    expected result: ../resources/testdata/noextension matches /tmp/newname/noextension
///
/// Scenario: the `files get` command downloads content from the network when the source is a file
/// and the destination is an existing file.
///
/// Details
/// -------
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `files_container_xor/noextension`
/// And a unique `dest` path is created and specified
/// And a subdirectory is created at `dest/newname`
/// And a dst is set to `dest/newname`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then `src` should be downloaded to `dest/noextension`
/// And `dest/newname/noextension` should have the same content as `noextension` in the testdata in the
/// repository.
#[test]
fn files_get_src_is_file_and_dst_exists_as_newname_dir() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("noextension");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let child = dst.child("newname");
    child.create_dir_all()?;
    let dst = child.path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(
        digest_file("../resources/testdata/noextension")?,
        digest_file(&format!("{}/noextension", dst))?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata/noextension /tmp/newname
///    src is a file
///    dst exists, and is a file with new name
///    expected result: ../resources/testdata/noextension matches /tmp/newname
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the source is a file
/// and the destination is an existing file.
///
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `files_container_xor/noextension`
/// And a unique `dest` path is created and specified
/// And a file is created at `dest/newname`
/// And a dst is set to `dest/newname`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then `src` should be downloaded to `dest/noextension`
/// And `dest/newname` should have the same content as `noextension` in the testdata in the
/// repository.
#[test]
fn files_get_src_is_file_and_dst_exists_as_newname_file() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("noextension");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let child = dst.child("newname");
    child.write_str("this file will be overwritten")?;
    let dst = child.path().display().to_string();

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(
        digest_file("../resources/testdata/noextension")?,
        digest_file(&dst)?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get ../resources/testdata/noextension /tmp/newname
///    src is a file
///    dst does not exist
///    expected result: ../resources/testdata/noextension matches /tmp/newname
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the source is a file
/// and the destination is an existing file.
///
/// Given a unique `tmp_data_path` directory is created
/// And a `tmp_data_path` sub directory is created
/// And the contents of `tmp_data_path` are uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `files_container_xor/noextension`
/// And a unique `dest` path is created and specified
/// And a dst is set to `dest/newname`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then `src` should be downloaded to `dest/noextension`
/// And `dest/newname` should have the same content as `noextension` in the testdata in the
/// repository.
#[test]
fn files_get_src_is_file_and_dst_newname_not_existing() -> Result<()> {
    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    tmp_data_path.copy_from("../resources/testdata", &["**"])?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let mut url = SafeUrl::from_url(&files_container_xor)?;
    url.set_content_version(None);
    url.set_path("noextension");
    let src = url.to_string();

    let dst = assert_fs::TempDir::new()?;
    let dst = format!("{}/newname", dst.path().display());

    // Act
    safe_cmd(
        [
            "files",
            "get",
            &src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(
        digest_file("../resources/testdata/noextension")?,
        digest_file(&dst)?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get <src> /tmp/newname
///    src is xor-url generated from `safe files put ./test_symlinks`
///    dst does not exist
///    expected result: ./test_symlinks matches /tmp/newname
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the content contains
/// relative symlinks.
///
/// Given the contents of `./test_symlinks` are uploaded using `files put ./test_symlinks/
/// --recursive` command
/// And `src` is set to the `files_container_xor`
/// And a unique `dest` path is created and specified
/// And a dst is set to `dest/newname`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then `src` should be downloaded to `dest/newname`
/// And the `dest/newname` tree should match the `./test_symlinks` tree with the relative symlinks
/// still functional.
#[test]
fn files_get_symlinks_relative() -> Result<()> {
    // Bail if test_symlinks not valid, or cannot write a test symlink.
    // Typically indicates missing perms on windows.
    if !test_symlinks_are_valid()? || !can_write_symlinks() {
        return Ok(());
    }

    // Arrange
    // The assert_fs `copy_from` function, that's used in all the other tests, doesn't work
    // correctly with the symlink directory, so we just upload it directly.
    let with_trailing_slash = true;
    let (files_container_xor, _processed_files, _) =
        upload_path("../resources/test_symlinks", with_trailing_slash)?;

    let src = &files_container_xor;
    let dst = assert_fs::TempDir::new()?;
    let dst = format!("{}/newname", dst.path().display());

    // Act
    safe_cmd(
        [
            "files",
            "get",
            src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(sum_tree("../resources/test_symlinks")?, sum_tree(&dst)?);
    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get <xor-url>/absolute_symlinks /tmp/newname
///    src is symlinks test dir containing absolute-path links
///    dst does not exist
///    expected result: source directory matches /tmp/newname
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the content contains
/// absolute symlinks.
///
/// Given a unique `tmp_data_path` directory is created
/// And absolute symlinks are created within the `tmp_data_path` directory
/// And `tmp_data_path` is uploaded using the `files put tmp_data_path/ --recursive` command
/// And `src` is set to the `files_container_xor`
/// And a unique `dest` path is created and specified
/// And a dst is set to `dest/newname`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then `src` should be downloaded to `dest/newname`
/// And the `dest/newname` tree should match the `tmp_data_path` tree with absolute symlinks.
#[test]
fn files_get_symlinks_absolute() -> Result<()> {
    // Bail if cannot write a test symlink.
    // Typically indicates missing perms on windows.
    if !can_write_symlinks() {
        return Ok(());
    }

    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    create_absolute_symlinks_directory(&tmp_data_path)?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let src = &files_container_xor;
    let dst = assert_fs::TempDir::new()?;
    let dst = format!("{}/newname", dst.path().display());

    // Act
    safe_cmd(
        [
            "files",
            "get",
            src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(
        sum_tree(&tmp_data_path.path().display().to_string())?,
        sum_tree(&dst)?
    );

    Ok(())
}

/// Synopsis
/// --------
/// Test:  safe files get <xor-url>/absolute_symlinks /tmp/newname
///    src is symlinks test dir containing absolute-path links
///    dst does not exist
///    expected result: source directory matches /tmp/newname
///
/// Details
/// -------
/// Scenario: the `files get` command downloads content from the network when the content contains
/// absolute symlinks.
///
/// Given a unique `tmp_data_path` directory is created
/// And absolute symlinks are created within the `tmp_data_path` directory
/// And `tmp_data_path` is uploaded using the `files put tmp_data_path/ --recursive` command
/// And a new text file is created at `tmp_data_path/new_symlink_target`
/// And a new `new_symlink_path` symlink is created to point to `tmp_data_path/new_symlink_target`
/// And a `safeurl` is created from `files_container_xor`
/// And the new file and symlink are uploaded using the `files sync safeurl tmp_data_path/`
/// And `src` is set to `safeurl`
/// And a unique `dest` path is created and specified
/// And `dest` is set to `dest/newname`
///
/// When the `files get src dst --exists=overwrite --progress=none` command runs
///
/// Then `src` should be downloaded to `dest/newname`
/// And the `dest/newname` tree should match the `tmp_data_path` tree with absolute symlinks and
/// the newly sync'd symlinks.
#[test]
fn files_get_symlinks_after_sync() -> Result<()> {
    // Bail if cannot write a test symlink.
    // Typically indicates missing perms on windows.
    if !can_write_symlinks() {
        return Ok(());
    }

    // Arrange
    let with_trailing_slash = true;
    let tmp_data_path = assert_fs::TempDir::new()?;
    create_absolute_symlinks_directory(&tmp_data_path)?;
    let (files_container_xor, _processed_files, _) =
        upload_path(&tmp_data_path, with_trailing_slash)?;

    let mut safeurl = safeurl_from(&files_container_xor)?;
    safeurl.set_content_version(None);

    let new_symlink_path = Path::new(tmp_data_path.path()).join("newlink");
    let new_symlink_target = tmp_data_path.child("newlink_target");
    new_symlink_target.write_str("content for target file")?;
    create_symlink(&new_symlink_target, &new_symlink_path, false).map_err(|e| eyre!("{:?}", e))?;

    safe_cmd_stdout(
        [
            "files",
            "sync",
            &format!("{}/", tmp_data_path.path().display()),
            &safeurl.to_string(),
        ],
        Some(0),
    )?;

    let src = &safeurl.to_string();
    let dst = assert_fs::TempDir::new()?;
    let dst = format!("{}/newname", dst.path().display());

    // Act
    safe_cmd(
        [
            "files",
            "get",
            src,
            &dst,
            "--exists=overwrite",
            "--progress=none",
        ],
        Some(0),
    )?;

    // Assert
    assert_eq!(
        sum_tree(&tmp_data_path.path().display().to_string())?,
        sum_tree(&dst)?
    );

    Ok(())
}
