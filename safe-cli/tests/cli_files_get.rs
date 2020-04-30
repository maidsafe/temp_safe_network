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

const TESTDATA: &str = "testdata";
const NEWNAME: &str = "newname";
const SUBFOLDER: &str = "subfolder";

const EXISTS_OVERWRITE: &str = "overwrite";
const EXISTS_PRESERVE: &str = "preserve";
const PROGRESS_NONE: &str = "none";

use multibase::{encode, Base};
use safe_api::xorurl::XorUrlEncoder;
use safe_cmd_test_utilities::{
    parse_files_put_or_sync_output, upload_testfolder_no_trailing_slash,
    upload_testfolder_trailing_slash, TEST_FOLDER,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_keccak::sha3_256;
use walkdir::{DirEntry, WalkDir};

const NOEXTENSION: &str = "noextension";
const NOEXTENSION_PATH: &str = "../testdata/noextension";

// ----------------------------------------
// Container URL (without url path) Tests
// ----------------------------------------

// Test:  safe files get <url> /tmp/testdata
//    src is a container url
//    dest exists, and is a directory
//
//    expected result: ../testdata matches /tmp/testdata/testdata
#[test]
fn files_get_src_is_container_and_dest_is_dir() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = &files_container_xor;
    let dest = dest_dir(&[TESTDATA]);
    let final_dest = dest_dir(&[TESTDATA, TESTDATA]);

    remove_dest(&dest);
    fs::create_dir_all(&dest).unwrap();

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(sum_tree(TEST_FOLDER), sum_tree(&final_dest));

    Ok(())
}

// Test:  safe files get <url> /tmp/testdata
//    src is a container url, uploaded with trailing slash
//    dest exists, and is an empty directory
//
//    expected result: ../testdata matches /tmp/testdata
#[test]
fn files_get_src_is_container_trailing_and_dest_is_dir() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_trailing_slash()?;

    let src = &files_container_xor;
    let dest = dest_dir(&[TESTDATA]);

    remove_dest(&dest);
    fs::create_dir_all(&dest).unwrap();

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(sum_tree(TEST_FOLDER), sum_tree(&dest));

    Ok(())
}

// Test:  safe files get <url> .
//    src is a container url, testdata put without slash.
//    dest is the current working directory.
//
//    expected result: ../testdata matches ./testdata
#[test]
fn files_get_src_is_container_and_dest_is_cwd() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = &files_container_xor;
    let dest = ".";
    let final_dest = Path::new(dest).join(TESTDATA).display().to_string();

    remove_dest(&final_dest);

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(sum_tree(TEST_FOLDER), sum_tree(&final_dest));

    Ok(())
}

// Test:  safe files get <url>
//    src is a container url, testdata put without slash.
//    dest is unspecified.  (should default to the current working directory)
//
//    expected result: ../testdata matches ./testdata
#[test]
fn files_get_src_is_container_and_dest_is_unspecified() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = &files_container_xor;
    let final_dest = Path::new(".").join(TESTDATA).display().to_string();

    remove_dest(&final_dest);

    files_get(
        &src,
        None, // dest is not provided!
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(sum_tree(TEST_FOLDER), sum_tree(&final_dest));

    Ok(())
}

// From files_get.rs
//
// It is an error/warning if the dest path attempts to use
// an existing file as a directory. But other files should
// still be written.  eg:
// $ mkdir -p /tmp/a/b/c && touch /tmp/a/file.txt
// $ mkdir /tmp/target && touch /tmp/target/b   (b is a file)
// $ cp -r /tmp/a/* /tmp/target
//    cp: cannot overwrite non-directory '/tmp/target/b' with directory '/tmp/a/b'
// $ ls -l /tmp/target/
//      total 0
//      -rw-rw-r-- 1 user user 0 Mar 31 14:38 b         (b still a file)
//      -rw-rw-r-- 1 user user 0 Mar 31 14:38 file.txt  (other file written)
//
// note: cp returns exit code of 1 when this occurs, but safe-cli
//       presently returns 0.
//
// ----------------
//
// Test:  safe files get <url> /tmp/testdata
//    src is a container url, uploaded with trailing slash
//    dest exists, and is a directory
//    src contains 'subfolder', which is a directory
//    dest contains 'subfolder' (a file) and is otherwise empty.
//
//    expected result:
//        exit code = 0 and
//        /tmp/testdata/subfolder is still a file and
//        /tmp/testdata/test.md exists
//        /tmp/testdata/another.md exists
//        stderr contains: "Warning: cannot overwrite non-directory"
#[test]
fn files_get_attempt_overwrite_sub_file_with_dir() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_trailing_slash()?;

    let src = &files_container_xor;
    let dest = dest_dir(&[TESTDATA]);

    remove_dest(&dest);
    fs::create_dir_all(&dest).unwrap();
    let existing_file = Path::new(&dest).join("subfolder");
    let f = fs::File::create(&existing_file).unwrap();
    drop(f); // close file.

    let cmd_output = files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    // verify that src and dest folders don't match.
    assert_ne!(sum_tree(TEST_FOLDER), sum_tree(&dest));

    // Check that exit code is 0, existing file remains, and new files written.
    assert_eq!(cmd_output.status.code().unwrap(), 0);
    assert!(existing_file.is_file());
    assert!(Path::new(&dest).join("test.md").is_file());
    assert!(Path::new(&dest).join("another.md").is_file());

    // Disabled for now because the warning is disabled by safe-cli if TTY
    // not detected.  So it doesn't appear in our output.  Perhaps later
    // safe-cli adds a flag to override TTY detection.
    // assert!(String::from_utf8_lossy(&cmd_output.stderr).into_owned().contains("Warning: cannot overwrite non-directory"));

    Ok(())
}

// ----------------------------------------
// nrs tests.
// ----------------------------------------

// Test:  safe files get <nrs_url>
//    src is an nrs url, linked to a container url, testdata put without slash.
//    dest is unspecified.  (should default to the current working directory)
//
//    expected result: ../testdata matches ./testdata
#[test]
fn files_get_src_is_nrs_and_dest_is_unspecified() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let mut nrs_name = "NRS_NAME".to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    nrs_name.push_str(&str_to_sha3_256(&format!("{}", now)));

    cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "nrs",
        "create",
        &nrs_name,
        "-l",
        &files_container_xor
    )
    .read()
    .map_err(|e| format!("{:#?}", e))?;

    let src = format!("safe://{}", &nrs_name);
    let final_dest = Path::new(".").join(TESTDATA).display().to_string();

    remove_dest(&final_dest);

    files_get(
        &src,
        None, // dest is not provided!
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(sum_tree(TEST_FOLDER), sum_tree(&final_dest));

    remove_dest(&final_dest);

    Ok(())
}

// Test:  safe files get <nrs_url>+path concatenated to <xor url>+path
//    src is an nrs url with a path, linked to an container xor url with a path.
//       xorurl ==> safe://.../testdata/subfolder
//       nrsurl ==> safe://nrsname/sub2.md
//    dest is /tmp/sub2.md
//
//    path to sub2.md in FileContainer is /testdata/subfolder/sub2.md
//
//    expected result: ../testdata/subfolder/sub2.md matches /tmp/sub2.md
#[test]
fn files_get_src_is_nrs_with_path_and_dest_is_unspecified() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    const TEST_FILE: &str = "sub2.md";

    // make safe://.../testdata/subfolder
    let xor_path = join_paths(&[TESTDATA, SUBFOLDER]);
    let mut e = XorUrlEncoder::from_url(&files_container_xor)?;
    e.set_path(&xor_path);
    let xor_url_with_path = e.to_string();

    let mut nrs_name = "NRS_NAME".to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    nrs_name.push_str(&str_to_sha3_256(&format!("{}", now)));

    cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "nrs",
        "create",
        &nrs_name,
        "-l",
        &xor_url_with_path
    )
    .read()
    .map_err(|e| format!("{:#?}", e))?;

    // make safe://nrsname/sub2.md
    let src = format!("safe://{}/{}", &nrs_name, TEST_FILE);

    let dest = dest_dir(&[]);
    let final_dest = dest_dir(&[TEST_FILE]);

    remove_dest(&final_dest);

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    let file_src = join_paths(&[TEST_FOLDER, SUBFOLDER, TEST_FILE]);
    assert_eq!(sum_tree(&file_src), sum_tree(&final_dest));

    remove_dest(&final_dest);

    Ok(())
}

// note: there should be additional NRS tests with paths here, but presently
// NRS and paths do not mix well.

// ----------------------------------------
// embedded spaces in paths tests.
// ----------------------------------------

// Test:  safe files get "safe://.../dir with space/file with space" "/tmp/new file"
//    src is a file, directory and file both contain embedded spaces, not url encoded.
//    dest does not exist
//
//    expected result: /tmp/new file is written without error.
#[test]
fn files_get_src_has_embedded_spaces_and_dest_also() -> Result<(), String> {
    const DIR_WITH_SPACE: &str = "dir with space";
    const FILE_WITH_SPACE: &str = "file with space";
    const NEW_FILE_WITH_SPACE: &str = "new file";

    // setup: remove (if existing) and then create "/tmp/dir with space/file with space"
    // which will be our source dir to PUT, then GET
    let src_dir = dest_dir(&[DIR_WITH_SPACE]);
    let src_file = dest_dir(&[DIR_WITH_SPACE, FILE_WITH_SPACE]);
    remove_dest(&src_dir);
    fs::create_dir_all(&src_dir).unwrap();
    let f = fs::File::create(&src_file).unwrap();
    drop(f); // close file.

    let files_container = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "files",
        "put",
        &src_dir,
        "--recursive",
        "--json"
    )
    .read()
    .map_err(|e| format!("{:#?}", e))?;

    let (files_container_xor, _) = parse_files_put_or_sync_output(&files_container);

    let src = source_path(&files_container_xor, &[DIR_WITH_SPACE, FILE_WITH_SPACE])?;
    let dest = dest_dir(&[NEW_FILE_WITH_SPACE]);

    remove_dest(&dest);

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert!(Path::new(&dest).is_file());
    assert_eq!(sum_tree(&dest), sum_tree(&src_file));

    Ok(())
}

// Test:  safe files get "safe://.../dir%20with%20space/file%20with%20space" "/tmp/new file"
//    src is a file, directory and file both contain embedded spaces, url-encoded.
//    dest does not exist
//
//    expected result: /tmp/new file is written without error.
#[test]
fn files_get_src_has_encoded_spaces_and_dest_also() -> Result<(), String> {
    const DIR_WITH_SPACE: &str = "dir with space";
    const DIR_WITH_SPACE_ENCODED: &str = "dir%20with%20space";
    const FILE_WITH_SPACE: &str = "file with space";
    const FILE_WITH_SPACE_ENCODED: &str = "file%20with%20space";
    const NEW_FILE_WITH_SPACE: &str = "new file";

    // setup: remove (if existing) and then create "/tmp/dir with space/file with space"
    // which will be our source dir to PUT, then GET
    let src_dir = dest_dir(&[DIR_WITH_SPACE]);
    let src_file = dest_dir(&[DIR_WITH_SPACE, FILE_WITH_SPACE]);
    remove_dest(&src_dir);
    fs::create_dir_all(&src_dir).unwrap();
    let f = fs::File::create(&src_file).unwrap();
    drop(f); // close file.

    let files_container = cmd!(
        env!("CARGO_BIN_EXE_safe"),
        "files",
        "put",
        &src_dir,
        "--recursive",
        "--json"
    )
    .read()
    .map_err(|e| format!("{:#?}", e))?;

    let (files_container_xor, _) = parse_files_put_or_sync_output(&files_container);

    let src = source_path(
        &files_container_xor,
        &[DIR_WITH_SPACE_ENCODED, FILE_WITH_SPACE_ENCODED],
    )?;
    let dest = dest_dir(&[NEW_FILE_WITH_SPACE]);

    remove_dest(&dest);

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert!(Path::new(&dest).is_file());
    assert_eq!(sum_tree(&dest), sum_tree(&src_file));

    Ok(())
}

// ----------------------------------------
// Option --exists Tests
// ----------------------------------------

// Note: not testing --exists=ask because it is interactive.

// Test:  safe files get --exists=preserve <url> /tmp/testdata
//    src is a container url, uploaded with trailing slash
//    dest exists, and is a directory
//    dest contains only test.md, with 0 bytes.
//
//    expected result:
//        ../testdata does not match /tmp/testdata/
//        /tmp/testdata still contains test.md with 0 bytes
//        /tmp/testdata also contains another.md
#[test]
fn files_get_exists_preserve() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_trailing_slash()?;

    let src = &files_container_xor;
    let dest = dest_dir(&[TESTDATA]);

    remove_dest(&dest);
    fs::create_dir_all(&dest).unwrap();
    let existing_file = Path::new(&dest).join("test.md");
    let f = fs::File::create(&existing_file).unwrap();
    drop(f); // close file.

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_PRESERVE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_ne!(sum_tree(TEST_FOLDER), sum_tree(&dest));
    assert_eq!(existing_file.metadata().unwrap().len(), 0); // file size = 0.
    assert!(Path::new(&dest).join("another.md").is_file());

    Ok(())
}

// Test:  safe files get --exists=overwrite <url> /tmp/testdata
//    src is a container url, uploaded with trailing slash
//    dest exists, and is a directory
//    dest contains only test.md, with 0 bytes.
//
//    expected result:
//        ../testdata matches /tmp/testdata
#[test]
fn files_get_exists_overwrite() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_trailing_slash()?;

    let src = &files_container_xor;
    let dest = dest_dir(&[TESTDATA]);

    remove_dest(&dest);
    fs::create_dir_all(&dest).unwrap();
    let existing_file = Path::new(&dest).join("test.md");
    let f = fs::File::create(&existing_file).unwrap();
    drop(f); // close file.

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(sum_tree(TEST_FOLDER), sum_tree(&dest));

    Ok(())
}

// ----------------------------------------
// Errors / Failure Tests
// ----------------------------------------

// Test:  safe files get <url>/path/is/invalid
//    src is a container url, uploaded without trailing slash
//    dest does not exist
//    src url contains a path, which is invalid, ie
//        no files in container match the path.
//
//    expected result:
//          command fails with exit code 1.
//          stderr contains string "Path not found"
#[test]
fn files_get_src_path_is_invalid() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &["path", "is", "invalid"])?;
    let dest = dest_dir(&[TESTDATA]);

    remove_dest(&dest);

    let cmd_output = files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(1), // exit code must be 1
    )?;

    assert!(String::from_utf8_lossy(&cmd_output.stderr)
        .into_owned()
        .contains("Path not found"));

    Ok(())
}

// Test:  safe files get <url> /tmp/foo/bar
//    src is a container url, uploaded without trailing slash
//    dest does not exist, /tmp/foo does not exist.
//
//    expected result:
//          command fails with exit code 1.
//          stderr contains string "No such directory:"
#[test]
fn files_get_dest_parent_does_not_exist() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[])?;
    let dest = dest_dir(&[TESTDATA, "foo", "bar"]);

    remove_dest(&dest_dir(&[TESTDATA]));

    let cmd_output = files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(1), // exit code must be 1
    )?;

    // Check that exit code is 1, and correct error message written to stderr.
    assert_eq!(cmd_output.status.code().unwrap(), 1);
    assert!(String::from_utf8_lossy(&cmd_output.stderr)
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

source     |source type| dest                      | dest exists | dest type | translated
---------------------------------------------------------------------------------------
testdata   | dir       | /tmp/testdata             | Y           | dir       | /tmp/testdata/testdata
testdata   | dir       | /tmp/testdata             | Y           | file      | error:  cannot overwrite non-directory '/tmp/testdata' with directory '../testdata/'
testdata   | dir       | /tmp/testdata             | N           | --        | /tmp/testdata

testdata   | dir       | /tmp/newname              | Y           | dir       | /tmp/newname/testdata
testdata   | dir       | /tmp/newname              | Y           | file      | error:  cannot overwrite non-directory '/tmp/testdata' with directory '../testdata/'
testdata   | dir       | /tmp/newname              | N           | --        | /tmp/newname

-- source is a file --

testdata   | file      | /tmp/testdata             | Y           | dir       | /tmp/testdata/testdata
testdata   | file      | /tmp/testdata             | Y           | file      | /tmp/testdata
testdata   | file      | /tmp/testdata             | N           | --        | /tmp/testdata

testdata   | file      | /tmp/newname              | Y           | dir       | /tmp/newname/testdata
testdata   | file      | /tmp/newname              | Y           | file      | /tmp/newname
testdata   | file      | /tmp/newname              | N           | --        | /tmp/newname

*/

// ----------------------------------------
// Path Matrix Tests: Source is a Directory
// ----------------------------------------

// Test:  safe files get ../testdata /tmp/testdata
//    src is a dir
//    dest exists, and is a dir
//
//    expected result: ../testdata matches /tmp/testdata/testdata
#[test]
fn files_get_src_is_dir_and_dest_exists_as_dir() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA])?;
    let dest = dest_dir(&[TESTDATA]);
    let final_dest = dest_dir(&[TESTDATA, TESTDATA]);

    remove_dest(&dest);
    fs::create_dir_all(&dest).unwrap();

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(sum_tree(TEST_FOLDER), sum_tree(&final_dest));

    Ok(())
}

// Test:  safe files get ../testdata /tmp/testdata
//    src is a dir
//    dest exists, and is a file
//
//    expected result:
//        exit code = 1 and
//        stderr contains: "[Error] FileSystemError - cannot overwrite non-directory"
#[test]
fn files_get_src_is_dir_and_dest_exists_as_file() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA])?;
    let dest = dest_dir(&[TESTDATA]);

    remove_dest(&dest);
    let f = fs::File::create(&dest).unwrap();
    drop(f); // close file.

    let cmd_output = files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(1), // exit code must be 1
    )?;

    assert!(String::from_utf8_lossy(&cmd_output.stderr)
        .into_owned()
        .contains("[Error] FileSystemError - cannot overwrite non-directory"));

    Ok(())
}

// Test:  safe files get ../testdata /tmp/testdata
//    src is a dir
//    dest does not exist
//
//    expected result: ../testdata matches /tmp/testdata
#[test]
fn files_get_src_is_dir_and_dest_not_existing() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA])?;
    let dest = dest_dir(&[TESTDATA]);

    remove_dest(&dest);

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(sum_tree(TEST_FOLDER), sum_tree(&dest));

    Ok(())
}

// ----

// Test:  safe files get ../testdata /tmp/newname
//    src is a dir
//    dest exists, and is a dir
//
//    expected result: ../testdata matches /tmp/newname/testdata
#[test]
fn files_get_src_is_dir_and_dest_exists_as_newname_dir() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA])?;
    let dest = dest_dir(&[NEWNAME]);
    let final_dest = dest_dir(&[NEWNAME, TESTDATA]);

    remove_dest(&dest);
    fs::create_dir_all(&dest).unwrap();

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(sum_tree(TEST_FOLDER), sum_tree(&final_dest));

    Ok(())
}

// Test:  safe files get ../testdata /tmp/newname
//    src is a dir
//    dest exists, and is a file
//
//    expected result:
//        exit code = 1 and
//        stderr contains: "[Error] FileSystemError - cannot overwrite non-directory"
#[test]
fn files_get_src_is_dir_and_dest_exists_as_newname_file() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA])?;
    let dest = dest_dir(&[NEWNAME]);

    remove_dest(&dest);
    let f = fs::File::create(&dest).unwrap();
    drop(f); // close file.

    let cmd_output = files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(1), // exit code must be 1
    )?;

    assert!(String::from_utf8_lossy(&cmd_output.stderr)
        .into_owned()
        .contains("[Error] FileSystemError - cannot overwrite non-directory"));

    Ok(())
}

// Test:  safe files get ../testdata /tmp/newname
//    src is a dir
//    dest does not exist
//
//    expected result: ../testdata matches /tmp/newname
#[test]
fn files_get_src_is_dir_and_dest_newname_not_existing() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA])?;
    let dest = dest_dir(&[NEWNAME]);

    remove_dest(&dest);

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(sum_tree(TEST_FOLDER), sum_tree(&dest));

    Ok(())
}

// ----------------------------------------
// Path Matrix Tests: Source is a file
// ----------------------------------------

// Test:  safe files get ../testdata/noextension /tmp/noextension
//    src is a file
//    dest exists, and is a dir
//
//    expected result: ../testdata/noextension matches /tmp/noextension/noextension
#[test]
fn files_get_src_is_file_and_dest_exists_as_dir() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA, NOEXTENSION])?;
    let dest = dest_dir(&[NOEXTENSION]);
    let final_dest = dest_dir(&[NOEXTENSION, NOEXTENSION]);

    remove_dest(&dest);
    fs::create_dir_all(&dest).unwrap();

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(digest_file(NOEXTENSION_PATH), digest_file(&final_dest));

    Ok(())
}

// Test:  safe files get ../testdata/noextension /tmp/noextension
//    src is a file
//    dest exists, and is a file
//
//    expected result: ../testdata/noextension matches /tmp/noextension
#[test]
fn files_get_src_is_file_and_dest_exists_as_file() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA, NOEXTENSION])?;
    let dest = dest_dir(&[NOEXTENSION]);

    remove_dest(&dest);
    let f = fs::File::create(&dest).unwrap();
    drop(f); // close file.

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(digest_file(NOEXTENSION_PATH), digest_file(&dest));

    Ok(())
}

// Test:  safe files get ../testdata/noextension /tmp/noextension
//    src is a file
//    dest does not exist
//
//    expected result: ../testdata/noextension matches /tmp/noextension
#[test]
fn files_get_src_is_file_and_dest_not_existing() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA, NOEXTENSION])?;
    let dest = dest_dir(&[NOEXTENSION]);

    remove_dest(&dest);

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(digest_file(NOEXTENSION_PATH), digest_file(&dest));

    Ok(())
}

// ----

// Test:  safe files get ../testdata/noextension /tmp/newname
//    src is a file
//    dest exists, and is a dir with new name.
//
//    expected result: ../testdata/noextension matches /tmp/newname/noextension
#[test]
fn files_get_src_is_file_and_dest_exists_as_newname_dir() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA, NOEXTENSION])?;
    let dest = dest_dir(&[NEWNAME]);
    let final_dest = dest_dir(&[NEWNAME, NOEXTENSION]);

    remove_dest(&dest);
    fs::create_dir_all(&dest).unwrap();

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(digest_file(NOEXTENSION_PATH), digest_file(&final_dest));

    Ok(())
}

// Test:  safe files get ../testdata/noextension /tmp/newname
//    src is a file
//    dest exists, and is a file with new name
//
//    expected result: ../testdata/noextension matches /tmp/newname
#[test]
fn files_get_src_is_file_and_dest_exists_as_newname_file() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA, NOEXTENSION])?;
    let dest = dest_dir(&[NEWNAME]);

    remove_dest(&dest);
    let f = fs::File::create(&dest).unwrap();
    drop(f); // close file.

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(digest_file(NOEXTENSION_PATH), digest_file(&dest));

    Ok(())
}

// Test:  safe files get ../testdata/noextension /tmp/newname
//    src is a file
//    dest does not exist
//
//    expected result: ../testdata/noextension matches /tmp/newname
#[test]
fn files_get_src_is_file_and_dest_newname_not_existing() -> Result<(), String> {
    let (files_container_xor, _processed_files) = upload_testfolder_no_trailing_slash()?;

    let src = source_path(&files_container_xor, &[TESTDATA, NOEXTENSION])?;
    let dest = dest_dir(&[NEWNAME]);

    remove_dest(&dest);

    files_get(
        &src,
        Some(&dest),
        Some(EXISTS_OVERWRITE),
        Some(PROGRESS_NONE),
        Some(0),
    )?;

    assert_eq!(digest_file(NOEXTENSION_PATH), digest_file(&dest));

    Ok(())
}

// recursively removes a directory, or a file.
// intended for removal of dir/files downloaded
// by 'safe files get' test cases.
fn remove_dest(path: &str) {
    let p = Path::new(path);
    if p.is_file() {
        fs::remove_file(&path).unwrap();
    } else if p.is_dir() {
        fs::remove_dir_all(&path).unwrap();
    }
}

// callback for WalkDir::new() in sum_tree()
fn not_hidden_or_empty(entry: &DirEntry, max_depth: usize) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() <= max_depth && (entry.depth() == 0 || !s.starts_with('.')))
        .unwrap_or(false)
}

// generates a sha3_256 digest/hash of a directory tree.
//
// Note: hidden files or empty directories are not included.
//  this is necessary for comparing ../testdata with
//  dest dir since `safe files put` presently ignores hidden
//  files.  The hidden files can be included once
//  'safe files put' is fixed to include them.
fn sum_tree(path: &str) -> String {
    let paths = WalkDir::new(path)
        .min_depth(1) // ignore top/root directory
        .follow_links(false)
        .sort_by(|a, b| a.path().cmp(b.path()))
        .into_iter()
        .filter_entry(|e| not_hidden_or_empty(e, 20))
        .filter_map(|v| v.ok());

    let mut digests = String::new();
    for p in paths {
        if p.path().is_dir() && dir_is_empty(&p.path()) {
            continue;
        }
        let relpath = p.path().strip_prefix(path).unwrap().display().to_string();
        digests.push_str(&str_to_sha3_256(&relpath));
        if p.path().is_file() {
            digests.push_str(&digest_file(&p.path().display().to_string()));
        }
    }
    str_to_sha3_256(&digests)
}

// checks if a directory is empty, ignoring hidden files.
// ie, a dir containing only hidden files is considered empty.
//
// Note: this is necessary for comparing ../testdata with
// dest dir since `safe files put` presently ignores hidden
// files.  The hidden files can be included once
// 'safe files put' is fixed to include them.
fn dir_is_empty(path: &Path) -> bool {
    let entries = path.read_dir().unwrap();
    for e in entries {
        if !e.unwrap().file_name().to_str().unwrap().starts_with('.') {
            return false;
        }
    }
    true
}

// returns sha3_256 digest/hash of a file as a string.
fn digest_file(path: &str) -> String {
    let data = fs::read_to_string(&path).unwrap();
    str_to_sha3_256(&data)
}

// Executes `safe files get` with dynamic args and options.
fn files_get(
    url: &str,
    dest: Option<&str>,
    exists: Option<&str>,
    progress: Option<&str>,
    expect_exit_code: Option<i32>,
) -> Result<process::Output, String> {
    // arg/option with empty string are filtered out.
    let args: Vec<String> = vec![
        "files".to_string(),
        "get".to_string(),
        url.to_string(),
        cmd_arg(dest),
        cmd_option("exists", exists),
        cmd_option("progress", progress),
    ]
    .into_iter()
    .filter(|a| !a.is_empty())
    .collect();

    let output = duct::cmd(env!("CARGO_BIN_EXE_safe"), &args)
        .stdout_capture()
        .stderr_capture()
        .unchecked()
        .run()
        .map_err(|e| format!("{:#?}", e))?;

    if let Some(ec) = expect_exit_code {
        match output.status.code() {
            Some(code) => assert_eq!(ec, code),
            None => return Err("Command returned no exit code".to_string()),
        }
    }
    Ok(output)
}

// For dynamically generating cmd args.
fn cmd_arg(val: Option<&str>) -> String {
    match val {
        Some(v) => v.to_string(),
        None => "".to_string(),
    }
}

// For dynamically generating cmd options.
//
// generates an "--option=value" string, or "" if
// val is None
fn cmd_option(name: &str, val: Option<&str>) -> String {
    match val {
        Some(v) => format!("--{}={}", name, v),
        None => "".to_string(),
    }
}

// returns sha3_256 hash of input string as a string.
fn str_to_sha3_256(s: &str) -> String {
    let bytes = sha3_256(&s.to_string().into_bytes());
    encode(Base::Base32, bytes)
}

// constructs a destination directory path
// within system temp directory.
fn dest_dir(path: &[&str]) -> String {
    let pb: PathBuf = path.iter().collect();
    env::temp_dir().join(pb).display().to_string()
}

// joins path components together.
fn join_paths(path: &[&str]) -> String {
    let pb: PathBuf = path.iter().collect();
    pb.display().to_string()
}

// sets/appends path in a provided safe URL.  preserves query string.
fn source_path(url: &str, path: &[&str]) -> Result<String, String> {
    let pb: PathBuf = path.iter().collect();

    let x = XorUrlEncoder::from_url(&url).map_err(|e| format!("{:#?}", e))?;

    let url = format!(
        "{}://{}/{}{}{}",
        x.scheme(),
        x.full_name(),
        pb.display().to_string(),
        x.query_string_with_separator(),
        x.fragment_with_separator()
    );
    Ok(url)
}
