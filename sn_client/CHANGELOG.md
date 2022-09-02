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

## v0.72.0 (2022-09-02)

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 4 calendar days.
 - 4 days passed between releases.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - unneeded iter methods removal ([`9214386`](https://github.com/maidsafe/safe_network/commit/921438659ccaf65b2ea8cc00efb61d8146ef71ef))
    - applied use_self lint ([`f5d436f`](https://github.com/maidsafe/safe_network/commit/f5d436fba99e0e9c258c7ab3c3a256be3be58f84))
    - chore(clippy) ([`d9ee11d`](https://github.com/maidsafe/safe_network/commit/d9ee11d228f8ac9f2d6cd3d09f1a1e29276100d1))
    - change throughput measure ([`388cd22`](https://github.com/maidsafe/safe_network/commit/388cd223677ecfa2e790c54c0df8ecb18c77299c))
    - adds missing bench to cargo.toml for client ([`5c54848`](https://github.com/maidsafe/safe_network/commit/5c54848b726188f273ffa16ee2870976914bb815))
    - toml ([`4aa9b02`](https://github.com/maidsafe/safe_network/commit/4aa9b02f375a30132712ca97306e5f2e9a8d36f7))
    - move benches that dont need the network ahead of network launch ([`a4e84ef`](https://github.com/maidsafe/safe_network/commit/a4e84ef4608a13ecc2f14dd87f5c23d185185513))
    - add msg serialization benchmark ([`d251dbe`](https://github.com/maidsafe/safe_network/commit/d251dbeb2e44707623c3bbb1215784b1bd4fae06))
    - sn_interface lints and fixes ([`b040ea1`](https://github.com/maidsafe/safe_network/commit/b040ea14e53247094838de6f1fa9af2830b051fa))
    - switch on clippy::unwrap_used as a warning ([`3a718d8`](https://github.com/maidsafe/safe_network/commit/3a718d8c0957957a75250b044c9d1ad1b5874ab0))
</details>

## v0.71.1 (2022-08-28)

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

 - 3 commits contributed to the release over the course of 2 calendar days.
 - 3 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.10.2/sn_client-0.71.1/sn_node-0.66.2/sn_cli-0.62.1 ([`2b26820`](https://github.com/maidsafe/safe_network/commit/2b268209e6910472558145a5d08b99e968550221))
    - implement `SecuredLinkedList` as a `MerkleRegister` ([`7cc2a00`](https://github.com/maidsafe/safe_network/commit/7cc2a00907381e93db266f31545b12ff76907e5d))
    - return error to client on unknown section key ([`b87617e`](https://github.com/maidsafe/safe_network/commit/b87617e44e9b20b8a79864e30e29ecee86444352))
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
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.10.1/sn_client-0.71.0/sn_node-0.66.1/sn_api-0.69.0/sn_cli-0.62.0 ([`401bc41`](https://github.com/maidsafe/safe_network/commit/401bc416c7aea65ae55e9adee2cbecf782c999cf))
    - Decode ReplicatedDataAddress from chunk filename ([`604556e`](https://github.com/maidsafe/safe_network/commit/604556e670d5fe0a9408bbd0d586363c7b4c0d6c))
    - churn example, wait longer before we attempt to read post-churn ([`a46ac6e`](https://github.com/maidsafe/safe_network/commit/a46ac6e18bbdfdb331caf89f8bb562a7c762b64b))
    - expose MAX_RETRIES for cmd/query ops in client builder ([`a8b3cd8`](https://github.com/maidsafe/safe_network/commit/a8b3cd855393d06a64734b34523e40ec00fb0580))
    - further reduce query retries and query timeout ([`9fbb067`](https://github.com/maidsafe/safe_network/commit/9fbb0672735306336f5020794a638f79752f0577))
    - reduce query timeout noe we;re faster in general ([`f40277c`](https://github.com/maidsafe/safe_network/commit/f40277c1680f56b043c4865ff201c65b66926b2d))
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
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.10.0/sn_dysfunction-0.9.0/sn_client-0.70.0/sn_node-0.66.0/sn_api-0.68.0/sn_cli-0.61.0 ([`43fcc7c`](https://github.com/maidsafe/safe_network/commit/43fcc7c517f95eab0e27ddc79cd9c6de3631c7c6))
    - removing unused CreateRegister::Populated msg type ([`28d95a2`](https://github.com/maidsafe/safe_network/commit/28d95a2e959e32ee69a70bdc855cba1fff1fc8d8))
    - removing unused sn_node::dbs::Error variants and RegisterExtend cmd ([`d3f66d6`](https://github.com/maidsafe/safe_network/commit/d3f66d6cfa838a5c65fb8f31fa68d48794b33dea))
    - adding more context information to sn_client::Error types ([`991ccd4`](https://github.com/maidsafe/safe_network/commit/991ccd452119137d9da046b7f222f091177e28f1))
    - move data replication steps ahead of elder check in FlowCtrl ([`dfed2a8`](https://github.com/maidsafe/safe_network/commit/dfed2a8d2751b6627250b64e7a78213b68ec6733))
    - add logs and tweaks to churn example ([`3b06876`](https://github.com/maidsafe/safe_network/commit/3b068764721cd74f4d52a279a606743415abff02))
    - reintroduce Arc<RwLock> for section tree ([`43ecab2`](https://github.com/maidsafe/safe_network/commit/43ecab2dda52cb0ede7c0d4b6e48eaffe1fb6b75))
    - associated functions to methods ([`2f8f8ca`](https://github.com/maidsafe/safe_network/commit/2f8f8ca6ba0f2faae5bb4631c708988edf907725))
    - upgrading sn_dbc to v8.0 ([`589f03c`](https://github.com/maidsafe/safe_network/commit/589f03ce8670544285f329fe35c19897d4bfced8))
    - renaming NetworkPrefixMap to SectionTree ([`f0fbe5f`](https://github.com/maidsafe/safe_network/commit/f0fbe5fd9bec0b2865271bb139c9fcb4ec225884))
    - expose serialisation/deserialisation utilities as public methods instead ([`1618cf6`](https://github.com/maidsafe/safe_network/commit/1618cf6a93117942946d152efee24fe3c7020e55))
    - remove long lived client conn test ([`06f5b60`](https://github.com/maidsafe/safe_network/commit/06f5b607cdfbacba082612965630249e3c0f7300))
    - reduce client qp2p default idle timeout ([`6155ad0`](https://github.com/maidsafe/safe_network/commit/6155ad0334104d367638373fbcbbd7e21631b3e6))
    - clean up unused functionality ([`11b8182`](https://github.com/maidsafe/safe_network/commit/11b8182a3de636a760d899cb15d7184d8153545a))
    - leave out unnecessary Arc<RwLock> ([`ddbbb53`](https://github.com/maidsafe/safe_network/commit/ddbbb53d61d6c94b00a47dc2b708a2aeda870d96))
    - remove unused Session member ([`1235f7d`](https://github.com/maidsafe/safe_network/commit/1235f7d8a92eb9f086c35696bf5c0a8baf67f2ac))
    - retry more times for connection fails w/ client ([`6471eb8`](https://github.com/maidsafe/safe_network/commit/6471eb88f7ce8c060909930ac23c855f30e8690a))
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

 - 68 commits contributed to the release over the course of 32 calendar days.
 - 34 days passed between releases.
 - 65 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.9.0/sn_dysfunction-0.8.0/sn_client-0.69.0/sn_node-0.65.0/sn_api-0.67.0/sn_cli-0.60.0 ([`53f60c2`](https://github.com/maidsafe/safe_network/commit/53f60c2327f8a69f0b2ef6d1a4e96644c10aa358))
    - follow rust convention for getters for prefixmap ([`707df06`](https://github.com/maidsafe/safe_network/commit/707df06b08d5b0457b201ce5772d6a1d4fe9f984))
    - sn_client to only read a default prefix map file, updates to be cached on disk by user ([`27ba2a6`](https://github.com/maidsafe/safe_network/commit/27ba2a63dcfa272cf7ef8c5301987fc6bfe18ed0))
    - remove wiremsg.priority as uneeded ([`6d60525`](https://github.com/maidsafe/safe_network/commit/6d60525874dc4efeb658433f1f253d54e0cba2d4))
    - have many-clients test to report the errors found when instantiating clients ([`95c33d1`](https://github.com/maidsafe/safe_network/commit/95c33d1ea2040bce4078be96ed8b1c9f2e966b21))
    - misc. fixes ([`42bde15`](https://github.com/maidsafe/safe_network/commit/42bde15e9a96dbe759575d4bccf4f769e13a695d))
    - add client builder code example ([`753443d`](https://github.com/maidsafe/safe_network/commit/753443da697a61e49eac977402731c4373e7f4f9))
    - remove client config in favour of builder ([`40a5f2d`](https://github.com/maidsafe/safe_network/commit/40a5f2d968aff30e7b92fe325aa299deddb49e69))
    - convert nodes joining interval to millis before passing it to launch-tool ([`0041e18`](https://github.com/maidsafe/safe_network/commit/0041e18ab7d1a21e4debb39df9c4b116e002a5e5))
    - serialize NetworkPrefixMap into JSON ([`29de67f`](https://github.com/maidsafe/safe_network/commit/29de67f1e3583eab867d517cb50ed2e404bd63fd))
    - nodes to cache their own individual prefix map file on disk ([`96da117`](https://github.com/maidsafe/safe_network/commit/96da1171d0cac240f772e5d6a15c56f63441b4b3))
    - reduce AE msgs to one msg with a kind field ([`6b1fee8`](https://github.com/maidsafe/safe_network/commit/6b1fee8cf3d0b2995f4b81e59dd684547593b5fa))
    - removing Token from sn_interfaces::type as it is now exposed by sn_dbc ([`dd2eb21`](https://github.com/maidsafe/safe_network/commit/dd2eb21352223f6340064e0021f4a7df402cd5c9))
    - organise usings, cleanup ([`8242f2f`](https://github.com/maidsafe/safe_network/commit/8242f2f1035b1c0718e53954951badffa30f3393))
    - remove NetworkPrefxiMap::genesis_key, NetworkKnowledge::genesis_key ([`820fcc9`](https://github.com/maidsafe/safe_network/commit/820fcc9a77f756fca308f247c3ea1b82f65d30b9))
    - fix deadlock introduced after removal of Arc from NetworkPrefixMap ([`0ed5075`](https://github.com/maidsafe/safe_network/commit/0ed5075304b090597f7760fb51c4a33435a853f1))
    - remove RwLock from NetworkPrefixMap ([`afcf083`](https://github.com/maidsafe/safe_network/commit/afcf083469c732f10c7c80f4a45e4c33ab111101))
    - make NetowrkPrefixMap::sections_dag private ([`17f0e8a`](https://github.com/maidsafe/safe_network/commit/17f0e8a08c9543d380c16a35d3d7bfe7834a9e5a))
    - move SectionChain into NetworkPrefixMap ([`ed37bb5`](https://github.com/maidsafe/safe_network/commit/ed37bb56e5e17d4cba7c1b2165746c193241d618))
    - rename traceroute fns ([`aafc560`](https://github.com/maidsafe/safe_network/commit/aafc560d3b3b1e375f7be224e0e63a3b567bbd86))
    - make traceroute a default feature ([`73dc9b4`](https://github.com/maidsafe/safe_network/commit/73dc9b4a1757393270e62d265328bab0c0aa3b35))
    - improve Display, Debug impl for Traceroute ([`0a653e4`](https://github.com/maidsafe/safe_network/commit/0a653e4becc4a8e14ffd6d0752cf035430067ce9))
    - improve traceroute readability and other improvements ([`9789797`](https://github.com/maidsafe/safe_network/commit/9789797e3f773285f23bd22957fe45a67aabec24))
    - use builder to instantiate ([`14ea6c7`](https://github.com/maidsafe/safe_network/commit/14ea6c7f4bbaee9c2ac4a30fba938ef2de2f77e5))
    - adjust client instantiation ([`923cdfd`](https://github.com/maidsafe/safe_network/commit/923cdfded98132e94473db04e01d5fe83f73ca3d))
    - return reference instead of clone ([`f666204`](https://github.com/maidsafe/safe_network/commit/f666204febb1044980412345236ce0cb8377b162))
    - remove TODOs from public docs ([`70ea782`](https://github.com/maidsafe/safe_network/commit/70ea78222875eb947e684af6db7544927f0bfe77))
    - remove unused error variants ([`c3196bf`](https://github.com/maidsafe/safe_network/commit/c3196bfdbca221dfa61f978331582fc7d6db72d3))
    - remove unused config dir/file ([`3fc072f`](https://github.com/maidsafe/safe_network/commit/3fc072f256dfe4b9e1a1a09c59800c7d78aa7360))
    - fix link to config field ([`49313f6`](https://github.com/maidsafe/safe_network/commit/49313f62b5a174a9b278c1c5d18baccdf4bb8c85))
    - remove unused channel from Client ([`947b6ca`](https://github.com/maidsafe/safe_network/commit/947b6cad014a41b0336de7f1c31f9902473c1a70))
    - remove providing path to qp2p cfg ([`db7dcdc`](https://github.com/maidsafe/safe_network/commit/db7dcdc7968d1d7e946274650d5a0c48719b4955))
    - ClientBuilder to instantiate Client ([`4772ff1`](https://github.com/maidsafe/safe_network/commit/4772ff129bd8da82465ef93e66d17a8fbbd38f7d))
    - remove more unused code ([`db4f4d0`](https://github.com/maidsafe/safe_network/commit/db4f4d07b155d732ad76d263563d81b5fee535f7))
    - Merge #1427 ([`949ee11`](https://github.com/maidsafe/safe_network/commit/949ee111717c8f07487f3f4db6fbc0043583916d))
    - more attempt when query too close to the spend cmd ([`f0ad7d5`](https://github.com/maidsafe/safe_network/commit/f0ad7d56a58a08a7591d978c8ead4c10db734276))
    - use sn_dbc::SpentProof API for verifying SpentProofShares ([`e0fb940`](https://github.com/maidsafe/safe_network/commit/e0fb940b24e87d86fe920095176362f73503ce79))
    - remove unused storage path ([`ca32230`](https://github.com/maidsafe/safe_network/commit/ca32230926e5a435d90694df8fbce1218ea397f0))
    - Revert "feat: make traceroute default for now" ([`e9b97c7`](https://github.com/maidsafe/safe_network/commit/e9b97c72b860053285ba866b098937f6b25d99bf))
    - having spent proofs and Txs within SpentbookCmd::Send msg to be a set instead of a vec ([`d3a05a7`](https://github.com/maidsafe/safe_network/commit/d3a05a728be8752ea9ebff4e38e7c4c85e5db09b))
    - make traceroute default for now ([`175d1b9`](https://github.com/maidsafe/safe_network/commit/175d1b909dff8c6729ac7f156ce1d0d22be8cc12))
    - Cleanup non-joined member sessions, regardless of connected state ([`8efbd96`](https://github.com/maidsafe/safe_network/commit/8efbd96a5fd3907ace5ca6ac282027595fefd8ef))
    - reflect the semantics not the type ([`ea490dd`](https://github.com/maidsafe/safe_network/commit/ea490ddf749ac9e0c7962c3c21c053663e6b6ee7))
    - rename gen_section_authority_provider to random_sap ([`3f577d2`](https://github.com/maidsafe/safe_network/commit/3f577d2a6fe70792d7d02e231b599ca3d44a5ed2))
    - upgrade blsttc to 7.0.0 ([`6f03b93`](https://github.com/maidsafe/safe_network/commit/6f03b93bd2d02f0ffe54b69fbf25070fbe64eab0))
    - increase query retry count as we may drop connections now ([`1ee345c`](https://github.com/maidsafe/safe_network/commit/1ee345ce00337f9b24d45db417b6bb3d54c67955))
    - impl traceroute for client cmds and cmd responses ([`4f2cf26`](https://github.com/maidsafe/safe_network/commit/4f2cf267ee030e5924a2fa999a2a46dbc072d208))
    - improve traceroute redability and resolve clippy ([`214aded`](https://github.com/maidsafe/safe_network/commit/214adedc31bca576c7f28ff52a1f4ff0a2676757))
    - impl traceroute feature to trace a message's flow in the network ([`a6fb1fc`](https://github.com/maidsafe/safe_network/commit/a6fb1fc516a9ef6dae7aa236f3dd440d50697ae2))
    - additional tests in sn-api for DBC verification failures ([`d4be0cc`](https://github.com/maidsafe/safe_network/commit/d4be0cc431947b035046cc4d56642a81c0880924))
    - reissuing DBCs for all sn_cli tests only once as a setup stage ([`9fde534`](https://github.com/maidsafe/safe_network/commit/9fde534277f359dfa0a1d91d917864776edb5138))
    - perform verification of input TX and spentproofs when depositing or reissuing a DBC ([`ba97ca0`](https://github.com/maidsafe/safe_network/commit/ba97ca06b67cd6e5de8e1c910b396fbe44f40fd7))
    - Merge #1356 ([`d9b4608`](https://github.com/maidsafe/safe_network/commit/d9b46080ac849cac259983dc80b4b879e58c13ba))
    - remove unused console-subscriber ([`39c3fdf`](https://github.com/maidsafe/safe_network/commit/39c3fdf4128462e5f7c5fec3c628d394f505e2f2))
    - setup step for tests to reissue a set of DBCs from genesis only once ([`5c82df6`](https://github.com/maidsafe/safe_network/commit/5c82df633e7c062fdf761a8e6e0a7ae8d26cc73a))
    - re-enable registers benchmark and tidy sled residue ([`1e8180c`](https://github.com/maidsafe/safe_network/commit/1e8180c23fab27ac92c93f201efd050cff00db10))
    - remove extra bootstrap step ([`ec8b69d`](https://github.com/maidsafe/safe_network/commit/ec8b69d642fc4ca0166ffff113306244e5c3936a))
    - use Eyre instead of boxed error ([`db52519`](https://github.com/maidsafe/safe_network/commit/db525193bed7662c5184810f18587abb0d22b26b))
    - add timeout for queries ([`df5ea26`](https://github.com/maidsafe/safe_network/commit/df5ea26c8243de70d16a75ac936bc322954c8436))
    - add binary to query chunk at adults ([`b9bfb42`](https://github.com/maidsafe/safe_network/commit/b9bfb425035fead587b7b5fc03a212a5d5aae4b3))
    - formatting with cargo fmt ([`00fae4d`](https://github.com/maidsafe/safe_network/commit/00fae4d5fd5dbad5696888f0c796fbd39b7e49ed))
    - unused async in sn_client ([`145b302`](https://github.com/maidsafe/safe_network/commit/145b302aad291120c52f1cffad8e7d116682f532))
    - unused async remove and up-chain ([`d8cc453`](https://github.com/maidsafe/safe_network/commit/d8cc45384f891a9d95a7cef30159f11ec0ff9269))
    - Remove unused Arc(RwLock) structure ([`a378e7b`](https://github.com/maidsafe/safe_network/commit/a378e7ba67ec18be708a2e1a9e08e63519da7451))
    - removing hard-coded test DBC from sn_api Wallet unit tests ([`f5af444`](https://github.com/maidsafe/safe_network/commit/f5af444b8ac37d2debfbe5e1d4dcdc48de963694))
    - upon receiving an AE msg update client knowledge of network sections chains ([`950b304`](https://github.com/maidsafe/safe_network/commit/950b3048d1aae1f9ad5d2218a42c34d662925e38))
    - tweak sn_client/Cargo.toml formatting TOML ([`4d717a2`](https://github.com/maidsafe/safe_network/commit/4d717a21a2daf6ef0b3b5826329a8848f2fe46ee))
    - move to dev-dependencies ([`5aeb15c`](https://github.com/maidsafe/safe_network/commit/5aeb15c8c309c16878dde510f68b0e5c2122cd8c))
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
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.2/sn_client-0.68.2/sn_node-0.64.2/sn_api-0.66.3/sn_cli-0.59.3 ([`34bd9bd`](https://github.com/maidsafe/safe_network/commit/34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8))
    - remove unused utilities ([`19ddeba`](https://github.com/maidsafe/safe_network/commit/19ddebad43aa53822cb7e781913ba34b848e2c89))
    - move more deps to clap-v3; rm some deps on rand ([`49e223e`](https://github.com/maidsafe/safe_network/commit/49e223e2c07695b4c63e253ba19ce43ec24d7112))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`45418f2`](https://github.com/maidsafe/safe_network/commit/45418f2f9b5cc58f2a153bf40966beb2bf36a62a))
    - ignore store_and_read_40mb as too heavy for CI ([`5cff2c5`](https://github.com/maidsafe/safe_network/commit/5cff2c5325a854f04788f9111439bca75b21c60f))
    - for QueryResponse, set correlation_id to be the origin msg_id ([`64eb333`](https://github.com/maidsafe/safe_network/commit/64eb333d532694f46f1d0b9dd5109961b3551802))
    - passing the churn test ([`3c383cc`](https://github.com/maidsafe/safe_network/commit/3c383ccf9ad0ed77080fb3e3ec459e5b02158505))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`94be181`](https://github.com/maidsafe/safe_network/commit/94be181789b0010f83ed5e89341f3f347575e37f))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`44411d5`](https://github.com/maidsafe/safe_network/commit/44411d511a496b13893670c8bc7d9f43f0ce9073))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`45309c4`](https://github.com/maidsafe/safe_network/commit/45309c4c0463dd9198a49537187417bf4bfdb847))
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

 - 15 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 9 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.1/sn_dysfunction-0.7.1/sn_client-0.68.1/sn_node-0.64.1/sn_api-0.66.1/sn_cli-0.59.1 ([`2b00cec`](https://github.com/maidsafe/safe_network/commit/2b00cec961561281f6b927e13e501342843f6a0f))
    - Merge #1315 ([`67686f7`](https://github.com/maidsafe/safe_network/commit/67686f73f9e7b18bb6fbf1eadc3fd3a256285396))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`f83724c`](https://github.com/maidsafe/safe_network/commit/f83724cff1e63b35f1612fc82dffdefbeaab6cc1))
    - Merge #1313 ([`7fe7be3`](https://github.com/maidsafe/safe_network/commit/7fe7be336799dec811c5b17e6d753ebe31e625f1))
    - Merge branch 'main' into cargo-husky-tweaks ([`6881855`](https://github.com/maidsafe/safe_network/commit/688185573bb71cc44a7103df17f3fbeea6740247))
    - adapt client_api spentbook test to read genesis DBC from first node in testnet ([`90059dc`](https://github.com/maidsafe/safe_network/commit/90059dc4edf35fc7d53bc25b485be291a1de9807))
    - Merge #1309 ([`f9fa4f7`](https://github.com/maidsafe/safe_network/commit/f9fa4f7857d8161e8c036cca06006bf187a6c6c3))
    - Merge branch 'main' into cargo-husky-tweaks ([`52dd02e`](https://github.com/maidsafe/safe_network/commit/52dd02e45ab4e160b0a26498919a79ce1aefb1bd))
    - now we only contact one adult at a time increase retry count ([`77cef49`](https://github.com/maidsafe/safe_network/commit/77cef496695e8cac9ccefccaf99cf350fb479eb9))
    - use latest sn_launch_tool release, sans StructOpt ([`da13669`](https://github.com/maidsafe/safe_network/commit/da13669193d93b3a56fff4a956c9ac9830055a7a))
    - replace StructOpt with Clap in sn_client ([`85ca7ce`](https://github.com/maidsafe/safe_network/commit/85ca7ce23414bf19e72236e32745b0fb6239664d))
    - `try!` macro is deprecated ([`4626226`](https://github.com/maidsafe/safe_network/commit/46262268fc167c05963e5b7bd6261310496e2379))
    - clippy runs cargo check already ([`8dccb7f`](https://github.com/maidsafe/safe_network/commit/8dccb7f1fc81385f9f5f25e6c354ad1d35759528))
    - churn example tweaks ([`2d0e23c`](https://github.com/maidsafe/safe_network/commit/2d0e23cdc7f94b0cc2d13ddf8203702cec4d3a07))
    - reduce client waiting time on receiving responses ([`1e3f865`](https://github.com/maidsafe/safe_network/commit/1e3f865ae5e32520958bb071bc7fbffe8b79a033))
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
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.0/sn_dysfunction-0.7.0/sn_client-0.68.0/sn_node-0.64.0/sn_api-0.66.0/sn_cli-0.59.0 ([`e4e2eb5`](https://github.com/maidsafe/safe_network/commit/e4e2eb56611a328806c59ed8bc80ca2567206bbb))
    - remove NetworkInfo::GenesisKey variant ([`6a2553a`](https://github.com/maidsafe/safe_network/commit/6a2553a11b1404ad404e67df29bf3ec535d1b954))
    - remove NodeConfig from sn_api::ipc, add sn_cli tests ([`5dbf50d`](https://github.com/maidsafe/safe_network/commit/5dbf50d92bf7e93acbb00e85f51910f32ac4a124))
    - update PrefixMap symlink if incorrect ([`849dfba`](https://github.com/maidsafe/safe_network/commit/849dfba283362d8fbdddd92be1078c3a963fb564))
    - remove node_connection_info.config from sn_node, sn_interface, sn_client ([`91da4d4`](https://github.com/maidsafe/safe_network/commit/91da4d4ac7aab039853b0651e5aafd9cdd31b9c4))
    - Docs - put symbols in backticks ([`9314a2d`](https://github.com/maidsafe/safe_network/commit/9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7))
    - clippy clea up unused ([`d4850ff`](https://github.com/maidsafe/safe_network/commit/d4850ff81d33751ebf9e3a7c7af438f160df6e44))
    - remove let bindings for unit returns ([`ddb7798`](https://github.com/maidsafe/safe_network/commit/ddb7798a7b0c5e60960e123414277d58f3da27eb))
    - have the nodes to attach valid Commitments to signed SpentProofShares ([`5dad80d`](https://github.com/maidsafe/safe_network/commit/5dad80d3f239f5844243fedb89f8d4baaee3b640))
    - remove unused asyncs (clippy) ([`4e04a2b`](https://github.com/maidsafe/safe_network/commit/4e04a2b0acc79140bf1d0aefd82c0ad5b046a3cd))
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
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.7.1/sn_dysfunction-0.6.1/sn_client-0.67.1/sn_node-0.63.1 ([`58890e5`](https://github.com/maidsafe/safe_network/commit/58890e5c919ada30f27d4e80c6b5e7291b99ed5c))
    - use node's section_key and own key for register ([`44b93fd`](https://github.com/maidsafe/safe_network/commit/44b93fde435214b363c009e555a2579bb3404e75))
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
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.7.0/sn_dysfunction-0.6.0/sn_client-0.67.0/sn_node-0.63.0/sn_api-0.65.0/sn_cli-0.58.0 ([`243cfc4`](https://github.com/maidsafe/safe_network/commit/243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e))
    - more tweaks to benchmarks for clippy ([`c85dc4c`](https://github.com/maidsafe/safe_network/commit/c85dc4c7a07d2f5343490328c593cceb0f50c6aa))
    - fix &Vec -> &[] clippy warning ([`6979475`](https://github.com/maidsafe/safe_network/commit/697947510688c114699b5317f219ef625c29c6d1))
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

 - 6 commits contributed to the release over the course of 2 calendar days.
 - 3 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.5/sn_dysfunction-0.5.3/sn_client-0.66.5/sn_node-0.62.8/sn_cli-0.57.6 ([`dc69a62`](https://github.com/maidsafe/safe_network/commit/dc69a62eec590b2d621ab2cbc3009cb052955e66))
    - make the measurement of client bench test more accurate ([`bee6968`](https://github.com/maidsafe/safe_network/commit/bee6968f85734b2202597d3f8e802eabe8d0c931))
    - misc cleanup and fixes ([`d7a8313`](https://github.com/maidsafe/safe_network/commit/d7a831329ad600ad7b5a1b6b68841f96b8ef8cfa))
    - Merge #1255 #1258 ([`ed0b5d8`](https://github.com/maidsafe/safe_network/commit/ed0b5d890e8404a59c25f8131eab5d23ce12eb7d))
    - Merge #1257 #1260 ([`19d89df`](https://github.com/maidsafe/safe_network/commit/19d89dfbbf8ac8ab2b08380ce9b4bed58a5dc0d9))
    - move it to its own file ([`1fbc762`](https://github.com/maidsafe/safe_network/commit/1fbc762305a581680b52e2cbdaa7aea2feaf05ab))
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
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.4/sn_dysfunction-0.5.2/sn_client-0.66.4/sn_node-0.62.7/sn_api-0.64.4 ([`d526e0a`](https://github.com/maidsafe/safe_network/commit/d526e0a32d3f09a788899d82db4fe6f13258568c))
    - remove unused deps and enum variants ([`d204cff`](https://github.com/maidsafe/safe_network/commit/d204cffdc25a08f604f3a7b97dd74c0f4181b696))
    - misc cleanup ([`c038635`](https://github.com/maidsafe/safe_network/commit/c038635cf88d32c52da89d11a8532e6c91c8bf38))
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

 - 2 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.3/sn_dysfunction-0.5.1/sn_client-0.66.3/sn_api-0.64.3/sn_cli-0.57.4 ([`f599c59`](https://github.com/maidsafe/safe_network/commit/f599c5973d50324aad1720166156666d5db1ed3d))
    - upgrade blsttc to 6.0.0 ([`4eb43fa`](https://github.com/maidsafe/safe_network/commit/4eb43fa884d7b047febb18c067ae905969a113bf))
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

 - 12 commits contributed to the release over the course of 2 calendar days.
 - 8 days passed between releases.
 - 10 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.2/sn_client-0.66.2/sn_node-0.62.4/sn_api-0.64.2/sn_cli-0.57.3 ([`46246f1`](https://github.com/maidsafe/safe_network/commit/46246f155ab65f3fcd61381345f1a7f747dfe957))
    - Merge remote-tracking branch 'origin/main' into drusu/remove-private-data ([`2057273`](https://github.com/maidsafe/safe_network/commit/2057273509c2488cafc7f6db2ae69a99efc3b350))
    - remove unused client_pk from Session::new(..) call ([`6c52f37`](https://github.com/maidsafe/safe_network/commit/6c52f37592fcda83243390565bd4fdefb821b9b4))
    - Merge branch 'main' into drusu/remove-private-data ([`0cd2007`](https://github.com/maidsafe/safe_network/commit/0cd2007e442086d6eb2a39ad1f452e590fad46a9))
    - add retry loop to wallet tests ([`fcec8ff`](https://github.com/maidsafe/safe_network/commit/fcec8ffaaf7cfb827db5338428b38a7b29cc67af))
    - replace at_least_one_elders with supermajority for sending cmd ([`616d8cb`](https://github.com/maidsafe/safe_network/commit/616d8cb12bfc257f9b3609239790065ebced8fe3))
    - some changes I missed in the initial private removal ([`60f5a68`](https://github.com/maidsafe/safe_network/commit/60f5a68a1df6114b65d7c57099fea0347ba3d1dd))
    - remove private registers ([`1b1cb77`](https://github.com/maidsafe/safe_network/commit/1b1cb77df6c2805ecfa741bb824b359214558929))
    - remove private data addresses ([`f1829f9`](https://github.com/maidsafe/safe_network/commit/f1829f99ef1415a83731f855757fbce9970fa4f0))
    - minor refactor in sn_client messaging function ([`da58fda`](https://github.com/maidsafe/safe_network/commit/da58fdaa0d6849837e3e473cd7000edb92efe1f0))
    - cmd sent to all elders ([`b818c3f`](https://github.com/maidsafe/safe_network/commit/b818c3fd10a4e3304b2c5f84dac843397873cba6))
    - make dbc reissue working in Windows ([`7778f99`](https://github.com/maidsafe/safe_network/commit/7778f992fb9f450addb50daa6edfbddb0502079e))
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

 - 4 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.1/sn_client-0.66.1/sn_node-0.62.1/sn_api-0.64.1 ([`489904e`](https://github.com/maidsafe/safe_network/commit/489904e325cfb8efca4289b05125904ad4029f3b))
    - Merge branch 'main' into Gabriel_Spentbook_PR1143 ([`0eda02a`](https://github.com/maidsafe/safe_network/commit/0eda02ac126be4f088af6bf9e7247c8496a389ba))
    - first version of Spentbook messaging, storage, and client API ([`dbda86b`](https://github.com/maidsafe/safe_network/commit/dbda86be03f912079776be514828ff5fd034830c))
    - Merge #1217 ([`2f26043`](https://github.com/maidsafe/safe_network/commit/2f2604325d533357bad7d917315cf4cba0b2d3c0))
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

 - 8 commits contributed to the release over the course of 4 calendar days.
 - 8 days passed between releases.
 - 7 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.0/sn_dysfunction-0.5.0/sn_client-0.66.0/sn_node-0.62.0/sn_api-0.64.0/sn_cli-0.57.0 ([`1bf7dfb`](https://github.com/maidsafe/safe_network/commit/1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9))
    - upgrade sn_dbc to 3.2.0 ([`c12e226`](https://github.com/maidsafe/safe_network/commit/c12e2269e3a537d96422bed96a4459a0add07deb))
    - upgrade sn_dbc to 3.2.0 ([`e548388`](https://github.com/maidsafe/safe_network/commit/e548388c693cfb71b270cf9e370b2f9b463044c5))
    - handover sap elder checks with membership knowledge ([`95de2ff`](https://github.com/maidsafe/safe_network/commit/95de2ffe6f57ae0e6cebf123da3e9b6c3ad84aaf))
    - cli will now use bls keys ([`f03fb7e`](https://github.com/maidsafe/safe_network/commit/f03fb7e35319dbb9e4745e3cb36c7913c4f220ac))
    - remove use of test-utils from test runs ([`210c54e`](https://github.com/maidsafe/safe_network/commit/210c54e8814877c15d87150248fe3858e83eeee8))
    - extend client with dbc owner field ([`48006b7`](https://github.com/maidsafe/safe_network/commit/48006b73547778bc08b077717e04fd5efb562eaf))
    - Merge #1192 ([`f9fc2a7`](https://github.com/maidsafe/safe_network/commit/f9fc2a76f083ba5161c8c4eef9013c53586b4693))
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

 - 6 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.5.0/sn_dysfunction-0.4.0/sn_client-0.65.0/sn_node-0.61.0/sn_api-0.63.0/sn_cli-0.56.0 ([`e5fcd03`](https://github.com/maidsafe/safe_network/commit/e5fcd032e1dd904e05bc23e119af1d06e3b85a06))
    - Merge #1202 ([`e42a2e3`](https://github.com/maidsafe/safe_network/commit/e42a2e3c212597e68238451a5bb4a8725c4761be))
    - cleanup of churn test ([`b9e5db2`](https://github.com/maidsafe/safe_network/commit/b9e5db241f437f9bb8fd03ca9080a0331757b9a5))
    - add more intesive churn data integrity test ([`14c92fb`](https://github.com/maidsafe/safe_network/commit/14c92fb0f18fc40176963ca5290914442d340256))
    - split out dispatcher periodic code ([`f9700e3`](https://github.com/maidsafe/safe_network/commit/f9700e3b6bb8b2b9949f33d627c99974c355ca2b))
    - remove sus node flows, replicate data per data ([`294549e`](https://github.com/maidsafe/safe_network/commit/294549ebc998d11a2f3621e2a9fd20a0dd9bcce5))
</details>

## v0.64.0 (2022-05-25)

<csr-id-ef56cf9cf8de45a9f13c2510c63de245b12aeae8/>

### Chore

 - <csr-id-ef56cf9cf8de45a9f13c2510c63de245b12aeae8/> sn_interface-0.4.0/sn_dysfunction-0.3.0/sn_client-0.64.0/sn_node-0.60.0/sn_api-0.62.0/sn_cli-0.55.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 3 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.4.0/sn_dysfunction-0.3.0/sn_client-0.64.0/sn_node-0.60.0/sn_api-0.62.0/sn_cli-0.55.0 ([`ef56cf9`](https://github.com/maidsafe/safe_network/commit/ef56cf9cf8de45a9f13c2510c63de245b12aeae8))
    - Merge #1195 ([`c6e6e32`](https://github.com/maidsafe/safe_network/commit/c6e6e324164028c6c15a78643783a9f86679f39e))
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
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_dysfunction-0.2.0/sn_client-0.63.0/sn_node-0.59.0/sn_api-0.61.0/sn_cli-0.54.0 ([`cf21d66`](https://github.com/maidsafe/safe_network/commit/cf21d66b9b726123e0a4320cd68481b67f7af03d))
    - de-dupe init_test_logger ([`8e2731d`](https://github.com/maidsafe/safe_network/commit/8e2731d8b7923a9050451b31ef3a92f892d2d6d3))
    - cargo test works without feature flag ([`f2742d9`](https://github.com/maidsafe/safe_network/commit/f2742d92b3c3b56ed80732aa1d6943885fcd4317))
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

 - 3 commits contributed to the release.
 - 8 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.4/sn_client-0.62.3/sn_api-0.60.2/sn_cli-0.53.1 ([`9b06304`](https://github.com/maidsafe/safe_network/commit/9b06304f46e1a1bda90a0fc6ff82edc928c2529d))
    - upgrade blsttc to v5.2.0 and rand to v0.8 ([`07504fa`](https://github.com/maidsafe/safe_network/commit/07504faeda6cbfd0b27abea25facde992398ecf9))
    - send some msgs in bg threads ([`fb7ec7f`](https://github.com/maidsafe/safe_network/commit/fb7ec7f4b90678cc0693d311a1f2efd87a6714a6))
</details>

## v0.62.2 (2022-05-10)

<csr-id-61ba367c308a846cb3f1ae065b1fbbdfb85838e4/>

### Chore

 - <csr-id-61ba367c308a846cb3f1ae065b1fbbdfb85838e4/> sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1

### Bug Fixes

 - <csr-id-ae4156228a4bb684ff10ac8c98917dd4dae434ea/> check Register permissions on ops locally to prevent failures when broadcasted to the network

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 3 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1 ([`61ba367`](https://github.com/maidsafe/safe_network/commit/61ba367c308a846cb3f1ae065b1fbbdfb85838e4))
    - Merge #1171 ([`06b4433`](https://github.com/maidsafe/safe_network/commit/06b4433f199ba7c622ad57e767d80f58f0b50a69))
    - check Register permissions on ops locally to prevent failures when broadcasted to the network ([`ae41562`](https://github.com/maidsafe/safe_network/commit/ae4156228a4bb684ff10ac8c98917dd4dae434ea))
    - Merge #1140 ([`459b641`](https://github.com/maidsafe/safe_network/commit/459b641f22b488f33825777b974da80512eabed5))
    - Merge #1169 ([`e5d0c17`](https://github.com/maidsafe/safe_network/commit/e5d0c17c335a3a25ee0bb4c81906fa176abeb7f5))
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

 - 22 commits contributed to the release over the course of 326 calendar days.
 - 19 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.1/sn_client-0.62.1/sn_node-0.58.15/sn_api-0.60.0/sn_cli-0.53.0 ([`737d906`](https://github.com/maidsafe/safe_network/commit/737d906a61f772593ac7df755d995d66059e8b5e))
    - reduce number of query attempts from client once again ([`9fc497a`](https://github.com/maidsafe/safe_network/commit/9fc497a3c27f2545c9dc2a8106e31feeb497ef3a))
    - client knowledge could not update ([`9f4c3a5`](https://github.com/maidsafe/safe_network/commit/9f4c3a523212c41079afcde8052a0891f3895f3b))
    - remove concept of 'sufficent' knowledge ([`c2f5f85`](https://github.com/maidsafe/safe_network/commit/c2f5f855191fa46d549adea15e9123674c24d44a))
    - make client targets relative to sap size ([`e8f4fbc`](https://github.com/maidsafe/safe_network/commit/e8f4fbca2acb81b3ddc1b275140e5f7b1b56f9a9))
    - more logging for insufficent elders ([`800008d`](https://github.com/maidsafe/safe_network/commit/800008d2ec43d6df3bc078c59b7ae405610e5539))
    - delay removing .safe folder on hosted runners ([`975520e`](https://github.com/maidsafe/safe_network/commit/975520e1abf6056bd50cc29ca5a569015b3a77e4))
    - Merge branch 'main' into Feat-InterfaceAuthKind ([`df40fb9`](https://github.com/maidsafe/safe_network/commit/df40fb94f6847b31aec730eb7cbc6c0b97fe9a0e))
    - Merge branch 'main' into Feat-InterfaceAuthKind ([`5db6533`](https://github.com/maidsafe/safe_network/commit/5db6533b2151e2377299a0be11e513210adfabd4))
    - rename MsgKind -> AuthKind ([`7766e7d`](https://github.com/maidsafe/safe_network/commit/7766e7d20b392cf5b8563d1dbc9560254b44e756))
    - Merge #1128 ([`e49d382`](https://github.com/maidsafe/safe_network/commit/e49d38239b3a8c468616ad3782e1208316e9b5e0))
    - sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0 ([`2f4e7e6`](https://github.com/maidsafe/safe_network/commit/2f4e7e6305ba387f2e28945aee71df650ac1d3eb))
    - tidy references in cargo manifests ([`318ee1d`](https://github.com/maidsafe/safe_network/commit/318ee1d22970b5f06e93a99b6e8fff6da638c589))
    - use Flat sampling in criterion upload tests ([`a05599e`](https://github.com/maidsafe/safe_network/commit/a05599e452dc7400e83e7a048488689db2c28e9e))
    - use Flat sampling in criterion upload tests ([`fec4741`](https://github.com/maidsafe/safe_network/commit/fec4741438b8de957b5de94e21b78cf15886713f))
    - set sn_client version to be new release ([`e3dca8b`](https://github.com/maidsafe/safe_network/commit/e3dca8b07441c86744b091fe883d16a9c750f702))
    - update sn_cli and api readme for sn_client extraction ([`ad7d340`](https://github.com/maidsafe/safe_network/commit/ad7d340720f0737f502b0d55023a15461dded91d))
    - remove unused deps after node/client split ([`8d041a8`](https://github.com/maidsafe/safe_network/commit/8d041a80b75bc773fcbe0e4c88940ade9bda4b9d))
    - move examples/bench -> sn_client where appropriate ([`2a731b9`](https://github.com/maidsafe/safe_network/commit/2a731b990dbe67a700468865288585ee8dff0d71))
    - test updates for sn_node and sn_client ([`54000b4`](https://github.com/maidsafe/safe_network/commit/54000b43cdd3688e6c691bef9dedc299da3c22aa))
    - pull sn_client out of the node codebase ([`88421d9`](https://github.com/maidsafe/safe_network/commit/88421d9cb7872b6397283a0035130bc14de6d4ff))
    - remove olde node github workflows ([`6383f03`](https://github.com/maidsafe/safe_network/commit/6383f038449ebba5e7c5dec1d3f8cc1f7deca581))
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

