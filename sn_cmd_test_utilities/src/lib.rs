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

    pub const DBC_WITH_12_230_000_000: &str = "ee787589d84b6a09c0230310c9d661177c6efd4ed3b079f3baff803dfe4e7a03816175f292d53f4999f7836ce43114954a40af5585871ff5fa106eb5a857b441c2caf8b4fa8f2d69fd36782be853f48d5e2dfafc9f4dcf5eda0c53b39b44030d7f6c96f2307c759b323035a50d94ff27dd92026c7620d30318cfeba2873c65622eac23674eb82f481738f09d4e82868a00c0a7e751a4ce8e03bc0979417f6588307970085a7537633d7616d21549b416c2fb1d6e0a31bc7767022d502984857664cfa9da97fd01b7619401cabfc0e1b43a2e0f47e84c929d31182214788609da5b5c2e1a9ef26717616d63592f8b93de51f6d1c3d3284a91ffa9c69d0702f6b7cf90aecad7d07b063bd6933801b5152574aee9513bc1d91a798d1d2687266f96c736b52f381583b55a958c9790ce7696fb1e58f792126ae8a69f24103a1e2767d7fbf1da4e9ceb2dfb676e39a21a232b1921c5a156e551d7d66d2c3f59c664b575a49cc6db4cc6e46fc46b40beab0b9239d55a4e2f5931b2978a199ab4e429a9c29f6c6e765341db4c9bb22cbd5837a4b6c488f7d8c130c785f42a8ada556763c48f8885156e106e18e14b32efb074b4d921e1f16f605ecd7263a27734d2658ebcf6e08c70db76159d6227142c2b2b1f5c629ef75efd1648c96e8f84b51a985fd27bd737b93b9d55263d3fa460794bb990095966724a4c1acd1f9d1dff4f1f7e39e63003e80659785062c5ed7e37921f6acea34859f77994811c90e152ba2e80457cd3599ed64da1f5da06d4664cc85f4428611fa2673ea25487cd4b965bf9cdec0d55c9bc45924d7b04298c4b3756ae6cff29082bd1b6f58cb690520559b246b0ede184c951b60023ade9d3660e465cbb4d7021cab1dfe6ad2351e887eb938be624b7ac84215d0965e67e9d9e36d6d773fc1e7aa8f68f2f66cfee16b8d826ca50e79576f146039284c78959006329810339dbd6f411b5d350f127da5bd0d9d0ad8045b32097a8bc45f9885f2940a6482119cb26d5107a482419a4307bc989983d79620c385286f66805865a8a68b09259dbda4e6d4ec907d44cf0285415cc362c6eb260eabd634f562bb836a03167c2e0faf7bb268dd37eb24d8e4de8a9172d2946d85121a07564e02db54b8d71b1f7d94d7da5b53b6b0abc86a8555a119c946ce4d24501c97567523caf078bc6858274695dfcfc8505f0439d423b73177eaa3fd47e73cfa92a0b0a3629ba6475368701c7df62264cd313b2430f9291f4f3936a9c0e704b1da31c0991d933c00d643e3195412738b0874a7e769c2a1393f0b380aa5a15d2b1e0cb2c29f5d733f0365b12fe4a4303a4bc3fccbd3e5c98c444b96ed922a200af35d6ac1f5b285485338800000000000003a01d56a59164605850ca63ecd9d68a781b071833a5bfe981b28bdf96f7363641ec5e596f34605a502ead75135c5f020ea4890817dd8463365d6b00faa71e2ecb00ab7a32d9b0198695dc35c13ac48969873ebc26403dce0a7414de4519774bc1b51b153a7779cb29a7d722553406ab078dd8f4ca602f39be2bf4aacdef93c51d87233c372e62a7f204e05c73aa9d1dcec57e076347288f4facb30cc521033da0041dabd2752299e85545f52eba607b32f7d224b0262ffd24d430a758ce2898f38ff795d193d54ec5af2f934400b72d548485a285568c47bb0d02a15f1a1e39f1c92c9d45c32d5b1ebcc25c0bddd51f7de3b518bf20dd035ea944e7dc97007565a1a096fc386685e0cf98a6de5e7fa2ad6807a44c060ebd438a54192706348e2caba3fc831b2ca960e601405c3bf69f27a561bda1e54166531683f05af3cd2982db50efa915a1b891ce2e9c6fd5c9b9d553c6987d3a7084a383e65b1fc57bf60491eff48db81213f9d528d004b8e3b72ea5a411c214fec52132e53f8cd2425fea9548f77cdbb1421d078ea25f92094923a94b1157016eed6c0f9cc7d97f3b76de22b105a1bcc378991127585a389d710c2b2caf5bb169e089c810671f03e94b74b769a473d277cc3194f7964bb7403cd993fbf01db3d50c98594ca8e425834444959b26b427ed255f55306db5204c6093b66629ab4b83b99ef4316e5795d0999a3017e102c99d57beac28fb791ea877dda9894ef32dd9db1f3f52960d0e7ad01b86d903c3b67c6e868ee75935eebe988689f8c6f8552a6db3cbdfa857c2fa9b2e88d68a11243da67627de3d78500e40d0aa0086b512d2465d5f1d7c36586e6a69119445e4c2def64c2e84c9bcb20267766f08cf2572c6ee01deb82c6282e384d7adb95f8dd1f395ad62fc03c7206e0c381237ff86ab493dacf7c4d73b01597b675d52979ef58b1e8e454e1302bc6d662c8979aa6e3872e9d8714b1546de055e07a072048233f2671732c67afb75f5b6b537d2de02bdc0bf0d6b68d084448fff75b13ce4d8adb56ebd2e02e583331e018e85a4b880979acc2f680e11b1e1a397c645157b750d97dc47b50ce0692d2c3d4aff634d81d67a14d2b8d403f6dde097683e116cc5a192f8b4e184aff8fc57e5b236df978077f6f3d052d8207e6aa27b0c06e562af543258c358a9b60e14bb78505933cc5b2beddb9f2bba54d08eb1bd48b9a03955a3519654f91c0aba98f686168570cd11cf4f10da3e2690b388327775313965deb4f764d3387645e85678b2ab5a584c1e6990072016cc4953af882bcb98c116085ca8db54a482724cb982e6350041722c8cef1bc97c7ba7d8bd87170fd4a45c7c5320a6a305f2c0369bf686de898cae9cc24a108423331baf5d1518122ce140ecd2749f54cff563ee7a0fd124ac4c24f8c1e5db057c943a4065bed5ec8800000000000003a068eecbebca0cce0f1de3a3d82dc1c9d757eb053f11f718bdbc5802d4e50ae602573b73b5f3a4a5bb8a02c6229109858800000000000000020d93917b310e05b6052ccb08d618b7f9ba840b44ef2248c6f3a167660dc0e2ca02a8577c436aaa2396206f01c9216c8f7aae0faee99f2b22bee78e790a3f7c7b9fe5dafe1e328c84ecd5ce1a8398528e01b7fe096d7e0af681b6e24e656f24a496535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe8800000000000000011ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b32d5b5cb4c4214606830338c1a941a7f8cc416a6a62fea720b9de6af2e3e51b22c166aa90ffefe7e3c51dcdbad8fa1377d86cfdb01baada35d620519d9f33bcd0000000000000001630a9a5331b8cc59a1902a4fdfc9da818730c22d29b61f3bc22c3eb3b8a16e03000000000000000100000000000000015f74debf101b87e707d9a77ed7a8a6b016303435dbbf076ad51d7e5f005bdd0dbe0d820d8341b93f9244dac38e12d60794d5618189cf6277a014abadfd794728e5ac043408982030ed04f320593f6bf3f097a3297d23745f17b460a2504f77ab65e38eb4bcab5bd8d8059641aa65188f6d2412dd0ae4676aa28114500eea0b5eae8919efc0f2d57a166e33ca3ada06b738aab683457882ebc8dc80cd829f9e292837a00789952636524a544ae1fada10439ed9478b6d0127c5fd13ed74e9e1870000000000000001b6d944cd99586d869ea8ad5fdac2f3c8aab7d5aec1e01cd7336d95cba8fb396b1ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b0000000000000001ee787589d84b6a09c0230310c9d661177c6efd4ed3b079f3baff803dfe4e7a03816175f292d53f4999f7836ce43114954a40af5585871ff5fa106eb5a857b441c2caf8b4fa8f2d69fd36782be853f48d5e2dfafc9f4dcf5eda0c53b39b44030d7f6c96f2307c759b323035a50d94ff27dd92026c7620d30318cfeba2873c65622eac23674eb82f481738f09d4e82868a00c0a7e751a4ce8e03bc0979417f6588307970085a7537633d7616d21549b416c2fb1d6e0a31bc7767022d502984857664cfa9da97fd01b7619401cabfc0e1b43a2e0f47e84c929d31182214788609da5b5c2e1a9ef26717616d63592f8b93de51f6d1c3d3284a91ffa9c69d0702f6b7cf90aecad7d07b063bd6933801b5152574aee9513bc1d91a798d1d2687266f96c736b52f381583b55a958c9790ce7696fb1e58f792126ae8a69f24103a1e2767d7fbf1da4e9ceb2dfb676e39a21a232b1921c5a156e551d7d66d2c3f59c664b575a49cc6db4cc6e46fc46b40beab0b9239d55a4e2f5931b2978a199ab4e429a9c29f6c6e765341db4c9bb22cbd5837a4b6c488f7d8c130c785f42a8ada556763c48f8885156e106e18e14b32efb074b4d921e1f16f605ecd7263a27734d2658ebcf6e08c70db76159d6227142c2b2b1f5c629ef75efd1648c96e8f84b51a985fd27bd737b93b9d55263d3fa460794bb990095966724a4c1acd1f9d1dff4f1f7e39e63003e80659785062c5ed7e37921f6acea34859f77994811c90e152ba2e80457cd3599ed64da1f5da06d4664cc85f4428611fa2673ea25487cd4b965bf9cdec0d55c9bc45924d7b04298c4b3756ae6cff29082bd1b6f58cb690520559b246b0ede184c951b60023ade9d3660e465cbb4d7021cab1dfe6ad2351e887eb938be624b7ac84215d0965e67e9d9e36d6d773fc1e7aa8f68f2f66cfee16b8d826ca50e79576f146039284c78959006329810339dbd6f411b5d350f127da5bd0d9d0ad8045b32097a8bc45f9885f2940a6482119cb26d5107a482419a4307bc989983d79620c385286f66805865a8a68b09259dbda4e6d4ec907d44cf0285415cc362c6eb260eabd634f562bb836a03167c2e0faf7bb268dd37eb24d8e4de8a9172d2946d85121a07564e02db54b8d71b1f7d94d7da5b53b6b0abc86a8555a119c946ce4d24501c97567523caf078bc6858274695dfcfc8505f0439d423b73177eaa3fd47e73cfa92a0b0a3629ba6475368701c7df62264cd313b2430f9291f4f3936a9c0e704b1da31c0991d933c00d643e3195412738b0874a7e769c2a1393f0b380aa5a15d2b1e0cb2c29f5d733f0365b12fe4a4303a4bc3fccbd3e5c98c444b96ed922a200af35d6ac1f5b285485338800000000000003a01d56a59164605850ca63ecd9d68a781b071833a5bfe981b28bdf96f7363641ec5e596f34605a502ead75135c5f020ea4890817dd8463365d6b00faa71e2ecb00ab7a32d9b0198695dc35c13ac48969873ebc26403dce0a7414de4519774bc1b51b153a7779cb29a7d722553406ab078dd8f4ca602f39be2bf4aacdef93c51d87233c372e62a7f204e05c73aa9d1dcec57e076347288f4facb30cc521033da0041dabd2752299e85545f52eba607b32f7d224b0262ffd24d430a758ce2898f38ff795d193d54ec5af2f934400b72d548485a285568c47bb0d02a15f1a1e39f1c92c9d45c32d5b1ebcc25c0bddd51f7de3b518bf20dd035ea944e7dc97007565a1a096fc386685e0cf98a6de5e7fa2ad6807a44c060ebd438a54192706348e2caba3fc831b2ca960e601405c3bf69f27a561bda1e54166531683f05af3cd2982db50efa915a1b891ce2e9c6fd5c9b9d553c6987d3a7084a383e65b1fc57bf60491eff48db81213f9d528d004b8e3b72ea5a411c214fec52132e53f8cd2425fea9548f77cdbb1421d078ea25f92094923a94b1157016eed6c0f9cc7d97f3b76de22b105a1bcc378991127585a389d710c2b2caf5bb169e089c810671f03e94b74b769a473d277cc3194f7964bb7403cd993fbf01db3d50c98594ca8e425834444959b26b427ed255f55306db5204c6093b66629ab4b83b99ef4316e5795d0999a3017e102c99d57beac28fb791ea877dda9894ef32dd9db1f3f52960d0e7ad01b86d903c3b67c6e868ee75935eebe988689f8c6f8552a6db3cbdfa857c2fa9b2e88d68a11243da67627de3d78500e40d0aa0086b512d2465d5f1d7c36586e6a69119445e4c2def64c2e84c9bcb20267766f08cf2572c6ee01deb82c6282e384d7adb95f8dd1f395ad62fc03c7206e0c381237ff86ab493dacf7c4d73b01597b675d52979ef58b1e8e454e1302bc6d662c8979aa6e3872e9d8714b1546de055e07a072048233f2671732c67afb75f5b6b537d2de02bdc0bf0d6b68d084448fff75b13ce4d8adb56ebd2e02e583331e018e85a4b880979acc2f680e11b1e1a397c645157b750d97dc47b50ce0692d2c3d4aff634d81d67a14d2b8d403f6dde097683e116cc5a192f8b4e184aff8fc57e5b236df978077f6f3d052d8207e6aa27b0c06e562af543258c358a9b60e14bb78505933cc5b2beddb9f2bba54d08eb1bd48b9a03955a3519654f91c0aba98f686168570cd11cf4f10da3e2690b388327775313965deb4f764d3387645e85678b2ab5a584c1e6990072016cc4953af882bcb98c116085ca8db54a482724cb982e6350041722c8cef1bc97c7ba7d8bd87170fd4a45c7c5320a6a305f2c0369bf686de898cae9cc24a108423331baf5d1518122ce140ecd2749f54cff563ee7a0fd124ac4c24f8c1e5db057c943a4065bed5ec8800000000000003a068eecbebca0cce0f1de3a3d82dc1c9d757eb053f11f718bdbc5802d4e50ae602573b73b5f3a4a5bb8a02c6229109858800000000000000020d93917b310e05b6052ccb08d618b7f9ba840b44ef2248c6f3a167660dc0e2ca02a8577c436aaa2396206f01c9216c8f7aae0faee99f2b22bee78e790a3f7c7b9fe5dafe1e328c84ecd5ce1a8398528e01b7fe096d7e0af681b6e24e656f24a496535faeae2d5b27e4c0b37f2d8667128dbe048a2814ce4d24fc60f7a0dda7c87e1d3b30254c25aba701ff6eeb9ebe8800000000000000011ca2ae5ab0ed3dc52cc9cd341c4a482e9d9f5f2d81d981a11032821603b863baa19a900087052a2f799092f29d7e2c8b32d5b5cb4c4214606830338c1a941a7f8cc416a6a62fea720b9de6af2e3e51b22c166aa90ffefe7e3c51dcdbad8fa1377d86cfdb01baada35d620519d9f33bcd0000000000000001630a9a5331b8cc59a1902a4fdfc9da818730c22d29b61f3bc22c3eb3b8a16e03000000000000000167d1153fd164ca0ea3c1e2e89133ff09560d00de7d279eb39fe474b68da1e1336ffade2e011700148ce4cd6e352a160d07421e69077962a102d1ca6230637e4e8592ce345e86e0fcc51b62f87b33c441fa618448e206762851585a24db5815b1a15b0e8912db828956a303f91fa82edac8b02df064b2ed111926cdfc437aede140f2497eed1aaf8c00000000000000280f6a383f7677d9adc41a11f716550ebf32074fbea7aaf02ce85aa5a96d1ddf85e2a8b1c640767e7d952aae6c04a42999e25e455e894b8b8abc03c417f0d382963f47465306ceeada07416c279b43c2c167c2a0ca9adfcd37078dfa383a829402f667f82f426c5c4bc731baef134cb0f973f977065698345d7ac748098022f18fc8449add772c9c978993c48055b6fba9e37308c9edd1010952846610da2df9bdec54d85c2d43e7aaf65ca83c5441380400000000000000208c198334c661452d01fc3fb19f31319537f790457f80e3bd2e0a8bcb84cdb6d266490c43c98a59dfc3a8aff74e673e89299fa246893bc7d67b6ea6f1b4a7de9e531354ec832dd73dd36494da0d21bd2800000000";

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

    /// Generates a `sha3_256` digest/hash of a directory tree, for the purposes of comparing it to
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

    pub fn parse_keys_create_output(output: &str) -> Result<(String, String)> {
        serde_json::from_str(output)
            .map_err(|_| eyre!("Failed to parse output of `safe keys create`: {}", output))
    }

    /// Runs safe with the arguments specified, with the option to assert on the exit code.
    ///
    /// This was changed to use the `assert_cmd` crate because the newer version of this crate
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
