// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use multibase::{encode, Base};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sn_api::{
    fetch::SafeData, files::ProcessedFiles, wallet::WalletSpendableBalances, Error, Keypair,
};
use sn_data_types::Money;
use std::collections::BTreeMap;
use std::path::Path;
use std::{env, fs, process, str::FromStr};
use tiny_keccak::sha3_256;
use walkdir::{DirEntry, WalkDir};

#[macro_use]
extern crate duct;

#[allow(dead_code)]
pub const CLI: &str = "safe";
#[allow(dead_code)]
pub const SAFE_PROTOCOL: &str = "safe://";

pub const TEST_FOLDER: &str = "../testdata/";
pub const TEST_FOLDER_NO_TRAILING_SLASH: &str = "../testdata";
pub const TEST_SYMLINKS_FOLDER: &str = "../test_symlinks";
pub const TEST_SYMLINK: &str = "../test_symlinks/file_link";

#[allow(dead_code)]
pub fn get_bin_location() -> String {
    let target_dir = match env::var("CARGO_TARGET_DIR") {
        Ok(target_dir) => target_dir,
        Err(_) => "../target".to_string(),
    };

    if cfg!(debug_assertions) {
        format!("{}{}", target_dir, "/debug/safe")
    } else {
        format!("{}{}", target_dir, "/release/safe")
    }
}

#[allow(dead_code)]
pub fn read_cmd(e: duct::Expression) -> Result<String, String> {
    e.read().map_err(|e| format!("{:#?}", e))
}

#[allow(dead_code)]
pub fn create_preload_and_get_keys(preload: &str) -> Result<(String, String), Error> {
    let pk_command_result = cmd!(
        get_bin_location(),
        "keys",
        "create",
        "--test-coins",
        "---preload",
        preload,
        "--json",
    )
    .read()
    .map_err(|e| Error::Unknown(e.to_string()))?;

    let (xorurl, (_pk, sk)): (String, (String, String)) =
        parse_keys_create_output(&pk_command_result);

    Ok((xorurl, sk))
}

#[allow(dead_code)]
pub fn create_wallet_with_balance(
    preload: &str,
    balance_name: Option<&str>,
) -> Result<(String, String, String), Error> {
    let (_pk, sk) = create_preload_and_get_keys(&preload)?;
    // we spent 1 nano for creating the SafeKey, so we now preload it
    // with 1 nano less than amount request provided
    let preload_nanos = Money::from_str(preload)
        .map_err(|e| Error::Unexpected(e.to_string()))?
        .as_nano();
    let preload_minus_costs = Money::from_nano(preload_nanos - 1).to_string();

    let wallet_create_result = cmd!(
        get_bin_location(),
        "wallet",
        "create",
        "--pay-with",
        &sk,
        "--preload",
        preload_minus_costs,
        "--name",
        balance_name.unwrap_or("default-balance"),
        "--json",
    )
    .read()
    .map_err(|e| Error::Unknown(e.to_string()))?;

    let (wallet_xor, _key_xorurl, key_pair) = parse_wallet_create_output(&wallet_create_result);
    let unwrapped_key_pair =
        key_pair.ok_or_else(|| Error::ContentError("Could not parse wallet".to_string()))?;
    Ok((
        wallet_xor,
        unwrapped_key_pair.public_key().to_string(),
        unwrapped_key_pair
            .secret_key()
            .expect("Error extracting SecretKey from keypair")
            .to_string(),
    ))
}

#[allow(dead_code)]
pub fn create_nrs_link(name: &str, link: &str) -> Result<String, String> {
    let nrs_creation = cmd!(
        get_bin_location(),
        "nrs",
        "create",
        &name,
        "-l",
        &link,
        "--json"
    )
    .read()
    .map_err(|e| Error::Unknown(e.to_string()))?;

    let (nrs_map_xorurl, _change_map) = parse_nrs_create_output(&nrs_creation);
    assert!(nrs_map_xorurl.contains("safe://"));

    Ok(nrs_map_xorurl)
}

#[allow(dead_code)]
pub fn upload_path_with_result(
    path: &str,
    add_trailing_slash: bool,
) -> Result<(String, ProcessedFiles, String), String> {
    let final_path = if add_trailing_slash {
        format!("{}/", path)
    } else {
        path.to_string()
    };

    let args = ["files", "put", &final_path, "--recursive", "--json"];
    let files_container = safe_cmd_stdout(&args, Some(0))?;

    let (container_xorurl, file_map) = parse_files_put_or_sync_output(&files_container);

    Ok((container_xorurl, file_map, final_path))
}

#[allow(dead_code)]
pub fn upload_test_folder_with_result(
    trailing_slash: bool,
) -> Result<(String, ProcessedFiles), String> {
    let d = upload_path_with_result(TEST_FOLDER_NO_TRAILING_SLASH, trailing_slash)?;
    Ok((d.0, d.1))
}

#[allow(dead_code)]
pub fn upload_testfolder_trailing_slash() -> Result<(String, ProcessedFiles), String> {
    upload_test_folder_with_result(true)
}

#[allow(dead_code)]
pub fn upload_testfolder_no_trailing_slash() -> Result<(String, ProcessedFiles), String> {
    upload_test_folder_with_result(false)
}

// keeping for compat with older tests
#[allow(dead_code)]
pub fn upload_test_folder() -> Result<(String, ProcessedFiles), Error> {
    upload_test_folder_with_result(true).map_err(Error::Unknown)
}

#[allow(dead_code)]
pub fn upload_test_symlinks_folder(
    trailing_slash: bool,
) -> Result<(String, ProcessedFiles, String), String> {
    upload_path_with_result(TEST_SYMLINKS_FOLDER, trailing_slash)
}

// Creates a tmp
#[allow(dead_code)]
fn create_tmp_absolute_symlinks_folder() -> Result<(String, String), String> {
    let paths = mk_emptyfolder("abs_symlinks")?;
    let symlinks = Path::new(&paths.1);

    let subdir = symlinks.join("subdir");
    fs::create_dir(&subdir).map_err(|e| format!("{:?}", e))?;

    let dir_link_path = symlinks.join("absolute_link_to_dir");
    create_symlink(&subdir, &dir_link_path, false).map_err(|e| format!("{:?}", e))?;

    let filepath = symlinks.join("file.txt");
    fs::write(&filepath, "Some data").map_err(|e| format!("{:?}", e))?;

    let file_link_path = symlinks.join("absolute_link_to_file.txt");
    create_symlink(&filepath, &file_link_path, false).map_err(|e| format!("{:?}", e))?;

    Ok(paths)
}

#[cfg(unix)]
pub fn create_symlink(target: &Path, link: &Path, _is_dir: bool) -> Result<(), std::io::Error> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
pub fn create_symlink(target: &Path, link: &Path, is_dir: bool) -> Result<(), std::io::Error> {
    if is_dir {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
}

// We create a folder named "emptyfolder" inside a randomly
// named folder in system temp dir, and return both paths.
#[allow(dead_code)]
pub fn mk_emptyfolder(folder_name: &str) -> Result<(String, String), String> {
    let name = get_random_nrs_string();
    let path_random = env::temp_dir().join(name);
    let path_emptyfolder = path_random.join(folder_name);
    fs::create_dir_all(&path_emptyfolder).map_err(|e| format!("{:?}", e))?;
    let empty_folder_path_trailing_slash = format!("{}/", path_emptyfolder.display().to_string());
    Ok((
        path_random.display().to_string(),
        empty_folder_path_trailing_slash,
    ))
}

#[allow(dead_code)]
pub fn create_and_upload_test_absolute_symlinks_folder(
    trailing_slash: bool,
) -> Result<(String, ProcessedFiles, String, String), String> {
    let paths = create_tmp_absolute_symlinks_folder()?;
    let d = upload_path_with_result(&paths.1, trailing_slash)?;
    Ok((d.0, d.1, paths.0, paths.1))
}

// generates a sha3_256 digest/hash of a directory tree.
//
// Note: hidden files or directories are not included.
//  this is necessary for comparing ../testdata with
//  dest dir since `safe files put` presently ignores hidden
//  files.  The hidden files can be included once
//  'safe files put' is fixed to include them.
pub fn sum_tree(path: &str) -> Result<String, Error> {
    let paths = WalkDir::new(path)
        .min_depth(1) // ignore top/root directory
        .follow_links(false)
        .sort_by(|a, b| a.path().cmp(b.path()))
        .into_iter()
        .filter_entry(|e| not_hidden_or_empty(e, 20))
        .filter_map(|v| v.ok());

    let mut digests = String::new();
    for p in paths {
        let relpath = p
            .path()
            .strip_prefix(path)
            .map_err(|e| Error::Unknown(e.to_string()))?
            .display()
            .to_string();
        digests.push_str(&str_to_sha3_256(&relpath));
        if p.path().is_file() {
            digests.push_str(&digest_file(&p.path().display().to_string())?);
        } else if p.path_is_symlink() {
            let target_path =
                fs::read_link(&p.path()).map_err(|e| Error::Unknown(e.to_string()))?;
            digests.push_str(&str_to_sha3_256(&target_path.display().to_string()));
        }
    }
    Ok(str_to_sha3_256(&digests))
}

// callback for WalkDir::new() in sum_tree()
fn not_hidden_or_empty(entry: &DirEntry, max_depth: usize) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| entry.depth() <= max_depth && (entry.depth() == 0 || !s.starts_with('.')))
        .unwrap_or(false)
}

// returns sha3_256 hash of input string as a string.
pub fn str_to_sha3_256(s: &str) -> String {
    let bytes = sha3_256(&s.to_string().into_bytes());
    encode(Base::Base32, bytes)
}

// returns sha3_256 digest/hash of a file as a string.
pub fn digest_file(path: &str) -> Result<String, Error> {
    let data = fs::read_to_string(&path).map_err(|e| Error::Unknown(e.to_string()))?;
    Ok(str_to_sha3_256(&data))
}

#[allow(dead_code)]
pub fn get_random_nrs_string() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(15).collect()
}

#[allow(dead_code)]
pub fn parse_files_container_output(
    output: &str,
) -> (String, BTreeMap<String, BTreeMap<String, String>>) {
    serde_json::from_str(output).expect("Failed to parse output of `safe cat` on FilesContainer")
}

#[allow(dead_code)]
pub fn parse_files_tree_output(output: &str) -> serde_json::Value {
    serde_json::from_str(output).expect("Failed to parse output of `safe tree`")
}

#[allow(dead_code)]
pub fn parse_files_put_or_sync_output(output: &str) -> (String, ProcessedFiles) {
    serde_json::from_str(output).expect("Failed to parse output of `safe files put/sync`")
}

#[allow(dead_code)]
pub fn parse_nrs_create_output(output: &str) -> (String, ProcessedFiles) {
    serde_json::from_str(output).expect("Failed to parse output of `safe nrs create`")
}

#[allow(dead_code)]
pub fn parse_wallet_create_output(output: &str) -> (String, String, Option<Keypair>) {
    serde_json::from_str(&output).expect("Failed to parse output of `safe wallet create`")
}

#[allow(dead_code)]
pub fn parse_cat_wallet_output(output: &str) -> (String, WalletSpendableBalances) {
    serde_json::from_str(output).expect("Failed to parse output of `safe cat wallet`")
}

#[allow(dead_code)]
pub fn parse_xorurl_output(output: &str) -> Vec<(String, String)> {
    serde_json::from_str(output).expect("Failed to parse output of `safe xorurl`")
}

#[allow(dead_code)]
pub fn parse_seq_store_output(output: &str) -> String {
    serde_json::from_str(output).expect("Failed to parse output of `safe seq store`")
}

#[allow(dead_code)]
pub fn parse_cat_seq_output(output: &str) -> (String, Vec<u8>) {
    serde_json::from_str(output).expect("Failed to parse output of `safe cat seq`")
}

#[allow(dead_code)]
pub fn parse_dog_output(output: &str) -> (String, Vec<SafeData>) {
    serde_json::from_str(output).expect("Failed to parse output of `safe dog`")
}

#[allow(dead_code)]
pub fn parse_keys_create_output(output: &str) -> (String, (String, String)) {
    serde_json::from_str(output).expect("Failed to parse output of `safe keys create`")
}

// Executes arbitrary `safe ` commands and returns
// output (stdout, stderr, exit code).
//
// If expect_exit_code is Some, then an Err is returned
// if value does not match process exit code.
#[allow(dead_code)]
pub fn safe_cmd(args: &[&str], expect_exit_code: Option<i32>) -> Result<process::Output, String> {
    println!("Executing: safe {}", args.join(" "));

    let output = duct::cmd(get_bin_location(), args)
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

// Executes arbitrary `safe ` commands and returns
// stdout output
//
// If expect_exit_code is Some, then an Err is returned
// if value does not match process exit code.
#[allow(dead_code)]
pub fn safe_cmd_stdout(args: &[&str], expect_exit_code: Option<i32>) -> Result<String, String> {
    let output = safe_cmd(&args, expect_exit_code)?;
    String::from_utf8(output.stdout).map_err(|_| "Invalid UTF-8".to_string())
}

// Executes arbitrary `safe ` commands and returns
// stderr output
//
// If expect_exit_code is Some, then an Err is returned
// if value does not match process exit code.
pub fn safe_cmd_stderr(args: &[&str], expect_exit_code: Option<i32>) -> Result<String, String> {
    let output = safe_cmd(&args, expect_exit_code)?;
    String::from_utf8(output.stderr).map_err(|_| "Invalid UTF-8".to_string())
}

// returns true if the OS permits writing symlinks
// For windows, this can fail if user does not have
// adequate perms.  The easiest way to find out is just
// to try writing one.
#[cfg(windows)]
pub fn can_write_symlinks() -> bool {
    let name_target = get_random_nrs_string();
    let name_link = get_random_nrs_string();
    let path_link = env::temp_dir().join(name_link);

    let result = std::os::windows::fs::symlink_file(name_target, &path_link);

    if result.is_ok() {
        // it worked, let's cleanup.
        let _r = std::fs::remove_file(&path_link);
    }

    result.is_ok()
}

// returns true if the OS permits writing symlinks
// For unix, this should always be true.
#[cfg(unix)]
pub fn can_write_symlinks() -> bool {
    true
}

// determines if path test_symlinks/file_link is a real symlink.
// This is a proxy to determine if symlinks within symlinks_test
// dir were created properly (eg by git checkout) since this
// will fail on windows without adequate permissions.
pub fn test_symlinks_are_valid() -> Result<bool, String> {
    let result = std::fs::symlink_metadata(TEST_SYMLINK);

    match result {
        Ok(meta) => Ok(meta.file_type().is_symlink()),
        Err(e) => Err(format!("{:?}", e)),
    }
}
