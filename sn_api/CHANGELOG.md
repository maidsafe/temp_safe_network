# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.54.0 (2022-02-01)

### Bug Fixes (BREAKING)

 - <csr-id-e0885987742226f72ed761e7b78b86e2fa72e256/> dry-runner was making a connection to the network
   - Removing unnecessary mutability in many Safe API.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 3 days passed between releases.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - dry-runner was making a connection to the network ([`e088598`](https://github.com/maidsafe/safe_network/commit/e0885987742226f72ed761e7b78b86e2fa72e256))
</details>

## v0.53.0 (2022-01-28)

### New Features

 - <csr-id-3d73dd03a7a6913a248e5cca7d714f8b8e4c0d01/> retrieve immutable content via nrs
   Changes the resolver to enable retrieval of NRS links to immutable content. Previously, the
   NRS `target_url` was incorrectly being checked to see if it contained a version. We now validate
   this URL using the same process used by NRS, which won't assert that a file link has a version. To
   reduce duplication, the `validate_nrs_url` function in the NRS module was changed to public.
   
   Tests were added to both the API and CLI to cover the scenario.
   
   The NrsMap struct was extended to include a subname_version field. This is going to be used to
   request the map with a particular version of a subname. If no version was specified when the
   container was retrieved, then the field won't be set. This is why it's an Option. Since we're
   storing this field on the NrsMap, the `version` field on the `NrsMapContainer` was removed.
   
   A couple of CLI NRS tests were also re-enabled, one of which happened to be related to immutable
   content.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 2 calendar days.
 - 6 days passed between releases.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.55.0/sn_api-0.53.0/sn_cli-0.46.0 ([`366eee2`](https://github.com/maidsafe/safe_network/commit/366eee25f4b982d5a20d90168368a1aa14aa3181))
    - Merge branch 'main' into nrs_resolve_immutable_content ([`099bf22`](https://github.com/maidsafe/safe_network/commit/099bf224714e667bf998de80099eeeabfd869d8b))
    - retrieve immutable content via nrs ([`3d73dd0`](https://github.com/maidsafe/safe_network/commit/3d73dd03a7a6913a248e5cca7d714f8b8e4c0d01))
    - nrs map fetch and rename multimap ([`889e0d9`](https://github.com/maidsafe/safe_network/commit/889e0d99a6f096329e875c812a29ec165e61f5ae))
</details>

## v0.52.0 (2022-01-22)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 3 calendar days.
 - 1 day passed between releases.
 - 5 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.54.0/sn_api-0.52.0/sn_cli-0.45.0 ([`0190f03`](https://github.com/maidsafe/safe_network/commit/0190f0305980bdaee30f9f2ab5eb5510149916db))
    - update remaining places ([`3dc2327`](https://github.com/maidsafe/safe_network/commit/3dc23278c6a4fabc250b27f4312f5c51f0f271a4))
    - Merge #958 ([`437a113`](https://github.com/maidsafe/safe_network/commit/437a113e6e5736e4eb4287f41228806678a9762e))
    - Merge branch 'main' into simplify-sn-api ([`33ef052`](https://github.com/maidsafe/safe_network/commit/33ef0524ae238391f25c8fb340627c34ea79fcb2))
    - remove one layer of indirection ([`3b5ce19`](https://github.com/maidsafe/safe_network/commit/3b5ce194213a7090ee83c02b0043700cda230796))
    - update from MIT/BSD3 to GPL3 ([`20f416c`](https://github.com/maidsafe/safe_network/commit/20f416cb7d0960a1d8d6f167a1ad1eed33ed6a7b))
    - Merge #962 ([`29d01da`](https://github.com/maidsafe/safe_network/commit/29d01da5233fd2a10b30699b555a0d85d7a7409a))
    - update year on files modified 2022 ([`7a7752f`](https://github.com/maidsafe/safe_network/commit/7a7752f830785ec39d301e751dc75f228d43d595))
</details>

## v0.51.0 (2022-01-20)

### Bug Fixes

 - <csr-id-e18c88019d37ab4f7618dde1a90e19ddf94db1c7/> VersioinHash use Display for encode
 - <csr-id-83ef7a66bb245e2303b80d98d6b8fa888b93d6ba/> make use of all the queries

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 13 calendar days.
 - 13 days passed between releases.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.53.0/sn_api-0.51.0/sn_cli-0.44.0 ([`923930a`](https://github.com/maidsafe/safe_network/commit/923930acb3769cfa7047954a1fee1853ec9e3062))
    - solving new clippy findings ([`57749b7`](https://github.com/maidsafe/safe_network/commit/57749b7d0671423fe205447bc84d9f8bfc99f54b))
    - Merge node-logrotate origin for rebase with main ([`6df7f6f`](https://github.com/maidsafe/safe_network/commit/6df7f6fec3ee9d37b44db188fd670e4b65796e8c))
    - Merge #885 ([`72a3f12`](https://github.com/maidsafe/safe_network/commit/72a3f1269c9c38add9b88455837655f2bc33b551))
    - fix additional wrongly setup test cases ([`941b83f`](https://github.com/maidsafe/safe_network/commit/941b83f3960c84cfee86a8c818233fbbc403c189))
    - Merge branch 'main' into kill-the-blob ([`5a055ba`](https://github.com/maidsafe/safe_network/commit/5a055ba679e6a4f2cd92700af68f8b36ac12a544))
    - Merge branch 'main' into kill-the-blob ([`411ce5b`](https://github.com/maidsafe/safe_network/commit/411ce5b9d4c396484d2384324ae09d346c79013f))
    - make use of all the queries ([`83ef7a6`](https://github.com/maidsafe/safe_network/commit/83ef7a66bb245e2303b80d98d6b8fa888b93d6ba))
    - Merge branch 'main' into kill-the-blob ([`9c5cd80`](https://github.com/maidsafe/safe_network/commit/9c5cd80c286308c6d075c5418d8a1650e87fddd5))
    - Merge #916 #918 #919 ([`5c4d3a9`](https://github.com/maidsafe/safe_network/commit/5c4d3a92ff28126468f07d599c6caf416661aba2))
    - Merge branch 'main' into kill-the-blob ([`fe814a6`](https://github.com/maidsafe/safe_network/commit/fe814a69e5ef5fbe4c62a056498ef88ce5897fef))
</details>

## v0.50.6 (2022-01-06)

### Bug Fixes

 - <csr-id-e18c88019d37ab4f7618dde1a90e19ddf94db1c7/> VersioinHash use Display for encode

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 1 day passed between releases.
 - 5 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.13/sn_api-0.50.6 ([`155ee03`](https://github.com/maidsafe/safe_network/commit/155ee032ee56cbbb34928f2d14529273ccb69559))
    - safe_network-0.52.10 ([`7b0cd4d`](https://github.com/maidsafe/safe_network/commit/7b0cd4d183a9f82d1d78dbb56572e5bf36714a0a))
    - VersioinHash use Display for encode ([`e18c880`](https://github.com/maidsafe/safe_network/commit/e18c88019d37ab4f7618dde1a90e19ddf94db1c7))
    - Merge branch 'main' into kill-the-blob ([`6f89f12`](https://github.com/maidsafe/safe_network/commit/6f89f129ece75dee45f311d30e52ca71b6b7bc98))
    - log EntryHash human readable ([`bf16c5e`](https://github.com/maidsafe/safe_network/commit/bf16c5ea7051386064233443921438cbbd79d907))
    - ties up the loose ends in unified data flow ([`9c9a537`](https://github.com/maidsafe/safe_network/commit/9c9a537ad12cc809540df321297c8552c52a8648))
</details>

## v0.50.5 (2022-01-06)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.11/sn_api-0.50.5/sn_cli-0.43.2 ([`99d012e`](https://github.com/maidsafe/safe_network/commit/99d012ef529df78ef4c84f5e6ea99d3a77414797))
    - Merge #917 ([`0eb6439`](https://github.com/maidsafe/safe_network/commit/0eb643910098ab6021561e5b997b6289be9e2c57))
    - Merge branch 'main' into kill-the-blob ([`40268a5`](https://github.com/maidsafe/safe_network/commit/40268a598aea8d14c1dbeb1c00712b9f9a664ef8))
</details>

## v0.50.4 (2022-01-04)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 5 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.9/sn_api-0.50.4 ([`a64c7e0`](https://github.com/maidsafe/safe_network/commit/a64c7e0414b77f545cb8cdbf64af0fb7212d1f2e))
    - Merge branch 'main' into kill-the-blob ([`7d38c3d`](https://github.com/maidsafe/safe_network/commit/7d38c3df14d03c042b645ad05be6cd3cc540d631))
    - safe_network-0.52.12 ([`4f29c28`](https://github.com/maidsafe/safe_network/commit/4f29c285a0b48220df1f1c6c52c4b487350eae08))
    - rename dest to dst ([`bebdae9`](https://github.com/maidsafe/safe_network/commit/bebdae9d52d03bd13b679ee19446452990d1e2cf))
    - revert change of fn name ([`ab8109c`](https://github.com/maidsafe/safe_network/commit/ab8109cf5aede62596abfdeb813a019d03201f96))
    - safe_network-0.52.8 ([`5214d5e`](https://github.com/maidsafe/safe_network/commit/5214d5e7f84a3c1cf213097a5d55bfb293f03324))
</details>

## v0.50.3 (2022-01-04)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.50.3 ([`5f7000c`](https://github.com/maidsafe/safe_network/commit/5f7000c5ec5895fb3f4c4a17a74ada52bb873fc7))
    - rename blob to file ([`c790077`](https://github.com/maidsafe/safe_network/commit/c790077bebca691f974000278d5525f4b011b8a7))
    - safe_network-0.52.5 ([`ab00eca`](https://github.com/maidsafe/safe_network/commit/ab00eca916d6ed8a0a137004a6b9fd24e7217a70))
</details>

## v0.50.2 (2022-01-04)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.6/sn_api-0.50.2 ([`0a70425`](https://github.com/maidsafe/safe_network/commit/0a70425fb314de4c165da54fdc29a127ae900d81))
    - safe_network-0.52.3 ([`2924661`](https://github.com/maidsafe/safe_network/commit/292466119e2d99c36043e7f2247b1bde9ec9ced9))
</details>

## v0.50.1 (2022-01-04)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.4/sn_api-0.50.1 ([`4bb2adf`](https://github.com/maidsafe/safe_network/commit/4bb2adf52efdac6187fffc299018bf13f3398e14))
    - safe_network-0.52.7 ([`40d1844`](https://github.com/maidsafe/safe_network/commit/40d1844e0b28578e8b8c6b270151dbb86961a766))
</details>

## v0.50.0 (2022-01-03)

### refactor (BREAKING)

 - <csr-id-715a154fe7448cd18decd0a666ae11fb02eadedb/> remove dry-run as arg from all APIs and make it a Safe instance mode

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.50.0/sn_cli-0.43.0 ([`ee86dc7`](https://github.com/maidsafe/safe_network/commit/ee86dc7ab1781731d3be19f9d7f414f157a91edb))
    - remove dry-run as arg from all APIs and make it a Safe instance mode ([`715a154`](https://github.com/maidsafe/safe_network/commit/715a154fe7448cd18decd0a666ae11fb02eadedb))
    - safe_network-0.52.2 ([`d490127`](https://github.com/maidsafe/safe_network/commit/d490127b17d53a7648f9e97aae690b232188b034))
</details>

## v0.49.0 (2022-01-03)

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

## v0.48.0 (2022-01-03)

### Bug Fixes

 - <csr-id-19e8f70c3f4369fae3a80d5de5e56161c5fa0258/> enable logging from api tests; resolve one failing test
 - <csr-id-7ba567f7f491836961e769c836226ebc9a4731f8/> when in dry-run was still requiring a connection by some APIs

### New Features (BREAKING)

 - <csr-id-4adaeaff4f07871840397adc3371ec8b3436e7ce/> change files APIs to accept std::Path for path args rather than only &str
   - Changed the files_container_create API to now create just an empty FilesContainer

### refactor (BREAKING)

 - <csr-id-ff1dd477aaea2a4dda6c9c15b5822b1b3a7514b7/> ProcessedFiles redefined on more specific data types instead of simply Strings

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 11 calendar days.
 - 11 days passed between releases.
 - 6 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.1/sn_api-0.48.0/sn_cli-0.41.0 ([`e38925e`](https://github.com/maidsafe/safe_network/commit/e38925e07d69432db310fc8ec9803200ea964ab2))
    - change files APIs to accept std::Path for path args rather than only &str ([`4adaeaf`](https://github.com/maidsafe/safe_network/commit/4adaeaff4f07871840397adc3371ec8b3436e7ce))
    - minor refactor and changes to CLI report errors ([`f1bb190`](https://github.com/maidsafe/safe_network/commit/f1bb1909f3fb506c1b7ec9b660ad533b7b8b9044))
    - ProcessedFiles redefined on more specific data types instead of simply Strings ([`ff1dd47`](https://github.com/maidsafe/safe_network/commit/ff1dd477aaea2a4dda6c9c15b5822b1b3a7514b7))
    - enable logging from api tests; resolve one failing test ([`19e8f70`](https://github.com/maidsafe/safe_network/commit/19e8f70c3f4369fae3a80d5de5e56161c5fa0258))
    - when in dry-run was still requiring a connection by some APIs ([`7ba567f`](https://github.com/maidsafe/safe_network/commit/7ba567f7f491836961e769c836226ebc9a4731f8))
</details>

## v0.47.0 (2021-12-22)

### New Features

 - <csr-id-1078e59be3a58ffedcd3c1460385b4bf00f18f6b/> use upload_and_verify by default in safe_client

### refactor (BREAKING)

 - <csr-id-1188ed58eed443b4b8c65b591376f2f9a21acc0d/> minor refactor to error types definitions

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 6 calendar days.
 - 6 days passed between releases.
 - 7 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.0/sn_api-0.47.0/sn_cli-0.40.0 ([`6b59ad8`](https://github.com/maidsafe/safe_network/commit/6b59ad852f89f033caf2b3c7dfcfa3019f8129e8))
    - minor refactor to error types definitions ([`1188ed5`](https://github.com/maidsafe/safe_network/commit/1188ed58eed443b4b8c65b591376f2f9a21acc0d))
    - safe_network-0.51.7 ([`c76c3ab`](https://github.com/maidsafe/safe_network/commit/c76c3ab638188cba38911f037829c209fcc45fc3))
    - calm down the retry loop tests ([`1d1487f`](https://github.com/maidsafe/safe_network/commit/1d1487f1c8e8f3e33df626af1ff027eea653f84c))
    - safe_network-0.51.6 ([`79b2d0a`](https://github.com/maidsafe/safe_network/commit/79b2d0a3f52de0335323773936dee9bdbe12a0cf))
    - safe_network-0.51.5 ([`45df3d7`](https://github.com/maidsafe/safe_network/commit/45df3d71cc4b3185602b9d27b8cb0f5bf65a4b43))
    - use upload_and_verify by default in safe_client ([`1078e59`](https://github.com/maidsafe/safe_network/commit/1078e59be3a58ffedcd3c1460385b4bf00f18f6b))
</details>

## v0.46.2 (2021-12-16)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.46.2 ([`6df94b1`](https://github.com/maidsafe/safe_network/commit/6df94b1d1fb017c9b02e566ca22a518f885397c8))
    - safe_network-0.51.4 ([`17d7906`](https://github.com/maidsafe/safe_network/commit/17d7906656bec401d6b39cc3551141112a3d77c4))
</details>

## v0.46.1 (2021-12-16)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.51.3/sn_api-0.46.1 ([`9be440b`](https://github.com/maidsafe/safe_network/commit/9be440b36db07e1c04ab688b44ef91e4a56ed576))
    - safe_network-0.51.2 ([`595541b`](https://github.com/maidsafe/safe_network/commit/595541b83284a5c5b60fbc00e47b1146117d7613))
</details>

## v0.46.0 (2021-12-16)

### New Features (BREAKING)

 - <csr-id-18879590ddfcf125133a6b2b8f3f372e8683be42/> rename Url to SafeUrl

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 2 calendar days.
 - 5 days passed between releases.
 - 11 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.46.0 ([`634a8f9`](https://github.com/maidsafe/safe_network/commit/634a8f9f307598c51305067444514b43c85f196d))
    - safe_network-0.51.1 ([`dcbb67f`](https://github.com/maidsafe/safe_network/commit/dcbb67fc699d7cb1f3a2c4632bcb8a5738916091))
    - safe_network-0.51.0 ([`c685838`](https://github.com/maidsafe/safe_network/commit/c685838d8f9c10b0f4e7541fe201862bb84e8555))
    - safe_network-0.50.0 ([`653f653`](https://github.com/maidsafe/safe_network/commit/653f653a775a101679904ab75c8012a72dfdedfb))
    - safe_network-0.49.3 ([`36ca20e`](https://github.com/maidsafe/safe_network/commit/36ca20e606899ecbdea24d845c34ba11ab889cf7))
    - adding a test for retrieving Blob with range over data length ([`edb8de8`](https://github.com/maidsafe/safe_network/commit/edb8de8b4d923e97d68eed40a7953f38461b0281))
    - fmt ([`d30aa0c`](https://github.com/maidsafe/safe_network/commit/d30aa0cb7440b9f3a44fefc3b6b9f7855480958c))
    - rename Url to SafeUrl ([`1887959`](https://github.com/maidsafe/safe_network/commit/18879590ddfcf125133a6b2b8f3f372e8683be42))
    - safe_network-0.49.2 ([`62d7479`](https://github.com/maidsafe/safe_network/commit/62d747969b739172910aabca6fcb273d2827fc8a))
    - safe_network-0.49.1 ([`69ae8c2`](https://github.com/maidsafe/safe_network/commit/69ae8c20e91dd9959ebfa5456efdf9c218a9d66f))
    - safe_network-0.49.0 ([`6f5516d`](https://github.com/maidsafe/safe_network/commit/6f5516d8bb677462ea6def46aa65a1094767d68c))
</details>

## v0.44.0 (2021-12-10)

### New Features

 - <csr-id-fd31bd2ef5ccc9149f8f0a2844c52af60bff3840/> use Vec of DataCmd instead of wrapper struct
 - <csr-id-544cfd21b2ede036615ac00673470f43f0399526/> dry run for NRS Multimaps Registers and FileContainers
 - <csr-id-da5136e3995307d71c11192769ad167b56962f26/> return nrs map on register fork, ignore fork errors of get for another subname
 - <csr-id-e2869c21b5b6c1c93f1a11cbad5c40b94b5e04ea/> improve register fork, fix dups issue, cleanup
 - <csr-id-9428700e5dec7aaabeaf078a8537d678aa0e5c4c/> Nrs with multimaps, resurected multimap tombstones
 - <csr-id-24f1aa0208e3e474862c18b21ea9f048cb6abf25/> expose API for calculate Blob/Spot address without a network connection

### Bug Fixes

<csr-id-b26833d80cf88b9a2dc1bb8478e74d9e37d6dc51/>
<csr-id-15dc3bd46686d25f679c21f85de091578d5f42eb/>

 - <csr-id-8955fcf9d69e869725177340d1de6b6b1e7a203b/> read_from client API was incorrectly using provided length value as an end index
   - Minor refactoring in sn_api moving the SafeData struct into its own file.

### New Features (BREAKING)

 - <csr-id-3fe9d7a6624fe5503f80395f6ed11426b131d3b1/> move Url to sn_api
 - <csr-id-c284f0787afe0d079e53b79b3a9d74cad04c4b0e/> `nrs create` only creates topnames

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 33 commits contributed to the release over the course of 14 calendar days.
 - 15 days passed between releases.
 - 33 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - `nrs create` only creates topnames ([`c284f07`](https://github.com/maidsafe/safe_network/commit/c284f0787afe0d079e53b79b3a9d74cad04c4b0e))
    - read_from client API was incorrectly using provided length value as an end index ([`8955fcf`](https://github.com/maidsafe/safe_network/commit/8955fcf9d69e869725177340d1de6b6b1e7a203b))
    - safe_network-0.48.0 ([`9eff035`](https://github.com/maidsafe/safe_network/commit/9eff03598ba09aa339180d7ecd57b50174180095))
    - files_container_create API was trying to send Register op to the network in dry-run ([`b26833d`](https://github.com/maidsafe/safe_network/commit/b26833d80cf88b9a2dc1bb8478e74d9e37d6dc51))
    - safe_network-0.47.0 ([`8570965`](https://github.com/maidsafe/safe_network/commit/85709655b0ce38246515658b956aa9b8f67cb55a))
    - re-enable fetch API tests to run in CI ([`66dc068`](https://github.com/maidsafe/safe_network/commit/66dc0682edb53c64a0660b3622bdc1a646114dee))
    - adapting test_files_container_remove_path test to new behavior ([`2b7c72c`](https://github.com/maidsafe/safe_network/commit/2b7c72c3f87f34051ea301250d1af258f9d310cb))
    - files container resolver was resolving path even when signaled to not to ([`15dc3bd`](https://github.com/maidsafe/safe_network/commit/15dc3bd46686d25f679c21f85de091578d5f42eb))
    - use Vec of DataCmd instead of wrapper struct ([`fd31bd2`](https://github.com/maidsafe/safe_network/commit/fd31bd2ef5ccc9149f8f0a2844c52af60bff3840))
    - dry run for NRS Multimaps Registers and FileContainers ([`544cfd2`](https://github.com/maidsafe/safe_network/commit/544cfd21b2ede036615ac00673470f43f0399526))
    - safe_network-0.46.6 ([`b5e9dcc`](https://github.com/maidsafe/safe_network/commit/b5e9dcc5b13b1eda711d4760d9feb8dc929a0c43))
    - refactoring files API tests ([`bb7abce`](https://github.com/maidsafe/safe_network/commit/bb7abce7330a432884e80baa2aa158409e9734d0))
    - return nrs map on register fork, ignore fork errors of get for another subname ([`da5136e`](https://github.com/maidsafe/safe_network/commit/da5136e3995307d71c11192769ad167b56962f26))
    - fmt ([`1120c99`](https://github.com/maidsafe/safe_network/commit/1120c99868a0a97e8e25a7611fea31838fe9f6f6))
    - improve register fork, fix dups issue, cleanup ([`e2869c2`](https://github.com/maidsafe/safe_network/commit/e2869c21b5b6c1c93f1a11cbad5c40b94b5e04ea))
    - fmt clippy ([`9afcb8b`](https://github.com/maidsafe/safe_network/commit/9afcb8b2776c39de1925742ccb19e36e9f3fec55))
    - Nrs with multimaps, resurected multimap tombstones ([`9428700`](https://github.com/maidsafe/safe_network/commit/9428700e5dec7aaabeaf078a8537d678aa0e5c4c))
    - clippy tidyup for rust 1.57 ([`05f6d98`](https://github.com/maidsafe/safe_network/commit/05f6d98cf21f0158f4b5161484c7c15a0561b6f4))
    - safe_network-0.46.5 ([`8657784`](https://github.com/maidsafe/safe_network/commit/86577846e845c110c49e15c95c6bd5595db51773))
    - safe_network-0.46.4 ([`de3051e`](https://github.com/maidsafe/safe_network/commit/de3051e7e809a8f75507c54f3cf053a4244fdf19))
    - safe_network-0.46.3 ([`69e9be2`](https://github.com/maidsafe/safe_network/commit/69e9be2a1567bfa211af7e9d7595381d9a0a3b38))
    - safe_network-0.46.2 ([`260eaab`](https://github.com/maidsafe/safe_network/commit/260eaabd2d1b0c26dec9febc963929e65d7ec912))
    - safe_network-0.46.1 ([`bf55f9b`](https://github.com/maidsafe/safe_network/commit/bf55f9b7e3b96319de4423e19333bf3b16fd1c78))
    - safe_network-0.46.0 ([`8ea9498`](https://github.com/maidsafe/safe_network/commit/8ea94983b37b1d559358a62d6ca075b97c193f0d))
    - safe_network-0.45.0 ([`a7e0585`](https://github.com/maidsafe/safe_network/commit/a7e058536ae6ae27228bd2254ea6465c5eface35))
    - expose API for calculate Blob/Spot address without a network connection ([`24f1aa0`](https://github.com/maidsafe/safe_network/commit/24f1aa0208e3e474862c18b21ea9f048cb6abf25))
    - safe_network-0.44.5 ([`aaab10b`](https://github.com/maidsafe/safe_network/commit/aaab10b3a5a44d9ec844757c71ac091016f51fd1))
    - safe_network-0.44.4 ([`984c5f8`](https://github.com/maidsafe/safe_network/commit/984c5f83e3f4d889dc4e0583b09571e540357cf9))
    - safe_network-0.44.3 ([`ec3dd49`](https://github.com/maidsafe/safe_network/commit/ec3dd4991535bb22235e2d1d413dd93489b8aedf))
    - safe_network-0.44.2 ([`51b0f00`](https://github.com/maidsafe/safe_network/commit/51b0f0068c9a279da9a1edf45509cf80a90e663d))
    - safe_network-0.44.1 ([`14c84c9`](https://github.com/maidsafe/safe_network/commit/14c84c9db23557626e4889eff0ff403a574dccad))
    - safe_network-0.44.0 ([`75a4b53`](https://github.com/maidsafe/safe_network/commit/75a4b537573d4e5e8767e38fa7d1b1126dffe148))
    - safe_network-0.43.0 ([`c78f470`](https://github.com/maidsafe/safe_network/commit/c78f4703a970e8b7466b091ad331d0f2233aa9a3))
</details>

## v0.43.0 (2021-11-25)

### New Features (BREAKING)

 - <csr-id-3fe9d7a6624fe5503f80395f6ed11426b131d3b1/> move Url to sn_api

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.42.0/sn_api-0.43.0 ([`ca21d1e`](https://github.com/maidsafe/safe_network/commit/ca21d1e97fcd28ca351887636affffff78e3aeb3))
    - move Url to sn_api ([`3fe9d7a`](https://github.com/maidsafe/safe_network/commit/3fe9d7a6624fe5503f80395f6ed11426b131d3b1))
</details>

## v0.42.0 (2021-11-25)

### New Features (BREAKING)

 - <csr-id-11750ed18391c7e8cb112d4a34a19f15eedaed1d/> propagate registers (with vec u8 instead of Url) change to sn_api

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.41.4/sn_api-0.42.0 ([`8b8a361`](https://github.com/maidsafe/safe_network/commit/8b8a3616673405005d77868dc397bd7542ab3ea7))
    - safe_network-0.41.3 ([`4b72bfc`](https://github.com/maidsafe/safe_network/commit/4b72bfc9a6c3a0db4821e7ebf1f4b5daa7cc56d1))
</details>

## v0.41.0 (2021-11-24)

### New Features (BREAKING)

 - <csr-id-11750ed18391c7e8cb112d4a34a19f15eedaed1d/> propagate registers (with vec u8 instead of Url) change to sn_api

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.41.0 ([`df25e49`](https://github.com/maidsafe/safe_network/commit/df25e4920c570771f6813ca03da02f6dfc8e59fb))
    - propagate registers (with vec u8 instead of Url) change to sn_api ([`11750ed`](https://github.com/maidsafe/safe_network/commit/11750ed18391c7e8cb112d4a34a19f15eedaed1d))
</details>

## v0.40.1 (2021-11-24)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.41.2/sn_api-0.40.1 ([`a973039`](https://github.com/maidsafe/safe_network/commit/a973039178af33b859d421cf36571de49cceff17))
    - safe_network-0.41.1 ([`62aa668`](https://github.com/maidsafe/safe_network/commit/62aa668d5777058ae617f8952cfcb62be002abf3))
    - revert "chore(release): safe_network-0.42.0/sn_api-0.41.0" ([`d8ec5a8`](https://github.com/maidsafe/safe_network/commit/d8ec5a81ae566e8d7068592e01cff4e808b1cad1))
    - safe_network-0.42.0/sn_api-0.41.0 ([`63432eb`](https://github.com/maidsafe/safe_network/commit/63432eb2e528401ae67da8eea0c82837ab42fc18))
</details>

## v0.40.0 (2021-11-23)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 8 days passed between releases.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.41.0 ([`14fdaa6`](https://github.com/maidsafe/safe_network/commit/14fdaa6537619483e94424ead5751d5ab41c8a01))
</details>

## v0.39.0 (2021-11-15)

<csr-id-60cc59ce18406609f36a37861afa920b96dcac99/>

### New Features

 - <csr-id-9098cff4c4d2ba15321dd072c970a18781a04a49/> add an example app showcasing how to fetch a Blob from Safe using sn_api
 - <csr-id-c5b0f0b630f673697367361508a30caf7ad787bd/> fixing multimap_remove and adding multimap_get_by_hash API
 - <csr-id-129f83f9b3a22669bf0fe0b5787546e4e3a924a0/> adding Register and Multimap public APIs
 - <csr-id-0beee4f8a1e876c1a7f482ba3be554e5b1654a42/> allow immutable reference to fetch method
 - <csr-id-cd3e1b71ef70bb7a4e5a786bdd337c2797250fdf/> allow aliases of Safe (immutable references)
   Tweaked a few method signatures to leave out the mutability binding.
   This means there can be multiple (immutable) references, instead of
   one unique reference to the Safe instance.
 - <csr-id-4f6c526e194f1b949c1b5b126442150157b7b0ba/> support transfers to BLS public keys
 - <csr-id-0f2f3c74dc81fefbc719e79f41af434023ac0462/> re-enabling dry-run feature for Blob API as well as for CLI commands
 - <csr-id-cb8ea4d6740f0f5e69be941608bcd620ed94e549/> when the --to value of keys transfer command doesn't start with safe:// and cannot be decoded as PK fallback to assume it's a URL
 - <csr-id-5499aeb2f755ce363b709c5379b860048c92ce5a/> pass SecretKey by reference
 - <csr-id-85462b6fb58f16f4797d7ef2816e96a287af7ad9/> adapting to new Seq type where Policy is immutable
 - <csr-id-45f11ae22df242e229d01bfc5dc2b6ac9de8536d/> customise the error message displayed when a panic occurred
 - <csr-id-ddb318d96c8a9bf8299bd14aa6824252296bde27/> store serialised NrsMap in a Blob and only its XOR-URL in the NrsMapContainer's underlying Sequence
 - <csr-id-1f377038ea693d5253f658d480885662df5b0542/> store serialised FilesMap in a Blob and only its XOR-URL in the FilesContainer's underlying Sequence
 - <csr-id-6c9cf24df423abae568fab63fc6615d9f7a3df68/> update sn_client and data types
 - <csr-id-2a43ca8fb10dcdbf085890643c673491399b1a8b/> command to configure networks in the config by providing a list of IP and ports
 - <csr-id-e366c878da84d2cf051bbc692e6b80c675ef8393/> add auth version subcommand which prints out the authd binary version
 - <csr-id-7a77ef4e3f3730d2f033d0365b2446bd560b1e21/> default to check balance of key assigned to CLI when no sk is provided to keys balance command
 - <csr-id-c20932f5c15fa16ccad907208522b9c9b52bb062/> support transfers to Ed25519 public keys in addition to using a Wallet or SafeKey URLs
 - <csr-id-b2e3faf9b0943ec779b1e513c76179048dbb0db3/> re-enable implementation for coins transfers with Ed25519 keys
 - <csr-id-d8186c309a7e6ca4862fb8855da6636a9f0d84c6/> insert and retrieve authorised apps keypairs into/from the Safe
 - <csr-id-d64d0fd9c0d6cd9002405985ff03f6d9ff7aa695/> reenable use of connect from auth responses
 - <csr-id-58ecf7fbae95dee0a103ce39d61efaac6e2cf550/> adapt authd client api and service names to new terminology of Safe creation and unlocking
 - <csr-id-805298fa5f4d1455015d2aa248143f460b4057ba/> store generated keypairs on the network
 - <csr-id-0cd75fbd9aceace5fbfc4b726a48d652177d61ce/> loop seq gets w/ timeout
   Add a timeout to sequence retrieval from the network to aid in
   convergence of data (ie. retry when get fails incase the responding
   nodes havent had time to update their own data yet).
   
   Timeout is configurable per safe app api instance, sop could be exposed
   for apps.
 - <csr-id-3f23687471b846e3ad1e2492c237a21f212b753f/> reenable decoding auth reqs and basic app key generation
 - <csr-id-b994b8d6ec1fcfc540e91aa9df79ba849aee7647/> setting up IPC for auth
 - <csr-id-07dd1956d4a53e2f4d09310b48e2a43a3a10e795/> moving in basics of auth func into the repo
 - <csr-id-de64d44d336611f4f7f427fd6b627a8c0880fd80/> remove sn_url crate form this repo and depend on the one just published on crates.io
 - <csr-id-7d2bd2f7ad27724866d04b7e10d69418f6decad9/> check for existing balance + err when creating acct

### Bug Fixes

 - <csr-id-d0cb7674071694885ebf9620a8cefcc4e62ecfcc/> NRS remove test
 - <csr-id-5582e995193ba17de3a3d0837271b405957d28ee/> NRS register failures
 - <csr-id-c88a1bc5fda40093bb129b4351eef73d2eb7c041/> resolution test except for the blob range tests
 - <csr-id-2fe38d13bc74a7c7cede96340d275ef3f94e1427/> NRS all tests except for the register issue
 - <csr-id-6c61764c7479405d2978bec1bbc5cbd11ca5e7c8/> parsing Url as xorurl
 - <csr-id-441be176521c20f9372b86d6daffc44bb97207fc/> add feature gate for app-specific error
 - <csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/> change node_path arg of 'node bin-path' command to node-path
 - <csr-id-8d0ac669b27247fed193cdc0543ab5c892bd7c0d/> styling by cargo fmt and clippy
 - <csr-id-e6b5cb012a8684c87c574ce98e55ad038d573d49/> removed more mut bindings to self methods
 - <csr-id-419bc87f345b4790a880b5def3b5b791d920a679/> fix set_content_version api in SafeUrl API to update its internal state
 - <csr-id-ce1823a437ffe59db01cfb412821fc7becd4634a/> pass approriate type
 - <csr-id-bfb09f28d9ff304b80354b971f8f02ffbf2a60c5/> adapt to new error returned by sn-client when Map entry is not found
 - <csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/> keypair API now returns a new randomly create Ed25519 key pair
 - <csr-id-9f18fb51a8c173ebb07e3ab9cea7a8f66b373fc5/> allocate testcoin to keypair generated for app upon being authorised the very first time
 - <csr-id-3694efb16b16fcfc0d34db51187c043d0e24f09c/> store Blob API was returning the wrong xorname
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-87d64dc3fe865b68f014da373aa253caf50d6512/> remove repeated data storing in SafeAppClient::store_sequence()
 - <csr-id-4bb48397cbe321bd551ec9530ca0325208dd138f/> set `formatting` feature for `time` dependency
   We use time formatting, so this feature should be necessary for the
   crate to compile. Weirdly, compilation only fails when publishing, not
   when checking or building. It's assumed this has something to do with
   feature resolution in workspaces 🤷

### chore (BREAKING)

 - <csr-id-60cc59ce18406609f36a37861afa920b96dcac99/> adapt test


### New Features (BREAKING)

 - <csr-id-8787f07281e249a344a217d7d5b0e732a7dd7959/> easy to use nrs_add and rigorous nrs_create

### Bug Fixes (BREAKING)

 - <csr-id-7ffda3021fb36533f22538b1100acfa71b13cd81/> nrs get with versions, nrs_map always returned

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 205 commits contributed to the release over the course of 404 calendar days.
 - 173 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#6](https://github.com/maidsafe/safe_network/issues/6)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#6](https://github.com/maidsafe/safe_network/issues/6)**
    - Drusu/review/feat multimap api ([`c63bb0f`](https://github.com/maidsafe/safe_network/commit/c63bb0fbfa8d661aef085dfc0582c6b537fd1349))
 * **Uncategorized**
    - safe_network v0.40.0/sn_api v0.39.0 ([`7001573`](https://github.com/maidsafe/safe_network/commit/70015730c3e08881f803e9ce59be7ca16185ae11))
    - safe_network v0.39.0 ([`ab00cf0`](https://github.com/maidsafe/safe_network/commit/ab00cf08d217654c57449437348b73576a65e89f))
    - update bls_dkg and blsttc to 0.7 and 0.3.4 respectively ([`213cb39`](https://github.com/maidsafe/safe_network/commit/213cb39be8fbfdf614f3eb6248b14fe161927a14))
    - safe_network v0.38.0 ([`cbdf023`](https://github.com/maidsafe/safe_network/commit/cbdf0236f7cd241e7addb1afe75ed5e5cfac00ab))
    - fix api tests. ([`3da9b26`](https://github.com/maidsafe/safe_network/commit/3da9b26f467cb8c468ffb0a319559a5a0f60e86e))
    - (cargo-release) version 0.37.0 ([`ef42af4`](https://github.com/maidsafe/safe_network/commit/ef42af483725286dd8a0cef6f922dfd1739412d8))
    - set `formatting` feature for `time` dependency ([`4bb4839`](https://github.com/maidsafe/safe_network/commit/4bb48397cbe321bd551ec9530ca0325208dd138f))
    - add env var for client query timeout ([`abfe737`](https://github.com/maidsafe/safe_network/commit/abfe7378604a74119accd7b9f86bef5682b0784a))
    - enable tests ([`d9c35b7`](https://github.com/maidsafe/safe_network/commit/d9c35b79a6d0aa787e627b747e665d96bb110c13))
    - NRS remove test ([`d0cb767`](https://github.com/maidsafe/safe_network/commit/d0cb7674071694885ebf9620a8cefcc4e62ecfcc))
    - NRS register failures ([`5582e99`](https://github.com/maidsafe/safe_network/commit/5582e995193ba17de3a3d0837271b405957d28ee))
    - bump rust edition ([`fc10d03`](https://github.com/maidsafe/safe_network/commit/fc10d037d64efc86796f1b1c6f255a4c7f91d3e1))
    - adapt test ([`60cc59c`](https://github.com/maidsafe/safe_network/commit/60cc59ce18406609f36a37861afa920b96dcac99))
    - easy to use nrs_add and rigorous nrs_create ([`8787f07`](https://github.com/maidsafe/safe_network/commit/8787f07281e249a344a217d7d5b0e732a7dd7959))
    - nrs get with versions, nrs_map always returned ([`7ffda30`](https://github.com/maidsafe/safe_network/commit/7ffda3021fb36533f22538b1100acfa71b13cd81))
    - appease clippy ([`407efd1`](https://github.com/maidsafe/safe_network/commit/407efd15e0b4854864b83ccdb7d2c3adbb0a02e2))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_cli_into_workspace ([`eea8307`](https://github.com/maidsafe/safe_network/commit/eea83074b9bbd334d80b80f12cfcce724d0e8ca3))
    - update cli to use new nrs api ([`8691034`](https://github.com/maidsafe/safe_network/commit/86910340897256bb4df77b6edaa0f2c9584d6dce))
    - switch from `chrono` to `time` ([`442c1bd`](https://github.com/maidsafe/safe_network/commit/442c1bdd3f9c22424aa9642d71d4016c868b0b58))
    - merge github.com:maidsafe/sn_cli into safe_network ([`414aca2`](https://github.com/maidsafe/safe_network/commit/414aca284b35f1bcb27e5d0cca2bfe451b69e27b))
    - upgrade `tracing-appender` and `tracing-subscriber` ([`0387123`](https://github.com/maidsafe/safe_network/commit/0387123114ff6ae42920577706497319c8a888cb))
    - update actions workflows for workspace refactor ([`3703819`](https://github.com/maidsafe/safe_network/commit/3703819c7f0da220c8ff21169ca1e8161a20157b))
    - move safe_network code into sn directory ([`2254329`](https://github.com/maidsafe/safe_network/commit/225432908839359800d301d9e5aa8274e4652ee1))
    - move sn_api code into an sn_api directory ([`2b5d177`](https://github.com/maidsafe/safe_network/commit/2b5d17740ca74fc379cab89cb95683e200589148))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_api_into_workspace ([`8ed5aff`](https://github.com/maidsafe/safe_network/commit/8ed5aff8b30ce798f71eac22d66eb3aa9b0bdcdd))
    - upgrade sn_api to use 0.36.x of safe_network ([`0118362`](https://github.com/maidsafe/safe_network/commit/01183625d7a1a60b652b1a295a908fa8ba04f6f7))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_api_into_workspace ([`3a2817a`](https://github.com/maidsafe/safe_network/commit/3a2817a4c802d74b57d475d88d7bc23223994147))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_api_into_workspace ([`50f48ae`](https://github.com/maidsafe/safe_network/commit/50f48aefcba272345df7d4cd45a59071a5844932))
    - Merge branch 'merge_sn_api_into_workspace' into nrs_resolver_refactor ([`a273d97`](https://github.com/maidsafe/safe_network/commit/a273d9733b8d50b94b0ea3faec1d9e721d86aa27))
    - various renamings and changes matching PR comments ([`335f9dc`](https://github.com/maidsafe/safe_network/commit/335f9dcfc4588624728b4b10c576953d51a08e1a))
    - fmt ([`49d04e1`](https://github.com/maidsafe/safe_network/commit/49d04e1414bf517cc76ebe2c6b86e0b3dd48e47a))
    - resolution test except for the blob range tests ([`c88a1bc`](https://github.com/maidsafe/safe_network/commit/c88a1bc5fda40093bb129b4351eef73d2eb7c041))
    - NRS all tests except for the register issue ([`2fe38d1`](https://github.com/maidsafe/safe_network/commit/2fe38d13bc74a7c7cede96340d275ef3f94e1427))
    - parsing Url as xorurl ([`6c61764`](https://github.com/maidsafe/safe_network/commit/6c61764c7479405d2978bec1bbc5cbd11ca5e7c8))
    - nrs and resolver ([`c0ac51a`](https://github.com/maidsafe/safe_network/commit/c0ac51ae4bf4dbd9df3dd39700887df439eec4f6))
    - update actions workflows for workspace refactor ([`d0134e8`](https://github.com/maidsafe/safe_network/commit/d0134e870bb097e095e1c8a33e607cf7994e6491))
    - add an example app showcasing how to fetch a Blob from Safe using sn_api ([`9098cff`](https://github.com/maidsafe/safe_network/commit/9098cff4c4d2ba15321dd072c970a18781a04a49))
    - Version change: sn_api v0.26.0; sn_cli v0.26.0; sn_authd v0.8.0 ([`3bcf8ef`](https://github.com/maidsafe/safe_network/commit/3bcf8efcee84c5fb45f5e03ec49d5a623147dc4d))
    - Version change: sn_api v0.25.3; sn_cli v0.25.3; sn_authd v0.7.3 ([`ab68342`](https://github.com/maidsafe/safe_network/commit/ab683420665c54df1ae3dae95055000518b543d1))
    - Version change: sn_api v0.25.2; sn_cli v0.25.2; sn_authd v0.7.2 ([`0282dd6`](https://github.com/maidsafe/safe_network/commit/0282dd6edfce91ac25314bda7b6d87fd1ae621fe))
    - update sn_client to 0.54.4 ([`91e1f0c`](https://github.com/maidsafe/safe_network/commit/91e1f0cefe1eb6a6d70e0df3d4d3b6f97e76ecef))
    - remove sn_url crate form this repo and depend on the one just published on crates.io ([`de64d44`](https://github.com/maidsafe/safe_network/commit/de64d44d336611f4f7f427fd6b627a8c0880fd80))
    - minor changes to rebase with master ([`7ceab22`](https://github.com/maidsafe/safe_network/commit/7ceab22ae72535159db7fbfdc5832b1aea891388))
    - moving out safe_url mod as a standalone sn_url crate ([`5780fd1`](https://github.com/maidsafe/safe_network/commit/5780fd1d6ba480cb775fd66e53e41f02d97b3a94))
    - Version change: sn_api v0.25.1; sn_cli v0.25.1; sn_authd v0.7.1 ([`7a8860d`](https://github.com/maidsafe/safe_network/commit/7a8860d71776958fb93e91fefe157b3de4277a8c))
    - add feature gate for app-specific error ([`441be17`](https://github.com/maidsafe/safe_network/commit/441be176521c20f9372b86d6daffc44bb97207fc))
    - Version change: sn_api v0.25.0; sn_cli v0.25.0; sn_authd v0.7.0 ([`60717f1`](https://github.com/maidsafe/safe_network/commit/60717f1a09aac06911f01cb3a811731721ae5708))
    - change node_path arg of 'node bin-path' command to node-path ([`d91150d`](https://github.com/maidsafe/safe_network/commit/d91150d8f46003cc0fa7813e4ae907b187379de8))
    - fixing multimap_remove and adding multimap_get_by_hash API ([`c5b0f0b`](https://github.com/maidsafe/safe_network/commit/c5b0f0b630f673697367361508a30caf7ad787bd))
    - adding first set of tests for Multimap API ([`6f05016`](https://github.com/maidsafe/safe_network/commit/6f0501699b0d0620a7c9d2b013944f90884ca1c3))
    - adding Register and Multimap public APIs ([`129f83f`](https://github.com/maidsafe/safe_network/commit/129f83f9b3a22669bf0fe0b5787546e4e3a924a0))
    - styling by cargo fmt and clippy ([`8d0ac66`](https://github.com/maidsafe/safe_network/commit/8d0ac669b27247fed193cdc0543ab5c892bd7c0d))
    - allow immutable reference to fetch method ([`0beee4f`](https://github.com/maidsafe/safe_network/commit/0beee4f8a1e876c1a7f482ba3be554e5b1654a42))
    - removed more mut bindings to self methods ([`e6b5cb0`](https://github.com/maidsafe/safe_network/commit/e6b5cb012a8684c87c574ce98e55ad038d573d49))
    - allow aliases of Safe (immutable references) ([`cd3e1b7`](https://github.com/maidsafe/safe_network/commit/cd3e1b71ef70bb7a4e5a786bdd337c2797250fdf))
    - fix set_content_version api in SafeUrl API to update its internal state ([`419bc87`](https://github.com/maidsafe/safe_network/commit/419bc87f345b4790a880b5def3b5b791d920a679))
    - Version change: sn_api v0.24.0; sn_cli v0.24.0; sn_authd v0.6.0 ([`47e7f0a`](https://github.com/maidsafe/safe_network/commit/47e7f0aea943a568d767a5226b0d8e71414508bc))
    - support transfers to BLS public keys ([`4f6c526`](https://github.com/maidsafe/safe_network/commit/4f6c526e194f1b949c1b5b126442150157b7b0ba))
    - removing unused files ([`ce56a35`](https://github.com/maidsafe/safe_network/commit/ce56a3504c8f27bfeb13bdf9051c2e91409230ea))
    - re-enabling dry-run feature for Blob API as well as for CLI commands ([`0f2f3c7`](https://github.com/maidsafe/safe_network/commit/0f2f3c74dc81fefbc719e79f41af434023ac0462))
    - re-organising files, nrs and xorurl files into their own mod folders ([`afd5422`](https://github.com/maidsafe/safe_network/commit/afd5422945fd1fc4ac509713e72471076ea4aee0))
    - when the --to value of keys transfer command doesn't start with safe:// and cannot be decoded as PK fallback to assume it's a URL ([`cb8ea4d`](https://github.com/maidsafe/safe_network/commit/cb8ea4d6740f0f5e69be941608bcd620ed94e549))
    - Version change: sn_api v0.23.2; sn_cli v0.23.2; sn_authd v0.5.2 ([`e939702`](https://github.com/maidsafe/safe_network/commit/e939702fdc1986c0021cf12223cbc707589b889f))
    - Version change: sn_api v0.23.1; sn_cli v0.23.1; sn_authd v0.5.1 ([`6aa920e`](https://github.com/maidsafe/safe_network/commit/6aa920e07a42b85ca8d081b8c93e7290553bb7ca))
    - Version change: sn_api v0.23.0; sn_cli v0.23.0; sn_authd v0.5.0 ([`e506e06`](https://github.com/maidsafe/safe_network/commit/e506e06acd50467834e80ebb15a3221261b45752))
    - fix styling by cargo fmt ([`a4b7d9b`](https://github.com/maidsafe/safe_network/commit/a4b7d9ba415831308c5ec6d902722843715a2d97))
    - pass approriate type ([`ce1823a`](https://github.com/maidsafe/safe_network/commit/ce1823a437ffe59db01cfb412821fc7becd4634a))
    - pass SecretKey by reference ([`5499aeb`](https://github.com/maidsafe/safe_network/commit/5499aeb2f755ce363b709c5379b860048c92ce5a))
    - Version change: sn_api v0.22.0; sn_cli v0.22.0; sn_authd v0.4.0 ([`fedab1b`](https://github.com/maidsafe/safe_network/commit/fedab1b7bc6c01b8be07ae2c54c034514bc70717))
    - update sn_client and sn_data_types to latest ([`0d4755e`](https://github.com/maidsafe/safe_network/commit/0d4755ed64a65c223bad253d9d7a03980ec12e8d))
    - init logger in sn_api tests ([`6a8a173`](https://github.com/maidsafe/safe_network/commit/6a8a173acf96d98d00548659b4641488f5fec2ee))
    - Version change: sn_api v0.21.0; sn_cli v0.21.0; sn_authd v0.3.0; qjsonrpc v0.2.0 ([`838238d`](https://github.com/maidsafe/safe_network/commit/838238d745a18aa28a8b366ab4adc62745656990))
    - fixing files and NRS APIs tests ([`b9bae62`](https://github.com/maidsafe/safe_network/commit/b9bae620533559c5671075fb7a3fe576fe14431f))
    - upgrade tokio to v1.3.0 and quinn to v0.10.1 ([`d77859a`](https://github.com/maidsafe/safe_network/commit/d77859a8138de0ddcd6b121b928efe13e0254e81))
    - adapting to new Seq type where Policy is immutable ([`85462b6`](https://github.com/maidsafe/safe_network/commit/85462b6fb58f16f4797d7ef2816e96a287af7ad9))
    - customise the error message displayed when a panic occurred ([`45f11ae`](https://github.com/maidsafe/safe_network/commit/45f11ae22df242e229d01bfc5dc2b6ac9de8536d))
    - re-enabling wallet APIs tests in CI ([`6cf5602`](https://github.com/maidsafe/safe_network/commit/6cf56022cd6beec4254d0b1667c4d654f87e6a5a))
    - Version change: sn_api v0.20.0; sn_cli v0.20.0; sn_authd v0.2.0; qjsonrpc v0.1.2 ([`a35ffb7`](https://github.com/maidsafe/safe_network/commit/a35ffb759bafd6e2b03d96bffa62747eb1965c89))
    - re-enabling xorurl & keys APIs tests in CI ([`4518154`](https://github.com/maidsafe/safe_network/commit/4518154481dbd3aeb397353d4ec296ea98ee3e9a))
    - re-enabling sequence & fetch APIs tests in CI ([`590c4c6`](https://github.com/maidsafe/safe_network/commit/590c4c634046ab655a84093c2ed60e8289415d44))
    - remove unused utility function ([`a09c4c2`](https://github.com/maidsafe/safe_network/commit/a09c4c26230b0cf60d7d792d89f0e275f2b64bc2))
    - store serialised NrsMap in a Blob and only its XOR-URL in the NrsMapContainer's underlying Sequence ([`ddb318d`](https://github.com/maidsafe/safe_network/commit/ddb318d96c8a9bf8299bd14aa6824252296bde27))
    - store serialised FilesMap in a Blob and only its XOR-URL in the FilesContainer's underlying Sequence ([`1f37703`](https://github.com/maidsafe/safe_network/commit/1f377038ea693d5253f658d480885662df5b0542))
    - reorganise files module ([`a6107ac`](https://github.com/maidsafe/safe_network/commit/a6107accfc6950b4beb2dcb84dfaa3c0e18bbd5d))
    - adding a README.md file for sn_api crate with a description of the current APIs ([`712d9a4`](https://github.com/maidsafe/safe_network/commit/712d9a4e72f62725a0b0ac5526bc68abe8ca503f))
    - Version change: sn_api v0.19.1; sn_cli v0.19.1 ([`edbdcb6`](https://github.com/maidsafe/safe_network/commit/edbdcb62c36a2998aab23dd3a4d0b13bae13b472))
    - update sn_client and data types ([`6c9cf24`](https://github.com/maidsafe/safe_network/commit/6c9cf24df423abae568fab63fc6615d9f7a3df68))
    - Version change: sn_api-v0.19.0; sn_cli-v0.19.0; sn_authd-v0.1.1; qjsonrpc-v0.1.1 ([`21f4733`](https://github.com/maidsafe/safe_network/commit/21f4733fbc32efd2c822337c7b3f077cca0f2992))
    - adding a step to check for unused dependencies ([`de482a5`](https://github.com/maidsafe/safe_network/commit/de482a5611333d069076d7da1b7c5a6017db65eb))
    - add a simple example app which uploads a file using the API ([`7d00d0a`](https://github.com/maidsafe/safe_network/commit/7d00d0ad089924915aa2bf564b5b925825aa4880))
    - use get_sequence_entry ([`d910e86`](https://github.com/maidsafe/safe_network/commit/d910e8607c8898bb8d8ccb0c824d8a5401a2c938))
    - upgrade sn_client to v0.46.12 and most of all dependencies to their latest published version ([`e3c6da3`](https://github.com/maidsafe/safe_network/commit/e3c6da38f92c354c560bd6b555d76f698779ebcf))
    - Remove unused ed25519 dependency ([`6f67254`](https://github.com/maidsafe/safe_network/commit/6f672547ee9a55b241aa6004b353d6919fb0e8cb))
    - update tiny-keccak from 1.5.0 to 2.0.2 ([`792bce8`](https://github.com/maidsafe/safe_network/commit/792bce8dd94192f17c51d6a1c0b63c7c214ad7c3))
    - connect to port 12000 by default in local testnets ([`472df87`](https://github.com/maidsafe/safe_network/commit/472df87678c07624d6163b8105f110cd22e4e3c9))
    - upgrade sn_client to v0.46.9 and solve clippy issues ([`b61e837`](https://github.com/maidsafe/safe_network/commit/b61e83716cce00c0ba02f3d50bf060cfc095051a))
    - set bootstrapping contacts for api-tests ([`6d90943`](https://github.com/maidsafe/safe_network/commit/6d909431d6a7164a06f2322d188c1d4764e5d0b8))
    - provide bootstrapping contacts list to sn_client as required by new sn_client API ([`43c675e`](https://github.com/maidsafe/safe_network/commit/43c675ee514aa73fb5192717dae58c97587521e7))
    - command to configure networks in the config by providing a list of IP and ports ([`2a43ca8`](https://github.com/maidsafe/safe_network/commit/2a43ca8fb10dcdbf085890643c673491399b1a8b))
    - add auth version subcommand which prints out the authd binary version ([`e366c87`](https://github.com/maidsafe/safe_network/commit/e366c878da84d2cf051bbc692e6b80c675ef8393))
    - migrating to use anyhow for CLI errors and use thiserror for sn_api error types ([`c2c6716`](https://github.com/maidsafe/safe_network/commit/c2c6716d29e56f387776202dad94ddda9b8fe2b2))
    - Version change: sn_api-v0.18.0; sn_cli--v0.18.0; sn_authd-v0.1.0; qjsonrpc-0.1.0 ([`fce96bf`](https://github.com/maidsafe/safe_network/commit/fce96bfb00279be41a139a360d1b2eac02d874cf))
    - upgrading sn_client to v0.46.2 ([`f0bef9e`](https://github.com/maidsafe/safe_network/commit/f0bef9e5e79381abf27eec7fae28f4ce0fecb370))
    - changes to remove any use of Arc for keypairs and secret keys ([`4ba83c7`](https://github.com/maidsafe/safe_network/commit/4ba83c720fabcace7a2859ad308be5922a6597c0))
    - update sn_client and dts ([`b38d840`](https://github.com/maidsafe/safe_network/commit/b38d840320d65b09ce85db9074f7b7a9487f83df))
    - Money -> Token ([`b04c1ca`](https://github.com/maidsafe/safe_network/commit/b04c1ca2090a32c423ffe2c23ac22be9f5ebbcf3))
    - Invalidate nrs names with troublesome characters ([`eabdd42`](https://github.com/maidsafe/safe_network/commit/eabdd4254e0725a8556a413d786966a00d9e6e3d))
    - Validate length of nrs name and subname ([`0f578dc`](https://github.com/maidsafe/safe_network/commit/0f578dc0079973ed9d1b49fa04c222d7f3460a6b))
    - Remove unused check for whitespace ([`71852f5`](https://github.com/maidsafe/safe_network/commit/71852f56e490cc3b0f31f15005c142eced0a72e1))
    - Invalidate public names containing slash char ([`4a33102`](https://github.com/maidsafe/safe_network/commit/4a331029c6d6f18fdf487f86bd6130a0455b0a4b))
    - Remove repeated 'cannot cannot' in error message ([`7cb0969`](https://github.com/maidsafe/safe_network/commit/7cb0969522f9845cddff8a4bd52725f500697975))
    - do not attempt to retry fetching a Sequence entry if not found the first time ([`2dff02d`](https://github.com/maidsafe/safe_network/commit/2dff02dc71bc3574763906c8592d32bde64337c9))
    - default to check balance of key assigned to CLI when no sk is provided to keys balance command ([`7a77ef4`](https://github.com/maidsafe/safe_network/commit/7a77ef4e3f3730d2f033d0365b2446bd560b1e21))
    - adapt to new error returned by sn-client when Map entry is not found ([`bfb09f2`](https://github.com/maidsafe/safe_network/commit/bfb09f28d9ff304b80354b971f8f02ffbf2a60c5))
    - minor logging improvements and upgrade sn_client to v0.44.16 ([`5ba32bd`](https://github.com/maidsafe/safe_network/commit/5ba32bd9a86e9c44cf3d59f54be899dc3e2ca861))
    - upgrade sn_client to v0.44.15 ([`4f89812`](https://github.com/maidsafe/safe_network/commit/4f89812ed5ca3394d2cd7b93e3c79aac2929d11d))
    - Version change: sn_api-v0.17.1; sn_cli-v0.17.1; sn_authd-v0.0.14 ([`3961969`](https://github.com/maidsafe/safe_network/commit/396196997f6d114b01e5b269447b3c4219250f35))
    - fix all clippy issues after updating to rust 1.49 ([`67b746f`](https://github.com/maidsafe/safe_network/commit/67b746f607501511c38fe752f64119a12985ab72))
    - minor change to error returned when parsing pk from hex ([`6e4ea36`](https://github.com/maidsafe/safe_network/commit/6e4ea368fdcedb10042b5d8dc94ab02eece47003))
    - support transfers to Ed25519 public keys in addition to using a Wallet or SafeKey URLs ([`c20932f`](https://github.com/maidsafe/safe_network/commit/c20932f5c15fa16ccad907208522b9c9b52bb062))
    - keypair API now returns a new randomly create Ed25519 key pair ([`f2589e0`](https://github.com/maidsafe/safe_network/commit/f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0))
    - minor enhancement to logs when instantiating safe client ([`84260fc`](https://github.com/maidsafe/safe_network/commit/84260fc32473c9a84f5c3e6fd54564a865f9d7fe))
    - return anyhow::Result/Error from all CLI tests ([`f6f0734`](https://github.com/maidsafe/safe_network/commit/f6f07349a7524304b3d6b1c22db65d77be519f4c))
    - remove Error::Unexpected and Error::Unknown errors from API ([`e692bec`](https://github.com/maidsafe/safe_network/commit/e692becbbf09e2500284cb1507916fac56149f02))
    - Version change: sn_api-v0.17.0; sn_cli-v0.17.0; sn_authd-v0.0.13 ([`23365d4`](https://github.com/maidsafe/safe_network/commit/23365d409b1a538b2eb8c5138623a409e45f9601))
    - temporarily disable running tests in CI ([`77e1c69`](https://github.com/maidsafe/safe_network/commit/77e1c693735760cf1c24c9a4552c9104e52cada0))
    - allocate testcoin to keypair generated for app upon being authorised the very first time ([`9f18fb5`](https://github.com/maidsafe/safe_network/commit/9f18fb51a8c173ebb07e3ab9cea7a8f66b373fc5))
    - updates and enhancements to the User Guide, and to some commands help messages ([`40bcd0f`](https://github.com/maidsafe/safe_network/commit/40bcd0f46dad6177b0052b73393d7789fd559b33))
    - Update sn_api/src/api/authenticator/mod.rs ([`7209603`](https://github.com/maidsafe/safe_network/commit/72096033f5bacdfbeb5c4a5da0a4b52d74f499de))
    - check for existing balance + err when creating acct ([`7d2bd2f`](https://github.com/maidsafe/safe_network/commit/7d2bd2f7ad27724866d04b7e10d69418f6decad9))
    - updating CLI User Guide ([`e107e13`](https://github.com/maidsafe/safe_network/commit/e107e1314957053db2d71357450cac65cba52a68))
    - store Blob API was returning the wrong xorname ([`3694efb`](https://github.com/maidsafe/safe_network/commit/3694efb16b16fcfc0d34db51187c043d0e24f09c))
    - adapt wallet tests and minor refactoring ([`7fb6bd9`](https://github.com/maidsafe/safe_network/commit/7fb6bd96a8bdaaee64592b5dc02596b9f6220165))
    - adapt tests to new transfer costs and minor refactor to transfers errors handling ([`7746947`](https://github.com/maidsafe/safe_network/commit/774694795114dc392db5219393fa63f204fcc905))
    - re-enable implementation for coins transfers with Ed25519 keys ([`b2e3faf`](https://github.com/maidsafe/safe_network/commit/b2e3faf9b0943ec779b1e513c76179048dbb0db3))
    - keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey ([`5a30bf3`](https://github.com/maidsafe/safe_network/commit/5a30bf331242ba8dd9b3189dc255b134fdf24587))
    - ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes ([`01cc289`](https://github.com/maidsafe/safe_network/commit/01cc2894b37908377eb822a826f46c7fef39347e))
    - remove unwrap instances from prod and test code ([`422547f`](https://github.com/maidsafe/safe_network/commit/422547f9081de77538f2241c727ac55b00e1e48b))
    - minor adjustments on cli test utilities ([`e3235b0`](https://github.com/maidsafe/safe_network/commit/e3235b0329c74b20092278c628a2115d50e206d7))
    - properly serialise key pairs in CLI commands output ([`ae026bb`](https://github.com/maidsafe/safe_network/commit/ae026bb9ce91b1373b8b300c41bfef0c3f295c7a))
    - minor reorganisation to cli test scripts ([`f9e0729`](https://github.com/maidsafe/safe_network/commit/f9e07293ea1f8cd5e4428d95a299ba06c0f30a20))
    - insert and retrieve authorised apps keypairs into/from the Safe ([`d8186c3`](https://github.com/maidsafe/safe_network/commit/d8186c309a7e6ca4862fb8855da6636a9f0d84c6))
    - minor renamings in authd status report with new terminology ([`5ac36cc`](https://github.com/maidsafe/safe_network/commit/5ac36cc64566561f4d442058c91b9857622e6f26))
    - populate each keypair generated for authorised apps with testcoins ([`b916d7b`](https://github.com/maidsafe/safe_network/commit/b916d7bdc83e9a02fd29e9cbd6623fc922066a6c))
    - adapt to latest sn-client api changes and further simplification of auth messages ([`b6eddcb`](https://github.com/maidsafe/safe_network/commit/b6eddcbf5d272e6a4430cfd6488f5236bef92a5d))
    - update credentials location ([`7b6445a`](https://github.com/maidsafe/safe_network/commit/7b6445a5b9903b1704c45759878bced097bcb82c))
    - simplify authd messages format and serialisation ([`e814ff3`](https://github.com/maidsafe/safe_network/commit/e814ff3b8c58ae7741938a1c73a22c87ed602883))
    - reenable use of connect from auth responses ([`d64d0fd`](https://github.com/maidsafe/safe_network/commit/d64d0fd9c0d6cd9002405985ff03f6d9ff7aa695))
    - fmt ([`b261640`](https://github.com/maidsafe/safe_network/commit/b261640cebdaf4f7fb1b9c13e911bf82bb46f33a))
    - simplify loop on err ([`274de45`](https://github.com/maidsafe/safe_network/commit/274de45a63122c9685617c97272deb603dd9a15b))
    - tidying up some logs ([`f9c19fa`](https://github.com/maidsafe/safe_network/commit/f9c19faa33db0fb76d392d8cbcd7d910131fedb2))
    - unwrap_or_else -> _or_ for empty funcs ([`e892a99`](https://github.com/maidsafe/safe_network/commit/e892a99f2feb0c36204cfff103b13ca2e5e96388))
    - loop seq gets w/ timeout ([`0cd75fb`](https://github.com/maidsafe/safe_network/commit/0cd75fbd9aceace5fbfc4b726a48d652177d61ce))
    - fix lint issues ([`9c3adff`](https://github.com/maidsafe/safe_network/commit/9c3adffd0889f045ac19110072a194072d294705))
    - adapt authd client api and service names to new terminology of Safe creation and unlocking ([`58ecf7f`](https://github.com/maidsafe/safe_network/commit/58ecf7fbae95dee0a103ce39d61efaac6e2cf550))
    - store generated keypairs on the network ([`805298f`](https://github.com/maidsafe/safe_network/commit/805298fa5f4d1455015d2aa248143f460b4057ba))
    - unused variable `address` ([`eb88c0c`](https://github.com/maidsafe/safe_network/commit/eb88c0cc1a8e7a21d7af4004c1e1a0a49297f22c))
    - remove repeated data storing in SafeAppClient::store_sequence() ([`87d64dc`](https://github.com/maidsafe/safe_network/commit/87d64dc3fe865b68f014da373aa253caf50d6512))
    - generate_random_ed_keypair ([`19ca515`](https://github.com/maidsafe/safe_network/commit/19ca515ac84d8cf5d99c4b2cba25561248597f13))
    - fix proptest regex ([`4db2c27`](https://github.com/maidsafe/safe_network/commit/4db2c27badcccac6d7368566ee6f483613c3aa93))
    - update sn_client ([`52c8c8d`](https://github.com/maidsafe/safe_network/commit/52c8c8d7d2f5a5f9c7d2862bcf3fca1902048695))
    - update pass/phrase derivation proptest ([`cfc3580`](https://github.com/maidsafe/safe_network/commit/cfc35809120cdf2144c8df5dc509eb316bcb0068))
    - clippy ([`eb2364c`](https://github.com/maidsafe/safe_network/commit/eb2364c77d6e755d6e184735e50db366faf266d2))
    - add login/creation seedable pk tests ([`9272bb5`](https://github.com/maidsafe/safe_network/commit/9272bb55edf690ccd33de5904530e7ff8036c0fe))
    - reliably derives seed ([`0e6ffb0`](https://github.com/maidsafe/safe_network/commit/0e6ffb0a3c216d891e6a60ac162d733d2fa28690))
    - clippy ([`2de7f66`](https://github.com/maidsafe/safe_network/commit/2de7f66f3f732b9dae55dad50f15888513b5a125))
    - setting up for no ClientId ([`b5a6d81`](https://github.com/maidsafe/safe_network/commit/b5a6d8115ad3975a17dd973430480adf6c483490))
    - Batch of changes for sk handling ([`29a978f`](https://github.com/maidsafe/safe_network/commit/29a978f3047464ad8014817e331218372b53c06c))
    - dep updates for dt and client ([`7780541`](https://github.com/maidsafe/safe_network/commit/77805418d129cb2924dc35b6a88f704771ef6e5c))
    - converting to more generic data types for keypair sk pk ([`dfabea0`](https://github.com/maidsafe/safe_network/commit/dfabea0a26f97f420f47ba314cae0882aae47dca))
    - small tidy ([`34ef8e8`](https://github.com/maidsafe/safe_network/commit/34ef8e8c31de6e013d3539a0ec595b32f9301194))
    - clippy ([`106407e`](https://github.com/maidsafe/safe_network/commit/106407e8125cc003794ba6249158aa1a655d3357))
    - reenable decoding auth reqs and basic app key generation ([`3f23687`](https://github.com/maidsafe/safe_network/commit/3f23687471b846e3ad1e2492c237a21f212b753f))
    - remove merge added file ([`c56e265`](https://github.com/maidsafe/safe_network/commit/c56e26502ee44af179bfa65c8a194a2190b16842))
    - clippy ([`e0a4de8`](https://github.com/maidsafe/safe_network/commit/e0a4de8de20e5023b83256d39beca19f759ba129))
    - update to reference renamed sn_client ([`5f8d61c`](https://github.com/maidsafe/safe_network/commit/5f8d61cf41eb547c13b65d2030beefd235b75820))
    - tidying up ([`4905fae`](https://github.com/maidsafe/safe_network/commit/4905fae6259063411c5e4ef5fd2afb531980630c))
    - update to reference renamed sn_node crate/repo ([`ee05ed3`](https://github.com/maidsafe/safe_network/commit/ee05ed31cb12d8e1d8bac7569beec90db52a5840))
    - fix merge bugs and readd some shell completion logic ([`b99e7de`](https://github.com/maidsafe/safe_network/commit/b99e7dee3e72e703b47888e3ff03d2baa933b408))
    - Merge branch 'master' into ExploreUpdatesForApis ([`34f9bc7`](https://github.com/maidsafe/safe_network/commit/34f9bc704f301ac903f768813fbd4140cd702f21))
    - rename artifacts and paths to match new naming convention ([`e389ab2`](https://github.com/maidsafe/safe_network/commit/e389ab24f2186fc515b115e736a06d20756ae031))
    - reenabling more transfer funcs ([`d3d04e3`](https://github.com/maidsafe/safe_network/commit/d3d04e36b8fb52ce070aecc2b1f77eb158983427))
    - further ffi cleanup ([`d5c1cd2`](https://github.com/maidsafe/safe_network/commit/d5c1cd2808f9844b06b846ec10dfe05146137023))
    - update safe-cmd-test-utilities name to ([`8f309da`](https://github.com/maidsafe/safe_network/commit/8f309dada1517afa10c263a52f5597429f764890))
    - remove mock/ffi builds + files ([`8b9b481`](https://github.com/maidsafe/safe_network/commit/8b9b481df5d124857abb02158739a6ded8f02af7))
    - update jsonrpc-quic crate name to qjsonrpc ([`6f2dd39`](https://github.com/maidsafe/safe_network/commit/6f2dd39d783812a9b3abd774b6bebd4cde2d5a1e))
    - upgrade multibase to v0.8.0 ([`08f5fec`](https://github.com/maidsafe/safe_network/commit/08f5fec47809bbf0aea61a939dfb3c909043703f))
    - tests updated for wallet changes ([`95200df`](https://github.com/maidsafe/safe_network/commit/95200df5f310911294ee72153d10d13f2e4fb737))
    - update safe-authd crate name to sn_authd ([`019370c`](https://github.com/maidsafe/safe_network/commit/019370cfd0ace44c656caf45c17248f2a547dbbf))
    - reenable wallet apis ([`873fe29`](https://github.com/maidsafe/safe_network/commit/873fe29ac9042b7ad28a29630d2c048bde3a7634))
    - reenabling map ([`9e19113`](https://github.com/maidsafe/safe_network/commit/9e191132a4281c53bd4872a756888234adfc0e2a))
    - update safe-api repo/crate name to sn_api ([`cb66e8f`](https://github.com/maidsafe/safe_network/commit/cb66e8f5a89872d018e48311738d96173ae8274c))
    - sn_client updated ([`4320a05`](https://github.com/maidsafe/safe_network/commit/4320a059224ef6018f7fc067f4a40a6534beeebb))
    - use dirs_next for dir finding ([`426158f`](https://github.com/maidsafe/safe_network/commit/426158fcbb6d7c1fe44755c138bba1ac825a0a0c))
    - clippy ([`0b17fdd`](https://github.com/maidsafe/safe_network/commit/0b17fddbe3418531df1d03a82d4eb510b819b811))
    - getting tests compiling ([`532aed1`](https://github.com/maidsafe/safe_network/commit/532aed1ed8e6b3957627ff2cc2f9d10d87fe3cb2))
    - reenabling some authd functionality ([`5a1cd27`](https://github.com/maidsafe/safe_network/commit/5a1cd2790b159e35c734dfb1fe64a43ea4409dfc))
    - reenabling some money apis ([`0a5c18d`](https://github.com/maidsafe/safe_network/commit/0a5c18d115820f7124050bc0a246503b5cc63fd9))
    - setting up IPC for auth ([`b994b8d`](https://github.com/maidsafe/safe_network/commit/b994b8d6ec1fcfc540e91aa9df79ba849aee7647))
    - sn_data_type updates ([`b863e7e`](https://github.com/maidsafe/safe_network/commit/b863e7eb299472b0c9dbd633b1b892cc221efb46))
    - use core ([`371e7f0`](https://github.com/maidsafe/safe_network/commit/371e7f00e6463063c99beb9823d8684355359d2a))
    - moving in basics of auth func into the repo ([`07dd195`](https://github.com/maidsafe/safe_network/commit/07dd1956d4a53e2f4d09310b48e2a43a3a10e795))
    - safe_nd -> sn_data_types ([`4466c48`](https://github.com/maidsafe/safe_network/commit/4466c48a3fcec76f6c90cf6fcf1f28b177978c90))
    - initial tweaks for app / auth changes ([`1e4c47a`](https://github.com/maidsafe/safe_network/commit/1e4c47aed1aeed3488e370ab0c33a7b5519e40f5))
</details>

