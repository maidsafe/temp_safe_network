// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#[allow(dead_code)]
pub mod util {
    use assert_cmd::Command;
    use color_eyre::{eyre::eyre, eyre::WrapErr, Help, Result};
    use multibase::{encode, Base};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use sn_api::{fetch::SafeData, files::ProcessedFiles, Keypair, Url};
    use std::{collections::BTreeMap, env, fs, path::Path, process};
    use tiny_keccak::{Hasher, Sha3};
    use walkdir::WalkDir;

    pub const CLI: &str = "safe";
    pub const SAFE_PROTOCOL: &str = "safe://";

    pub const TEST_FOLDER: &str = "./testdata/";
    pub const TEST_FOLDER_NO_TRAILING_SLASH: &str = "./testdata";
    pub const TEST_SYMLINKS_FOLDER: &str = "./test_symlinks";
    pub const TEST_SYMLINK: &str = "./test_symlinks/file_link";

    #[ctor::ctor]
    fn init() {
        let _ = color_eyre::install();
    }

    pub fn get_directory_len(directory_path: impl AsRef<Path>) -> Result<u64> {
        fs::read_dir(directory_path.as_ref())
            .wrap_err(format!(
                "Error reading directory at {}",
                directory_path.as_ref().display()
            ))?
            .map(|entry| get_file_len(entry?.path()))
            .sum()
    }

    pub fn get_file_len(path: impl AsRef<Path>) -> Result<u64> {
        let metadata = std::fs::metadata(&path)
            .wrap_err(format!(
                "Cannot retrieve metadata for: {}",
                &path.as_ref().display()
            ))
            .suggestion(format!(
                "Verify that {} exists and that the user has read permissions on it",
                &path.as_ref().display()
            ))?;
        Ok(metadata.len())
    }

    pub fn get_bin_location() -> String {
        let target_dir = match env::var("CARGO_TARGET_DIR") {
            Ok(target_dir) => target_dir,
            Err(_) => "target".to_string(),
        };

        if cfg!(debug_assertions) {
            format!("{}{}", target_dir, "/debug/safe")
        } else {
            format!("{}{}", target_dir, "/release/safe")
        }
    }

    pub fn create_and_get_keys() -> Result<(String, String)> {
        let pk_command_result =
            safe_cmd_stdout(["keys", "create", "--test-coins", "--json"], Some(0))?;

        let (xorurl, (_pk, sk)): (String, (String, String)) =
            parse_keys_create_output(&pk_command_result);

        Ok((xorurl, sk))
    }

    pub fn create_nrs_link(name: &str, link: &str) -> Result<String> {
        let nrs_creation = safe_cmd_stdout(["nrs", "create", name, "-l", link, "--json"], Some(0))?;

        let (nrs_map_xorurl, _change_map) = parse_nrs_create_output(&nrs_creation);
        assert!(nrs_map_xorurl.contains("safe://"));

        Ok(nrs_map_xorurl)
    }

    pub fn upload_path(
        path: impl AsRef<Path>,
        add_trailing_slash: bool,
    ) -> Result<(String, ProcessedFiles, String)> {
        let final_path = if add_trailing_slash {
            format!("{}/", path.as_ref().display())
        } else {
            String::from(path.as_ref().to_str().unwrap())
        };

        let files_container = safe_cmd_stdout(
            ["files", "put", &final_path, "--recursive", "--json"],
            Some(0),
        )?;
        let (container_xorurl, file_map) = parse_files_put_or_sync_output(&files_container);
        Ok((container_xorurl, file_map, final_path))
    }

    pub fn upload_test_folder(trailing_slash: bool) -> Result<(String, ProcessedFiles)> {
        let d = upload_path(TEST_FOLDER_NO_TRAILING_SLASH, trailing_slash)?;
        Ok((d.0, d.1))
    }

    pub fn upload_testfolder_trailing_slash() -> Result<(String, ProcessedFiles)> {
        upload_test_folder(true)
    }

    pub fn upload_testfolder_no_trailing_slash() -> Result<(String, ProcessedFiles)> {
        upload_test_folder(false)
    }

    pub fn upload_test_symlinks_folder(
        trailing_slash: bool,
    ) -> Result<(String, ProcessedFiles, String)> {
        upload_path(TEST_SYMLINKS_FOLDER, trailing_slash)
    }

    /// Creates a temporary directory at the specified path and populates it with absolute
    /// symlinks.
    ///
    /// The purpose of passing in the directory is to allow the test to have control over where the
    /// temporary directory is created.
    pub fn create_absolute_symlinks_directory(path: impl AsRef<Path>) -> Result<()> {
        let symlinks = path.as_ref();

        let subdir = symlinks.join("subdir");
        fs::create_dir(&subdir)
            .wrap_err(format!("Failed to create directory: {}", subdir.display()))?;

        let dir_link_path = symlinks.join("absolute_link_to_dir");
        create_symlink(&subdir, &dir_link_path, false).wrap_err(format!(
            "Failed to create symlink '{}' to: {}",
            dir_link_path.display(),
            subdir.display()
        ))?;

        let filepath = symlinks.join("file.txt");
        fs::write(&filepath, "Some data").wrap_err(format!(
            "Failed to write to file at: {}",
            filepath.display()
        ))?;

        let file_link_path = symlinks.join("absolute_link_to_file.txt");
        create_symlink(&filepath, &file_link_path, false).wrap_err(format!(
            "Failed to create symlink '{}' to: {}",
            file_link_path.display(),
            filepath.display()
        ))?;

        Ok(())
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
    pub fn mk_emptyfolder(folder_name: &str) -> Result<(String, String)> {
        let name = get_random_nrs_string();
        let path_random = env::temp_dir().join(name);
        let path_emptyfolder = path_random.join(folder_name);
        fs::create_dir_all(&path_emptyfolder).wrap_err(format!(
            "Failed to create path: {}",
            path_emptyfolder.display()
        ))?;
        let empty_folder_path_trailing_slash =
            format!("{}/", path_emptyfolder.display().to_string());
        Ok((
            path_random.display().to_string(),
            empty_folder_path_trailing_slash,
        ))
    }

    /// Generates a sha3_256 digest/hash of a directory tree, for the purposes of comparing it to
    /// another tree.
    ///
    /// This function was originally written to ignore 'hidden' files, because safe didn't upload
    /// those. Safe does upload these now, so it's been modified to take that into account. We also
    /// need this modification because `assert_fs` prefixes the directories it creates with '.', so
    /// it was skipping those.
    pub fn sum_tree(path: &str) -> Result<String> {
        let paths = WalkDir::new(path)
            .follow_links(false)
            .sort_by(|a, b| a.path().cmp(b.path()))
            .into_iter()
            .filter_map(|v| v.ok());

        let mut digests = String::new();
        for p in paths {
            let relpath = p
                .path()
                .strip_prefix(path)
                .wrap_err(format!(
                    "Failed to strip prefix '{}' to '{}'",
                    path,
                    p.path().display()
                ))?
                .display()
                .to_string();
            digests.push_str(&str_to_sha3_256(&relpath));
            if p.path().is_file() {
                digests.push_str(&digest_file(&p.path().display().to_string())?);
            } else if p.path_is_symlink() {
                let target_path = fs::read_link(&p.path()).wrap_err(format!(
                    "Failed to follow link from file at: {}",
                    p.path().display()
                ))?;
                digests.push_str(&str_to_sha3_256(&target_path.display().to_string()));
            }
        }
        Ok(str_to_sha3_256(&digests))
    }

    // returns sha3_256 hash of input string as a string.
    pub fn str_to_sha3_256(s: &str) -> String {
        let s_bytes = s.as_bytes();
        let mut hasher = Sha3::v256();
        let mut bytes = [0; 32];
        hasher.update(s_bytes);
        hasher.finalize(&mut bytes);
        encode(Base::Base32, bytes)
    }

    // returns sha3_256 digest/hash of a file as a string.
    pub fn digest_file(path: &str) -> Result<String> {
        let data = fs::read_to_string(&path)
            .wrap_err(format!("Failed to read string from file at: {}", path))?;
        Ok(str_to_sha3_256(&data))
    }

    pub fn get_random_nrs_string() -> String {
        thread_rng().sample_iter(&Alphanumeric).take(15).collect()
    }

    pub fn safeurl_from(url: &str) -> Result<Url> {
        Url::from_url(url).map_err(|e| eyre!("Failed to parse URL: {}", e))
    }

    pub fn parse_files_container_output(
        output: &str,
    ) -> (String, BTreeMap<String, BTreeMap<String, String>>) {
        serde_json::from_str(output)
            .expect("Failed to parse output of `safe cat` on FilesContainer")
    }

    pub fn parse_files_tree_output(output: &str) -> serde_json::Value {
        serde_json::from_str(output).expect("Failed to parse output of `safe tree`")
    }

    pub fn parse_files_put_or_sync_output(output: &str) -> (String, ProcessedFiles) {
        serde_json::from_str(output).expect("Failed to parse output of `safe files put/sync`")
    }

    pub fn parse_nrs_create_output(output: &str) -> (String, ProcessedFiles) {
        serde_json::from_str(output).expect("Failed to parse output of `safe nrs create`")
    }

    pub fn parse_wallet_create_output(output: &str) -> (String, String, Option<Keypair>) {
        serde_json::from_str(output).expect("Failed to parse output of `safe wallet create`")
    }

    pub fn parse_xorurl_output(output: &str) -> Vec<(String, String)> {
        serde_json::from_str(output).expect("Failed to parse output of `safe xorurl`")
    }

    pub fn parse_seq_store_output(output: &str) -> String {
        serde_json::from_str(output).expect("Failed to parse output of `safe seq store`")
    }

    pub fn parse_cat_seq_output(output: &str) -> (String, Vec<u8>) {
        serde_json::from_str(output).expect("Failed to parse output of `safe cat seq`")
    }

    pub fn parse_dog_output(output: &str) -> (String, Vec<SafeData>) {
        serde_json::from_str(output).expect("Failed to parse output of `safe dog`")
    }

    pub fn parse_keys_create_output(output: &str) -> (String, (String, String)) {
        serde_json::from_str(output).expect("Failed to parse output of `safe keys create`")
    }

    /// Runs safe with the arguments specified, with the option to assert on the exit code.
    ///
    /// This was changed to use the assert_cmd crate because the newer version of this crate
    /// provides *both* the stdout and stderr if the process doesn't exit as expected. This is
    /// extremely useful in this test suite because there are lots of commands used to setup the
    /// context for the tests, and you need to be able to see why those fail too.
    pub fn safe_cmd<'a>(
        args: impl IntoIterator<Item = &'a str>,
        expect_exit_code: Option<i32>,
    ) -> Result<process::Output> {
        safe_cmd_at(args, env::current_dir()?, expect_exit_code)
    }

    pub fn safe_cmd_at<'a>(
        args: impl IntoIterator<Item = &'a str>,
        working_directory: impl AsRef<Path>,
        expect_exit_code: Option<i32>,
    ) -> Result<process::Output> {
        let args: Vec<&str> = args.into_iter().collect();
        println!("Executing: safe {}", args.join(" "));
        let code = expect_exit_code.unwrap_or(0);
        let mut cmd = Command::cargo_bin("safe")?;
        Ok(cmd
            .args(args)
            .current_dir(working_directory)
            .assert()
            .code(code)
            .get_output()
            .to_owned())
    }

    // Executes arbitrary `safe ` commands and returns
    // stdout output
    //
    // If expect_exit_code is Some, then an Err is returned
    // if value does not match process exit code.
    pub fn safe_cmd_stdout<'a>(
        args: impl IntoIterator<Item = &'a str>,
        expect_exit_code: Option<i32>,
    ) -> Result<String> {
        let output = safe_cmd(args, expect_exit_code)?;
        let stdout = String::from_utf8(output.stdout)
            .wrap_err("Failed to parse the error output as a UTF-8 string".to_string())?;
        Ok(stdout.trim().to_string())
    }

    // Executes arbitrary `safe ` commands and returns
    // stderr output
    //
    // If expect_exit_code is Some, then an Err is returned
    // if value does not match process exit code.
    pub fn safe_cmd_stderr<'a>(
        args: impl IntoIterator<Item = &'a str>,
        expect_exit_code: Option<i32>,
    ) -> Result<String> {
        let output = safe_cmd(args, expect_exit_code)?;
        String::from_utf8(output.stderr)
            .wrap_err("Failed to parse the error output as a UTF-8 string".to_string())
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
    pub fn test_symlinks_are_valid() -> Result<bool> {
        let result = std::fs::symlink_metadata(TEST_SYMLINK);

        match result {
            Ok(meta) => Ok(meta.file_type().is_symlink()),
            Err(e) => Err(eyre!("{:?}", e)),
        }
    }
}
