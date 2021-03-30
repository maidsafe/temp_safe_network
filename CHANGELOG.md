# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

## [11.0.0](https://github.com/maidsafe/sn_messaging/compare/v10.0.0...v11.0.0) (2021-03-30)


### ⚠ BREAKING CHANGES

* **deps:** Changes to node reward messages

### Bug Fixes

* clarify api ([6db2924](https://github.com/maidsafe/sn_messaging/commit/6db29249540c3329a7e92c2f3187933ffa5f7159))


* **deps:** update sn_data_types ([2597df9](https://github.com/maidsafe/sn_messaging/commit/2597df926eef60298314d79764f5d0153c4225b6))

## [10.0.0](https://github.com/maidsafe/sn_messaging/compare/v9.0.0...v10.0.0) (2021-03-26)


### ⚠ BREAKING CHANGES

* Split message type into two, for passing process errors
or messages to be processed.

### Features

* add id func ([2a6ccea](https://github.com/maidsafe/sn_messaging/commit/2a6ccead51550424ea6ac9db6227dddfd9f9ebb7))
* add processing error reasons ([7455746](https://github.com/maidsafe/sn_messaging/commit/7455746f827b4cfd4bfa9fc34a4af46872e44688))
* expose processing error ([2237743](https://github.com/maidsafe/sn_messaging/commit/22377431752bb2430fcd57845c4d1e87526bbba8))
* functional msg id ([fd4062a](https://github.com/maidsafe/sn_messaging/commit/fd4062ab750f3c7ec49281ced38f5d548308b4ad))
* initial LazyError proposal ([2a971aa](https://github.com/maidsafe/sn_messaging/commit/2a971aab884a8ed4e8538262a92892f3fc5b6634))
* make headerinfo fields pub ([a868218](https://github.com/maidsafe/sn_messaging/commit/a86821861d58dba2838dc66197a99c15faef4d96))
* make source message optional ([943048b](https://github.com/maidsafe/sn_messaging/commit/943048baa6438339ed962b1e4885259fd3485b92))
* remove processErrorReason, just use standard error message ([72283d8](https://github.com/maidsafe/sn_messaging/commit/72283d86eb7703e0e8faa1f97d90a97c81cdf2ae))
* **GetSectionQuery:** use PK instead of Xor so we have a response pk ([49f29fb](https://github.com/maidsafe/sn_messaging/commit/49f29fb3a6709a902cbb9c6c2e23d2c29dd5f036))
* **Redirect:** provide elder name too ([7129f9d](https://github.com/maidsafe/sn_messaging/commit/7129f9d5e5f01235e42b87e0e15652241f7ae1c7))
* **serialisation:** add destination XorName and destination section public key to WireMsgHeader ([7c65ff1](https://github.com/maidsafe/sn_messaging/commit/7c65ff1eacf98d97ee6cebc1e7796b5981c54e36))
* **trait:** derive Clone Trait for multiple types ([eea57c2](https://github.com/maidsafe/sn_messaging/commit/eea57c276622b31e055fcfaaeaabd76f199d9d3c))
* one message to rule them all ([8cb9c49](https://github.com/maidsafe/sn_messaging/commit/8cb9c49405782b9bd313ea98435799f31bd445f2))


### Bug Fixes

* expose create_processing_error_msg ([0070d89](https://github.com/maidsafe/sn_messaging/commit/0070d899cee49a18be5477f2585a0283d8c02c08))
* missing export of NodeSystemQueryResponse ([1883557](https://github.com/maidsafe/sn_messaging/commit/1883557ff72acc72eeaf9c33425d7be722f4f08e))

## [9.0.0](https://github.com/maidsafe/sn_messaging/compare/v8.0.0...v9.0.0) (2021-03-22)


### ⚠ BREAKING CHANGES

* DT update. Naming and message structs for split
* SetupSections as opposed to GetSectionWallet, allows passing of sibling PK for proper setup of both section wallets

### Features

* Remove GetReplicaKeys trasnfer query ([c4c7a2a](https://github.com/maidsafe/sn_messaging/commit/c4c7a2a019ac9478f2a81d513117c1a21308d7f1))
* **cmds:** add CreateSectionWallet cmd ([8afb2cf](https://github.com/maidsafe/sn_messaging/commit/8afb2cf72252b2c737a695628920d725e12ce468))
* **msg_id:** generate from content ([386c092](https://github.com/maidsafe/sn_messaging/commit/386c0925c5ce974b8b08a634f7a98be6e03c297e))
* **msgs:** rename query ([9bb508b](https://github.com/maidsafe/sn_messaging/commit/9bb508b7b6f760cc214300450a6fda04d4d33528))
* **node:** add PromotedToElder event ([010bad2](https://github.com/maidsafe/sn_messaging/commit/010bad2595a9054c934f2464064b06cf2a654c13))
* **systemcmd:** extend with wallet proposals ([553adaa](https://github.com/maidsafe/sn_messaging/commit/553adaa67e54130964e04d8e845f38cbcaa60dfc))
* **transfers:** propagate the credit proof only ([059eb74](https://github.com/maidsafe/sn_messaging/commit/059eb7427e99a4aadd50129c81b355757a41fb1d))
* updates to message naming + sibling key passing ([05b0a32](https://github.com/maidsafe/sn_messaging/commit/05b0a32c2f9ef045f2c75e28af670899795c569d))
* updates to message naming, and removing sibling key passing ([75c9b0b](https://github.com/maidsafe/sn_messaging/commit/75c9b0b57708ef68667c5119029c297ae065f4d8))


### Bug Fixes

* remove leftover GetReplicaKeys refs ([ecde8bc](https://github.com/maidsafe/sn_messaging/commit/ecde8bc2d2c1b078579026f01d472feb5f32fe36))
* remove redundant origin field ([21bf1cb](https://github.com/maidsafe/sn_messaging/commit/21bf1cb88531f5268a0808ae558fb0609aa784e2))


* DT dep update ([b82b223](https://github.com/maidsafe/sn_messaging/commit/b82b2237fe67cc72d294d94f9cb61f31c2ee6ef6))

## [8.0.0](https://github.com/maidsafe/sn_messaging/compare/v7.0.1...v8.0.0) (2021-03-03)


### ⚠ BREAKING CHANGES

* **Seq:** Policy mutation operations are removed.

### Features

* **Seq:** upgrading sn_data_types to v0.16.0 and removing operations that are meant for mutating Seuquence's Policy ([306d8c1](https://github.com/maidsafe/sn_messaging/commit/306d8c16ea627f2aaed597d8c0df3698ab7d3a3e))

### [7.0.1](https://github.com/maidsafe/sn_messaging/compare/v7.0.0...v7.0.1) (2021-03-03)

## [7.0.0](https://github.com/maidsafe/sn_messaging/compare/v6.0.2...v7.0.0) (2021-02-26)


### ⚠ BREAKING CHANGES

* **api:** location scheme updated, breaking the current messaging api

### Features

* **api:** api updated ([4e11d0e](https://github.com/maidsafe/sn_messaging/commit/4e11d0ecf10eb1b9c5ead1ac5be0de1e079bff05))

### [6.0.2](https://github.com/maidsafe/sn_messaging/compare/v6.0.1...v6.0.2) (2021-02-26)

### [6.0.1](https://github.com/maidsafe/sn_messaging/compare/v6.0.0...v6.0.1) (2021-02-24)

## [6.0.0](https://github.com/maidsafe/sn_messaging/compare/v5.0.0...v6.0.0) (2021-02-24)


### ⚠ BREAKING CHANGES

* **location:** this adds a new variant to SrcLocation

### Features

* **location:** add support for accumulation at destination ([89cadad](https://github.com/maidsafe/sn_messaging/commit/89cadad9669295f2833f0a161acd252d04e4218a))

## [5.0.0](https://github.com/maidsafe/sn_messaging/compare/v4.0.5...v5.0.0) (2021-02-22)


### ⚠ BREAKING CHANGES

* **deps:** remove msgenvelope, change infrastructure msg

### Features

* **enduser:** add bootstrap msg variants ([129924e](https://github.com/maidsafe/sn_messaging/commit/129924e03eb020881322b1ce3d5412de70c02172))
* **enduser:** replace socketaddr with a hash ([45ac67f](https://github.com/maidsafe/sn_messaging/commit/45ac67f80dd010c3536a4632dcbf952d0f01a007))
* **messages:** implement location ([cf37569](https://github.com/maidsafe/sn_messaging/commit/cf37569d55515d35e5652c2c06f9ac3e8b3b7dbc))
* **messages:** remove MsgEnvelope ([d54b6c4](https://github.com/maidsafe/sn_messaging/commit/d54b6c42d119221f066d24109805b0995caf224b))


### Bug Fixes

* add string payload to invalid bootstrap error ([23ed16c](https://github.com/maidsafe/sn_messaging/commit/23ed16c2dd68f75c2554f7cc4d54c6fd6a9c7522))
* improve debug msg ([17daa0b](https://github.com/maidsafe/sn_messaging/commit/17daa0bca15a1d48a8869642969eca213196312e))
* post-rebase issues ([93578c7](https://github.com/maidsafe/sn_messaging/commit/93578c7b60fe8ec30dfbfd8d4371e8566284a9c1))


* **deps:** update sn_data_types ([555e4fb](https://github.com/maidsafe/sn_messaging/commit/555e4fbb3222ba0a46fd189c9c62bfd8052d9d19))

### [4.0.5](https://github.com/maidsafe/sn_messaging/compare/v4.0.4...v4.0.5) (2021-02-22)

### [4.0.4](https://github.com/maidsafe/sn_messaging/compare/v4.0.3...v4.0.4) (2021-02-18)

### [4.0.3](https://github.com/maidsafe/sn_messaging/compare/v4.0.2...v4.0.3) (2021-02-17)

### [4.0.2](https://github.com/maidsafe/sn_messaging/compare/v4.0.1...v4.0.2) (2021-02-16)

### [4.0.1](https://github.com/maidsafe/sn_messaging/compare/v4.0.0...v4.0.1) (2021-02-16)

## [4.0.0](https://github.com/maidsafe/sn_messaging/compare/v3.0.2...v4.0.0) (2021-02-15)


### ⚠ BREAKING CHANGES

* adds more infrastructure information to bootstrap and on section key errors
* Adds pk to messages and helper on MsgEnvelope

### Features

* add infrastructure information. ([9ca78b7](https://github.com/maidsafe/sn_messaging/commit/9ca78b78a8acf0cc3f6d9b9195a1483c66934d49))
* error messages related to target pk ([08d31d3](https://github.com/maidsafe/sn_messaging/commit/08d31d3f694bf92562499a498bc0b7dd903ff61c))
* make infra error its own type, use that in client::Error. ([122bc07](https://github.com/maidsafe/sn_messaging/commit/122bc0755078602a65275d4b7ccf2e8d759c8ef9))
* require a section key PK to be passed with all messages ([60f5240](https://github.com/maidsafe/sn_messaging/commit/60f5240ac8242d04e970773cdccfcb7ccd4a9e3e))

### [3.0.2](https://github.com/maidsafe/sn_messaging/compare/v3.0.1...v3.0.2) (2021-02-08)

### [3.0.1](https://github.com/maidsafe/sn_messaging/compare/v3.0.0...v3.0.1) (2021-02-08)

## [3.0.0](https://github.com/maidsafe/sn_messaging/compare/v2.0.0...v3.0.0) (2021-02-03)


### ⚠ BREAKING CHANGES

* **types:** moving client messages to its own module and publis namespace.

### Features

* **types:** adding Ping, NodeMessage and InfrastructureQuery definitions and support in serialisation ([dcd6b32](https://github.com/maidsafe/sn_messaging/commit/dcd6b321154714000d67c38137d1155433c4672a))

## [2.0.0](https://github.com/maidsafe/sn_messaging/compare/v1.6.1...v2.0.0) (2021-02-01)


### ⚠ BREAKING CHANGES

* rename money to token

* rename money to token ([eb53ef5](https://github.com/maidsafe/sn_messaging/commit/eb53ef577da48c9850e8997fcb91ebc6ae9fefd2))

### [1.6.1](https://github.com/maidsafe/sn_messaging/compare/v1.6.0...v1.6.1) (2021-02-01)

## [1.6.0](https://github.com/maidsafe/sn_messaging/compare/v1.5.5...v1.6.0) (2021-01-29)


### Features

* **genesis:** add msgs for genesis section init ([a808d3f](https://github.com/maidsafe/sn_messaging/commit/a808d3fbcf1ab10c8b21876ab177d97ffab47abc))
* **node_transfers:** add section payout error ([86114a5](https://github.com/maidsafe/sn_messaging/commit/86114a53593786ced19def470ddf262821d927ba))
* **nodeevents:** add SectionPayoutRegistered ([b782d19](https://github.com/maidsafe/sn_messaging/commit/b782d19cfa94a2d8b76cde714c3102dcfc9dc944))
* **rewards:** use share for payout validation ([041330b](https://github.com/maidsafe/sn_messaging/commit/041330bec25561e350a4fe28cc36cba4eb5f4d51))

### [1.5.5](https://github.com/maidsafe/sn_messaging/compare/v1.5.4...v1.5.5) (2021-01-15)

### [1.5.4](https://github.com/maidsafe/sn_messaging/compare/v1.5.3...v1.5.4) (2021-01-15)

### [1.5.3](https://github.com/maidsafe/sn_messaging/compare/v1.5.2...v1.5.3) (2021-01-13)

### [1.5.2](https://github.com/maidsafe/sn_messaging/compare/v1.5.1...v1.5.2) (2021-01-13)

### [1.5.1](https://github.com/maidsafe/sn_messaging/compare/v1.5.0...v1.5.1) (2021-01-13)


### Bug Fixes

* **verification:** verification of message_env checks underlying message, not itself ([ee90aef](https://github.com/maidsafe/sn_messaging/commit/ee90aef0c3164db4d57aba0022a6b82b941eec1b))

## [1.5.0](https://github.com/maidsafe/sn_messaging/compare/v1.4.0...v1.5.0) (2021-01-12)


### Features

* **errrors:** remove unexpectedNode error. ([effc838](https://github.com/maidsafe/sn_messaging/commit/effc838d7ff1d3297eced4026a5584f0ac90291b))

## [1.4.0](https://github.com/maidsafe/sn_messaging/compare/v1.3.0...v1.4.0) (2021-01-12)


### Features

* **errors:** add node relocation error ([cc8887f](https://github.com/maidsafe/sn_messaging/commit/cc8887f37b667242b861f8f9554c1cca0b64eb7d))

## [1.3.0](https://github.com/maidsafe/sn_messaging/compare/v1.2.0...v1.3.0) (2021-01-12)


### Features

* **serialisation:** add a size field to the wire message header and support only Msgpack serialisation type for protocol v1 ([b9eb6d6](https://github.com/maidsafe/sn_messaging/commit/b9eb6d6db6148a1554cf2d42e2a177f7ac6e0db7))
* **serialisation:** serialize to JSON with a wire message header ([806f3e4](https://github.com/maidsafe/sn_messaging/commit/806f3e4042c752cd69a3e0970e677e6affc37488))
* **serialisation:** support Msgpack serialisation type ([74870b1](https://github.com/maidsafe/sn_messaging/commit/74870b11bbe4e35d7887304bccf3d3e81362ac38))


### Bug Fixes

* **serialisation:** minor refactor and fix to Msgpack deserialisation logic ([d7c84e6](https://github.com/maidsafe/sn_messaging/commit/d7c84e6e1dd4f594613dac54ed2cc0ae0e958849))

## [1.2.0](https://github.com/maidsafe/sn_messaging/compare/v1.1.0...v1.2.0) (2021-01-05)


### Features

* **deps:** use crates.io sn_data_types ([0a4270a](https://github.com/maidsafe/sn_messaging/commit/0a4270a18100fa4d046d658f54553a8fcbcdf168))

## 1.1.0 (2021-01-05)


### Features

* more errors ([b8144bc](https://github.com/maidsafe/sn_messaging/commit/b8144bcbb88ee3bdcad3a9933c80c9fc2ac2ed76))
* **init:** initial port of messaging from sn_data_types ([10b874c](https://github.com/maidsafe/sn_messaging/commit/10b874c01e853a86f65947136498450bf5ff293d))

# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.
