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
    use anyhow::{anyhow, bail, Context, Result};
    use duct::cmd;
    use multibase::{encode, Base};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use sn_api::{
        fetch::SafeData, files::ProcessedFiles, Keypair, SafeUrl,
    };
    use std::{collections::BTreeMap, env, fs, path::Path, process};
    use tiny_keccak::{Hasher, Sha3};
    use walkdir::{DirEntry, WalkDir};

    pub const CLI: &str = "safe";
    pub const SAFE_PROTOCOL: &str = "safe://";

    pub const TEST_FOLDER: &str = "./testdata/";
    pub const TEST_FOLDER_NO_TRAILING_SLASH: &str = "./testdata";
    pub const TEST_SYMLINKS_FOLDER: &str = "./test_symlinks";
    pub const TEST_SYMLINK: &str = "./test_symlinks/file_link";

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

    pub fn create_preload_and_get_keys(preload: &str) -> Result<(String, String)> {
        let pk_command_result = cmd!(
            get_bin_location(),
            "keys",
            "create",
            "--test-coins",
            "---preload",
            preload,
            "--json",
        )
        .read()?;

        let (xorurl, (_pk, sk)): (String, (String, String)) =
            parse_keys_create_output(&pk_command_result);

        Ok((xorurl, sk))
    }

    pub fn create_wallet_with_balance(
        preload: &str,
        balance_name: Option<&str>,
    ) -> Result<(String, String, String)> {
        let (_, sk) = create_preload_and_get_keys(&preload)?;

        let wallet_create_result = cmd!(
            get_bin_location(),
            "wallet",
            "create",
            "--pay-with",
            &sk,
            "--preload",
            preload,
            "--name",
            balance_name.unwrap_or("default-balance"),
            "--json",
        )
        .read()?;

        let (wallet_xor, _key_xorurl, key_pair) = parse_wallet_create_output(&wallet_create_result);
        let unwrapped_key_pair =
            key_pair.ok_or_else(|| anyhow!("Could not parse wallet".to_string()))?;

        Ok((
            wallet_xor,
            unwrapped_key_pair.public_key().to_string(),
            unwrapped_key_pair
                .secret_key()
                .context("Error extracting SecretKey from keypair")?
                .to_string(),
        ))
    }

    pub fn create_nrs_link(name: &str, link: &str) -> Result<String> {
        let nrs_creation = cmd!(
            get_bin_location(),
            "nrs",
            "create",
            &name,
            "-l",
            &link,
            "--json"
        )
        .read()?;

        let (nrs_map_xorurl, _change_map) = parse_nrs_create_output(&nrs_creation);
        assert!(nrs_map_xorurl.contains("safe://"));

        Ok(nrs_map_xorurl)
    }

    pub fn upload_path(
        path: &str,
        add_trailing_slash: bool,
    ) -> Result<(String, ProcessedFiles, String)> {
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

    // Creates a tmp
    fn create_tmp_absolute_symlinks_folder() -> Result<(String, String)> {
        let paths = mk_emptyfolder("abs_symlinks")?;
        let symlinks = Path::new(&paths.1);

        let subdir = symlinks.join("subdir");
        fs::create_dir(&subdir)
            .context(format!("Failed to create directory: {}", subdir.display()))?;

        let dir_link_path = symlinks.join("absolute_link_to_dir");
        create_symlink(&subdir, &dir_link_path, false).context(format!(
            "Failed to create symlink '{}' to: {}",
            dir_link_path.display(),
            subdir.display()
        ))?;

        let filepath = symlinks.join("file.txt");
        fs::write(&filepath, "Some data").context(format!(
            "Failed to write to file at: {}",
            filepath.display()
        ))?;

        let file_link_path = symlinks.join("absolute_link_to_file.txt");
        create_symlink(&filepath, &file_link_path, false).context(format!(
            "Failed to create symlink '{}' to: {}",
            file_link_path.display(),
            filepath.display()
        ))?;

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
    pub fn mk_emptyfolder(folder_name: &str) -> Result<(String, String)> {
        let name = get_random_nrs_string();
        let path_random = env::temp_dir().join(name);
        let path_emptyfolder = path_random.join(folder_name);
        fs::create_dir_all(&path_emptyfolder).context(format!(
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

    pub fn create_and_upload_test_absolute_symlinks_folder(
        trailing_slash: bool,
    ) -> Result<(String, ProcessedFiles, String, String)> {
        let paths = create_tmp_absolute_symlinks_folder()?;
        let d = upload_path(&paths.1, trailing_slash)?;
        Ok((d.0, d.1, paths.0, paths.1))
    }

    // generates a sha3_256 digest/hash of a directory tree.
    //
    // Note: hidden files or directories are not included.
    //  this is necessary for comparing ./testdata with
    //  dest dir since `safe files put` presently ignores hidden
    //  files.  The hidden files can be included once
    //  'safe files put' is fixed to include them.
    pub fn sum_tree(path: &str) -> Result<String> {
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
                .context(format!(
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
                let target_path = fs::read_link(&p.path()).context(format!(
                    "Failed to follow link from file at: {}",
                    p.path().display()
                ))?;
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
        let s_bytes = s.as_bytes();
        let mut hasher = Sha3::v256();
        let mut bytes = [0; 32];
        hasher.update(&s_bytes);
        hasher.finalize(&mut bytes);
        encode(Base::Base32, bytes)
    }

    // returns sha3_256 digest/hash of a file as a string.
    pub fn digest_file(path: &str) -> Result<String> {
        let data = fs::read_to_string(&path)
            .context(format!("Failed to read string from file at: {}", path))?;
        Ok(str_to_sha3_256(&data))
    }

    pub fn get_random_nrs_string() -> String {
        thread_rng().sample_iter(&Alphanumeric).take(15).collect()
    }

    pub fn safeurl_from(url: &str) -> Result<SafeUrl> {
        SafeUrl::from_url(url).map_err(|e| anyhow!("Failed to parse URL: {}", e))
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
        serde_json::from_str(&output).expect("Failed to parse output of `safe wallet create`")
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

    // Executes arbitrary `safe ` commands and returns
    // output (stdout, stderr, exit code).
    //
    // If expect_exit_code is Some, then an Err is returned
    // if value does not match process exit code.
    pub fn safe_cmd(args: &[&str], expect_exit_code: Option<i32>) -> Result<process::Output> {
        println!("Executing: safe {}", args.join(" "));

        let output = duct::cmd(get_bin_location(), args)
            .stdout_capture()
            .stderr_capture()
            .unchecked()
            .run()
            .with_context(|| {
                format!("Failed to run 'safe' command with args: {}", args.join(" "))
            })?;

        if let Some(ec) = expect_exit_code {
            match output.status.code() {
                Some(code) => assert_eq!(ec, code),
                None => bail!("Command returned no exit code".to_string()),
            }
        }
        Ok(output)
    }

    // Executes arbitrary `safe ` commands and returns
    // stdout output
    //
    // If expect_exit_code is Some, then an Err is returned
    // if value does not match process exit code.
    pub fn safe_cmd_stdout(args: &[&str], expect_exit_code: Option<i32>) -> Result<String> {
        let output = safe_cmd(&args, expect_exit_code)?;
        String::from_utf8(output.stdout)
            .context("Failed to parse the error output as a UTF-8 string".to_string())
    }

    // Executes arbitrary `safe ` commands and returns
    // stderr output
    //
    // If expect_exit_code is Some, then an Err is returned
    // if value does not match process exit code.
    pub fn safe_cmd_stderr(args: &[&str], expect_exit_code: Option<i32>) -> Result<String> {
        let output = safe_cmd(&args, expect_exit_code)?;
        String::from_utf8(output.stderr)
            .context("Failed to parse the error output as a UTF-8 string".to_string())
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
            Err(e) => Err(anyhow!("{:?}", e)),
        }
    }
}
