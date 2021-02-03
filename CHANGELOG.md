# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

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
