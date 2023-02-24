# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## [0.60.0](https://github.com/maidsafe/sn_client/compare/v0.59.3...v0.60.0) (2021-06-11)


### ⚠ BREAKING CHANGES

* sn_messaging bump to 35

### Features

* update sn_messaging ([1656625](https://github.com/maidsafe/sn_client/commit/165662508532ec343bcf367e53a8f4b1f54d128e))

### [0.59.3](https://github.com/maidsafe/sn_client/compare/v0.59.2...v0.59.3) (2021-06-10)

### [0.59.2](https://github.com/maidsafe/sn_client/compare/v0.59.1...v0.59.2) (2021-06-10)


### Features

* discard blob errors if we get a positive one instead ([cc131a2](https://github.com/maidsafe/sn_client/commit/cc131a22e1d9cb177c6cd598810a50b22ade65be))

### [0.59.1](https://github.com/maidsafe/sn_client/compare/v0.59.0...v0.59.1) (2021-06-10)


### Features

* always reconnect to elders ([1a3afa3](https://github.com/maidsafe/sn_client/commit/1a3afa3117d5f44036b48bf6799f2695cc3dfd78))

## [0.59.0](https://github.com/maidsafe/sn_client/compare/v0.58.0...v0.59.0) (2021-06-09)


### ⚠ BREAKING CHANGES

* sn_messaging bump non-backward compatible.

* changes to upgrade sn_messaging to v34.0 ([3d405a7](https://github.com/maidsafe/sn_client/commit/3d405a73bf555e0d9aef32ae71c3cd92d322e52a))

## [0.58.0](https://github.com/maidsafe/sn_client/compare/v0.57.5...v0.58.0) (2021-06-08)


### ⚠ BREAKING CHANGES

* sn_messaging bump

### Features

* update sn_messaging ([458ec64](https://github.com/maidsafe/sn_client/commit/458ec6471fd2e962e0b6b55679d92e048bc212fc))


### Bug Fixes

* handle history failure more properly ([f46f025](https://github.com/maidsafe/sn_client/commit/f46f025a79f7ac5fb0f5e9baf13e69fea110aebf))

### [0.57.5](https://github.com/maidsafe/sn_client/compare/v0.57.4...v0.57.5) (2021-06-07)

### [0.57.4](https://github.com/maidsafe/sn_client/compare/v0.57.3...v0.57.4) (2021-06-05)

### [0.57.3](https://github.com/maidsafe/sn_client/compare/v0.57.2...v0.57.3) (2021-06-04)


### Bug Fixes

* **messaging:** don't discard error responses for Blob Queries ([c81a35a](https://github.com/maidsafe/sn_client/commit/c81a35abe96de7b397a8cc3780e6870bd0a98c33))

### [0.57.2](https://github.com/maidsafe/sn_client/compare/v0.57.1...v0.57.2) (2021-06-03)


### Bug Fixes

* **query:** timeout when we don't get a response for a query ([9e6b782](https://github.com/maidsafe/sn_client/commit/9e6b7827f8cd054200a61887bc20c33e634b93bb))

### [0.57.1](https://github.com/maidsafe/sn_client/compare/v0.57.0...v0.57.1) (2021-06-03)


### Features

* add payment buffer for storecost fluctuations ([a4eecfa](https://github.com/maidsafe/sn_client/commit/a4eecfa035dca6b6c39e131f36b5204be6e7c0c6))

## [0.57.0](https://github.com/maidsafe/sn_client/compare/v0.56.1...v0.57.0) (2021-06-02)


### ⚠ BREAKING CHANGES

* **messaging:** sn_messaging updated

Also expands some logging

### Features

* **messaging:** Update sn_messaging ([19852a3](https://github.com/maidsafe/sn_client/commit/19852a343fc287269257a1895b81f659988465cd))

### [0.56.1](https://github.com/maidsafe/sn_client/compare/v0.56.0...v0.56.1) (2021-05-31)

## [0.56.0](https://github.com/maidsafe/sn_client/compare/v0.55.2...v0.56.0) (2021-05-24)


### ⚠ BREAKING CHANGES

* **session:** removing the EndUser registration step from the boostrapping stage

### Features

* **session:** make Sessions agnostic of the keypair used to sign each individual client message ([cbe16fd](https://github.com/maidsafe/sn_client/commit/cbe16fd8ea78bbf6ac44c99831f31ae21629420d))

### [0.55.2](https://github.com/maidsafe/sn_client/compare/v0.55.1...v0.55.2) (2021-05-20)

### [0.55.1](https://github.com/maidsafe/sn_client/compare/v0.55.0...v0.55.1) (2021-05-19)


### Features

* **examples:** add a simple example using Blob API ([5c5e764](https://github.com/maidsafe/sn_client/commit/5c5e764e5052d00301e269d1ff9a27499f23feeb))

## [0.55.0](https://github.com/maidsafe/sn_client/compare/v0.54.13...v0.55.0) (2021-05-19)


### ⚠ BREAKING CHANGES

* **cicd:** This should be bumped with messaging changes

This isn't _actually_ a breaking change, but a bump due to an earlier
commit missing one.

PRs starting with the title `Automated version bump` are auto generated as
part of the CI/CD process and so it is duplicate work running the PR workflow
on them. These changes skip PR CI for them.
This PR also switches the scheduled security audit to only run on the MaidSafe
org repo, not on forks.

### Features

* **cicd:** exclude prs with title 'Automated version bump` ([1e28cf4](https://github.com/maidsafe/sn_client/commit/1e28cf40cbadc394d5ad73f21e91a45add039a60))

### [0.54.13](https://github.com/maidsafe/sn_client/compare/v0.54.12...v0.54.13) (2021-05-19)

### [0.54.12](https://github.com/maidsafe/sn_client/compare/v0.54.11...v0.54.12) (2021-05-18)


### Features

* **errors:** receive CmdErrors from the network ([ee194d5](https://github.com/maidsafe/sn_client/commit/ee194d58f9243e764e581d3f29c067e0bb4722c0))

### [0.54.11](https://github.com/maidsafe/sn_client/compare/v0.54.10...v0.54.11) (2021-05-13)


### Features

* **anti-entropy:** updates for sn_messaging new message enum ([5dfc53c](https://github.com/maidsafe/sn_client/commit/5dfc53cd4618affa63271bab88f59c954b8fcde1))


### Bug Fixes

* rebase atop T5 ([e97ca23](https://github.com/maidsafe/sn_client/commit/e97ca238072fdf69408cdc4181c966f68f863fbe))

### [0.54.10](https://github.com/maidsafe/sn_client/compare/v0.54.9...v0.54.10) (2021-05-12)


### Bug Fixes

* **messaging:** remove all non-bootstrapped elders from local list of elders ([275c353](https://github.com/maidsafe/sn_client/commit/275c353b5fb3595aa7812c3f6ab2066577d63288))

### [0.54.9](https://github.com/maidsafe/sn_client/compare/v0.54.8...v0.54.9) (2021-05-06)


### Features

* **storecost:** handle updated query response ([aa47973](https://github.com/maidsafe/sn_client/commit/aa47973c78f602100567d5946929fa36975ded17))

### [0.54.8](https://github.com/maidsafe/sn_client/compare/v0.54.7...v0.54.8) (2021-05-05)

### [0.54.7](https://github.com/maidsafe/sn_client/compare/v0.54.6...v0.54.7) (2021-05-05)

### [0.54.6](https://github.com/maidsafe/sn_client/compare/v0.54.5...v0.54.6) (2021-05-05)

### [0.54.5](https://github.com/maidsafe/sn_client/compare/v0.54.4...v0.54.5) (2021-05-05)

### [0.54.4](https://github.com/maidsafe/sn_client/compare/v0.54.3...v0.54.4) (2021-05-04)

### [0.54.3](https://github.com/maidsafe/sn_client/compare/v0.54.2...v0.54.3) (2021-05-04)

### [0.54.2](https://github.com/maidsafe/sn_client/compare/v0.54.1...v0.54.2) (2021-05-04)

### [0.54.1](https://github.com/maidsafe/sn_client/compare/v0.54.0...v0.54.1) (2021-05-04)


### Bug Fixes

* **query-listener:** listen to query responses from any of the elders ([b157eee](https://github.com/maidsafe/sn_client/commit/b157eeee20e27db68ccbb0b5ee07c10fc7baf37d))

## [0.54.0](https://github.com/maidsafe/sn_client/compare/v0.53.4...v0.54.0) (2021-05-03)


### ⚠ BREAKING CHANGES

* **deps:** update sn_messaging version to 20.0.1

* **deps:** update sn_messaging version to 20.0.1 ([4c8249d](https://github.com/maidsafe/sn_client/commit/4c8249d22e5cbd2b424dc76bed833656cf39915e))

### [0.53.4](https://github.com/maidsafe/sn_client/compare/v0.53.3...v0.53.4) (2021-04-29)


### Features

* **connMgr:** send queries to the 3 Elders closest to the name of target data ([94526ee](https://github.com/maidsafe/sn_client/commit/94526eede01c3722f671f7b41d43c88dc02cdb75))

### [0.53.3](https://github.com/maidsafe/sn_client/compare/v0.53.2...v0.53.3) (2021-04-29)

### [0.53.2](https://github.com/maidsafe/sn_client/compare/v0.53.1...v0.53.2) (2021-04-28)

### [0.53.1](https://github.com/maidsafe/sn_client/compare/v0.53.0...v0.53.1) (2021-04-28)

## [0.53.0](https://github.com/maidsafe/sn_client/compare/v0.52.19...v0.53.0) (2021-04-28)


### ⚠ BREAKING CHANGES

* **deps:** the updated dependencies have breaking changes

* **deps:** update sn_messaging to 19.0.1 and sn_data_types to ([91709e5](https://github.com/maidsafe/sn_client/commit/91709e555c9747629d3cacc3b1b9e91246b244e7))

### [0.52.19](https://github.com/maidsafe/sn_client/compare/v0.52.18...v0.52.19) (2021-04-27)

### [0.52.18](https://github.com/maidsafe/sn_client/compare/v0.52.17...v0.52.18) (2021-04-27)


### Bug Fixes

* **deps:** use released version of qp2p instead of git branch ([c9b2392](https://github.com/maidsafe/sn_client/commit/c9b23920aa1acb13fc62c488f5d4c1b1fb82bf53))

### [0.52.17](https://github.com/maidsafe/sn_client/compare/v0.52.16...v0.52.17) (2021-04-26)

### [0.52.16](https://github.com/maidsafe/sn_client/compare/v0.52.15...v0.52.16) (2021-04-22)


### Bug Fixes

* **test:** fix assertion in blob_deletions test ([ad7d2ab](https://github.com/maidsafe/sn_client/commit/ad7d2ab7d46fc114856be799f7914ed4d640ce3c))

### [0.52.15](https://github.com/maidsafe/sn_client/compare/v0.52.14...v0.52.15) (2021-04-22)

### [0.52.14](https://github.com/maidsafe/sn_client/compare/v0.52.13...v0.52.14) (2021-04-21)


### Features

* **api:** adding new Register data type API ([c567542](https://github.com/maidsafe/sn_client/commit/c567542a49dc728f2e208152093f454dc4907715))

### [0.52.13](https://github.com/maidsafe/sn_client/compare/v0.52.12...v0.52.13) (2021-04-21)

### [0.52.12](https://github.com/maidsafe/sn_client/compare/v0.52.11...v0.52.12) (2021-04-21)

### [0.52.11](https://github.com/maidsafe/sn_client/compare/v0.52.10...v0.52.11) (2021-04-20)


### Features

* **api:** expose Blob utility API to be able to generate a data map without needing to connect to the network ([817efee](https://github.com/maidsafe/sn_client/commit/817efee20a6d4ff3f1170d0c3142f71891389e79))

### [0.52.10](https://github.com/maidsafe/sn_client/compare/v0.52.9...v0.52.10) (2021-04-12)

### [0.52.9](https://github.com/maidsafe/sn_client/compare/v0.52.8...v0.52.9) (2021-04-08)

### [0.52.8](https://github.com/maidsafe/sn_client/compare/v0.52.7...v0.52.8) (2021-04-08)

### [0.52.7](https://github.com/maidsafe/sn_client/compare/v0.52.6...v0.52.7) (2021-04-07)


### Bug Fixes

* **bootstrap:** reverting previous change to again wrap endpoint.connect_to() with a timeout ([0d51c57](https://github.com/maidsafe/sn_client/commit/0d51c57bba21b5ef914576d537db5ba3ac6fddc7))

### [0.52.6](https://github.com/maidsafe/sn_client/compare/v0.52.5...v0.52.6) (2021-04-07)

### [0.52.5](https://github.com/maidsafe/sn_client/compare/v0.52.4...v0.52.5) (2021-04-07)


### Bug Fixes

* **bootstrap:** wrap endpoint.connect_to() with a timeout ([7ea75b1](https://github.com/maidsafe/sn_client/commit/7ea75b13fcdad521ab49854f964694bc58d85227))

### [0.52.4](https://github.com/maidsafe/sn_client/compare/v0.52.3...v0.52.4) (2021-04-07)


### Bug Fixes

* **connection_manager:** set forward-port to true to use public address ([0e5a21f](https://github.com/maidsafe/sn_client/commit/0e5a21f0642952390982d69d25e6c2781c039c04))

### [0.52.3](https://github.com/maidsafe/sn_client/compare/v0.52.2...v0.52.3) (2021-04-02)


### Bug Fixes

* create listeners before sending queries ([2651bfb](https://github.com/maidsafe/sn_client/commit/2651bfb9715ebf86cce7f683b4fe27eb7dadba1b))

### [0.52.2](https://github.com/maidsafe/sn_client/compare/v0.52.1...v0.52.2) (2021-04-02)


### Bug Fixes

* Logging during conn attempts fixed ([fdeb84f](https://github.com/maidsafe/sn_client/commit/fdeb84f3c125d5774f77d59293f4d1ff64e7e6e3))
* TEMP_HACK: use random port to not throw conn pool out of whack ([311fa30](https://github.com/maidsafe/sn_client/commit/311fa301b5b932b5ed5ec03ff216360742b8624b))

### [0.52.1](https://github.com/maidsafe/sn_client/compare/v0.52.0...v0.52.1) (2021-04-01)

## [0.52.0](https://github.com/maidsafe/sn_client/compare/v0.51.5...v0.52.0) (2021-04-01)


### ⚠ BREAKING CHANGES

* **deps:** the qp2p update includes a breaking change

* **deps:** update qp2p version ([c91c555](https://github.com/maidsafe/sn_client/commit/c91c555a3fe3e4a2faf543134aa1ee322fbde158))

### [0.51.5](https://github.com/maidsafe/sn_client/compare/v0.51.4...v0.51.5) (2021-03-25)


### Bug Fixes

* **bootstrap:** fix bootstrap logic when we get SectionInfo::Redirect ([cd6a24e](https://github.com/maidsafe/sn_client/commit/cd6a24ef46936fde1879bbb6df7b4feeb3ade24d))

### [0.51.4](https://github.com/maidsafe/sn_client/compare/v0.51.3...v0.51.4) (2021-03-24)

### [0.51.3](https://github.com/maidsafe/sn_client/compare/v0.51.2...v0.51.3) (2021-03-24)


### Features

* use known vs all elders, supermajority ([c8ba2b5](https://github.com/maidsafe/sn_client/commit/c8ba2b57d53a0c2b9228223777829b8a9723b61c))
* use supermajority for assesing responses ([8659f62](https://github.com/maidsafe/sn_client/commit/8659f62cea16ddf3ac840c11f6f23cf2e105f916))


### Bug Fixes

* dont connect to elders until we have had a full section response/pk ([a3ec50e](https://github.com/maidsafe/sn_client/commit/a3ec50e1be7110995e65234fa4f7888e9aac712e))

### [0.51.2](https://github.com/maidsafe/sn_client/compare/v0.51.1...v0.51.2) (2021-03-23)

### [0.51.1](https://github.com/maidsafe/sn_client/compare/v0.51.0...v0.51.1) (2021-03-23)


### Bug Fixes

* transfer listener cleanup should happen only once ([66454f7](https://github.com/maidsafe/sn_client/commit/66454f72e675e57d208688068c2f87d00c61fb03))

## [0.51.0](https://github.com/maidsafe/sn_client/compare/v0.50.3...v0.51.0) (2021-03-22)


### ⚠ BREAKING CHANGES

* DT update. Messaging updates

* DT dep update ([ded2602](https://github.com/maidsafe/sn_client/commit/ded260297119a6025b9dcac92889ca3ebf09afc5))

### [0.50.3](https://github.com/maidsafe/sn_client/compare/v0.50.2...v0.50.3) (2021-03-18)


### Bug Fixes

* threshold and error text ([11151d8](https://github.com/maidsafe/sn_client/commit/11151d8b448f3dede5e52cb9dd7b3f674cc348dd))

### [0.50.2](https://github.com/maidsafe/sn_client/compare/v0.50.1...v0.50.2) (2021-03-18)


### Features

* **elders:** remove hard coded elder count ([41b986b](https://github.com/maidsafe/sn_client/commit/41b986ba38ca1b2a2ee3c4f130bad82b22c5d950))

### [0.50.1](https://github.com/maidsafe/sn_client/compare/v0.50.0...v0.50.1) (2021-03-16)


### Bug Fixes

* **bootstrap:** connect to all nodes and dont overwrite qp2p ([bcb31bd](https://github.com/maidsafe/sn_client/commit/bcb31bd410172c9f8c1245a9389b70776f9b7d6a))

## [0.50.0](https://github.com/maidsafe/sn_client/compare/v0.49.0...v0.50.0) (2021-03-11)


### ⚠ BREAKING CHANGES

* **tokio:** new Tokio runtime version is not backward compatible with tokio versions < 1.

* **tokio:** upgrade qp2p to v0.10.1 and tokio to v1.3.0 ([07ce604](https://github.com/maidsafe/sn_client/commit/07ce6045f371b3cdef7c8f23c027b304b506cb2a))

## [0.49.0](https://github.com/maidsafe/sn_client/compare/v0.48.1...v0.49.0) (2021-03-10)


### ⚠ BREAKING CHANGES

* **api:** Policy mutation APIs are removed.

### Features

* **api:** removing APIs that are meant for mutating Seuquence's Policy as they are now immutable ([9ad657b](https://github.com/maidsafe/sn_client/commit/9ad657b366b754c08772c2a446e7e9f7ceff57ea))

### [0.48.1](https://github.com/maidsafe/sn_client/compare/v0.48.0...v0.48.1) (2021-03-04)

## [0.48.0](https://github.com/maidsafe/sn_client/compare/v0.47.2...v0.48.0) (2021-02-25)


### ⚠ BREAKING CHANGES

* **deps:** new version of sn_messaging includes a breaking change

### update

* **deps:** update to the latest version of sn_messaging ([4882ad0](https://github.com/maidsafe/sn_client/commit/4882ad0986c186d7c7d539b2fb1fb9f5fe73dce2))

### [0.47.2](https://github.com/maidsafe/sn_client/compare/v0.47.1...v0.47.2) (2021-02-24)

### [0.47.1](https://github.com/maidsafe/sn_client/compare/v0.47.0...v0.47.1) (2021-02-23)

## [0.47.0](https://github.com/maidsafe/sn_client/compare/v0.46.14...v0.47.0) (2021-02-22)


### ⚠ BREAKING CHANGES

* **deps:** updated sn_messaging version

### Features

* **bootstrap:** update for changes to bootstrap flow ([5af7cbe](https://github.com/maidsafe/sn_client/commit/5af7cbe255722dd7ddcf1a7f7334e317aa7c03d6))
* us our section pk when messaging ([c917b10](https://github.com/maidsafe/sn_client/commit/c917b108733c5765e520f6370ce4f336e8ae7ef2))


### Bug Fixes

* set response elected flag after electing best of the rest ([27726ee](https://github.com/maidsafe/sn_client/commit/27726eeb063500b48116d680659434429771045d))
* **connection_manager:** remove incorrect cloning of session ([67060d1](https://github.com/maidsafe/sn_client/commit/67060d1cb3d67f53d7d3653328ee3acd954deea0))
* clippy ([d060aac](https://github.com/maidsafe/sn_client/commit/d060aac99ab1e4d209e43f5d3f67f2c3b4883aa3))
* double serialization bug ([7ef69a6](https://github.com/maidsafe/sn_client/commit/7ef69a6c224f3a5d25a366f0975acabecf83c662))
* remove unimplemented for match wildcard ([ea22dc8](https://github.com/maidsafe/sn_client/commit/ea22dc80365abee3e164531e18f40d194b30c0b6))
* use msg id new fn for random id ([103beb5](https://github.com/maidsafe/sn_client/commit/103beb584fac08d9e58cd638c54206406eabe72a))


* **deps:** update sn_transfers, sn_messaging, sn_data_types ([4b5d876](https://github.com/maidsafe/sn_client/commit/4b5d876aea68f6252c100f13c6766ea38e67d2d4))

### [0.46.14](https://github.com/maidsafe/sn_client/compare/v0.46.13...v0.46.14) (2021-02-22)

### [0.46.13](https://github.com/maidsafe/sn_client/compare/v0.46.12...v0.46.13) (2021-02-19)

### [0.46.12](https://github.com/maidsafe/sn_client/compare/v0.46.11...v0.46.12) (2021-02-16)

### [0.46.11](https://github.com/maidsafe/sn_client/compare/v0.46.10...v0.46.11) (2021-02-16)

### [0.46.10](https://github.com/maidsafe/sn_client/compare/v0.46.9...v0.46.10) (2021-02-15)

### [0.46.9](https://github.com/maidsafe/sn_client/compare/v0.46.8...v0.46.9) (2021-02-11)


### Features

* **config:** read config file from an optionally provided path ([8d8724b](https://github.com/maidsafe/sn_client/commit/8d8724ba8824d91bc38a16dd144311005698b249))

### [0.46.8](https://github.com/maidsafe/sn_client/compare/v0.46.7...v0.46.8) (2021-02-11)

### [0.46.7](https://github.com/maidsafe/sn_client/compare/v0.46.6...v0.46.7) (2021-02-10)


### Features

* use redirected addesses for elders ([cbd89b5](https://github.com/maidsafe/sn_client/commit/cbd89b564da12d42fdbd62b4af92f80e6bf26cb4))


### Bug Fixes

* doc tests ([e70db31](https://github.com/maidsafe/sn_client/commit/e70db31856d3951a6127ac46e73d8ae754bee112))

### [0.46.6](https://github.com/maidsafe/sn_client/compare/v0.46.5...v0.46.6) (2021-02-09)


### Features

* **test:** enable logger in tests using tracing-subscriber ([448522b](https://github.com/maidsafe/sn_client/commit/448522b7e994df7c13b5203ce7326c40aad900de))

### [0.46.5](https://github.com/maidsafe/sn_client/compare/v0.46.4...v0.46.5) (2021-02-08)

### [0.46.4](https://github.com/maidsafe/sn_client/compare/v0.46.3...v0.46.4) (2021-02-08)

### [0.46.3](https://github.com/maidsafe/sn_client/compare/v0.46.2...v0.46.3) (2021-02-08)


### Features

* remove logging implementation ([cc320a0](https://github.com/maidsafe/sn_client/commit/cc320a04f01625f7a0e94d5c7df32e5d5d990fc8))

### [0.46.2](https://github.com/maidsafe/sn_client/compare/v0.46.1...v0.46.2) (2021-02-04)

### [0.46.1](https://github.com/maidsafe/sn_client/compare/v0.46.0...v0.46.1) (2021-02-03)


### Bug Fixes

* doc tests ([deffab3](https://github.com/maidsafe/sn_client/commit/deffab301c2c18f02c8ce283ffac415cc5fca745))

## [0.46.0](https://github.com/maidsafe/sn_client/compare/v0.45.0...v0.46.0) (2021-02-01)


### ⚠ BREAKING CHANGES

* rename money to token

* rename money to token ([62f816a](https://github.com/maidsafe/sn_client/commit/62f816a5552b09822745c7f50b4d9b9c73824aca))

## [0.45.0](https://github.com/maidsafe/sn_client/compare/v0.44.24...v0.45.0) (2021-02-01)


### ⚠ BREAKING CHANGES

* This updates client creation, Arc<Keypair> is no longer
needed, as they keypair itself contains the Arcs we need.

### Features

* Arc<Keypair>->Keypair updates to accommodate Dt and transfers ([dd23579](https://github.com/maidsafe/sn_client/commit/dd2357943f511a6fd90af837fea208bb1d9a4741))

### [0.44.24](https://github.com/maidsafe/sn_client/compare/v0.44.23...v0.44.24) (2021-01-29)


### Bug Fixes

* adjust for dt updates ([19d77a9](https://github.com/maidsafe/sn_client/commit/19d77a9b35afb9e31a0c42a0bbb0694934fdeac5))
* remove unused method ([ae2453e](https://github.com/maidsafe/sn_client/commit/ae2453e4b91a6972218ac9a393865d7224b9ad5e))

### [0.44.23](https://github.com/maidsafe/sn_client/compare/v0.44.22...v0.44.23) (2021-01-29)


### Features

* set simulated-payouts as a default feature for now ([de6b2c9](https://github.com/maidsafe/sn_client/commit/de6b2c93fc994e0166943199e991befee923df80))

### [0.44.22](https://github.com/maidsafe/sn_client/compare/v0.44.21...v0.44.22) (2021-01-28)


### Bug Fixes

* use latest qp2p & so dont remove good connections from pool ([fa8fbd2](https://github.com/maidsafe/sn_client/commit/fa8fbd2573840533f23186f7b9764150863027d4))

### [0.44.21](https://github.com/maidsafe/sn_client/compare/v0.44.20...v0.44.21) (2021-01-28)

### [0.44.20](https://github.com/maidsafe/sn_client/compare/v0.44.19...v0.44.20) (2021-01-26)


### Features

* update client default config for idle/keep alive time changes. ([547dbdd](https://github.com/maidsafe/sn_client/commit/547dbdd2c7e77b66c8cc5715961c9c68d0fceaf2))
* update elder listeners when incoming messages available ([90f36ee](https://github.com/maidsafe/sn_client/commit/90f36eed6b98b5329f997a22b2c76518a2adc205))

### [0.44.19](https://github.com/maidsafe/sn_client/compare/v0.44.18...v0.44.19) (2021-01-21)


### Features

* warn when we have an unused incoming message stream ([d348a57](https://github.com/maidsafe/sn_client/commit/d348a57729cabbd4e8ac366a901e7d0cdefee45e))

### [0.44.18](https://github.com/maidsafe/sn_client/compare/v0.44.17...v0.44.18) (2021-01-21)

### [0.44.17](https://github.com/maidsafe/sn_client/compare/v0.44.16...v0.44.17) (2021-01-18)


### Features

* **error:** re-export sn_messaging::Error as ErrorMessage on the public API ([e3829b0](https://github.com/maidsafe/sn_client/commit/e3829b0d5d00cc262ca69fef92a1670118162a52))

### [0.44.16](https://github.com/maidsafe/sn_client/compare/v0.44.15...v0.44.16) (2021-01-18)

### [0.44.15](https://github.com/maidsafe/sn_client/compare/v0.44.14...v0.44.15) (2021-01-18)

### [0.44.14](https://github.com/maidsafe/sn_client/compare/v0.44.13...v0.44.14) (2021-01-18)


### Features

* listen for bootstrap response on IncomingMessages also ([f880f98](https://github.com/maidsafe/sn_client/commit/f880f9823e77b3727253f9dee01a304cc4e3eddd))
* **connections:** updates to listen to all messages from each elder ([76c1836](https://github.com/maidsafe/sn_client/commit/76c1836db1eda7cc98e99bdef3d3c336fa03ab7f))


### Bug Fixes

* responses require threshold again ([d86bce4](https://github.com/maidsafe/sn_client/commit/d86bce44b2f59a76942b50d50439f2322a450641))
* **responses:** Remove unneeded shortcircuit and clarify logs in repsonse handling ([a86bbcd](https://github.com/maidsafe/sn_client/commit/a86bbcda6517a25b2080696b0890cf826d86fd65))

### [0.44.13](https://github.com/maidsafe/sn_client/compare/v0.44.12...v0.44.13) (2021-01-05)

### [0.44.12](https://github.com/maidsafe/sn_client/compare/v0.44.11...v0.44.12) (2021-01-05)


### Features

* **errors:** Use updated sn_messaging ([e513ab3](https://github.com/maidsafe/sn_client/commit/e513ab3d737ac43b024d3216a689f36f3af476c6))

### [0.44.11](https://github.com/maidsafe/sn_client/compare/v0.44.10...v0.44.11) (2020-12-30)

### [0.44.10](https://github.com/maidsafe/sn_client/compare/v0.44.9...v0.44.10) (2020-12-29)


### Features

* **errors:** use thiserror for error creation ([bc093b6](https://github.com/maidsafe/sn_client/commit/bc093b6fc5cb43fe1bdfa8214e2f1907935e98a0))

### [0.44.9](https://github.com/maidsafe/sn_client/compare/v0.44.8...v0.44.9) (2020-12-28)

### [0.44.8](https://github.com/maidsafe/sn_client/compare/v0.44.7...v0.44.8) (2020-12-28)


### Features

* **blob:** remove local blob cache ([8a1b871](https://github.com/maidsafe/sn_client/commit/8a1b871ebf70ce5ebcf8aaa9146705b29927f925))

### [0.44.7](https://github.com/maidsafe/sn_client/compare/v0.44.6...v0.44.7) (2020-12-24)

### [0.44.6](https://github.com/maidsafe/sn_client/compare/v0.44.5...v0.44.6) (2020-12-17)


### Bug Fixes

* choose longest history w/ simu payout responses ([d22aa01](https://github.com/maidsafe/sn_client/commit/d22aa0153518d9588b229234f634042baa4c22b4))
* CmdError handling for validation reqs ([a7a7ad4](https://github.com/maidsafe/sn_client/commit/a7a7ad4e732b2d6e8b6fa24397ada97c4593eb8d))

### [0.44.5](https://github.com/maidsafe/sn_client/compare/v0.44.4...v0.44.5) (2020-12-09)

### [0.44.4](https://github.com/maidsafe/sn_client/compare/v0.44.3...v0.44.4) (2020-12-09)

### [0.44.3](https://github.com/maidsafe/sn_client/compare/v0.44.2...v0.44.3) (2020-12-07)

### [0.44.2](https://github.com/maidsafe/sn_client/compare/v0.44.1...v0.44.2) (2020-12-07)

### [0.44.1](https://github.com/maidsafe/sn_client/compare/v0.44.0...v0.44.1) (2020-12-07)

### [0.44.0](https://github.com/maidsafe/sn_client/compare/v0.43.0...v0.44.0) (2020-12-07)


### Features

* add configurable bootstrap to client ([5ad120b](https://github.com/maidsafe/sn_client/commit/5ad120bfc7e734b543fafcb96acf877b32adaeb4))
* remove seq cache ([afc516b](https://github.com/maidsafe/sn_client/commit/afc516b6cb2e8ec0c54a9dc2232f21818ad802b8))
* update lseq data type ([b064eff](https://github.com/maidsafe/sn_client/commit/b064eff303f43c3f1f98d22c1b43aee8dba64b5c))
* **api:** Add get_balance_for api for specificly PK requests ([78847f8](https://github.com/maidsafe/sn_client/commit/78847f8c3e289a87b9088be9f2d166ede11bfad1))
* **apis:** remove get_seq/unseq_map apis, and go for cleaner get_map ([3b47500](https://github.com/maidsafe/sn_client/commit/3b4750082e9ea21193f098045ebac31a27d1dc03))
* **arc:** Require an arc wrapped keypair for init ([38e7ef3](https://github.com/maidsafe/sn_client/commit/38e7ef32ac416336af853cf663a82d57b919c8c3))
* **blob:** expose self_ecnrypt API for dry run ([d3abe53](https://github.com/maidsafe/sn_client/commit/d3abe53d28ee15c1cb758399153e6c6a91a52165))
* **ci:** auto generate dependency graph via CI ([ac13840](https://github.com/maidsafe/sn_client/commit/ac13840c0bcee2db67c38275b83eef2be3e3f24f))
* **conn:** make query response threhsold dynamic ([ebf310a](https://github.com/maidsafe/sn_client/commit/ebf310a38b9506f7241a4c7d4296ee0d14ed28f5))
* **connection_manager:** improve handling of connections ([158ba06](https://github.com/maidsafe/sn_client/commit/158ba0690451e34ed5bdb10e7c771602b1b501fb))
* **connections:** set up listener for events/errors ([deeecc6](https://github.com/maidsafe/sn_client/commit/deeecc62bb65e99663683f6b2712c1156420adbc))
* **err_listener:** implement CmdError listener and fix map data tests ([b57ba9a](https://github.com/maidsafe/sn_client/commit/b57ba9ad2780b280dc884e609b423a091fc8296b))
* **errors:** add error for insufficient elder connections ([357ca33](https://github.com/maidsafe/sn_client/commit/357ca33290f3ab19edfbb3d08f6414004b5a142f))
* **listen:** Initial implementation of listen on network ([b38c9bf](https://github.com/maidsafe/sn_client/commit/b38c9bf922f0a10480e13c98076c6a8b2fa70f18))
* **map:** refactoring Map API ([6b8cabc](https://github.com/maidsafe/sn_client/commit/6b8cabc5c51e7ead597035ede8e4e9676bed8b46))
* **qp2p:** Inital set up to enable listeners for qp2p ([63adbc7](https://github.com/maidsafe/sn_client/commit/63adbc7cbca5736850c880cb2316202bffebd94a))
* **qp2p:** update qp2p version ([41958b3](https://github.com/maidsafe/sn_client/commit/41958b3a0bbcbcc6be9b3ff853d858ae476680d1))
* **rand:** use OsRng instead of thread ([437340a](https://github.com/maidsafe/sn_client/commit/437340af6736d47b1650f6054a3930c60acc298b))
* **self-encrypt:** re add self encryption to client blob apis ([e550dad](https://github.com/maidsafe/sn_client/commit/e550dad3137d240d901077f04bc8cde1a23eed3c))
* **seq:** Sign ops before applying locally + sending to network ([08d43c8](https://github.com/maidsafe/sn_client/commit/08d43c8a35643f25aecd5dc9c03911d1d2291067))
* **seq:** Update to sn_data_types and update seq apis ([ad248a7](https://github.com/maidsafe/sn_client/commit/ad248a7e7fa6ab015ca02f61075642e6dc2ee619))
* **seq:** Use signed ops for sequence append ([62c7d46](https://github.com/maidsafe/sn_client/commit/62c7d46fbd1b11aafac495a26ccabf8dbc6da1df))
* **transfer_id:** Provide u64 and pk of transfer to be used as id ([7bcd6b3](https://github.com/maidsafe/sn_client/commit/7bcd6b310b8fad52124b537a88fc74222b2f66de))
* **transfers:** impl DebitAgreementProof aggregator ([8ad8c39](https://github.com/maidsafe/sn_client/commit/8ad8c395f8ac9838cbba3a71c08b86644cbce647))
* **transfers:** impl StoreCost for data writes ([efaf2b0](https://github.com/maidsafe/sn_client/commit/efaf2b03b2dae6b02ffbc428fb2d816adf3bc8ae))
* instantiate the client w/ fullId not just sk ([79f064f](https://github.com/maidsafe/sn_client/commit/79f064f75e6b106ef3bc04357041b963303f0f9e))


### Bug Fixes

* **blob:** fix blob flows and simplify the self_encryption process for ([423002e](https://github.com/maidsafe/sn_client/commit/423002edb99691f8b32b91d4441b1869644d92ff))
* **blob:** fix bug in blog storage ([d7dec70](https://github.com/maidsafe/sn_client/commit/d7dec705f63221faff2f538263eb87962291885f))
* **blob:** fix writing blob with always our PublicKey ([ae2aea4](https://github.com/maidsafe/sn_client/commit/ae2aea4150d7f33959e08dfb29626ec42ef2ed5c))
* **ci:** fix dependency graph script ([0e178e2](https://github.com/maidsafe/sn_client/commit/0e178e267a726a9d293dcd8b6a8e777dc9a3e8db))
* **connections:** Dont fail when one elder cannot connect ([cf4db2c](https://github.com/maidsafe/sn_client/commit/cf4db2c465aade7ab45443758bd2ae0ebc2a5ed9))
* **connections:** retry failed connections ([f14ebe6](https://github.com/maidsafe/sn_client/commit/f14ebe6b6bee0e035bb0e485753cc8810ce54c53))
* **data_types:** update to new MsgEnvelope ([4d53f14](https://github.com/maidsafe/sn_client/commit/4d53f147c026015fe5c6092695edf75f38b46378))
* **msgs:** fix random msg id generation ([624a5b0](https://github.com/maidsafe/sn_client/commit/624a5b058d4930f9e417ef33e603373e715d7378))
* **new:** Fix simulated payout dot and related tests ([a795d4a](https://github.com/maidsafe/sn_client/commit/a795d4a02fd6c2258534f5b635b8d88a7793f2b9))
* **qp2p:** Fixes for latest qp2p master ([0a5001b](https://github.com/maidsafe/sn_client/commit/0a5001b2fa21e22513a37621e7f35636fe6d840e))
* **seq:** sign op.crdt_op ([a584ef0](https://github.com/maidsafe/sn_client/commit/a584ef0c3ed672cce2cfc5bfee980c681d00d0f6))
* **test:** don't hide exported tests behind #[cfg(test)] ([40d0766](https://github.com/maidsafe/sn_client/commit/40d0766efa2e57013c117e565c01f149dc455d4d))
* **tests:** fix actor and Money transfer tests ([ad67e08](https://github.com/maidsafe/sn_client/commit/ad67e08ebdb981c9558d6b37c39503641defdbd0))
* add testing feature flag again ([21412e0](https://github.com/maidsafe/sn_client/commit/21412e09563daca70585d731ef8cdec9d941ab01))

### [0.43.0](https://github.com/maidsafe/sn_client/compare/safe_core-0.42.1-safe_auth-0.17.1-safe_app-0.17.1...v0.43.0) (2020-07-30)
* fix/clippy: fix minor clippy fix

### [0.42.1] (2020-07-16)
* Update ffi-utils to 0.17.0

### [0.42.0]
* Added of SequenceData APIs
* Removed of AppendOnlyData APIs
* Standardize cargo dependency versioning

### [0.41.3]
* Fix CI deploy

### [0.41.2]
* Update the number of responses required to process a request.

### [0.41.1]
* Update quic-p2p to 0.6.2
* Update sn_data_types to 0.9.0
* Refactor to use updated request/response types

### [0.41.0]
* Use Async/await rust.

### [0.40.0]
* Update quic-p2p to 0.5.0
* Attempt to bootstrap multiple times before returning an error

### [0.39.0]
* Add position and index to get_value
* Refactor the connection manager to use new quic-p2p API
* Always use random port instead of default
* Implement multi-vault connection manager
* Implement the new handshake protocol and manage connection state transitions
* Remove unused imports and linting
* Remove macro_use style
* Add support for GET_NEXT_VERSION in more places
* Expose a new `gen_data_map` API which generates a file's data map without putting the chunks on the network
* Make returned error codes to be positive numbers
* Remove pedantic warnings

### [0.38.1]
* Fix broken master workflow

### [0.38.0]
* Update to sn_data_types 0.7.2
* Update to lazy_static 1.4.0
* Update ffi_utils to 0.15.0
* Use GHA for Android libs build
* Expose `gen_data_map` API which generates a file's data map without putting the chunks on the network

### [0.37.3]
* Make another fix to automatic publishing

### [0.37.2]
* Refactor and reenable client mock tests
* Fix automatic publishing

### [0.37.1]
* Fix automatic deploys and releases

### [0.37.0]
* Remove Rust Sodium dependency

### [0.36.0]
* Update to quic-p2p 0.3.0
* Add `set_config_dir_path` API to set a custom path for configuration files.
* Deprecate the `maidsafe_utilities` and `config_file_handler` dependencies.
* Migrate to GitHub actions for CI / CD for all platforms except Mac OS builds.
* Fix inconsistency with real vault.

### [0.35.0]
* Remove unused `routing` module and fix errors
* Rework MDataKey and MDataValue to use FFI conventions
* Make miscellaneous doc fixes
* Clean up FFI documentation

### [0.34.0]
* Technical release to solve some issues in our automated publishing process

### [0.33.0]
* Remove Routing dependency from safe_core.
* Use quic-p2p for communication with Vaults.
* Use new data types from sn_data_types (AppendOnlyData and unpublished ImmutableData).
* Add Safecoin-related tests and features.
* Use the `stable` branch of the Rust compiler and Rust edition 2018.

### [0.32.1]
* Move module-level documentation to wiki, replace with link.
* Make general documentation fixes and improvements.
* Fix some compiler errors.

### [0.32.0]
* Switch to base32 encodings for case-insensitive URIs for IPC
* Send a mock bit with ipc messages so that mock and non-mock components trying to communicate results in an error
* Fix the mock-routing bug which was resulting in corrupted MockVault files
* Remove `is_mock_build` function, replace with `auth_is_mock` and `app_is_mock`

### [0.31.0]
* Refactor `Client` struct to a trait for a better separation of concerns
* Implement `CoreClient` as a bare-bones network client for tests
* Move Authenticator-related `Client` functions to `safe_authenticator`

### [0.30.0]
* Use rust 1.26.1 stable / 2018-02-29 nightly
* rustfmt-nightly 0.8.2 and clippy-0.0.206
* Updated license from dual Maidsafe/GPLv3 to GPLv3
* Add `MDataEntry` struct
* Implement bindings generation

### [0.29.0]
* Use rust 1.22.1 stable / 2018-01-10 nightly
* rustfmt 0.9.0 and clippy-0.0.179
* Fix naming conventions in callback parameters and elsewhere

### [0.28.0]
* Move `AccessContainerEntry` to safe_core
* Add FFI wrapper for `MDataInfo`
* Add access container entry to `AuthGranted`
* Add `MDataKey` and `MDataValue` structs
* Add function for checking mock-routing status of build
* Add config file functionality with options for unlimited mock mutations, in-memory mock storage, and custom mock vault path.
* Add environment variables to override config options for unlimited mock mutations and custom mock vault path.
* Add support for providing arbitrary user data along with `IpcReq::Unregistered` auth request
* Improve documentation for callback parameters
* Improve NFS tests
* Remove unnecessary constants equivalent to environment variables names

### [0.27.0]
* Improve documentation and fix bugs
* Nonce in the MDataInfo struct is no longer optional. This is a breaking external change
* Remove of the neccessity to pass `--feature testing` to run tests
* Replace all secret keys with drop-in equivalents that implement secure cloning. They don't actually clone the underlying data but instead implicitly share it.

### [0.26.2]
* Update routing to 0.33.2

### [0.26.1]
* Update routing to 0.33.1
* Fix mock vault write mode

### [0.26.0]
* Update routing to 0.33.0
* Decryption in MDataInfo tries both the new and old encryption keys before giving up
* Env var to control in-mem or on-disk storage for mock vault
* Change and improve account packet structure
* Fix mock vault deserialisation

### [0.25.1]
* Update routing to 0.32.2

### [0.25.0]
* Add new functions for operations recovery in the `safe_core::recovery` module (e.g. if a `mutate_mdata_entries` operation is failed with the `InvalidSuccessor` error, it will be retried with an increased version)
* Add new testing features to mock-routing (allowing to override certain requests with predefined responses)
* Improve the NFS test coverage
* Update to use Rust Stable 1.19.0 / Nightly 2017-07-20, clippy 0.0.144, and rustfmt 0.9.0
* Update `routing` to 0.32.0 to include more descriptive Map errors
* Update other dependencies

### [0.24.0]
* Use asynchronous I/O and futures for interfacing with Routing
* Deprecate and remove StructuredData and AppendableData types
* Introduce a new data type instead: Map
* Implement URI-based IPC interaction between apps required for supporting mobile devices
* Integrate with routing 0.31.0
* Move all FFI functions to their own separate crates
* Refactor and simplify the NFS module

### [0.23.0]
* Integrate with routing 0.28.5
* Invitation based account creation support in client (API change)
* Invitation-generator and populator example binary
* New error types for ivitation errors
* Serde instead of rustc-serialize in production
* Use chrono instead of time crate (default serde serialisable)
* Fix bugs concerning to unclaimable SD and re-claiming SD via PUT; test cases updated

### [0.22.4]
* Integrate with routing 0.28.4 (0.28.3 is skipped and is yanked from crates).
* Use rust 1.16.0, nightly-2017-03-16 and clippy 0.0.120
* Add a few trace messages for better diagnostics.
* Cleanup README.md

### [0.22.3]
* Integrate with routing 0.28.2

### [0.22.2]
* Integrate with routing 0.28.0

### [0.22.1]
* API to get MAID-Public signing key.

### [0.22.0]
* New error type - MutationError::DataTooLarge.
* New Delete handling and update of code and test cases.
* New APIs - Put to re-claim deleted data (specify version), make data unclaimable.
* Changes and fixes in mock-routing to conform to routing and vaults for error deduction and error types.

### [0.21.2]
* Serialisation and deserialisation for Sign Keys.
* API for getting Filtered keys from AppendableData.
* Fix accidental name mangling of C function.

### [0.21.1]
* Reverting the commit to remove dir-tag from dir-key: commit e829423 reverts commit 4fbc044.
* Trim credentials in examples to not include a `\n`.

### [0.21.0]
* Removal of base64 indirection as we no longer have JSON interface to `safe_core`.
* Many more test cases to thoroughly check low-level-api
* Add new api's wanted by launcher - ownership assertion, version exposure, more serialisations etc.
* Make tag-types for versioned and unversioned StructuredData MaidSafe constants and remove them from `DirectoryKey`.

### [0.20.0]
* API changed from JSON to direct FFI calls for interfacing with other languages.
* Provide low-level-api for finer grained control for manipulation of MaidSafe data types.
* Provide Private & Public Appendable Data operations and manipulations.
* Code APPEND API.
* Update mock-routing to comply with above changes to mimic basic routing and vault functionality for purposes of independent testing.
* Introduce Object Caching - a method in which `safe_core` keeps cache of object in LRU cache and gives only a POD (u64) handle via FFI.
* Increase test cases performace when using mock routing by not writing data to file for test-cases.
* Dependency update - routing updated to 0.26.0.

### [0.19.0]
* Dependency update - routing updated to 0.23.4.
* Log path exposed to FFI so that frontend is intimated where it is expected to create its log files.
* Dependency on rust_sodium instead of sodiumoxide and removal of libsodium instruction from CI builds.

### [0.18.1]
* Dependency update - routing reduced to 0.23.3 and safe_network_common increased to 0.7.0.

### [0.18.0]
* Requests made to safe_core will now timeout after 2 min if there is no response from routing.
* Self_encrypt write used by safe_core via sequential encryptor will now try to put data onto the Network immediately if possible leading to better progress indication across FFI.
* Logging added to safe_core.
* Accessing DNS will not do a bunch of checks which it used to previously because it lead to erroneous corner cases in which one user could not access websites created by other before they created their own DNS first etc.

### [0.17.0]
* Instead of requiring all 3 of PIN, Keyword and Password, have user type only one secure pass-phrase and derive the required credentials internally.

### [0.16.2]
* Expose get-account-info functionality in FFI for launcher to consume.
* Fix sodiumoxide to v0.0.10 as the new released v0.0.12 does not support rustc-serializable types anymore and breaks builds.
* Update dependencies

### [0.16.1]
* Update Routing to 0.23.2
* Add logging to network events.
* Delete existing log file due to issue in v3 of log4rs which instead of truncating/appending overwrites the existing log file garbling it.
* Rustfmt and clippy errors addressed.
* Error recovery test case.
* Extract sub-errors out of Self Encryption errors and convert them to C error codes for FFI.

### [0.16.0]
* Update dependencies
* Refactor FFI as `Box::into_raw()` is stable
* Refactor FFI to deal with pointer to concrete types instead of ptr to void for more type safety
* Fix undefined behaviour in transmute to unrelated type in FFI
* Fix non-termination of background thread which got exposed after fixing the above
* Reorder Imports
* Resolve many Clippy errors
* Expose functionality to collect stats on GETs/PUTs/POSTs/DELETEs
* Error recovery for failure in intermediary steps of a composite operation (like DNS register and delete).

### [0.15.1]
* Upgrade routing to 0.22.0
* Upgrade safe_network_common to 0.3.0

### [0.15.0]
* Upgrade to new routing and self_encryption.

### [0.14.6]
* Merge safe_ffi into safe_core.

### [0.14.5]
* Updating routing to 0.19.1

### [0.14.4]
* Dependency update

### [0.14.3]
* Dependency update

### [0.14.2]
* Pointing and conforming to Routing 0.15.0
* Removal of feature use-mock-crust
* internal code improvement - removing now-a-one-liner function

### [0.14.1]
* Updated dependencies.

### [0.14.0]
* Migrate to Routing 0.13.0.

### [0.13.1]
* Updated dependencies.

### [0.13.0]
* Added minimal support for mock crust.
* Updated dependencies.

### [0.12.1]
* Updated dependencies.

### [0.12.0]
* Integrated with safe_network_common.
* Response handling in case of errors made complete with reason for errors coded in.
* Mock routing updated to give correct reason in cases for errors. All corresponding test cases update to thoroughly test most of scenarios.

### [0.11.0]
* Reintegrated messaging API.
* Fixed a bug in file metadata serialisation which caused the frontend app to crash on Windows.

### [0.10.0]
* Code made more resilient to precision of time resolution on host machines by including dedicated version counter in file metadata. This is also part of public API.
* self_authentication example gives better error message on trying to hijack pre-existing user network name.
* Updated dependencies.

### [0.9.0]
* Updated response handling in line with network behaviour changes.
* Updated dependencies.

### [0.8.0]
* Nfs and Dns modules and examples merged into safe_core.

### [0.7.0]
* Disconnect event detection and translation to ffi compatible value

### [0.6.1]
* self_encryption updated to 0.2.6

### [0.6.0]
* Migrated to Routing 0.7.0
* Switched LOGIN_PACKET_TYPE_TAG to 0

### [0.5.0]
* Refactored to comply with new routing API
* Compiles and passes tests with Mock with stable Rust

### [0.4.0]
* Refactored to comply with new routing API

### [0.3.1]
* Remove wildcard dependencies

### [0.3.0]
* [MAID-1423](https://maidsafe.atlassian.net/browse/MAID-1423) Rename safe_client to safe_core

### [0.2.1]
* Routing crate updated to version 0.4.*

### [0.2.0]
* [MAID-1295](https://maidsafe.atlassian.net/browse/MAID-1295) Remove all unwraps() AND Check for Ok(r#try!( and see if really required (ie., for error conversion etc)
* [MAID-1296](https://maidsafe.atlassian.net/browse/MAID-1296) Remove unwanted errors and Unexpected should take an &str instead of String
* [MAID-1297](https://maidsafe.atlassian.net/browse/MAID-1297) Evaluate test_utils in client
* [MAID-1298](https://maidsafe.atlassian.net/browse/MAID-1298) Put debug statements
* [MAID-1299](https://maidsafe.atlassian.net/browse/MAID-1299) check for all muts (eg., response_getter etc) and validate if really required
* [MAID-1300](https://maidsafe.atlassian.net/browse/MAID-1300) Error conditions in Mock Routing
* [MAID-1301](https://maidsafe.atlassian.net/browse/MAID-1301) Test cases for Error conditions in Mock
* [MAID-1303](https://maidsafe.atlassian.net/browse/MAID-1303) Address the TODO’s and make temporary fixes as permanent (eg., listening to bootstrapped signal)
* [MAID-1304](https://maidsafe.atlassian.net/browse/MAID-1304) Test cases for TODO's and temp fixes as permanent

### [0.1.5]
* Wait for routing to fire a bootstrap completion event
* Added support for environment logger

### [0.1.4]
* [MAID-1219](https://maidsafe.atlassian.net/browse/MAID-1219) Implement Private and Public types
* [MAID-1249](https://maidsafe.atlassian.net/browse/MAID-1249) Implement Unified Structured Datatype
    - [MAID-1252](https://maidsafe.atlassian.net/browse/MAID-1252) Mock Unified StructuredData and ImmutableData
    - [MAID-1253](https://maidsafe.atlassian.net/browse/MAID-1253) Update Mock Routing to support Mock Unified SturcturedData and ImmutableData
    - [MAID-1222](https://maidsafe.atlassian.net/browse/MAID-1222) Compute size of Structured Data
    - [MAID-1223](https://maidsafe.atlassian.net/browse/MAID-1223) Implement a handler for Storing UnVersioned Structured Data
    - [MAID-1224](https://maidsafe.atlassian.net/browse/MAID-1224) Implement a handler for Retrieving Content of UnVersioned Structured Data
    - [MAID-1225](https://maidsafe.atlassian.net/browse/MAID-1225) Write Test Cases for UnVersioned Structured Data handler
    - [MAID-1230](https://maidsafe.atlassian.net/browse/MAID-1230) Implement a handler for Storing Versioned Structured Data
    - [MAID-1231](https://maidsafe.atlassian.net/browse/MAID-1231) Create MaidSafe Specific configuration directory
    - [MAID-1232](https://maidsafe.atlassian.net/browse/MAID-1232) Write Test Cases for Versioned Structured Data handler
    - [MAID-1226](https://maidsafe.atlassian.net/browse/MAID-1226) Implement Session Packet as UnVersioned Structure DataType
    - [MAID-1227](https://maidsafe.atlassian.net/browse/MAID-1227) Update the test cases in Core API
    - [MAID-1228](https://maidsafe.atlassian.net/browse/MAID-1228) Update the test cases in mock routing framework
    - [MAID-1234](https://maidsafe.atlassian.net/browse/MAID-1234) Update Hybrid Encrypt and Decrypt

### [0.1.3]
* [MAID-1283](https://maidsafe.atlassian.net/browse/MAID-1283) Rename repositories from "maidsafe_" to "safe_"

### [0.1.2]
* [MAID-1209](https://maidsafe.atlassian.net/browse/MAID-1209) Remove NFS API

### [0.1.1]
* Updated dependencies' versions
* Fixed lint warnings caused by latest Rust nightly

### [0.1.0] RUST-2 sprint
* Account Creation
    - Register
    - Login
* Implement Storage API
    - Implement types
        - Implement MetaData, File and DirectoryListing types
    - Implement Helpers
        - Directory Helper
            - Save DirectoryListing
            - Get Directory
            - Get Directory Versions
        - File Helper
            - Create File, update file and Metatdata
            - Get Versions
            - Read File
        - Unit test cases for Directory and File Helpers
    - Implement REST DataTypes
        - Container & Blob types
            - Implement Blob and Container types
        - REST API methods in Container
            - Create Container & Get Container
            - List Containers, Update / Get Container Metadata
            - Delete Container
            - Create Blob
            - List Blobs
            - Get Blob
            - Update Blob Content
            - Get Blob Content
            - List Blob Version
            - Delete Blob
            - Copy Blob
            - Update / Get Blob Metadata
        - Unit test cases for API
    - Implement Version Cache (cache key,(blob/container) info to reduce network traffic)
    - Root Directory handling
* Create Example:
    - Self authentication Example
    - Example to demonstrate Storage API

## v0.79.0 (2023-02-24)

### Chore

 - <csr-id-c112a24847ec6800da444192b35a78a75af65de1/> allow processing to backpressure down to qp2p
 - <csr-id-5a29d0d8ad6853e4bb46bd4c122a8fe80dd2cde2/> update for qp2p recv stream ownership changes
 - <csr-id-67867b1379b9225f4be3d584ea2df5c3b0afca3a/> sn_interface-0.17.10/sn_comms-0.3.5/sn_client-0.79.0/sn_node-0.75.0/sn_api-0.77.0/sn_cli-0.70.0

### New Features

 - <csr-id-ab20e8f98a52021cb3e3caa5a33392e20ac6717f/> sn_client to not have any default cmd/query timeout value set
   - sn_client is now set to send msgs to the network without any default timeout
   values, the user of the lib can still set them with SN_CMD_TIMEOUT and
   SN_QUERY_TIMEOUT env vars according to its own requirements.
 - <csr-id-9fc53e718889986f132daeac6df2b10d294094da/> dbcs without ringcts integration

### Bug Fixes

 - <csr-id-03c84a6f9f1520daafe588e4ac83c4d0596b57a9/> unit test key use error
 - <csr-id-fb3d569089bbd75448f3dfb9f76440ea9bb939b9/> bad error type in test
 - <csr-id-b3a245f04e9d342516335870b294ea705f069e49/> use correct sn_dbc version

### Refactor

 - <csr-id-d682cef91723d778501323aef1f03818d0425ee7/> pass public addr in --first
   The genesis node will output a network contact file which contains its
   own address. This public address was specified by a separate flag but
   was removed, making the genesis incapable of producing a proper contacts
   file. This changes the `--first` flag to be used to pass in that public
   address, which should be done for all genesis nodes, so the proper
   external address can be advertised.
 - <csr-id-e513ab35693a86393925ed5e529dcede1bdbe8b3/> pass public addr in --first
   The genesis node will output a network contact file which contains its
   own address. This public address was specified by a separate flag but
   was removed, making the genesis incapable of producing a proper contacts
   file. This changes the `--first` flag to be used to pass in that public
   address, which should be done for all genesis nodes, so the proper
   external address can be advertised.
 - <csr-id-679591e53ed65fa3f0d78f15b5054cd05085e8d9/> split out ae

### New Features (BREAKING)

 - <csr-id-5b79b839f293cd8786783193b33279ed68a86211/> include msg-id in sn_client::Error::CmdAckValidationTimeout
   - Running all sn-client tests multi-threaded in CI/Bors.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 22 commits contributed to the release over the course of 8 calendar days.
 - 9 days passed between releases.
 - 12 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Revert "chore(release): sn_interface-0.17.10/sn_comms-0.3.5/sn_client-0.79.0/sn_node-0.75.0/sn_api-0.77.0/sn_cli-0.70.0" ([`7d41c76`](https://github.com/maidsafe/safe_network/commit/7d41c763221d52e44e0f3faefbbb0a4d4aeca0a2))
    - Merge #2065 #2117 ([`7f4f814`](https://github.com/maidsafe/safe_network/commit/7f4f8144f68ea235e2508699a7e843d0004028e1))
    - Merge branch 'main' into UseParkingLotDeadlockDetection ([`4eddb41`](https://github.com/maidsafe/safe_network/commit/4eddb41639b8845ed7567d8518199944de62f907))
    - Allow processing to backpressure down to qp2p ([`c112a24`](https://github.com/maidsafe/safe_network/commit/c112a24847ec6800da444192b35a78a75af65de1))
    - Update for qp2p recv stream ownership changes ([`5a29d0d`](https://github.com/maidsafe/safe_network/commit/5a29d0d8ad6853e4bb46bd4c122a8fe80dd2cde2))
    - Pass public addr in --first ([`d682cef`](https://github.com/maidsafe/safe_network/commit/d682cef91723d778501323aef1f03818d0425ee7))
    - Pass public addr in --first ([`e513ab3`](https://github.com/maidsafe/safe_network/commit/e513ab35693a86393925ed5e529dcede1bdbe8b3))
    - Merge #2087 #2107 ([`be64f75`](https://github.com/maidsafe/safe_network/commit/be64f75991cbe72899dd7bde6aab8c1ed66aaae9))
    - Merge branch 'main' into dbc_without_ringct ([`803b158`](https://github.com/maidsafe/safe_network/commit/803b1581880f24267f5b7389cac9e268d42c5702))
    - Merge #2113 ([`eb4f372`](https://github.com/maidsafe/safe_network/commit/eb4f37255646015f5c6a6145b9a476afded1bf59))
    - Chore(general): renaming variants and types - This better reflects the domain. ([`9d126b6`](https://github.com/maidsafe/safe_network/commit/9d126b60e2ac72b7bce0baa0de9b68f2f85e5e56))
    - Unit test key use error ([`03c84a6`](https://github.com/maidsafe/safe_network/commit/03c84a6f9f1520daafe588e4ac83c4d0596b57a9))
    - Merge #2092 ([`82057ec`](https://github.com/maidsafe/safe_network/commit/82057ecb0875217efa47f0bcfaad48b43d29d8aa))
    - Merge branch 'main' into dbc_without_ringct ([`ca4781b`](https://github.com/maidsafe/safe_network/commit/ca4781b551fb40edc71f199c00097eb83ef7cb7b))
    - Bad error type in test ([`fb3d569`](https://github.com/maidsafe/safe_network/commit/fb3d569089bbd75448f3dfb9f76440ea9bb939b9))
    - Sn_client to not have any default cmd/query timeout value set ([`ab20e8f`](https://github.com/maidsafe/safe_network/commit/ab20e8f98a52021cb3e3caa5a33392e20ac6717f))
    - Split out ae ([`679591e`](https://github.com/maidsafe/safe_network/commit/679591e53ed65fa3f0d78f15b5054cd05085e8d9))
    - Sn_interface-0.17.10/sn_comms-0.3.5/sn_client-0.79.0/sn_node-0.75.0/sn_api-0.77.0/sn_cli-0.70.0 ([`67867b1`](https://github.com/maidsafe/safe_network/commit/67867b1379b9225f4be3d584ea2df5c3b0afca3a))
    - Include msg-id in sn_client::Error::CmdAckValidationTimeout ([`5b79b83`](https://github.com/maidsafe/safe_network/commit/5b79b839f293cd8786783193b33279ed68a86211))
    - Use correct sn_dbc version ([`b3a245f`](https://github.com/maidsafe/safe_network/commit/b3a245f04e9d342516335870b294ea705f069e49))
    - Merge branch 'main' into dbc_without_ringct ([`f4bfef2`](https://github.com/maidsafe/safe_network/commit/f4bfef20db8c718aacef188f0150e07673eba1b0))
    - Dbcs without ringcts integration ([`9fc53e7`](https://github.com/maidsafe/safe_network/commit/9fc53e718889986f132daeac6df2b10d294094da))
</details>

## v0.78.7 (2023-02-14)

<csr-id-b50fa2c502915fbca752354f271ee0d370f5b06e/>

### Chore

 - <csr-id-b50fa2c502915fbca752354f271ee0d370f5b06e/> update sn_launch_tool

### Chore

 - <csr-id-a7799933cfeadfb70fdb7c1b20ebef9982ea9a41/> sn_client-0.78.7/sn_node-0.74.13/sn_cli-0.69.3

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_client-0.78.7/sn_node-0.74.13/sn_cli-0.69.3 ([`a779993`](https://github.com/maidsafe/safe_network/commit/a7799933cfeadfb70fdb7c1b20ebef9982ea9a41))
    - Update sn_launch_tool ([`b50fa2c`](https://github.com/maidsafe/safe_network/commit/b50fa2c502915fbca752354f271ee0d370f5b06e))
</details>

## v0.78.6 (2023-02-13)

<csr-id-7b209d81ce679308c5e4f4123b07558cdd33a6b8/>

### Chore

 - <csr-id-7b209d81ce679308c5e4f4123b07558cdd33a6b8/> sn_client-0.78.6

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 4 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_client-0.78.6 ([`7b209d8`](https://github.com/maidsafe/safe_network/commit/7b209d81ce679308c5e4f4123b07558cdd33a6b8))
    - Merge #1886 ([`6082509`](https://github.com/maidsafe/safe_network/commit/6082509275b3813bc7e5ffe8da5f93bc3ce5fded))
</details>

## v0.78.5 (2023-02-08)

<csr-id-b09329be395afc79c01c8276a83db02fbd8feded/>
<csr-id-c8bbc738158c35020c0a7c4c5108aceb744a0247/>

### Other

 - <csr-id-b09329be395afc79c01c8276a83db02fbd8feded/> enable 40/100mb tests

### Chore

 - <csr-id-c8bbc738158c35020c0a7c4c5108aceb744a0247/> sn_interface-0.17.6/sn_client-0.78.5/sn_node-0.74.5

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.17.6/sn_client-0.78.5/sn_node-0.74.5 ([`c8bbc73`](https://github.com/maidsafe/safe_network/commit/c8bbc738158c35020c0a7c4c5108aceb744a0247))
    - Merge #2079 ([`acca5f3`](https://github.com/maidsafe/safe_network/commit/acca5f30d7ce2080e0cd8ef38f4039412b201e06))
    - Chore: remove redundant enum variant - `ClientDataResponse` variant `CommunicationIssues` was not differentiated on client, and was carrying the exact same error enum as the variant `NetworkIssue`. ([`a34243e`](https://github.com/maidsafe/safe_network/commit/a34243e89d735512a7eee2b6bf3a96d2a9cbea59))
    - Enable 40/100mb tests ([`b09329b`](https://github.com/maidsafe/safe_network/commit/b09329be395afc79c01c8276a83db02fbd8feded))
</details>

## v0.78.4 (2023-02-06)

<csr-id-e967cc4d827c460bb47748decdf564c9cf7e1e6d/>

### Chore

 - <csr-id-e967cc4d827c460bb47748decdf564c9cf7e1e6d/> sn_interface-0.17.3/sn_comms-0.3.0/sn_client-0.78.4/sn_node-0.74.0/sn_cli-0.69.2

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.17.3/sn_comms-0.3.0/sn_client-0.78.4/sn_node-0.74.0/sn_cli-0.69.2 ([`e967cc4`](https://github.com/maidsafe/safe_network/commit/e967cc4d827c460bb47748decdf564c9cf7e1e6d))
    - Merge #2048 ([`ae06c94`](https://github.com/maidsafe/safe_network/commit/ae06c9458ad904863a925f1d2b2e253a67456298))
</details>

## v0.78.3 (2023-02-06)

<csr-id-6446eb9695d9d12f7677e79603697c3ee44dbfb8/>

### New Features (BREAKING)

 - <csr-id-af38f56c7e76a076f0accca7d44a74c055dd74e1/> remove DataQueryVariant

### New Features

 - <csr-id-7a5d6975153f9d78e742e0a799919852bcfc33ab/> pass client msgs onwards with no deserialisation

### Chore

 - <csr-id-6446eb9695d9d12f7677e79603697c3ee44dbfb8/> sn_client-0.78.3

### Bug Fixes

 - <csr-id-5575f4be90839f7d0b56914aaa259a4b890d0b48/> ignore successfully stopped stream on
   If we call finish, but the node has already signalled `stop(0)`, then
   our bytes are already received successfully, so we do not need to error
   out or retry.
 - <csr-id-cbced3b621c2900ead79952dcd2f867c2043e560/> readd InsufficientNodeCount error

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 3 calendar days.
 - 3 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_client-0.78.3 ([`6446eb9`](https://github.com/maidsafe/safe_network/commit/6446eb9695d9d12f7677e79603697c3ee44dbfb8))
    - Merge #2069 ([`3c75ec0`](https://github.com/maidsafe/safe_network/commit/3c75ec0f0a9f51071ed94723e1125911d525666e))
    - Ignore successfully stopped stream on ([`5575f4b`](https://github.com/maidsafe/safe_network/commit/5575f4be90839f7d0b56914aaa259a4b890d0b48))
    - Merge branch 'main' into sap_change_force_dkg_termination ([`876d78a`](https://github.com/maidsafe/safe_network/commit/876d78a911e852b8cc1c33b2130e4cf9b28dd510))
    - Readd InsufficientNodeCount error ([`cbced3b`](https://github.com/maidsafe/safe_network/commit/cbced3b621c2900ead79952dcd2f867c2043e560))
    - Remove DataQueryVariant ([`af38f56`](https://github.com/maidsafe/safe_network/commit/af38f56c7e76a076f0accca7d44a74c055dd74e1))
    - Pass client msgs onwards with no deserialisation ([`7a5d697`](https://github.com/maidsafe/safe_network/commit/7a5d6975153f9d78e742e0a799919852bcfc33ab))
</details>

## v0.78.2 (2023-02-02)

<csr-id-e706848522d6c52d6ed5eddf638376996cc947a9/>
<csr-id-382236271c4ea283168a02d26a797dd43fdf5c52/>
<csr-id-3831dae3e34623ef252298645a43cbafcc923a13/>

### Chore

 - <csr-id-e706848522d6c52d6ed5eddf638376996cc947a9/> add clippy check for unused async

### Chore

 - <csr-id-3831dae3e34623ef252298645a43cbafcc923a13/> sn_interface-0.17.1/sn_fault_detection-0.15.3/sn_comms-0.2.1/sn_client-0.78.2/sn_node-0.73.3/sn_api-0.76.1

### Refactor

 - <csr-id-382236271c4ea283168a02d26a797dd43fdf5c52/> unused async removal

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.17.1/sn_fault_detection-0.15.3/sn_comms-0.2.1/sn_client-0.78.2/sn_node-0.73.3/sn_api-0.76.1 ([`3831dae`](https://github.com/maidsafe/safe_network/commit/3831dae3e34623ef252298645a43cbafcc923a13))
    - Merge #2061 ([`bab8208`](https://github.com/maidsafe/safe_network/commit/bab82087260ac4f1f44e688db2e67ca2387a7175))
    - Add clippy check for unused async ([`e706848`](https://github.com/maidsafe/safe_network/commit/e706848522d6c52d6ed5eddf638376996cc947a9))
    - Unused async removal ([`3822362`](https://github.com/maidsafe/safe_network/commit/382236271c4ea283168a02d26a797dd43fdf5c52))
    - Merge branch 'main' into sap_change_force_dkg_termination ([`7d3665b`](https://github.com/maidsafe/safe_network/commit/7d3665bfe05f61d170229df9f4424c5663b116d5))
</details>

## v0.78.1 (2023-02-01)

<csr-id-55acb9be12fc1b0523e598033f1a86408ddd4581/>
<csr-id-92e7e2b7b7c6fe4d160fe61f6c6db27cf4944ed8/>

### Refactor

 - <csr-id-55acb9be12fc1b0523e598033f1a86408ddd4581/> remove query result indirection

### Chore

 - <csr-id-92e7e2b7b7c6fe4d160fe61f6c6db27cf4944ed8/> sn_client-0.78.1

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_client-0.78.1 ([`92e7e2b`](https://github.com/maidsafe/safe_network/commit/92e7e2b7b7c6fe4d160fe61f6c6db27cf4944ed8))
    - Merge #2058 ([`22b09ec`](https://github.com/maidsafe/safe_network/commit/22b09ec4ab2060d89e0d365a106df9aab2063631))
    - Remove query result indirection ([`55acb9b`](https://github.com/maidsafe/safe_network/commit/55acb9be12fc1b0523e598033f1a86408ddd4581))
</details>

## v0.78.0 (2023-02-01)

<csr-id-69f8ade1ea8bb3e77c169b17ae21a40370bfab58/>
<csr-id-f779144986a6b2b06f550d3a2a4cbc39c64af83d/>
<csr-id-47e0f87d5ccad33cfa82ef80f3648fe8270acaaa/>
<csr-id-9ef9a2f2c8711895b62b82d25cb9d208c464cad6/>

### Chore

 - <csr-id-69f8ade1ea8bb3e77c169b17ae21a40370bfab58/> sn_interface-0.17.0/sn_comms-0.2.0/sn_client-0.78.0/sn_node-0.73.0/sn_api-0.76.0/sn_cli-0.69.0

### Refactor

 - <csr-id-f779144986a6b2b06f550d3a2a4cbc39c64af83d/> idle_timeout from 10s to 70s
   This was the timeout before this pull request. 70s was deduced to be a
   value that gave CI the time to pass many tests. Although this value is
   not optimal in a real world scenario, as routers might close the holes
   where the connections rely on, and thus the connections will actually
   timeout sooner in some cases. An optimal value was previously decided to
   be 18s, as some routers supposedly close holes after 20s.
 - <csr-id-47e0f87d5ccad33cfa82ef80f3648fe8270acaaa/> remove passing parameters to qp2p
   Rely on defaults in qp2p instead, which are sensible. This means the
   idle timeout from now on will be 10s which is currently the default in
   quinn too.

### Refactor (BREAKING)

 - <csr-id-9ef9a2f2c8711895b62b82d25cb9d208c464cad6/> implement new qp2p version
   This introduces quite some changes to the API that hopefully simplifies
   much of the internals of the node and client.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 5 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.17.0/sn_comms-0.2.0/sn_client-0.78.0/sn_node-0.73.0/sn_api-0.76.0/sn_cli-0.69.0 ([`69f8ade`](https://github.com/maidsafe/safe_network/commit/69f8ade1ea8bb3e77c169b17ae21a40370bfab58))
    - Merge #1996 ([`bb7b2db`](https://github.com/maidsafe/safe_network/commit/bb7b2dbcae9c0a67fc0a23c279537df49d88a07a))
    - Idle_timeout from 10s to 70s ([`f779144`](https://github.com/maidsafe/safe_network/commit/f779144986a6b2b06f550d3a2a4cbc39c64af83d))
    - Remove passing parameters to qp2p ([`47e0f87`](https://github.com/maidsafe/safe_network/commit/47e0f87d5ccad33cfa82ef80f3648fe8270acaaa))
    - Implement new qp2p version ([`9ef9a2f`](https://github.com/maidsafe/safe_network/commit/9ef9a2f2c8711895b62b82d25cb9d208c464cad6))
</details>

## v0.77.9 (2023-01-27)

<csr-id-6b923515f0f0cd1b0d1a4ed51d3ca418e41d9916/>
<csr-id-01ff2ccf45dfc9d45c5ad540144d7a4a640830fc/>

### Chore

 - <csr-id-6b923515f0f0cd1b0d1a4ed51d3ca418e41d9916/> fix issues reported by new clippy

### Chore

 - <csr-id-01ff2ccf45dfc9d45c5ad540144d7a4a640830fc/> sn_interface-0.16.18/sn_comms-0.1.4/sn_client-0.77.9/sn_node-0.72.34/sn_api-0.75.5/sn_cli-0.68.6

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 6 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.16.18/sn_comms-0.1.4/sn_client-0.77.9/sn_node-0.72.34/sn_api-0.75.5/sn_cli-0.68.6 ([`01ff2cc`](https://github.com/maidsafe/safe_network/commit/01ff2ccf45dfc9d45c5ad540144d7a4a640830fc))
    - Merge branch 'main' into chore-comms-remove-unused-async ([`e92dd49`](https://github.com/maidsafe/safe_network/commit/e92dd49f38f9b56c7276e86ba79f7fd8f816af76))
    - Merge branch 'main' into RevertDkgCache ([`24ff625`](https://github.com/maidsafe/safe_network/commit/24ff6257f85922090cfaa5fa83044082d3ef8dab))
    - Fix issues reported by new clippy ([`6b92351`](https://github.com/maidsafe/safe_network/commit/6b923515f0f0cd1b0d1a4ed51d3ca418e41d9916))
</details>

## v0.77.8 (2023-01-20)

<csr-id-21af053a5be2317be356e760c2b581c0f870a396/>
<csr-id-4b1bc4edfad3ad25711a4833181a629746abba19/>
<csr-id-04525595bc5de39f85a128cfb691644b71a3fb79/>
<csr-id-783d62461a65eb7c06b0d4f399b97216b6c75519/>

### Chore

 - <csr-id-21af053a5be2317be356e760c2b581c0f870a396/> happy new year 2023
 - <csr-id-4b1bc4edfad3ad25711a4833181a629746abba19/> update sn_client readme
 - <csr-id-04525595bc5de39f85a128cfb691644b71a3fb79/> disabling keep-alive msgs from client to nodes
   - Setting sn_node idle-timeout to 70secs (to match ADULT_RESPONSE_TIMEOUT),
   which allows the node to keep client connections a bit longer since it may
   need more time (when under stress) to send back a response before closing them.
   - Setting sn_client default idle_timeout to match query/cmd timeout values.

### Chore

 - <csr-id-783d62461a65eb7c06b0d4f399b97216b6c75519/> sn_interface-0.16.14/sn_client-0.77.8/sn_node-0.72.25/sn_api-0.75.4/sn_cli-0.68.5

### New Features

 - <csr-id-cf6daa778c1d4278b444f1a61da3513506c14ea9/> expose a public API to query chunks to specific data replicas
   - Exposing also an `sn_api` public API to fetch a file from a specified set of
   data replicas indexes and a `SafeUrl`.

### Bug Fixes

 - <csr-id-986ef817c053c4b5d8de78d13429fa85244228d9/> retry only once when check-replicas test query fails due to diff responses
   - Run e2e sn_client tests in multi-threaded mode.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 20 commits contributed to the release over the course of 23 calendar days.
 - 24 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.16.14/sn_client-0.77.8/sn_node-0.72.25/sn_api-0.75.4/sn_cli-0.68.5 ([`783d624`](https://github.com/maidsafe/safe_network/commit/783d62461a65eb7c06b0d4f399b97216b6c75519))
    - Merge #1930 #1993 ([`1d2a822`](https://github.com/maidsafe/safe_network/commit/1d2a8220f77743b03ff85c6a7083b8ee22534f44))
    - Retry only once when check-replicas test query fails due to diff responses ([`986ef81`](https://github.com/maidsafe/safe_network/commit/986ef817c053c4b5d8de78d13429fa85244228d9))
    - Merge #1964 ([`6f08edb`](https://github.com/maidsafe/safe_network/commit/6f08edb32a0e93c879ddd13cda1abc6e6b098889))
    - Expose a public API to query chunks to specific data replicas ([`cf6daa7`](https://github.com/maidsafe/safe_network/commit/cf6daa778c1d4278b444f1a61da3513506c14ea9))
    - Merge #1951 ([`24ca31f`](https://github.com/maidsafe/safe_network/commit/24ca31fd53c570c7c97849b74ded850c05273353))
    - Happy new year 2023 ([`21af053`](https://github.com/maidsafe/safe_network/commit/21af053a5be2317be356e760c2b581c0f870a396))
    - Merge #1926 #1936 ([`acc88c5`](https://github.com/maidsafe/safe_network/commit/acc88c5d94900c840cb6c3111ef92fc24b0f3a3d))
    - Merge branch 'main' into proposal_refactor ([`c9cf412`](https://github.com/maidsafe/safe_network/commit/c9cf4124bc88d4d739ba6e443b1c429c3f3855e0))
    - Merge #1834 ([`982bdfc`](https://github.com/maidsafe/safe_network/commit/982bdfcb3ab275252895a9887a3d8eabaa99cf4c))
    - Merge branch 'main' into proposal_refactor ([`0bc7f94`](https://github.com/maidsafe/safe_network/commit/0bc7f94c72c374d667a9b455c4f4f1830366e4a4))
    - Feat(storage): use nodes where adults were used - This continues the move over to also using elders for storage. ([`250da72`](https://github.com/maidsafe/safe_network/commit/250da72ea38b82037ae928ac0eeb8c4b91568448))
    - Fix(cmds): eventually try all elders in happy path - The last elder was mistakenly left out previously. - Also fixes so that we only require 1 ack on happy path. ([`729b4ee`](https://github.com/maidsafe/safe_network/commit/729b4eefc7ddb54acd4a5cbf9b8b3241e4ec9546))
    - Chore(ci): add msg-happy-path test job to merge - Adds `check-replicas` to the `msg-happy-path` feature. ([`c276929`](https://github.com/maidsafe/safe_network/commit/c276929180ef3c6524db797efdbbda409070db89))
    - Fix(naming): disambiguate fn name - Renames from `send_to` to `send_msg_and_check_acks`. ([`dfe116c`](https://github.com/maidsafe/safe_network/commit/dfe116cbed79d3f0f42c30105a7210a1e53da6b2))
    - Update sn_client readme ([`4b1bc4e`](https://github.com/maidsafe/safe_network/commit/4b1bc4edfad3ad25711a4833181a629746abba19))
    - Feat(queries): add happy path feature - This allows clients to default to a lower impact interaction with elders (todo: only expanding the impact on failures). - Adds combined feat for both cmd and query happy path. ([`a77d95b`](https://github.com/maidsafe/safe_network/commit/a77d95b57ff179d1f8fedc00529c69204a8f89e0))
    - Feat(cmds): add happy path feature - This allows clients to default to a lower impact interaction with elders, only expanding the impact on failures. ([`21b4167`](https://github.com/maidsafe/safe_network/commit/21b4167f68b7bd145d02dcdf1b5d8f9acb7971a8))
    - Merge #1927 ([`8f7f2a4`](https://github.com/maidsafe/safe_network/commit/8f7f2a4fc2e1d6cabb4f4849510234df4e1255be))
    - Disabling keep-alive msgs from client to nodes ([`0452559`](https://github.com/maidsafe/safe_network/commit/04525595bc5de39f85a128cfb691644b71a3fb79))
</details>

## v0.77.7 (2022-12-27)

<csr-id-a38cd49958df82fd65d0a3f13670693f40a1e6b2/>

### Chore

 - <csr-id-a38cd49958df82fd65d0a3f13670693f40a1e6b2/> sn_interface-0.16.13/sn_client-0.77.7/sn_node-0.72.24

### Bug Fixes

 - <csr-id-220fd52ab3e1bac776ba74793d5042de220bb315/> set default keep-alive interval to be 1/2 of idle_timeout value set
   - By default the sn_client keep_alive msgs interval will now be set to 1/2 the
   value set for the idle_timeout value.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 5 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.16.13/sn_client-0.77.7/sn_node-0.72.24 ([`a38cd49`](https://github.com/maidsafe/safe_network/commit/a38cd49958df82fd65d0a3f13670693f40a1e6b2))
    - Merge #1924 ([`be2cded`](https://github.com/maidsafe/safe_network/commit/be2cdedb19154adf324782d7178f0e25018cd16c))
    - Set default keep-alive interval to be 1/2 of idle_timeout value set ([`220fd52`](https://github.com/maidsafe/safe_network/commit/220fd52ab3e1bac776ba74793d5042de220bb315))
</details>

## v0.77.6 (2022-12-22)

<csr-id-6bef36cadd09bba0bff9171a352813e3e860ee2c/>

### Chore

 - <csr-id-6bef36cadd09bba0bff9171a352813e3e860ee2c/> sn_interface-0.16.11/sn_client-0.77.6/sn_node-0.72.19

### Bug Fixes

 - <csr-id-c4b47f1fa7b3d814a0de236f8a50b2c9f89750f2/> dont bail on join if sap update errors

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.16.11/sn_client-0.77.6/sn_node-0.72.19 ([`6bef36c`](https://github.com/maidsafe/safe_network/commit/6bef36cadd09bba0bff9171a352813e3e860ee2c))
    - Merge #1917 ([`94fecdf`](https://github.com/maidsafe/safe_network/commit/94fecdff1270a7f215095f7419cfa1bb649213ce))
    - Dont bail on join if sap update errors ([`c4b47f1`](https://github.com/maidsafe/safe_network/commit/c4b47f1fa7b3d814a0de236f8a50b2c9f89750f2))
</details>

## v0.77.5 (2022-12-21)

<csr-id-2af98acaa6b078570fa24b7538705c61d6654f9e/>

### Chore

 - <csr-id-2af98acaa6b078570fa24b7538705c61d6654f9e/> sn_client-0.77.5

### New Features

 - <csr-id-79421d660dfcfebc8d8bf3955ee2534cc3d98e2d/> verify members in network knowledge are the expected
   - Additional checks added to all_nodes_joined.sh script to be able
   to detect nodes in the network knowledge which are not those that
   effectively joined the network.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_client-0.77.5 ([`2af98ac`](https://github.com/maidsafe/safe_network/commit/2af98acaa6b078570fa24b7538705c61d6654f9e))
    - Merge #1908 ([`8875a59`](https://github.com/maidsafe/safe_network/commit/8875a59d21db86edf0ca8f4affcc80ad7618231f))
    - Verify members in network knowledge are the expected ([`79421d6`](https://github.com/maidsafe/safe_network/commit/79421d660dfcfebc8d8bf3955ee2534cc3d98e2d))
</details>

## v0.77.4 (2022-12-20)

<csr-id-aed73cfa0eb0dc3271defa7de2a90a96c790bc8d/>

### Chore

 - <csr-id-aed73cfa0eb0dc3271defa7de2a90a96c790bc8d/> sn_interface-0.16.9/sn_client-0.77.4/sn_node-0.72.15

### New Features

 - <csr-id-96e8c7c5315090462e1269c48027cdba1bfea23a/> retry sending msg to peer cleaning up all cached bad connections
   - When sending a msg to a peer, if it fails with an existing cached connection,
   it will keep retrying till it either finds another cached connection which it
   succeeds with, or it cleans them all up from the cache creating a new connection
   to the peer as last attempt.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.16.9/sn_client-0.77.4/sn_node-0.72.15 ([`aed73cf`](https://github.com/maidsafe/safe_network/commit/aed73cfa0eb0dc3271defa7de2a90a96c790bc8d))
    - Merge #1899 ([`d88b5dd`](https://github.com/maidsafe/safe_network/commit/d88b5dd5c8c5799c6896b19a9c4de094943b377f))
    - Retry sending msg to peer cleaning up all cached bad connections ([`96e8c7c`](https://github.com/maidsafe/safe_network/commit/96e8c7c5315090462e1269c48027cdba1bfea23a))
</details>

## v0.77.3 (2022-12-20)

<csr-id-a6addd1dde96833d6629e75b418ac2a244ab31f3/>

### Chore

 - <csr-id-a6addd1dde96833d6629e75b418ac2a244ab31f3/> sn_interface-0.16.7/sn_client-0.77.3/sn_node-0.72.11/sn_api-0.75.3/sn_cli-0.68.3

### Bug Fixes

 - <csr-id-22402ca6acb0215ecfe9b1fdbf306c0f9cb87d95/> genesis_sap is required to create the `SectionTree`
   - The fields of the tree are assumed to be in sync. But it is not the
   case for a newly created tree.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.16.7/sn_client-0.77.3/sn_node-0.72.11/sn_api-0.75.3/sn_cli-0.68.3 ([`a6addd1`](https://github.com/maidsafe/safe_network/commit/a6addd1dde96833d6629e75b418ac2a244ab31f3))
    - Merge #1848 ([`ddaf857`](https://github.com/maidsafe/safe_network/commit/ddaf8571749c142e9960407cfd9cfa94231a36ad))
    - Genesis_sap is required to create the `SectionTree` ([`22402ca`](https://github.com/maidsafe/safe_network/commit/22402ca6acb0215ecfe9b1fdbf306c0f9cb87d95))
</details>

## v0.77.2 (2022-12-16)

<csr-id-01dc60676d5740dc7dd6250edb130b46a33cc168/>
<csr-id-119ae2d7661d162371749b8466cfd2e9b85d910f/>

### Chore

 - <csr-id-01dc60676d5740dc7dd6250edb130b46a33cc168/> fix new clippy warnings

### Chore

 - <csr-id-119ae2d7661d162371749b8466cfd2e9b85d910f/> sn_interface-0.16.3/sn_client-0.77.2/sn_api-0.75.2/sn_cli-0.68.1

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 day passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.16.3/sn_client-0.77.2/sn_api-0.75.2/sn_cli-0.68.1 ([`119ae2d`](https://github.com/maidsafe/safe_network/commit/119ae2d7661d162371749b8466cfd2e9b85d910f))
    - Fix new clippy warnings ([`01dc606`](https://github.com/maidsafe/safe_network/commit/01dc60676d5740dc7dd6250edb130b46a33cc168))
</details>

## v0.77.1 (2022-12-15)

<csr-id-7620ede57d6f01a63380ac144684b5d504ae4fb4/>
<csr-id-80201067111349306a651a3f42a8ca740f48abaa/>
<csr-id-841a004786767c53ab9d60d4a310299d535b86bc/>
<csr-id-89e1e40ed9100b28a1ad5ed196620a6d6415706e/>
<csr-id-82c0cf683f8052374eafbb859176c69d52956c72/>

### Chore

 - <csr-id-7620ede57d6f01a63380ac144684b5d504ae4fb4/> removing unused 'url' dependency
 - <csr-id-80201067111349306a651a3f42a8ca740f48abaa/> use latest 0.33 qp2p
 - <csr-id-841a004786767c53ab9d60d4a310299d535b86bc/> make stream.finish non blocking where we can
 - <csr-id-89e1e40ed9100b28a1ad5ed196620a6d6415706e/> ignore qp2p::SendStream::finish errors
   They dont mean a msg was not sent.

### Chore

 - <csr-id-82c0cf683f8052374eafbb859176c69d52956c72/> sn_interface-0.16.1/sn_client-0.77.1/sn_node-0.72.1/sn_api-0.75.1

### Bug Fixes

 - <csr-id-b67adb74f03e4e8784ec4d391032d9a1eacb847d/> write all Register cmds to disk even if one or more failed
   - When writting Register cmds log to disk, we log and return the error for
   any of them failing, but we don't prevent the rest to be written to disk.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.16.1/sn_client-0.77.1/sn_node-0.72.1/sn_api-0.75.1 ([`82c0cf6`](https://github.com/maidsafe/safe_network/commit/82c0cf683f8052374eafbb859176c69d52956c72))
    - Merge #1887 ([`2b66221`](https://github.com/maidsafe/safe_network/commit/2b6622144178d6a67db1392dfd4929232cb4ca62))
    - Write all Register cmds to disk even if one or more failed ([`b67adb7`](https://github.com/maidsafe/safe_network/commit/b67adb74f03e4e8784ec4d391032d9a1eacb847d))
    - Merge #1885 ([`79439fb`](https://github.com/maidsafe/safe_network/commit/79439fb7c2d3ec01115960a893fcd8ce03da1790))
    - Removing unused 'url' dependency ([`7620ede`](https://github.com/maidsafe/safe_network/commit/7620ede57d6f01a63380ac144684b5d504ae4fb4))
    - Use latest 0.33 qp2p ([`8020106`](https://github.com/maidsafe/safe_network/commit/80201067111349306a651a3f42a8ca740f48abaa))
    - Make stream.finish non blocking where we can ([`841a004`](https://github.com/maidsafe/safe_network/commit/841a004786767c53ab9d60d4a310299d535b86bc))
    - Ignore qp2p::SendStream::finish errors ([`89e1e40`](https://github.com/maidsafe/safe_network/commit/89e1e40ed9100b28a1ad5ed196620a6d6415706e))
</details>

## v0.77.0 (2022-12-13)

<csr-id-812640dd910e8accbb73e831d1f819c8e1c7f6db/>
<csr-id-64b6c35105168b9fa4b0fb9d626ed9552fd0bed3/>
<csr-id-e5b0dda1315a5299131cacd135b1d1ab66ed7073/>
<csr-id-f06b3e75ce97e7c749d2969276ad6533369806bb/>
<csr-id-e344c3149cf39b6b22a91b755d7e8b0a8ca87dec/>
<csr-id-7ac8d43bb3f559d01d9eac829a19e171a401e1a8/>
<csr-id-2691c53daa36b82185a664482a55d9c893dc8439/>
<csr-id-7f288b389175f3165fdca383dfe5f51097cc591f/>
<csr-id-38b8f55121d8b7c461efa6dd0c0407c4fae93418/>
<csr-id-a55b74b4c8f9bede3c91a9426d4687df01138257/>
<csr-id-233bf64f33a632bef2fdaed409888efaa6f10b63/>
<csr-id-667009dc02e6bb17bfaa60e2374d5ab7b75a7be5/>
<csr-id-860f326a9baf7e62d191eec13359fa5313e6956d/>
<csr-id-ee824e7785b8da770b5aa6bba3415a274a4e0d68/>
<csr-id-7405a23895d4d412db3c5f25b1ca85b534e469eb/>
<csr-id-4431c843d5c7f326848826e68c88cc36d8b300b2/>
<csr-id-f2174d4150580c770ed2e7e6a4a457d35dca1383/>
<csr-id-317fdb18ce227bc383f5637e6dd300ec94af20ff/>
<csr-id-c71fcd1fd61a4a2f9075d3d3e5f922b6d06644e6/>
<csr-id-2b9268c2aff5aae5eb1584d2698f282c8beae73e/>
<csr-id-0ef5a2cfd82de58f02a23a96a305b172f27c33c8/>
<csr-id-f15cee66c4bbc0aa8cd8cd81652745726915e595/>
<csr-id-f2dff3636d3bf1446af53790a42a46473079698f/>
<csr-id-1bf23ff2d00b66232267403f94d3fa133416fdd3/>
<csr-id-5b0b8589bc9c90fac6285f626a46907b2dd0e819/>
<csr-id-20003daa9ff41de61fc2a9509364c0483c92d136/>
<csr-id-27cbeb647b90e7105e5b650b436944d3cdd813c5/>
<csr-id-a05d7ca9b7d2319085a2dea9119735e3e44f50c1/>
<csr-id-98abbbe7af8c870faa22d62819691054e07df718/>
<csr-id-f459a85663d93a8a516dab85cf91f378a3982020/>
<csr-id-9aeac110cad307f0013d67589c48c39a184468e4/>
<csr-id-f856d853c0166f35bb2b98b24e3f3c8c09783b2d/>
<csr-id-a736dc6bd6a107d85981c3687cb5bcb39c62653c/>
<csr-id-40ce02d611027f748d7934fa21adb4142422f7fc/>
<csr-id-428d2e9528c567e5ac46256100ecadcf496dd8e1/>
<csr-id-bef40c17867f33e4775af8ee5e9b617d52ee2667/>
<csr-id-c2d0a0c98569b7fbeacad6796d7b245d2c3e6828/>
<csr-id-033d17f355bea910939e094770af73be89e642ad/>
<csr-id-b74d424f41990c5cf048a9472031983e22099947/>
<csr-id-8a569827c64c89c9b7268e7c6b103a42a4e45dae/>
<csr-id-68c54c90eb622a33bed1981ef765adb9bded2d96/>
<csr-id-9a6f4a3cf20852f4b5604bf08a04aba592dca0fa/>
<csr-id-e57d83235f60a16bd7e1ee801f35a599113dc71a/>
<csr-id-0f11b2ed16c765bee3e25f2f25b374e3f06f8e8f/>
<csr-id-5a539a74d0dcca7a8671910d45bbb08ca4382671/>
<csr-id-a6d225ba65c4817ada16b82626910bed3071d2bf/>
<csr-id-411ea371660b5f76a5c3f887f78331e58f8b6961/>
<csr-id-151a22f63a0aaaf94070f3bd0e1f6bd2f9239856/>
<csr-id-100e2ae70d21e141e1ebbc324f8b06e3d3f1a01c/>
<csr-id-03da7f67fff1fa5bb06d60a66dfdb531506dec4c/>
<csr-id-e867df4d5d5be662cf29b6d7dd8efc1333328141/>
<csr-id-b550ed00dd3e3af878758aea1a2efb6eba0e8d66/>
<csr-id-3a860ace8096e84ab5421e409e2d58bcf491ec55/>
<csr-id-16fafb026bb9286b1724dd152587dfd9b23f7b0c/>
<csr-id-213d7c89adf4706ebd928f56ec0954fcc97a810b/>
<csr-id-daaf79dbd780e03a99baf0e19a738a08bf2f954e/>
<csr-id-c7de08209c659ec93557d6ea10e0bcd8c3b74d8b/>
<csr-id-230a6ed7f1f4193fa36b2fbb83bea072f4944c1d/>
<csr-id-b8d7cfad51e1051391288094bf3c4e4263b6eb94/>
<csr-id-6bc0892e50dddbda393eb37abac44e2bf7e626eb/>
<csr-id-a612553e6dd4c21bc1fec34425fcb6fe2875b044/>
<csr-id-d41de505ef9fc0ce7ebb84c1271e1da77ca29f85/>
<csr-id-9f539e9a8dd9c22e7440539114b2fbdaaeb34515/>
<csr-id-093ea5bfc200f940662c5c0e458c38c5c77294a9/>
<csr-id-4b6569ab2a9face420385d29d7baab31d8ca4d1e/>
<csr-id-c3bce22d6f780441207f23ad0355c9081e60f323/>
<csr-id-c3934b9b9bb6122294e40e0810c5f7f8ad59746a/>
<csr-id-10c2420505b85914eb7518c48a3951a5402331cf/>
<csr-id-93c7a054d87df7054224664c4a03c4507bcfbae6/>
<csr-id-9fad752ce1849763ae16cdb42159b9dccf1a13d0/>
<csr-id-633dfc836c10eafc54dedefc53b2cbc9526970bb/>
<csr-id-ab22c6989f55994065f0d859b91e141f7489a722/>
<csr-id-32744bcf6c94d9a7cda81813646560b70be53eb2/>
<csr-id-07d0991fed28d49c9be85d44a3343b66fac076d9/>
<csr-id-452ef9c5778ad88270f4e251adc49ccbc9b3cb09/>
<csr-id-85f4d00e81ac5bf67b6be89d7ff51b7bb1060ed6/>
<csr-id-1152b2764e955edd80fb33921a8d8fe52654a896/>
<csr-id-60e333d4ced688f3382cde513300d38790613692/>
<csr-id-73f5531790ef8817ed3551fd9e4bcbcc7fc6f4f9/>
<csr-id-4affb147382403fa4d60809558c671810307df05/>
<csr-id-6dde9550c7be949fd40b60757f682d082836138b/>
<csr-id-04d0774bbd65967944932456aeb75152ae015ef6/>
<csr-id-02322c68c449863ea475531cd953a05fbb2ee71b/>
<csr-id-38fc82067065e9e67422388c07456d3ea896614b/>
<csr-id-961b6ba0f60c3fa5cd337d8954b4be8cc196274b/>
<csr-id-ff59202d1374c7e5dcc570d50ed8b399fafe488d/>
<csr-id-110d45ea649783e74d1eb3f449fa55aa0baa8e8a/>
<csr-id-f53337e6e0c7c4f804489f0d370d4cc97331597f/>
<csr-id-18bea3c1e56268b28826643e1ff8936dfa6d4896/>
<csr-id-6be0ea16b0ffe2c153c6a13f36916a91fb58cd05/>
<csr-id-b98d8f6b11a19a72187535b188faef0caf8ba578/>
<csr-id-80917f19125222ce6892e45487f2abe098fefd7a/>
<csr-id-bdf50e7ad1214ef4bb48c0a12db8a7700193bb2a/>
<csr-id-004263308ee31a9568c77aa9655dc186fde75e75/>
<csr-id-a973b62a8ef48acc92af8735e7e7bcac94e0092f/>
<csr-id-ea1d0490f0b67a9f39bd98b2bd5830a0f63fbf6e/>
<csr-id-e3bb817e20843f68ee21e9a5dd7e52c8a6e92b88/>

### Chore

 - <csr-id-812640dd910e8accbb73e831d1f819c8e1c7f6db/> retry conns on any LinkError
 - <csr-id-64b6c35105168b9fa4b0fb9d626ed9552fd0bed3/> pass in context to NodeMsg handling
   Thus reducing read lock calls
 - <csr-id-e5b0dda1315a5299131cacd135b1d1ab66ed7073/> minor logging improvements to help debug msgs arriving/processing on client and nodes
 - <csr-id-f06b3e75ce97e7c749d2969276ad6533369806bb/> upgrading qp2p to version 0.31.0
 - <csr-id-e344c3149cf39b6b22a91b755d7e8b0a8ca87dec/> don't assume Register response with more entries as its most up to date replica
   - If a Register's replica is found to contain more entries than another replica retrieved from
   another Adult, it doesn't necessarily mean it's a more up to date version of it, it could be the
   case that the last operation the user has made was to remove an entry from such Register.
 - <csr-id-7ac8d43bb3f559d01d9eac829a19e171a401e1a8/> connect to relevant nodes first
 - <csr-id-2691c53daa36b82185a664482a55d9c893dc8439/> remove force_new_link as unused
 - <csr-id-7f288b389175f3165fdca383dfe5f51097cc591f/> address review comments
 - <csr-id-38b8f55121d8b7c461efa6dd0c0407c4fae93418/> Pass around MyNodeState to avoid holding locks
   For longer running message handling, we now pass around the inital
   MyNodeState. This avoids a tonnnn of read locks and therefore hopefully
   prevents holding up write and reads needlessly.
 - <csr-id-a55b74b4c8f9bede3c91a9426d4687df01138257/> replace `TestSAP` with `TestSapBuilder`
 - <csr-id-233bf64f33a632bef2fdaed409888efaa6f10b63/> make `WireMsg` immutable
 - <csr-id-667009dc02e6bb17bfaa60e2374d5ab7b75a7be5/> remove duplicate strum/strum_macros/heck deps
 - <csr-id-860f326a9baf7e62d191eec13359fa5313e6956d/> criterion 0.3 -> 0.4, tracing-subscriber 0.2 -> 0.3
 - <csr-id-ee824e7785b8da770b5aa6bba3415a274a4e0d68/> bump blsttc to 8.0.0
 - <csr-id-7405a23895d4d412db3c5f25b1ca85b534e469eb/> post-rebase fixes
 - <csr-id-4431c843d5c7f326848826e68c88cc36d8b300b2/> small clippy fix
 - <csr-id-f2174d4150580c770ed2e7e6a4a457d35dca1383/> deterministically order AE msg elders
   This allows us to arbitrarily sort incoming SAP and ensure 1->1 mapping
   for resending a msg to distinct elders (depending on the original order of the initial msg target elders)
 - <csr-id-317fdb18ce227bc383f5637e6dd300ec94af20ff/> tidy up some error types
 - <csr-id-c71fcd1fd61a4a2f9075d3d3e5f922b6d06644e6/> Send AE retries to all elders in updated section
   This should avoid issues where we always send to the same node
   regardless of initial target elder (as "closest" node in a new prefix
   will be the same regardless of the initial elder)
 - <csr-id-2b9268c2aff5aae5eb1584d2698f282c8beae73e/> retry loop for bidi initialisation
 - <csr-id-0ef5a2cfd82de58f02a23a96a305b172f27c33c8/> add in error msg in the event of missing stream during adult query
 - <csr-id-f15cee66c4bbc0aa8cd8cd81652745726915e595/> dont debug log AE bounced_msg
 - <csr-id-f2dff3636d3bf1446af53790a42a46473079698f/> tweaks to reduce use of async
 - <csr-id-1bf23ff2d00b66232267403f94d3fa133416fdd3/> set nodes join interval to 30secs for testnet in sn-api tests job
   - Upgrading qp2p to v0.30.1.
   - Include bi-stream id in logs both on client and node sides.
   - Removing unused sn_api and sn_client test helpers.
   - Adding a 1sec delay in sn-api tests before querying uploaded data.
 - <csr-id-5b0b8589bc9c90fac6285f626a46907b2dd0e819/> cleanup unused deps
 - <csr-id-20003daa9ff41de61fc2a9509364c0483c92d136/> cleanup un-necessary attempts counter
 - <csr-id-27cbeb647b90e7105e5b650b436944d3cdd813c5/> log MsgId in InsufficentAck error
 - <csr-id-a05d7ca9b7d2319085a2dea9119735e3e44f50c1/> only enqueue msgs at start, thereafter process all enqueued
 - <csr-id-98abbbe7af8c870faa22d62819691054e07df718/> remove ExpiringConnection struct
   dont disconnect link when sutting down channel
   
   Allow dropping of link to do all cleanup
 - <csr-id-f459a85663d93a8a516dab85cf91f378a3982020/> use keep alive for elder comms
 - <csr-id-9aeac110cad307f0013d67589c48c39a184468e4/> don't remove peer link if one listener is done... there may be more listeners running?
 - <csr-id-f856d853c0166f35bb2b98b24e3f3c8c09783b2d/> allow too many args
 - <csr-id-a736dc6bd6a107d85981c3687cb5bcb39c62653c/> dont debug log AE bounced_msg
 - <csr-id-40ce02d611027f748d7934fa21adb4142422f7fc/> clients reuse same msg id for same query while looping
 - <csr-id-428d2e9528c567e5ac46256100ecadcf496dd8e1/> deem spentproof shares < 5 data not found
 - <csr-id-bef40c17867f33e4775af8ee5e9b617d52ee2667/> increase query loop time
 - <csr-id-c2d0a0c98569b7fbeacad6796d7b245d2c3e6828/> return error responses event if we dont have 7.
   queries acutally only go to 3 elders.. so we can return errors here, and they should bubble up after apprioraite retires (if that err is data not found eg)
 - <csr-id-033d17f355bea910939e094770af73be89e642ad/> rejig retry loop, use max retries and keep querying if data not found until timeout
   * rejigs data_not_found() for query response to take into account
   EntryNotFound errors too. (This should hopefully stabilise some
   register permission tests)
 - <csr-id-b74d424f41990c5cf048a9472031983e22099947/> fix unintentionally fast query retry time
 - <csr-id-8a569827c64c89c9b7268e7c6b103a42a4e45dae/> check for longest spentproof response
 - <csr-id-68c54c90eb622a33bed1981ef765adb9bded2d96/> removing retry looping in tests
 - <csr-id-9a6f4a3cf20852f4b5604bf08a04aba592dca0fa/> dont manually clean up client peers
   further responses for same opId may come in and succeed. Fastest response would naturally be NotFound... So this would skew results
 - <csr-id-e57d83235f60a16bd7e1ee801f35a599113dc71a/> write query responses as they come in
   as opposed to requiring a channel to be in place.
 - <csr-id-0f11b2ed16c765bee3e25f2f25b374e3f06f8e8f/> cleanup dropped peers
   and bubble up send channel errors to be handled
 - <csr-id-5a539a74d0dcca7a8671910d45bbb08ca4382671/> refactor client ACK receipt
   moves to write to a map on ack receipt, regardless of when it come sin.
   Previously we could miss ACKs if they were in before our DashMap was updated
 - <csr-id-a6d225ba65c4817ada16b82626910bed3071d2bf/> use same MsgId for all Cmds going out
 - <csr-id-411ea371660b5f76a5c3f887f78331e58f8b6961/> dont try and establish connections for ServiceMsgs
 - <csr-id-151a22f63a0aaaf94070f3bd0e1f6bd2f9239856/> rename a helper func
 - <csr-id-100e2ae70d21e141e1ebbc324f8b06e3d3f1a01c/> remove spend retry on client
   The spend retry depends on providing new network knowledge. We will be using another mechanism to
   obtain this knowledge, which is not available at the moment. Once it's available, we'll add the
   retry again.
   
   For now we decided it's best to remove it and only merge the node-side changes.
   
   This also fixes up various changes after the merge of the new SectionsDAG that replaced the
   SecuredLinkedList.
 - <csr-id-03da7f67fff1fa5bb06d60a66dfdb531506dec4c/> optimizations and code cleanup

### Chore

 - <csr-id-ea1d0490f0b67a9f39bd98b2bd5830a0f63fbf6e/> upgrade sn_dbc and blsttc
   Upgrade both of these crates to resolve a publishing issue regarding a crate that had been yanked
   being pulled in to the dependency graph.
 - <csr-id-e3bb817e20843f68ee21e9a5dd7e52c8a6e92b88/> sn_interface-0.16.0/sn_dysfunction-0.15.0/sn_client-0.77.0/sn_node-0.72.0/sn_api-0.75.0/sn_cli-0.68.0

### New Features

<csr-id-9b8b20b1a191f7283624cf3f9953f020d21ed6d8/>
<csr-id-5a39a843c5570993b0e27780a1c2887bbf7a3212/>
<csr-id-3089b2c8d3f3ee099ff9e0880a96720b319e52a4/>
<csr-id-3fd0a00bad2f9ca266a56de2086b54088459e153/>
<csr-id-95436a1f722bfd02a735dc3cf2f171a8b70de552/>
<csr-id-a5bf211daa0272597f1a2d852a17592258a2115a/>
<csr-id-5019dd3896d278227878ffd91ce14d0cecb2b9dd/>
<csr-id-f5d53c188a03150a06bdee97fb585f7900b7c251/>
<csr-id-5c8b1f50d1bf346d45bd2a9faf92bbf33cb448da/>
<csr-id-057ce1ce1e174102e23d96cfcd2ab1d090a6f1dc/>
<csr-id-0cd47ad56e0d93e3e99feb0dfcea8094f871ff6f/>

 - <csr-id-707627f8915a6032390b035786e3e39d1f7bac8d/> allow to change the Elder-to-Adult query responses timeout by env var
   - feat(node): allow to change the timeout for Elder-to-Adult query responses
   by setting `SN_ADULT_RESPONSE_TIMEOUT` env var
* Debugging message to indicate a spend request being processed correctly, which proved useful when
     trying to get the automated test working.
* Remove the current section key from the unknown section key error. It's not necessary to include
     this.
* When running the baby fleming network with the Makefile, include log messages from `sn_interface`.
* Fix up git-based references to `sn_dbc` crate.

### Bug Fixes

<csr-id-4c039335707851c8f7ec71703acfb646184fa30a/>
<csr-id-eccb7090e024e38bf561efa3b7ff2abfe491ba72/>
<csr-id-7f448bcb3106b429c711d6ff81ded5c33138bca7/>
<csr-id-c73b83b0a4ac0e3072b41e1af5b42b90c8a54177/>
<csr-id-bfaf6eac15cfa6e7f829cdc9a79181836a27f473/>
<csr-id-c98cc98674552794960e5953c5dbf405d961c333/>
<csr-id-8f887c0f3f128f5d59304a0b47f6105cb52f3155/>
<csr-id-e4721ab8c3bfcb3379fe09bbd069c1eab0048753/>
<csr-id-54222aeea627209c53a1f595578436deb2763ef0/>

 - <csr-id-de8ed40b9f1ad353c9a8ded58db5de76acee21e1/> reconnect upon any LinkError::Connection(_) error when sending a msg on a bi-stream
   - Upgrading qp2p to v0.32.0.

### Other

 - <csr-id-e867df4d5d5be662cf29b6d7dd8efc1333328141/> run churn tests on GHA infra
   Used only in merge run to avoid overbilling in regular PRs
 - <csr-id-b550ed00dd3e3af878758aea1a2efb6eba0e8d66/> ignore large clippy variants for now
 - <csr-id-3a860ace8096e84ab5421e409e2d58bcf491ec55/> fix prebuilt rg install
 - <csr-id-16fafb026bb9286b1724dd152587dfd9b23f7b0c/> no retries in ci profile
 - <csr-id-213d7c89adf4706ebd928f56ec0954fcc97a810b/> fail fast
 - <csr-id-daaf79dbd780e03a99baf0e19a738a08bf2f954e/> add client test logs
 - <csr-id-c7de08209c659ec93557d6ea10e0bcd8c3b74d8b/> minor refactoring and fixing issue reported by new clippy version
 - <csr-id-230a6ed7f1f4193fa36b2fbb83bea072f4944c1d/> spend with updated network knowledge
   Previously I had a placeholder in for this case, but now have something working.
   
   The test requires having two network sections and one of the input DBCs for a transaction being
   signed by the other section key.
   
   The `TestNodeBuilder` was extended with a function that creates a section without a creating a node,
   and this included being able to provide a section chain and tree.

### Refactor

 - <csr-id-b8d7cfad51e1051391288094bf3c4e4263b6eb94/> use internal send_query API to send initial probe msg to the network
 - <csr-id-6bc0892e50dddbda393eb37abac44e2bf7e626eb/> removing query retrying loops and sleeps in sn-api/sn-client tests
 - <csr-id-a612553e6dd4c21bc1fec34425fcb6fe2875b044/> removing redundant ordering of new target Elders upon handling AE response
 - <csr-id-d41de505ef9fc0ce7ebb84c1271e1da77ca29f85/> remove client internal channels from within Session mod
   - Also additional errors are defined now for the responses listener to report any type
   of failure so they are not considered as a lack of response.
   - Setting a limit to the number of retries upon AntiEntropy responses received for a msg.
 - <csr-id-9f539e9a8dd9c22e7440539114b2fbdaaeb34515/> provide age pattern to generate `NodeInfo`
 - <csr-id-093ea5bfc200f940662c5c0e458c38c5c77294a9/> organize key related test utilites
 - <csr-id-4b6569ab2a9face420385d29d7baab31d8ca4d1e/> organize network_knowledge test utilites
 - <csr-id-c3bce22d6f780441207f23ad0355c9081e60f323/> remove from sn_client
 - <csr-id-c3934b9b9bb6122294e40e0810c5f7f8ad59746a/> improvements to data-replicas check Error types
 - <csr-id-10c2420505b85914eb7518c48a3951a5402331cf/> verify each Chunk individually in the `upload_and_verify` API
   - This shouldn't only reduce the amount of memory used when uploading large files,
   but it shall also be more precise as to when a Chunk is ready for verification after
   uploading it, and which chunk specifically failed the verification.
   - Sending a DataCmd now is not retried but only make sure it returns within
   the configured cmd timeout window.
   - Using `expect()` in some client file tests to make sure Rust return useful info when errors
   occur, there is an issue with Rust test when Err is returned not showing enough failure info.
 - <csr-id-93c7a054d87df7054224664c4a03c4507bcfbae6/> use one channel per Cmd/Query sent to await for responses
 - <csr-id-9fad752ce1849763ae16cdb42159b9dccf1a13d0/> remove some ? noise in tests
 - <csr-id-633dfc836c10eafc54dedefc53b2cbc9526970bb/> AuthKind into MsgKind without node sig
 - <csr-id-ab22c6989f55994065f0d859b91e141f7489a722/> assert_lists asserts instead of returning Result
 - <csr-id-32744bcf6c94d9a7cda81813646560b70be53eb2/> remove `SectionAuthorityProvider`, `SectionTreeUpdate` messages
 - <csr-id-07d0991fed28d49c9be85d44a3343b66fac076d9/> adapt confusing auth related code
 - <csr-id-452ef9c5778ad88270f4e251adc49ccbc9b3cb09/> rename a bunch of auth to sig
 - <csr-id-85f4d00e81ac5bf67b6be89d7ff51b7bb1060ed6/> simplify section sig
 - <csr-id-1152b2764e955edd80fb33921a8d8fe52654a896/> get public commitments from sn_dbc
   The code for retrieving and validating the public commitments was moved out to the sn_dbc crate.
   
   It's needed for both the spend request and test setup code which is going to be referenced in both
   `sn_node` and `sn_client`.
   
   Also fixed a clippy error in `SectionTree::get_signed_by_key`.
 - <csr-id-60e333d4ced688f3382cde513300d38790613692/> bundle proof chain, SAP into `SectionTreeUpdate`
   The `SectionTree` always requires a Proof chain and a SAP to update it,
   hence bundle it together to make it cleaner
    - A `proof chain` is a chain of keys where each key is signed by the previous one
    - A `section chain` is a proof chain to our current section key

### Style

 - <csr-id-73f5531790ef8817ed3551fd9e4bcbcc7fc6f4f9/> applied clippy nightly
   Mainly lints `iter_kv_map` (e.g. use `keys()`, instead of `iter()`, which
   iterates over both keys and values where the value is not used) and `needless_borrow`.

### Test

 - <csr-id-4affb147382403fa4d60809558c671810307df05/> add test for size upload limit
 - <csr-id-6dde9550c7be949fd40b60757f682d082836138b/> do more detailed error matching
 - <csr-id-04d0774bbd65967944932456aeb75152ae015ef6/> reuse code with setup fn
 - <csr-id-02322c68c449863ea475531cd953a05fbb2ee71b/> add the test stubs in sn_client
 - <csr-id-38fc82067065e9e67422388c07456d3ea896614b/> tweak churn test node startup interval to 3s
 - <csr-id-961b6ba0f60c3fa5cd337d8954b4be8cc196274b/> reignore 40mb test until we have stability over ack response time
 - <csr-id-ff59202d1374c7e5dcc570d50ed8b399fafe488d/> feature-gated send_query API to verify data is stored in all replicas for each query
   - Introducing a new `check-replicas` feature to sn_client (as well as sn_api and CLI), which switches
   the `Client::send_query` API behavior to send the query to all Adults replicas and it expects a
   valid response from all of them instead of from just one.
   - Running sn_client, sn_api, and CLI tests in CI with new `check-replicas` feature enabled to verify
   data was stored in all Adults replicas.
   - With `check-replicas` feature enabled, query responses from all Adults replicas are also compared
   to verify they are all the same.
 - <csr-id-110d45ea649783e74d1eb3f449fa55aa0baa8e8a/> igore 40mb test for now
 - <csr-id-f53337e6e0c7c4f804489f0d370d4cc97331597f/> ignore spentbook test that tries to handle client w/o a stream etc
 - <csr-id-18bea3c1e56268b28826643e1ff8936dfa6d4896/> verify client generate entry_hash

### Chore (BREAKING)

 - <csr-id-6be0ea16b0ffe2c153c6a13f36916a91fb58cd05/> attempt to reduce allocations


### New Features (BREAKING)

<csr-id-7106b7533e119dc94bbf19fa304f3eb1f8dc9425/>
<csr-id-718248bfdabfff68de6dd775a9e6292e1c5733b4/>

 - <csr-id-7afd7a95d39098fb5166785c215881233bab528a/> retry once if connection was lost when trying to send on a bi-stream
 - <csr-id-f225a2d84ad3422b4f466fa2bf713c3a767588dc/> adding more context info to some node Error types
   - Initialising logger in sn_client spentbook API tests.

### Refactor (BREAKING)

 - <csr-id-b98d8f6b11a19a72187535b188faef0caf8ba578/> moving the `sn_interface::types::connections` module to the `sn_client` crate
   - This module doesn't belong to the `sn_interface` crate since it's not
   part of the network's protocol/API.
 - <csr-id-80917f19125222ce6892e45487f2abe098fefd7a/> breaking up client msg type separating requests from responses
   - A new messaging type `ClientMsgResponse` is introduced explicitly for client msg responses.
   - With new msg type, a new msg kind `MsgKind::ClientMsgResponse` is introduced which removes
   the need of providing a fake client authority in each of the responses sent by nodes to clients.
 - <csr-id-bdf50e7ad1214ef4bb48c0a12db8a7700193bb2a/> removing unused Error types and adding context info to a couple of them
 - <csr-id-004263308ee31a9568c77aa9655dc186fde75e75/> test helper function for sleeping before querying published Register ops
   - Also removing unused test utilty from sn_api and unused sn_client::Error types.
   - Disabling log instrumentation for large args of `spend_dbc` API.
 - <csr-id-a973b62a8ef48acc92af8735e7e7bcac94e0092f/> removing op id from query response
   - Use the query msg id to generate the operation id to track the response from Adults
   - Remove peers from pending data queries when response was obtained from Adults
   - Removing correlation id from SystemMsg node query/response
   - Redefine system::NodeQueryResponse type just as an alias to data::QueryResponse

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 178 commits contributed to the release over the course of 85 calendar days.
 - 85 days passed between releases.
 - 123 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge #1882 ([`16e82d1`](https://github.com/maidsafe/safe_network/commit/16e82d13cfeee993c85c04f1c6f90e4305c90487))
    - Upgrade sn_dbc and blsttc ([`ea1d049`](https://github.com/maidsafe/safe_network/commit/ea1d0490f0b67a9f39bd98b2bd5830a0f63fbf6e))
    - Sn_interface-0.16.0/sn_dysfunction-0.15.0/sn_client-0.77.0/sn_node-0.72.0/sn_api-0.75.0/sn_cli-0.68.0 ([`e3bb817`](https://github.com/maidsafe/safe_network/commit/e3bb817e20843f68ee21e9a5dd7e52c8a6e92b88))
    - Merge #1870 ([`dd23f77`](https://github.com/maidsafe/safe_network/commit/dd23f7704e689ba2c2fb12be5d03b0ac4ea2c83c))
    - Add test for size upload limit ([`4affb14`](https://github.com/maidsafe/safe_network/commit/4affb147382403fa4d60809558c671810307df05))
    - Fix(chunks): match on specific error - Matching on any error was wrong, but it was hidden until we actually returned any other error than the expected. With `limit-client-upload-size`, we started returning more errors. ([`2699f03`](https://github.com/maidsafe/safe_network/commit/2699f036e0ba5cab539e93b30b0e418569e9bc00))
    - Merge branch 'main' into message_handling ([`80e4030`](https://github.com/maidsafe/safe_network/commit/80e4030820b1380450b86fa6e8c57ee41344a0ed))
    - Merge #1849 ([`248d4f2`](https://github.com/maidsafe/safe_network/commit/248d4f2f84d9c97f0331aedf2e32f9fc8fb7964b))
    - Run churn tests on GHA infra ([`e867df4`](https://github.com/maidsafe/safe_network/commit/e867df4d5d5be662cf29b6d7dd8efc1333328141))
    - Merge branch 'main' into message_handling ([`6d92148`](https://github.com/maidsafe/safe_network/commit/6d92148dc42bf5f53a21f98131523502133fd169))
    - Merge branch 'main' into Fix-PreventHardNodeLoopOnError ([`ddcc8b2`](https://github.com/maidsafe/safe_network/commit/ddcc8b2e8cd76b63b8a434b4b8b4178747636dfb))
    - Merge branch 'main' into Fix-PreventHardNodeLoopOnError ([`1d14544`](https://github.com/maidsafe/safe_network/commit/1d14544e015d02348dac04f4fa20de36316ce616))
    - Merge #1815 ([`ea487fc`](https://github.com/maidsafe/safe_network/commit/ea487fc3f734830997851326d66121e55a7ee9fb))
    - Allow to change the Elder-to-Adult query responses timeout by env var ([`707627f`](https://github.com/maidsafe/safe_network/commit/707627f8915a6032390b035786e3e39d1f7bac8d))
    - Merge #1864 ([`d3bf27b`](https://github.com/maidsafe/safe_network/commit/d3bf27b78c05700d8c9d57000789f260e4f2af15))
    - Retry conns on any LinkError ([`812640d`](https://github.com/maidsafe/safe_network/commit/812640dd910e8accbb73e831d1f819c8e1c7f6db))
    - Merge #1857 ([`b221a08`](https://github.com/maidsafe/safe_network/commit/b221a08705ab8579d3012a10bffad523f94e9c05))
    - Feat(client): limit upload size - Adds a `limit-client-upload-size` feature, under which a (currently hard-coded @10MiB) file size upload limit is enforced. ([`3d40cf5`](https://github.com/maidsafe/safe_network/commit/3d40cf525a9dab3e4371dff3f2c1408a0c140fc4))
    - Merge #1845 ([`9d0f958`](https://github.com/maidsafe/safe_network/commit/9d0f958a0d2bceb9aad7b93b51aa17acf3394b30))
    - Reconnect upon any LinkError::Connection(_) error when sending a msg on a bi-stream ([`de8ed40`](https://github.com/maidsafe/safe_network/commit/de8ed40b9f1ad353c9a8ded58db5de76acee21e1))
    - Merge #1830 ([`dee4ffc`](https://github.com/maidsafe/safe_network/commit/dee4ffc9544b5c979a240bc547ecdef21b5801ca))
    - Moving the `sn_interface::types::connections` module to the `sn_client` crate ([`b98d8f6`](https://github.com/maidsafe/safe_network/commit/b98d8f6b11a19a72187535b188faef0caf8ba578))
    - Merge #1826 ([`2f8a406`](https://github.com/maidsafe/safe_network/commit/2f8a406a95447d9e1bf5d2422fb564d1170542ff))
    - Cap the number of concurrent chunks to be uploaded/retrieved for a file ([`9b8b20b`](https://github.com/maidsafe/safe_network/commit/9b8b20b1a191f7283624cf3f9953f020d21ed6d8))
    - Merge #1820 ([`1bfbdd3`](https://github.com/maidsafe/safe_network/commit/1bfbdd31ce1b132bc468a433a2211c621b95291e))
    - Retry once if connection was lost when trying to send on a bi-stream ([`7afd7a9`](https://github.com/maidsafe/safe_network/commit/7afd7a95d39098fb5166785c215881233bab528a))
    - Merge #1824 ([`9494582`](https://github.com/maidsafe/safe_network/commit/949458280b567aa6dce387b276c06c2cb55d7ca4))
    - Applied clippy nightly ([`73f5531`](https://github.com/maidsafe/safe_network/commit/73f5531790ef8817ed3551fd9e4bcbcc7fc6f4f9))
    - Merge #1819 ([`ec37ad6`](https://github.com/maidsafe/safe_network/commit/ec37ad6142930c59d1aad2325ac09b8d6383484d))
    - Make client receive stream log clearer ([`4c03933`](https://github.com/maidsafe/safe_network/commit/4c039335707851c8f7ec71703acfb646184fa30a))
    - Merge #1818 ([`e95ebba`](https://github.com/maidsafe/safe_network/commit/e95ebba6e50879e9110b21afb5933685a591b85a))
    - Merge #1817 ([`7fd2bb0`](https://github.com/maidsafe/safe_network/commit/7fd2bb09faf12c65712ee25dab3fd08841cf2d4c))
    - Adding more context info to some node Error types ([`f225a2d`](https://github.com/maidsafe/safe_network/commit/f225a2d84ad3422b4f466fa2bf713c3a767588dc))
    - Pass in context to NodeMsg handling ([`64b6c35`](https://github.com/maidsafe/safe_network/commit/64b6c35105168b9fa4b0fb9d626ed9552fd0bed3))
    - Merge #1796 ([`e15180b`](https://github.com/maidsafe/safe_network/commit/e15180b53d1daaec76b7eba4637ffc16076c80af))
    - Merge #1814 ([`90f25b4`](https://github.com/maidsafe/safe_network/commit/90f25b42177034f42433c235f1c4fabd7b0a9a19))
    - Use internal send_query API to send initial probe msg to the network ([`b8d7cfa`](https://github.com/maidsafe/safe_network/commit/b8d7cfad51e1051391288094bf3c4e4263b6eb94))
    - Test(spentbook): remove old and commented tests - Cleanup of tests that will not be implemented there. ([`ee15a1d`](https://github.com/maidsafe/safe_network/commit/ee15a1de648f1192137c32b00b32d90d7cd84f4a))
    - Test(spentbook): disable test 6 - spend with updated network knowledge should update the node - This test should not be tested from sn_client, instead this should be an sn_node unit test. It just needs the state of a node ready to accept the state from the msg. ([`f1dd854`](https://github.com/maidsafe/safe_network/commit/f1dd854289ed2035bbf4f6fa90a9885e96108e72))
    - Test(spentbook): add test 5 - spend with random key image should return spentbook error ([`bbc2775`](https://github.com/maidsafe/safe_network/commit/bbc2775b41b1d2c045920f48d8395010ce34087b))
    - Test(spentbook): disable test 4 - transaction with no inputs should return spentbook error - This test cannot be tested from sn_client, this is a "Spentbook"-api unit test. ([`322f897`](https://github.com/maidsafe/safe_network/commit/322f8975c96949ea9d9b5aae1441391c8c203dba))
    - Test(spentbook): add test 3 - spent proofs do not relate to input dbcs should return spentbook error - NB: this can probably be simplified ([`aa23604`](https://github.com/maidsafe/safe_network/commit/aa23604f2dd26a4ce51735796a9822152059f15f))
    - Test(spentbook): add test 2 - spent proof with key not in section chain should return cmd error response ([`444257a`](https://github.com/maidsafe/safe_network/commit/444257a2abd5c57d1cfce4bfe3a6b69c1172bc96))
    - Do more detailed error matching ([`6dde955`](https://github.com/maidsafe/safe_network/commit/6dde9550c7be949fd40b60757f682d082836138b))
    - Test(spentbook): add test 1 - spent proof with invalid pk should return spentbook error ([`a98b3f7`](https://github.com/maidsafe/safe_network/commit/a98b3f7b1e44fde27e0c057ec86fb19b4bfe96b2))
    - Reuse code with setup fn ([`04d0774`](https://github.com/maidsafe/safe_network/commit/04d0774bbd65967944932456aeb75152ae015ef6))
    - Add the test stubs in sn_client ([`02322c6`](https://github.com/maidsafe/safe_network/commit/02322c68c449863ea475531cd953a05fbb2ee71b))
    - Merge #1793 ([`c5ab10f`](https://github.com/maidsafe/safe_network/commit/c5ab10f2831cc1f6978dfa518293649f08033e03))
    - Attempt to reduce allocations ([`6be0ea1`](https://github.com/maidsafe/safe_network/commit/6be0ea16b0ffe2c153c6a13f36916a91fb58cd05))
    - Use latest client knowledge to get new target elders ([`eccb709`](https://github.com/maidsafe/safe_network/commit/eccb7090e024e38bf561efa3b7ff2abfe491ba72))
    - Merge #1744 #1792 ([`ea83392`](https://github.com/maidsafe/safe_network/commit/ea83392ccc9cbb79b175c29ba77c4a7e27a5398f))
    - Minor logging improvements to help debug msgs arriving/processing on client and nodes ([`e5b0dda`](https://github.com/maidsafe/safe_network/commit/e5b0dda1315a5299131cacd135b1d1ab66ed7073))
    - Removing query retrying loops and sleeps in sn-api/sn-client tests ([`6bc0892`](https://github.com/maidsafe/safe_network/commit/6bc0892e50dddbda393eb37abac44e2bf7e626eb))
    - Upgrading qp2p to version 0.31.0 ([`f06b3e7`](https://github.com/maidsafe/safe_network/commit/f06b3e75ce97e7c749d2969276ad6533369806bb))
    - Merge #1789 ([`7fa2ab8`](https://github.com/maidsafe/safe_network/commit/7fa2ab88ddefaad9f157b70b8a700824ce986f31))
    - Feat(ack): remove dataaddress from ack Client should keep track of this, no need to burdon the network with more data transmission. ([`ead623b`](https://github.com/maidsafe/safe_network/commit/ead623bf50bfc3a5cb6539159ed2c863356d6f8c))
    - Merge #1786 ([`cdad707`](https://github.com/maidsafe/safe_network/commit/cdad707cff1d392c377470282919d34bd517c27b))
    - Don't assume Register response with more entries as its most up to date replica ([`e344c31`](https://github.com/maidsafe/safe_network/commit/e344c3149cf39b6b22a91b755d7e8b0a8ca87dec))
    - Merge #1783 ([`8ac3344`](https://github.com/maidsafe/safe_network/commit/8ac33440e0101a560ca27f45cd15e9f31bcb1ade))
    - Removing redundant ordering of new target Elders upon handling AE response ([`a612553`](https://github.com/maidsafe/safe_network/commit/a612553e6dd4c21bc1fec34425fcb6fe2875b044))
    - Merge #1776 ([`bb65746`](https://github.com/maidsafe/safe_network/commit/bb657464f8217aa1a41501c4025ceb5dc6d0aca7))
    - Connect to relevant nodes first ([`7ac8d43`](https://github.com/maidsafe/safe_network/commit/7ac8d43bb3f559d01d9eac829a19e171a401e1a8))
    - Remove force_new_link as unused ([`2691c53`](https://github.com/maidsafe/safe_network/commit/2691c53daa36b82185a664482a55d9c893dc8439))
    - Merge #1765 ([`90a870e`](https://github.com/maidsafe/safe_network/commit/90a870ebe1ce5110b4b264e8e317acc30152ceb1))
    - Making AE msg for clients to be a variant of client response msg type ([`7106b75`](https://github.com/maidsafe/safe_network/commit/7106b7533e119dc94bbf19fa304f3eb1f8dc9425))
    - Merge #1769 ([`cb8acf7`](https://github.com/maidsafe/safe_network/commit/cb8acf761cd6e39b48d6afab5c9337af3f2fb5aa))
    - Tweak churn test node startup interval to 3s ([`38fc820`](https://github.com/maidsafe/safe_network/commit/38fc82067065e9e67422388c07456d3ea896614b))
    - Merge #1766 ([`19ffd04`](https://github.com/maidsafe/safe_network/commit/19ffd04ac02fe98c72c0c4d497c29bdf961e9201))
    - Address review comments ([`7f288b3`](https://github.com/maidsafe/safe_network/commit/7f288b389175f3165fdca383dfe5f51097cc591f))
    - Refactor(responses): return correct cmd response - Returns the ack corresponding to the cmd. - Renames `ClientMsgResponse` to `ClientDataResponse`. - Makes `NodeDataResponse` be handled like `ClientDataResponse`. - Moves data write acks to `NodeDataReponse`. - Makes `NodeEvent` only be Adult to Elder notifications. ([`bd3b46e`](https://github.com/maidsafe/safe_network/commit/bd3b46e686a6f47cc006ce1f5da2f3041a614b2d))
    - Merge #1749 ([`ad2574c`](https://github.com/maidsafe/safe_network/commit/ad2574cb7fad692c2f9924fd87130f0b0bb9e2c2))
    - Reignore 40mb test until we have stability over ack response time ([`961b6ba`](https://github.com/maidsafe/safe_network/commit/961b6ba0f60c3fa5cd337d8954b4be8cc196274b))
    - Pass around MyNodeState to avoid holding locks ([`38b8f55`](https://github.com/maidsafe/safe_network/commit/38b8f55121d8b7c461efa6dd0c0407c4fae93418))
    - Replace `TestSAP` with `TestSapBuilder` ([`a55b74b`](https://github.com/maidsafe/safe_network/commit/a55b74b4c8f9bede3c91a9426d4687df01138257))
    - Merge #1637 ([`45903a9`](https://github.com/maidsafe/safe_network/commit/45903a9988528f543b09afbb56a89d21effbb929))
    - Breaking up client msg type separating requests from responses ([`80917f1`](https://github.com/maidsafe/safe_network/commit/80917f19125222ce6892e45487f2abe098fefd7a))
    - Merge #1746 ([`7b45313`](https://github.com/maidsafe/safe_network/commit/7b453132af84827deea125aea477b6ad829c2253))
    - Remove client internal channels from within Session mod ([`d41de50`](https://github.com/maidsafe/safe_network/commit/d41de505ef9fc0ce7ebb84c1271e1da77ca29f85))
    - Provide age pattern to generate `NodeInfo` ([`9f539e9`](https://github.com/maidsafe/safe_network/commit/9f539e9a8dd9c22e7440539114b2fbdaaeb34515))
    - Organize key related test utilites ([`093ea5b`](https://github.com/maidsafe/safe_network/commit/093ea5bfc200f940662c5c0e458c38c5c77294a9))
    - Organize network_knowledge test utilites ([`4b6569a`](https://github.com/maidsafe/safe_network/commit/4b6569ab2a9face420385d29d7baab31d8ca4d1e))
    - Make `WireMsg` immutable ([`233bf64`](https://github.com/maidsafe/safe_network/commit/233bf64f33a632bef2fdaed409888efaa6f10b63))
    - Remove from sn_client ([`c3bce22`](https://github.com/maidsafe/safe_network/commit/c3bce22d6f780441207f23ad0355c9081e60f323))
    - Merge #1724 ([`ef69747`](https://github.com/maidsafe/safe_network/commit/ef697470545ac8b3c359f721bb30b0f8b7854b65))
    - Improvements to data-replicas check Error types ([`c3934b9`](https://github.com/maidsafe/safe_network/commit/c3934b9b9bb6122294e40e0810c5f7f8ad59746a))
    - Feature-gated send_query API to verify data is stored in all replicas for each query ([`ff59202`](https://github.com/maidsafe/safe_network/commit/ff59202d1374c7e5dcc570d50ed8b399fafe488d))
    - Remove duplicate strum/strum_macros/heck deps ([`667009d`](https://github.com/maidsafe/safe_network/commit/667009dc02e6bb17bfaa60e2374d5ab7b75a7be5))
    - Criterion 0.3 -> 0.4, tracing-subscriber 0.2 -> 0.3 ([`860f326`](https://github.com/maidsafe/safe_network/commit/860f326a9baf7e62d191eec13359fa5313e6956d))
    - Bump blsttc to 8.0.0 ([`ee824e7`](https://github.com/maidsafe/safe_network/commit/ee824e7785b8da770b5aa6bba3415a274a4e0d68))
    - Post-rebase fixes ([`7405a23`](https://github.com/maidsafe/safe_network/commit/7405a23895d4d412db3c5f25b1ca85b534e469eb))
    - Small clippy fix ([`4431c84`](https://github.com/maidsafe/safe_network/commit/4431c843d5c7f326848826e68c88cc36d8b300b2))
    - Deterministically order AE msg elders ([`f2174d4`](https://github.com/maidsafe/safe_network/commit/f2174d4150580c770ed2e7e6a4a457d35dca1383))
    - Tidy up some error types ([`317fdb1`](https://github.com/maidsafe/safe_network/commit/317fdb18ce227bc383f5637e6dd300ec94af20ff))
    - Igore 40mb test for now ([`110d45e`](https://github.com/maidsafe/safe_network/commit/110d45ea649783e74d1eb3f449fa55aa0baa8e8a))
    - Ignore large clippy variants for now ([`b550ed0`](https://github.com/maidsafe/safe_network/commit/b550ed00dd3e3af878758aea1a2efb6eba0e8d66))
    - Send AE retries to all elders in updated section ([`c71fcd1`](https://github.com/maidsafe/safe_network/commit/c71fcd1fd61a4a2f9075d3d3e5f922b6d06644e6))
    - Retry loop for bidi initialisation ([`2b9268c`](https://github.com/maidsafe/safe_network/commit/2b9268c2aff5aae5eb1584d2698f282c8beae73e))
    - Ignore spentbook test that tries to handle client w/o a stream etc ([`f53337e`](https://github.com/maidsafe/safe_network/commit/f53337e6e0c7c4f804489f0d370d4cc97331597f))
    - Add in error msg in the event of missing stream during adult query ([`0ef5a2c`](https://github.com/maidsafe/safe_network/commit/0ef5a2cfd82de58f02a23a96a305b172f27c33c8))
    - Query adult update for query send changes ([`7f448bc`](https://github.com/maidsafe/safe_network/commit/7f448bcb3106b429c711d6ff81ded5c33138bca7))
    - Cmd responses sent from adults over stream ([`5a39a84`](https://github.com/maidsafe/safe_network/commit/5a39a843c5570993b0e27780a1c2887bbf7a3212))
    - Dont debug log AE bounced_msg ([`f15cee6`](https://github.com/maidsafe/safe_network/commit/f15cee66c4bbc0aa8cd8cd81652745726915e595))
    - Do not consider as a data-not-found case when not enough spent-proof-shares were retrieved from SpentBook ([`c73b83b`](https://github.com/maidsafe/safe_network/commit/c73b83b0a4ac0e3072b41e1af5b42b90c8a54177))
    - Removing unused Error types and adding context info to a couple of them ([`bdf50e7`](https://github.com/maidsafe/safe_network/commit/bdf50e7ad1214ef4bb48c0a12db8a7700193bb2a))
    - Tweaks to reduce use of async ([`f2dff36`](https://github.com/maidsafe/safe_network/commit/f2dff3636d3bf1446af53790a42a46473079698f))
    - Set nodes join interval to 30secs for testnet in sn-api tests job ([`1bf23ff`](https://github.com/maidsafe/safe_network/commit/1bf23ff2d00b66232267403f94d3fa133416fdd3))
    - Missing MsgKind import after rebase ([`bfaf6ea`](https://github.com/maidsafe/safe_network/commit/bfaf6eac15cfa6e7f829cdc9a79181836a27f473))
    - Additional contextual information for a few sn_client::Error types ([`718248b`](https://github.com/maidsafe/safe_network/commit/718248bfdabfff68de6dd775a9e6292e1c5733b4))
    - Try to reconnect once when the client lost a connection to a peer ([`3089b2c`](https://github.com/maidsafe/safe_network/commit/3089b2c8d3f3ee099ff9e0880a96720b319e52a4))
    - Test helper function for sleeping before querying published Register ops ([`0042633`](https://github.com/maidsafe/safe_network/commit/004263308ee31a9568c77aa9655dc186fde75e75))
    - Refactor(cmds): replace ack+error with response BREAKING CHANGE: ClientMsg enum variants changed. ([`df19b12`](https://github.com/maidsafe/safe_network/commit/df19b120bd769d0b375a27162f07a4a421f97ec0))
    - Verify each Chunk individually in the `upload_and_verify` API ([`10c2420`](https://github.com/maidsafe/safe_network/commit/10c2420505b85914eb7518c48a3951a5402331cf))
    - Cleanup unused deps ([`5b0b858`](https://github.com/maidsafe/safe_network/commit/5b0b8589bc9c90fac6285f626a46907b2dd0e819))
    - Fix prebuilt rg install ([`3a860ac`](https://github.com/maidsafe/safe_network/commit/3a860ace8096e84ab5421e409e2d58bcf491ec55))
    - Use one channel per Cmd/Query sent to await for responses ([`93c7a05`](https://github.com/maidsafe/safe_network/commit/93c7a054d87df7054224664c4a03c4507bcfbae6))
    - Use repsonse_stream for ae responses to client. ([`c98cc98`](https://github.com/maidsafe/safe_network/commit/c98cc98674552794960e5953c5dbf405d961c333))
    - Spawn a task to read query/cmd responses from bi-stream ([`8f887c0`](https://github.com/maidsafe/safe_network/commit/8f887c0f3f128f5d59304a0b47f6105cb52f3155))
    - Use bi stream from client; process in Node ([`3fd0a00`](https://github.com/maidsafe/safe_network/commit/3fd0a00bad2f9ca266a56de2086b54088459e153))
    - Cleanup un-necessary attempts counter ([`20003da`](https://github.com/maidsafe/safe_network/commit/20003daa9ff41de61fc2a9509364c0483c92d136))
    - Move to event driven msg handling ([`95436a1`](https://github.com/maidsafe/safe_network/commit/95436a1f722bfd02a735dc3cf2f171a8b70de552))
    - Log MsgId in InsufficentAck error ([`27cbeb6`](https://github.com/maidsafe/safe_network/commit/27cbeb647b90e7105e5b650b436944d3cdd813c5))
    - Only enqueue msgs at start, thereafter process all enqueued ([`a05d7ca`](https://github.com/maidsafe/safe_network/commit/a05d7ca9b7d2319085a2dea9119735e3e44f50c1))
    - Remove ExpiringConnection struct ([`98abbbe`](https://github.com/maidsafe/safe_network/commit/98abbbe7af8c870faa22d62819691054e07df718))
    - Use keep alive for elder comms ([`f459a85`](https://github.com/maidsafe/safe_network/commit/f459a85663d93a8a516dab85cf91f378a3982020))
    - Don't remove peer link if one listener is done... there may be more listeners running? ([`9aeac11`](https://github.com/maidsafe/safe_network/commit/9aeac110cad307f0013d67589c48c39a184468e4))
    - Allow too many args ([`f856d85`](https://github.com/maidsafe/safe_network/commit/f856d853c0166f35bb2b98b24e3f3c8c09783b2d))
    - Dont debug log AE bounced_msg ([`a736dc6`](https://github.com/maidsafe/safe_network/commit/a736dc6bd6a107d85981c3687cb5bcb39c62653c))
    - Clients reuse same msg id for same query while looping ([`40ce02d`](https://github.com/maidsafe/safe_network/commit/40ce02d611027f748d7934fa21adb4142422f7fc))
    - Removing op id from query response ([`a973b62`](https://github.com/maidsafe/safe_network/commit/a973b62a8ef48acc92af8735e7e7bcac94e0092f))
    - Deem spentproof shares < 5 data not found ([`428d2e9`](https://github.com/maidsafe/safe_network/commit/428d2e9528c567e5ac46256100ecadcf496dd8e1))
    - Increase query loop time ([`bef40c1`](https://github.com/maidsafe/safe_network/commit/bef40c17867f33e4775af8ee5e9b617d52ee2667))
    - Return error responses event if we dont have 7. ([`c2d0a0c`](https://github.com/maidsafe/safe_network/commit/c2d0a0c98569b7fbeacad6796d7b245d2c3e6828))
    - Rejig retry loop, use max retries and keep querying if data not found until timeout ([`033d17f`](https://github.com/maidsafe/safe_network/commit/033d17f355bea910939e094770af73be89e642ad))
    - Fix unintentionally fast query retry time ([`b74d424`](https://github.com/maidsafe/safe_network/commit/b74d424f41990c5cf048a9472031983e22099947))
    - Check for longest spentproof response ([`8a56982`](https://github.com/maidsafe/safe_network/commit/8a569827c64c89c9b7268e7c6b103a42a4e45dae))
    - Removing retry looping in tests ([`68c54c9`](https://github.com/maidsafe/safe_network/commit/68c54c90eb622a33bed1981ef765adb9bded2d96))
    - Dont manually clean up client peers ([`9a6f4a3`](https://github.com/maidsafe/safe_network/commit/9a6f4a3cf20852f4b5604bf08a04aba592dca0fa))
    - No retries in ci profile ([`16fafb0`](https://github.com/maidsafe/safe_network/commit/16fafb026bb9286b1724dd152587dfd9b23f7b0c))
    - Dont force new cmd link ([`28bcff2`](https://github.com/maidsafe/safe_network/commit/28bcff2c01fdf505c06b4ff6a39ee12e2ada1bb4))
    - Fail fast ([`213d7c8`](https://github.com/maidsafe/safe_network/commit/213d7c89adf4706ebd928f56ec0954fcc97a810b))
    - Write query responses as they come in ([`e57d832`](https://github.com/maidsafe/safe_network/commit/e57d83235f60a16bd7e1ee801f35a599113dc71a))
    - Retry cmd if any sends failed ([`e4721ab`](https://github.com/maidsafe/safe_network/commit/e4721ab8c3bfcb3379fe09bbd069c1eab0048753))
    - Cleanup dropped peers ([`0f11b2e`](https://github.com/maidsafe/safe_network/commit/0f11b2ed16c765bee3e25f2f25b374e3f06f8e8f))
    - Refactor client ACK receipt ([`5a539a7`](https://github.com/maidsafe/safe_network/commit/5a539a74d0dcca7a8671910d45bbb08ca4382671))
    - Use same MsgId for all Cmds going out ([`a6d225b`](https://github.com/maidsafe/safe_network/commit/a6d225ba65c4817ada16b82626910bed3071d2bf))
    - Add client test logs ([`daaf79d`](https://github.com/maidsafe/safe_network/commit/daaf79dbd780e03a99baf0e19a738a08bf2f954e))
    - Force retries to use fresh connection ([`a5bf211`](https://github.com/maidsafe/safe_network/commit/a5bf211daa0272597f1a2d852a17592258a2115a))
    - Dont try and establish connections for ServiceMsgs ([`411ea37`](https://github.com/maidsafe/safe_network/commit/411ea371660b5f76a5c3f887f78331e58f8b6961))
    - Remove some ? noise in tests ([`9fad752`](https://github.com/maidsafe/safe_network/commit/9fad752ce1849763ae16cdb42159b9dccf1a13d0))
    - Merge #1703 ([`297004f`](https://github.com/maidsafe/safe_network/commit/297004fe04bba05765eb4d02394210024dfcf559))
    - AuthKind into MsgKind without node sig ([`633dfc8`](https://github.com/maidsafe/safe_network/commit/633dfc836c10eafc54dedefc53b2cbc9526970bb))
    - Merge #1685 ([`992f917`](https://github.com/maidsafe/safe_network/commit/992f917830c6d7b10fbd4d1f03a81eb5e8a64fdc))
    - Remove node auth ([`5019dd3`](https://github.com/maidsafe/safe_network/commit/5019dd3896d278227878ffd91ce14d0cecb2b9dd))
    - Assert_lists asserts instead of returning Result ([`ab22c69`](https://github.com/maidsafe/safe_network/commit/ab22c6989f55994065f0d859b91e141f7489a722))
    - Remove `SectionAuthorityProvider`, `SectionTreeUpdate` messages ([`32744bc`](https://github.com/maidsafe/safe_network/commit/32744bcf6c94d9a7cda81813646560b70be53eb2))
    - Adapt confusing auth related code ([`07d0991`](https://github.com/maidsafe/safe_network/commit/07d0991fed28d49c9be85d44a3343b66fac076d9))
    - Rename a bunch of auth to sig ([`452ef9c`](https://github.com/maidsafe/safe_network/commit/452ef9c5778ad88270f4e251adc49ccbc9b3cb09))
    - Rename a helper func ([`151a22f`](https://github.com/maidsafe/safe_network/commit/151a22f63a0aaaf94070f3bd0e1f6bd2f9239856))
    - Simplify section sig ([`85f4d00`](https://github.com/maidsafe/safe_network/commit/85f4d00e81ac5bf67b6be89d7ff51b7bb1060ed6))
    - Refactor ([`4beec97`](https://github.com/maidsafe/safe_network/commit/4beec978b1f2eae2198bcd85e3e0bf377d97575c))
    - Refactor: ([`444ed16`](https://github.com/maidsafe/safe_network/commit/444ed16e55d8e962404c8c7b643b00f0685eed18))
    - Fix for rename in benches ([`54222ae`](https://github.com/maidsafe/safe_network/commit/54222aeea627209c53a1f595578436deb2763ef0))
    - Refactor: ([`50d48bf`](https://github.com/maidsafe/safe_network/commit/50d48bfc4fcc54266125bc0f1a3369097376497c))
    - Refactor: ServiceAuth -> ClientAuth Service -> Client NodeBlsShare -> SectionPart BlsShareAuth -> SectionAuthPart SystemMsg -> Node2NodeMsg OutgoingMsg::System -> OutgoingMsg::Node2Node + fmt / fix ([`0b9d08b`](https://github.com/maidsafe/safe_network/commit/0b9d08bf88b6892b53dabf82fa988674fdd9992a))
    - Merge #1550 ([`c6f2e2f`](https://github.com/maidsafe/safe_network/commit/c6f2e2fb98e29911336f86f54c1d9b9605037b57))
    - Compiling sdkg integration ([`f5d53c1`](https://github.com/maidsafe/safe_network/commit/f5d53c188a03150a06bdee97fb585f7900b7c251))
    - Client retry spend on unknown section key ([`5c8b1f5`](https://github.com/maidsafe/safe_network/commit/5c8b1f50d1bf346d45bd2a9faf92bbf33cb448da))
    - Get public commitments from sn_dbc ([`1152b27`](https://github.com/maidsafe/safe_network/commit/1152b2764e955edd80fb33921a8d8fe52654a896))
    - Minor refactoring and fixing issue reported by new clippy version ([`c7de082`](https://github.com/maidsafe/safe_network/commit/c7de08209c659ec93557d6ea10e0bcd8c3b74d8b))
    - Merge #1557 ([`6cac22a`](https://github.com/maidsafe/safe_network/commit/6cac22af4994651719f64bc76391d729a3efb656))
    - Remove spend retry on client ([`100e2ae`](https://github.com/maidsafe/safe_network/commit/100e2ae70d21e141e1ebbc324f8b06e3d3f1a01c))
    - Spend with updated network knowledge ([`230a6ed`](https://github.com/maidsafe/safe_network/commit/230a6ed7f1f4193fa36b2fbb83bea072f4944c1d))
    - Bundle proof chain, SAP into `SectionTreeUpdate` ([`60e333d`](https://github.com/maidsafe/safe_network/commit/60e333d4ced688f3382cde513300d38790613692))
    - Retry dbc spend on unknown section key ([`057ce1c`](https://github.com/maidsafe/safe_network/commit/057ce1ce1e174102e23d96cfcd2ab1d090a6f1dc))
    - Merge #1527 ([`1f06d6e`](https://github.com/maidsafe/safe_network/commit/1f06d6e90da6f889221f37cc8eac32b6933a94ba))
    - Optimizations and code cleanup ([`03da7f6`](https://github.com/maidsafe/safe_network/commit/03da7f67fff1fa5bb06d60a66dfdb531506dec4c))
    - Replace `SecuredLinkedList` with `SectionsDAG` ([`0cd47ad`](https://github.com/maidsafe/safe_network/commit/0cd47ad56e0d93e3e99feb0dfcea8094f871ff6f))
    - Verify client generate entry_hash ([`18bea3c`](https://github.com/maidsafe/safe_network/commit/18bea3c1e56268b28826643e1ff8936dfa6d4896))
</details>

## v0.76.0 (2022-09-19)

<csr-id-a8a9fb90791b29496e8559090dca4161e04054da/>
<csr-id-a0bc2562df4f427752ec0f3ab85d9befe2d20050/>
<csr-id-2d1221999b959bf4d0879cf42050d5e1e3119445/>

### Chore

 - <csr-id-a8a9fb90791b29496e8559090dca4161e04054da/> sn_interface-0.15.0/sn_dysfunction-0.14.0/sn_client-0.76.0/sn_node-0.71.0/sn_api-0.74.0/sn_cli-0.67.0
 - <csr-id-a0bc2562df4f427752ec0f3ab85d9befe2d20050/> cleanup unused deps

### Refactor (BREAKING)

 - <csr-id-2d1221999b959bf4d0879cf42050d5e1e3119445/> flattening up ServiceMsg::ServiceError and ServiceMsg::CmdError types

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 2 calendar days.
 - 9 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.15.0/sn_dysfunction-0.14.0/sn_client-0.76.0/sn_node-0.71.0/sn_api-0.74.0/sn_cli-0.67.0 ([`a8a9fb9`](https://github.com/maidsafe/safe_network/commit/a8a9fb90791b29496e8559090dca4161e04054da))
    - Flattening up ServiceMsg::ServiceError and ServiceMsg::CmdError types ([`2d12219`](https://github.com/maidsafe/safe_network/commit/2d1221999b959bf4d0879cf42050d5e1e3119445))
    - Cleanup unused deps ([`a0bc256`](https://github.com/maidsafe/safe_network/commit/a0bc2562df4f427752ec0f3ab85d9befe2d20050))
</details>

## v0.75.0 (2022-09-09)

<csr-id-448694176dd3b40a12bd8ecc16d9bb66fd171a37/>
<csr-id-7d4a15a7855429d604c0216f67e46620fea80e6f/>
<csr-id-278f29ea80352211c7c0606945f7dfc4908ea9ca/>

### Chore

 - <csr-id-448694176dd3b40a12bd8ecc16d9bb66fd171a37/> sn_interface-0.14.0/sn_dysfunction-0.13.0/sn_client-0.75.0/sn_node-0.70.0/sn_api-0.73.0/sn_cli-0.66.0
 - <csr-id-7d4a15a7855429d604c0216f67e46620fea80e6f/> loop upload verification to avoid early NoDataFound
   The underlying API now returns NoDataFound once it ahs queried all adults... So we loop atop this to ensure we don't hit this too early as not all chunks may have been uploaded yet
 - <csr-id-278f29ea80352211c7c0606945f7dfc4908ea9ca/> remove send_msg_in_bg as it hides errors from us

### Bug Fixes

 - <csr-id-7513fbf431f369e3f6eab9e32d23988ec0cec056/> enable statemap feature in churn test

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.14.0/sn_dysfunction-0.13.0/sn_client-0.75.0/sn_node-0.70.0/sn_api-0.73.0/sn_cli-0.66.0 ([`4486941`](https://github.com/maidsafe/safe_network/commit/448694176dd3b40a12bd8ecc16d9bb66fd171a37))
    - Enable statemap feature in churn test ([`7513fbf`](https://github.com/maidsafe/safe_network/commit/7513fbf431f369e3f6eab9e32d23988ec0cec056))
    - Merge #1556 ([`d3d6593`](https://github.com/maidsafe/safe_network/commit/d3d6593989d9d16148b8490a6227acbe0871d267))
    - Merge branch 'main' into Chore-ClientRetriesOnDataNotFound ([`bbca976`](https://github.com/maidsafe/safe_network/commit/bbca97680840e1069c88278fe14ddee153b97dbb))
    - Loop upload verification to avoid early NoDataFound ([`7d4a15a`](https://github.com/maidsafe/safe_network/commit/7d4a15a7855429d604c0216f67e46620fea80e6f))
    - Remove send_msg_in_bg as it hides errors from us ([`278f29e`](https://github.com/maidsafe/safe_network/commit/278f29ea80352211c7c0606945f7dfc4908ea9ca))
</details>

## v0.74.0 (2022-09-07)

<csr-id-fe659c5685289fe0071b54298dcac394e83c0dce/>
<csr-id-b1329158b3c2427a7c1939060ba1fe3ef9e72bf9/>

### Chore

 - <csr-id-fe659c5685289fe0071b54298dcac394e83c0dce/> sn_interface-0.13.0/sn_dysfunction-0.12.0/sn_client-0.74.0/sn_node-0.69.0/sn_api-0.72.0/sn_cli-0.65.0
 - <csr-id-b1329158b3c2427a7c1939060ba1fe3ef9e72bf9/> retry DataNotFound errors for data_copy_count * 2
   We do this twice in case of connection issues during prev run

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.13.0/sn_dysfunction-0.12.0/sn_client-0.74.0/sn_node-0.69.0/sn_api-0.72.0/sn_cli-0.65.0 ([`fe659c5`](https://github.com/maidsafe/safe_network/commit/fe659c5685289fe0071b54298dcac394e83c0dce))
    - Retry DataNotFound errors for data_copy_count * 2 ([`b132915`](https://github.com/maidsafe/safe_network/commit/b1329158b3c2427a7c1939060ba1fe3ef9e72bf9))
</details>

## v0.73.0 (2022-09-06)

<csr-id-d28fdf3ddd0a39f7bbc6426e1e71d990319b0ec7/>
<csr-id-921438659ccaf65b2ea8cc00efb61d8146ef71ef/>
<csr-id-f5d436fba99e0e9c258c7ab3c3a256be3be58f84/>
<csr-id-388cd223677ecfa2e790c54c0df8ecb18c77299c/>
<csr-id-5c54848b726188f273ffa16ee2870976914bb815/>
<csr-id-b040ea14e53247094838de6f1fa9af2830b051fa/>
<csr-id-3a718d8c0957957a75250b044c9d1ad1b5874ab0/>
<csr-id-a4e84ef4608a13ecc2f14dd87f5c23d185185513/>
<csr-id-d251dbeb2e44707623c3bbb1215784b1bd4fae06/>
<csr-id-1b9e0a6564e9564201ef3a3e04adb0bfbef6ac14/>

### Chore

 - <csr-id-d28fdf3ddd0a39f7bbc6426e1e71d990319b0ec7/> sn_interface-0.11.0/sn_dysfunction-0.10.0/sn_client-0.72.0/sn_node-0.67.0/sn_api-0.70.0/sn_cli-0.63.0
 - <csr-id-921438659ccaf65b2ea8cc00efb61d8146ef71ef/> unneeded iter methods removal
 - <csr-id-f5d436fba99e0e9c258c7ab3c3a256be3be58f84/> applied use_self lint
 - <csr-id-388cd223677ecfa2e790c54c0df8ecb18c77299c/> change throughput measure
   use Bytes.len, not the size of the Bytes struct!
 - <csr-id-5c54848b726188f273ffa16ee2870976914bb815/> adds missing bench to cargo.toml for client
 - <csr-id-b040ea14e53247094838de6f1fa9af2830b051fa/> sn_interface lints and fixes
   Apply lints used in other crates, as far as they can easily be applied.
   The `unused_results` lint has been left out, as that is too much
   cleaning up to do, just like adding documentation to all the public
   interfaces.
 - <csr-id-3a718d8c0957957a75250b044c9d1ad1b5874ab0/> switch on clippy::unwrap_used as a warning


### Chore

 - <csr-id-1b9e0a6564e9564201ef3a3e04adb0bfbef6ac14/> sn_interface-0.12.0/sn_dysfunction-0.11.0/sn_client-0.73.0/sn_node-0.68.0/sn_api-0.71.0/sn_cli-0.64.0

### Bug Fixes

 - <csr-id-6bdc82295dfdcaa617c7c1e36d2b72f085e50042/> update qp2p for unique ids
   Latest qp2p should provide global unique connection id
   
   previously duplication of ids could have been breaking
   connection management

### Other

 - <csr-id-a4e84ef4608a13ecc2f14dd87f5c23d185185513/> move benches that dont need the network ahead of network launch
   In order to fail faster
 - <csr-id-d251dbeb2e44707623c3bbb1215784b1bd4fae06/> add msg serialization benchmark
   This should allow us to evaluate any changes to msg serialisation
   in order to reduce time/memory when resending the same message to many
   peers

### New Features (BREAKING)

 - <csr-id-f5361d91f8215585651229eb6dc2535f2ecb631c/> update qp2p to use UsrMsgBytes and avoid reserializing bytes
   This makes use of udpate qp2p to avoid having to reserialise the
   WireMsgheader for every message when we're just updating the Dst.
   
   This in turn avoids the neccesity to clone the msg payload when
   serilizing; allowing us to to use the shared data struct Bytes for all
   parts, reducing both compute and memory use.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 17 commits contributed to the release over the course of 8 calendar days.
 - 8 days passed between releases.
 - 12 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.12.0/sn_dysfunction-0.11.0/sn_client-0.73.0/sn_node-0.68.0/sn_api-0.71.0/sn_cli-0.64.0 ([`1b9e0a6`](https://github.com/maidsafe/safe_network/commit/1b9e0a6564e9564201ef3a3e04adb0bfbef6ac14))
    - Update qp2p to use UsrMsgBytes and avoid reserializing bytes ([`f5361d9`](https://github.com/maidsafe/safe_network/commit/f5361d91f8215585651229eb6dc2535f2ecb631c))
    - Merge #1544 ([`e8202a6`](https://github.com/maidsafe/safe_network/commit/e8202a6ea8c07f8ae0a04273b2cda350758352ab))
    - Update qp2p for unique ids ([`6bdc822`](https://github.com/maidsafe/safe_network/commit/6bdc82295dfdcaa617c7c1e36d2b72f085e50042))
    - Sn_interface-0.11.0/sn_dysfunction-0.10.0/sn_client-0.72.0/sn_node-0.67.0/sn_api-0.70.0/sn_cli-0.63.0 ([`d28fdf3`](https://github.com/maidsafe/safe_network/commit/d28fdf3ddd0a39f7bbc6426e1e71d990319b0ec7))
    - Unneeded iter methods removal ([`9214386`](https://github.com/maidsafe/safe_network/commit/921438659ccaf65b2ea8cc00efb61d8146ef71ef))
    - Applied use_self lint ([`f5d436f`](https://github.com/maidsafe/safe_network/commit/f5d436fba99e0e9c258c7ab3c3a256be3be58f84))
    - Merge #1535 ([`7327112`](https://github.com/maidsafe/safe_network/commit/7327112da76871d52b5039546419ab18e41982f8))
    - Chore(clippy) ([`d9ee11d`](https://github.com/maidsafe/safe_network/commit/d9ee11d228f8ac9f2d6cd3d09f1a1e29276100d1))
    - Change throughput measure ([`388cd22`](https://github.com/maidsafe/safe_network/commit/388cd223677ecfa2e790c54c0df8ecb18c77299c))
    - Adds missing bench to cargo.toml for client ([`5c54848`](https://github.com/maidsafe/safe_network/commit/5c54848b726188f273ffa16ee2870976914bb815))
    - Toml ([`4aa9b02`](https://github.com/maidsafe/safe_network/commit/4aa9b02f375a30132712ca97306e5f2e9a8d36f7))
    - Move benches that dont need the network ahead of network launch ([`a4e84ef`](https://github.com/maidsafe/safe_network/commit/a4e84ef4608a13ecc2f14dd87f5c23d185185513))
    - Add msg serialization benchmark ([`d251dbe`](https://github.com/maidsafe/safe_network/commit/d251dbeb2e44707623c3bbb1215784b1bd4fae06))
    - Sn_interface lints and fixes ([`b040ea1`](https://github.com/maidsafe/safe_network/commit/b040ea14e53247094838de6f1fa9af2830b051fa))
    - Merge branch 'main' into avoid_testing_data_collision ([`60c368b`](https://github.com/maidsafe/safe_network/commit/60c368b8494eaeb219572c2304bf787a168cfee0))
    - Switch on clippy::unwrap_used as a warning ([`3a718d8`](https://github.com/maidsafe/safe_network/commit/3a718d8c0957957a75250b044c9d1ad1b5874ab0))
</details>

## v0.72.0 (2022-09-04)

<csr-id-921438659ccaf65b2ea8cc00efb61d8146ef71ef/>
<csr-id-f5d436fba99e0e9c258c7ab3c3a256be3be58f84/>
<csr-id-388cd223677ecfa2e790c54c0df8ecb18c77299c/>
<csr-id-5c54848b726188f273ffa16ee2870976914bb815/>
<csr-id-b040ea14e53247094838de6f1fa9af2830b051fa/>
<csr-id-3a718d8c0957957a75250b044c9d1ad1b5874ab0/>
<csr-id-a4e84ef4608a13ecc2f14dd87f5c23d185185513/>
<csr-id-d251dbeb2e44707623c3bbb1215784b1bd4fae06/>

### Chore

 - <csr-id-921438659ccaf65b2ea8cc00efb61d8146ef71ef/> unneeded iter methods removal
 - <csr-id-f5d436fba99e0e9c258c7ab3c3a256be3be58f84/> applied use_self lint
 - <csr-id-388cd223677ecfa2e790c54c0df8ecb18c77299c/> change throughput measure
   use Bytes.len, not the size of the Bytes struct!
 - <csr-id-5c54848b726188f273ffa16ee2870976914bb815/> adds missing bench to cargo.toml for client
 - <csr-id-b040ea14e53247094838de6f1fa9af2830b051fa/> sn_interface lints and fixes
   Apply lints used in other crates, as far as they can easily be applied.
   The `unused_results` lint has been left out, as that is too much
   cleaning up to do, just like adding documentation to all the public
   interfaces.
 - <csr-id-3a718d8c0957957a75250b044c9d1ad1b5874ab0/> switch on clippy::unwrap_used as a warning


### Other

 - <csr-id-a4e84ef4608a13ecc2f14dd87f5c23d185185513/> move benches that dont need the network ahead of network launch
   In order to fail faster
 - <csr-id-d251dbeb2e44707623c3bbb1215784b1bd4fae06/> add msg serialization benchmark
   This should allow us to evaluate any changes to msg serialisation
   in order to reduce time/memory when resending the same message to many
   peers

## v0.71.1 (2022-08-28)

<csr-id-2b268209e6910472558145a5d08b99e968550221/>

### Chore

 - <csr-id-2b268209e6910472558145a5d08b99e968550221/> sn_interface-0.10.2/sn_client-0.71.1/sn_node-0.66.2/sn_cli-0.62.1

### New Features

 - <csr-id-7cc2a00907381e93db266f31545b12ff76907e5d/> implement `SecuredLinkedList` as a `MerkleRegister`
 - <csr-id-b87617e44e9b20b8a79864e30e29ecee86444352/> return error to client on unknown section key
   If one of the spent proofs sent by the client have been signed with a key this section is not
   currently aware of, return an error back to the client.
   
   This introduces a new SpentProofUnknownSectionKey variant to the messaging data errors, because none
   of the existing variants seemed appropriate for this scenario.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 2 calendar days.
 - 3 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.10.2/sn_client-0.71.1/sn_node-0.66.2/sn_cli-0.62.1 ([`2b26820`](https://github.com/maidsafe/safe_network/commit/2b268209e6910472558145a5d08b99e968550221))
    - Implement `SecuredLinkedList` as a `MerkleRegister` ([`7cc2a00`](https://github.com/maidsafe/safe_network/commit/7cc2a00907381e93db266f31545b12ff76907e5d))
    - Merge #1512 ([`3ca0038`](https://github.com/maidsafe/safe_network/commit/3ca0038a32539cf20b61292661b755886d02717e))
    - Return error to client on unknown section key ([`b87617e`](https://github.com/maidsafe/safe_network/commit/b87617e44e9b20b8a79864e30e29ecee86444352))
</details>

## v0.71.0 (2022-08-25)

<csr-id-401bc416c7aea65ae55e9adee2cbecf782c999cf/>
<csr-id-a46ac6e18bbdfdb331caf89f8bb562a7c762b64b/>
<csr-id-9fbb0672735306336f5020794a638f79752f0577/>
<csr-id-f40277c1680f56b043c4865ff201c65b66926b2d/>

### Chore

 - <csr-id-401bc416c7aea65ae55e9adee2cbecf782c999cf/> sn_interface-0.10.1/sn_client-0.71.0/sn_node-0.66.1/sn_api-0.69.0/sn_cli-0.62.0
 - <csr-id-a46ac6e18bbdfdb331caf89f8bb562a7c762b64b/> churn example, wait longer before we attempt to read post-churn
 - <csr-id-9fbb0672735306336f5020794a638f79752f0577/> further reduce query retries and query timeout
 - <csr-id-f40277c1680f56b043c4865ff201c65b66926b2d/> reduce query timeout noe we;re faster in general

### Bug Fixes

 - <csr-id-604556e670d5fe0a9408bbd0d586363c7b4c0d6c/> Decode ReplicatedDataAddress from chunk filename
   We were previously encoding a ReplicatedDataAddress, but
   decoding as a ChunkAddress

### New Features (BREAKING)

 - <csr-id-a8b3cd855393d06a64734b34523e40ec00fb0580/> expose MAX_RETRIES for cmd/query ops in client builder

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 1 day passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.10.1/sn_client-0.71.0/sn_node-0.66.1/sn_api-0.69.0/sn_cli-0.62.0 ([`401bc41`](https://github.com/maidsafe/safe_network/commit/401bc416c7aea65ae55e9adee2cbecf782c999cf))
    - Decode ReplicatedDataAddress from chunk filename ([`604556e`](https://github.com/maidsafe/safe_network/commit/604556e670d5fe0a9408bbd0d586363c7b4c0d6c))
    - Churn example, wait longer before we attempt to read post-churn ([`a46ac6e`](https://github.com/maidsafe/safe_network/commit/a46ac6e18bbdfdb331caf89f8bb562a7c762b64b))
    - Expose MAX_RETRIES for cmd/query ops in client builder ([`a8b3cd8`](https://github.com/maidsafe/safe_network/commit/a8b3cd855393d06a64734b34523e40ec00fb0580))
    - Further reduce query retries and query timeout ([`9fbb067`](https://github.com/maidsafe/safe_network/commit/9fbb0672735306336f5020794a638f79752f0577))
    - Reduce query timeout noe we;re faster in general ([`f40277c`](https://github.com/maidsafe/safe_network/commit/f40277c1680f56b043c4865ff201c65b66926b2d))
</details>

## v0.70.0 (2022-08-23)

<csr-id-3b068764721cd74f4d52a279a606743415abff02/>
<csr-id-2f8f8ca6ba0f2faae5bb4631c708988edf907725/>
<csr-id-589f03ce8670544285f329fe35c19897d4bfced8/>
<csr-id-ddbbb53d61d6c94b00a47dc2b708a2aeda870d96/>
<csr-id-1235f7d8a92eb9f086c35696bf5c0a8baf67f2ac/>
<csr-id-6471eb88f7ce8c060909930ac23c855f30e8690a/>
<csr-id-06f5b607cdfbacba082612965630249e3c0f7300/>
<csr-id-1618cf6a93117942946d152efee24fe3c7020e55/>
<csr-id-11b8182a3de636a760d899cb15d7184d8153545a/>
<csr-id-28d95a2e959e32ee69a70bdc855cba1fff1fc8d8/>
<csr-id-d3f66d6cfa838a5c65fb8f31fa68d48794b33dea/>
<csr-id-f0fbe5fd9bec0b2865271bb139c9fcb4ec225884/>
<csr-id-43fcc7c517f95eab0e27ddc79cd9c6de3631c7c6/>

### Chore

 - <csr-id-3b068764721cd74f4d52a279a606743415abff02/> add logs and tweaks to churn example
 - <csr-id-2f8f8ca6ba0f2faae5bb4631c708988edf907725/> associated functions to methods
 - <csr-id-589f03ce8670544285f329fe35c19897d4bfced8/> upgrading sn_dbc to v8.0
 - <csr-id-ddbbb53d61d6c94b00a47dc2b708a2aeda870d96/> leave out unnecessary Arc<RwLock>
 - <csr-id-1235f7d8a92eb9f086c35696bf5c0a8baf67f2ac/> remove unused Session member
 - <csr-id-6471eb88f7ce8c060909930ac23c855f30e8690a/> retry more times for connection fails w/ client

### Chore

 - <csr-id-43fcc7c517f95eab0e27ddc79cd9c6de3631c7c6/> sn_interface-0.10.0/sn_dysfunction-0.9.0/sn_client-0.70.0/sn_node-0.66.0/sn_api-0.68.0/sn_cli-0.61.0

### Bug Fixes

 - <csr-id-dfed2a8d2751b6627250b64e7a78213b68ec6733/> move data replication steps ahead of elder check in FlowCtrl
 - <csr-id-43ecab2dda52cb0ede7c0d4b6e48eaffe1fb6b75/> reintroduce Arc<RwLock> for section tree
   The RwLock was mistakenly removed by me. This meant that network updates
   to the section tree were not propagated back to the client's session.
 - <csr-id-6155ad0334104d367638373fbcbbd7e21631b3e6/> reduce client qp2p default idle timeout

### Other

 - <csr-id-06f5b607cdfbacba082612965630249e3c0f7300/> remove long lived client conn test
   No longer relevant, client conns can be cleaned up by nodes every X time.
   So clients have to be resilient and retry (which they do). So this (long)
   test can be dropped

### Refactor

 - <csr-id-1618cf6a93117942946d152efee24fe3c7020e55/> expose serialisation/deserialisation utilities as public methods instead
   - Also include the genesis key of each network in the list shown by CLI networks cmd.
 - <csr-id-11b8182a3de636a760d899cb15d7184d8153545a/> clean up unused functionality
   `closest` is a method that will find a prefix that is closest, but if
   not returning any, it means the set is empty. The `closest_or_opposite`
   used this function internally, but actually never got to the opposite,
   because `closest` would always return a SAP.
   
   This method was used in a few places where no exclusions were given, so
   it is clear in that case that it would always find a prefix. In a single
   case, it was called with an exclusion, where it would find a section
   closer than its own section.

### New Features (BREAKING)

 - <csr-id-991ccd452119137d9da046b7f222f091177e28f1/> adding more context information to sn_client::Error types

### Refactor (BREAKING)

 - <csr-id-28d95a2e959e32ee69a70bdc855cba1fff1fc8d8/> removing unused CreateRegister::Populated msg type
 - <csr-id-d3f66d6cfa838a5c65fb8f31fa68d48794b33dea/> removing unused sn_node::dbs::Error variants and RegisterExtend cmd
 - <csr-id-f0fbe5fd9bec0b2865271bb139c9fcb4ec225884/> renaming NetworkPrefixMap to SectionTree
   - Changing CLI and sn_client default path for network contacts to `$HOME/.safe/network_contacts`.
   - Renaming variables and functions referring to "prefix map" to now refer to "network contacts".

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 17 commits contributed to the release over the course of 8 calendar days.
 - 9 days passed between releases.
 - 17 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.10.0/sn_dysfunction-0.9.0/sn_client-0.70.0/sn_node-0.66.0/sn_api-0.68.0/sn_cli-0.61.0 ([`43fcc7c`](https://github.com/maidsafe/safe_network/commit/43fcc7c517f95eab0e27ddc79cd9c6de3631c7c6))
    - Removing unused CreateRegister::Populated msg type ([`28d95a2`](https://github.com/maidsafe/safe_network/commit/28d95a2e959e32ee69a70bdc855cba1fff1fc8d8))
    - Removing unused sn_node::dbs::Error variants and RegisterExtend cmd ([`d3f66d6`](https://github.com/maidsafe/safe_network/commit/d3f66d6cfa838a5c65fb8f31fa68d48794b33dea))
    - Adding more context information to sn_client::Error types ([`991ccd4`](https://github.com/maidsafe/safe_network/commit/991ccd452119137d9da046b7f222f091177e28f1))
    - Move data replication steps ahead of elder check in FlowCtrl ([`dfed2a8`](https://github.com/maidsafe/safe_network/commit/dfed2a8d2751b6627250b64e7a78213b68ec6733))
    - Add logs and tweaks to churn example ([`3b06876`](https://github.com/maidsafe/safe_network/commit/3b068764721cd74f4d52a279a606743415abff02))
    - Reintroduce Arc<RwLock> for section tree ([`43ecab2`](https://github.com/maidsafe/safe_network/commit/43ecab2dda52cb0ede7c0d4b6e48eaffe1fb6b75))
    - Associated functions to methods ([`2f8f8ca`](https://github.com/maidsafe/safe_network/commit/2f8f8ca6ba0f2faae5bb4631c708988edf907725))
    - Upgrading sn_dbc to v8.0 ([`589f03c`](https://github.com/maidsafe/safe_network/commit/589f03ce8670544285f329fe35c19897d4bfced8))
    - Renaming NetworkPrefixMap to SectionTree ([`f0fbe5f`](https://github.com/maidsafe/safe_network/commit/f0fbe5fd9bec0b2865271bb139c9fcb4ec225884))
    - Expose serialisation/deserialisation utilities as public methods instead ([`1618cf6`](https://github.com/maidsafe/safe_network/commit/1618cf6a93117942946d152efee24fe3c7020e55))
    - Remove long lived client conn test ([`06f5b60`](https://github.com/maidsafe/safe_network/commit/06f5b607cdfbacba082612965630249e3c0f7300))
    - Reduce client qp2p default idle timeout ([`6155ad0`](https://github.com/maidsafe/safe_network/commit/6155ad0334104d367638373fbcbbd7e21631b3e6))
    - Clean up unused functionality ([`11b8182`](https://github.com/maidsafe/safe_network/commit/11b8182a3de636a760d899cb15d7184d8153545a))
    - Leave out unnecessary Arc<RwLock> ([`ddbbb53`](https://github.com/maidsafe/safe_network/commit/ddbbb53d61d6c94b00a47dc2b708a2aeda870d96))
    - Remove unused Session member ([`1235f7d`](https://github.com/maidsafe/safe_network/commit/1235f7d8a92eb9f086c35696bf5c0a8baf67f2ac))
    - Retry more times for connection fails w/ client ([`6471eb8`](https://github.com/maidsafe/safe_network/commit/6471eb88f7ce8c060909930ac23c855f30e8690a))
</details>

## v0.69.0 (2022-08-14)

<csr-id-707df06b08d5b0457b201ce5772d6a1d4fe9f984/>
<csr-id-6d60525874dc4efeb658433f1f253d54e0cba2d4/>
<csr-id-42bde15e9a96dbe759575d4bccf4f769e13a695d/>
<csr-id-29de67f1e3583eab867d517cb50ed2e404bd63fd/>
<csr-id-8242f2f1035b1c0718e53954951badffa30f3393/>
<csr-id-820fcc9a77f756fca308f247c3ea1b82f65d30b9/>
<csr-id-afcf083469c732f10c7c80f4a45e4c33ab111101/>
<csr-id-17f0e8a08c9543d380c16a35d3d7bfe7834a9e5a/>
<csr-id-aafc560d3b3b1e375f7be224e0e63a3b567bbd86/>
<csr-id-73dc9b4a1757393270e62d265328bab0c0aa3b35/>
<csr-id-0a653e4becc4a8e14ffd6d0752cf035430067ce9/>
<csr-id-9789797e3f773285f23bd22957fe45a67aabec24/>
<csr-id-c3196bfdbca221dfa61f978331582fc7d6db72d3/>
<csr-id-3fc072f256dfe4b9e1a1a09c59800c7d78aa7360/>
<csr-id-947b6cad014a41b0336de7f1c31f9902473c1a70/>
<csr-id-8efbd96a5fd3907ace5ca6ac282027595fefd8ef/>
<csr-id-ea490ddf749ac9e0c7962c3c21c053663e6b6ee7/>
<csr-id-6f03b93bd2d02f0ffe54b69fbf25070fbe64eab0/>
<csr-id-1ee345ce00337f9b24d45db417b6bb3d54c67955/>
<csr-id-214adedc31bca576c7f28ff52a1f4ff0a2676757/>
<csr-id-39c3fdf4128462e5f7c5fec3c628d394f505e2f2/>
<csr-id-1e8180c23fab27ac92c93f201efd050cff00db10/>
<csr-id-ec8b69d642fc4ca0166ffff113306244e5c3936a/>
<csr-id-b9bfb425035fead587b7b5fc03a212a5d5aae4b3/>
<csr-id-00fae4d5fd5dbad5696888f0c796fbd39b7e49ed/>
<csr-id-5aeb15c8c309c16878dde510f68b0e5c2122cd8c/>
<csr-id-27ba2a63dcfa272cf7ef8c5301987fc6bfe18ed0/>
<csr-id-6b1fee8cf3d0b2995f4b81e59dd684547593b5fa/>
<csr-id-ed37bb56e5e17d4cba7c1b2165746c193241d618/>
<csr-id-14ea6c7f4bbaee9c2ac4a30fba938ef2de2f77e5/>
<csr-id-db4f4d07b155d732ad76d263563d81b5fee535f7/>
<csr-id-e0fb940b24e87d86fe920095176362f73503ce79/>
<csr-id-ca32230926e5a435d90694df8fbce1218ea397f0/>
<csr-id-3f577d2a6fe70792d7d02e231b599ca3d44a5ed2/>
<csr-id-9fde534277f359dfa0a1d91d917864776edb5138/>
<csr-id-5c82df633e7c062fdf761a8e6e0a7ae8d26cc73a/>
<csr-id-f5af444b8ac37d2debfbe5e1d4dcdc48de963694/>
<csr-id-4d717a21a2daf6ef0b3b5826329a8848f2fe46ee/>
<csr-id-95c33d1ea2040bce4078be96ed8b1c9f2e966b21/>
<csr-id-d4be0cc431947b035046cc4d56642a81c0880924/>
<csr-id-db7dcdc7968d1d7e946274650d5a0c48719b4955/>
<csr-id-d3a05a728be8752ea9ebff4e38e7c4c85e5db09b/>
<csr-id-96da1171d0cac240f772e5d6a15c56f63441b4b3/>
<csr-id-dd2eb21352223f6340064e0021f4a7df402cd5c9/>
<csr-id-53f60c2327f8a69f0b2ef6d1a4e96644c10aa358/>

### Chore

 - <csr-id-707df06b08d5b0457b201ce5772d6a1d4fe9f984/> follow rust convention for getters for prefixmap

 - <csr-id-6d60525874dc4efeb658433f1f253d54e0cba2d4/> remove wiremsg.priority as uneeded
 - <csr-id-42bde15e9a96dbe759575d4bccf4f769e13a695d/> misc. fixes
 - <csr-id-29de67f1e3583eab867d517cb50ed2e404bd63fd/> serialize NetworkPrefixMap into JSON
 - <csr-id-8242f2f1035b1c0718e53954951badffa30f3393/> organise usings, cleanup
 - <csr-id-820fcc9a77f756fca308f247c3ea1b82f65d30b9/> remove NetworkPrefxiMap::genesis_key, NetworkKnowledge::genesis_key
   Remove the feilds as they can be obtained from NetworkPrefixMap::sections_dag
 - <csr-id-afcf083469c732f10c7c80f4a45e4c33ab111101/> remove RwLock from NetworkPrefixMap
 - <csr-id-17f0e8a08c9543d380c16a35d3d7bfe7834a9e5a/> make NetowrkPrefixMap::sections_dag private
 - <csr-id-aafc560d3b3b1e375f7be224e0e63a3b567bbd86/> rename traceroute fns
 - <csr-id-73dc9b4a1757393270e62d265328bab0c0aa3b35/> make traceroute a default feature
 - <csr-id-0a653e4becc4a8e14ffd6d0752cf035430067ce9/> improve Display, Debug impl for Traceroute
 - <csr-id-9789797e3f773285f23bd22957fe45a67aabec24/> improve traceroute readability and other improvements
   - simplfies creating identites for traceroute to avoid locking
   - implements Display and Debug for traceroute
   - add clearer logs for traceroute
 - <csr-id-c3196bfdbca221dfa61f978331582fc7d6db72d3/> remove unused error variants
 - <csr-id-3fc072f256dfe4b9e1a1a09c59800c7d78aa7360/> remove unused config dir/file
 - <csr-id-947b6cad014a41b0336de7f1c31f9902473c1a70/> remove unused channel from Client
   Previously there used to be a channel that propagated up errors, this
   usage was removed in commit 1e4049fc94
 - <csr-id-8efbd96a5fd3907ace5ca6ac282027595fefd8ef/> Cleanup non-joined member sessions, regardless of connected state
 - <csr-id-ea490ddf749ac9e0c7962c3c21c053663e6b6ee7/> reflect the semantics not the type
 - <csr-id-6f03b93bd2d02f0ffe54b69fbf25070fbe64eab0/> upgrade blsttc to 7.0.0
   This version has a more helpful error message for the shares interpolation problem.
 - <csr-id-1ee345ce00337f9b24d45db417b6bb3d54c67955/> increase query retry count as we may drop connections now
 - <csr-id-214adedc31bca576c7f28ff52a1f4ff0a2676757/> improve traceroute redability and resolve clippy
 - <csr-id-39c3fdf4128462e5f7c5fec3c628d394f505e2f2/> remove unused console-subscriber
 - <csr-id-1e8180c23fab27ac92c93f201efd050cff00db10/> re-enable registers benchmark and tidy sled residue
 - <csr-id-ec8b69d642fc4ca0166ffff113306244e5c3936a/> remove extra bootstrap step
   Remove `Session::genesis_key` as it's unused. Can be obtained
   from `NetworkPrefixMap` if required.
 - <csr-id-b9bfb425035fead587b7b5fc03a212a5d5aae4b3/> add binary to query chunk at adults
 - <csr-id-00fae4d5fd5dbad5696888f0c796fbd39b7e49ed/> formatting with cargo fmt
 - <csr-id-5aeb15c8c309c16878dde510f68b0e5c2122cd8c/> move to dev-dependencies

### Chore

 - <csr-id-53f60c2327f8a69f0b2ef6d1a4e96644c10aa358/> sn_interface-0.9.0/sn_dysfunction-0.8.0/sn_client-0.69.0/sn_node-0.65.0/sn_api-0.67.0/sn_cli-0.60.0

### Documentation

 - <csr-id-753443da697a61e49eac977402731c4373e7f4f9/> add client builder code example
 - <csr-id-70ea78222875eb947e684af6db7544927f0bfe77/> remove TODOs from public docs
 - <csr-id-49313f62b5a174a9b278c1c5d18baccdf4bb8c85/> fix link to config field

### New Features

<csr-id-ba97ca06b67cd6e5de8e1c910b396fbe44f40fd7/>
<csr-id-df5ea26c8243de70d16a75ac936bc322954c8436/>

 - <csr-id-4772ff129bd8da82465ef93e66d17a8fbbd38f7d/> ClientBuilder to instantiate Client
   This applies the builder pattern for creating a Client
 - <csr-id-175d1b909dff8c6729ac7f156ce1d0d22be8cc12/> make traceroute default for now
 - <csr-id-4f2cf267ee030e5924a2fa999a2a46dbc072d208/> impl traceroute for client cmds and cmd responses
 - <csr-id-a6fb1fc516a9ef6dae7aa236f3dd440d50697ae2/> impl traceroute feature to trace a message's flow in the network
   - implements traceroute for Client queries and is logged at the client on return

### Bug Fixes

<csr-id-0ed5075304b090597f7760fb51c4a33435a853f1/>
<csr-id-f0ad7d56a58a08a7591d978c8ead4c10db734276/>
<csr-id-db525193bed7662c5184810f18587abb0d22b26b/>
<csr-id-145b302aad291120c52f1cffad8e7d116682f532/>
<csr-id-d8cc45384f891a9d95a7cef30159f11ec0ff9269/>
<csr-id-a378e7ba67ec18be708a2e1a9e08e63519da7451/>
<csr-id-950b3048d1aae1f9ad5d2218a42c34d662925e38/>

 - <csr-id-0041e18ab7d1a21e4debb39df9c4b116e002a5e5/> convert nodes joining interval to millis before passing it to launch-tool
   - Also pass the default prefix map file path as the network contacts file path to CLI node join cmd.

### Refactor

 - <csr-id-27ba2a63dcfa272cf7ef8c5301987fc6bfe18ed0/> sn_client to only read a default prefix map file, updates to be cached on disk by user
   - CLI to cache the up to date PrefixMap after all commands were executed and right before exiting.
   - Refactoring sn_cli::Config to remove some redundant code.
 - <csr-id-6b1fee8cf3d0b2995f4b81e59dd684547593b5fa/> reduce AE msgs to one msg with a kind field
 - <csr-id-ed37bb56e5e17d4cba7c1b2165746c193241d618/> move SectionChain into NetworkPrefixMap
 - <csr-id-14ea6c7f4bbaee9c2ac4a30fba938ef2de2f77e5/> use builder to instantiate
 - <csr-id-db4f4d07b155d732ad76d263563d81b5fee535f7/> remove more unused code
 - <csr-id-e0fb940b24e87d86fe920095176362f73503ce79/> use sn_dbc::SpentProof API for verifying SpentProofShares
 - <csr-id-ca32230926e5a435d90694df8fbce1218ea397f0/> remove unused storage path
 - <csr-id-3f577d2a6fe70792d7d02e231b599ca3d44a5ed2/> rename gen_section_authority_provider to random_sap
 - <csr-id-9fde534277f359dfa0a1d91d917864776edb5138/> reissuing DBCs for all sn_cli tests only once as a setup stage
 - <csr-id-5c82df633e7c062fdf761a8e6e0a7ae8d26cc73a/> setup step for tests to reissue a set of DBCs from genesis only once
 - <csr-id-f5af444b8ac37d2debfbe5e1d4dcdc48de963694/> removing hard-coded test DBC from sn_api Wallet unit tests

### Style

 - <csr-id-4d717a21a2daf6ef0b3b5826329a8848f2fe46ee/> tweak sn_client/Cargo.toml formatting TOML

### Test

 - <csr-id-95c33d1ea2040bce4078be96ed8b1c9f2e966b21/> have many-clients test to report the errors found when instantiating clients
 - <csr-id-d4be0cc431947b035046cc4d56642a81c0880924/> additional tests in sn-api for DBC verification failures

### Chore (BREAKING)

 - <csr-id-db7dcdc7968d1d7e946274650d5a0c48719b4955/> remove providing path to qp2p cfg
   This configuration seems never to be provided or stored anyway. It looks
   like some code was also taking this parameter to be the client config,
   not the qp2p config, which is a source of confusion.
 - <csr-id-d3a05a728be8752ea9ebff4e38e7c4c85e5db09b/> having spent proofs and Txs within SpentbookCmd::Send msg to be a set instead of a vec

### New Features (BREAKING)

 - <csr-id-40a5f2d968aff30e7b92fe325aa299deddb49e69/> remove client config in favour of builder
   The config_handler module handled most of the configurable parameters to
   the client, but this is superseded by the client builder. The client
   builder is now the dedicated place for the logic pertaining to
   instantiating the client.
 - <csr-id-923cdfded98132e94473db04e01d5fe83f73ca3d/> adjust client instantiation
   Force the user to provide a DBC owner and Keypair to use with the
   client. The builder pattern will then take over the convenience of
   providing defaults (generating new keys) for this if not provided.
 - <csr-id-f666204febb1044980412345236ce0cb8377b162/> return reference instead of clone
   Let the end user decide on wether to clone a value that is taken from
   the struct.

### Refactor (BREAKING)

 - <csr-id-96da1171d0cac240f772e5d6a15c56f63441b4b3/> nodes to cache their own individual prefix map file on disk
 - <csr-id-dd2eb21352223f6340064e0021f4a7df402cd5c9/> removing Token from sn_interfaces::type as it is now exposed by sn_dbc

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 69 commits contributed to the release over the course of 32 calendar days.
 - 34 days passed between releases.
 - 61 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.9.0/sn_dysfunction-0.8.0/sn_client-0.69.0/sn_node-0.65.0/sn_api-0.67.0/sn_cli-0.60.0 ([`53f60c2`](https://github.com/maidsafe/safe_network/commit/53f60c2327f8a69f0b2ef6d1a4e96644c10aa358))
    - Follow rust convention for getters for prefixmap ([`707df06`](https://github.com/maidsafe/safe_network/commit/707df06b08d5b0457b201ce5772d6a1d4fe9f984))
    - Sn_client to only read a default prefix map file, updates to be cached on disk by user ([`27ba2a6`](https://github.com/maidsafe/safe_network/commit/27ba2a63dcfa272cf7ef8c5301987fc6bfe18ed0))
    - Remove wiremsg.priority as uneeded ([`6d60525`](https://github.com/maidsafe/safe_network/commit/6d60525874dc4efeb658433f1f253d54e0cba2d4))
    - Have many-clients test to report the errors found when instantiating clients ([`95c33d1`](https://github.com/maidsafe/safe_network/commit/95c33d1ea2040bce4078be96ed8b1c9f2e966b21))
    - Misc. fixes ([`42bde15`](https://github.com/maidsafe/safe_network/commit/42bde15e9a96dbe759575d4bccf4f769e13a695d))
    - Add client builder code example ([`753443d`](https://github.com/maidsafe/safe_network/commit/753443da697a61e49eac977402731c4373e7f4f9))
    - Remove client config in favour of builder ([`40a5f2d`](https://github.com/maidsafe/safe_network/commit/40a5f2d968aff30e7b92fe325aa299deddb49e69))
    - Convert nodes joining interval to millis before passing it to launch-tool ([`0041e18`](https://github.com/maidsafe/safe_network/commit/0041e18ab7d1a21e4debb39df9c4b116e002a5e5))
    - Serialize NetworkPrefixMap into JSON ([`29de67f`](https://github.com/maidsafe/safe_network/commit/29de67f1e3583eab867d517cb50ed2e404bd63fd))
    - Nodes to cache their own individual prefix map file on disk ([`96da117`](https://github.com/maidsafe/safe_network/commit/96da1171d0cac240f772e5d6a15c56f63441b4b3))
    - Reduce AE msgs to one msg with a kind field ([`6b1fee8`](https://github.com/maidsafe/safe_network/commit/6b1fee8cf3d0b2995f4b81e59dd684547593b5fa))
    - Removing Token from sn_interfaces::type as it is now exposed by sn_dbc ([`dd2eb21`](https://github.com/maidsafe/safe_network/commit/dd2eb21352223f6340064e0021f4a7df402cd5c9))
    - Chore(style): organise usings, cleanup - Removes some boilerplate, using fn of `Cmd` to instantiate a send cmd. - Housekeeping, continuing to minimize bloat of usings, by colocating them. - Housekeeping, continuing keeping positions of usings in a file according to a system, from closest (self) on top, down to furthest away (3rd part). ([`8242f2f`](https://github.com/maidsafe/safe_network/commit/8242f2f1035b1c0718e53954951badffa30f3393))
    - Remove NetworkPrefxiMap::genesis_key, NetworkKnowledge::genesis_key ([`820fcc9`](https://github.com/maidsafe/safe_network/commit/820fcc9a77f756fca308f247c3ea1b82f65d30b9))
    - Fix deadlock introduced after removal of Arc from NetworkPrefixMap ([`0ed5075`](https://github.com/maidsafe/safe_network/commit/0ed5075304b090597f7760fb51c4a33435a853f1))
    - Remove RwLock from NetworkPrefixMap ([`afcf083`](https://github.com/maidsafe/safe_network/commit/afcf083469c732f10c7c80f4a45e4c33ab111101))
    - Make NetowrkPrefixMap::sections_dag private ([`17f0e8a`](https://github.com/maidsafe/safe_network/commit/17f0e8a08c9543d380c16a35d3d7bfe7834a9e5a))
    - Move SectionChain into NetworkPrefixMap ([`ed37bb5`](https://github.com/maidsafe/safe_network/commit/ed37bb56e5e17d4cba7c1b2165746c193241d618))
    - Rename traceroute fns ([`aafc560`](https://github.com/maidsafe/safe_network/commit/aafc560d3b3b1e375f7be224e0e63a3b567bbd86))
    - Make traceroute a default feature ([`73dc9b4`](https://github.com/maidsafe/safe_network/commit/73dc9b4a1757393270e62d265328bab0c0aa3b35))
    - Improve Display, Debug impl for Traceroute ([`0a653e4`](https://github.com/maidsafe/safe_network/commit/0a653e4becc4a8e14ffd6d0752cf035430067ce9))
    - Improve traceroute readability and other improvements ([`9789797`](https://github.com/maidsafe/safe_network/commit/9789797e3f773285f23bd22957fe45a67aabec24))
    - Use builder to instantiate ([`14ea6c7`](https://github.com/maidsafe/safe_network/commit/14ea6c7f4bbaee9c2ac4a30fba938ef2de2f77e5))
    - Adjust client instantiation ([`923cdfd`](https://github.com/maidsafe/safe_network/commit/923cdfded98132e94473db04e01d5fe83f73ca3d))
    - Return reference instead of clone ([`f666204`](https://github.com/maidsafe/safe_network/commit/f666204febb1044980412345236ce0cb8377b162))
    - Remove TODOs from public docs ([`70ea782`](https://github.com/maidsafe/safe_network/commit/70ea78222875eb947e684af6db7544927f0bfe77))
    - Remove unused error variants ([`c3196bf`](https://github.com/maidsafe/safe_network/commit/c3196bfdbca221dfa61f978331582fc7d6db72d3))
    - Remove unused config dir/file ([`3fc072f`](https://github.com/maidsafe/safe_network/commit/3fc072f256dfe4b9e1a1a09c59800c7d78aa7360))
    - Fix link to config field ([`49313f6`](https://github.com/maidsafe/safe_network/commit/49313f62b5a174a9b278c1c5d18baccdf4bb8c85))
    - Remove unused channel from Client ([`947b6ca`](https://github.com/maidsafe/safe_network/commit/947b6cad014a41b0336de7f1c31f9902473c1a70))
    - Remove providing path to qp2p cfg ([`db7dcdc`](https://github.com/maidsafe/safe_network/commit/db7dcdc7968d1d7e946274650d5a0c48719b4955))
    - ClientBuilder to instantiate Client ([`4772ff1`](https://github.com/maidsafe/safe_network/commit/4772ff129bd8da82465ef93e66d17a8fbbd38f7d))
    - Refactor(messaging): remove more unused code More reuse of methods to replace duplication of code. Deprecates delivery group, since it is no longer used. Also, `DstLocation` and `SrcLocation` are removed. BREAKING CHANGE: WireMsg public type is changed. ([`db4f4d0`](https://github.com/maidsafe/safe_network/commit/db4f4d07b155d732ad76d263563d81b5fee535f7))
    - More attempt when query too close to the spend cmd ([`f0ad7d5`](https://github.com/maidsafe/safe_network/commit/f0ad7d56a58a08a7591d978c8ead4c10db734276))
    - Use sn_dbc::SpentProof API for verifying SpentProofShares ([`e0fb940`](https://github.com/maidsafe/safe_network/commit/e0fb940b24e87d86fe920095176362f73503ce79))
    - Remove unused storage path ([`ca32230`](https://github.com/maidsafe/safe_network/commit/ca32230926e5a435d90694df8fbce1218ea397f0))
    - Revert "feat: make traceroute default for now" ([`e9b97c7`](https://github.com/maidsafe/safe_network/commit/e9b97c72b860053285ba866b098937f6b25d99bf))
    - Having spent proofs and Txs within SpentbookCmd::Send msg to be a set instead of a vec ([`d3a05a7`](https://github.com/maidsafe/safe_network/commit/d3a05a728be8752ea9ebff4e38e7c4c85e5db09b))
    - Make traceroute default for now ([`175d1b9`](https://github.com/maidsafe/safe_network/commit/175d1b909dff8c6729ac7f156ce1d0d22be8cc12))
    - Chore: Cleanup non-joined member sessions, regardless of connected state This reverts commit 7d12399edec6c1191c521528c1d569afc96bca99. ([`8efbd96`](https://github.com/maidsafe/safe_network/commit/8efbd96a5fd3907ace5ca6ac282027595fefd8ef))
    - Chore(naming): reflect the semantics not the type The type is named Kind but the semantics of it is Auth. Often we mindlessly name things after the type names instead of what they represent in the domain. BREAKING CHANGE: fields of public msg renamed ([`ea490dd`](https://github.com/maidsafe/safe_network/commit/ea490ddf749ac9e0c7962c3c21c053663e6b6ee7))
    - Rename gen_section_authority_provider to random_sap ([`3f577d2`](https://github.com/maidsafe/safe_network/commit/3f577d2a6fe70792d7d02e231b599ca3d44a5ed2))
    - Upgrade blsttc to 7.0.0 ([`6f03b93`](https://github.com/maidsafe/safe_network/commit/6f03b93bd2d02f0ffe54b69fbf25070fbe64eab0))
    - Increase query retry count as we may drop connections now ([`1ee345c`](https://github.com/maidsafe/safe_network/commit/1ee345ce00337f9b24d45db417b6bb3d54c67955))
    - Impl traceroute for client cmds and cmd responses ([`4f2cf26`](https://github.com/maidsafe/safe_network/commit/4f2cf267ee030e5924a2fa999a2a46dbc072d208))
    - Improve traceroute redability and resolve clippy ([`214aded`](https://github.com/maidsafe/safe_network/commit/214adedc31bca576c7f28ff52a1f4ff0a2676757))
    - Impl traceroute feature to trace a message's flow in the network ([`a6fb1fc`](https://github.com/maidsafe/safe_network/commit/a6fb1fc516a9ef6dae7aa236f3dd440d50697ae2))
    - Additional tests in sn-api for DBC verification failures ([`d4be0cc`](https://github.com/maidsafe/safe_network/commit/d4be0cc431947b035046cc4d56642a81c0880924))
    - Reissuing DBCs for all sn_cli tests only once as a setup stage ([`9fde534`](https://github.com/maidsafe/safe_network/commit/9fde534277f359dfa0a1d91d917864776edb5138))
    - Perform verification of input TX and spentproofs when depositing or reissuing a DBC ([`ba97ca0`](https://github.com/maidsafe/safe_network/commit/ba97ca06b67cd6e5de8e1c910b396fbe44f40fd7))
    - Remove unused console-subscriber ([`39c3fdf`](https://github.com/maidsafe/safe_network/commit/39c3fdf4128462e5f7c5fec3c628d394f505e2f2))
    - Setup step for tests to reissue a set of DBCs from genesis only once ([`5c82df6`](https://github.com/maidsafe/safe_network/commit/5c82df633e7c062fdf761a8e6e0a7ae8d26cc73a))
    - Re-enable registers benchmark and tidy sled residue ([`1e8180c`](https://github.com/maidsafe/safe_network/commit/1e8180c23fab27ac92c93f201efd050cff00db10))
    - Remove extra bootstrap step ([`ec8b69d`](https://github.com/maidsafe/safe_network/commit/ec8b69d642fc4ca0166ffff113306244e5c3936a))
    - Use Eyre instead of boxed error ([`db52519`](https://github.com/maidsafe/safe_network/commit/db525193bed7662c5184810f18587abb0d22b26b))
    - Add timeout for queries ([`df5ea26`](https://github.com/maidsafe/safe_network/commit/df5ea26c8243de70d16a75ac936bc322954c8436))
    - Add binary to query chunk at adults ([`b9bfb42`](https://github.com/maidsafe/safe_network/commit/b9bfb425035fead587b7b5fc03a212a5d5aae4b3))
    - Formatting with cargo fmt ([`00fae4d`](https://github.com/maidsafe/safe_network/commit/00fae4d5fd5dbad5696888f0c796fbd39b7e49ed))
    - Unused async in sn_client ([`145b302`](https://github.com/maidsafe/safe_network/commit/145b302aad291120c52f1cffad8e7d116682f532))
    - Unused async remove and up-chain ([`d8cc453`](https://github.com/maidsafe/safe_network/commit/d8cc45384f891a9d95a7cef30159f11ec0ff9269))
    - Remove unused Arc(RwLock) structure ([`a378e7b`](https://github.com/maidsafe/safe_network/commit/a378e7ba67ec18be708a2e1a9e08e63519da7451))
    - Removing hard-coded test DBC from sn_api Wallet unit tests ([`f5af444`](https://github.com/maidsafe/safe_network/commit/f5af444b8ac37d2debfbe5e1d4dcdc48de963694))
    - Merge branch 'main' into feat-cat-wallet-improvements ([`08a3b85`](https://github.com/maidsafe/safe_network/commit/08a3b85ae73b2360e63f9d4fbdec23e349dc0626))
    - Merge #1323 ([`ec1499d`](https://github.com/maidsafe/safe_network/commit/ec1499d2a2ff0177b571f510c585ab71a2176cda))
    - Upon receiving an AE msg update client knowledge of network sections chains ([`950b304`](https://github.com/maidsafe/safe_network/commit/950b3048d1aae1f9ad5d2218a42c34d662925e38))
    - Merge branch 'main' into feat-cat-wallet-improvements ([`e2e89e6`](https://github.com/maidsafe/safe_network/commit/e2e89e6b061ae0827cdeeb1d8b17e702d2f3607a))
    - Tweak sn_client/Cargo.toml formatting TOML ([`4d717a2`](https://github.com/maidsafe/safe_network/commit/4d717a21a2daf6ef0b3b5826329a8848f2fe46ee))
    - Move to dev-dependencies ([`5aeb15c`](https://github.com/maidsafe/safe_network/commit/5aeb15c8c309c16878dde510f68b0e5c2122cd8c))
</details>

## v0.68.2 (2022-07-10)

<csr-id-19ddebad43aa53822cb7e781913ba34b848e2c89/>
<csr-id-49e223e2c07695b4c63e253ba19ce43ec24d7112/>
<csr-id-5cff2c5325a854f04788f9111439bca75b21c60f/>
<csr-id-34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8/>

### Chore

 - <csr-id-19ddebad43aa53822cb7e781913ba34b848e2c89/> remove unused utilities
 - <csr-id-49e223e2c07695b4c63e253ba19ce43ec24d7112/> move more deps to clap-v3; rm some deps on rand
 - <csr-id-5cff2c5325a854f04788f9111439bca75b21c60f/> ignore store_and_read_40mb as too heavy for CI

### Chore

 - <csr-id-34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8/> sn_interface-0.8.2/sn_client-0.68.2/sn_node-0.64.2/sn_api-0.66.3/sn_cli-0.59.3

### Bug Fixes

 - <csr-id-64eb333d532694f46f1d0b9dd5109961b3551802/> for QueryResponse, set correlation_id to be the origin msg_id
 - <csr-id-3c383ccf9ad0ed77080fb3e3ec459e5b02158505/> passing the churn test
   This commit contains the work to passing the churn test.
   There are mainly two fixes:
   1, Only trigger data reorganization when there is membership update.
   Previously, data reorganzation get undertaken whenever there is
   incoming message. Which result in a looping of messaging among
   nodes.
   2, Only broadcast result when the QueryResponse is not an error.
   Previously, this will cause the client thinking the whole query
   is failed whenever an error response received.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 2 calendar days.
 - 2 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.8.2/sn_client-0.68.2/sn_node-0.64.2/sn_api-0.66.3/sn_cli-0.59.3 ([`34bd9bd`](https://github.com/maidsafe/safe_network/commit/34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8))
    - Remove unused utilities ([`19ddeba`](https://github.com/maidsafe/safe_network/commit/19ddebad43aa53822cb7e781913ba34b848e2c89))
    - Move more deps to clap-v3; rm some deps on rand ([`49e223e`](https://github.com/maidsafe/safe_network/commit/49e223e2c07695b4c63e253ba19ce43ec24d7112))
    - Merge #1301 ([`9c6914e`](https://github.com/maidsafe/safe_network/commit/9c6914e2688f70a25ad5dfe74307572cb8e8fcc2))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`45418f2`](https://github.com/maidsafe/safe_network/commit/45418f2f9b5cc58f2a153bf40966beb2bf36a62a))
    - Ignore store_and_read_40mb as too heavy for CI ([`5cff2c5`](https://github.com/maidsafe/safe_network/commit/5cff2c5325a854f04788f9111439bca75b21c60f))
    - For QueryResponse, set correlation_id to be the origin msg_id ([`64eb333`](https://github.com/maidsafe/safe_network/commit/64eb333d532694f46f1d0b9dd5109961b3551802))
    - Passing the churn test ([`3c383cc`](https://github.com/maidsafe/safe_network/commit/3c383ccf9ad0ed77080fb3e3ec459e5b02158505))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`45309c4`](https://github.com/maidsafe/safe_network/commit/45309c4c0463dd9198a49537187417bf4bfdb847))
    - Merge branch 'main' into feat-cli-wallet-show-deposited-amount ([`6268fe7`](https://github.com/maidsafe/safe_network/commit/6268fe76e9dd81d291492b4611094273f8d1e223))
</details>

## v0.68.1 (2022-07-07)

<csr-id-77cef496695e8cac9ccefccaf99cf350fb479eb9/>
<csr-id-da13669193d93b3a56fff4a956c9ac9830055a7a/>
<csr-id-85ca7ce23414bf19e72236e32745b0fb6239664d/>
<csr-id-46262268fc167c05963e5b7bd6261310496e2379/>
<csr-id-8dccb7f1fc81385f9f5f25e6c354ad1d35759528/>
<csr-id-2d0e23cdc7f94b0cc2d13ddf8203702cec4d3a07/>
<csr-id-90059dc4edf35fc7d53bc25b485be291a1de9807/>
<csr-id-2b00cec961561281f6b927e13e501342843f6a0f/>

### Chore

 - <csr-id-77cef496695e8cac9ccefccaf99cf350fb479eb9/> now we only contact one adult at a time increase retry count
   This should get us more contact with more elders in the same amount of time as previous.
   Only returning faster if initial adult query returns
 - <csr-id-da13669193d93b3a56fff4a956c9ac9830055a7a/> use latest sn_launch_tool release, sans StructOpt
 - <csr-id-85ca7ce23414bf19e72236e32745b0fb6239664d/> replace StructOpt with Clap in sn_client
 - <csr-id-46262268fc167c05963e5b7bd6261310496e2379/> `try!` macro is deprecated
   No need for rustfmt to check/replace this, as the compiler will already
   warn for this. Deprecated since 1.39.
   
   Removing the option seems to trigger a couple of formatting changes that
   rustfmt did not seem to pick on before.
 - <csr-id-8dccb7f1fc81385f9f5f25e6c354ad1d35759528/> clippy runs cargo check already
 - <csr-id-2d0e23cdc7f94b0cc2d13ddf8203702cec4d3a07/> churn example tweaks

### Test

 - <csr-id-90059dc4edf35fc7d53bc25b485be291a1de9807/> adapt client_api spentbook test to read genesis DBC from first node in testnet
   We temporarily allow double spents in this test. Once we have the SpentBook implementation which prevents
   double spents, we'll need to adapt this test to verify there is no double spent of the genesis DBC.

### Chore

 - <csr-id-2b00cec961561281f6b927e13e501342843f6a0f/> sn_interface-0.8.1/sn_dysfunction-0.7.1/sn_client-0.68.1/sn_node-0.64.1/sn_api-0.66.1/sn_cli-0.59.1

### Bug Fixes

 - <csr-id-1e3f865ae5e32520958bb071bc7fbffe8b79a033/> reduce client waiting time on receiving responses

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 19 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.8.1/sn_dysfunction-0.7.1/sn_client-0.68.1/sn_node-0.64.1/sn_api-0.66.1/sn_cli-0.59.1 ([`2b00cec`](https://github.com/maidsafe/safe_network/commit/2b00cec961561281f6b927e13e501342843f6a0f))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`f83724c`](https://github.com/maidsafe/safe_network/commit/f83724cff1e63b35f1612fc82dffdefbeaab6cc1))
    - Merge branch 'main' into feat-cli-wallet-show-deposited-amount ([`cd2f9aa`](https://github.com/maidsafe/safe_network/commit/cd2f9aa2f7001ae779273745f9ac78fc289525e3))
    - Merge #1308 ([`8421959`](https://github.com/maidsafe/safe_network/commit/8421959b6a80e4386c34fcd6f86a1af5044280ec))
    - Merge branch 'main' into feat-cli-wallet-show-deposited-amount ([`39bd5b4`](https://github.com/maidsafe/safe_network/commit/39bd5b471b6b3acb6ebe90489335c995b0aca82f))
    - Merge branch 'main' into cargo-husky-tweaks ([`6881855`](https://github.com/maidsafe/safe_network/commit/688185573bb71cc44a7103df17f3fbeea6740247))
    - Adapt client_api spentbook test to read genesis DBC from first node in testnet ([`90059dc`](https://github.com/maidsafe/safe_network/commit/90059dc4edf35fc7d53bc25b485be291a1de9807))
    - Merge #1309 ([`f9fa4f7`](https://github.com/maidsafe/safe_network/commit/f9fa4f7857d8161e8c036cca06006bf187a6c6c3))
    - Merge branch 'main' into feat-cmd-parent-id ([`e10aaa2`](https://github.com/maidsafe/safe_network/commit/e10aaa2cf0404bfa10ef55b7c9dc7ae6fc0d28e5))
    - Merge branch 'main' into cargo-husky-tweaks ([`52dd02e`](https://github.com/maidsafe/safe_network/commit/52dd02e45ab4e160b0a26498919a79ce1aefb1bd))
    - Merge branch 'main' into refactor_messaging ([`349d432`](https://github.com/maidsafe/safe_network/commit/349d43295a44b529cbb138cf2fff9483b03fea07))
    - Now we only contact one adult at a time increase retry count ([`77cef49`](https://github.com/maidsafe/safe_network/commit/77cef496695e8cac9ccefccaf99cf350fb479eb9))
    - Use latest sn_launch_tool release, sans StructOpt ([`da13669`](https://github.com/maidsafe/safe_network/commit/da13669193d93b3a56fff4a956c9ac9830055a7a))
    - Replace StructOpt with Clap in sn_client ([`85ca7ce`](https://github.com/maidsafe/safe_network/commit/85ca7ce23414bf19e72236e32745b0fb6239664d))
    - `try!` macro is deprecated ([`4626226`](https://github.com/maidsafe/safe_network/commit/46262268fc167c05963e5b7bd6261310496e2379))
    - Clippy runs cargo check already ([`8dccb7f`](https://github.com/maidsafe/safe_network/commit/8dccb7f1fc81385f9f5f25e6c354ad1d35759528))
    - Merge #1304 ([`6af41dc`](https://github.com/maidsafe/safe_network/commit/6af41dcbad76903cb5526b270100e650aa483191))
    - Churn example tweaks ([`2d0e23c`](https://github.com/maidsafe/safe_network/commit/2d0e23cdc7f94b0cc2d13ddf8203702cec4d3a07))
    - Reduce client waiting time on receiving responses ([`1e3f865`](https://github.com/maidsafe/safe_network/commit/1e3f865ae5e32520958bb071bc7fbffe8b79a033))
</details>

## v0.68.0 (2022-07-04)

<csr-id-9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7/>
<csr-id-d4850ff81d33751ebf9e3a7c7af438f160df6e44/>
<csr-id-ddb7798a7b0c5e60960e123414277d58f3da27eb/>
<csr-id-4e04a2b0acc79140bf1d0aefd82c0ad5b046a3cd/>
<csr-id-6a2553a11b1404ad404e67df29bf3ec535d1b954/>
<csr-id-5dbf50d92bf7e93acbb00e85f51910f32ac4a124/>
<csr-id-849dfba283362d8fbdddd92be1078c3a963fb564/>
<csr-id-91da4d4ac7aab039853b0651e5aafd9cdd31b9c4/>
<csr-id-e4e2eb56611a328806c59ed8bc80ca2567206bbb/>

### Chore

 - <csr-id-9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7/> Docs - put symbols in backticks
 - <csr-id-d4850ff81d33751ebf9e3a7c7af438f160df6e44/> clippy clea up unused
 - <csr-id-ddb7798a7b0c5e60960e123414277d58f3da27eb/> remove let bindings for unit returns
 - <csr-id-4e04a2b0acc79140bf1d0aefd82c0ad5b046a3cd/> remove unused asyncs (clippy)
   Upon removing async keywords from
   sn_interface/src/network_knowledge/mod.rs a lot of removal propagated up
   and removed most of it with help of Clippy. Clippy does not yet detect
   unnecessary async in methods
   (https://github.com/rust-lang/rust-clippy/issues/9024), but will soon.
   
   With the help of a new Clippy lint:
   cargo clippy --all-targets --all-features -- -W clippy::unused_async
   And automatically fixing code with:
   cargo fix --broken-code --allow-dirty --all-targets --all-features
   
   Results mostly from the single thread work of @joshuef in #1253 (and
   ongoing efforts).

### Chore

 - <csr-id-e4e2eb56611a328806c59ed8bc80ca2567206bbb/> sn_interface-0.8.0/sn_dysfunction-0.7.0/sn_client-0.68.0/sn_node-0.64.0/sn_api-0.66.0/sn_cli-0.59.0

### Refactor

 - <csr-id-6a2553a11b1404ad404e67df29bf3ec535d1b954/> remove NetworkInfo::GenesisKey variant
 - <csr-id-5dbf50d92bf7e93acbb00e85f51910f32ac4a124/> remove NodeConfig from sn_api::ipc, add sn_cli tests
 - <csr-id-849dfba283362d8fbdddd92be1078c3a963fb564/> update PrefixMap symlink if incorrect
 - <csr-id-91da4d4ac7aab039853b0651e5aafd9cdd31b9c4/> remove node_connection_info.config from sn_node, sn_interface, sn_client

### New Features (BREAKING)

 - <csr-id-5dad80d3f239f5844243fedb89f8d4baaee3b640/> have the nodes to attach valid Commitments to signed SpentProofShares

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 6 calendar days.
 - 6 days passed between releases.
 - 10 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.8.0/sn_dysfunction-0.7.0/sn_client-0.68.0/sn_node-0.64.0/sn_api-0.66.0/sn_cli-0.59.0 ([`e4e2eb5`](https://github.com/maidsafe/safe_network/commit/e4e2eb56611a328806c59ed8bc80ca2567206bbb))
    - Remove NetworkInfo::GenesisKey variant ([`6a2553a`](https://github.com/maidsafe/safe_network/commit/6a2553a11b1404ad404e67df29bf3ec535d1b954))
    - Remove NodeConfig from sn_api::ipc, add sn_cli tests ([`5dbf50d`](https://github.com/maidsafe/safe_network/commit/5dbf50d92bf7e93acbb00e85f51910f32ac4a124))
    - Update PrefixMap symlink if incorrect ([`849dfba`](https://github.com/maidsafe/safe_network/commit/849dfba283362d8fbdddd92be1078c3a963fb564))
    - Remove node_connection_info.config from sn_node, sn_interface, sn_client ([`91da4d4`](https://github.com/maidsafe/safe_network/commit/91da4d4ac7aab039853b0651e5aafd9cdd31b9c4))
    - Docs - put symbols in backticks ([`9314a2d`](https://github.com/maidsafe/safe_network/commit/9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7))
    - Clippy clea up unused ([`d4850ff`](https://github.com/maidsafe/safe_network/commit/d4850ff81d33751ebf9e3a7c7af438f160df6e44))
    - Remove let bindings for unit returns ([`ddb7798`](https://github.com/maidsafe/safe_network/commit/ddb7798a7b0c5e60960e123414277d58f3da27eb))
    - Have the nodes to attach valid Commitments to signed SpentProofShares ([`5dad80d`](https://github.com/maidsafe/safe_network/commit/5dad80d3f239f5844243fedb89f8d4baaee3b640))
    - Remove unused asyncs (clippy) ([`4e04a2b`](https://github.com/maidsafe/safe_network/commit/4e04a2b0acc79140bf1d0aefd82c0ad5b046a3cd))
</details>

## v0.67.1 (2022-06-28)

<csr-id-8c69306dc86a99a8be443ab8213253983540f1cf/>
<csr-id-58890e5c919ada30f27d4e80c6b5e7291b99ed5c/>

### New Features

 - <csr-id-44b93fde435214b363c009e555a2579bb3404e75/> use node's section_key and own key for register
 - <csr-id-6bfd101ed12a16f3f6a9a0b55252d45d200af7c6/> Select which adult to query
   Let the client pick the adult to query, based on the XOR distance.

### Refactor

 - <csr-id-8c69306dc86a99a8be443ab8213253983540f1cf/> Rename DataQuery with suffix Variant
   A new structure with the name DataQuery will be introduced that has common data for all these
   variants.

### Chore

 - <csr-id-58890e5c919ada30f27d4e80c6b5e7291b99ed5c/> sn_interface-0.7.1/sn_dysfunction-0.6.1/sn_client-0.67.1/sn_node-0.63.1

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 2 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.7.1/sn_dysfunction-0.6.1/sn_client-0.67.1/sn_node-0.63.1 ([`58890e5`](https://github.com/maidsafe/safe_network/commit/58890e5c919ada30f27d4e80c6b5e7291b99ed5c))
    - Use node's section_key and own key for register ([`44b93fd`](https://github.com/maidsafe/safe_network/commit/44b93fde435214b363c009e555a2579bb3404e75))
    - Select which adult to query ([`6bfd101`](https://github.com/maidsafe/safe_network/commit/6bfd101ed12a16f3f6a9a0b55252d45d200af7c6))
    - Rename DataQuery with suffix Variant ([`8c69306`](https://github.com/maidsafe/safe_network/commit/8c69306dc86a99a8be443ab8213253983540f1cf))
</details>

## v0.67.0 (2022-06-26)

<csr-id-243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e/>
<csr-id-c85dc4c7a07d2f5343490328c593cceb0f50c6aa/>
<csr-id-697947510688c114699b5317f219ef625c29c6d1/>

### Chore

 - <csr-id-243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e/> sn_interface-0.7.0/sn_dysfunction-0.6.0/sn_client-0.67.0/sn_node-0.63.0/sn_api-0.65.0/sn_cli-0.58.0
 - <csr-id-c85dc4c7a07d2f5343490328c593cceb0f50c6aa/> more tweaks to benchmarks for clippy
 - <csr-id-697947510688c114699b5317f219ef625c29c6d1/> fix &Vec -> &[] clippy warning

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.7.0/sn_dysfunction-0.6.0/sn_client-0.67.0/sn_node-0.63.0/sn_api-0.65.0/sn_cli-0.58.0 ([`243cfc4`](https://github.com/maidsafe/safe_network/commit/243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e))
    - More tweaks to benchmarks for clippy ([`c85dc4c`](https://github.com/maidsafe/safe_network/commit/c85dc4c7a07d2f5343490328c593cceb0f50c6aa))
    - Fix &Vec -> &[] clippy warning ([`6979475`](https://github.com/maidsafe/safe_network/commit/697947510688c114699b5317f219ef625c29c6d1))
    - Merge #1268 ([`e9adc0d`](https://github.com/maidsafe/safe_network/commit/e9adc0d3ba2f33fe0b4590a5fe11fea56bd4bda9))
</details>

## v0.66.5 (2022-06-24)

<csr-id-d7a831329ad600ad7b5a1b6b68841f96b8ef8cfa/>
<csr-id-1fbc762305a581680b52e2cbdaa7aea2feaf05ab/>
<csr-id-bee6968f85734b2202597d3f8e802eabe8d0c931/>
<csr-id-dc69a62eec590b2d621ab2cbc3009cb052955e66/>

### Chore

 - <csr-id-d7a831329ad600ad7b5a1b6b68841f96b8ef8cfa/> misc cleanup and fixes

### Test

 - <csr-id-bee6968f85734b2202597d3f8e802eabe8d0c931/> make the measurement of client bench test more accurate

### Chore

 - <csr-id-dc69a62eec590b2d621ab2cbc3009cb052955e66/> sn_interface-0.6.5/sn_dysfunction-0.5.3/sn_client-0.66.5/sn_node-0.62.8/sn_cli-0.57.6

### Refactor

 - <csr-id-1fbc762305a581680b52e2cbdaa7aea2feaf05ab/> move it to its own file

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 3 calendar days.
 - 3 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.6.5/sn_dysfunction-0.5.3/sn_client-0.66.5/sn_node-0.62.8/sn_cli-0.57.6 ([`dc69a62`](https://github.com/maidsafe/safe_network/commit/dc69a62eec590b2d621ab2cbc3009cb052955e66))
    - Make the measurement of client bench test more accurate ([`bee6968`](https://github.com/maidsafe/safe_network/commit/bee6968f85734b2202597d3f8e802eabe8d0c931))
    - Merge #1266 ([`366be4d`](https://github.com/maidsafe/safe_network/commit/366be4d3ddc39f32beea0e26d0addd161acc90c2))
    - Chore(misc): misc cleanup and fixes - Complete `msg_kind` => `auth_kind` renaming. - Fix broken `routing_stress` startup. - Clarify context of `HandleTimeout` and `ScheduleTimeout` by inserting `Dkg`. - Tweak `network_split` example. - Set various things, such as payload debug, under `test-utils` flag. - Fix comments/logs: the opposite group of `full` adults are `non-full`, not `empty`. ([`d7a8313`](https://github.com/maidsafe/safe_network/commit/d7a831329ad600ad7b5a1b6b68841f96b8ef8cfa))
    - Merge #1257 #1260 ([`19d89df`](https://github.com/maidsafe/safe_network/commit/19d89dfbbf8ac8ab2b08380ce9b4bed58a5dc0d9))
    - Refactor(msg_type):  move it to its own file - Moves priority fns to service- and system msg. - Moves deserialise of payload to wire_msg fn when getting priority. ([`1fbc762`](https://github.com/maidsafe/safe_network/commit/1fbc762305a581680b52e2cbdaa7aea2feaf05ab))
    - Merge branch 'main' into refactor-event-channel ([`024883e`](https://github.com/maidsafe/safe_network/commit/024883e9a1b853c02c29daa5c447b03570af2473))
</details>

## v0.66.4 (2022-06-21)

<csr-id-d204cffdc25a08f604f3a7b97dd74c0f4181b696/>
<csr-id-c038635cf88d32c52da89d11a8532e6c91c8bf38/>
<csr-id-d526e0a32d3f09a788899d82db4fe6f13258568c/>

### Chore

 - <csr-id-d204cffdc25a08f604f3a7b97dd74c0f4181b696/> remove unused deps and enum variants
 - <csr-id-c038635cf88d32c52da89d11a8532e6c91c8bf38/> misc cleanup

### Chore

 - <csr-id-d526e0a32d3f09a788899d82db4fe6f13258568c/> sn_interface-0.6.4/sn_dysfunction-0.5.2/sn_client-0.66.4/sn_node-0.62.7/sn_api-0.64.4

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 5 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.6.4/sn_dysfunction-0.5.2/sn_client-0.66.4/sn_node-0.62.7/sn_api-0.64.4 ([`d526e0a`](https://github.com/maidsafe/safe_network/commit/d526e0a32d3f09a788899d82db4fe6f13258568c))
    - Chore: remove unused deps and enum variants Was made aware by a comment on the forum that there was a sled dep in `sn_interface`, which seemed wrong, and from there I found more. ([`d204cff`](https://github.com/maidsafe/safe_network/commit/d204cffdc25a08f604f3a7b97dd74c0f4181b696))
    - Chore: misc cleanup - Organise usings - Add missing license headers - Update license years As it would take too long to go through all files, a partial cleanup of the code base is made here. It is based on where the using of `sn_interface` has been introduced, as it was a low hanging fruit to cover many occurrences of duplication in many files. ([`c038635`](https://github.com/maidsafe/safe_network/commit/c038635cf88d32c52da89d11a8532e6c91c8bf38))
</details>

## v0.66.3 (2022-06-15)

<csr-id-4eb43fa884d7b047febb18c067ae905969a113bf/>
<csr-id-f599c5973d50324aad1720166156666d5db1ed3d/>

### Chore

 - <csr-id-4eb43fa884d7b047febb18c067ae905969a113bf/> upgrade blsttc to 6.0.0
   There were various other crates that had to be upgraded in this process:
   * secured_linked_list to v0.5.2 because it was also upgraded to reference v6.0.0 of blsttc
   * bls_dkg to v0.10.3 because it was also upgraded to reference v6.0.0 of blsttc
   * sn_consensus to v2.1.1 because it was also upgraded to reference v6.0.0 of blsttc
   * sn_dbc to v4.0.0 because it was also upgraded to reference v6.0.0 of blsttc

### Chore

 - <csr-id-f599c5973d50324aad1720166156666d5db1ed3d/> sn_interface-0.6.3/sn_dysfunction-0.5.1/sn_client-0.66.3/sn_api-0.64.3/sn_cli-0.57.4

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.6.3/sn_dysfunction-0.5.1/sn_client-0.66.3/sn_api-0.64.3/sn_cli-0.57.4 ([`f599c59`](https://github.com/maidsafe/safe_network/commit/f599c5973d50324aad1720166156666d5db1ed3d))
    - Merge #1241 ([`f9c7544`](https://github.com/maidsafe/safe_network/commit/f9c7544f369e15fb3b6f91158ac3277656737fa4))
    - Upgrade blsttc to 6.0.0 ([`4eb43fa`](https://github.com/maidsafe/safe_network/commit/4eb43fa884d7b047febb18c067ae905969a113bf))
</details>

## v0.66.2 (2022-06-15)

<csr-id-da58fdaa0d6849837e3e473cd7000edb92efe1f0/>
<csr-id-b818c3fd10a4e3304b2c5f84dac843397873cba6/>
<csr-id-46246f155ab65f3fcd61381345f1a7f747dfe957/>

### New Features

 - <csr-id-1b1cb77df6c2805ecfa741bb824b359214558929/> remove private registers
 - <csr-id-f1829f99ef1415a83731f855757fbce9970fa4f0/> remove private data addresses

### Bug Fixes

 - <csr-id-6c52f37592fcda83243390565bd4fdefb821b9b4/> remove unused client_pk from Session::new(..) call
 - <csr-id-fcec8ffaaf7cfb827db5338428b38a7b29cc67af/> add retry loop to wallet tests
 - <csr-id-616d8cb12bfc257f9b3609239790065ebced8fe3/> replace at_least_one_elders with supermajority for sending cmd
 - <csr-id-60f5a68a1df6114b65d7c57099fea0347ba3d1dd/> some changes I missed in the initial private removal
 - <csr-id-7778f992fb9f450addb50daa6edfbddb0502079e/> make dbc reissue working in Windows

### Refactor

 - <csr-id-da58fdaa0d6849837e3e473cd7000edb92efe1f0/> minor refactor in sn_client messaging function

### Chore

 - <csr-id-46246f155ab65f3fcd61381345f1a7f747dfe957/> sn_interface-0.6.2/sn_client-0.66.2/sn_node-0.62.4/sn_api-0.64.2/sn_cli-0.57.3

### Test

 - <csr-id-b818c3fd10a4e3304b2c5f84dac843397873cba6/> cmd sent to all elders

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 14 commits contributed to the release over the course of 2 calendar days.
 - 8 days passed between releases.
 - 10 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.6.2/sn_client-0.66.2/sn_node-0.62.4/sn_api-0.64.2/sn_cli-0.57.3 ([`46246f1`](https://github.com/maidsafe/safe_network/commit/46246f155ab65f3fcd61381345f1a7f747dfe957))
    - Merge #1216 ([`9877101`](https://github.com/maidsafe/safe_network/commit/9877101c74dcf75d78520a804cb6f2b7aaddaffb))
    - Remove unused client_pk from Session::new(..) call ([`6c52f37`](https://github.com/maidsafe/safe_network/commit/6c52f37592fcda83243390565bd4fdefb821b9b4))
    - Merge branch 'main' into simplify_safeurl ([`a0175ab`](https://github.com/maidsafe/safe_network/commit/a0175abfa15e558e54fbb25dc3baf49343f040ac))
    - Merge branch 'main' into drusu/remove-private-data ([`0cd2007`](https://github.com/maidsafe/safe_network/commit/0cd2007e442086d6eb2a39ad1f452e590fad46a9))
    - Merge #1224 ([`2fe452b`](https://github.com/maidsafe/safe_network/commit/2fe452b07d2db0cc622021b76d05605b5d4841c3))
    - Add retry loop to wallet tests ([`fcec8ff`](https://github.com/maidsafe/safe_network/commit/fcec8ffaaf7cfb827db5338428b38a7b29cc67af))
    - Replace at_least_one_elders with supermajority for sending cmd ([`616d8cb`](https://github.com/maidsafe/safe_network/commit/616d8cb12bfc257f9b3609239790065ebced8fe3))
    - Some changes I missed in the initial private removal ([`60f5a68`](https://github.com/maidsafe/safe_network/commit/60f5a68a1df6114b65d7c57099fea0347ba3d1dd))
    - Remove private registers ([`1b1cb77`](https://github.com/maidsafe/safe_network/commit/1b1cb77df6c2805ecfa741bb824b359214558929))
    - Remove private data addresses ([`f1829f9`](https://github.com/maidsafe/safe_network/commit/f1829f99ef1415a83731f855757fbce9970fa4f0))
    - Minor refactor in sn_client messaging function ([`da58fda`](https://github.com/maidsafe/safe_network/commit/da58fdaa0d6849837e3e473cd7000edb92efe1f0))
    - Cmd sent to all elders ([`b818c3f`](https://github.com/maidsafe/safe_network/commit/b818c3fd10a4e3304b2c5f84dac843397873cba6))
    - Make dbc reissue working in Windows ([`7778f99`](https://github.com/maidsafe/safe_network/commit/7778f992fb9f450addb50daa6edfbddb0502079e))
</details>

## v0.66.1 (2022-06-07)

<csr-id-489904e325cfb8efca4289b05125904ad4029f3b/>

### Chore

 - <csr-id-489904e325cfb8efca4289b05125904ad4029f3b/> sn_interface-0.6.1/sn_client-0.66.1/sn_node-0.62.1/sn_api-0.64.1

### New Features

 - <csr-id-dbda86be03f912079776be514828ff5fd034830c/> first version of Spentbook messaging, storage, and client API
   - Storage is implemented using Register as the underlying data type. To be changed when
   actual SpentBook native data type is put in place.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.6.1/sn_client-0.66.1/sn_node-0.62.1/sn_api-0.64.1 ([`489904e`](https://github.com/maidsafe/safe_network/commit/489904e325cfb8efca4289b05125904ad4029f3b))
    - Merge #1214 ([`992c495`](https://github.com/maidsafe/safe_network/commit/992c4951670afc769feea7e6cd38db021aed88a7))
    - First version of Spentbook messaging, storage, and client API ([`dbda86b`](https://github.com/maidsafe/safe_network/commit/dbda86be03f912079776be514828ff5fd034830c))
</details>

## v0.66.0 (2022-06-05)

<csr-id-1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9/>
<csr-id-c12e2269e3a537d96422bed96a4459a0add07deb/>
<csr-id-e548388c693cfb71b270cf9e370b2f9b463044c5/>
<csr-id-210c54e8814877c15d87150248fe3858e83eeee8/>

### Chore

 - <csr-id-1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9/> sn_interface-0.6.0/sn_dysfunction-0.5.0/sn_client-0.66.0/sn_node-0.62.0/sn_api-0.64.0/sn_cli-0.57.0
 - <csr-id-c12e2269e3a537d96422bed96a4459a0add07deb/> upgrade sn_dbc to 3.2.0
   This new release has utilities for serializing/deserializing `Dbc` to/from hex.
 - <csr-id-e548388c693cfb71b270cf9e370b2f9b463044c5/> upgrade sn_dbc to 3.2.0
   This new release has utilities for serializing/deserializing `Dbc` to/from hex.
 - <csr-id-210c54e8814877c15d87150248fe3858e83eeee8/> remove use of test-utils from test runs
   After doing a rebase from main, the test-utils feature was removed. I updated the testing targets
   and also replaced bad references to logger initialisation functions.

### New Features

 - <csr-id-95de2ffe6f57ae0e6cebf123da3e9b6c3ad84aaf/> handover sap elder checks with membership knowledge

### New Features (BREAKING)

 - <csr-id-f03fb7e35319dbb9e4745e3cb36c7913c4f220ac/> cli will now use bls keys
 - <csr-id-48006b73547778bc08b077717e04fd5efb562eaf/> extend client with dbc owner field

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 1 calendar day.
 - 8 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.6.0/sn_dysfunction-0.5.0/sn_client-0.66.0/sn_node-0.62.0/sn_api-0.64.0/sn_cli-0.57.0 ([`1bf7dfb`](https://github.com/maidsafe/safe_network/commit/1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9))
    - Upgrade sn_dbc to 3.2.0 ([`c12e226`](https://github.com/maidsafe/safe_network/commit/c12e2269e3a537d96422bed96a4459a0add07deb))
    - Upgrade sn_dbc to 3.2.0 ([`e548388`](https://github.com/maidsafe/safe_network/commit/e548388c693cfb71b270cf9e370b2f9b463044c5))
    - Handover sap elder checks with membership knowledge ([`95de2ff`](https://github.com/maidsafe/safe_network/commit/95de2ffe6f57ae0e6cebf123da3e9b6c3ad84aaf))
    - Cli will now use bls keys ([`f03fb7e`](https://github.com/maidsafe/safe_network/commit/f03fb7e35319dbb9e4745e3cb36c7913c4f220ac))
    - Remove use of test-utils from test runs ([`210c54e`](https://github.com/maidsafe/safe_network/commit/210c54e8814877c15d87150248fe3858e83eeee8))
    - Extend client with dbc owner field ([`48006b7`](https://github.com/maidsafe/safe_network/commit/48006b73547778bc08b077717e04fd5efb562eaf))
</details>

## v0.65.0 (2022-05-27)

<csr-id-e5fcd032e1dd904e05bc23e119af1d06e3b85a06/>
<csr-id-b9e5db241f437f9bb8fd03ca9080a0331757b9a5/>
<csr-id-f9700e3b6bb8b2b9949f33d627c99974c355ca2b/>
<csr-id-14c92fb0f18fc40176963ca5290914442d340256/>

### Chore

 - <csr-id-e5fcd032e1dd904e05bc23e119af1d06e3b85a06/> sn_interface-0.5.0/sn_dysfunction-0.4.0/sn_client-0.65.0/sn_node-0.61.0/sn_api-0.63.0/sn_cli-0.56.0
 - <csr-id-b9e5db241f437f9bb8fd03ca9080a0331757b9a5/> cleanup of churn test
 - <csr-id-f9700e3b6bb8b2b9949f33d627c99974c355ca2b/> split out dispatcher periodic code
   Just clean up the dispatcher file, splitting
   out all the periodic checks etc into their
   own file, leaving the bones of dispatcher in one
   place

### Other

 - <csr-id-14c92fb0f18fc40176963ca5290914442d340256/> add more intesive churn data integrity test
   The network split test doesnt cover the basic 'new nodes added' membership case. This churn test now does that.

### New Features (BREAKING)

 - <csr-id-294549ebc998d11a2f3621e2a9fd20a0dd9bcce5/> remove sus node flows, replicate data per data

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 2 calendar days.
 - 2 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.5.0/sn_dysfunction-0.4.0/sn_client-0.65.0/sn_node-0.61.0/sn_api-0.63.0/sn_cli-0.56.0 ([`e5fcd03`](https://github.com/maidsafe/safe_network/commit/e5fcd032e1dd904e05bc23e119af1d06e3b85a06))
    - Merge #1208 ([`6c9b851`](https://github.com/maidsafe/safe_network/commit/6c9b851dd5bab8b2f5d9b3ef1db72d198706ac9d))
    - Cleanup of churn test ([`b9e5db2`](https://github.com/maidsafe/safe_network/commit/b9e5db241f437f9bb8fd03ca9080a0331757b9a5))
    - Add more intesive churn data integrity test ([`14c92fb`](https://github.com/maidsafe/safe_network/commit/14c92fb0f18fc40176963ca5290914442d340256))
    - Split out dispatcher periodic code ([`f9700e3`](https://github.com/maidsafe/safe_network/commit/f9700e3b6bb8b2b9949f33d627c99974c355ca2b))
    - Remove sus node flows, replicate data per data ([`294549e`](https://github.com/maidsafe/safe_network/commit/294549ebc998d11a2f3621e2a9fd20a0dd9bcce5))
    - Merge branch 'main' into bump-consensus-2.0.0 ([`a1c592a`](https://github.com/maidsafe/safe_network/commit/a1c592a71247660e7372e019e5f9a6ea23299e0f))
</details>

## v0.64.0 (2022-05-25)

<csr-id-ef56cf9cf8de45a9f13c2510c63de245b12aeae8/>

### Chore

 - <csr-id-ef56cf9cf8de45a9f13c2510c63de245b12aeae8/> sn_interface-0.4.0/sn_dysfunction-0.3.0/sn_client-0.64.0/sn_node-0.60.0/sn_api-0.62.0/sn_cli-0.55.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 3 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.4.0/sn_dysfunction-0.3.0/sn_client-0.64.0/sn_node-0.60.0/sn_api-0.62.0/sn_cli-0.55.0 ([`ef56cf9`](https://github.com/maidsafe/safe_network/commit/ef56cf9cf8de45a9f13c2510c63de245b12aeae8))
</details>

## v0.63.0 (2022-05-21)

<csr-id-cf21d66b9b726123e0a4320cd68481b67f7af03d/>
<csr-id-8e2731d8b7923a9050451b31ef3a92f892d2d6d3/>
<csr-id-f2742d92b3c3b56ed80732aa1d6943885fcd4317/>

### Chore

 - <csr-id-cf21d66b9b726123e0a4320cd68481b67f7af03d/> sn_dysfunction-0.2.0/sn_client-0.63.0/sn_node-0.59.0/sn_api-0.61.0/sn_cli-0.54.0

### Refactor

 - <csr-id-8e2731d8b7923a9050451b31ef3a92f892d2d6d3/> de-dupe init_test_logger
 - <csr-id-f2742d92b3c3b56ed80732aa1d6943885fcd4317/> cargo test works without feature flag

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 1 calendar day.
 - 3 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_dysfunction-0.2.0/sn_client-0.63.0/sn_node-0.59.0/sn_api-0.61.0/sn_cli-0.54.0 ([`cf21d66`](https://github.com/maidsafe/safe_network/commit/cf21d66b9b726123e0a4320cd68481b67f7af03d))
    - De-dupe init_test_logger ([`8e2731d`](https://github.com/maidsafe/safe_network/commit/8e2731d8b7923a9050451b31ef3a92f892d2d6d3))
    - Cargo test works without feature flag ([`f2742d9`](https://github.com/maidsafe/safe_network/commit/f2742d92b3c3b56ed80732aa1d6943885fcd4317))
</details>

## v0.62.3 (2022-05-18)

<csr-id-07504faeda6cbfd0b27abea25facde992398ecf9/>
<csr-id-fb7ec7f4b90678cc0693d311a1f2efd87a6714a6/>
<csr-id-9b06304f46e1a1bda90a0fc6ff82edc928c2529d/>

### Chore

 - <csr-id-07504faeda6cbfd0b27abea25facde992398ecf9/> upgrade blsttc to v5.2.0 and rand to v0.8
 - <csr-id-fb7ec7f4b90678cc0693d311a1f2efd87a6714a6/> send some msgs in bg threads
   This should unblock client threads on initial contact and on queries

### Chore

 - <csr-id-9b06304f46e1a1bda90a0fc6ff82edc928c2529d/> sn_interface-0.2.4/sn_client-0.62.3/sn_api-0.60.2/sn_cli-0.53.1

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 7 calendar days.
 - 8 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.2.4/sn_client-0.62.3/sn_api-0.60.2/sn_cli-0.53.1 ([`9b06304`](https://github.com/maidsafe/safe_network/commit/9b06304f46e1a1bda90a0fc6ff82edc928c2529d))
    - Merge #1190 ([`8833cb8`](https://github.com/maidsafe/safe_network/commit/8833cb8a4ae13f04ea86c67e92fce4d82a107f5a))
    - Upgrade blsttc to v5.2.0 and rand to v0.8 ([`07504fa`](https://github.com/maidsafe/safe_network/commit/07504faeda6cbfd0b27abea25facde992398ecf9))
    - Merge #1189 ([`00f41b4`](https://github.com/maidsafe/safe_network/commit/00f41b4a96bcc172d91620aa0da0cb799db5483c))
    - Send some msgs in bg threads ([`fb7ec7f`](https://github.com/maidsafe/safe_network/commit/fb7ec7f4b90678cc0693d311a1f2efd87a6714a6))
    - Merge branch 'main' into sap_sig_checks ([`f8ec2e5`](https://github.com/maidsafe/safe_network/commit/f8ec2e54943eaa18b50bd9d7562d41f57d5d3248))
</details>

## v0.62.2 (2022-05-10)

<csr-id-61ba367c308a846cb3f1ae065b1fbbdfb85838e4/>

### Chore

 - <csr-id-61ba367c308a846cb3f1ae065b1fbbdfb85838e4/> sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1

### Bug Fixes

 - <csr-id-ae4156228a4bb684ff10ac8c98917dd4dae434ea/> check Register permissions on ops locally to prevent failures when broadcasted to the network

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 2 calendar days.
 - 3 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1 ([`61ba367`](https://github.com/maidsafe/safe_network/commit/61ba367c308a846cb3f1ae065b1fbbdfb85838e4))
    - Merge branch 'main' into nightly-improvements ([`ee3bbe1`](https://github.com/maidsafe/safe_network/commit/ee3bbe188cea756384dc38d490fe58c59c050292))
    - Merge #1172 ([`837c44c`](https://github.com/maidsafe/safe_network/commit/837c44cda38c2757f689cc4db4a84fa7c02091c0))
    - Check Register permissions on ops locally to prevent failures when broadcasted to the network ([`ae41562`](https://github.com/maidsafe/safe_network/commit/ae4156228a4bb684ff10ac8c98917dd4dae434ea))
    - Merge branch 'main' into main ([`d3f07bb`](https://github.com/maidsafe/safe_network/commit/d3f07bbe5192174082e24869ba86125b6a7b1b20))
    - Merge branch 'main' into retry-count-input ([`925a8a4`](https://github.com/maidsafe/safe_network/commit/925a8a4aaade025433c29028229947de28fcb214))
</details>

## v0.62.1 (2022-05-06)

<csr-id-9fc497a3c27f2545c9dc2a8106e31feeb497ef3a/>
<csr-id-c2f5f855191fa46d549adea15e9123674c24d44a/>
<csr-id-e8f4fbca2acb81b3ddc1b275140e5f7b1b56f9a9/>
<csr-id-800008d2ec43d6df3bc078c59b7ae405610e5539/>
<csr-id-7766e7d20b392cf5b8563d1dbc9560254b44e756/>
<csr-id-2f4e7e6305ba387f2e28945aee71df650ac1d3eb/>
<csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/>
<csr-id-e3dca8b07441c86744b091fe883d16a9c750f702/>
<csr-id-ad7d340720f0737f502b0d55023a15461dded91d/>
<csr-id-8d041a80b75bc773fcbe0e4c88940ade9bda4b9d/>
<csr-id-2a731b990dbe67a700468865288585ee8dff0d71/>
<csr-id-88421d9cb7872b6397283a0035130bc14de6d4ff/>
<csr-id-6383f038449ebba5e7c5dec1d3f8cc1f7deca581/>
<csr-id-975520e1abf6056bd50cc29ca5a569015b3a77e4/>
<csr-id-fec4741438b8de957b5de94e21b78cf15886713f/>
<csr-id-a05599e452dc7400e83e7a048488689db2c28e9e/>
<csr-id-54000b43cdd3688e6c691bef9dedc299da3c22aa/>
<csr-id-737d906a61f772593ac7df755d995d66059e8b5e/>

### Chore

 - <csr-id-9fc497a3c27f2545c9dc2a8106e31feeb497ef3a/> reduce number of query attempts from client once again
 - <csr-id-c2f5f855191fa46d549adea15e9123674c24d44a/> remove concept of 'sufficent' knowledge
   Previously we waited on 7 elders at least... but we should just trust saps provided
   they are valid. So here we remove this check
 - <csr-id-e8f4fbca2acb81b3ddc1b275140e5f7b1b56f9a9/> make client targets relative to sap size
   The proivided sap could be different from expected, but
   we should be able to trust if if it's valid... As such
   we base target counts off of the provided SAP
 - <csr-id-800008d2ec43d6df3bc078c59b7ae405610e5539/> more logging for insufficent elders
 - <csr-id-7766e7d20b392cf5b8563d1dbc9560254b44e756/> rename MsgKind -> AuthKind
   This feels more correct given that the kind is actually about the authority that
   the message carries.
 - <csr-id-2f4e7e6305ba387f2e28945aee71df650ac1d3eb/> sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0
 - <csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/> tidy references in cargo manifests
   All references are organised alphabetically, and random usage of long-form references are removed in
   favour of the short-form version, unless the long-form style is justified, e.g., when lots of
   features are being used.
 - <csr-id-e3dca8b07441c86744b091fe883d16a9c750f702/> set sn_client version to be new release
   previously sn_client was its own repo and crate, we havent published under this name in some time. This will bring us back into this namespace ad on crates.io, but at a new updated version
 - <csr-id-ad7d340720f0737f502b0d55023a15461dded91d/> update sn_cli and api readme for sn_client extraction
 - <csr-id-8d041a80b75bc773fcbe0e4c88940ade9bda4b9d/> remove unused deps after node/client split
 - <csr-id-2a731b990dbe67a700468865288585ee8dff0d71/> move examples/bench -> sn_client where appropriate
 - <csr-id-88421d9cb7872b6397283a0035130bc14de6d4ff/> pull sn_client out of the node codebase
 - <csr-id-6383f038449ebba5e7c5dec1d3f8cc1f7deca581/> remove olde node github workflows

### Chore

 - <csr-id-737d906a61f772593ac7df755d995d66059e8b5e/> sn_interface-0.2.1/sn_client-0.62.1/sn_node-0.58.15/sn_api-0.60.0/sn_cli-0.53.0

### Bug Fixes

 - <csr-id-9f4c3a523212c41079afcde8052a0891f3895f3b/> client knowledge could not update
   adds network knowledge storage to clients.
   Previously we were seeing issues where knowledge could not be
   updated after receiving one of two sibling saps after split.
   
   now we store the whole knowledge and validate against this chain
 - <csr-id-ae4156228a4bb684ff10ac8c98917dd4dae434ea/> check Register permissions on ops locally to prevent failures when broadcasted to the network

### Other

 - <csr-id-975520e1abf6056bd50cc29ca5a569015b3a77e4/> delay removing .safe folder on hosted runners
   this should hopefully occasional avoid cleanup errors
 - <csr-id-fec4741438b8de957b5de94e21b78cf15886713f/> use Flat sampling in criterion upload tests
   Criterion auto sampling is designed for tests in the pico/nano sec
   range. Flat sampling for for longer running tests like ours.
 - <csr-id-a05599e452dc7400e83e7a048488689db2c28e9e/> use Flat sampling in criterion upload tests
   Criterion auto sampling is designed for tests in the pico/nano sec
   range. Flat sampling for for longer running tests like ours.
 - <csr-id-54000b43cdd3688e6c691bef9dedc299da3c22aa/> test updates for sn_node and sn_client

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 21 commits contributed to the release over the course of 326 calendar days.
 - 19 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.2.1/sn_client-0.62.1/sn_node-0.58.15/sn_api-0.60.0/sn_cli-0.53.0 ([`737d906`](https://github.com/maidsafe/safe_network/commit/737d906a61f772593ac7df755d995d66059e8b5e))
    - Merge #1160 ([`d46e85b`](https://github.com/maidsafe/safe_network/commit/d46e85bf508be983017b90e6ce18f588039b16ac))
    - Reduce number of query attempts from client once again ([`9fc497a`](https://github.com/maidsafe/safe_network/commit/9fc497a3c27f2545c9dc2a8106e31feeb497ef3a))
    - Client knowledge could not update ([`9f4c3a5`](https://github.com/maidsafe/safe_network/commit/9f4c3a523212c41079afcde8052a0891f3895f3b))
    - Remove concept of 'sufficent' knowledge ([`c2f5f85`](https://github.com/maidsafe/safe_network/commit/c2f5f855191fa46d549adea15e9123674c24d44a))
    - Make client targets relative to sap size ([`e8f4fbc`](https://github.com/maidsafe/safe_network/commit/e8f4fbca2acb81b3ddc1b275140e5f7b1b56f9a9))
    - More logging for insufficent elders ([`800008d`](https://github.com/maidsafe/safe_network/commit/800008d2ec43d6df3bc078c59b7ae405610e5539))
    - Delay removing .safe folder on hosted runners ([`975520e`](https://github.com/maidsafe/safe_network/commit/975520e1abf6056bd50cc29ca5a569015b3a77e4))
    - Merge #1139 ([`22abbc7`](https://github.com/maidsafe/safe_network/commit/22abbc73f909131a0208ddc6e9471d073061134a))
    - Rename MsgKind -> AuthKind ([`7766e7d`](https://github.com/maidsafe/safe_network/commit/7766e7d20b392cf5b8563d1dbc9560254b44e756))
    - Sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0 ([`2f4e7e6`](https://github.com/maidsafe/safe_network/commit/2f4e7e6305ba387f2e28945aee71df650ac1d3eb))
    - Tidy references in cargo manifests ([`318ee1d`](https://github.com/maidsafe/safe_network/commit/318ee1d22970b5f06e93a99b6e8fff6da638c589))
    - Use Flat sampling in criterion upload tests ([`a05599e`](https://github.com/maidsafe/safe_network/commit/a05599e452dc7400e83e7a048488689db2c28e9e))
    - Use Flat sampling in criterion upload tests ([`fec4741`](https://github.com/maidsafe/safe_network/commit/fec4741438b8de957b5de94e21b78cf15886713f))
    - Set sn_client version to be new release ([`e3dca8b`](https://github.com/maidsafe/safe_network/commit/e3dca8b07441c86744b091fe883d16a9c750f702))
    - Update sn_cli and api readme for sn_client extraction ([`ad7d340`](https://github.com/maidsafe/safe_network/commit/ad7d340720f0737f502b0d55023a15461dded91d))
    - Remove unused deps after node/client split ([`8d041a8`](https://github.com/maidsafe/safe_network/commit/8d041a80b75bc773fcbe0e4c88940ade9bda4b9d))
    - Move examples/bench -> sn_client where appropriate ([`2a731b9`](https://github.com/maidsafe/safe_network/commit/2a731b990dbe67a700468865288585ee8dff0d71))
    - Test updates for sn_node and sn_client ([`54000b4`](https://github.com/maidsafe/safe_network/commit/54000b43cdd3688e6c691bef9dedc299da3c22aa))
    - Pull sn_client out of the node codebase ([`88421d9`](https://github.com/maidsafe/safe_network/commit/88421d9cb7872b6397283a0035130bc14de6d4ff))
    - Remove olde node github workflows ([`6383f03`](https://github.com/maidsafe/safe_network/commit/6383f038449ebba5e7c5dec1d3f8cc1f7deca581))
</details>

## v0.62.0 (2022-04-23)

<csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/>
<csr-id-e3dca8b07441c86744b091fe883d16a9c750f702/>
<csr-id-ad7d340720f0737f502b0d55023a15461dded91d/>
<csr-id-8d041a80b75bc773fcbe0e4c88940ade9bda4b9d/>
<csr-id-2a731b990dbe67a700468865288585ee8dff0d71/>
<csr-id-88421d9cb7872b6397283a0035130bc14de6d4ff/>
<csr-id-6383f038449ebba5e7c5dec1d3f8cc1f7deca581/>
<csr-id-fec4741438b8de957b5de94e21b78cf15886713f/>
<csr-id-a05599e452dc7400e83e7a048488689db2c28e9e/>
<csr-id-54000b43cdd3688e6c691bef9dedc299da3c22aa/>

### Chore

 - <csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/> tidy references in cargo manifests
   All references are organised alphabetically, and random usage of long-form references are removed in
   favour of the short-form version, unless the long-form style is justified, e.g., when lots of
   features are being used.
 - <csr-id-e3dca8b07441c86744b091fe883d16a9c750f702/> set sn_client version to be new release
   previously sn_client was its own repo and crate, we havent published under this name in some time. This will bring us back into this namespace ad on crates.io, but at a new updated version
 - <csr-id-ad7d340720f0737f502b0d55023a15461dded91d/> update sn_cli and api readme for sn_client extraction
 - <csr-id-8d041a80b75bc773fcbe0e4c88940ade9bda4b9d/> remove unused deps after node/client split
 - <csr-id-2a731b990dbe67a700468865288585ee8dff0d71/> move examples/bench -> sn_client where appropriate
 - <csr-id-88421d9cb7872b6397283a0035130bc14de6d4ff/> pull sn_client out of the node codebase
 - <csr-id-6383f038449ebba5e7c5dec1d3f8cc1f7deca581/> remove olde node github workflows

### Other

 - <csr-id-fec4741438b8de957b5de94e21b78cf15886713f/> use Flat sampling in criterion upload tests
   Criterion auto sampling is designed for tests in the pico/nano sec
   range. Flat sampling for for longer running tests like ours.
 - <csr-id-a05599e452dc7400e83e7a048488689db2c28e9e/> use Flat sampling in criterion upload tests
   Criterion auto sampling is designed for tests in the pico/nano sec
   range. Flat sampling for for longer running tests like ours.
 - <csr-id-54000b43cdd3688e6c691bef9dedc299da3c22aa/> test updates for sn_node and sn_client

