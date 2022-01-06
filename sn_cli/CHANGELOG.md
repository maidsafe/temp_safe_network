# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

## v0.43.2 (2022-01-06)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 1 calendar day.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - rename dest to dst ([`bebdae9`](https://github.com/maidsafe/safe_network/commit/bebdae9d52d03bd13b679ee19446452990d1e2cf))
    - safe_network-0.52.9/sn_api-0.50.4 ([`a64c7e0`](https://github.com/maidsafe/safe_network/commit/a64c7e0414b77f545cb8cdbf64af0fb7212d1f2e))
    - sn_api-0.50.3 ([`5f7000c`](https://github.com/maidsafe/safe_network/commit/5f7000c5ec5895fb3f4c4a17a74ada52bb873fc7))
    - safe_network-0.52.6/sn_api-0.50.2 ([`0a70425`](https://github.com/maidsafe/safe_network/commit/0a70425fb314de4c165da54fdc29a127ae900d81))
</details>

## v0.43.1 (2022-01-04)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_cli-0.43.1 ([`db51539`](https://github.com/maidsafe/safe_network/commit/db515397771f117b3bf095e1a4afb897eb4acafe))
    - safe_network-0.52.4/sn_api-0.50.1 ([`4bb2adf`](https://github.com/maidsafe/safe_network/commit/4bb2adf52efdac6187fffc299018bf13f3398e14))
</details>

## v0.43.0 (2022-01-03)

### refactor (BREAKING)

 - <csr-id-715a154fe7448cd18decd0a666ae11fb02eadedb/> remove dry-run as arg from all APIs and make it a Safe instance mode

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.50.0/sn_cli-0.43.0 ([`ee86dc7`](https://github.com/maidsafe/safe_network/commit/ee86dc7ab1781731d3be19f9d7f414f157a91edb))
    - remove dry-run as arg from all APIs and make it a Safe instance mode ([`715a154`](https://github.com/maidsafe/safe_network/commit/715a154fe7448cd18decd0a666ae11fb02eadedb))
</details>

## v0.42.0 (2022-01-03)

### Bug Fixes (BREAKING)

 - <csr-id-fe13166b6dc4ae0fdd96b20a135baf7ebef3647b/> properly handle scenarios when retrieving empty FilesContainers
   - Also removing the Default impl for VersioHash as it's meaningless, and invalid content version hash.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.49.0/sn_cli-0.42.0 ([`4f600e1`](https://github.com/maidsafe/safe_network/commit/4f600e179bfbf6ac018876cca6f7fc193f5b5f1e))
    - properly handle scenarios when retrieving empty FilesContainers ([`fe13166`](https://github.com/maidsafe/safe_network/commit/fe13166b6dc4ae0fdd96b20a135baf7ebef3647b))
</details>

## v0.41.0 (2022-01-03)

### Documentation

 - <csr-id-4e0d82ec847edd0da621208e906deffb5aa89ac2/> align cli user guide with current features
   The CLI user guide hadn't been updated for some time, so I brought it more in line with current features. Where relevant so far, I also trimmed the text to make it more concise.
   
   The following updates were made:
   
   * Reduce the indentation levels of the whole document.
* Add a Quickstart section to get users going with a single command if they want to avoid reading lots of setup text. Windows was explicitly left out because having a Git Bash installation does not constitute the ability to quick start.
* Re-work the previous 'Download' section to an 'Installation and Setup' section.
* Restructure the 'Networks' section with a new example using remote networks.
* Temporarily remove the 'Interactive Shell' section. This was using a feature that didn't exist any more.
* Temporarily remove the 'SafeKeys' section, which used examples referring to removed features. This can be added back in but applied to currently relevant features. I didn't understand it enough to apply it at the moment.
* Trim the wordy prose in the 'Files' section. This helps the reader get to the salient issues quicker.
* Temporarily remove documentation for `files sync`. This command wasn't behaving as described currently, and I'm not sure the NRS stuff still applied. Didn't have time to go into it in detail. This section also has wordy prose that could benefit from being trimmed down. I'll do that when I add it back in.
* Temporarily remove the NRS section. This is completely out of sync with the new NRS commands and terminology and I suspect it can also be made less verbose.
* Remove the 'Auth' section, since this feature has been removed.
* Remove the 'Sequences' section, since this feature has been removed.
* Remove the 'Updates' section, since this feature is currently not enabled.
* Remove the `shell` style from the ``` code blocks since we don't need shell syntax highlighting. There's only one line of shell and the rest is showing the output of the command, which isn't shell code.

### New Features

 - <csr-id-a8f84f7002cf7b043fb7606a20987ddfb29972f8/> make all read/write access to CLI config file async by using tokio::fs

### New Features (BREAKING)

 - <csr-id-4adaeaff4f07871840397adc3371ec8b3436e7ce/> change files APIs to accept std::Path for path args rather than only &str
   - Changed the files_container_create API to now create just an empty FilesContainer

### refactor (BREAKING)

 - <csr-id-ff1dd477aaea2a4dda6c9c15b5822b1b3a7514b7/> ProcessedFiles redefined on more specific data types instead of simply Strings

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 10 calendar days.
 - 6 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.1/sn_api-0.48.0/sn_cli-0.41.0 ([`e38925e`](https://github.com/maidsafe/safe_network/commit/e38925e07d69432db310fc8ec9803200ea964ab2))
    - change files APIs to accept std::Path for path args rather than only &str ([`4adaeaf`](https://github.com/maidsafe/safe_network/commit/4adaeaff4f07871840397adc3371ec8b3436e7ce))
    - align cli user guide with current features ([`4e0d82e`](https://github.com/maidsafe/safe_network/commit/4e0d82ec847edd0da621208e906deffb5aa89ac2))
    - minor refactor and changes to CLI report errors ([`f1bb190`](https://github.com/maidsafe/safe_network/commit/f1bb1909f3fb506c1b7ec9b660ad533b7b8b9044))
    - ProcessedFiles redefined on more specific data types instead of simply Strings ([`ff1dd47`](https://github.com/maidsafe/safe_network/commit/ff1dd477aaea2a4dda6c9c15b5822b1b3a7514b7))
    - make all read/write access to CLI config file async by using tokio::fs ([`a8f84f7`](https://github.com/maidsafe/safe_network/commit/a8f84f7002cf7b043fb7606a20987ddfb29972f8))
</details>

## v0.40.0 (2021-12-22)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 6 calendar days.
 - 8 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.0/sn_api-0.47.0/sn_cli-0.40.0 ([`6b59ad8`](https://github.com/maidsafe/safe_network/commit/6b59ad852f89f033caf2b3c7dfcfa3019f8129e8))
    - sn_cli-v0.39.2 ([`f6ffbdb`](https://github.com/maidsafe/safe_network/commit/f6ffbdb5d999f84e3531a6dcd9dcdbacefd50d18))
    - re-introduce version arg for cli install script ([`d201f7e`](https://github.com/maidsafe/safe_network/commit/d201f7e3480a8a12f488e2a54886cca942904a18))
    - use s3 as download source for sn_node ([`dffcd4e`](https://github.com/maidsafe/safe_network/commit/dffcd4e3dd07f99dd3a4f4330637cab9380db9c3))
    - remove `self-update` feature from `node install` ([`f59ec2c`](https://github.com/maidsafe/safe_network/commit/f59ec2c6da30b13fc2606d2834fad108a56c3621))
    - replacing calls to unwrap() and expect(...) with proper Result handling ([`b6f0c3f`](https://github.com/maidsafe/safe_network/commit/b6f0c3f193e8116bcd08126b949eb1a2e9b5aaa5))
    - re-enable CLI tests in CI ([`8aeca3d`](https://github.com/maidsafe/safe_network/commit/8aeca3dffdf92341d34e1f6856160cff57cf0d6a))
    - reduce default query timeout for cli ([`7dce30c`](https://github.com/maidsafe/safe_network/commit/7dce30c10262573362e6f60c284a51696de36d01))
</details>

## v0.39.2 (2021-12-21)

This is a manually generated changelog, as `smart-release` seemed to have some issue detecting a change in the `sn_cli` crate.

* refactor: use s3 as download source for sn_node
* chore: remove `self-update` feature from `node install`

## v0.39.1 (2021-12-16)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_cli-0.39.1 ([`943166a`](https://github.com/maidsafe/safe_network/commit/943166ab6d88266909ec6cd8a8e98bbbf21ec18d))
    - sn_api-0.46.2 ([`6df94b1`](https://github.com/maidsafe/safe_network/commit/6df94b1d1fb017c9b02e566ca22a518f885397c8))
</details>

## v0.39.0 (2021-12-16)

This is a manual changelog entry. Subsequent CLI releases will use the automated changelog generated by `smart-release`.

### New Features (BREAKING)

* The `nrs create` command has been renamed to `nrs register`
* The `nrs add` command now has some different names for arguments
* The `node join` command was updated such that you can now pass the genesis key and some other arguments were renamed for clarity
* Various commands were updated to have their error handling and suggestions improved (using the [color_eyre crate](https://docs.rs/color-eyre/latest/color_eyre/))
* The 'self update' feature of the CLI has been disabled for the time being
* The CLI is now compatible with various API updates that have been made
 - <csr-id-18879590ddfcf125133a6b2b8f3f372e8683be42/> rename Url to SafeUrl

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 18 commits contributed to the release over the course of 6 calendar days.
 - 15 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_cli-v0.39.0 ([`ae44ebb`](https://github.com/maidsafe/safe_network/commit/ae44ebbf46a72bf1897f8c6004466290f8425db7))
    - safe_network-0.51.3/sn_api-0.46.1 ([`9be440b`](https://github.com/maidsafe/safe_network/commit/9be440b36db07e1c04ab688b44ef91e4a56ed576))
    - remove use of `indicatif` crate ([`b09f830`](https://github.com/maidsafe/safe_network/commit/b09f8307fd1047eb92d2cfe1a6ed38731f6e09e2))
    - sn_api-0.46.0 ([`634a8f9`](https://github.com/maidsafe/safe_network/commit/634a8f9f307598c51305067444514b43c85f196d))
    - add recursion limit for clippy ([`e52a0c0`](https://github.com/maidsafe/safe_network/commit/e52a0c063f747e0be1525f07f8f759f4b9d042a7))
    - disable self update feature in sn_cli ([`1a8bbb7`](https://github.com/maidsafe/safe_network/commit/1a8bbb7ebe4737f931cec259dc7863b84531f2c3))
    - build asset download url in cli ([`8d402e8`](https://github.com/maidsafe/safe_network/commit/8d402e8b2255edf139e3c3507e6597e581719ad4))
    - use versioned self update crate ([`855f304`](https://github.com/maidsafe/safe_network/commit/855f3042859dd641231135de618520050861c348))
    - rename Url to SafeUrl ([`1887959`](https://github.com/maidsafe/safe_network/commit/18879590ddfcf125133a6b2b8f3f372e8683be42))
    - safe_network-0.49.0 ([`6f5516d`](https://github.com/maidsafe/safe_network/commit/6f5516d8bb677462ea6def46aa65a1094767d68c))
    - update use of nrs in other areas ([`45f1f02`](https://github.com/maidsafe/safe_network/commit/45f1f02bbdb61e7c698f1f6a5a62fb63ed01aae3))
    - address feedback from pr ([`735a68a`](https://github.com/maidsafe/safe_network/commit/735a68a45aa264a5462642f4fb1e26f05bdf28ca))
    - remove previous nrs tests ([`ffea442`](https://github.com/maidsafe/safe_network/commit/ffea442b710f0051483523297968c9bcbc81419b))
    - feat(sn_cli): rename nrs `create` to `register` ([`d08858c`](https://github.com/maidsafe/safe_network/commit/d08858cde855e89b6bf44ae5194500fd6c754288))
    - add coverage for `nrs remove` command ([`db98472`](https://github.com/maidsafe/safe_network/commit/db98472ad8af2a61aa8edf594119ed6b7a92d3ad))
    - feat(sn_cli): refined ux and behaviour for `nrs add` ([`76fee46`](https://github.com/maidsafe/safe_network/commit/76fee4674f4a4933c79b90afcd66598e1a05c5fd))
    - feat(sn_cli)!: `nrs create` only creates topnames ([`c284f07`](https://github.com/maidsafe/safe_network/commit/c284f0787afe0d079e53b79b3a9d74cad04c4b0e))
    - minor improvement to client log msgs related to configured timeouts ([`58632a2`](https://github.com/maidsafe/safe_network/commit/58632a27d271140fc4d777f25a76b0daea582426))
</details>

## v0.38.0 (2021-12-08)

<csr-id-f3d3ab2b059040ff08b6239c8a6583c64eac160e/>

### ⚠ BREAKING CHANGES

* Blob and Spot entries in the DataType enum are merged into a single Bytes entry.

The client API in the network has now been updated to encapsulate the Blob and Spot data. This makes
the client API much easier to use, as callers no longer need to be concerned about the size of the
data they're trying to store and retrieve.

This was tested against v0.33.12 of `safe_network`.

* update cli for 0.38.x of sn_api ([487e0ab](https://github.com/maidsafe/sn_cli/commit/487e0ab1a5af333377b3743cafb74b7551846621))

### Bug Fixes (BREAKING)

 - <csr-id-7ffda3021fb36533f22538b1100acfa71b13cd81/> nrs get with versions, nrs_map always returned

### New Features (BREAKING)

 - <csr-id-8787f07281e249a344a217d7d5b0e732a7dd7959/> easy to use nrs_add and rigorous nrs_create

### chore (BREAKING)

 - <csr-id-f3d3ab2b059040ff08b6239c8a6583c64eac160e/> include remote addrs in listerners threads log entries


### New Features

 - <csr-id-b09e769db55aa3f362f49e62d7124d8030ed34bf/> add baby fleming to networks list
   The `node run-baby-fleming` command is updated to add the baby-fleming network to the networks list.
   
   This is something that makes sense from a UX point of view, but it can also make it easier to use
   some automation to add extra nodes to the network in an automated process that tries to get a local
   network to settle to having 7 Elders, which is required to use some functionality for the network.
   With the `baby-fleming` network added, it can be used as part of the `node join` command.
 - <csr-id-c5b0f0b630f673697367361508a30caf7ad787bd/> fixing multimap_remove and adding multimap_get_by_hash API
 - <csr-id-6e700fc9776409e88abd0a2e61e450f400985801/> add 'xorurl pk' subcommand which generates the SafeKey XOR-URL for the given public key
 - <csr-id-ac02f3ed351298ab86d650ffe95b666530138138/> allow to pass a SafeKey URL top 'keys show' command to display its public key
 - <csr-id-4f6c526e194f1b949c1b5b126442150157b7b0ba/> support transfers to BLS public keys
 - <csr-id-0f2f3c74dc81fefbc719e79f41af434023ac0462/> re-enabling dry-run feature for Blob API as well as for CLI commands
 - <csr-id-97d131435515406eb5a2c93aa9d61d0929e39ba2/> adding a new 'keys show' subcommand to display CLI's owned SafeKey, pk, xorurl and sk
 - <csr-id-42de5f0fbb57c7e5d4f98dbbe8bf47bd04dbf9b1/> add --for-cli flag to keys create command which set the newly created SafeKey for CLI use
 - <csr-id-5499aeb2f755ce363b709c5379b860048c92ce5a/> pass SecretKey by reference
 - <csr-id-fedc64f43e586680933865ec16b4d976b3b68e39/> log format matches sn_node log format
   [module_path] level time [file:line] args
   See
   https://github.com/maidsafe/sn_node/blob/de12c4d36c63451ed5283d98d0989fcac224b937/src/utils.rs#L60-L76
   for matching sn_node log format
 - <csr-id-85462b6fb58f16f4797d7ef2816e96a287af7ad9/> adapting to new Seq type where Policy is immutable
 - <csr-id-45f11ae22df242e229d01bfc5dc2b6ac9de8536d/> customise the error message displayed when a panic occurred
 - <csr-id-6c9cf24df423abae568fab63fc6615d9f7a3df68/> update sn_client and data types
 - <csr-id-a836991edbc0e0394d762525ad49025c5071a2cc/> have CLI and authd to set client config path to ~/.safe/client/sn_client.config
 - <csr-id-2a43ca8fb10dcdbf085890643c673491399b1a8b/> command to configure networks in the config by providing a list of IP and ports
 - <csr-id-e366c878da84d2cf051bbc692e6b80c675ef8393/> add auth version subcommand which prints out the authd binary version
 - <csr-id-7a77ef4e3f3730d2f033d0365b2446bd560b1e21/> default to check balance of key assigned to CLI when no sk is provided to keys balance command
 - <csr-id-c20932f5c15fa16ccad907208522b9c9b52bb062/> support transfers to Ed25519 public keys in addition to using a Wallet or SafeKey URLs
 - <csr-id-b2e3faf9b0943ec779b1e513c76179048dbb0db3/> re-enable implementation for coins transfers with Ed25519 keys
 - <csr-id-58ecf7fbae95dee0a103ce39d61efaac6e2cf550/> adapt authd client api and service names to new terminology of Safe creation and unlocking
 - <csr-id-3f23687471b846e3ad1e2492c237a21f212b753f/> reenable decoding auth reqs and basic app key generation
 - <csr-id-b994b8d6ec1fcfc540e91aa9df79ba849aee7647/> setting up IPC for auth

### Documentation

 - <csr-id-49ad9eecbff02a0455560449faf308165c13e10f/> update cli readme s3 node config location
   Having readme examples point to the correct location should help avoid any confusion
 - <csr-id-56e076b74747ad4e9f82a7df7e82f1c97e2b4496/> add visual c++ libs to windows requirements
   Testing has shown that without the VS C++ redistribution package installed we get the error `error while loading shared libraries: api-ms-win-crt-locale-l1-1-0.dll: cannot open shared object file: No such file or directory`
 - <csr-id-f5c3106834b6e0033adf19ea631e8d2fc5c2ed1e/> add notes for sn_fs

### ⚠ BREAKING CHANGES

* Multimap entries for NRS will be assigned a different data type when they are
created.

Note, the breaking change relates to the previous commit. For some reason the workflow for updating
the version and deploying a new release didn't trigger.

This commit and PR is just to try and force a new release.
* Multimap entries for NRS will be assigned a different data type when they are
created.

Updates to the latest version of sn_api and also removes the ignore attribute from NRS tests, since
the new version of the API has a fix for NRS issues.

* force new release with readme update ([99c3760](https://github.com/maidsafe/sn_cli/commit/99c37603ba5ab80fac27e7a4752df71376ee16ff))
* upgrade sn_api to 0.37.0 ([9da8a77](https://github.com/maidsafe/sn_cli/commit/9da8a77cfca5eb99ef4cea6d38de856408624537))

## [0.36.0](https://github.com/maidsafe/sn_cli/compare/v0.35.0...v0.36.0) (2021-10-04)

### ⚠ BREAKING CHANGES

* This is to facilitate the new way that files are stored on the network, which
was updated to support changes in self encryption.

The `files get` command was updated to support the Spot data type.

There were some misc test updates to support these new changes:
* All test runs were updated to use a single thread, since multi threading seemed to result in more
  failures.
* A new cat test case was added for checking the retrieval of spot files.
* Files tests were updated to support the addition of a new large file in the test data.
* Files tests were updated for new file counts upon addition of the new test data file.

Right now, all tests relating to NRS have been marked as ignore, as there is some kind of problem
with NRS. This will be investigated shortly and once it's resolved, these tests will be re-enabled.

### Features

* support retrieval of spots ([348ebda](https://github.com/maidsafe/sn_cli/commit/348ebda6f1aedad61c84773ec4cf4e3cfab9ab00))

## [0.35.0](https://github.com/maidsafe/sn_cli/compare/v0.34.0...v0.35.0) (2021-09-28)

### ⚠ BREAKING CHANGES

* the `network set` command now has different arguments.

The `network set` command creates a new network in the configuration, that has a name and a set of
nodes. Now that we require a genesis key to connect to a network, a genesis key argument was created
for the command.

Test coverage has been added for this change and also to various other parts of the `Config` struct.
They've been added as unit style tests rather than to the integration suite, because these commands
are fairly simple and don't require a full integration test. The coverage is mostly on the `Config`
struct rather than the commands, because the `add` and `set` commands basically just call
`Config::add_network` and it would be quite wasteful and a bit of a maintenance issue to cover both.
One test was created for the `set` command just to make sure it correctly maps its arguments onto
the `add_network`.

Some refactoring took place to make testing a bit easier. The `Config::read` function was changed to
a constructor that accepted both the CLI config file path and the default node config path. This
gives us the ability to pass temporary file paths that are created with `assertfs`, meaning we're not
working with user profile directories when the tests are run on a someone's development machine. Any
command handlers that used the `Config::read` function were updated to accept a reference to a
`Config` as a parameter. The `Config` can be created in the CLI config, and that's where we can pass
in the real profile directory parameters.

The `NetworkInfo` enum entries were renamed from Addresses -> NodeConfig and ConnInfoURL ->
ConnInfoLocation. Since the addition of the genesis key, the network info was no longer just a set
of addresses, and in terms of the connection info, that could be either a URL or a file path, so it
made more sense to name that a bit more generally.

### Features

* updates for new sn_api and safe_network 0.31.x ([944596d](https://github.com/maidsafe/sn_cli/commit/944596dfcb97272ce220bb9a026233c42c6c0505))
* use genesis key in network set command ([3b9f60c](https://github.com/maidsafe/sn_cli/commit/3b9f60cd1202a6713098378006e7e485c5ae90bd))

## [0.34.0](https://github.com/maidsafe/sn_cli/compare/v0.33.8...v0.34.0) (2021-09-28)

### ⚠ BREAKING CHANGES

* there have been commands that have been removed and we also can't upload things
like files with 0 bytes in them any more.

### Features

* updates for new sn_api and safe_network 0.28.x ([4bbf857](https://github.com/maidsafe/sn_cli/commit/4bbf857454126a7c0b733a477a8086354fd613d2))
* updates for new sn_api and safe_network 0.31.x ([ec6dd1b](https://github.com/maidsafe/sn_cli/commit/ec6dd1bb76b91cbee6d4945bf873ea6b88fc5418))
* upgrade to 0.34.1 of sn_api ([41c6d1d](https://github.com/maidsafe/sn_cli/commit/41c6d1d1655485ba9cc915104cdde2919aa7e9a5))


* upgrade sn_api to 0.35.x ([76c0290](https://github.com/maidsafe/sn_cli/commit/76c0290d42c9598b7634ae4d1532cd87273b80eb))

### [0.33.8](https://github.com/maidsafe/sn_cli/compare/v0.33.7...v0.33.8) (2021-08-23)

### [0.33.7](https://github.com/maidsafe/sn_cli/compare/v0.33.6...v0.33.7) (2021-08-09)

### [0.33.6](https://github.com/maidsafe/sn_cli/compare/v0.33.5...v0.33.6) (2021-08-02)

### Features

* **install:** Support Specific Version ([f482dda](https://github.com/maidsafe/sn_cli/commit/f482dda403a73b49965339a518881eff5ec2125f))

### [0.33.5](https://github.com/maidsafe/sn_cli/compare/v0.33.4...v0.33.5) (2021-07-30)

### Features

* **install:** Support Aarch64 in Install Script ([5022064](https://github.com/maidsafe/sn_cli/commit/50220648c599b01f12f2f7cfaf359e9ade05a296))


### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* Remove Use of Fork and Temp Branch ([201271f](https://github.com/maidsafe/sn_cli/commit/201271fd827eda42e933e5e0b66326de7abee876))
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.33.4](https://github.com/maidsafe/sn_cli/compare/v0.33.3...v0.33.4) (2021-07-29)

### Features

* **node:** Support Install of Specific Version ([67cd251](https://github.com/maidsafe/sn_cli/commit/67cd251d1a7e5b84fb89d75b39ac5549aced8e25))

### [0.33.3](https://github.com/maidsafe/sn_cli/compare/v0.33.2...v0.33.3) (2021-07-27)

### Features

* **update:** Flag Argument to Remove User Input ([fba67bb](https://github.com/maidsafe/sn_cli/commit/fba67bbb56b77c2868d3a4ce7c794a248860cc82))

### [0.33.2](https://github.com/maidsafe/sn_cli/compare/v0.33.1...v0.33.2) (2021-07-26)

### [0.33.1](https://github.com/maidsafe/sn_cli/compare/v0.33.0...v0.33.1) (2021-07-20)

### Features

* retry on cat failure to take advantage of local cache ([683905f](https://github.com/maidsafe/sn_cli/commit/683905fc9b7b1e522cce3d2208876ec07e528d7b))

## [0.33.0](https://github.com/maidsafe/sn_cli/compare/v0.32.1...v0.33.0) (2021-07-20)

### ⚠ BREAKING CHANGES

* sn dep breaking change update

### Features

* update safe_network dep to 0.9.x ([1624381](https://github.com/maidsafe/sn_cli/commit/1624381bc630ab8409d76e53d38d32474e265fca))

### [0.32.1](https://github.com/maidsafe/sn_cli/compare/v0.32.0...v0.32.1) (2021-07-16)

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* **deploy:** Only Package Binary File ([5e6b281](https://github.com/maidsafe/sn_cli/commit/5e6b281679ee29152588ab9069ba9f30b2f89e8e))
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### ⚠ BREAKING CHANGES

* removes tokens and wallets and authd

### Features

* update sn dep, remove auth ([08c19b6](https://github.com/maidsafe/sn_cli/commit/08c19b66b4f0f597611a6a22a1e5eaefed58c7a7))

### [0.31.2](https://github.com/maidsafe/sn_cli/compare/v0.31.1...v0.31.2) (2021-07-14)

### [0.31.1](https://github.com/maidsafe/sn_cli/compare/v0.31.0...v0.31.1) (2021-07-01)

### Features

* **args:** add clear_data arg for Join cmd ([c64f34d](https://github.com/maidsafe/sn_cli/commit/c64f34dd9e524b8bc9f0045f404e88e7bed29166))
* **node:** take local and public args for node join ([a4920a6](https://github.com/maidsafe/sn_cli/commit/a4920a6739b784697f8b5832f276b07d630f475a))

## [0.31.0](https://github.com/maidsafe/sn_cli/compare/v0.30.2...v0.31.0) (2021-06-30)

### ⚠ BREAKING CHANGES

* blsstc for bls

### Features

* update to use blsstc sn_api ([b30ed62](https://github.com/maidsafe/sn_cli/commit/b30ed62090c36f96c4e70a7f75dd373bda8b4d12))

### [0.30.2](https://github.com/maidsafe/sn_cli/compare/v0.30.1...v0.30.2) (2021-06-29)

### Features

* add 10 min default timeout ([111f64b](https://github.com/maidsafe/sn_cli/commit/111f64b7d93aae839c95d8143dc665e89995d487))
* override timeout w/ env var ([b904587](https://github.com/maidsafe/sn_cli/commit/b904587ef2965b4ff07ba0e3f3f47f4d3c81b846))
* updates sn_api dep ([1fa58d5](https://github.com/maidsafe/sn_cli/commit/1fa58d5458678f67e4bd61fcfeb5dcef41a1d78d))

### [0.30.1](https://github.com/maidsafe/sn_cli/compare/v0.30.0...v0.30.1) (2021-06-24)

## [0.30.0](https://github.com/maidsafe/sn_cli/compare/v0.29.2...v0.30.0) (2021-06-23)

### ⚠ BREAKING CHANGES

* update sn_api version using safe_network repo uner the hood

### Features

* deps. ([864940c](https://github.com/maidsafe/sn_cli/commit/864940ca403aefd69a04ea9213751cc6512b1f6a))

### [0.29.2](https://github.com/maidsafe/sn_cli/compare/v0.29.1...v0.29.2) (2021-06-17)

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* update launch tool for new log locations ([4213661](https://github.com/maidsafe/sn_cli/commit/4213661b8ee884433359ccc6d2fc8afc829d5c8e))
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.29.1](https://github.com/maidsafe/sn_cli/compare/v0.29.0...v0.29.1) (2021-06-17)

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* tie tokio to 1.6 for stability ([2642c3b](https://github.com/maidsafe/sn_cli/commit/2642c3b7c0cd2925639e49cea77adff46636dfc6))
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### ⚠ BREAKING CHANGES

* update to sn_api v0.29 is a breaking change

* Update dependencies ([459c70d](https://github.com/maidsafe/sn_cli/commit/459c70d1ce7c6974796928cb986d98e0c99b32dc))

### [0.28.1](https://github.com/maidsafe/sn_cli/compare/v0.28.0...v0.28.1) (2021-06-15)

## [0.28.0](https://github.com/maidsafe/sn_cli/compare/v0.27.1...v0.28.0) (2021-06-11)

### ⚠ BREAKING CHANGES

* **deps:** sn_cli dependency update

* **deps:** update to latest dependencies ([6ae87bd](https://github.com/maidsafe/sn_cli/commit/6ae87bdd2f6978163e8b685feb126aea8451c96c))

### [0.27.1](https://github.com/maidsafe/sn_cli/compare/v0.27.0...v0.27.1) (2021-06-07)

## [0.27.0](https://github.com/maidsafe/sn_cli/compare/v0.26.4...v0.27.0) (2021-06-02)

### ⚠ BREAKING CHANGES

* api version updated w/ new messaging

### Features

* api updated ([da02bf5](https://github.com/maidsafe/sn_cli/commit/da02bf58c74d2b5ec81fd6c67593a18220bc191f))

### [0.26.4](https://github.com/maidsafe/sn_cli/compare/v0.26.3...v0.26.4) (2021-05-21)

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* **self_update:** switch self_update to run from new sn_cli repo ([d0efea0](https://github.com/maidsafe/sn_cli/commit/d0efea0584c33a880da37d99517af15aaa0c7958))
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.26.3](https://github.com/maidsafe/sn_cli/compare/v0.26.2...v0.26.3) (2021-05-21)

### [0.26.2](https://github.com/maidsafe/sn_cli/compare/v0.26.1...v0.26.2) (2021-05-21)

### [0.26.1](https://github.com/maidsafe/sn_cli/compare/v0.26.0...v0.26.1) (2021-05-21)

### Features

* **api:** add an example app showcasing how to fetch a Blob from Safe using sn_api ([9098cff](https://github.com/maidsafe/sn_cli/commit/9098cff4c4d2ba15321dd072c970a18781a04a49))
* **sn_cli:** move codebase from sn_api::safe_cli module ([7c9fee9](https://github.com/maidsafe/sn_cli/commit/7c9fee9ebe445871deed02a5f99447525081d060))

### [0.26.0](https://github.com/maidsafe/sn_api/compare/v0.25.3...v0.26.0) (2021-05-06)

* ***api*** update sn_client to 0.54.9

### [0.25.3](https://github.com/maidsafe/sn_api/compare/v0.25.2...v0.25.3) (2021-05-05)

* ***cli*** query each file with a single query rather than in chunks for files get command
* ***api*** update to sn_client 0.54.5

### [0.25.2](https://github.com/maidsafe/sn_api/compare/v0.25.1...v0.25.2) (2021-05-04)

* ***api*** update sn_client to v0.54.4
* ***api*** move safe_url into a separate crate
* ***api*** remove tiny-keccak dependency and use xor_name to calculate sha3 of NRS string

### [0.25.1](https://github.com/maidsafe/sn_api/compare/v0.25.0...v0.25.1) (2021-05-03)

* ***api** add feature gate to app-specific error

### [0.25.0](https://github.com/maidsafe/sn_api/compare/v0.24.0...v0.25.0) (2021-05-03)

* ***cli*** change node_path arg of 'node bin-path' command to node-path
* ***api*** register API takes an address and ownership over data
* ***api*** adding Register and Multimap public APIs
* ***api*** allow immutable reference to fetch method
* ***api*** removed more mut bindings to self methods
* ***api*** allow aliases of Safe (immutable references)
* ***api*** fix set_content_version api in SafeUrl API to update its internal state
* ***cli*** add 'xorurl pk' subcommand which generates the SafeKey XOR-URL for the given public key

### [0.24.0](https://github.com/maidsafe/sn_api/compare/v0.23.3...v0.24.0) (2021-04-21)

### Features

* ***api*** re-enabling dry-run feature for Blob API as well as for CLI commands
* ***api*** re-organising files, nrs and xorurl files into their own mod folders, and renaming XorUrl to SafeUrl module
* ***api*** support transfers to BLS public keys
* ***cli*** adding a new 'keys show' subcommand to display CLI's owned SafeKey, pk, xorurl and sk
* ***cli*** when the --to value of keys transfer command doesn't start with safe:// and cannot be decoded as PK, it now fallbacks to assume it's a URL
* ***cli*** remove keypair cmd and --pk argument from the keys command as they are not necessary anymore


### [0.23.3](https://github.com/maidsafe/sn_api/compare/v0.23.2...v0.23.3) (2021-04-15)

### Features

* ***cli*** add '--for-cli' flag to 'keys create' command which sets the newly created SafeKey for CLI use

### [0.23.2](https://github.com/maidsafe/sn_api/compare/v0.23.1...v0.23.2) (2021-04-15)

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* ***api*** change stack size to 8mb for Windows builds
* ***cli*** change stack size to 8mb for Windows builds
* ***authd*** change stack size to 8mb for Windows builds
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.23.1](https://github.com/maidsafe/sn_api/compare/v0.23.0...v0.23.1) (2021-04-13)

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* ***api*** upgrading sn_client to v0.52.10
* ***cli*** enhance CLI 'networks switch' command message explaining how to restart authd
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.23.0](https://github.com/maidsafe/sn_api/compare/v0.22.0...v0.23.0) (2021-04-08)

### Features

* ***api*** Upgrade sn_client library to v0.52.9

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* ***cli*** node join command was not passing multiple peers addresses correctly to launch tool
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.22.0](https://github.com/maidsafe/sn_api/compare/v0.21.0...v0.22.0) (2021-04-07)

### Features

* ***cli*** Update log format to match sn_node

### [0.21.0](https://github.com/maidsafe/sn_api/compare/v0.20.0...v0.21.0) (2021-03-15)

### Features

* ***cli*** upgrade tokio to v1.3.0 and quinn to v0.10.1

* ***authd*** upgrade tokio to v1.3.0 and quinn to v0.10.1

* ***cli*** customise the error message displayed when a panic occurred

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* ***api*** fix API tests by retrying some operations when network is not fully in sync
* ***cli*** add instructions to CLI User Guide to install VS C++ redistribution package as Windows requirements
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.20.0](https://github.com/maidsafe/sn_api/compare/v0.19.1...v0.20.0) (2021-03-04)

### Features

* ***api*** store serialised NrsMap/FilesMap in a Blob, and only their XOR-URLs in the underlying Sequence of the NrsMapContainer/FilesContainer

* ***api*** adding a README.md file to sn_api crate with a description of the current APIs

* ***cli*** adds bin-version subcommands to node & auth, supressing -V for all subcommands

* ***qjsonrpc*** adds JSON-RPC spec-defined error code constants

### [0.19.1](https://github.com/maidsafe/sn_api/compare/v0.19.0...v0.19.1) (2021-02-23)

### Features

* ***api*** Update to `sn_client` 0.47.1

### [0.19.0](https://github.com/maidsafe/sn_api/compare/v0.18.0...v0.19.0) (2021-02-17)

### Features

* ***cli*** add auth version subcommand which prints out the authd binary version

* ***cli*** command to configure networks in the config by providing a list of IP and ports

* ***cli*** have CLI and authd to set client config path to ~/.safe/client/sn_client.config

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* ***cli*** remove short option used for dry run
* ***cli*** ignore error when listing networks and current network is not set in the system
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.18.0](https://github.com/maidsafe/sn_api/compare/v0.17.2...v0.18.0) (2021-02-04)

### Features

* ***authd*** Prioritise --config over env vars
* ***api*** Invalidate NRS names with troublesome characters
* ***api*** Validate length of NRS name and subname
* ***qjsonrpc*** Add qjsonrpc minimal ping example
* ***api*** Invalidate public names containing slash char


### [0.17.2](https://github.com/maidsafe/sn_api/compare/v0.17.1...v0.17.2) (2021-01-25)

### Features

* ***cli*** Defaults to checking balance of key assigned to CLI
* ***cli*** Update sn_launch_tool dep to get defaults for qp2p idle-timeout and keepalive
* ***api*** sn_client updated to v0.44.15

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* ***api & authd*** Adds a new error for when Map Entry is not found.
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.17.1](https://github.com/maidsafe/sn_api/compare/v0.17.0...v0.17.1) (2021-01-11)

### Features

* ***cli*** Control self_update by a cargo feature

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* ***api*** keypair API now returns a new randomly create Ed25519 key pair
* ***api*** support transfers to Ed25519 public keys in addition to using a Wallet or SafeKey URLs
* ***cli*** fix failing CLI build for MUSL targets
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.17.0](https://github.com/maidsafe/sn_api/compare/v0.16.0...v0.17.0) (2020-12-23)

### Features

* ***cli*** change the default number of nodes launched by `$ safe node run-baby-fleming` command to 11 (eleven nodes): also by PR #660

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* ***api*** known issue in authorising CLI, as reported in last week's dev update, was solved: https://github.com/maidsafe/sn_api/pull/659
* ***cli*** fix `$ safe update` command as it was not looking up in the correct URL: https://github.com/maidsafe/sn_api/pull/660
* ***cli*** install script had an issue for Mac users: https://github.com/maidsafe/sn_api/pull/661
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.16.0](https://github.com/maidsafe/sn_api/compare/v0.15.0...v0.16.0) (2020-12-17)

### Features

* ***api*** re-enable implementation for coins transfers with Ed25519 keys

* ***authd*** insert and retrieve authorised apps keypairs into/from the Safe

* ***api*** reenable use of connect from auth responses

* ***api*** loop seq gets w/ timeout

* ***authd*** adapt authd client api and service names to new terminology of Safe creation and unlocking

* ***authd*** store generated keypairs on the network

* ***authd*** reenable decoding auth reqs and basic app key generation

* ***authd*** setting up IPC for auth

* ***authd*** moving in basics of auth func into the repo

* ***cli*** install script to auto detect latest version of CLI released

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* ***api*** store Blob API was returning the wrong xorname
* ***api*** keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
* ***api*** ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
* ***api*** remove repeated data storing in SafeAppClient::store_sequence
* ***ffi*** fix typo in authorise_app API
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### [0.15.0](https://github.com/maidsafe/sn_api/compare/v0.14.0...v0.15.0) (2020-07-16)

### Features

* ***api*** implements support for storing, retrieving and resolving symlinks ([01c62df](https://github.com/maidsafe/safe-api/commit/01c62dfc1f8d55ad005a67de0aff14eb54516369))

* ***api & cli*** first draft implementation of a Sequence API and CLI commands ([e287d28](https://github.com/maidsafe/safe-api/commit/e287d2838e8a0c11c700b342989afa6e4b829cd3))

* ***api*** migrate public FilesContainers and NRSContainers to use PublicSequence as its native data type ([3d00203](https://github.com/maidsafe/safe-api/commit/3d00203bd4fe073efed8f3f8921f2dd85c98954f)

* ***api & cli*** allow to store, append and retrieve Private Sequence with API and CLI ([9c1a80b](https://github.com/maidsafe/safe-api/commit/9c1a80b1eb57948e08f5c548f318b4cbc36ea365))

* ***cli*** show the native data XOR-URL in the dog output ([9abbecb](https://github.com/maidsafe/safe-api/commit/9abbecb5a909d3e38e471bd758ec6dd1a648151b))

* ***ffi*** expose sequence data APIs from the ffi ([dfc3ca7](https://github.com/maidsafe/safe-api/commit/dfc3ca7aedd892d1497d4c9cc355ad7e08f8e572))

### Bug Fixes

<csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/>
<csr-id-241692b88a6a078682b17a87b37cfd5bd66764f9/>
<csr-id-723b52fba9d536f411dbbb7c62b160dcebde711a/>
<csr-id-859f51cf13372bacc3380bbd37c70a66e27e0927/>
<csr-id-ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7/>
<csr-id-8a77fef8c67178000f86c29e964578b99f83562d/>
<csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/>
<csr-id-497ab1b68b4053e519740335d8e8d28e07ef30f7/>

* ***cli*** XOR-URL of a resolved NRS Container was displaying subnames as part of the output of the dog cmd ([bb9b15c](https://github.com/maidsafe/safe-api/commit/bb9b15cbd252ebd23b34253317535315d3d81f74))
* ***api*** return an error when resolving a URL which contains subnames but targetting content non supporting subnames ([f1a9c60](https://github.com/maidsafe/safe-api/commit/f1a9c600ff05fca1481f13fe51358afe18819d01))
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-acb34489f91a0b327bcf13f68cfa679b41162523/> CLI update command was looking up in the wrong URL
   - Changing the default number of nodes launched by `node run-baby-fleming` command to 11.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 153 commits contributed to the release over the course of 452 calendar days.
 - 124 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - clippy tidyup for rust 1.57 ([`05f6d98`](https://github.com/maidsafe/safe_network/commit/05f6d98cf21f0158f4b5161484c7c15a0561b6f4))
    - cross platform support for cli unit tests ([`256c504`](https://github.com/maidsafe/safe_network/commit/256c504e49121fa0550ae1bcff33f22b8efc78df))
    - unify skip port forwarding arguments ([`3f77429`](https://github.com/maidsafe/safe_network/commit/3f77429e8bd659a5b2e7aa377437fac1b3d709c0))
    - feat(sn_cli): genesis key argument for `node join` ([`f99a418`](https://github.com/maidsafe/safe_network/commit/f99a418a8befcb3baee84b993f4fcc2e23aed396))
    - ignore for bugs that have been identified ([`ae19a45`](https://github.com/maidsafe/safe_network/commit/ae19a45f58bf8d107eea8560af17801d2f619626))
    - minor changes for out of date tests ([`06fda58`](https://github.com/maidsafe/safe_network/commit/06fda580250745abbd8a6a12bab19c05a61f5615))
    - parse different output from `nrs create` ([`578d905`](https://github.com/maidsafe/safe_network/commit/578d9054c668dcf5871cdac26f2c16aa5df13d58))
    - include remote addrs in listerners threads log entries ([`f3d3ab2`](https://github.com/maidsafe/safe_network/commit/f3d3ab2b059040ff08b6239c8a6583c64eac160e))
    - safe_network-0.43.0 ([`c78f470`](https://github.com/maidsafe/safe_network/commit/c78f4703a970e8b7466b091ad331d0f2233aa9a3))
    - safe_network-0.42.0/sn_api-0.43.0 ([`ca21d1e`](https://github.com/maidsafe/safe_network/commit/ca21d1e97fcd28ca351887636affffff78e3aeb3))
    - safe_network-0.41.4/sn_api-0.42.0 ([`8b8a361`](https://github.com/maidsafe/safe_network/commit/8b8a3616673405005d77868dc397bd7542ab3ea7))
    - sn_api-0.41.0 ([`df25e49`](https://github.com/maidsafe/safe_network/commit/df25e4920c570771f6813ca03da02f6dfc8e59fb))
    - safe_network-0.41.2/sn_api-0.40.1 ([`a973039`](https://github.com/maidsafe/safe_network/commit/a973039178af33b859d421cf36571de49cceff17))
    - revert "chore(release): safe_network-0.42.0/sn_api-0.41.0" ([`d8ec5a8`](https://github.com/maidsafe/safe_network/commit/d8ec5a81ae566e8d7068592e01cff4e808b1cad1))
    - safe_network-0.42.0/sn_api-0.41.0 ([`63432eb`](https://github.com/maidsafe/safe_network/commit/63432eb2e528401ae67da8eea0c82837ab42fc18))
    - safe_network-0.41.0 ([`14fdaa6`](https://github.com/maidsafe/safe_network/commit/14fdaa6537619483e94424ead5751d5ab41c8a01))
    - safe_network v0.40.0/sn_api v0.39.0 ([`7001573`](https://github.com/maidsafe/safe_network/commit/70015730c3e08881f803e9ce59be7ca16185ae11))
    - update bls_dkg and blsttc to 0.7 and 0.3.4 respectively ([`213cb39`](https://github.com/maidsafe/safe_network/commit/213cb39be8fbfdf614f3eb6248b14fe161927a14))
    - update sn_api reference style ([`6f5e0a7`](https://github.com/maidsafe/safe_network/commit/6f5e0a767a1c8519abdf06d42c7c958a812011ec))
    - bump rust edition ([`fc10d03`](https://github.com/maidsafe/safe_network/commit/fc10d037d64efc86796f1b1c6f255a4c7f91d3e1))
    - improve test names ([`4f788a3`](https://github.com/maidsafe/safe_network/commit/4f788a31ae7b4a2d602b5141946deacffef64a60))
    - easy to use nrs_add and rigorous nrs_create ([`8787f07`](https://github.com/maidsafe/safe_network/commit/8787f07281e249a344a217d7d5b0e732a7dd7959))
    - nrs get with versions, nrs_map always returned ([`7ffda30`](https://github.com/maidsafe/safe_network/commit/7ffda3021fb36533f22538b1100acfa71b13cd81))
    - add baby fleming to networks list ([`b09e769`](https://github.com/maidsafe/safe_network/commit/b09e769db55aa3f362f49e62d7124d8030ed34bf))
    - unit test node run-baby-fleming command ([`25ad76b`](https://github.com/maidsafe/safe_network/commit/25ad76bc374e461c1df786def45ca79bd1f7484a))
    - remove unused dependencies ([`1fbfc04`](https://github.com/maidsafe/safe_network/commit/1fbfc0444882d2b950be9eca70df2118606db9c3))
    - appease clippy ([`407efd1`](https://github.com/maidsafe/safe_network/commit/407efd15e0b4854864b83ccdb7d2c3adbb0a02e2))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_cli_into_workspace ([`eea8307`](https://github.com/maidsafe/safe_network/commit/eea83074b9bbd334d80b80f12cfcce724d0e8ca3))
    - update cli to use new nrs api ([`8691034`](https://github.com/maidsafe/safe_network/commit/86910340897256bb4df77b6edaa0f2c9584d6dce))
    - Merge branch 'main' into merge_sn_cli_into_workspace ([`c337617`](https://github.com/maidsafe/safe_network/commit/c3376177e095802a7da7d49908314213c169e079))
    - move cli code into sn_cli crate directory ([`a3e0b80`](https://github.com/maidsafe/safe_network/commit/a3e0b805af544205e82ac0c6d2a6e2ed1c55011f))
    - Version change: sn_api v0.26.0; sn_cli v0.26.0; sn_authd v0.8.0 ([`3bcf8ef`](https://github.com/maidsafe/safe_network/commit/3bcf8efcee84c5fb45f5e03ec49d5a623147dc4d))
    - files get command to query each file with a single query rather than in chunks ([`497ab1b`](https://github.com/maidsafe/safe_network/commit/497ab1b68b4053e519740335d8e8d28e07ef30f7))
    - Version change: sn_api v0.25.3; sn_cli v0.25.3; sn_authd v0.7.3 ([`ab68342`](https://github.com/maidsafe/safe_network/commit/ab683420665c54df1ae3dae95055000518b543d1))
    - Version change: sn_api v0.25.2; sn_cli v0.25.2; sn_authd v0.7.2 ([`0282dd6`](https://github.com/maidsafe/safe_network/commit/0282dd6edfce91ac25314bda7b6d87fd1ae621fe))
    - moving out safe_url mod as a standalone sn_url crate ([`5780fd1`](https://github.com/maidsafe/safe_network/commit/5780fd1d6ba480cb775fd66e53e41f02d97b3a94))
    - Version change: sn_api v0.25.0; sn_cli v0.25.0; sn_authd v0.7.0 ([`60717f1`](https://github.com/maidsafe/safe_network/commit/60717f1a09aac06911f01cb3a811731721ae5708))
    - Version change: sn_api v0.25.1; sn_cli v0.25.1; sn_authd v0.7.1 ([`7a8860d`](https://github.com/maidsafe/safe_network/commit/7a8860d71776958fb93e91fefe157b3de4277a8c))
    - change node_path arg of 'node bin-path' command to node-path ([`d91150d`](https://github.com/maidsafe/safe_network/commit/d91150d8f46003cc0fa7813e4ae907b187379de8))
    - fixing multimap_remove and adding multimap_get_by_hash API ([`c5b0f0b`](https://github.com/maidsafe/safe_network/commit/c5b0f0b630f673697367361508a30caf7ad787bd))
    - adding first set of tests for Multimap API ([`6f05016`](https://github.com/maidsafe/safe_network/commit/6f0501699b0d0620a7c9d2b013944f90884ca1c3))
    - Version change: sn_api v0.24.0; sn_cli v0.24.0; sn_authd v0.6.0 ([`47e7f0a`](https://github.com/maidsafe/safe_network/commit/47e7f0aea943a568d767a5226b0d8e71414508bc))
    - add 'xorurl pk' subcommand which generates the SafeKey XOR-URL for the given public key ([`6e700fc`](https://github.com/maidsafe/safe_network/commit/6e700fc9776409e88abd0a2e61e450f400985801))
    - allow to pass a SafeKey URL top 'keys show' command to display its public key ([`ac02f3e`](https://github.com/maidsafe/safe_network/commit/ac02f3ed351298ab86d650ffe95b666530138138))
    - support transfers to BLS public keys ([`4f6c526`](https://github.com/maidsafe/safe_network/commit/4f6c526e194f1b949c1b5b126442150157b7b0ba))
    - re-enabling dry-run feature for Blob API as well as for CLI commands ([`0f2f3c7`](https://github.com/maidsafe/safe_network/commit/0f2f3c74dc81fefbc719e79f41af434023ac0462))
    - re-organising files, nrs and xorurl files into their own mod folders ([`afd5422`](https://github.com/maidsafe/safe_network/commit/afd5422945fd1fc4ac509713e72471076ea4aee0))
    - adding a new 'keys show' subcommand to display CLI's owned SafeKey, pk, xorurl and sk ([`97d1314`](https://github.com/maidsafe/safe_network/commit/97d131435515406eb5a2c93aa9d61d0929e39ba2))
    - remove keypair cmd and --pk argument from the keys command as they are not necessary anymore ([`8a77fef`](https://github.com/maidsafe/safe_network/commit/8a77fef8c67178000f86c29e964578b99f83562d))
    - Version change: sn_cli v0.23.3 ([`241be4a`](https://github.com/maidsafe/safe_network/commit/241be4ab6851ff2de49838851afbd57830801850))
    - add --for-cli flag to keys create command which set the newly created SafeKey for CLI use ([`42de5f0`](https://github.com/maidsafe/safe_network/commit/42de5f0fbb57c7e5d4f98dbbe8bf47bd04dbf9b1))
    - Version change: sn_api v0.23.2; sn_cli v0.23.2; sn_authd v0.5.2 ([`e939702`](https://github.com/maidsafe/safe_network/commit/e939702fdc1986c0021cf12223cbc707589b889f))
    - Version change: sn_api v0.23.1; sn_cli v0.23.1; sn_authd v0.5.1 ([`6aa920e`](https://github.com/maidsafe/safe_network/commit/6aa920e07a42b85ca8d081b8c93e7290553bb7ca))
    - enhance cli doc, how to safe auth restart ([`73d192e`](https://github.com/maidsafe/safe_network/commit/73d192e741f76c874e3998c48fc2d4152d2540b0))
    - update S3 connection info URL ([`a4e6bb9`](https://github.com/maidsafe/safe_network/commit/a4e6bb9646521585e2d166fcbc072f43740d1000))
    - Version change: sn_api v0.23.0; sn_cli v0.23.0; sn_authd v0.5.0 ([`e506e06`](https://github.com/maidsafe/safe_network/commit/e506e06acd50467834e80ebb15a3221261b45752))
    - node join command was not passing multiple peers addresses correctly to launch tool ([`ca0d9eb`](https://github.com/maidsafe/safe_network/commit/ca0d9eb5b103bcd591b55c1ddd3ac779d6a1aef7))
    - pass SecretKey by reference ([`5499aeb`](https://github.com/maidsafe/safe_network/commit/5499aeb2f755ce363b709c5379b860048c92ce5a))
    - Version change: sn_api v0.22.0; sn_cli v0.22.0; sn_authd v0.4.0 ([`fedab1b`](https://github.com/maidsafe/safe_network/commit/fedab1b7bc6c01b8be07ae2c54c034514bc70717))
    - update sn_client and sn_data_types to latest ([`0d4755e`](https://github.com/maidsafe/safe_network/commit/0d4755ed64a65c223bad253d9d7a03980ec12e8d))
    - log format matches sn_node log format ([`fedc64f`](https://github.com/maidsafe/safe_network/commit/fedc64f43e586680933865ec16b4d976b3b68e39))
    - Version change: sn_api v0.21.0; sn_cli v0.21.0; sn_authd v0.3.0; qjsonrpc v0.2.0 ([`838238d`](https://github.com/maidsafe/safe_network/commit/838238d745a18aa28a8b366ab4adc62745656990))
    - upgrade tokio to v1.3.0 and quinn to v0.10.1 ([`d77859a`](https://github.com/maidsafe/safe_network/commit/d77859a8138de0ddcd6b121b928efe13e0254e81))
    - update cli readme s3 node config location ([`49ad9ee`](https://github.com/maidsafe/safe_network/commit/49ad9eecbff02a0455560449faf308165c13e10f))
    - adapting to new Seq type where Policy is immutable ([`85462b6`](https://github.com/maidsafe/safe_network/commit/85462b6fb58f16f4797d7ef2816e96a287af7ad9))
    - customise the error message displayed when a panic occurred ([`45f11ae`](https://github.com/maidsafe/safe_network/commit/45f11ae22df242e229d01bfc5dc2b6ac9de8536d))
    - add visual c++ libs to windows requirements ([`56e076b`](https://github.com/maidsafe/safe_network/commit/56e076b74747ad4e9f82a7df7e82f1c97e2b4496))
    - Version change: sn_api v0.20.0; sn_cli v0.20.0; sn_authd v0.2.0; qjsonrpc v0.1.2 ([`a35ffb7`](https://github.com/maidsafe/safe_network/commit/a35ffb759bafd6e2b03d96bffa62747eb1965c89))
    - Version change: sn_api v0.19.1; sn_cli v0.19.1 ([`edbdcb6`](https://github.com/maidsafe/safe_network/commit/edbdcb62c36a2998aab23dd3a4d0b13bae13b472))
    - update sn_client and data types ([`6c9cf24`](https://github.com/maidsafe/safe_network/commit/6c9cf24df423abae568fab63fc6615d9f7a3df68))
    - Adds bin-version subcommand to node & auth, removes -V for all subcommands ([`318f694`](https://github.com/maidsafe/safe_network/commit/318f6942ac1cd40391b283349bcfa959586422b5))
    - Version change: sn_api-v0.19.0; sn_cli-v0.19.0; sn_authd-v0.1.1; qjsonrpc-v0.1.1 ([`21f4733`](https://github.com/maidsafe/safe_network/commit/21f4733fbc32efd2c822337c7b3f077cca0f2992))
    - adding a step to check for unused dependencies ([`de482a5`](https://github.com/maidsafe/safe_network/commit/de482a5611333d069076d7da1b7c5a6017db65eb))
    - upgrade sn_client to v0.46.12 and most of all dependencies to their latest published version ([`e3c6da3`](https://github.com/maidsafe/safe_network/commit/e3c6da38f92c354c560bd6b555d76f698779ebcf))
    - adds clippy exception for unused result on windows ([`644c1e0`](https://github.com/maidsafe/safe_network/commit/644c1e0d7b2bf346937aa5baf35adab58a49d39e))
    - update tiny-keccak from 1.5.0 to 2.0.2 ([`792bce8`](https://github.com/maidsafe/safe_network/commit/792bce8dd94192f17c51d6a1c0b63c7c214ad7c3))
    - have CLI and authd to set client config path to ~/.safe/client/sn_client.config ([`a836991`](https://github.com/maidsafe/safe_network/commit/a836991edbc0e0394d762525ad49025c5071a2cc))
    - group all config related functions in Config struct with methods ([`cd57437`](https://github.com/maidsafe/safe_network/commit/cd57437baf74af370c07d2be6dd9cd51be6d5f52))
    - upgrade sn_client to v0.46.9 and solve clippy issues ([`b61e837`](https://github.com/maidsafe/safe_network/commit/b61e83716cce00c0ba02f3d50bf060cfc095051a))
    - ignore error when listing networks and current network is not set in the system ([`859f51c`](https://github.com/maidsafe/safe_network/commit/859f51cf13372bacc3380bbd37c70a66e27e0927))
    - add details about the new networks set command ([`304a16d`](https://github.com/maidsafe/safe_network/commit/304a16d443a0347e50e0868057486d1067a37b4a))
    - provide bootstrapping contacts list to sn_client as required by new sn_client API ([`43c675e`](https://github.com/maidsafe/safe_network/commit/43c675ee514aa73fb5192717dae58c97587521e7))
    - command to configure networks in the config by providing a list of IP and ports ([`2a43ca8`](https://github.com/maidsafe/safe_network/commit/2a43ca8fb10dcdbf085890643c673491399b1a8b))
    - add auth version subcommand which prints out the authd binary version ([`e366c87`](https://github.com/maidsafe/safe_network/commit/e366c878da84d2cf051bbc692e6b80c675ef8393))
    - migrating to use anyhow for CLI errors and use thiserror for sn_api error types ([`c2c6716`](https://github.com/maidsafe/safe_network/commit/c2c6716d29e56f387776202dad94ddda9b8fe2b2))
    - remove short option used for dry run ([`723b52f`](https://github.com/maidsafe/safe_network/commit/723b52fba9d536f411dbbb7c62b160dcebde711a))
    - Version change: sn_api-v0.18.0; sn_cli--v0.18.0; sn_authd-v0.1.0; qjsonrpc-0.1.0 ([`fce96bf`](https://github.com/maidsafe/safe_network/commit/fce96bfb00279be41a139a360d1b2eac02d874cf))
    - changes to remove any use of Arc for keypairs and secret keys ([`4ba83c7`](https://github.com/maidsafe/safe_network/commit/4ba83c720fabcace7a2859ad308be5922a6597c0))
    - update sn_client and dts ([`b38d840`](https://github.com/maidsafe/safe_network/commit/b38d840320d65b09ce85db9074f7b7a9487f83df))
    - don't attempt to read authd credentials from env vars if --config was passed ([`88a26d3`](https://github.com/maidsafe/safe_network/commit/88a26d3af44b751d04bbfdddd6fa305bea736939))
    - Authd prioritizes --config over env vars ([`c8f54a1`](https://github.com/maidsafe/safe_network/commit/c8f54a1741271584401218ec939b0277bbca6321))
    - Remove trailing spaces ([`6ad032a`](https://github.com/maidsafe/safe_network/commit/6ad032aff127ce74ec9371cc1dffa8983899288b))
    - do not attempt to retry fetching a Sequence entry if not found the first time ([`2dff02d`](https://github.com/maidsafe/safe_network/commit/2dff02dc71bc3574763906c8592d32bde64337c9))
    - Version change: sn_api-v0.17.2; sn_authd-v0.0.15; qjsonrpc-0.0.10 ([`0e1822c`](https://github.com/maidsafe/safe_network/commit/0e1822cc91f5ca9241d451758077d76554d28b2b))
    - update launch tool dep ([`a46ae88`](https://github.com/maidsafe/safe_network/commit/a46ae886bb041ff46fc69a812f7dad65517dc7f4))
    - default to check balance of key assigned to CLI when no sk is provided to keys balance command ([`7a77ef4`](https://github.com/maidsafe/safe_network/commit/7a77ef4e3f3730d2f033d0365b2446bd560b1e21))
    - upgrade sn_client to v0.44.15 ([`4f89812`](https://github.com/maidsafe/safe_network/commit/4f89812ed5ca3394d2cd7b93e3c79aac2929d11d))
    - Version change: sn_api-v0.17.1; sn_cli-v0.17.1; sn_authd-v0.0.14 ([`3961969`](https://github.com/maidsafe/safe_network/commit/396196997f6d114b01e5b269447b3c4219250f35))
    - update CI/CD to produce musl binary ([`7ec5ed7`](https://github.com/maidsafe/safe_network/commit/7ec5ed71eac3def72967a16f45607ff4f8e03c0a))
    - Corrected clippy warnings ([`c3d2b75`](https://github.com/maidsafe/safe_network/commit/c3d2b7522ed0b3cbdcfdd99d87c7442a281c9561))
    - Control self_update by a cargo feature ([`e4adc68`](https://github.com/maidsafe/safe_network/commit/e4adc688d4125190fd6ee9c61074ce0480197b1b))
    - fix all clippy issues after updating to rust 1.49 ([`67b746f`](https://github.com/maidsafe/safe_network/commit/67b746f607501511c38fe752f64119a12985ab72))
    - fix failing CLI build for MUSL targets ([`241692b`](https://github.com/maidsafe/safe_network/commit/241692b88a6a078682b17a87b37cfd5bd66764f9))
    - minor change to error returned when parsing pk from hex ([`6e4ea36`](https://github.com/maidsafe/safe_network/commit/6e4ea368fdcedb10042b5d8dc94ab02eece47003))
    - support transfers to Ed25519 public keys in addition to using a Wallet or SafeKey URLs ([`c20932f`](https://github.com/maidsafe/safe_network/commit/c20932f5c15fa16ccad907208522b9c9b52bb062))
    - keypair API now returns a new randomly create Ed25519 key pair ([`f2589e0`](https://github.com/maidsafe/safe_network/commit/f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0))
    - return anyhow::Result/Error from all CLI tests ([`f6f0734`](https://github.com/maidsafe/safe_network/commit/f6f07349a7524304b3d6b1c22db65d77be519f4c))
    - remove Error::Unexpected and Error::Unknown errors from API ([`e692bec`](https://github.com/maidsafe/safe_network/commit/e692becbbf09e2500284cb1507916fac56149f02))
    - Version change: sn_api-v0.17.0; sn_cli-v0.17.0; sn_authd-v0.0.13 ([`23365d4`](https://github.com/maidsafe/safe_network/commit/23365d409b1a538b2eb8c5138623a409e45f9601))
    - CLI update command was looking up in the wrong URL ([`acb3448`](https://github.com/maidsafe/safe_network/commit/acb34489f91a0b327bcf13f68cfa679b41162523))
    - updates and enhancements to the User Guide, and to some commands help messages ([`40bcd0f`](https://github.com/maidsafe/safe_network/commit/40bcd0f46dad6177b0052b73393d7789fd559b33))
    - updating CLI User Guide ([`e107e13`](https://github.com/maidsafe/safe_network/commit/e107e1314957053db2d71357450cac65cba52a68))
    - adapt wallet tests and minor refactoring ([`7fb6bd9`](https://github.com/maidsafe/safe_network/commit/7fb6bd96a8bdaaee64592b5dc02596b9f6220165))
    - adapt tests to new transfer costs and minor refactor to transfers errors handling ([`7746947`](https://github.com/maidsafe/safe_network/commit/774694795114dc392db5219393fa63f204fcc905))
    - re-enable implementation for coins transfers with Ed25519 keys ([`b2e3faf`](https://github.com/maidsafe/safe_network/commit/b2e3faf9b0943ec779b1e513c76179048dbb0db3))
    - keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey ([`5a30bf3`](https://github.com/maidsafe/safe_network/commit/5a30bf331242ba8dd9b3189dc255b134fdf24587))
    - ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes ([`01cc289`](https://github.com/maidsafe/safe_network/commit/01cc2894b37908377eb822a826f46c7fef39347e))
    - remove unwrap instances from prod and test code ([`422547f`](https://github.com/maidsafe/safe_network/commit/422547f9081de77538f2241c727ac55b00e1e48b))
    - properly serialise key pairs in CLI commands output ([`ae026bb`](https://github.com/maidsafe/safe_network/commit/ae026bb9ce91b1373b8b300c41bfef0c3f295c7a))
    - minor reorganisation to cli test scripts ([`f9e0729`](https://github.com/maidsafe/safe_network/commit/f9e07293ea1f8cd5e4428d95a299ba06c0f30a20))
    - minor renamings in authd status report with new terminology ([`5ac36cc`](https://github.com/maidsafe/safe_network/commit/5ac36cc64566561f4d442058c91b9857622e6f26))
    - adapt to latest sn-client api changes and further simplification of auth messages ([`b6eddcb`](https://github.com/maidsafe/safe_network/commit/b6eddcbf5d272e6a4430cfd6488f5236bef92a5d))
    - update credentials location ([`7b6445a`](https://github.com/maidsafe/safe_network/commit/7b6445a5b9903b1704c45759878bced097bcb82c))
    - simplify authd messages format and serialisation ([`e814ff3`](https://github.com/maidsafe/safe_network/commit/e814ff3b8c58ae7741938a1c73a22c87ed602883))
    - fix lint issues ([`9c3adff`](https://github.com/maidsafe/safe_network/commit/9c3adffd0889f045ac19110072a194072d294705))
    - adapt authd client api and service names to new terminology of Safe creation and unlocking ([`58ecf7f`](https://github.com/maidsafe/safe_network/commit/58ecf7fbae95dee0a103ce39d61efaac6e2cf550))
    - fix typos and clarify posix vs fuse in readme ([`d19d57a`](https://github.com/maidsafe/safe_network/commit/d19d57a2f369cb79fe5ac0c755f57b4005535540))
    - add notes for sn_fs ([`f5c3106`](https://github.com/maidsafe/safe_network/commit/f5c3106834b6e0033adf19ea631e8d2fc5c2ed1e))
    - setting up for no ClientId ([`b5a6d81`](https://github.com/maidsafe/safe_network/commit/b5a6d8115ad3975a17dd973430480adf6c483490))
    - Batch of changes for sk handling ([`29a978f`](https://github.com/maidsafe/safe_network/commit/29a978f3047464ad8014817e331218372b53c06c))
    - converting to more generic data types for keypair sk pk ([`dfabea0`](https://github.com/maidsafe/safe_network/commit/dfabea0a26f97f420f47ba314cae0882aae47dca))
    - clippy ([`106407e`](https://github.com/maidsafe/safe_network/commit/106407e8125cc003794ba6249158aa1a655d3357))
    - reenable decoding auth reqs and basic app key generation ([`3f23687`](https://github.com/maidsafe/safe_network/commit/3f23687471b846e3ad1e2492c237a21f212b753f))
    - tidying up ([`4905fae`](https://github.com/maidsafe/safe_network/commit/4905fae6259063411c5e4ef5fd2afb531980630c))
    - fix merge bugs and readd some shell completion logic ([`b99e7de`](https://github.com/maidsafe/safe_network/commit/b99e7dee3e72e703b47888e3ff03d2baa933b408))
    - Merge branch 'master' into ExploreUpdatesForApis ([`34f9bc7`](https://github.com/maidsafe/safe_network/commit/34f9bc704f301ac903f768813fbd4140cd702f21))
    - further ffi cleanup ([`d5c1cd2`](https://github.com/maidsafe/safe_network/commit/d5c1cd2808f9844b06b846ec10dfe05146137023))
    - chore(mock/ffi): remove mock/ffi builds + files ([`8b9b481`](https://github.com/maidsafe/safe_network/commit/8b9b481df5d124857abb02158739a6ded8f02af7))
    - reenable wallet apis ([`873fe29`](https://github.com/maidsafe/safe_network/commit/873fe29ac9042b7ad28a29630d2c048bde3a7634))
    - use dirs_next for dir finding ([`426158f`](https://github.com/maidsafe/safe_network/commit/426158fcbb6d7c1fe44755c138bba1ac825a0a0c))
    - getting tests compiling ([`532aed1`](https://github.com/maidsafe/safe_network/commit/532aed1ed8e6b3957627ff2cc2f9d10d87fe3cb2))
    - reenabling some authd functionality ([`5a1cd27`](https://github.com/maidsafe/safe_network/commit/5a1cd2790b159e35c734dfb1fe64a43ea4409dfc))
    - reenabling some money apis ([`0a5c18d`](https://github.com/maidsafe/safe_network/commit/0a5c18d115820f7124050bc0a246503b5cc63fd9))
    - setting up IPC for auth ([`b994b8d`](https://github.com/maidsafe/safe_network/commit/b994b8d6ec1fcfc540e91aa9df79ba849aee7647))
    - sn_data_type updates ([`b863e7e`](https://github.com/maidsafe/safe_network/commit/b863e7eb299472b0c9dbd633b1b892cc221efb46))
    - safe_nd -> sn_data_types ([`4466c48`](https://github.com/maidsafe/safe_network/commit/4466c48a3fcec76f6c90cf6fcf1f28b177978c90))
    - update to reference renamed sn_node crate/repo ([`ee05ed3`](https://github.com/maidsafe/safe_network/commit/ee05ed31cb12d8e1d8bac7569beec90db52a5840))
    - update to reference renamed sn_app ([`9651140`](https://github.com/maidsafe/safe_network/commit/96511403687f23516658f1a4fab1b6c6ab3fab45))
    - rename artifacts and paths to match new naming convention ([`e389ab2`](https://github.com/maidsafe/safe_network/commit/e389ab24f2186fc515b115e736a06d20756ae031))
    - update s3 bucket name to sn-api ([`67e6ce1`](https://github.com/maidsafe/safe_network/commit/67e6ce1190ec1def43d4d2437456d985b5c07642))
    - update safe-cmd-test-utilities name to ([`8f309da`](https://github.com/maidsafe/safe_network/commit/8f309dada1517afa10c263a52f5597429f764890))
    - update safe-authd crate name to sn_authd ([`019370c`](https://github.com/maidsafe/safe_network/commit/019370cfd0ace44c656caf45c17248f2a547dbbf))
    - update safe-cli crate name to sn_cli ([`70c67c7`](https://github.com/maidsafe/safe_network/commit/70c67c749c504ddd552aba6663109d2b1839082a))
</details>

## 0.14.0 (2020-06-11)

### Features

* ***cli*** allow vault join command to join currently setup network if no network name provided ([f89642b](https://github.com/maidsafe/safe-api/commit/f89642b0def681dc8df2581e24bef3351a867eaf))

* ***cli*** support for passing a network name to vault join command ([e70812d](https://github.com/maidsafe/safe-api/commit/e70812d36ae139211e055d4fc22a827c7e181dec))

* ***cli*** use maidsafe nlt master branch ([f2df7f5](https://github.com/maidsafe/safe-api/commit/f2df7f5ad6a96305de72b7441de5cfbe8022fe59))

* ***cli*** support multiple hcc in vault join command ([d65e6fd](https://github.com/maidsafe/safe-api/commit/d65e6fd02c4529f5aaaf272d3fe7d0c8399bd031))

* ***cli*** run a vault and join a network ([92384cb](https://github.com/maidsafe/safe-api/commit/92384cb712c8f5ca14dbc1e11fec1265552f1a23))

* ***cli*** test safe-cli against phase-2b vaults ([01f3d89](https://github.com/maidsafe/safe-api/commit/01f3d89099b1792957ceebd9b856a03f973b58e1))

* ***api*** add code coverage for safe-api to CI ([32e1756](https://github.com/maidsafe/safe-api/commit/32e17565b292f3fd446cbead34813f4478df77b1))

### Bug Fixes

* ***cli*** solved self_update conflict wth async when running vault install cmd ([d3cabdd](https://github.com/maidsafe/safe-api/commit/d3cabdd8ca7743eacb9cd741d1be54b730fa3be2))

## 0.13.0 (2020-06-01)

### Features

* ***api*** Adapt codebase to use the newly exposed safe_client_libs async API ([508429b2](https://github.com/maidsafe/safe-api/commit/508429b24a79ec61c40dc7ce95a1334953e776db))

* ***cli*** files put/sync commands now include files that begin with '.' (a.k.a. hidden on Unix). Also, we now omit sub-dirs if recursive flag is not present ([e417cb91](https://github.com/maidsafe/safe-api/commit/e417cb9173149afbc1a41391f9c392e7a23f2118))

* ***cli*** Support empty directories and lays groundwork for symlinks and hidden files. ([284db242](https://github.com/maidsafe/safe-api/commit/284db2427560f83f7c96e5bf4172993802687db1))

* ***ffi*** Refactor files API to return FilesMap object. ([d5de80c4](https://github.com/maidsafe/safe-api/commit/d5de80c4cc77a2b4d97f702633273b979f566a07))

* ***api*** Remove parse_and_resolve_url from the public safe-api API. ([89312c07](https://github.com/maidsafe/safe-api/commit/89312c07be4481be1ec07ab67cf4f354b90e24ce))

* ***api*** Return the complete resolution chain from the inspect API. ([84f57da6](https://github.com/maidsafe/safe-api/commit/84f57da6d9c9384901195824bf69533a024a96af))

* ***cli*** Show complete URL resolution chain from dog command. ([3f98b4be](https://github.com/maidsafe/safe-api/commit/3f98b4be0bd40e8e045efccf2308dc082ea1e87e))

* ***api*** Significant reworking of SafeUrl and its API. ([85941f1b](https://github.com/maidsafe/safe-api/commit/85941f1b7080008eb1ffe6d7d408588a1baac33d))

* ***api*** Rename SafeUrl properties to match changes in rfc-0052 PR: https://github.com/maidsafe/rfcs/pull/356. ([326cfccd](https://github.com/maidsafe/safe-api/commit/326cfccd433441d602070d04e4e5a3843803131e))

* ***api*** Use full_name(), public_name(), sub_names() in SafeUrl instead of name(), top_name(), sub_names().  also improvements and additional tests. ([606be860](https://github.com/maidsafe/safe-api/commit/606be8606f7e86724ede1040d0882ac668946213))

* ***ffi*** Expose vec<string> as string[] in FFI. ([f531ea38](https://github.com/maidsafe/safe-api/commit/f531ea380d4ab5eb6d4e093120b417f9a2997e86))

* ***api*** Parse query params in any type of URL and keep them as part of SafeUrl info. ([3efdacc6](https://github.com/maidsafe/safe-api/commit/3efdacc684a4574b3e99adf59025688c254fa5ba))

* ***test*** First version of cmd-test-utils crate. ([81284c43](https://github.com/maidsafe/safe-api/commit/81284c4381d4fcf27531ee28a1dce3cd91e4bd7b))

### Bug Fixes

* ***cli*** Addresses issue #510, 'files ls' now uses NRS paths ([eae5c9ee](https://github.com/maidsafe/safe-api/commit/eae5c9eef9ee867ccb4bb63c8dda19b96d89dde8))
* ***nrs*** Subnames were not properly resolved if they contained nested subnames ([d8654602](https://github.com/maidsafe/safe-api/commit/d8654602d632f89f57e5b0b8f1f21ee857bbdc96))
* ***cli*** files get was not resolving URLs with more than one resolution step and paths. ([66018ac0](https://github.com/maidsafe/safe-api/commit/66018ac01716be18c99f8ca781ba0e4ea59f595e))
* ***ffi*** Return `null` when value is `None`. ([b5d54cba](https://github.com/maidsafe/safe-api/commit/b5d54cbaefcca3515d02fb2f89545cffc9a1df4e))
* ***ffi*** Remove ffi panic when media_type has no value. ([69d7ce57](https://github.com/maidsafe/safe-api/commit/69d7ce57bcd1bfa34b8de67ac32fda6be33c4f14))
* ***cli*** Remove trailing / from filename when syncing or adding files to container. ([9098b075](https://github.com/maidsafe/safe-api/commit/9098b075e28d751abd148540550069a275bc1229))

## 0.12.0 (2020-04-14)

### Features

* ***cli*** implements 'safe files get' subcommand to copy a remote FilesContainer onto a local path ([361be1fe](https://github.com/maidsafe/safe-api/commit/361be1fe1da78e526f6968e28e34c5b05ef88de0))

* ***authd*** launch authd as a detached process for all platforms using native Rust utilities, plus properly format authd log entries ([605aa9fc](https://github.com/maidsafe/safe-api/commit/605aa9fcd6bbf8a766d98c52b028947f860a438d))

* ***bench*** Setup basic benchmarking. Adds safe vault run-baby-fleming -t command for simpler test setup ([7673d6f6](https://github.com/maidsafe/safe-api/commit/7673d6f6a764f05dd65aa86501fece99671dbd19))

* ***ffi*** expose set_config_dir_path and app_is_mock API ([a3d63952](https://github.com/maidsafe/safe-api/commit/a3d639525c9b027392d6226a8cc0388791a5d72b))

* ***ffi*** expose IPC and logging APIs in safe-ffi ([865fd927](https://github.com/maidsafe/safe-api/commit/865fd927f0713e9797780741f809a489f2c9a59f))

### Bug Fixes

* ***api*** verify in SafeUrl::from_url that the URL provided has the minimum length to be valid before processing it ([2554d738](https://github.com/maidsafe/safe-api/commit/2554d738410436f534685160f73eb32d277cf679))
* ***cli*** info and warning about missing/invalid credentials are now being logged rather than sent to stdout ([73959656](https://github.com/maidsafe/safe-api/commit/739596560384196b9bebcb68ec9da317080ee834))
* ***cli*** connect with read-only access if CLI fails to connect with existing credentials ([54c8d87a](https://github.com/maidsafe/safe-api/commit/54c8d87a355d008440d81aba62863327ee4c4d64))
* ***cli*** create folder for storing network conn info if path doesn't exist ([dc583aad](https://github.com/maidsafe/safe-api/commit/dc583aad07e240ea901a729d0dc1ae14064aab9d))

## 0.11.0 (2020-03-26)

### Features

* ***ffi:*** add files_contains_remove_path API ([a0b2f9c8](https://github.com/maidsafe/safe-api/commit/a0b2f9c8b90ceb0d6b9dbadcebd6992e45ac87e2))


## 0.10.1 (2020-03-20)

### Bug Fixes

* ***api:*** don't spawn the command when invoking authd executable ([f1867af0](https://github.com/maidsafe/safe-api/commit/f1867af021622085c6748951519aa786d9edd322))

## 0.10.0 (2020-03-17)

### Features

* ***api:*** add range based request for the immutable data ([469f47f](https://github.com/maidsafe/safe-api/commit/469f47f5703538716066723faf3c06bf8f170c41))
* ***cli:*** implements 'safe files tree' subcommand ([4d0acb8](https://github.com/maidsafe/safe-api/commit/4d0acb89f3b64f1bae3417d8e7faba6d0a249fdc))
* ***api:*** make the APIs which need to operate with the network to be async ([ad3c655](https://github.com/maidsafe/safe-api/commit/ad3c655cf3932f25093716355837296dab6e030a))
* ***cli:*** create an installation script, and add install instructions to UserGuide ([5eb893a](https://github.com/maidsafe/safe-api/commit/5eb893a8ec5010ed394f0fe6a449992e8e4d050d))
* ***api:*** support for creation of empty FilesContainers with files_container_create API ([3684c22](https://github.com/maidsafe/safe-api/commit/3684c223d6826f99d1860f11ce367cae6ef94539))
* ***ffi:*** adapt bindings to support change in api for creation of empty FilesContainers with files_container_create API ([3684c22](https://github.com/maidsafe/safe-api/commit/3684c223d6826f99d1860f11ce367cae6ef94539))
* ***cli:*** implement a vault update command ([2bab68a](https://github.com/maidsafe/safe-api/commit/2bab68a51decb097ba9697ea7c3e84aa504562da))


### Bug Fixes

* ***cli:*** document 'files tree' subcommand, improve bash setup one liner ([94ff08b](https://github.com/maidsafe/safe-api/commit/94ff08b5954c56cdfd685e50fdf1e1c1746f66c5))
* ***api:*** refactor fetch api to be non-recursive to support native Rust async ([872549d](https://github.com/maidsafe/safe-api/commit/872549dcfa8c40c6d7fbbb8793e6d86afeee26f1))
* ***cli:*** use tokio macros for async tests (tokio::test) and for cli main function (tokio::main) ([cbe39b3](https://github.com/maidsafe/safe-api/commit/cbe39b30f6382bbc19b03d3dac7213ec8e5dcf24))
* ***cli:*** clarify CLI readme commands and fix typos ([649cfc3](https://github.com/maidsafe/safe-api/commit/649cfc3593cd202c4065bdc9eb2a82e07c55f3db))
* ***cli:*** add config file and env variable auth login details ([0a50300](https://github.com/maidsafe/safe-api/commit/0a5030089155c4336f52c486de887dbb7a683cd8))
* ***cli:*** Update auth login --help text to include format ([a40c42f](https://github.com/maidsafe/safe-api/commit/a40c42faedfe856d322e854624aa5ba2c3edd790))
* ***cli:*** remove addition of ./ ([92df716](https://github.com/maidsafe/safe-api/commit/92df716b71d9e1fab8dd633fb5f575f97d507048))
* ***cli:*** use async_std to block main instead of tokio::main as it conflicts with self-update when it creates a runtime ([1a7bc5c](https://github.com/maidsafe/safe-api/commit/1a7bc5ca7d1ac9c6094b6916ecdd8d952c35c670))

## 0.9.0 (2020-03-03)

### Features

* **authd:** change default authd log dir from /tmp to ~/.safe/authd/logs ([8b0eb0e](https://github.com/maidsafe/safe-api/commit/8b0eb0ecd13bf5fe24ed37f4ac466d7012fd6a70))
* **files:** implementation of an API and cli command to remove a file or path from a FilesContainer ([11cfbc6](https://github.com/maidsafe/safe-api/commit/11cfbc6b3aee0ced21acbd8a4319889e92e2bcb7))
* **cli:** allow to set the interval in secs to be used between launching each vault when running a local network ([1492fa7](https://github.com/maidsafe/safe-api/commit/1492fa756597f830e33de5738ae6bfd04386bee0))
* **UserGuide:** add a section with details of the new vault commands ([d7f3703](https://github.com/maidsafe/safe-api/commit/d7f3703be347b0b20339e931da307fcb18f2d1a4))
* **cli:** allow to set vault's verbosity when running it ([ea1914e](https://github.com/maidsafe/safe-api/commit/ea1914ec629125eab9591dd6db2593aa5d479737))
* **cli:** add a vault subcommand to shutdown all running safe_vault processes ([f5d8629](https://github.com/maidsafe/safe-api/commit/f5d8629a21366f540feabe286ebea68aaba506a7))
* **cli:** implement vault subcommand to run a local single-section network ([10cee6](https://github.com/maidsafe/safe-api/commit/10cee65a6fb5fe8be47296a4151f7e8ead76347b))
* **cli:** implement vault subcommand to install vault binary in the system at '~/.safe/vault/' ([8554571](https://github.com/maidsafe/safe-api/commit/8554571afa04d365eb3b7a555445402170398525))
* **files:** when attempting to upload a file to the same target path, report diff error messges depending if the content is the same or different to the existing one on the network ([5125917](https://github.com/maidsafe/safe-api/commit/5125917f74adc62c363062f94b292eec0e669e50))
* **files:** use xorurl (instead of file size/type) to decide if a file needs to be uploaded on a FilesContainer on a sync operation ([9a6ccea](https://github.com/maidsafe/safe-api/commit/9a6ccea65ae43ba4288706a8263b774f316a179e))
* **jsonrpc-quic:** expose JSON-RPC request/response objects hiding serialisation/deserialisation logic, and make QUIC endpoints API cleaner ([79e673a](https://github.com/maidsafe/safe-api/commit/79e673a51c98ad424f88729148e2564668ce5443))
* **authd:** upgrade quinn to v0.5.3, sn_data_types to v0.8.0, self_update to v0.12.0, and migrate to use async/await in authd impl ([33f84e7](https://github.com/maidsafe/safe-api/commit/33f84e7f8ff52d0751e5ac57809dd7d3adcee44d))


### Bug Fixes

* **cli:** download vault binaries from S3 for install cmd, and pick musl package for linux platforms ([83b5868](https://github.com/maidsafe/safe-api/commit/83b5868cca30ca3964af13db740b239b32fc9da3))
* **shell:** add missing commands to the interacive shell ([2a27ed](https://github.com/maidsafe/safe-api/commit/2a27ede6f7f8f8aa6a283b1df94ebff71b608d4c))
* **ci:** enable the Hardened Runtime for Mac notarisation ([6cf0aa](https://github.com/maidsafe/safe-api/commit/6cf0aa67879b1398c2365c07d8cc4f1e54cfe2de))
* **ci:** publish dev libs to S3 needed for mobile testing ([ac5b0cb](https://github.com/maidsafe/safe-api/commit/ac5b0cb226db02b18d28b1047a8daa1f37ed9d03))

## 0.8.1 (2020-01-29)

### Bug Fixes

* **authd:** properly parse account credentials removing enclosing string quotes ([da8c593](https://github.com/maidsafe/safe-api/commit/da8c593ff12c325d377dfd7a6678dd85306df003))
* **release:** authd zip files checksums were incorrectly set in release description ([b885c8e](https://github.com/maidsafe/safe-api/commit/b885c8e0c4fadaec6b7671e3d70c23aeb904ae3a))

## 0.8.0 (2020-01-29)

### Features

* **authd:** don't try to register authd service if there is one already registered ([15753ef](https://github.com/maidsafe/safe-api/commit/15753efe2866a284553ff69f80397b8adcc7e649))
* **cli:** add files ls command ([3ec8143](https://github.com/maidsafe/safe-api/commit/3ec81437bfbd8e946038b7c872a565971de04839))
* **cli:** show login status in authd status report as Yes/No instead of true/false ([6b080e9](https://github.com/maidsafe/safe-api/commit/6b080e941b326735b6a73ff05088eb129f9ee994))
* **cli:** xorurl decode command ([#399](https://github.com/maidsafe/safe-api/issues/399)) ([7e396b5](https://github.com/maidsafe/safe-api/commit/7e396b54ed58de64cfcc378789420da2ff4af817)), closes [#396](https://github.com/maidsafe/safe-api/issues/396)


### Bug Fixes

* **fetch:** Attempting to fetch felative asset from ID now fails. ([b49ec5d](https://github.com/maidsafe/safe-api/commit/b49ec5dcf3d37c346cacb945124219bf6689db2f)), closes [#403](https://github.com/maidsafe/safe-api/issues/403)
* **update:** set execution permissions to binary after installed/updated ([d4bc7bb](https://github.com/maidsafe/safe-api/commit/d4bc7bbfcccabbcf51c0fd33bfb6e89b81567f03))

## 0.7.2 (2020-01-23)

### Bug Fixes

* **ci:** set release tag to be the cli version ([e467084](https://github.com/maidsafe/safe-api/commit/e467084f949eed10c2ebb27c3cb873cc252e228e))
* **ci:** upload same release assets to S3 only if their size changed ([aa236c2](https://github.com/maidsafe/safe-api/commit/aa236c2100bf8a0a865010cd2af39bfad109cfe1))

## 0.7.1 (2020-01-22)

### Bug Fixes

* **cli:** make sure target installation path exists before installing authd ([4886ef0](https://github.com/maidsafe/safe-api/commit/4886ef00ad07e5baca76db28bfd9772f04b23a90))

## 0.7.0 (2020-01-20)

### Features

* **cli:** add 'setup completions [shell]' for dumping shell completions ([217abed](https://github.com/maidsafe/safe-api/commit/217abed1393d5eab1333d28fcb177cd380589c6d))
* **cli:** allow to pass authd executable path in auth commands ([d5e4260](https://github.com/maidsafe/safe-api/commit/d5e42601334cb0102c0be2cd0ff0328314f5443d))
* **cli:** command to install safe-authd from safe-cli ([7387074](https://github.com/maidsafe/safe-api/commit/738707493fa4a337ae835fb1c46a77ed43a8ab8c))
* **cli:** command to update safe-authd from safe-cli ([71e6be8](https://github.com/maidsafe/safe-api/commit/71e6be8ea7d6a932084def858161385d2ff1a197))
* **cli:** connect with read-only access when no credentials were found ([298a621](https://github.com/maidsafe/safe-api/commit/298a62114cf929ca2d5634e2f92daaee93114df6))
* **cli:** enable reading file data from stdin for 'safe files add' ([d36cb54](https://github.com/maidsafe/safe-api/commit/d36cb54dd889bbaf2d98f3d35ac06bb7fa27986a))
* **connect:** use connect API for unregistered client using unregistered auth response ([adb6fba](https://github.com/maidsafe/safe-api/commit/adb6fba5ba28b5df58bca2be569bad881bdf6f3f))
* **ffi:** add auth_app function for app authentication ([bbd233d](https://github.com/maidsafe/safe-api/commit/bbd233d3b4d564a432ebef70338190f2b1b52217))
* **files:** expand dry-run to files xorurl and expose top lovel command for convenience ([2484cca](https://github.com/maidsafe/safe-api/commit/2484cca324fc1fffe359e871a457bcc7d0505037))


### Bug Fixes

* **authd:** minor change to adapt to self_update api change ([b7bf05b](https://github.com/maidsafe/safe-api/commit/b7bf05b138c706600481c7e1225f58cc9c01047d))
* **authd:** use S3 as backend server instead of GitHub for updating to new releases ([19af3e8](https://github.com/maidsafe/safe-api/commit/19af3e8c022fe0baae95aeda73944c93d8d7c3c8))
* **ci:** minor corrections to test and release notes scripts ([8af6384](https://github.com/maidsafe/safe-api/commit/8af6384f5fcd294b5a138ed708e1848353a5fe38))
* **ci:** update deploy file name ([aadcecb](https://github.com/maidsafe/safe-api/commit/aadcecbfe6e10f6bc967b92a86a333aa0064a407))
* **cli:** remove cached network connection information when it's removed from config ([034e71f](https://github.com/maidsafe/safe-api/commit/034e71fc9ef900de97f2722b8c284aab61e789b9))
* **cli:** throw error if arg read from stdin is not a valid utf8 string ([e608a0a](https://github.com/maidsafe/safe-api/commit/e608a0a2a5df1f13742fb87a818726cdd4d8b53c))
* **clippy:** fix clippy issues ([60f227c](https://github.com/maidsafe/safe-api/commit/60f227cfb70d568144beef206397f9129fce1584))
* **UserGuide:** minor enhancements and corrections ([052105b](https://github.com/maidsafe/safe-api/commit/052105bf73ab8f554df094e426529c7b1a5b0094))

## 0.6.0 (2019-12-03)

### Features

* **api:** add APIs to start/stop/restart authd ([cbf6ae9](https://github.com/maidsafe/safe-api/commit/cbf6ae904c3e96e66c23b6e1e8744bf2b874281f))
* **api:** additional subscribe API which automatically launches a QUIC endpoint to receive auth reqs notifications from authd ([10cdcd4](https://github.com/maidsafe/safe-api/commit/10cdcd492ab3b84bf17fe2eff18b229b03dc6212))
* **api:** additional subscribe API which automatically launches a QUIC endpoint to receive auth reqs notifications from authd ([9f63fd3](https://github.com/maidsafe/safe-api/commit/9f63fd326147a839152709384d10c28f90c47429))
* **api:** expose a Safe::default() which uses default value for XOR URL base encoding ([7d45947](https://github.com/maidsafe/safe-api/commit/7d4594704ae16f7bdacd03da32d48a31a42bdf69))
* **api:** first draft implementation of safe-authd client APIs and safe-cli commands ([dc71315](https://github.com/maidsafe/safe-api/commit/dc713150de63417341f793a33ae612bf6c3a9e81))
* **api:** first draft implementation of SafeAuthenticator API and using it from authd to expose the first set of auth services ([0480f3b](https://github.com/maidsafe/safe-api/commit/0480f3b1b6611edf729be4ab012bbd97f020238f))
* **api:** return a PendingAuthReqs type when retrieving the list pending auth reqs fom authd ([5ecb082](https://github.com/maidsafe/safe-api/commit/5ecb0824aaa998ee51567f66cbddcf36332f834c))
* **authd:** allow to receive endpoint certificate in the subscription request ([3a0a9b6](https://github.com/maidsafe/safe-api/commit/3a0a9b6da30b088ae434839b6f217c8666f5bfc8))
* **authd:** automatically clear up the list of pending auth reqs when the user logs out ([48922a2](https://github.com/maidsafe/safe-api/commit/48922a2b564b99362625fe747ad5ccb60777955e))
* **authd:** clone the list of notif endpoints so it doesn't lock the mutex while sending notifications ([c553d92](https://github.com/maidsafe/safe-api/commit/c553d9221d7f8edaf78f2a5282a51bdb3731aac8))
* **authd:** expose a service to retrieve an status report from authd ([f7ea7c9](https://github.com/maidsafe/safe-api/commit/f7ea7c94ace454e5ccb49ec85be3689affc94869))
* **authd:** first draft implementation of allow/deny/subscribe APIs for auth reqs keeping reqs and subscriptions within authd ([d16117d](https://github.com/maidsafe/safe-api/commit/d16117d82a3cb7293454940236247d39c29407c7))
* **authd:** first draft implementation of authd binary exposing QUIC end point ([7ea759f](https://github.com/maidsafe/safe-api/commit/7ea759f5356ce337143664d9d0e366303f67f807))
* **authd:** first draft implementation of sending auth req notifications to subscribed endpoints ([05c8a74](https://github.com/maidsafe/safe-api/commit/05c8a741a9acd2bd43cf40969a55b37514d924cf))
* **authd:** first draft implementation of stop and restart authd, and adding commands to CLI to also start/stop/restart authd ([cfbf6e3](https://github.com/maidsafe/safe-api/commit/cfbf6e3d179f26bb594442fdd2b938c99d0c287d))
* **authd:** implement json-rpc format for the authd requests ([07adb3a](https://github.com/maidsafe/safe-api/commit/07adb3aeed7bdd3446756ef8b82ca64603252a30))
* **authd:** make sure auth reqs eventually time out and removed from the internal queue ([2ed1839](https://github.com/maidsafe/safe-api/commit/2ed183995b8f1d79713b09d5a0aef018b8fadf10))
* **authd:** reject notif subscription if the endpoint URL is already subscribed ([1cc71b4](https://github.com/maidsafe/safe-api/commit/1cc71b44f199d91f5ddc18177848e89edb06c1a9))
* **authd:** support to start/stop authd from CLI in Windows and realise of correct authd and vault certificates paths ([23d9fe1](https://github.com/maidsafe/safe-api/commit/23d9fe11e9284d408a836df77e8592ce7161a46a))
* **authd:** use json-rpc for authd notifications format. Move json-rpc and quic client functions to its own crate ([a11fe71](https://github.com/maidsafe/safe-api/commit/a11fe71f7e0f3ba01c3dd02dd406220e3c675221))
* **authd:** use ProgramData as the folder to share QUIC certificates ([9781076](https://github.com/maidsafe/safe-api/commit/9781076fa6849ef6370c67ecadd3dc9d6ecdf00c))
* **authd:** Windows support to run authd as a service ([b51a696](https://github.com/maidsafe/safe-api/commit/b51a69635b1378fe356af41d770a6af8e31bd072))
* **authd-api:** automatically send an unsubscribe request to authd when dropping an authd client API instance ([a5eff57](https://github.com/maidsafe/safe-api/commit/a5eff572dc11b1305fbbf9e8791c8b75b0bbe745))
* **authd-api:** implement json-rpc for receiving and responding to authd notifications ([aac18e6](https://github.com/maidsafe/safe-api/commit/aac18e608bca2218bf618f7a581f354508b9d396))
* **authd-api:** return a full AuthReq instance in the auth req notifications callback ([d969f3a](https://github.com/maidsafe/safe-api/commit/d969f3adda3b3ec0df47e819113fcfc4009cdc21))
* **cli:** add command to check current network set up in the system ([ff78239](https://github.com/maidsafe/safe-api/commit/ff782394c8b7b9feea703a0e8f2c60402f0cb52a))
* **cli:** allow to find authd in PATH if neither a path to authd client api was passed, nor SAFE_AUTHD_PATH was set ([ae2696e](https://github.com/maidsafe/safe-api/commit/ae2696ece0797ccbedd0916e819d125e5fd34d8d))
* **cli:** allow to pass account creation/login details with env vars or config file ([6354325](https://github.com/maidsafe/safe-api/commit/63543253527bde64d5180b240c63eb5ad6aba399))
* **cli:** allow to pass the authd endpoint address as an arg in the commands ([0cef9a2](https://github.com/maidsafe/safe-api/commit/0cef9a20654d335be03cfe4d2b8c44685bb67e2e))
* **cli:** allow to provide the path of the safe-authd executable with SAFE_AUTHD_PATH env var ([39b4ffb](https://github.com/maidsafe/safe-api/commit/39b4ffb2774533a20108658e0e721e7e39481803))
* **cli:** first implementation of CLI interactive shell ([11ba883](https://github.com/maidsafe/safe-api/commit/11ba883ab91ca08021a4d1d4b24291b9820b9e22))
* **cli:** support additional --output options, jsonpretty and yaml ([67eb5ad](https://github.com/maidsafe/safe-api/commit/67eb5ad317e7e09998c0243099050c65edc4009c))
* **cli:** support for caching current network conn info in CLI networks config ([2d48f71](https://github.com/maidsafe/safe-api/commit/2d48f71bc7b48b39a7a91b9024a99dab47922464))
* **cli:** support for having config settings, and a network command to switch networks ([31f054f](https://github.com/maidsafe/safe-api/commit/31f054f604afbab5fd3daaacb97f2f91ca70baf1))
* **jsonrpc:** make a distinction between errors on the client side and those received from the server within a json-rpc response ([22f22ef](https://github.com/maidsafe/safe-api/commit/22f22ef52e052c68533985c94e5286215929ffd4))
* **safe-cli:** improve afe-cli shell UI in the scenario of receiving an auth req notification ([8abfdc0](https://github.com/maidsafe/safe-api/commit/8abfdc02f9cfc801d05d93048a886f19d4209845))
* **xorurl:** add Display impl for XorUrlBase struct ([f55baab](https://github.com/maidsafe/safe-api/commit/f55baabce9708aa5d90f62df4c4dab5a6920fc98))


### Bug Fixes

* **api:** provide more descriptive error messages for login/create_acc functions ([7532cb7](https://github.com/maidsafe/safe-api/commit/7532cb7224888d9e27f01bb38e29a328d81076bb))
* **authd:** add a small delay when restarting authd right after it has been stopped ([2093669](https://github.com/maidsafe/safe-api/commit/20936693ca7c25f426c984d2c7de6d79aba972b0))
* **authd:** prevent from trying to update auth-req to a notified state if it was already removed from the queue/list ([0df5a07](https://github.com/maidsafe/safe-api/commit/0df5a073cd4cc692326ddbbfa3d186fa43b504a2))
* **authd:** set default certificate storage location to be local project config dir ([e3e0d44](https://github.com/maidsafe/safe-api/commit/e3e0d44a41d8e7f5059bceef9fba9246a9e3d323))
* **ci:** use correct source path ([8a94bf0](https://github.com/maidsafe/safe-api/commit/8a94bf0c9e0fa654f9eea6144507c304b32e539b))
* **cli:** files and nrs subcommand help text referred to keys subcommand ([02119c3](https://github.com/maidsafe/safe-api/commit/02119c3ab495a5c7cbc852ae0ed80bfbf81a1dee))
* **cli:** make sure credential file is not cleared with every auth command ([e7bc4f2](https://github.com/maidsafe/safe-api/commit/e7bc4f208aed75055e93670d52c1a763d21becd2))
* **cli:** show a more informative error when an invalid TX id is provided in a wallet/safekey transfer command ([ca809c9](https://github.com/maidsafe/safe-api/commit/ca809c94aeccd42b33a3b8e46ce915c57434428c))
* **mac:** Changes for mac compatability. ([f404cf9](https://github.com/maidsafe/safe-api/commit/f404cf976a1e9e7d085f03611eb73a7e402b146e))
* **mac:** remove hardened runtime for catalina ([5c32119](https://github.com/maidsafe/safe-api/commit/5c321192dfed5ce7d7de4ab8ade6e3b18e589045))
* **safe-api:** handle an error response from authd when trying to unsubscribe an authd client ([45ced7b](https://github.com/maidsafe/safe-api/commit/45ced7ba2c6fbc5c18ce81c097cff02a1fa41b33))

## 0.5.3 (2019-11-06)

### Features

* **api:** migrate ImmutableData API to use self-encryption mechanism/chunking for all published and (and unencrypted) immutable data stored with this API ([9f19b23](https://github.com/maidsafe/safe-api/commit/9f19b2380a73b3d5433469b89cc3bdba0bb2a984))


### Bug Fixes

* **api:** use type tag decoded from xorurl when fetching a FilesContainer ([59392e9](https://github.com/maidsafe/safe-api/commit/59392e99de614723da654a052dba912011117ecc))
* **ci:** use correct job references ([0b7ef5c](https://github.com/maidsafe/safe-api/commit/0b7ef5c79437598c2cdd40b37e2eb1dd0bd69239))
* **cli:** fix issue 203 storing credentials onto XDG_DATA_HOME based path ([9ee9df0](https://github.com/maidsafe/safe-api/commit/9ee9df0485fb19944d0c3fdfcff509abf976e533))
* **mobile-build:** fix mobile builds by removing the `reqwest` ([603bbe1](https://github.com/maidsafe/safe-api/commit/603bbe13fcf656d800af295c01ce729c7cc06325))

## 0.5.0 (2019-10-16)

### Features

* **bindings:** add bindings setup, expose fetch and connect function ([370fadd](https://github.com/maidsafe/safe-api/commit/370fadd6b03bec25a6874232addedc576ab9f818))
* **bindings:** add keys bindings ([326d869](https://github.com/maidsafe/safe-api/commit/326d8694560e0793d48c53b80894fc5039732a23))
* **bindings:** add nrs bindings ([920c282](https://github.com/maidsafe/safe-api/commit/920c28283a8600f0f74c755d01822a3c4ff7758b))
* **bindings:** add rust logging from SCL ([5c8ee5f](https://github.com/maidsafe/safe-api/commit/5c8ee5fc9c17ec45335e60daf49debc420f60b37))
* **bindings:** add structs for blskeypair and xorurlencoder ([6515306](https://github.com/maidsafe/safe-api/commit/6515306b588ce2cdd33882548a4f6d7acfe3299a))
* **bindings:** add xorurl bindings ([bce7e1f](https://github.com/maidsafe/safe-api/commit/bce7e1fcef96abd52f92ee7b4d48d9ce583e60ee))
* **bindings:** generate static libs ([9810eda](https://github.com/maidsafe/safe-api/commit/9810edaf2571a9ebf47f10f97c83be3296ce8b21))
* **bindings:** WIP wallet bindings ([1cf55ed](https://github.com/maidsafe/safe-api/commit/1cf55ed4acf35013d8fc455dbf269b8357f40390))
* **cli:** new dog command to inspect content retrieving only metadata ([5ee29ec](https://github.com/maidsafe/safe-api/commit/5ee29ec963dd63a479553feb7b3bf907259abd03))
* **fetch:** add a new API which allows to inspect a safe URL without retrieving the actual content ([37b0bd6](https://github.com/maidsafe/safe-api/commit/37b0bd6d762f8d299975795e36097a060836d953))
* **ffi:** create FFI for new inspect API ([308b00a](https://github.com/maidsafe/safe-api/commit/308b00aeb1aff796c89258b64352f29c699d76d8))
* **wallet:** allow to specify a specific speandable balance (as source or destination) for wallet operations ([a0237d0](https://github.com/maidsafe/safe-api/commit/a0237d03c408db32713c79dc97431cd4b210de7c))


### Bug Fixes

* **ci:** correct mount point for container ([23bd3c6](https://github.com/maidsafe/safe-api/commit/23bd3c68410e950ecd29dfca1ce3daa38477b1ee))
* **ffi:** adapt FFI return type for parse_and_resolve_url function ([0621ec6](https://github.com/maidsafe/safe-api/commit/0621ec687201d6294df6aa86409d10dad66c69a4))
* **ffi:** fix bindings to use into_raw() instead of as_ptr() for CString(s) ([96fbab3](https://github.com/maidsafe/safe-api/commit/96fbab329c5596734e282e95ed6f296d0ecf0eeb))
* **ffi:** fix keys_create to return new BlsKeyPair instance when None returned from API ([5aadae5](https://github.com/maidsafe/safe-api/commit/5aadae5daff0fe36917de4df77a497f3ed280ba9))
* **ffi:** fix native lib file name ([dfa878d](https://github.com/maidsafe/safe-api/commit/dfa878d7067fb56ebf364cd9b40db3f6b1f7780c))
* **ffi:** fix typo in structure name ([647d705](https://github.com/maidsafe/safe-api/commit/647d705fc216cb77acac496c97ad517d6e807f16))
* **ffi:** fixed build.rs to not add ref for appPtr ([fe6e3cd](https://github.com/maidsafe/safe-api/commit/fe6e3cded2906970e98a85c8b031b3eba8e610e5))
* **ffi:** minor fixes to issues introduced after rebasing ffi code with master ([b40a94b](https://github.com/maidsafe/safe-api/commit/b40a94bb1b527473a2ee7bc437720553ae8e21eb))
* **files:** check local file path before trying to attempt a files add operantion ([6bab08c](https://github.com/maidsafe/safe-api/commit/6bab08c31129779ebaf8b528d43f32834f5090e5))
* **wallet:** make sure we use the path when using Wallet NRS URLs in transfer operations ([872d69c](https://github.com/maidsafe/safe-api/commit/872d69c7b89961f74ebb6659e7dc51d8060dd3eb))
* **wallet:** return a specific error when the Wallet URL has an invalid spendable balance name as the path ([dbce607](https://github.com/maidsafe/safe-api/commit/dbce607370ab86acb30b5a9a0a69f52decb1179c))

## 0.4.0 (2019-09-19)

### Features

* **files:** implementation of files-add command and API for uploading single local files onto an existing FilesContainer ([68da824](https://github.com/maidsafe/safe-api/commit/68da8246d63bf248aa42df10a8d2368dee392fa6))
* **files:** support for adding a file using a safe:// URL to an existing FilesContainer ([5177dea](https://github.com/maidsafe/safe-api/commit/5177dea38ee9bcab14d2972c0a7eed79ecc5d27d))
* **files:** upload files as Raw content-type when their detected media-type is not supported, plus minor enhancements to errors reported by wallet API ([fc22254](https://github.com/maidsafe/safe-api/commit/fc2225422a0fbc732396d56a30d5badfa72e2573))
* **lib:** add files_container_add_from_raw API to add a file to an existing FilesContainer from raw bytes ([bd3a68b](https://github.com/maidsafe/safe-api/commit/bd3a68b56cd29a6a3b5e86e951c48e681b204da1))
* **xorurl:** allow to encode media-type in XOR-URLs for ImmutableData files ([b2affd5](https://github.com/maidsafe/safe-api/commit/b2affd5076ffec87b533d8cbc8415df430783f40))
* **xorurl:** support a subset of IANA media-types and encode them in XOR-URLs ([5910ca9](https://github.com/maidsafe/safe-api/commit/5910ca91dbe3539352d7af94f551d4f2ed164dc4))


### Bug Fixes

* **files:** report an error when adding a file with same name as existing one on target evne if its content is different ([cdd194f](https://github.com/maidsafe/safe-api/commit/cdd194f36a7e3626aaa8543c0e769ad41f022c8f))
* **wallet:** minor enhancements to error messages (issue [#238](https://github.com/maidsafe/safe-api/issues/238) and [#213](https://github.com/maidsafe/safe-api/issues/213)) ([a0ed709](https://github.com/maidsafe/safe-api/commit/a0ed7097ce16985855235fa871d1fca42a4fcc7c))

## 0.3.0 (2019-09-05)

### Features

* **SafeKey:** cat cmd to show information when targeting a SafeKey ([894ed15](https://github.com/maidsafe/safe-api/commit/894ed150b6b6fa9ddd467e7095ee2088b6aafad0))
* **safekeys:** implementation of a safe keys transfer cmd ([bcd4990](https://github.com/maidsafe/safe-api/commit/bcd4990afebbc3302063c46221a227d4ffb89d89))
* **transfers:** allow to pass a --tx-id to the keys/wallet transfer cmds to specify a TX ID ([80287d1](https://github.com/maidsafe/safe-api/commit/80287d1a6a546e55e4377547d2a330c5304ac5c0))


### Bug Fixes

* **ci:** integration tests were not running for dev builds ([3db2c47](https://github.com/maidsafe/safe-api/commit/3db2c473956c738e1f240893c4882a59ee0c4239))
* **ci:** perform strip correctly ([501cf1c](https://github.com/maidsafe/safe-api/commit/501cf1cf4dba7a44c496e97e49e28aa7cf2b04ab))
* **ci:** remove dir structure from zips ([ac7c6e2](https://github.com/maidsafe/safe-api/commit/ac7c6e23154652ea58ce8c7bbe109e9b6abc55b3))
* **cli:** make sure cli connects authorised to network before performing keys transfer cmd ([6bbdd42](https://github.com/maidsafe/safe-api/commit/6bbdd42be3095222646ff806f624f8fb430caa9f))
* **files sync:** when sync-ing a FilesContainer using an NRS name it was not correctly realising the latest version ([4ca7bd4](https://github.com/maidsafe/safe-api/commit/4ca7bd444355b38eac0d6074394abc4bab6d115d))
* **wallet:** change wallet transfer args from being positional to --from and --to ([865d365](https://github.com/maidsafe/safe-api/commit/865d3651ccdfa4da8afee0226e624fc774e177a0))

## 0.2.2 (2019-08-29)

### Features

* **cli:** display version in the xorurl for files sync feedback information ([96e4102](https://github.com/maidsafe/safe-api/commit/96e41020d263da914256d77736fed5d6d2ce4943))
* **fetch:** support for fetching a FilesContainer with a subfolder path ([3ad0955](https://github.com/maidsafe/safe-api/commit/3ad095507d387b4413419281f59604dc55f3c53b))
* **lib:** handle access denied error from wallet transfer API ([88da83e](https://github.com/maidsafe/safe-api/commit/88da83ef3c712f38fcd636a1d03095f34102991b))
* **wallet:** support for fetching the content of a Wallet and listing it with cat cmd ([7b79c95](https://github.com/maidsafe/safe-api/commit/7b79c9520304a7eff9455d47189005b923b1442a))


### Bug Fixes

* **cli:** minor fix to show the Wallet XOR-URL as the first line in the output of wallet create cmd ([199c577](https://github.com/maidsafe/safe-api/commit/199c5772173cc5407eecd0d2b456da30cc160b6c))
* **lib:** catch the correct error for insufficient balance from SCL, plus cosmetic improvement to CLI output when generating a key pair ([544139c](https://github.com/maidsafe/safe-api/commit/544139c765d03b10795d9ac5ebd3ecb1a73e7a59))
* **lib:** use the client instance's transfer_coin instead of the client independent wallet_transfer_coins API ([e3353c6](https://github.com/maidsafe/safe-api/commit/e3353c649efb3ca3e9f22d498cbb88394b2bff7e))
* **wallet:** add test and check in fake-scl for scenario when transferring 0 amount ([380e979](https://github.com/maidsafe/safe-api/commit/380e9793e1f21e7a4b13fcb55567afcadac0a64c))
* **wallet:** make use of the --sk when provided without a --keyurl in a wallet create cmd ([b3817b5](https://github.com/maidsafe/safe-api/commit/b3817b53a0f3abaaa0f3e8dcb3c03031ce395eaf))
* **wallet:** update default when set in wallet insert cmd, plus add details to User Guide about fetching Wallets and subfolders from FilesContainers ([ee457b0](https://github.com/maidsafe/safe-api/commit/ee457b0dfa347824f70e169160707e81be3a670d))

## 0.1.0 (2019-08-22)

### Features

* **cat:** implement an additional level for --info to cat command, i.e. -iii argument, to show a summary of the NRS map when retrieving content using public name ([ba57f31](https://github.com/maidsafe/safe-api/commit/ba57f318b68faf319ae44760bebd9ba32bf2cc9d))
* **cli:** check for release availability ([950dc0b](https://github.com/maidsafe/safe-api/commit/950dc0b13663781ae1b0fa63d3a02464893b9653))
* **cli:** have the files sync command to return the xorurl with new version (rather than the provided one) when the output is --json ([da7c57d](https://github.com/maidsafe/safe-api/commit/da7c57d69f60ce2baa51160539390bd767e0fc13))
* **cli:** initial use of self_update crate ([4532f35](https://github.com/maidsafe/safe-api/commit/4532f351e2fa3d02aafec50e79b55d0e8685f411))
* **cli:** introduce a --pay-with argument for keys and wallet commands to choose the payer wallet more explicitly ([9a24664](https://github.com/maidsafe/safe-api/commit/9a24664a191346c161bb5bc6d8c2906417f0b98b))
* **cli:** pull down new version ([59c0649](https://github.com/maidsafe/safe-api/commit/59c0649d76f9334666994d2d7657a61c1fd20c54))
* **fetch:** return NRS container info and render it with CLI if -ii passed ([f75981c](https://github.com/maidsafe/safe-api/commit/f75981c0db76df998ba33c95a0d39933e80efba8))
* **files sync:** support update-nrs arg in 'files sync' cmd which automatically updates the NRS link if an NRS-URL was passed ([370ffda](https://github.com/maidsafe/safe-api/commit/370ffda23ada391b060b06572e2bfd906f7dfecf))
* **lib:** make sure NRS name provided to nrs create/add, and target URL provided to files sync commands are unversioned ([624a51e](https://github.com/maidsafe/safe-api/commit/624a51e3933cc135737db333d2df19586584cbef))
* **nrs:** first draft code for nrs remove command ([1208062](https://github.com/maidsafe/safe-api/commit/120806238c4aa2a4e1db81b51da07b07a780ccf9))
* **nrs:** make NRS resolution to work only with versioned links, unless the linked content is unversionable ([1804390](https://github.com/maidsafe/safe-api/commit/1804390b017821dab019963f585937fd4d14067e))
* **nrs:** set default link as soft-links (to other sub names) and allow to set them as hard-links as well (to final link) ([febd818](https://github.com/maidsafe/safe-api/commit/febd818ee895ff1ef44590d8bfb8762865285a4e))
* **nrs:** support for fetching a specific version of NRS Map container by providing it in the URL ([1bdbe76](https://github.com/maidsafe/safe-api/commit/1bdbe764c698d4d0e6e222671a73ba60fed1eddf))
* **NRS:** Enable adding / updating NRS names + subnames. ([bfef3d2](https://github.com/maidsafe/safe-api/commit/bfef3d288a61947abdfee373762e8e7fc4981422)), closes [#142](https://github.com/maidsafe/safe-api/issues/142)
* **NRS:** Subname creation and resolution. ([91cb91a](https://github.com/maidsafe/safe-api/commit/91cb91a7fc1ca678fe6dddcf7fc6325d4f6d3d00))
* **update:** provide an update command ([c95df9d](https://github.com/maidsafe/safe-api/commit/c95df9d09d49c62aaa0d6eebdd4a155687216874))
* **urls:** support not only XOR URLs but also NRS URLs in all commands ([cdcab58](https://github.com/maidsafe/safe-api/commit/cdcab58beb76e1d17964b86a8ee9f152a12bdeed))
* **wallet:** support Key's URL (apart from Wallet's URL) as the destination for a wallet transfer operation, plus some additional info to the User Guide ([641a3f9](https://github.com/maidsafe/safe-api/commit/641a3f92db66b205bc775a6916a52885d1a6488f))
* **xorurl:** support for decoding the version from XOR URLs and fetching the specified version from FilesContainers ([9458663](https://github.com/maidsafe/safe-api/commit/9458663c91cf0a312c1a64bfec41ba174e3b5609))
* **xorurl:** use one byte to encode SAFE native data type, and separate two bytes for the content type info ([da086c5](https://github.com/maidsafe/safe-api/commit/da086c5600fd3cf4ea3258bcec37b1c6aad8823d))


### Bug Fixes

* **cat:** properly print out data and avoid panic-ing when retriving binary content ([65f86f3](https://github.com/maidsafe/safe-api/commit/65f86f3470110b41a6c159176bf8b6d01717bee7))
* **cli:** change owner back to maidsafe ([d897e0f](https://github.com/maidsafe/safe-api/commit/d897e0ff77515386bddf6a70e1853031b73669ab))
* **cli:** remove one non-supported --version arg from CLI help menu ([a78ecaf](https://github.com/maidsafe/safe-api/commit/a78ecaf73f62ef90afd14b42a44a199cd1920896))
* **files sync:** files sync was not committing the changes in FilesContainer when all changes were files removal ([37b01c6](https://github.com/maidsafe/safe-api/commit/37b01c6a026820267a1d4d36784c5d5c5d9f1c52))
* **nrs:** use higher precision (nanos) for the timestamp in the NRS Map container entries to prevent from collisions ([bbff014](https://github.com/maidsafe/safe-api/commit/bbff014476c4c5b5a5d342e39820c8b3be1ceced))
* **NRS:** Subname addition fixed. ([b6acddc](https://github.com/maidsafe/safe-api/commit/b6acddc4950765d4ad7ed3260530ee6e9abadab3))
* **nrs_map:** minor fix for when resolving a subname which doesn't have a link ([c364a29](https://github.com/maidsafe/safe-api/commit/c364a29b6490a005aaaa9eb981fac5e6b5472111))
* **scl:** minor fix related to handling versions with safe_client_libs ([7f423ce](https://github.com/maidsafe/safe-api/commit/7f423ceea8d1aae5a4decf2d28d265a883ba5e94))
* **tests:** minor fix to tests and resolve several issues reported by clippy ([8cc94d9](https://github.com/maidsafe/safe-api/commit/8cc94d953f52559211a807a60f2a8542ca014663))
* **update:** bin name based on target ([76ccd53](https://github.com/maidsafe/safe-api/commit/76ccd53ce50d9734b3130d40f3c1e252c2cd1721))

### 0.0.4 (2019-07-23)

### Features

* **API:** add mock API for unpublished append only data ([cc4e9df](https://github.com/maidsafe/safe-api/commit/cc4e9df443e2b1f383ab6d884ebc66d985dc714b))
* **API:** finalise SCL mock impl to allow wallet API testing ([8dcb27f](https://github.com/maidsafe/safe-api/commit/8dcb27fb5d482394fd419f77ee1645cb5d1aa87d))
* **API:** first draft implementation of keys_create function ([44a50e5](https://github.com/maidsafe/safe-api/commit/44a50e5d78f4d374ecbb3ccd56f68690279b23b9))
* **API:** first draft of the SCL API mock needed for testing ([a75807a](https://github.com/maidsafe/safe-api/commit/a75807a05e974b8a3812f748a92514a69a9320eb))
* **API:** use Hex encoded strings for sk and pk exposed/accepted from the API ([5137f9e](https://github.com/maidsafe/safe-api/commit/5137f9e53096b3cf353983607654a672ce2d888e))
* **auth:** allow to set the port number where to send the auth request to ([983ac63](https://github.com/maidsafe/safe-api/commit/983ac636ef29c4c2760818d9b1e986a46f2fffdf))
* **cat:** show additional info about native data type of content fetched ([c0d3f35](https://github.com/maidsafe/safe-api/commit/c0d3f3525d34f683484998281c53f9587e66429b))
* **cat:** show created and modified timestamps for each file ([3c29919](https://github.com/maidsafe/safe-api/commit/3c29919ffaa07ba47212f27214fd2f9dbe096071))
* **Cat:** Enable cat of safe://xor/some/path data. ([ef29698](https://github.com/maidsafe/safe-api/commit/ef296980cac4f57164aecf5d0228d2634173162d))
* **cli:** change default output to be human readable, plus explain cat cmd in user guide ([1712a8d](https://github.com/maidsafe/safe-api/commit/1712a8d3b20b80c4f592baa19ec5a3e4ce99df70))
* **cli:** implementation of auth command to get authorisation from SAFE Authenticator ([9a0a247](https://github.com/maidsafe/safe-api/commit/9a0a247c969467e0b92fa8bde5e4d26cc1984097))
* **cli:** make the Key XOR-URL arg to be optional for keys commands ([9f7f2fa](https://github.com/maidsafe/safe-api/commit/9f7f2faea1429d7c618f02a4776b8c5d95aa81b0))
* **cli:** making top lovel flags and options global for all cmds and subcmds ([7670499](https://github.com/maidsafe/safe-api/commit/76704996e48ef71de90be1ae2ec0cb07a7d98f2f))
* **errors:** make Key arg optional in wallet commands, plus enhancements to error handling in SCL integration code to have all tests passing with SCL mock-network feature ([f5309be](https://github.com/maidsafe/safe-api/commit/f5309be9ca91b86f62a4126cd140078aa9ba9e19))
* **files:** cleaning up tests for files put cmd and documenting command in user guide ([48149e1](https://github.com/maidsafe/safe-api/commit/48149e1ac1abc2c758b158157dc16d66725181e8))
* **files:** implementation of files sync command reporting add/modified/delete on each file uploaded ([79c5638](https://github.com/maidsafe/safe-api/commit/79c56380388359f21ad33d5f5f9a6e31ab46e669))
* **files:** implementation of the --dry-run for files put and files sync commands ([4e32c3b](https://github.com/maidsafe/safe-api/commit/4e32c3bc694a3a863a9e878831d18d8d9fa3daea))
* **files:** restrict the use of --delete flag for files sync to only when --recursive is also set ([d49a214](https://github.com/maidsafe/safe-api/commit/d49a214ae7e7afdb679e7264dfcbc6c1b1cc6eea))
* **files:** return and show current version of FilesContainer upon a sync/cat cmd ([9e9008b](https://github.com/maidsafe/safe-api/commit/9e9008b8d5150fb4b30a2339ab0f106dbba94639))
* **files:** support non-recursive put and sync for directories ([ae958d1](https://github.com/maidsafe/safe-api/commit/ae958d19dc340d56b2f66b6dd6cae55e20838694))
* **Files:** Enable setting alternate route for FilesMap RDF ([9047f7d](https://github.com/maidsafe/safe-api/commit/9047f7da2badb0d1c10b934134b2c13673c7d3ff))
* **Files:** Init of Files subcommand. ([da01954](https://github.com/maidsafe/safe-api/commit/da019549019aa096fdd0ea608ce0a25765c48021))
* **FilesContainer:** first draft impl of FilesContainer put and cat plus general clean up ([e7efba5](https://github.com/maidsafe/safe-api/commit/e7efba52990bfb74cafbbdb2693fea286e8969c7))
* **filesmap:** draft code to generate a serialised FilesMap ([5a3814b](https://github.com/maidsafe/safe-api/commit/5a3814bbbbe5fbcf4f9f3f9fbb302212bdc17265))
* **Init:** Initial code setup ([#4](https://github.com/maidsafe/safe-api/issues/4)) ([60c810a](https://github.com/maidsafe/safe-api/commit/60c810aefda01238814c35d67eb1d6e89e939caa))
* **keypair:** implement 'keys keypair' sub-command which generates a key-pair without creating a Key on the network ([f5e4cc5](https://github.com/maidsafe/safe-api/commit/f5e4cc581997abf3ef4904e0b2e65977d04036c8))
* **keys:** first draft implementation for the integration with SCL MoneyBalance API ([e73041c](https://github.com/maidsafe/safe-api/commit/e73041c58feb7bdcdd666a141410968132a754d4))
* **keys:** making the 'source' arg for 'keys create' optional and to be a SecretKey ([60317da](https://github.com/maidsafe/safe-api/commit/60317da29cd42ce0b94e2618412fe925c799ebb8))
* **keys:** making the 'target' arg totally optional for 'keys create' and make changes to have all keys unit tests to pass ([9d8e979](https://github.com/maidsafe/safe-api/commit/9d8e97954ae7f173de189bdbd1697fe2b86be9b6))
* **lib:** add function to create a Key and allocate test coins into it ([dc60d55](https://github.com/maidsafe/safe-api/commit/dc60d550a33aec6ce712d61451409889be8090a1))
* **lib:** first draft of lib's custom Error enum to be returned by all functions of its API ([f797198](https://github.com/maidsafe/safe-api/commit/f7971984da9ab5df19ebd7e67d46d958b059cf97))
* **MD:** Add remove mock func ([7bf197b](https://github.com/maidsafe/safe-api/commit/7bf197b808dc1abe243c3b4fe5286e639de9c8c0))
* **mock:** read/write mock file at creation/drop of MockSCL struct ([bbec6fa](https://github.com/maidsafe/safe-api/commit/bbec6fa94653b3589540f3cc47111babd7cfe9d4))
* **NRS:** Add basic NRS creation and fetching. ([65c6e68](https://github.com/maidsafe/safe-api/commit/65c6e688834c291444f66675c72136003a5dd60b)), closes [#68](https://github.com/maidsafe/safe-api/issues/68) [#149](https://github.com/maidsafe/safe-api/issues/149)
* **SCL:** Bindings for seq_appendable_data ([a3d33d3](https://github.com/maidsafe/safe-api/commit/a3d33d32981615cb515f43d56ae6933af82a38d5))
* **SCL:** initial integration functions with SCL. ([fa13b90](https://github.com/maidsafe/safe-api/commit/fa13b9065565ab945c7de4d5363a8a779c858caf)), closes [#23](https://github.com/maidsafe/safe-api/issues/23)
* **SCL:** Integration for published immutable data ([b2a8d49](https://github.com/maidsafe/safe-api/commit/b2a8d4940ecf029b808133a72bfa72256e3fe8f9))
* **Versioned Data:** Add simple versioned data put and test ([dd78b51](https://github.com/maidsafe/safe-api/commit/dd78b5114ad31412a5412910fb3e395ce64f63ff))
* **wallet:** add first draft implementation for wallet add and balance commands ([c2abe65](https://github.com/maidsafe/safe-api/commit/c2abe65621f030a616b0f44bc27ef52ddad90b8a))
* **wallet:** add first draft implementation of wallet create API and command ([7db002d](https://github.com/maidsafe/safe-api/commit/7db002d920399251103f840216562b94c767f971))
* **wallet:** Basic transfer set up. ([f57185b](https://github.com/maidsafe/safe-api/commit/f57185b9bb4ee68ea8322f43eb2dbed6bb79bea4))
* **Wallet:** Improve create / insert commands. ([1d264f0](https://github.com/maidsafe/safe-api/commit/1d264f0a791bacecd06429aee3bf42692327b680)), closes [#92](https://github.com/maidsafe/safe-api/issues/92)
* **xor-url:** add support for XOR-URL using CID and allow base32 (default) and base32z encoding to be chosen by the user ([84090d3](https://github.com/maidsafe/safe-api/commit/84090d3d10ec408d16d59414a35ff73fb150243b))
* **xorurl:** add path to XOR-URL when converting it to string ([da6d25f](https://github.com/maidsafe/safe-api/commit/da6d25f9fd13d2b8af92b2da5ec82394ca84cf57))
* **xorurl:** encode version, content-type and type tag in the XOR-URL ([be3dd30](https://github.com/maidsafe/safe-api/commit/be3dd30377e4b98e18f01b1ee06fe34d70258414))
* **xorurl:** enhance the XOR-URL encoding to remove zero bytes from type tag bytes, plus general cleanup and tests for fetch function ([bcf3a61](https://github.com/maidsafe/safe-api/commit/bcf3a617d0cfbc98bd38f7a4754b959d90989db0))
* **xorurl:** remove CID from xorurls for now and make base32z default base encoding, add first version of fetch and first impl of rendering a Filescontainer with cat cmd ([1994d9a](https://github.com/maidsafe/safe-api/commit/1994d9a550aa0cf72b787262031203be3caf7519))


#### Bug Fixes

* **auth:** resolve the user home directory using 'dirs' crate ([0d4e010](https://github.com/maidsafe/safe-api/commit/0d4e01088f3664c1c80a36bbddd498210bc7dc6a))
* **auth:** show informative message if cannot send auth request to Authenticator ([65d6d4e](https://github.com/maidsafe/safe-api/commit/65d6d4e77978c5e49e6cd74b90b9e742f9176747))
* **CI:** dockerfile additions for CI ([effbb18](https://github.com/maidsafe/safe-api/commit/effbb1893974ecf4048b1dbcbdce67fb85d3159a))
* **CI:** Windows remove mock vault before tests ([3208115](https://github.com/maidsafe/safe-api/commit/32081153222ff419b1773bebf6cfb8536a09a549))
* **CI:** Windows SCl-Vault remote command tweak ([395d866](https://github.com/maidsafe/safe-api/commit/395d86634be07a206ac2b7e51df58e79a7486036))
* **cli:** return error message if no <source> is provided when creating a Key ([8bdf7fc](https://github.com/maidsafe/safe-api/commit/8bdf7fce001b4708cdd99a523c011b2db0b782d1))
* **files:** remove --set-root arg in favor of using source and dest args, taking into account trailing slash in each of them to realise the final destination path, both for files put and sync commands ([a6da1a7](https://github.com/maidsafe/safe-api/commit/a6da1a7dfb747e70ea77028bd06d42b1221fab6f))
* **files:** use timestamp with nanos for the FilesContainer entry key in SeqAppendOnlyData ([aadd43e](https://github.com/maidsafe/safe-api/commit/aadd43ed07b380b9fecada77366ee910e6ff4abb))
* **Files:** Trailing slash for files commands now indicates to use folder as root ([#130](https://github.com/maidsafe/safe-api/issues/130)) ([dc81f95](https://github.com/maidsafe/safe-api/commit/dc81f95a0fa9ce6a25f54b129f766e75ce808c3a))
* **Files/Sync:** Use AD version not url schema version :| ([ae34a08](https://github.com/maidsafe/safe-api/commit/ae34a089cfb3a68eb23f36d45e102e900bc16bae))
* **keypair:** Enable keypair generation without authentication ([0dc81f0](https://github.com/maidsafe/safe-api/commit/0dc81f0d8997f3afdcd7843fd5c3dfbb454ad912))
* **keys:** gracefully handle the error when the source Key is not found upon Key creation ([a328f80](https://github.com/maidsafe/safe-api/commit/a328f80bf1006252197126ec763a51364b9c56f6))
* **keys:** restrict the preload amount to be numeric in all cases when creating keys ([bd76f40](https://github.com/maidsafe/safe-api/commit/bd76f405a72e9fcf1afcca7032ec98daad56270d))
* **lib:** fix issue [#158](https://github.com/maidsafe/safe-api/issues/158) by removing unwrap statement an returning proper Error ([88a0db5](https://github.com/maidsafe/safe-api/commit/88a0db527a2d0aa6d69a655f812ea67a3ba17d8b))
* **lib:** fix issues [#30](https://github.com/maidsafe/safe-api/issues/30) and [#31](https://github.com/maidsafe/safe-api/issues/31) to handle invalid args from CLI ([504e85c](https://github.com/maidsafe/safe-api/commit/504e85c370452c85b80de39b4fb9fdaf051a3b50))
* **lib:** gracefully handle the scenario when there is not enough funds at source for preloading a new Key ([4d84f91](https://github.com/maidsafe/safe-api/commit/4d84f91f22df6e95b1221b557306b498daa91774))
* **lib:** handle errors for invalid xorurl or pk when querying for Key balance ([e766cdd](https://github.com/maidsafe/safe-api/commit/e766cddcbfdac130288207ad9372a4cab5f7c545))
* **paths:** fixing treatment of paths to normalise them always to use '/' separator ([100d757](https://github.com/maidsafe/safe-api/commit/100d757eb94caff36520c555b8ddf87779975b4d))
* **SCL:** Post rebase fixes to SCL integration. ([1f2df4c](https://github.com/maidsafe/safe-api/commit/1f2df4cb9fd55d2dd3bc728bfa586501252c2339))
* **wallet:** enhancements to error message returned by wallet balance cmd ([4461686](https://github.com/maidsafe/safe-api/commit/4461686421e1bdc48102a9e405c298710477abda))
* **wallet:** handle errors for invalid/insufficient amounts for wallet transfers, and for wallets with no default balance ([2a4d42c](https://github.com/maidsafe/safe-api/commit/2a4d42c75c0206b792f73155b670cea8346927b0))
* **Wallet:** Validate SK and PK upon wallet creation. ([59bd39a](https://github.com/maidsafe/safe-api/commit/59bd39a2232632a24e09fe775284765bff10b559)), closes [#118](https://github.com/maidsafe/safe-api/issues/118) [#119](https://github.com/maidsafe/safe-api/issues/119)
* Handle files with no extension ([#132](https://github.com/maidsafe/safe-api/issues/132)) ([21a3ec1](https://github.com/maidsafe/safe-api/commit/21a3ec1913e4950c1c212adeac3474e12b6aef49))
* **Wallet:** Enable insert <name> to be optional. Update readme ([983c888](https://github.com/maidsafe/safe-api/commit/983c88879e7553f0ce8d2da7be7c976bd7cb6c7d))

