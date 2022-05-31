// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#[allow(dead_code)]
pub mod util {
    use assert_cmd::Command;
    use color_eyre::{eyre::eyre, eyre::WrapErr, Help, Result};
    use multibase::{encode, Base};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use sn_api::{files::ProcessedFiles, resolver::SafeData, SafeUrl};
    use std::path::Path;
    use std::{collections::BTreeMap, env, fs, process};
    use tiny_keccak::{Hasher, Sha3};
    use walkdir::WalkDir;

    const GITHUB_API_URL: &str = "https://api.github.com";
    pub const CLI: &str = "safe";
    pub const SAFE_PROTOCOL: &str = "safe://";

    pub const TEST_FOLDER: &str = "../resources/testdata/";
    pub const TEST_FOLDER_NO_TRAILING_SLASH: &str = "../resources/testdata";
    pub const TEST_SYMLINKS_FOLDER: &str = "../resources/test_symlinks";
    pub const TEST_SYMLINK: &str = "../resources/test_symlinks/file_link";

    pub const DBC_WITH_12_230_000_000: &str = "2fce3f70d7ad38d48d81f3f67c11ea10388a95efff66ce444f3274fb261c119a839e3de7a4c97b3b2b12fde3af09990ed20f48b3a23d79ca9b39cf0b6f492b142c03d3c0ee34d087fa3b9989cab7f7fb4e2328587330ce226277b8062881b3a231b11365712ad3a82ece13af3699fb171f16001af12522a92d1cf6d788a919bf7aa7bc1cd70f0eac4f96967bdd9cb38338aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e1870000000000000001c428455eb4b5485f9465049a3a1b519a89b50ec3bef0c4ebbf45b7198cc348321ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b00000000000000016caf52b6919fa145146588809f6c0d2a0b70053c9501a52730bbfba00de94dfe72f14801d29d5b2d93c8b626d443498d34bf24d89d5012a6067ccb301919272553dcdf50cbf95240e47a3c7e967ba98e70b2a97e866c810a0ba806bb96a1b8308eb9b22b29b44ea5d33ae9e9c057261b09e21456123981952224dbf9cadfd50cfb02e0dc57ce55156f844c3a2c704a5be432d2444765dd15beb7fca57a20508f73df1043f8e2e5d8a56a2fb17fe475f2cca88c8d23eae27f3d172e3d6fad45c1b3232dc10ee46a7016df417935865a99b3761bffdefea6373584f50258ab9d60cddfc80ed76c9454bcaa8083f58e6c53d3095e06b02638eccd668031cf232b87ea9474b4166849a34ffbc805310520c8b19bafa7424df714becb9ce02d6d705b51b168a0b806798da550a0807fc1a8a9122908dbbd7e4838e65a9c9bdbc1fbf3e11a79187444274a828618e4debbcc3cee1ea0d1a157f21b8624bccc8eb58c91a60d84deb16ac42241ace0f3174c3abbca0d3331990e7c6db07ff24c8537060ea76348f96af31927c49c84a4e2c4cb8cdd95f86c0fcc18cb1d6c2a519ced3ad2e9903061dd3f3c8e02fafd969f53f52274faddb0b4f5aef19dc0e62277cbcda3e3ac39cfa4b709efbbdab65440a42e628fff051c83acd583a5e3bbfb2eb4807c5aa2303c7b52f1d539899572b674bd8fb4a529354e8087067a4e374e92b855a7ec523295867f310f7cea5d48bb0295a1c3ea47b6c4229f000d0c83ae22eb449656c25083a5fc99f99a23cf25fc2020941ffb392b3c8788c25b687b21610d35e3a5f12974b20778144d6930772381a694be5605f21142ca2121a7358e377fd20d50ff00ae3550aa2622ac8d8a642c41aec117b74c22732b297dbb4e4637595bb5b7d5a69414c7822e55efb9c40ddbc2a293532ee49c705e9bb2f3fff3f2dcce8c72b90c486b3a750ee594fd2b04c2dd9553e02a226001956bed24790349d243ca5d51167c4a348d19e42bf2cabc72c0510b8217e0ce5997b763fd11a881c8757f565e3ca3766fc5c3e011ad40d634666859a3a5fdbab6fb627c5abd1d80894424c5a1b0de74cfdf5eb17b9c8bb873f4515adc3e9d62d76f4f5bbb0ef453e369ad5ffa853ab6661f2bf242d91d098f5c59ceb4c1151b47f4cd5ee9dc98daca45898168f4ff69fb230d01d7660d0718cc1b6facbb7f8e9a73cb6a8b93aba7dddc450628588fbb916641abaab4a228bc2584950e34f86a8154b4f5773d07a4ad70b9c2fdff3d16933011664e8de0298c61cd2da7511b8a0cd63acf47156bd7f667929fce5b7257582f336683abf609ad7a4b9dca4395849d81da230d13822d01224d49f4d249d9881d2e876335824514448300000000000003a00af4916a22e0177947732d2d58db7e0325d004f1de5718b3275ae009845fa53ce6948ddd9623e445c8433d30c162c9a22b05a80d417ac2efc2bf684e091bc26a81bda1ec9253ed4657c2440710623741007bdb90e9d94121babea7484d25a49432e71b55250be2a5c8ac7716ec2c61218cca98458ae648e11d31c185d20e1eb21300e7b360f1553a128ff97d54c1510a26178421859bef1dae17e4cf496acc39b20059e48b5c94c7ab244dbfc368d12868af7cac467d8c18e8cb5b60c22b7e9151e013dd28b813254e65a73c24c0efae47b29cdc3a34286dabb6fd09d9ad8b121655394d272934013773667eae726cdbab0b42cfc2c0bd988ecacafead6f179046fa3a4f5fb8ca7a790e5685f262c0e770c20a700fdab00aeac3a8634d84102f750996b5323356ad1b3199702c2d23a64279726f35e0eb630f91794dda905093995d6b457782792564be257a191f2bdeedd01e76051f7aa740735b8c408c618b3b6edac93f7504d441855d89adb6e13ed2a60c81ac0a0b445a2d1447bc30f0f30376ab55995ad0b639c1dfddb3e29e8f8ddccb59e55d863117ca6bc1c8dbf42983ea889cd03b19ab7c76addddb6dbdd53856b92df8cc929f3d37eb27ba6ab68194fc55f9ca2943acd265538751335cdbf527675494d176282e59a9c8deb95645b5b2b8660456434701b182e08915dbb6de5bfacc5480e867a9f08c8fe2d209eaa44175912ce5fb782425e264587d8ad13a70eb83d0ac0b02095a8f6d029d588b80b4f7a96c73720344247eae9613a9ff6ca368dbd823c225304725a36aee0c1822fe554733bc074c50ea296ff6b60c8711f7ef216e1ff1b532b398ba13e6da8ebd4546dcf296a4c1f7b27da5ec6e771d23afc139af51d5e9b5f2be9f32a7ab949e86cb5158c0f61b5a31f81e9c9ddc8412fcdf971a7cc296e742bcae1d444928152841a927dd7985bb7ef06c1fcfa9a1b908e6b0f52a38321f325f4bff3af4064ce8266c93821bb81f9a0404ffff81520ff370ba1b831cae5db58cb943e59dac65a9092db9fb12b00424c49dc0b92356a5b2801c40b43f8d5262278b44eb980e5857397b6ec9570dde9ca80b5ab041bf81872f3f999182589771a75c8ef0e3b8603888dade9b1d449adaac0f08c7cddbe85df0871722ded236de4b7e5f999ad3f2c8e01cab184eaec5f2fa35b22586e7838e4dde010a37ff24927c8a4845794ee699ab54b57ba50e038008af5ea0bbaf3f4e627fe03809f973980b81ced08a602b7458158ab7684376535265761dc039bb13d01ff90e732b8df17d783338db830193034d48dca8ed42b077293db0aa1da1b14c0c4c0e76c7859b832a226464e2cad01aa41bf8fd1686933b55fd961ab336cdb697e851bf3b6c1142019475e0b2925d6727e15de30cbf7b342daf509778d2af676c53a3e962ddff846c310d768e00000000000003a058998e2c08c59208820948b33b051910a3156d3beff7984798c030f971904a9085c61447ded4a76575b5fcc40b5acd93b1d0b17ba74231d94f3df0f666e751067fdc9364a3a5d07ce0881d44fabf4cf0f478a2ec3d14a668e3e5f249d57138890a5db2846b458f82e77256f2f4b3f1412b0434831aad2b96fed8c9ed2e520a5c54ddfb9f72ee0932eea6ab7a58b2f14c5ef062717e9d1da9c7f213dd11bed65bd7693398c732e08b6fe96095c68e4d7e2d3378c3632e736358e88db56bad7329bfd83bac50c0649fcf4f8baf1f95a3b16a004a02b8d0a469b687ee7f91743232a46ae0eb27d0f45d18dcbc04aeb9595995802a4ddf42f2712248d38d5b7656916211ecc6bbfb4c62b6f9d3393061ead919662e53f356bf2327536dccedd7eee28b06514de978fcc965a3d6d507aba3b3af78e3ac52e3141e9fc1b51302616138f2e14ba340d1baea0b2d3d65e21fc94545f17e956292b4cfa8eaecdfa46344a88eb15f9e230a384caefaced24f6553f5ba7c7e41f9c9bbec71a9a5d8a33b994b6d1ce62a1334351057838c2fbe0aa3a204c17fb11e1970f36a8ef3a16d634890a98dee795c197ad586a5c0cf5a23e72d5931481ddf33a342c5bcc320965b98b46bfc2c51a106ce2dedc8f7196b1af0f30825854d5a34d7fc398aea4548e13d0289fe76ec71f4d9531e56d44d39562ca349fd9b62321e28e7d1c1c406199b5dd4841e12d1c649e9e3dd7db84c8499f61621c188fcf56da4915001bc82a8b8aaafa7001486df8dc215d02261640dfce04c79ba5d66ea5daec4fe8a7f2e39527c8c5ca8d8e05fe00e9f909d49a852a9c2ab96f185aa63ccd5e542b0355d8ee66b252a5a57ab0b30c704ea8055eea246d5700c0aadf23520a445e70b331b24fb8f8c43739403cba21d87a6a63e05cddfcf6934230c4f9420b796b70df0eb70f0c1a94c09c70b7e1bb6e2494fe4ae028c358a9048404e31e6191749b83f8cc942fc796459ce1669ea7c268d3c55cb2be9e8ed54400cd831c974fe59d63b043963e98330c7179038c3be277bcb8d9713234a48a7f2e84bdc41c6f822e7292a02407c9966834fc014727d1aa86ca08f39d5fb53b745bb16d2eb11f23af7955b1a5507ff670dc39d8bd929a2ac8ad2e72ed27d36c386223672d434474f9149d39079bd72977f573bee8686584eddcb1467c24c16a38b0e6f6a9c727f7cca2b7a7a86589fa507438ee0bdaa383ddb95ac2d3abe8d07fe06dc4980bbf6423d7496d4283f85cfc9dc79091eb8695d583ebf31a6aad857a6510e3bf268b634ec62ee7112be92a89633f394792332d8e0f5fcff45353bf8e55db172dae5d60dacd372c1e47f9ce8d497f7a18438fe6a3930c240b29ab1d91176ebd5934d23230b629f46f294f9926d4f33c5ba2c1cee78df2b3bb2d43c67423751cfd78ec652596dec00a06db900000000000003a0d28d2cf0c4c23a0ae40ab3a4a29ed7eaa11a3957bf5ee7b8e61c4f3858982f5fe8f40bb19cf728f76121c2cd51ab9a970000000000000003e8c9885501863a2462cf63cd2d4109746aeda1643d286fd6fb13014bef689dc66b34b6e5a6f83e88aead415c06737aaee5eb3c7fda86f635b3345c72329088cbae5eb375e33f880727993c4c01b736f93fc1520f3fa1b41a8c8eb7002dd6a1a996535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe8800000000000000011ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b158a686df05f9cb8c9f01f6499b27d21c499d31ad7d8274c4dff49192e73c184018a600ba8fc0e3734dd643706ae2af8922fffa4574a630845a4a91f0ce23518000000000000000125cf08c3ee49b669e5cfb0ad13de8e19691287addf6b5228a78fc769ce76c74e00000000000000018f4ba55337b209afca288eb85bfdd9d8919f20b31022d399f34a0170e4163a86dacac52a77e655366987ffdd726dd50bd51f2cc1b554fe2445b95cc78d176ac62f11e3c39cfb0fc1b3d02ad3b255eaba46d5d8587fe0faa7aae3e7d07aaabb9964eecbcba01e1dfcba56deb73c61fab162bdfbb3ce4bce16baa2e01c437b91dfc8213a62da4a4f5e00000000000000282697644c1845c35353295cdef403482ed7d11a8cab4e1d97817f61145a835c17f219c5ead6e1eec40b8fb085d256dd8020d97692739067230e15649334dbbc63bdfcc2d65a435ab30aadaee4494990ffddf25f5cabf451b691376deab52ade138932ac3b1fcab84cd6b9a6d88ee1d044bf495c5d1d08b3a448c5037d88d055dc481d2832a00c1d12349b1ed6ab62b6b718db76ee5066add05240cd94adb26c9afd87e674ddbc9f8d8578449cb7d6435f00000000000000204bcea50da17574d7dca87d2b13ccdd5754394d6d073c2ddfaee89a9a57b44e87efe52ff2a09ccdc8fea40d71af8be598d2c07cbc4a634060bbe2c11bcb537089130b30325c57b23492d9c0263fd01e5800000000";

    #[ctor::ctor]
    fn init() {
        let _ = color_eyre::install();
    }

    pub fn get_sn_node_latest_released_version() -> Result<String> {
        let latest_release_url = format!(
            "{}/repos/maidsafe/safe_network/releases/latest",
            GITHUB_API_URL
        );
        let response = reqwest::blocking::Client::new()
            .get(latest_release_url)
            .header(reqwest::header::USER_AGENT, "sn_cmd_test_utilities")
            .header(reqwest::header::ACCEPT, "application/vnd.github.v3+json")
            .send()?;
        let response_json = response.json::<serde_json::Value>()?;
        let tag_name = response_json["tag_name"].as_str().ok_or_else(|| {
            eyre!(format!(
                "Failed to parse the tag_name field from the response: {}",
                response_json
            ))
        })?;
        let version = get_version_from_release_version(tag_name)?;
        Ok(version)
    }

    pub fn get_directory_file_count(directory_path: impl AsRef<Path>) -> Result<usize> {
        let paths = WalkDir::new(directory_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|v| v.ok());
        // The `directory_path` itself is returned by walkdir. We're only interested in how many
        // entries are *inside* `directory_path`.
        Ok(paths.count() - 1)
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
        let pk_cmd_result = safe_cmd_stdout(["keys", "create", "--json"], Some(0))?;

        let (xorurl, (_pk, sk)): (SafeUrl, (String, String)) =
            parse_keys_create_output(&pk_cmd_result)?;

        Ok((xorurl.to_string(), sk))
    }

    pub fn create_nrs_link(name: &str, link: &str) -> Result<SafeUrl> {
        let nrs_creation = safe_cmd_stdout(["nrs", "create", name, "-l", link, "--json"], Some(0))?;
        let (_, nrs_map_xorurl, _change_map) = parse_nrs_register_output(&nrs_creation)?;
        Ok(nrs_map_xorurl)
    }

    pub fn upload_path(
        path: impl AsRef<Path>,
        add_trailing_slash: bool,
    ) -> Result<(String, ProcessedFiles, String)> {
        let final_path = if add_trailing_slash {
            format!("{}/", path.as_ref().display())
        } else {
            format!("{}", path.as_ref().display())
        };

        let path = Path::new(&final_path);
        let files_container = if path.is_dir() {
            safe_cmd_stdout(
                ["files", "put", &final_path, "--recursive", "--json"],
                Some(0),
            )?
        } else {
            safe_cmd_stdout(["files", "put", &final_path, "--json"], Some(0))?
        };
        let (container_xorurl, file_map) = parse_files_put_or_sync_output(&files_container)?;
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
        let empty_folder_path_trailing_slash = format!("{}/", path_emptyfolder.display());
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
        encode(Base::Base32Z, bytes)
    }

    // returns sha3_256 digest/hash of a file as a string.
    pub fn digest_file(path: &str) -> Result<String> {
        let data = fs::read_to_string(&path)
            .wrap_err(format!("Failed to read string from file at: {}", path))?;
        Ok(str_to_sha3_256(&data))
    }

    pub fn get_random_nrs_string() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect()
    }

    pub fn safeurl_from(url: &str) -> Result<SafeUrl> {
        SafeUrl::from_url(url).map_err(|e| eyre!("Failed to parse URL: {}", e))
    }

    #[allow(clippy::type_complexity)]
    pub fn parse_files_container_output(
        output: &str,
    ) -> Result<(String, BTreeMap<String, BTreeMap<String, String>>)> {
        serde_json::from_str(output).map_err(|_| {
            eyre!(
                "Failed to parse output of `safe cat` on FilesContainer: {}",
                output
            )
        })
    }

    pub fn parse_files_tree_output(output: &str) -> Result<serde_json::Value> {
        serde_json::from_str(output)
            .map_err(|_| eyre!("Failed to parse output of `safe tree`: {}", output))
    }

    pub fn parse_files_put_or_sync_output(output: &str) -> Result<(String, ProcessedFiles)> {
        serde_json::from_str(output).map_err(|_| {
            eyre!(
                "Failed to parse output of `safe files put/sync`: {}",
                output
            )
        })
    }

    pub fn parse_nrs_register_output(
        output: &str,
    ) -> Result<(String, SafeUrl, (String, String, String))> {
        serde_json::from_str(output)
            .map_err(|_| eyre!("Failed to parse output of `safe nrs register`: {}", output))
    }

    pub fn parse_wallet_create_output(output: &str) -> Result<String> {
        serde_json::from_str(output)
            .map_err(|_| eyre!("Failed to parse output of `safe wallet create`: {}", output))
    }

    pub fn parse_xorurl_output(output: &str) -> Result<Vec<(String, String)>> {
        serde_json::from_str(output)
            .map_err(|_| eyre!("Failed to parse output of `safe xorurl`: {}", output))
    }

    pub fn parse_dog_output(output: &str) -> Result<(String, Vec<SafeData>)> {
        serde_json::from_str(output)
            .map_err(|_| eyre!("Failed to parse output of `safe dog`: {}", output))
    }

    pub fn parse_keys_create_output(output: &str) -> Result<(SafeUrl, (String, String))> {
        serde_json::from_str(output)
            .map_err(|_| eyre!("Failed to parse output of `safe keys create`: {}", output))
    }

    /// Runs safe with the arguments specified, with the option to assert on the exit code.
    ///
    /// This was changed to use the assert_cmd crate because the newer version of this crate
    /// provides *both* the stdout and stderr if the process doesn't exit as expected. This is
    /// extremely useful in this test suite because there are lots of cmds used to setup the
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

    // Executes arbitrary `safe` cmds and returns
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

    // Executes arbitrary `safe` cmds and returns
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

    fn get_version_from_release_version(release_version: &str) -> Result<String> {
        let mut parts = release_version.split('-');
        parts.next();
        parts.next();
        parts.next();
        let version = parts
            .next()
            .ok_or_else(|| {
                eyre!(format!(
                    "Could not parse version number from {}",
                    release_version
                ))
            })?
            .to_string();
        Ok(version)
    }
}
