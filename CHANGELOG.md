# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### [0.7.11](https://github.com/maidsafe/safe_network/compare/v0.7.10...v0.7.11) (2021-07-08)


### Bug Fixes

* no concurrent chunks; go back a SE version ([18c000c](https://github.com/maidsafe/safe_network/commit/18c000c8170e9aff7d940b02e0006e40b64696b5))

### [0.7.10](https://github.com/maidsafe/safe_network/compare/v0.7.9...v0.7.10) (2021-07-07)


### Features

* add flags for json + resource logging output ([f7977bb](https://github.com/maidsafe/safe_network/commit/f7977bb3ea1a3592fececa199ea1b6bc0d2a98a2))
* **logs:** add periodic resource usage logging ([b0aaf59](https://github.com/maidsafe/safe_network/commit/b0aaf591f745b826bd1fdbbb8bfafc0f09a43600))


### Bug Fixes

* **logs:** use json formatting for better parsing ([85e094d](https://github.com/maidsafe/safe_network/commit/85e094d6f364ce87fad55bde82573c57c0421acf))
* **windows:** pid comparator changed for x-platform stability ([e89e9eb](https://github.com/maidsafe/safe_network/commit/e89e9eb9f98febab6716fbc451641b3523f16030))

### [0.7.9](https://github.com/maidsafe/safe_network/compare/v0.7.8...v0.7.9) (2021-07-07)

### [0.7.8](https://github.com/maidsafe/safe_network/compare/v0.7.7...v0.7.8) (2021-07-06)


### Bug Fixes

* **client:** Fix a doc comment typo ([8002db5](https://github.com/maidsafe/safe_network/commit/8002db5da8b301c1d27a8a9b5af06b0e2de27bdc))

### [0.7.7](https://github.com/maidsafe/safe_network/compare/v0.7.6...v0.7.7) (2021-07-06)

### [0.7.6](https://github.com/maidsafe/safe_network/compare/v0.7.5...v0.7.6) (2021-07-06)

### [0.7.5](https://github.com/maidsafe/safe_network/compare/v0.7.4...v0.7.5) (2021-07-06)


### Bug Fixes

* **audit:** remove unused encryption function and replace tmpdir with ([96d67a5](https://github.com/maidsafe/safe_network/commit/96d67a570c66b44e87643f0611c420882b418e42))
* **client:** use RwLock for cache and fix client test by clearing the ([25e6a70](https://github.com/maidsafe/safe_network/commit/25e6a70cabe81961b44f5653378e86fc74a78f26))

### [0.7.4](https://github.com/maidsafe/safe_network/compare/v0.7.3...v0.7.4) (2021-07-05)

### [0.7.3](https://github.com/maidsafe/safe_network/compare/v0.7.2...v0.7.3) (2021-07-05)


### Features

* **metadata:** improve concurrency ([8464d81](https://github.com/maidsafe/safe_network/commit/8464d81d9adbcedcad3cd4c9b8167d88e3f220d3))

### [0.7.2](https://github.com/maidsafe/safe_network/compare/v0.7.1...v0.7.2) (2021-07-05)


### Bug Fixes

* **routing:** forward client msgs when not for us ([1ce5920](https://github.com/maidsafe/safe_network/commit/1ce5920e4824d30690b6923ff17a2cc2109e3e2d))

### [0.7.1](https://github.com/maidsafe/safe_network/compare/v0.7.0...v0.7.1) (2021-07-03)


### Bug Fixes

* **tests:** add delay before reading a chunk write ([2f71715](https://github.com/maidsafe/safe_network/commit/2f7171585327c49dc1c63d179538bc4d033602eb))

## [0.7.0](https://github.com/maidsafe/safe_network/compare/v0.6.1...v0.7.0) (2021-07-03)


### ⚠ BREAKING CHANGES

* transfers removed

### Features

* **data:** add event store ([f7bab91](https://github.com/maidsafe/safe_network/commit/f7bab910704c9665692c4e9efd2539ef8d6140d7))
* **map:** remove unsequenced ([897c85e](https://github.com/maidsafe/safe_network/commit/897c85e184e07cf74788a8da54811acbca8af468))
* **register:** replace btreemap w dashmap ([201c993](https://github.com/maidsafe/safe_network/commit/201c99322069ca06a50e66b2dfa3e46861148edf))


### Bug Fixes

* **db:** store all under same db dir ([3b00bb1](https://github.com/maidsafe/safe_network/commit/3b00bb1ee32b607b319c6ac69e1a404047b6022e))
* remove node level check on query affinity ([781547c](https://github.com/maidsafe/safe_network/commit/781547cf4691ebd0c00d1333bb629f0ce6d52844))
* **concurrency:** remove several bottlenecks ([08f84f7](https://github.com/maidsafe/safe_network/commit/08f84f75d3ec52518a8cde245b6d9fdc1cb165d4))
* post-rebase issues ([ba66db1](https://github.com/maidsafe/safe_network/commit/ba66db1a7d981621af9ba512ae3b60d9701aa915))
* **doctests:** replace outdated docs with TODO ([418ba37](https://github.com/maidsafe/safe_network/commit/418ba379d6db3fc8d1c884fb4d6f376ae8b708ee))
* **tests:** update expected value ([dfa162d](https://github.com/maidsafe/safe_network/commit/dfa162d9e7fb0e24d715322cb5bf983771907603))


* remove transfers, payments, rewards ([ffb6865](https://github.com/maidsafe/safe_network/commit/ffb6865907cf5fa854b37898ff256994df9b8e57))

### [0.6.1](https://github.com/maidsafe/safe_network/compare/v0.6.0...v0.6.1) (2021-07-01)

## [0.6.0](https://github.com/maidsafe/safe_network/compare/v0.5.3...v0.6.0) (2021-07-01)


### ⚠ BREAKING CHANGES

* log format and filenaming changed now we use tracing's
rotation and appender

### Features

* use tracing instead of log ([2e4df7b](https://github.com/maidsafe/safe_network/commit/2e4df7bb10cc593a62cd13ae01fe613a34bcaba4))


### Bug Fixes

* ensure RUST_LOG takes precedence ([f27d2bf](https://github.com/maidsafe/safe_network/commit/f27d2bfe4dea1093f4b64d1b93590c8e3f8707ed))


* set ci logs, and use compact by default ([1cff0b1](https://github.com/maidsafe/safe_network/commit/1cff0b1616738e02830dd78f8685c26a37854b78))

### [0.5.3](https://github.com/maidsafe/safe_network/compare/v0.5.2...v0.5.3) (2021-06-30)


### Features

* **client:** add basic blob cache ([71918a8](https://github.com/maidsafe/safe_network/commit/71918a8a9fa558c702c42e2eb311c689cab4cefc))

### [0.5.2](https://github.com/maidsafe/safe_network/compare/v0.5.1...v0.5.2) (2021-06-30)


### Bug Fixes

* send back bounced message within SectionKnowledge ([91132d3](https://github.com/maidsafe/safe_network/commit/91132d32e018a75548a63334aa02a08a682cb915))

### [0.5.1](https://github.com/maidsafe/safe_network/compare/v0.5.0...v0.5.1) (2021-06-30)

## [0.5.0](https://github.com/maidsafe/safe_network/compare/v0.4.0...v0.5.0) (2021-06-30)


### ⚠ BREAKING CHANGES

* This removes a type and an enum variant from the public
API. Although they're not used by any maidsafe repos it's still a
breaking change.
* This removes types and enum variants from the public
API. Although I couldn't find any uses of them, this still constitutes a
breaking change.

* Remove redundant `section_info::Error` ([d21cd8d](https://github.com/maidsafe/safe_network/commit/d21cd8d37d890072ea1684a92aa14a3ad1fad808))
* Remove redundant SectionInfoUpdate variants ([38cf3de](https://github.com/maidsafe/safe_network/commit/38cf3de1199a196d71e583d03334ffb096699925))

## [0.4.0](https://github.com/maidsafe/safe_network/compare/v0.3.0...v0.4.0) (2021-06-29)


### ⚠ BREAKING CHANGES

* updates to use blsstc

### Features

* replace threshold_crytpo with blsstc ([a37947f](https://github.com/maidsafe/safe_network/commit/a37947f2b9d0f8c5151bea47473a34d8ef00f0b8))

## [0.3.0](https://github.com/maidsafe/safe_network/compare/v0.2.21...v0.3.0) (2021-06-29)


### ⚠ BREAKING CHANGES

* **client:** changes client API to add a query timeout

### Features

* **client:** adapt client tests to dropped errors change ([cb90f98](https://github.com/maidsafe/safe_network/commit/cb90f9828a4822a88881bb5263cf28e51c0e0ee4))
* **responses:** drop data error responses if NoSuchChunk is encountered ([dfa6be1](https://github.com/maidsafe/safe_network/commit/dfa6be1ee3e112a23c9b93bc6856d0e0342426bc))


### Bug Fixes

* **misc:** fixup docs ([00c6822](https://github.com/maidsafe/safe_network/commit/00c68228f179b270fc624fa1b611a22a86873798))
* **response:** drop error-ed blob responses at Elders instead of Adults ([ee763d9](https://github.com/maidsafe/safe_network/commit/ee763d950307b8e6d7d0dea9c80bc30bb0a48d22))
* **tests:** override timeouts on deletion tests ([c882fb6](https://github.com/maidsafe/safe_network/commit/c882fb60e35ac98d2bf1c614cdb54caff68a7267))

### [0.2.21](https://github.com/maidsafe/safe_network/compare/v0.2.20...v0.2.21) (2021-06-29)


### Bug Fixes

* **bin:** use multi-threaded runtime for node bin ([2ccb751](https://github.com/maidsafe/safe_network/commit/2ccb75181495a53867bae3bff1259bc788d51de8))

### [0.2.20](https://github.com/maidsafe/safe_network/compare/v0.2.19...v0.2.20) (2021-06-28)

### [0.2.19](https://github.com/maidsafe/safe_network/compare/v0.2.18...v0.2.19) (2021-06-28)


### Bug Fixes

* **comm:** properly close temp endpoint created for is_reachable() test ([c0843e3](https://github.com/maidsafe/safe_network/commit/c0843e397ccfa203d13b771f11e41ec78da1470e))

### [0.2.18](https://github.com/maidsafe/safe_network/compare/v0.2.17...v0.2.18) (2021-06-28)

### [0.2.17](https://github.com/maidsafe/safe_network/compare/v0.2.16...v0.2.17) (2021-06-24)

### [0.2.16](https://github.com/maidsafe/safe_network/compare/v0.2.15...v0.2.16) (2021-06-24)

### [0.2.15](https://github.com/maidsafe/safe_network/compare/v0.2.14...v0.2.15) (2021-06-24)


### Features

* **joins:** add MessageKind::Joins and avoid filtering the same at routing ([e1808e6](https://github.com/maidsafe/safe_network/commit/e1808e6f332ee9195ad78ae27b2777fca14eca02))


### Bug Fixes

* **joins:** add bytevalue to MessageKind::JoinRequest ([b04ac76](https://github.com/maidsafe/safe_network/commit/b04ac7645889b7257c11da506286d44a450dffc0))
* **joins:** do not check prefix on join retries ([e3931cf](https://github.com/maidsafe/safe_network/commit/e3931cfe53fb7bd5ccd134d127e2b2c36f6eee88))
* **joins:** redirect new node to it's closest section ([4af7086](https://github.com/maidsafe/safe_network/commit/4af70861086ad7eee8923da4a840572626ad14c2))
* **joins:** use the correct API for fetching redirect SAP ([b2af2cc](https://github.com/maidsafe/safe_network/commit/b2af2cc59a3968d98f6ea45162901b648fd27c9f))
* **keys:** always use new keys when joining network ([019bd02](https://github.com/maidsafe/safe_network/commit/019bd02ab7f82eccfc5c03c85e45f32607db7210))

### [0.2.14](https://github.com/maidsafe/safe_network/compare/v0.2.13...v0.2.14) (2021-06-24)

### [0.2.13](https://github.com/maidsafe/safe_network/compare/v0.2.12...v0.2.13) (2021-06-23)

### [0.2.12](https://github.com/maidsafe/safe_network/compare/v0.2.11...v0.2.12) (2021-06-23)

### [0.2.11](https://github.com/maidsafe/safe_network/compare/v0.2.10...v0.2.11) (2021-06-23)


### Bug Fixes

* **updater:** point self_update to new repo name ([73c52e1](https://github.com/maidsafe/safe_network/commit/73c52e13cd39198620fbab0e3d11fd9ced3688ba))

### [0.2.10](https://github.com/maidsafe/safe_network/compare/v0.2.9...v0.2.10) (2021-06-23)


### Bug Fixes

* merge remote section info during Sync ([ba28a37](https://github.com/maidsafe/safe_network/commit/ba28a3786c0b4ee3f5c5f844883e4be800e61e83))

### [0.2.9](https://github.com/maidsafe/safe_network/compare/v0.2.8...v0.2.9) (2021-06-23)


### Bug Fixes

* **logs:** use new module name for default logging ([3de6338](https://github.com/maidsafe/safe_network/commit/3de6338a3f12ca6a7edcb1ce7db2ce20f5a8b7ba))

### [0.2.8](https://github.com/maidsafe/safe_network/compare/v0.2.7...v0.2.8) (2021-06-22)


### Bug Fixes

* fmt ([499f7dd](https://github.com/maidsafe/safe_network/commit/499f7dda37151a3ab26d6bca4f44108b89909b97))
* remove the client timeout ([96b78c5](https://github.com/maidsafe/safe_network/commit/96b78c50f25339c3c974bb28a0247e8f51b9e2e8))

### [0.2.7](https://github.com/maidsafe/safe_network/compare/v0.2.6...v0.2.7) (2021-06-22)


### Bug Fixes

* Commit modified Cargo.lock ([545eaab](https://github.com/maidsafe/safe_network/commit/545eaab36cecdbd283f5502647d6e2cdd3f21cff))

### [0.2.6](https://github.com/maidsafe/safe_network/compare/v0.2.5...v0.2.6) (2021-06-22)


### Features

* **client:** AE for transfer updates to elders ([4c548e1](https://github.com/maidsafe/safe_network/commit/4c548e191a5ff1847894b36d3efc0649ddb0fb1f))

### [0.2.5](https://github.com/maidsafe/safe_network/compare/v0.2.4...v0.2.5) (2021-06-21)


### Bug Fixes

* dont double serialise every single message ([2204eea](https://github.com/maidsafe/safe_network/commit/2204eea265acd2d07c15bbba08c1ec705c50a5c1))

### [0.2.4](https://github.com/maidsafe/safe_network/compare/v0.2.3...v0.2.4) (2021-06-21)

### [0.2.3](https://github.com/maidsafe/safe_network/compare/v0.2.2...v0.2.3) (2021-06-21)


### Bug Fixes

* **node-bin:** log the node's addr the network failed to connect to as part of the connectivity test ([4c8ab3f](https://github.com/maidsafe/safe_network/commit/4c8ab3f71293fff723e45470b679d3adf353f16d))

### [0.2.2](https://github.com/maidsafe/safe_network/compare/v0.2.1...v0.2.2) (2021-06-21)

### [0.2.1](https://github.com/maidsafe/safe_network/compare/v0.2.0...v0.2.1) (2021-06-21)

## [0.2.0](https://github.com/maidsafe/safe_network/compare/v0.1.50...v0.2.0) (2021-06-21)


### ⚠ BREAKING CHANGES

* **chunk:** Blob is renamed to Chunk.
* Renaming in API.
* **relocate:** updates from routing PR2584

Co-authored-by: bochaco <gabrielviganotti@gmail.com>
* sn_messaging bump to 35
* **api-usage:** dependency updates
* **connectivity:** sn_messaging includes a breaking change
* rename Proven to SectionSigned, MemberInfo to NodeState, PeerState
* sn_messaging bump non-backward compatible.
* **routing:** adds a new routing message variant
* **bootstrap:** new node Join messaging is not backward compatible.
* **section_info:** SectionInfoMsg payload changed as well as it's name.
* sn_messaging bump
* **chunk-replication:** sn_messaging is updated to v31.0.0
* **join:** new Join messages are not backward compatible.
* refactor of SAP
removal of bls_signature_aggregator
rename Proof to Signed
* **messages:** adds new variant to the get-section-response
* **variant:** affects sn_routing's Variant handling
* **deps:** the messaging update includes a breaking change
* It's actually insufficient payment, as we dont check the balance of a wallet when doing ops
* **messaging:** sn_messaging updated

Also expands some logging
* removal of proofchain alters RoutingMsg type
* **api:** includes a breaking change to the public API
* sn_messaging used by this version is not backward compatible.
* **msgs:** some of these changes impact the pubic API requiring some newly introduced traits usage.
* **routingMsg:** RoutingMsg now contains its full hierarchy rather than just serialised msg bytes.
* Update DstLocation Direct->DirectAndUnrouted for clarity.
* removing support for Ping message type.
* removing the use of msg::Msg message type which breaks backward compatibility with clients.
* removing support for Ping messages.
* **header:** header messages are not backward compatible.
* new version of sn_messaging is not backward compatible for sn_node messages.
* removing msg::Msg enum which breaks backward compatibility.
* **session:** removing the EndUser registration step from the boostrapping stage
* **client-msgs:** using a non-backward compatible version of sn_messaging
* **messaging:** this version uses a non backward-compatbile version of sn_messaging
* **section_info:** EndUser is now a struct with just the use xorname and a socket id xorname.
* **cicd:** This should be bumped with messaging changes

This isn't _actually_ a breaking change, but a bump due to an earlier
commit missing one.

PRs starting with the title `Automated version bump` are auto generated as
part of the CI/CD process and so it is duplicate work running the PR workflow
on them. These changes skip PR CI for them.
This PR also switches the scheduled security audit to only run on the MaidSafe
org repo, not on forks.
* Messaging dep update.
* AE work
* **version:** Anti-entropy related changes before this commit
included breaking changes
* **event:** `Event` enum variants changed and new added.
* **store_cost:** Updated sn_messaging query response api.
* **deps:** Query response content changed.
* **storecost:** GetStoreCost query result content changed
* **chunk-org:** updates to sn_messaging 20.0.0 and sn_routing 0.65.0
* **deps:** update sn_messaging version to 20.0.1
* **deps:** update sn_messaging to v20.0.0
* **all:** this deprecates some of the messages
* **deps:** the updated dependencies have breaking changes
* **err:** updates to sn_messaging 19.0.0 and sn_data_types 0.18.3
and sn_routing 0.64.0
* **deps:** update sn_messaging to 0.19.0
* **err:** this renames one of the Error variants
* **queries:** `NodeCmdResult` removed from `Message` enum.
* **deps:** sn_messaging major version bump
* **adult_ack:** NodeCmdResult removed from Message enum.
* **data_sync:** sn_messaging and sn_routing breaking changes.
* **deps:** New major version for sn_messaging.
* **dataexchange:** Updated members on ReceiveExistingData cmd.
* for aggregate_at_src message, notify sn_node with proof as well
* **msg:** this adds a new variant to the message enum
* re-enable aggregate at source
* **deps:** Members of node cmds changed.
* **deps:** Node message members changed.
* **chunks:** Node messages changed members.
* **deps:** sn_routing major version bump
* **join:** this updates to the latest version of routing and qp2p
which have breaking changes
* new version of routing
- This commit is mainly to cover the change of Peer. Which used by
a public struct but won't trigger the version update automatically.
* **deps:** new version of sn_messaging
- This removes the unused `AtSource` aggregation scheme.
* **deps:** new version of sn_messaging
- Also removes handling of the unused `AtSource` aggregation scheme.
* **config:** this changes the fields of the node configuration
* **dep:** the new qp2p version includes a breaking change
* **deps:** the qp2p update includes a breaking change
* **config:** this commit changes some fields in the node config
* **deps:** Reward flow overhaul
* Events removed and event members changed.
* The `proof_chain` field of `Event::MessageReceived` is now `Option`.
* Aggregation scheme variant removed.
* This reverts commits in release 10.0.0
* **deps:** Changes to node reward messages
* Split message type into two, for passing process errors
or messages to be processed.
* Added `additional_proof_chain_key` parameter to  `Routing::send_message`, added `proof_chain` field to `Event::MessageReceived`.
* `Routing::neighbour_sections` renamed to `other_sections`.
* messaging and DT udpates
* **accumulation:** this changes uses a new version of sn_messaging with a
breaking change
* **messaging:** new version of sn_messaging includes a breaking change
* DT update. Naming and message structs for splits
* DT update. Messaging updates
* DT update. Naming and message structs for split
* SetupSections as opposed to GetSectionWallet, allows passing of sibling PK for proper setup of both section wallets
* `Routing::match_section` renamed to `Routing::matching_section`
* remove `Event::PromotedToAdult` and the `startup_relocation` field of `Event::MemberJoined`, both parts of public API.
* **tokio:** new Tokio runtime version is not backward compatible with tokio versions < 1.
* **tokio:** new Tokio v1 is not backward compatible with previous runtime versions < 1.
* **api:** Policy mutation APIs are removed.
* **routing:** Policy mutation operations are removed.

Co-authored-by: oetyng <oetyng@gmail.com>
* **Seq:** Policy mutation operations are removed.
* **tokio:** new Tokio v1 is not backward compatible with previous runtime versions < 1.
* **data-types:** new Sequence data-type doesn't allow Policy mutations.
* **Seq:** Policy mutation operations are removed.
* **messaging:** send_message api now requires an itinerary argument
* **api:** location scheme updated, breaking the current messaging api
* **deps:** new version of sn_messaging includes a breaking change
* **accumulation:** this changes uses a new version of sn_messaging with a
breaking change
* **accumulation:** this uses a new version of sn_messaging with a breaking
change
* added new field to the `Event::EldersChanged` variant.
* **location:** this adds a new variant to SrcLocation
* **deps:** New bootstrap flows and modified messaging types.
* **deps:** removes send_message_to_client api,
* **deps:** updated sn_messaging version
* **deps:** remove msgenvelope, change infrastructure msg
* adds more infrastructure information to bootstrap and on section key errors
* Adds pk to messages and helper on MsgEnvelope
* this changes the return type of State::new
* remove unused Error::NodeMessaging variant
* **types:** moving client messages to its own module and publis namespace.
* rename money to token
* rename money to token
* rename money to token
* This updates client creation, Arc<Keypair> is no longer
needed, as they keypair itself contains the Arcs we need.
*     - remove `Error::BadLocation` (use the more specific `InvalidSrcLocation` / `InvalidDstLocation` instead)
    - rename `Error::InvalidSource` to `Error::InvalidSrcLocation`
    - rename `Error::InvalidDestination` to `Error::InvalidDstLocation`
* this affects the `Error` type which is a part of the public API.
*     - remove `Routing::secret_key_share` (use `Routing::sign_with_secret_key_share` instead).
    - Rename `Error::InvalidElderDkgResult` to `Error::MissingSecretKeyShare`
    - `Routing::public_key_set` and `Routing::our_index` now return `MissingSecretKeyShare` instead of `InvalidState` on error.
* use `use sn_routing::Event;` instead of `use sn_routing::event::Event;`.
* `Event` changes:

- Remove `Event::Connected` - not needed because `Routing::new` now returns fully connected routing instance.
- Add `Event::Relocated` - replaces `Event::Connected(Connected::Relocate)`
- Remove `Event::InfantJoined` - merged with `MemberJoined`
- Change `Event::MemberJoined::previous_name` to `Option` to allow distinguishing between new and relocated peers.
* remove size fields within routing::Config
* remove NetworkParams
* some methods of `Routing` that previosuly returned `Option<T>` or `Result<T>` now return just T.
* rename Instance to Routing
* `Node` and `NodeConfig` are part of the public API.

### Features

* **joins:** add is_join_request method for WireMsg ([d205fb2](https://github.com/maidsafe/safe_network/commit/d205fb29057c3066ed8bc7ff0ea9d04eaf4dc2d7))
* **logs:** update flexi_logger and introduce log rotation ([831758a](https://github.com/maidsafe/safe_network/commit/831758a29279c7fda46d80807e5787ea342e2e06))
* **network:** add timeout on initializing routing ([ed5839e](https://github.com/maidsafe/safe_network/commit/ed5839ee705762ac8db5fa2f4f865461fdf7ac6e))
* add client codebase ([1bc8b34](https://github.com/maidsafe/safe_network/commit/1bc8b34adf5b4afd159c310f6893ab2deacb9a75))
* add messaging codebase ([0fae0dd](https://github.com/maidsafe/safe_network/commit/0fae0dd96b57320029a35d1d0d5529a8a3a3c520))
* add node codebase ([3b495df](https://github.com/maidsafe/safe_network/commit/3b495df9157e19197a42a50698d0ea58b708c250))
* add payment buffer for storecost fluctuations ([a4eecfa](https://github.com/maidsafe/safe_network/commit/a4eecfa035dca6b6c39e131f36b5204be6e7c0c6))
* add routing codebase ([195a5dd](https://github.com/maidsafe/safe_network/commit/195a5dd5e2785c7effbad3a081468e384258db79))
* always reconnect to elders ([1a3afa3](https://github.com/maidsafe/safe_network/commit/1a3afa3117d5f44036b48bf6799f2695cc3dfd78))
* deps ([5e00a26](https://github.com/maidsafe/safe_network/commit/5e00a26ce545443c61564b66fdc7724fdcdb40f3))
* discard blob errors if we get a positive one instead ([cc131a2](https://github.com/maidsafe/safe_network/commit/cc131a22e1d9cb177c6cd598810a50b22ade65be))
* Rerorg to monorepo and single crate ([92b446d](https://github.com/maidsafe/safe_network/commit/92b446da714367dd4f9fd7e70afb146c4e23cbd3))
* update sn_messaging ([1656625](https://github.com/maidsafe/safe_network/commit/165662508532ec343bcf367e53a8f4b1f54d128e))
* **AE:** flesh out remaining Anti-Entropy flow ([b28c422](https://github.com/maidsafe/safe_network/commit/b28c42261750b9c5db3715e50ab59a208776b953))
* **anti-entropy:** updates for sn_messaging new message enum ([5dfc53c](https://github.com/maidsafe/safe_network/commit/5dfc53cd4618affa63271bab88f59c954b8fcde1))
* **cicd:** exclude prs with title 'Automated version bump` ([1e28cf4](https://github.com/maidsafe/safe_network/commit/1e28cf40cbadc394d5ad73f21e91a45add039a60))
* **client:** add client signature to queries and commands ([eaa3b2a](https://github.com/maidsafe/safe_network/commit/eaa3b2acfce90c632c5f8464c90f4f1a095a0cdf))
* **client-msgs:** adapt to changes to client messages to receive client signature in each message ([0432987](https://github.com/maidsafe/safe_network/commit/043298714e70919fde269c462fcb009b6ef4cdd3))
* **connectivity:** refactor handling of lost connections ([96aecd9](https://github.com/maidsafe/safe_network/commit/96aecd9eb1d61395d1e1722e832f7a4a36f146ee))
* **errors:** maintain Error chain in our Error types while customising them when additional context is available/useful ([c89c3a4](https://github.com/maidsafe/safe_network/commit/c89c3a4ae169f822f3782484b6607ad228da0b04))
* **errors:** receive CmdErrors from the network ([ee194d5](https://github.com/maidsafe/safe_network/commit/ee194d58f9243e764e581d3f29c067e0bb4722c0))
* **examples:** add a simple example using Blob API ([5c5e764](https://github.com/maidsafe/safe_network/commit/5c5e764e5052d00301e269d1ff9a27499f23feeb))
* **header:** include the message id in the header ([d2c49c7](https://github.com/maidsafe/safe_network/commit/d2c49c7a6ffbe0ea70ef090bc3834cdc41e14263))
* **messaging:** Update sn_messaging ([19852a3](https://github.com/maidsafe/safe_network/commit/19852a343fc287269257a1895b81f659988465cd))
* **routing:** add variant to signal elders to start the connectivity ([710c44e](https://github.com/maidsafe/safe_network/commit/710c44ef5d619aab8757f3a69f8e4e5959fd9456))
* enable to directly generate wiremsg ([9ab515d](https://github.com/maidsafe/safe_network/commit/9ab515d02a08bcc397b2fc387b10dbb111bb77b9))
* force network to be joinable feature flag ([b2fb496](https://github.com/maidsafe/safe_network/commit/b2fb496f4a164ea2e549f3b6b4675d9383e72b76))
* InsufficientBalance error renamed ([8063c67](https://github.com/maidsafe/safe_network/commit/8063c67ba4eb6c565aaaedc6e2adc17f1ed57340))
* **adult_ops:** compute new holders for chunks and republish them on ([ce8d9e5](https://github.com/maidsafe/safe_network/commit/ce8d9e5b0e808fe8b8a1143d4674e09b8265d541))
* **adult_ops:** compute new holders for chunks and republish them on ([75e5c6e](https://github.com/maidsafe/safe_network/commit/75e5c6e568f84f9fc603346eaa77766b31a8496e))
* **api:** add new API for an Elder node to propose that a node has gone ([2937e59](https://github.com/maidsafe/safe_network/commit/2937e5938e84560850efd4eb892dcd353bc7790e))
* **api:** expose Blob utility API to be able to generate a data map without needing to connect to the network ([817efee](https://github.com/maidsafe/safe_network/commit/817efee20a6d4ff3f1170d0c3142f71891389e79))
* **chunks:** don't return an error when trying to write a private chunk which already exists ([83f5063](https://github.com/maidsafe/safe_network/commit/83f50637b605679b601d9ea5ea44752ffb84b077))
* **data-organisation:** republish data on AdultsChanged events ([d4289f0](https://github.com/maidsafe/safe_network/commit/d4289f05b43b5efb8d74505ba058e806625c70f3))
* move section_key into SectionAuthorityProvider ([7d2d476](https://github.com/maidsafe/safe_network/commit/7d2d4760dcb1e037612f9848884b5690ee0a67c2))
* send SectionKnowledge notifications during SAP change ([a99cf78](https://github.com/maidsafe/safe_network/commit/a99cf78f959515f6065710831879473015855ff0))
* update sn_messaging ([458ec64](https://github.com/maidsafe/safe_network/commit/458ec6471fd2e962e0b6b55679d92e048bc212fc))
* **api:** add new QueryResponse::is_success API ([8d1425e](https://github.com/maidsafe/safe_network/commit/8d1425e054797a4046eb5730cbe394facb3b9c21))
* **api:** adding new Register data type API ([c567542](https://github.com/maidsafe/safe_network/commit/c567542a49dc728f2e208152093f454dc4907715))
* **chunk-org:** track adult liveliness for republishing of data too ([6e2eca5](https://github.com/maidsafe/safe_network/commit/6e2eca5142c0581fc95489446bd2c1030451dbd6))
* **chunk-storage:** use CHUNK_COPY_COUNT when checking condition for ([667cf91](https://github.com/maidsafe/safe_network/commit/667cf91952a0212751def033c86a27f233862d82))
* **chunks:** report full when at 50% ([23e1fdb](https://github.com/maidsafe/safe_network/commit/23e1fdb56baa5b2ba703b1f88ac6c22787d37308))
* **chunks:** restore reg of liveness with queryresponse ([e11898d](https://github.com/maidsafe/safe_network/commit/e11898dc76a3c15b5b5565e72dceb2e7a8312904))
* **connMgr:** send queries to the 3 Elders closest to the name of target data ([94526ee](https://github.com/maidsafe/safe_network/commit/94526eede01c3722f671f7b41d43c88dc02cdb75))
* **data:** add NodeQuery variants to facilitate sharing of data to New Elders ([487af89](https://github.com/maidsafe/safe_network/commit/487af895238412a1b8fa66fcab6501ff3630b13f))
* **data:** share data on churn to New Elders ([8b101d9](https://github.com/maidsafe/safe_network/commit/8b101d9496375403f0803ec9c55db90d8ede9c9d))
* **data_section:** propose unresponsive Adult as offline using Routing ([339dd13](https://github.com/maidsafe/safe_network/commit/339dd1370bb3eeb6da0aef19c09b7dcb4c80ca5b)), closes [#1433](https://github.com/maidsafe/safe_network/issues/1433)
* **data_section:** track responsiveness of Adults to data requests so ([bcad135](https://github.com/maidsafe/safe_network/commit/bcad135bfaa85851a04ab8dc1613ef2964b9179f))
* **data-organisation:** republish data on AdultsChanged events ([6aea15d](https://github.com/maidsafe/safe_network/commit/6aea15d719536cdf5764ad58b9af57eb8a8adaa0))
* **joining:** open for new joins when nodes leave ([b307730](https://github.com/maidsafe/safe_network/commit/b307730a8cc4e50a906c6148f48166480c282b5f))
* **joins:** enable limiting joins again ([328b8d4](https://github.com/maidsafe/safe_network/commit/328b8d45b9ae9002017d9dff4e562934953946f7))
* **launch_network:** use NODE_COUNT env variable to set number of nodes ([aff6d4e](https://github.com/maidsafe/safe_network/commit/aff6d4eb19cc1be99a18ef1f2d90071a8efce8ff))
* **local_network:** connect to random elder ([21cd191](https://github.com/maidsafe/safe_network/commit/21cd1917c7facab19c80eac6daa679fea1d5830c))
* **message:** add Section PK to Messages ([9251792](https://github.com/maidsafe/safe_network/commit/9251792acb8aeb4613a4c99988a6ebf05eeedcde))
* **messaging:** restore target group size ([02fca6e](https://github.com/maidsafe/safe_network/commit/02fca6ead186bddc3577e1ae2177c90e2b6e69d1))
* **msg:** add convenience deserialize method ([7a83ab7](https://github.com/maidsafe/safe_network/commit/7a83ab7df527edd10ece17bcfa8478b204c2514a))
* **msg:** add convenience wrapper for client and node msgs ([06fd752](https://github.com/maidsafe/safe_network/commit/06fd75271b50dd5124c4b5f1ea35b84026a3d20b))
* **msg:** add message variant for NodeCmdResult ([f3a6e48](https://github.com/maidsafe/safe_network/commit/f3a6e487e18a29d7ba08b52edaf62b1b40be8feb))
* **node-cmds:** add a couple of convenience functions to serialise/deserialise NodeCmdMessages ([9fd827d](https://github.com/maidsafe/safe_network/commit/9fd827d21d9a54afc6edc16dbead37030f84d81a))
* **nodemsg:** add general error enum variant ([80c7056](https://github.com/maidsafe/safe_network/commit/80c7056324f6391c9532889a9e8930aab655b7e2))
* **register:** adding messages for Register data type operations ([082f544](https://github.com/maidsafe/safe_network/commit/082f544a6bc889cbc75ca806998c06504e9dbad8))
* **routingMsg:** adding all RoutingMsg definitions ([0a50f63](https://github.com/maidsafe/safe_network/commit/0a50f63cc02539a8d0e8f2625a49c3248c568d9c))
* **serialisation:** add source section public key to WireMsgHeader ([66320e3](https://github.com/maidsafe/safe_network/commit/66320e306a050df7c8294a7985a09269ed7a55ee))
* **session:** make Sessions agnostic of the keypair used to sign each individual client message ([cbe16fd](https://github.com/maidsafe/safe_network/commit/cbe16fd8ea78bbf6ac44c99831f31ae21629420d))
* **variant:** remove ConnectivityComplaint variant ([71c0b5f](https://github.com/maidsafe/safe_network/commit/71c0b5f72ba429ee396764b6b08567706d970e9d))
* add a SupportingInfo message variant ([4ac0399](https://github.com/maidsafe/safe_network/commit/4ac03999cce1ed3b29fa07c548ce22aecd199746))
* add SupportingInfo message support ([74d1c9f](https://github.com/maidsafe/safe_network/commit/74d1c9f763bbd00e92e0dd1f002b7b4f75207297))
* handle sending + receiving updated section wallet history ([fe4327d](https://github.com/maidsafe/safe_network/commit/fe4327d0e3feb5011267e9d3eda570028ecf504f))
* Initital set up for some lazy message sending ([a29b16c](https://github.com/maidsafe/safe_network/commit/a29b16ce13199dd05ff6e5d6f0ba490c97de5d25))
* send proofchain only for specific messages ([642401b](https://github.com/maidsafe/safe_network/commit/642401b73d3bce47de09512d8afec783353c819e))
* **dataexchange:** add structs ([3e7b68b](https://github.com/maidsafe/safe_network/commit/3e7b68ba3b1954beae8a5615643ca4c29160d60e))
* **multi-threading:** initial refactor for a multi-threaded node ([3962254](https://github.com/maidsafe/safe_network/commit/3962254bb08009c7eed1f716e41791ecbbdd567f))
* **storage:** changes to support the new Register data type in storage ([717bb01](https://github.com/maidsafe/safe_network/commit/717bb012539de0e1cf0a1f48e5a87bab3c623248))
* **store_cost:** return section key and bytes on query ([dd12653](https://github.com/maidsafe/safe_network/commit/dd12653424b8f46d8269c971f22a684ef308e002))
* **storecost:** expand query result with more data ([c5656c2](https://github.com/maidsafe/safe_network/commit/c5656c2cabf24bc1d4d51b588ad90abb80bcb41a))
* **storecost:** handle updated query response ([aa47973](https://github.com/maidsafe/safe_network/commit/aa47973c78f602100567d5946929fa36975ded17))
* include the info when join is allowed ([16a5870](https://github.com/maidsafe/safe_network/commit/16a587007e024a109b052cb176670dc4a15dc5e0))
* kill elder received too many connectivity complaints ([cc9ca8a](https://github.com/maidsafe/safe_network/commit/cc9ca8a39a24ff048d47d6a6c4d9dff07f1e1f40))
* nodes using different ages ([abb39c1](https://github.com/maidsafe/safe_network/commit/abb39c1e190582df02367ad75fb7e6d6f3a4e985))
* notify adult nodes with own section's adult list ([b4dddc0](https://github.com/maidsafe/safe_network/commit/b4dddc0fcc13ca196ccb66ff43a05ea91c72c732))
* restore aggregate at source ([4e86a20](https://github.com/maidsafe/safe_network/commit/4e86a20c6479a5cafda953e38fb61ca2b6d347d7))
* return TryJoinLater error when network disallow join ([a5e4d4b](https://github.com/maidsafe/safe_network/commit/a5e4d4bc0a086a9165545c40c9ba7e1471b043ff))
* Update DstLocation Direct->DirectAndUnrouted for clarity. ([2cfef24](https://github.com/maidsafe/safe_network/commit/2cfef2434c7957c95d774416489b686d8483dd0c))
* update sn_messaging. ([14e1f04](https://github.com/maidsafe/safe_network/commit/14e1f04dfab2f67051e887738d377b3808699054))
* use message_id instead of hash for message_filter ([9f937a7](https://github.com/maidsafe/safe_network/commit/9f937a75074076d31592537580c81ffa2be93763))
* use msg id for outgoing filter ([cc3e144](https://github.com/maidsafe/safe_network/commit/cc3e14405190e6c7588f1207265868a40ffb148c))
* use signature as outgoing message_id ([02dffda](https://github.com/maidsafe/safe_network/commit/02dffda7de5b892e831e344978fbc9d2910d0fb1))
* vote DKG non_participants off ([c4d6067](https://github.com/maidsafe/safe_network/commit/c4d6067679003de74380f218cd91e9f529c8bb5d))
* write network keypair to disk ([ad4c3ab](https://github.com/maidsafe/safe_network/commit/ad4c3ab2df7379e199bd1b1f53161321fe2e5af0))
* **accumulation:** accumulate elder to adult messages at destination ([12c2312](https://github.com/maidsafe/safe_network/commit/12c23122125f67eb7969366e7c49501677c562a8))
* **config:** rename local config to loopback and add lan option ([9e6a83d](https://github.com/maidsafe/safe_network/commit/9e6a83d143b16b320b6deaca9f68558d6bafe48b))
* **errors:** add specific errors ([9ba436c](https://github.com/maidsafe/safe_network/commit/9ba436c701b8fd38cf9f17ee6e9476611d65be84))
* **GetSectionQuery:** use PK instead of Xor so we have a response pk ([49f29fb](https://github.com/maidsafe/safe_network/commit/49f29fb3a6709a902cbb9c6c2e23d2c29dd5f036))
* **payment:** add payment to section funds ([3383e2d](https://github.com/maidsafe/safe_network/commit/3383e2d5d216ad124dde9b7dad768298f0e286f2))
* **Redirect:** provide elder name too ([7129f9d](https://github.com/maidsafe/safe_network/commit/7129f9d5e5f01235e42b87e0e15652241f7ae1c7))
* **rewards:** distribute to many based on age ([230ec03](https://github.com/maidsafe/safe_network/commit/230ec03ca9af7ad3e0b8ef71cb700fe8e080a964))
* **rewards:** limit supply ([1dae6c2](https://github.com/maidsafe/safe_network/commit/1dae6c24edeedce94703a715a3f6aa97304e3eb1))
* **rewards:** mint and reward at churn ([4ff05c4](https://github.com/maidsafe/safe_network/commit/4ff05c47bd3fa53f43dd28696f13c218c3f7f509))
* **section_funds:** remove section wallet ([2d48ce2](https://github.com/maidsafe/safe_network/commit/2d48ce2775b3b5364e01bd44c722ea3f0e79f233))
* **serialisation:** add destination XorName and destination section public key to WireMsgHeader ([7c65ff1](https://github.com/maidsafe/safe_network/commit/7c65ff1eacf98d97ee6cebc1e7796b5981c54e36))
* **tokens:** add the actual minting of new tokens ([d885117](https://github.com/maidsafe/safe_network/commit/d885117b6ce1d4cd074a77d92da27be181312595))
* add get balance handling ([75cfeb0](https://github.com/maidsafe/safe_network/commit/75cfeb03f2c181372bcec8e53f8acab81488d4ec))
* add id func ([2a6ccea](https://github.com/maidsafe/safe_network/commit/2a6ccead51550424ea6ac9db6227dddfd9f9ebb7))
* add processing error reasons ([7455746](https://github.com/maidsafe/safe_network/commit/7455746f827b4cfd4bfa9fc34a4af46872e44688))
* can register transfers again ([30156c5](https://github.com/maidsafe/safe_network/commit/30156c52ca1a54d96cf6481077109d803e1c0ff3))
* enable client section payout and history query of balance ([0639a8f](https://github.com/maidsafe/safe_network/commit/0639a8fb3a9749259968d05eb46f1c79a6eee190))
* expose processing error ([2237743](https://github.com/maidsafe/safe_network/commit/22377431752bb2430fcd57845c4d1e87526bbba8))
* functional msg id ([fd4062a](https://github.com/maidsafe/safe_network/commit/fd4062ab750f3c7ec49281ced38f5d548308b4ad))
* initial LazyError proposal ([2a971aa](https://github.com/maidsafe/safe_network/commit/2a971aab884a8ed4e8538262a92892f3fc5b6634))
* keep the genesis key and use it for fallback proofs ([99fb5ca](https://github.com/maidsafe/safe_network/commit/99fb5cacb4bd0782e3cbea3065b01c47ab1ee840))
* last byte of node's name represents its age ([69cef7a](https://github.com/maidsafe/safe_network/commit/69cef7aa7564b7ce86374de22314431c88073470))
* make headerinfo fields pub ([a868218](https://github.com/maidsafe/safe_network/commit/a86821861d58dba2838dc66197a99c15faef4d96))
* make source message optional ([943048b](https://github.com/maidsafe/safe_network/commit/943048baa6438339ed962b1e4885259fd3485b92))
* one message to rule them all ([8cb9c49](https://github.com/maidsafe/safe_network/commit/8cb9c49405782b9bd313ea98435799f31bd445f2))
* remove neighbour restriction ([269cff0](https://github.com/maidsafe/safe_network/commit/269cff02f17da755996f8189d20d4c1b2d2f3101))
* remove processErrorReason, just use standard error message ([72283d8](https://github.com/maidsafe/safe_network/commit/72283d86eb7703e0e8faa1f97d90a97c81cdf2ae))
* **aggregation:** set AtDestination where needed, and use section src ([814bb78](https://github.com/maidsafe/safe_network/commit/814bb785e5a1da79bdf0db6ec877df9c1293acb6))
* **api:** removing APIs that are meant for mutating Seuquence's Policy as they are now immutable ([9ad657b](https://github.com/maidsafe/safe_network/commit/9ad657b366b754c08772c2a446e7e9f7ceff57ea))
* **chain:** expose SectionChain via API ([1590414](https://github.com/maidsafe/safe_network/commit/15904147c279bbdd628fd3048d00d706e81061ea))
* **chunks:** handle read/write ([06b888d](https://github.com/maidsafe/safe_network/commit/06b888d1c2a303a1113d95d5176330f0d19bdc6b))
* **chunks:** set chunks at start, reset when levelup, set when level down ([797a17b](https://github.com/maidsafe/safe_network/commit/797a17bf99c1dd6743202496f94a8d301fd93c6d))
* **churning wallets:** simplify churn ([10485b8](https://github.com/maidsafe/safe_network/commit/10485b8b5bffe3835929e7d2422369396ef3f1ea))
* **cmds:** add CreateSectionWallet cmd ([8afb2cf](https://github.com/maidsafe/safe_network/commit/8afb2cf72252b2c737a695628920d725e12ce468))
* **data_cmd:** process payment for data command ([888337d](https://github.com/maidsafe/safe_network/commit/888337dad29d3a96a4a156709982be56759e4fc0))
* **elders:** remove hard coded elder count ([41b986b](https://github.com/maidsafe/safe_network/commit/41b986ba38ca1b2a2ee3c4f130bad82b22c5d950))
* **event:** add separate genesis event ([681d2c7](https://github.com/maidsafe/safe_network/commit/681d2c7c4d244f0ebf9016169c5b23c406b9f723))
* **event:** expose previous key in elderschanged ([0718e0c](https://github.com/maidsafe/safe_network/commit/0718e0ca7d11fb3cd4b0d3571909f3318514ec0c))
* **event:** update elders_changed event ([af37d06](https://github.com/maidsafe/safe_network/commit/af37d065b3eb3171ec9f68e4e665ad89ef01da81))
* **funds_split:** use genesis flow for creating new wallet ([3810f29](https://github.com/maidsafe/safe_network/commit/3810f29270da05cc85f1bca692d0313af440ec02))
* **lazy:** keep msg context for domain logic errors ([18edbf7](https://github.com/maidsafe/safe_network/commit/18edbf7e341bdb405738f8e7ff5c85c34295df7e))
* **msg_id:** generate from content ([386c092](https://github.com/maidsafe/safe_network/commit/386c0925c5ce974b8b08a634f7a98be6e03c297e))
* **msgs:** rename query ([9bb508b](https://github.com/maidsafe/safe_network/commit/9bb508b7b6f760cc214300450a6fda04d4d33528))
* **node:** add PromotedToElder event ([010bad2](https://github.com/maidsafe/safe_network/commit/010bad2595a9054c934f2464064b06cf2a654c13))
* **node:** handle promotion and demotion ([9a18633](https://github.com/maidsafe/safe_network/commit/9a186338b9b18d141bc4c80bcfb8c1ab67346c5a))
* **node:** init transfers and metadata after genesis ([815b0d0](https://github.com/maidsafe/safe_network/commit/815b0d0370e016571371f4d4dce6a3e52d719d90))
* **node_cmds:** enable mapping of node -> messages to process DataCmds ([3180519](https://github.com/maidsafe/safe_network/commit/31805191171c51998e90b2262be5fbd403805a15))
* **node_duties:** rename get wallet query ([e1fff62](https://github.com/maidsafe/safe_network/commit/e1fff62e37f6184c5f7c3a2c5ebcbb8f2219c4bd))
* **promotion:** allow Adult to receive Elder ops ([392f566](https://github.com/maidsafe/safe_network/commit/392f5664974844652553b3a5905c40e348b27c6c))
* **replication:** enable chunk replication on member left ([fad76e1](https://github.com/maidsafe/safe_network/commit/fad76e112e37604eedb7a8471398e9da532e6cd8))
* **rewards:** add reward payout mod ([d1e7e0f](https://github.com/maidsafe/safe_network/commit/d1e7e0f10ba42cc14cce558dcdac1227188bfdf2))
* **trait:** derive Clone Trait for multiple types ([eea57c2](https://github.com/maidsafe/safe_network/commit/eea57c276622b31e055fcfaaeaabd76f199d9d3c))
* add sibling key to Event::EldersChanged ([afd33e3](https://github.com/maidsafe/safe_network/commit/afd33e3607b6a467145042fffd9ff274dd5c89b4))
* chain payouts for section funds split ([f0e89c3](https://github.com/maidsafe/safe_network/commit/f0e89c3cc60f46a7be97f1cfe803f2de0cd6b5f8))
* create two transfers on split ([10cca6f](https://github.com/maidsafe/safe_network/commit/10cca6f5d891932894a89ec52ca22c69ecf6fca3))
* ElderPrep stage for adults ([0f3469e](https://github.com/maidsafe/safe_network/commit/0f3469e8ca63eda4d733f7f786986e814ea1d16e))
* implement new SectionChain that can resolve forks ([a3d786f](https://github.com/maidsafe/safe_network/commit/a3d786feb6f2bf6314c550423ec2789313fbf7be))
* listen for bootstrap response on IncomingMessages also ([f880f98](https://github.com/maidsafe/safe_network/commit/f880f9823e77b3727253f9dee01a304cc4e3eddd))
* new API: Routing::section_key ([486ee61](https://github.com/maidsafe/safe_network/commit/486ee61dfd77eaffcb2bd86c8c0eba6f470ec678))
* Remove GetReplicaKeys trasnfer query ([c4c7a2a](https://github.com/maidsafe/safe_network/commit/c4c7a2a019ac9478f2a81d513117c1a21308d7f1))
* remove unused events and event properties ([238a301](https://github.com/maidsafe/safe_network/commit/238a3016a1731a3abc7ca91b83e546992af85ec0))
* replace (old) SectionProofChain with (new) SectionChain ([03fb82c](https://github.com/maidsafe/safe_network/commit/03fb82cbf20b8f881bdb0df6a781e3f95f8f0118))
* simulated payout ([8293d03](https://github.com/maidsafe/safe_network/commit/8293d03beb2aea51be24d3462452dbcb0d410e7b))
* start new elders straight away, dont wait for data to come in ([bc3b736](https://github.com/maidsafe/safe_network/commit/bc3b736204187bafbde9d97861b931d9f8925e20))
* storecost ([b7f49ad](https://github.com/maidsafe/safe_network/commit/b7f49ad686dd0c68b1afacdf2e443ece3dd45c75))
* support adding additional proof chain keys to user messages ([2275730](https://github.com/maidsafe/safe_network/commit/2275730e276a5296dfe3a6b8c95fb6f516787aba))
* support dst accumulation with any message variant ([cc2f413](https://github.com/maidsafe/safe_network/commit/cc2f41361162a9ab0b2eab3d144de6cfb8152fe3))
* updates to message naming + sibling key passing ([05b0a32](https://github.com/maidsafe/safe_network/commit/05b0a32c2f9ef045f2c75e28af670899795c569d))
* updates to message naming, and removing sibling key passing ([75c9b0b](https://github.com/maidsafe/safe_network/commit/75c9b0b57708ef68667c5119029c297ae065f4d8))
* us our section pk when messaging ([c917b10](https://github.com/maidsafe/safe_network/commit/c917b108733c5765e520f6370ce4f336e8ae7ef2))
* use known vs all elders, supermajority ([c8ba2b5](https://github.com/maidsafe/safe_network/commit/c8ba2b57d53a0c2b9228223777829b8a9723b61c))
* use src from itinerary for dst accumulated user message ([31838e9](https://github.com/maidsafe/safe_network/commit/31838e99772cf8e2cc3cc901ba3ce47466270d11))
* use supermajority agreement + increase elder size to 7 ([b729a87](https://github.com/maidsafe/safe_network/commit/b729a870b58ea1e99099a374e4d21da76109b7f5))
* use supermajority for assesing responses ([8659f62](https://github.com/maidsafe/safe_network/commit/8659f62cea16ddf3ac840c11f6f23cf2e105f916))
* **accumulation:** accumulate elder to adult messages at destination ([a91e3d3](https://github.com/maidsafe/safe_network/commit/a91e3d3603330c21a46a61f0bd076c8e7fe9de37))
* **accumulation:** add support for accumlation at dest node ([f892838](https://github.com/maidsafe/safe_network/commit/f892838c994f243e6be17b5276b1c80ff10f5c3a))
* **age:** add age getter API ([07430a0](https://github.com/maidsafe/safe_network/commit/07430a07f5c4772014fc9db7108d3c9404f5702a))
* **api:** api updated ([4e11d0e](https://github.com/maidsafe/safe_network/commit/4e11d0ecf10eb1b9c5ead1ac5be0de1e079bff05))
* **bootstrap:** update for changes to bootstrap flow ([5af7cbe](https://github.com/maidsafe/safe_network/commit/5af7cbe255722dd7ddcf1a7f7334e317aa7c03d6))
* **churn:** handle wallet churn msgs ([3413025](https://github.com/maidsafe/safe_network/commit/3413025e9a3f278ae48ddc022d9fa98121758bb4))
* **churn:** put newbies into churn mode as well ([0162036](https://github.com/maidsafe/safe_network/commit/0162036af5e4ceb472b59bcc2d857fa91aea47cb))
* **enduser:** add bootstrap msg variants ([129924e](https://github.com/maidsafe/safe_network/commit/129924e03eb020881322b1ce3d5412de70c02172))
* **enduser:** add mapping between socketaddr and pk ([1ff902d](https://github.com/maidsafe/safe_network/commit/1ff902da2f28d89ed6ecb3efe502efea8476135e))
* **enduser:** replace socketaddr with a hash ([45ac67f](https://github.com/maidsafe/safe_network/commit/45ac67f80dd010c3536a4632dcbf952d0f01a007))
* **location:** add support for accumulation at destination ([89cadad](https://github.com/maidsafe/safe_network/commit/89cadad9669295f2833f0a161acd252d04e4218a))
* **messages:** implement location ([cf37569](https://github.com/maidsafe/safe_network/commit/cf37569d55515d35e5652c2c06f9ac3e8b3b7dbc))
* **messages:** remove MsgEnvelope ([57df069](https://github.com/maidsafe/safe_network/commit/57df069f6ed5d7c9afe3b665158181cce70ceb15))
* **messages:** remove MsgEnvelope ([d54b6c4](https://github.com/maidsafe/safe_network/commit/d54b6c42d119221f066d24109805b0995caf224b))
* **rewards:** enable reward payout again ([e8298da](https://github.com/maidsafe/safe_network/commit/e8298da568392a5ef621553ac456d9f3941fff36))
* **Seq:** upgrading sn_data_types to v0.16.0 and removing operations that are meant for mutating Seuquence's Policy ([306d8c1](https://github.com/maidsafe/safe_network/commit/306d8c16ea627f2aaed597d8c0df3698ab7d3a3e))
* **Seq:** upgrading sn_data_types to v0.16.0 which makes the Policy of a Sequence data type immutable. ([1334b08](https://github.com/maidsafe/safe_network/commit/1334b0876e4dabea492d425180e8199227b4c5b3))
* **systemcmd:** extend with wallet proposals ([553adaa](https://github.com/maidsafe/safe_network/commit/553adaa67e54130964e04d8e845f38cbcaa60dfc))
* **transfers:** propagate the credit proof only ([059eb74](https://github.com/maidsafe/safe_network/commit/059eb7427e99a4aadd50129c81b355757a41fb1d))
* add Envelope and InfrastructureQuery ([e0b999f](https://github.com/maidsafe/safe_network/commit/e0b999f961b971b068cad65bfe8e8f938bf4ab41))
* add infrastructure information. ([9ca78b7](https://github.com/maidsafe/safe_network/commit/9ca78b78a8acf0cc3f6d9b9195a1483c66934d49))
* Arc<Keypair>->Keypair updates to accommodate Dt and transfers ([dd23579](https://github.com/maidsafe/safe_network/commit/dd2357943f511a6fd90af837fea208bb1d9a4741))
* do not create connection when failed to send to client ([d5eadd8](https://github.com/maidsafe/safe_network/commit/d5eadd8dc2ae88af2ed26f2e9b0d58c20a69a516))
* error messages related to target pk ([08d31d3](https://github.com/maidsafe/safe_network/commit/08d31d3f694bf92562499a498bc0b7dd903ff61c))
* give Config public interface ([4b859d8](https://github.com/maidsafe/safe_network/commit/4b859d8449f6caf75dc544be0cd8652f3adf0ced))
* having EldersInfo change candidate considered as DKG in progress ([6137123](https://github.com/maidsafe/safe_network/commit/61371230e2eab7ceff4fd80073843d6b46ff4adf))
* improve fork diagnostics ([dbf9807](https://github.com/maidsafe/safe_network/commit/dbf98072a98bba734c6e0458936fa3aaa56ddeb6))
* log send to client error ([ddeff5e](https://github.com/maidsafe/safe_network/commit/ddeff5e0bf41dfdba3430a8df9ed4b51224822f9))
* make infra error its own type, use that in client::Error. ([122bc07](https://github.com/maidsafe/safe_network/commit/122bc0755078602a65275d4b7ccf2e8d759c8ef9))
* make use of sn_messaging crate for messaging serialisation/deserialisation ([cbc4802](https://github.com/maidsafe/safe_network/commit/cbc48026e6d1e32cde8a3f1f7ab92ca7aed801ad))
* modify bootstrap to use infrastructure queries ([9fb438f](https://github.com/maidsafe/safe_network/commit/9fb438f6a3c209a50733fd6b894cf4e4ca2861bc))
* notify client of incorrect section_key ([c54f034](https://github.com/maidsafe/safe_network/commit/c54f034fdc304106d1a3e56e00012773b1e85a9d))
* removal signature aggregate ([8bac521](https://github.com/maidsafe/safe_network/commit/8bac52163748bdc1fde54b3436c042fbd8f46b02))
* remove logging implementation ([cc320a0](https://github.com/maidsafe/safe_network/commit/cc320a04f01625f7a0e94d5c7df32e5d5d990fc8))
* remove old DKG sessions ([c8db72f](https://github.com/maidsafe/safe_network/commit/c8db72f8120c538ed41cbe1d036106ba3c0c04d9))
* remove unused Error::NodeMessaging variant ([0b70c28](https://github.com/maidsafe/safe_network/commit/0b70c28792076599af88dd61f9f6482116c2f3e4))
* require a section key PK to be passed with all messages ([60f5240](https://github.com/maidsafe/safe_network/commit/60f5240ac8242d04e970773cdccfcb7ccd4a9e3e))
* updates for section key response changes ([71f89d8](https://github.com/maidsafe/safe_network/commit/71f89d8c54008ff9974f740eff5be9ac2b893f26))
* **config:** read config file from an optionally provided path ([8d8724b](https://github.com/maidsafe/safe_network/commit/8d8724ba8824d91bc38a16dd144311005698b249))
* support multiple concurrent DKGs ([98fc101](https://github.com/maidsafe/safe_network/commit/98fc10194ddd73387a5539ad1e29423a224583d5))
* use redirected addesses for elders ([cbd89b5](https://github.com/maidsafe/safe_network/commit/cbd89b564da12d42fdbd62b4af92f80e6bf26cb4))
* **adult:** enable chunk duplication at adults ([771c618](https://github.com/maidsafe/safe_network/commit/771c618d9e35fccb2cafb2362eb4929ee63d04f5))
* **arc:** Require an arc wrapped keypair for init ([38e7ef3](https://github.com/maidsafe/safe_network/commit/38e7ef32ac416336af853cf663a82d57b919c8c3))
* **blob:** remove local blob cache ([8a1b871](https://github.com/maidsafe/safe_network/commit/8a1b871ebf70ce5ebcf8aaa9146705b29927f925))
* **config:** add support for --clean and --fresh flags ([0c29503](https://github.com/maidsafe/safe_network/commit/0c2950305eafeddc9f193e49bd246028f56dfb57))
* **connections:** updates to listen to all messages from each elder ([76c1836](https://github.com/maidsafe/safe_network/commit/76c1836db1eda7cc98e99bdef3d3c336fa03ab7f))
* **deps:** use crates.io sn_data_types ([0a4270a](https://github.com/maidsafe/safe_network/commit/0a4270a18100fa4d046d658f54553a8fcbcdf168))
* **deps:** use updated client ([468b690](https://github.com/maidsafe/safe_network/commit/468b6901f5b4c3c8ceaca3c0b7bf9f7f79f45e0d))
* **elder_change:** add finish step ([ef17827](https://github.com/maidsafe/safe_network/commit/ef17827de2e120f9f66dd6c1dd76946bfa9626bf))
* **error:** re-export sn_messaging::Error as ErrorMessage on the public API ([e3829b0](https://github.com/maidsafe/safe_network/commit/e3829b0d5d00cc262ca69fef92a1670118162a52))
* **errors:** add new more specific errors for invalid messages ([38a801a](https://github.com/maidsafe/safe_network/commit/38a801a57004b65305f01e6de7fb16131c9184a7))
* **errors:** add node relocation error ([cc8887f](https://github.com/maidsafe/safe_network/commit/cc8887f37b667242b861f8f9554c1cca0b64eb7d))
* **errors:** more mapping to sn_messages ([22fdd7d](https://github.com/maidsafe/safe_network/commit/22fdd7dcdb523178b422d5d12627b98b1cc592f2))
* **errors:** use thiserror for error construction ([946e3c2](https://github.com/maidsafe/safe_network/commit/946e3c2e38d88afd3082a9d345db1fbef155359b))
* **errors:** use thiserror for error construction ([678384e](https://github.com/maidsafe/safe_network/commit/678384e741822c1fa29b8cb1e6b48be160235316))
* **errors:** use thiserror for error creation ([bc093b6](https://github.com/maidsafe/safe_network/commit/bc093b6fc5cb43fe1bdfa8214e2f1907935e98a0))
* **errors:** Use updated sn_messaging ([e513ab3](https://github.com/maidsafe/safe_network/commit/e513ab3d737ac43b024d3216a689f36f3af476c6))
* **errrors:** remove unexpectedNode error. ([effc838](https://github.com/maidsafe/safe_network/commit/effc838d7ff1d3297eced4026a5584f0ac90291b))
* **genesis:** add msgs for genesis section init ([a808d3f](https://github.com/maidsafe/safe_network/commit/a808d3fbcf1ab10c8b21876ab177d97ffab47abc))
* **serialisation:** add a size field to the wire message header and support only Msgpack serialisation type for protocol v1 ([b9eb6d6](https://github.com/maidsafe/safe_network/commit/b9eb6d6db6148a1554cf2d42e2a177f7ac6e0db7))
* **serialisation:** serialise to JSON with a wire message header ([806f3e4](https://github.com/maidsafe/safe_network/commit/806f3e4042c752cd69a3e0970e677e6affc37488))
* **serialisation:** support Msgpack serialisation type ([74870b1](https://github.com/maidsafe/safe_network/commit/74870b11bbe4e35d7887304bccf3d3e81362ac38))
* add configurable bootstrap to client ([5ad120b](https://github.com/maidsafe/safe_network/commit/5ad120bfc7e734b543fafcb96acf877b32adaeb4))
* add stress test example ([cf25c48](https://github.com/maidsafe/safe_network/commit/cf25c48d3ba613db0a1e631620727a31f87d2661))
* carry out resource proofing during bootstrap ([a047ca1](https://github.com/maidsafe/safe_network/commit/a047ca1f88c65cc1d9b99c0602b856bb7acb4f9b))
* more errors ([b8144bc](https://github.com/maidsafe/safe_network/commit/b8144bcbb88ee3bdcad3a9933c80c9fc2ac2ed76))
* more logs ([14cc036](https://github.com/maidsafe/safe_network/commit/14cc0366dbb5ea1ba7bb04b7fa315986c933ccbc))
* relocate one infant with age increased by one when set joins_allowed flag ([03d9827](https://github.com/maidsafe/safe_network/commit/03d9827e591bf79fa5ecb775801ff8c325109fde))
* remove bootstrap stream listening also ([74855e2](https://github.com/maidsafe/safe_network/commit/74855e2bc2b1b14631c5921f52a40c3c16ea1dd6))
* remove client challenge ([50e3ed4](https://github.com/maidsafe/safe_network/commit/50e3ed45802c09ada8af2f1b8b2315e4e20319e7))
* remove stream storage for client management ([3313cd5](https://github.com/maidsafe/safe_network/commit/3313cd51d67541d8011b2295569d0cf1489a9128))
* **comm:** detect lost connections ([f4e9e3a](https://github.com/maidsafe/safe_network/commit/f4e9e3a00ce5b8905be06d7d6ffa6ea522108466))
* **err_listener:** implement CmdError listener and fix map data tests ([b57ba9a](https://github.com/maidsafe/safe_network/commit/b57ba9ad2780b280dc884e609b423a091fc8296b))
* **init:** initial port of messaging from sn_data_types ([10b874c](https://github.com/maidsafe/safe_network/commit/10b874c01e853a86f65947136498450bf5ff293d))
* **keycache:** adds a key cache and removes exposure of secret key ([b312446](https://github.com/maidsafe/safe_network/commit/b312446b6db2c2beaf6007d39619dd8969fc8428))
* **map:** refactoring Map API ([6b8cabc](https://github.com/maidsafe/safe_network/commit/6b8cabc5c51e7ead597035ede8e4e9676bed8b46))
* **seq:** Sign ops before applying locally + sending to network ([08d43c8](https://github.com/maidsafe/safe_network/commit/08d43c8a35643f25aecd5dc9c03911d1d2291067))
* add bootstrap message backlog ([75f0a5c](https://github.com/maidsafe/safe_network/commit/75f0a5c751835aba15a3cd42ae3b30900f6b1428))
* cancel running timers on drop ([d8f420f](https://github.com/maidsafe/safe_network/commit/d8f420f239ef3c2e0311681f4b620c230326d250))
* expose `Event` directly, hide `event` module ([d940b77](https://github.com/maidsafe/safe_network/commit/d940b77effde39376b8c7671dbf94f6607ce46ba))
* implement DKG message bouncing ([551c427](https://github.com/maidsafe/safe_network/commit/551c4276b0c737269716fe05da83fc2b34cfd63c))
* implement message resending ([cc2fcbd](https://github.com/maidsafe/safe_network/commit/cc2fcbd163eb80ec85a567b0eb8bc160fc84a312))
* instantiate the client w/ fullId not just sk ([79f064f](https://github.com/maidsafe/safe_network/commit/79f064f75e6b106ef3bc04357041b963303f0f9e))
* minor changes to the Event enum ([56e658f](https://github.com/maidsafe/safe_network/commit/56e658fe6a2fb0b2e1aeac8018f126512c944345))
* notify when key got changed during relocation ([2540a27](https://github.com/maidsafe/safe_network/commit/2540a27a3aafac61979d6b664e62655796c795ad))
* ping peers on connection loss to detect if they went offline ([d6be64f](https://github.com/maidsafe/safe_network/commit/d6be64f087341f31838d51dfbdfb067ed24895df))
* set filter number boundary ([c129bff](https://github.com/maidsafe/safe_network/commit/c129bff69d92400202bcefd1983eb028e3a26155))
* **blob:** expose self_ecnrypt API for dry run ([d3abe53](https://github.com/maidsafe/safe_network/commit/d3abe53d28ee15c1cb758399153e6c6a91a52165))
* **chaos:** add chaos macro to randomly perform chaos ([cfbf3a0](https://github.com/maidsafe/safe_network/commit/cfbf3a01bafc2edf02e85e71e63c81b0c5c73011))
* **launch:** network launcher will build current sn_node before launch ([6f5c49d](https://github.com/maidsafe/safe_network/commit/6f5c49d368f65e938c02506be5d118c58e7ed9c4))
* **launch_tool:** pass RUST_LOG value to the launch_tool --rust-log arg ([662c827](https://github.com/maidsafe/safe_network/commit/662c827817c62b615c5ca68586b32e4278141a4b))
* **multisig-actor:** use transfer share logic ([1e437a4](https://github.com/maidsafe/safe_network/commit/1e437a45b8a45f546e193f24f2500677766a64a9))
* **node_transfers:** add section payout error ([86114a5](https://github.com/maidsafe/safe_network/commit/86114a53593786ced19def470ddf262821d927ba))
* **nodeevents:** add SectionPayoutRegistered ([b782d19](https://github.com/maidsafe/safe_network/commit/b782d19cfa94a2d8b76cde714c3102dcfc9dc944))
* **qp2p:** update qp2p version ([41958b3](https://github.com/maidsafe/safe_network/commit/41958b3a0bbcbcc6be9b3ff853d858ae476680d1))
* **rewards:** use share for payout validation ([041330b](https://github.com/maidsafe/safe_network/commit/041330bec25561e350a4fe28cc36cba4eb5f4d51))
* **section_funds:** initiate section actor WIP ([e093675](https://github.com/maidsafe/safe_network/commit/e09367560975f0197e919454e97186338cfa0457))
* **section_funds:** use other section as replicas ([43e61b6](https://github.com/maidsafe/safe_network/commit/43e61b63e43598c4ac53254b888e81fdb1230235))
* **seq:** Use signed ops for sequence append ([62c7d46](https://github.com/maidsafe/safe_network/commit/62c7d46fbd1b11aafac495a26ccabf8dbc6da1df))
* **storage:** impl adult storage tracking at Elders ([11215bd](https://github.com/maidsafe/safe_network/commit/11215bd241bd653b9cc739202c63d164be943e2b))
* **storage:** monitor section storage and flip joins_allowed accordingly ([24ff1ce](https://github.com/maidsafe/safe_network/commit/24ff1ce94346cd04213b5c1bd510a0e408d3ee50))
* **stress test:** improve output ([33eac1b](https://github.com/maidsafe/safe_network/commit/33eac1b61383f231d0c34657db98d00cc84cf7c3))
* **test:** enable logger in tests using tracing-subscriber ([448522b](https://github.com/maidsafe/safe_network/commit/448522b7e994df7c13b5203ce7326c40aad900de))
* **transfers:** impl multisig validation proposal ([56a9ef3](https://github.com/maidsafe/safe_network/commit/56a9ef386a11c35f78150f9f812377fa6ba03754))
* **types:** adding Ping, NodeMessage and InfrastructureQuery definitions and support in serialisation ([dcd6b32](https://github.com/maidsafe/safe_network/commit/dcd6b321154714000d67c38137d1155433c4672a))
* allow rejoin with same name ([ded038d](https://github.com/maidsafe/safe_network/commit/ded038d8526246fab6c8a9c63918a74a02a4848e))
* do not expose BLS secret key share ([e8fa12e](https://github.com/maidsafe/safe_network/commit/e8fa12e4b528ce1e23657c2a2450f48adc3d20de))
* joins_allowed flag to toggle accept new node or not ([5def794](https://github.com/maidsafe/safe_network/commit/5def79408bfe16e37d7455b5c83037415429ce78))
* relocate all joining infants during startup phase ([492f4d7](https://github.com/maidsafe/safe_network/commit/492f4d7a5715fe48d1d1757b100fc8ac186ba669))
* relocation during startup no-longer required ([cf937e4](https://github.com/maidsafe/safe_network/commit/cf937e47bf41cc8b8724e7496f5040e69f95d67e))
* remove seq cache ([afc516b](https://github.com/maidsafe/safe_network/commit/afc516b6cb2e8ec0c54a9dc2232f21818ad802b8))
* set simulated-payouts as a default feature for now ([de6b2c9](https://github.com/maidsafe/safe_network/commit/de6b2c93fc994e0166943199e991befee923df80))
* start the first node with higher age ([d23914e](https://github.com/maidsafe/safe_network/commit/d23914ed998eb415a0e0f7af616eca6bf6ea4333))
* update client default config for idle/keep alive time changes. ([547dbdd](https://github.com/maidsafe/safe_network/commit/547dbdd2c7e77b66c8cc5715961c9c68d0fceaf2))
* update data types and client deps ([55249e1](https://github.com/maidsafe/safe_network/commit/55249e1db0c06334fa583e1370a40cd72d3da045))
* update elder listeners when incoming messages available ([90f36ee](https://github.com/maidsafe/safe_network/commit/90f36eed6b98b5329f997a22b2c76518a2adc205))
* update lseq data type ([b064eff](https://github.com/maidsafe/safe_network/commit/b064eff303f43c3f1f98d22c1b43aee8dba64b5c))
* use tracing for logging ([a68af40](https://github.com/maidsafe/safe_network/commit/a68af409d0700eaf6c25d1ccac65afc0626902d0))
* warn when we have an unused incoming message stream ([d348a57](https://github.com/maidsafe/safe_network/commit/d348a57729cabbd4e8ac366a901e7d0cdefee45e))
* **conn:** make query response threhsold dynamic ([ebf310a](https://github.com/maidsafe/safe_network/commit/ebf310a38b9506f7241a4c7d4296ee0d14ed28f5))
* **rand:** use OsRng instead of thread ([437340a](https://github.com/maidsafe/safe_network/commit/437340af6736d47b1650f6054a3930c60acc298b))
* **transfer_id:** Provide u64 and pk of transfer to be used as id ([7bcd6b3](https://github.com/maidsafe/safe_network/commit/7bcd6b310b8fad52124b537a88fc74222b2f66de))
* **transfers:** impl DebitAgreementProof aggregator ([8ad8c39](https://github.com/maidsafe/safe_network/commit/8ad8c395f8ac9838cbba3a71c08b86644cbce647))
* **transfers:** impl StoreCost for data writes ([70f93c7](https://github.com/maidsafe/safe_network/commit/70f93c72adc307df35bb58820f9f8efa20c9b877))
* **transfers:** impl StoreCost for data writes ([efaf2b0](https://github.com/maidsafe/safe_network/commit/efaf2b03b2dae6b02ffbc428fb2d816adf3bc8ae))
* **upnp:** use new version of qp2p with UPnP and echo service ([afb609e](https://github.com/maidsafe/safe_network/commit/afb609e030acf3002599e2cee14e80f81dae7b21))
* add testnet launcher bin, using snlt ([90710ea](https://github.com/maidsafe/safe_network/commit/90710ea74638f9f47df483803d32579121f5f978))
* implement lost peer detection ([cbc57ba](https://github.com/maidsafe/safe_network/commit/cbc57baea9d44637d7439d62872dd8bde0df40b9))
* implement proper node termination ([0fbced8](https://github.com/maidsafe/safe_network/commit/0fbced8a2efaac6be063aee2fb30b8f74f2e7df8))
* improve Comm api and documentation ([9ecfe8a](https://github.com/maidsafe/safe_network/commit/9ecfe8a5cf949ec741d6cf197930a83515538412))
* make the log identifier work again ([48d7ce7](https://github.com/maidsafe/safe_network/commit/48d7ce79d15f6b7da1cea328980aff835690b4ca))
* make the resend delay configurable ([8a0d043](https://github.com/maidsafe/safe_network/commit/8a0d043dc4079a4ff677b211c07bc4ffccdf9fdb))
* relocate only the oldest peers that pass the relocation check ([d7855b5](https://github.com/maidsafe/safe_network/commit/d7855b5cf3e18d49517f7f4daac96f0add47a8cf))
* remove join timeout - to be handled by the upper layers instead ([cb4f6fe](https://github.com/maidsafe/safe_network/commit/cb4f6feb6dc9949e1b865f6c8876d34cfd93322f))
* remove resend delay ([9b0971e](https://github.com/maidsafe/safe_network/commit/9b0971e1aea11b2ada4cc56d70d1d0195631aaad))
* remove Variant::Ping ([18a9d40](https://github.com/maidsafe/safe_network/commit/18a9d40f9e8a8210b53a00afbe40bada2abcac3f))
* **async:** adapt tests and fix typo-induced bug ([cbcb44d](https://github.com/maidsafe/safe_network/commit/cbcb44dcbf7537608f9054a256bbce232cdbec40))
* **async:** adds used_space max_capacity() getter ([7ca06eb](https://github.com/maidsafe/safe_network/commit/7ca06eb4c12aee2ddc4655d559ce6d72a942025f))
* **async:** load keys/age concurrently on startup ([a48d6a4](https://github.com/maidsafe/safe_network/commit/a48d6a441eda274e7e365714e87715d40ce8a900))
* **async:** made used space tracking async-safe ([1c7a621](https://github.com/maidsafe/safe_network/commit/1c7a6210d747dd0b56677dc001119f2560fecca4))
* use unbounded channel for Events ([fb5a3aa](https://github.com/maidsafe/safe_network/commit/fb5a3aa2eb1af018d82fcdfbe11a9a3b156525b1))
* **api:** Add get_balance_for api for specificly PK requests ([78847f8](https://github.com/maidsafe/safe_network/commit/78847f8c3e289a87b9088be9f2d166ede11bfad1))
* **api:** expose an async event stream API, and adapt node module to use qp2p async API ([a42b065](https://github.com/maidsafe/safe_network/commit/a42b065edad3225ccbcad30ed9755e7eff78cd10))
* **apis:** remove get_seq/unseq_map apis, and go for cleaner get_map ([3b47500](https://github.com/maidsafe/safe_network/commit/3b4750082e9ea21193f098045ebac31a27d1dc03))
* **async:** introduce async logging and make functions async/await to ([1b18a95](https://github.com/maidsafe/safe_network/commit/1b18a956bb769f517eb442744326e5fcd2c6faae))
* **async-log:** re-introduce async logging using a wrapper ([337ac57](https://github.com/maidsafe/safe_network/commit/337ac5715dc85d20ac16b8c14d7ed084a70f1b63))
* **chunkduplication:** enable duplication trigger ([48799c2](https://github.com/maidsafe/safe_network/commit/48799c244a1fd7d4ac7efbe48c58d33bf9f5c38b))
* **ci:** auto generate dependency graph via CI ([ac13840](https://github.com/maidsafe/safe_network/commit/ac13840c0bcee2db67c38275b83eef2be3e3f24f))
* **connection_manager:** improve handling of connections ([158ba06](https://github.com/maidsafe/safe_network/commit/158ba0690451e34ed5bdb10e7c771602b1b501fb))
* **connections:** set up listener for events/errors ([deeecc6](https://github.com/maidsafe/safe_network/commit/deeecc62bb65e99663683f6b2712c1156420adbc))
* **duty_cfg:** cover first node, adult and elder ([2c17416](https://github.com/maidsafe/safe_network/commit/2c17416bda0e181cf59805c32cb9b8b7951734c7))
* **elder:** set bls keys on promoted ([4233ec6](https://github.com/maidsafe/safe_network/commit/4233ec6bdea8f54f109202113f33a7fbb8774d54))
* **errors:** add error for insufficient elder connections ([357ca33](https://github.com/maidsafe/safe_network/commit/357ca33290f3ab19edfbb3d08f6414004b5a142f))
* **farming:** accumulate reward on data write ([16310b3](https://github.com/maidsafe/safe_network/commit/16310b313198286a57de7382428d95a466b7a822))
* **farming:** add some temp calcs of base cost ([e250759](https://github.com/maidsafe/safe_network/commit/e250759035a61337c845f4a0a37d95d4ca448906))
* **farming:** new section account on elder churn ([062cab6](https://github.com/maidsafe/safe_network/commit/062cab6d9bd32ddd215fd10f728894e4c9ea2509))
* **farming:** update metrics on elder churn ([7d9c55c](https://github.com/maidsafe/safe_network/commit/7d9c55c52dface58b9512efd59ac387b41b2f6f9))
* **genesis:** first node introduces all money ([3068865](https://github.com/maidsafe/safe_network/commit/3068865a7368d61402bd192313f4917f10db0373))
* **launch:** network launcher will build current sn_node before launch ([2c1c56a](https://github.com/maidsafe/safe_network/commit/2c1c56a32bce11d8206cde4e2c5770e0ce6ff9b4))
* **listen:** Initial implementation of listen on network ([b38c9bf](https://github.com/maidsafe/safe_network/commit/b38c9bf922f0a10480e13c98076c6a8b2fa70f18))
* **logs:** create separate log files for each thread ([d0dd77a](https://github.com/maidsafe/safe_network/commit/d0dd77a7f76813f87698578c848a6452f84bde56))
* **metadata:** set and delete chunk holders ([d4817b5](https://github.com/maidsafe/safe_network/commit/d4817b542a811460c8dfac659707d1e2ac58dc17))
* **msganalysis:** add detection of node transfers ([99b12c2](https://github.com/maidsafe/safe_network/commit/99b12c27f3a4f52d283f1a0f235ed298e238807f))
* **node:** cache Connections to nodes ([a78c305](https://github.com/maidsafe/safe_network/commit/a78c30596400e360b880caafb41a8c94c3bc5b67))
* **node wallet:** simplify pubkey to/from config ([505de20](https://github.com/maidsafe/safe_network/commit/505de2060567ce11da5f21e2bbe2d4fd379d0506))
* **node_wallet:** separate node id and wallet key ([18868ea](https://github.com/maidsafe/safe_network/commit/18868ea12ab517a89bb4d29c9b49f875784e7ae9))
* **payment:** add query for store cost ([6071931](https://github.com/maidsafe/safe_network/commit/60719318b4143f431d2d5fb4b90530d427450ca6))
* **qp2p:** Inital set up to enable listeners for qp2p ([63adbc7](https://github.com/maidsafe/safe_network/commit/63adbc7cbca5736850c880cb2316202bffebd94a))
* **replica:** complete the init query flow ([92a9a4b](https://github.com/maidsafe/safe_network/commit/92a9a4b9444c9aae6ef65d0daa1aa82dd867b5f1))
* **rewards:** accumulate reward counter ([96936e6](https://github.com/maidsafe/safe_network/commit/96936e64074420c94550d88aff7fc79b7f8dbf44))
* **rewards:** payout rewards to elders on split ([44bc3ea](https://github.com/maidsafe/safe_network/commit/44bc3ea753bcf1b1c438d0110d97fe935327198b))
* **rewards:** rewards declining with network size ([2060107](https://github.com/maidsafe/safe_network/commit/20601071e7bf2e3d9cd7f1dadcc57c6069a0448f))
* **rewards:** set node reward wallet at startup ([b062fda](https://github.com/maidsafe/safe_network/commit/b062fda7dbcba0d4e9bc6d34f87d36535c2e4ac3))
* **rewards:** use msg_id for idempotency ([04220f4](https://github.com/maidsafe/safe_network/commit/04220f459e4d1d98d0d2b8b3498755bac6ad1ba6))
* **section_actor:** enable naive transition ([61e5954](https://github.com/maidsafe/safe_network/commit/61e595416127371d827efc26153d741156b7e25f))
* **section_funds:** set new actor on elder churn ([ff41cf4](https://github.com/maidsafe/safe_network/commit/ff41cf4fed16a177005a68a88d4bd5fd5571df78))
* **self-encrypt:** re add self encryption to client blob apis ([e550dad](https://github.com/maidsafe/safe_network/commit/e550dad3137d240d901077f04bc8cde1a23eed3c))
* **seq:** update for latest seq data_type changes ([34dfb17](https://github.com/maidsafe/safe_network/commit/34dfb17b4a96e844be1a9ac792ef41aa002c4896))
* **seq:** Update to sn_data_types and update seq apis ([ad248a7](https://github.com/maidsafe/safe_network/commit/ad248a7e7fa6ab015ca02f61075642e6dc2ee619))
* **transfers:** keep key subset at replicas ([0943f06](https://github.com/maidsafe/safe_network/commit/0943f066098b3760e1224421bde48452bd657e50))
* **transfers:** store transfers to disk ([82d65cf](https://github.com/maidsafe/safe_network/commit/82d65cf5e0db43f4409ab8d261113f2860202937))
* **writes:** use dynamic rate limit for writes ([0b86894](https://github.com/maidsafe/safe_network/commit/0b868948234ad5809d3aa3271bc2d75e1b0cacc5))
* add `phase-one` feature for updates ([7a1c1ca](https://github.com/maidsafe/safe_network/commit/7a1c1ca0f0b9b1a647513579af85b164606fe66d))
* complete farming flow ([e9db602](https://github.com/maidsafe/safe_network/commit/e9db60298a3a09a7875bb5018003369b03ad08e0))
* use XorName instead of Prefix for section message src ([d2347ee](https://github.com/maidsafe/safe_network/commit/d2347eee21a3d5e86ae0c76e133e00cc1a850eeb))


### Bug Fixes

* add node signing to adult and elder state ([bba2b96](https://github.com/maidsafe/safe_network/commit/bba2b96523d4e4f76a86c6a835baf2fc90657f2a))
* add sibling key to constellation change ([d2551ac](https://github.com/maidsafe/safe_network/commit/d2551ac4566df4ff9d4b1e5b386d537d7133eedf))
* fixing stress statistic couting error ([47f2024](https://github.com/maidsafe/safe_network/commit/47f2024a364721c39644f9df45475ccf27eb76c1))
* use main branch for bumping against ([032a0c6](https://github.com/maidsafe/safe_network/commit/032a0c6a9059940ad1d0f3fb7dfe2ff2542060e3))
* use updated logfile on e2e fail ([c01a1cb](https://github.com/maidsafe/safe_network/commit/c01a1cb3db99a4aa9feed991b6c2db6606f6270b))
* **filter:** ignore JoinRequests from filtering ([0a7f720](https://github.com/maidsafe/safe_network/commit/0a7f720d175afb03faea654f89c79f6777bbafc6))
* avoid cache dead-lock ([8aa62ba](https://github.com/maidsafe/safe_network/commit/8aa62ba656983f450e80c62ab887502d44539769))
* avoid re-votes of JoinsAllowed got filter out ([f3c9090](https://github.com/maidsafe/safe_network/commit/f3c9090a53eecc28800dfdd729765ecd05f63448))
* await on future for the stress test ([cf66585](https://github.com/maidsafe/safe_network/commit/cf6658592d8634be9200929d0990ba7b83af7949))
* catering qp2p error change ([1747cc3](https://github.com/maidsafe/safe_network/commit/1747cc36873f0725873fd0e3579412d49cb698bc))
* changes according to latest code, plus some clippy and fmt fixes ([dae49ee](https://github.com/maidsafe/safe_network/commit/dae49ee5b65b654e255642992cfd8e79f79eb608))
* cleanup and PR comments ([125806a](https://github.com/maidsafe/safe_network/commit/125806aac6f1b275b67af76fdf631b7036d092b8))
* dont expose no file err, only use it if available ([a40417c](https://github.com/maidsafe/safe_network/commit/a40417c56995821c7192187abe3bfa0ecbbe40e8))
* get a write lock for handling transfers ([f671b32](https://github.com/maidsafe/safe_network/commit/f671b32fe78b7f757271ff9bfd657473942b32e7))
* handle history failure more properly ([f46f025](https://github.com/maidsafe/safe_network/commit/f46f025a79f7ac5fb0f5e9baf13e69fea110aebf))
* increase store cost buffer ([8f1a806](https://github.com/maidsafe/safe_network/commit/8f1a8065e3c1284e7b657ff519e898580a8e784d))
* logging properly during split ([3a5ed09](https://github.com/maidsafe/safe_network/commit/3a5ed09c7bf542930120269b5049ac97cbd13333))
* make JoinRequest be handled properly by AE after section split ([5ae7f1d](https://github.com/maidsafe/safe_network/commit/5ae7f1d63df612e58bf2e00b515bea54ad458621))
* manual Role debug impl ([e0bc9b5](https://github.com/maidsafe/safe_network/commit/e0bc9b5e64b7227b0b5ab67a3d1ecb1d768ecfaa))
* multiple fixes and rebase atop T5 ([a2c56bc](https://github.com/maidsafe/safe_network/commit/a2c56bcdfc1da1c4f37edc4b4c158b2d632dce5c))
* node now uses contact config file when no HCC supplied ([234cdfe](https://github.com/maidsafe/safe_network/commit/234cdfe6b1da333fb515b134789ac3641780b09d))
* post-rebase issues ([2bedd59](https://github.com/maidsafe/safe_network/commit/2bedd59cbc31932f0bed264ae35c43f891e8ba7b))
* proper differentiate local close and peer un-reachable ([655d8a2](https://github.com/maidsafe/safe_network/commit/655d8a2e55a8ea1747b03705783ba87b3a07d676))
* rebase atop T5 ([e97ca23](https://github.com/maidsafe/safe_network/commit/e97ca238072fdf69408cdc4181c966f68f863fbe))
* remove some get_history calls. ([dbf53ea](https://github.com/maidsafe/safe_network/commit/dbf53ea44c2808e354002e514aa033a80d2848be))
* remove the obsolete member of Network ([6b6788b](https://github.com/maidsafe/safe_network/commit/6b6788bb4ee1b3ee7733d2eac733bc9bea32f9b5))
* resolving failing AE tests ([fcdf30a](https://github.com/maidsafe/safe_network/commit/fcdf30af2c56d7d9cd3bb4f1e4f08e6995bfe02a))
* restore outgoing filter and only have one wire_msg copy for multiple recipients ([ba98b41](https://github.com/maidsafe/safe_network/commit/ba98b41380193397d8bbeabd3b2a876c572a0235))
* RwLock changes in a couple of instances ([4c09366](https://github.com/maidsafe/safe_network/commit/4c09366b10b2364da71419b1bf8edecdb9ceea32))
* tests ([3c6cee4](https://github.com/maidsafe/safe_network/commit/3c6cee454cd651c37553187b775901bee178de53))
* tie tokio to 1.6 for stability ([1df79d1](https://github.com/maidsafe/safe_network/commit/1df79d13a62084a5efee4ed9a92bcbc8ba3c07f3))
* various master workflow fixes ([730148a](https://github.com/maidsafe/safe_network/commit/730148a0b9f7191c2b88c868c8e6e2e5b4a1c4d7))
* **adult-liveliness:** hold only node addresses for write liveliness ([fd019b5](https://github.com/maidsafe/safe_network/commit/fd019b5205f9af545aa16d18a338921ae10aaa08))
* **AE:** rebase fixes of AE atop T4.2 ([ac8a030](https://github.com/maidsafe/safe_network/commit/ac8a0304f20567b67eca5a0b57bd73fb86961d82))
* **always-joinable:** fix configuration check ([a08312b](https://github.com/maidsafe/safe_network/commit/a08312b6a2d2ac330cfad7ac729bacad8ebb8dea))
* **blob:** don't mark Blob responses for aggregation at destination ([a4efe5b](https://github.com/maidsafe/safe_network/commit/a4efe5b060879da7c7459d1a6fc6e417429a2cc0))
* **cache:** address PR comments ([1e6c0c4](https://github.com/maidsafe/safe_network/commit/1e6c0c4a3934725a9eca5c51ccfac9e4f08c40ac))
* **clippy:** box NodeTask variant to fix clippy error ([6cfd416](https://github.com/maidsafe/safe_network/commit/6cfd41609955070c27ed80025e59677c3a5cd456))
* **comm:** ignore connection loss ([693f61c](https://github.com/maidsafe/safe_network/commit/693f61c3e92495a3acb2872684dd9a88c4a8bd1a))
* **comms:** do not take connectivity complaints from adults ([f76b3a2](https://github.com/maidsafe/safe_network/commit/f76b3a2b12b3791e08d2c5a65d0ae594ddbf8c99))
* **docs:** fix broken doc tests ([2871061](https://github.com/maidsafe/safe_network/commit/287106131f986c77682570760d0d036ffa5953b3))
* **end-user:** assign clients a xorname which always matches the section prefix so they are propoerly routed in a multi-section network ([ac4a27c](https://github.com/maidsafe/safe_network/commit/ac4a27cdee61048273be49fc9c9500d4009c6192))
* **error-handling:** return a message to the sender when any error was encountered ([1327cc2](https://github.com/maidsafe/safe_network/commit/1327cc264986b9ab52ea7d8f4be4d19831b6acb8))
* **examples:** Migrate routing examples to crate root ([7ca0c63](https://github.com/maidsafe/safe_network/commit/7ca0c63189c3099c74fba444ec6f945094020fcf))
* **messaging:** don't discard error responses for Blob Queries ([c81a35a](https://github.com/maidsafe/safe_network/commit/c81a35abe96de7b397a8cc3780e6870bd0a98c33))
* **messaging:** only try get section key when to aggregate ([28a5bda](https://github.com/maidsafe/safe_network/commit/28a5bda17a6125003e8624e91dde8467818293de))
* **msg:** attach correct proof_chain to messages ([e0cd846](https://github.com/maidsafe/safe_network/commit/e0cd8462959697e6565bfba1d3cee7e08d2001ee))
* **query:** timeout when we don't get a response for a query ([9e6b782](https://github.com/maidsafe/safe_network/commit/9e6b7827f8cd054200a61887bc20c33e634b93bb))
* **rate_limit:** return u64 max value as store cost when no nodes ([d2b5b58](https://github.com/maidsafe/safe_network/commit/d2b5b585f6847e9516490c8d403e37e67b89cd55))
* **store_cost:** add network full error ([dbf81ed](https://github.com/maidsafe/safe_network/commit/dbf81ed764137996bbffadb51392d394196645c3))
* **test:** fix AE tests ([4d8ae27](https://github.com/maidsafe/safe_network/commit/4d8ae27bd2d27edfb08ccc042ea25fa8792c7d6e))
* **test:** fix node_msg_to_self test ([a563a77](https://github.com/maidsafe/safe_network/commit/a563a77797482460a04daf5acdd9bf1a9cc763ae))
* **tests:** fix tests after refactor and rebase ([20e857e](https://github.com/maidsafe/safe_network/commit/20e857e2418679de1818a916322917f38cb2f74d))
* **variant:** verify chain for SectionKnowledge variant also ([056766c](https://github.com/maidsafe/safe_network/commit/056766c9b1488e38f7c241e7bb969087541d2c0c))
* convert SrcLocation to DstLocation in AE messages ([314dc3e](https://github.com/maidsafe/safe_network/commit/314dc3eb0c71dc327ebfd04c4618e8e171d88208))
* fix dst_info for send_or_handle method ([649d27b](https://github.com/maidsafe/safe_network/commit/649d27b58313e2e35af998eb5a8351fcac63fed4))
* fix dst_key in send_or_handle ([6436aee](https://github.com/maidsafe/safe_network/commit/6436aee5be7df1e28a50fa60634b6e4bdc6a319f))
* post-rebase issues ([ddd0682](https://github.com/maidsafe/safe_network/commit/ddd06821a29a380ed75c67164c19b2597a704ac9))
* **adult_ops:** avoid underflowing decrement ([896e6b0](https://github.com/maidsafe/safe_network/commit/896e6b00db5377b610c7f0946e25cdcb8d75cc3d))
* **adult-tracking:** misc. fixes for republishing data and tracking adult responsiveness ([4c931be](https://github.com/maidsafe/safe_network/commit/4c931beb6faa994a2d54f2dcc29b5818d37d1026))
* **AE:** implement SrcAhead flow ([ade92fb](https://github.com/maidsafe/safe_network/commit/ade92fba775e768e2d99c8a1cbad6df94dd9546e))
* **blob-storage:** handle edge-cases when republishing Blob data ([e596869](https://github.com/maidsafe/safe_network/commit/e596869cf97cd44731a37d73b3e7dfd1b9de7434))
* **blob-storage:** handle edge-cases when republishing Blob data ([aadffef](https://github.com/maidsafe/safe_network/commit/aadffef59ed36928a446fbe4cc5c0629475eab18))
* **capacity:** allow joins after split ([45fc453](https://github.com/maidsafe/safe_network/commit/45fc453817894e430051eb9a81dba7d94ecbf369))
* **chunk:** remove message aggregation for Chunks queries ([20dd687](https://github.com/maidsafe/safe_network/commit/20dd6872e68e2ae42ce3a7a7e15f0bc2bb59df37))
* **chunk_storage:** exclude full adults while computing closest adults ([9069b3f](https://github.com/maidsafe/safe_network/commit/9069b3fe97675eed3f45ecd179ddf9699897f5f8))
* **chunk-ops:** propagate errors back if the blob-write was client ([7ced9d8](https://github.com/maidsafe/safe_network/commit/7ced9d863037d29de21be57cd759e9980c4df4a6))
* **chunk-ops:** respond back to clients with an error when adults are ([4c10c17](https://github.com/maidsafe/safe_network/commit/4c10c17f38b0fcdd4f9751631f1c1843e939eaff))
* **connectivity:** use separate endpoint to test connectivity to new ([26a2bcc](https://github.com/maidsafe/safe_network/commit/26a2bccdcc7659e3d585bacd24b31ab842e4f5c0))
* **data:** multiple fixes on usage of chunkstore and sharing of data ([ae12c29](https://github.com/maidsafe/safe_network/commit/ae12c29f7cdfd29faa4500a5616b00de31aa4d4d))
* **data:** skip data sharing on network startup ([a6a6beb](https://github.com/maidsafe/safe_network/commit/a6a6beb30c1a2662d914a05f96d4eb98ebd1f33c))
* **data_section:** remove offline nodes from the pending adult ([63498b6](https://github.com/maidsafe/safe_network/commit/63498b6e0e131adea4cf56b4c591b4bff8c36ec2))
* **delivery_targets:** deliver to all when targets are final dst ([f26722b](https://github.com/maidsafe/safe_network/commit/f26722b156194cb834d02cf33e53a54b1cd3b6a0))
* **deps:** use released version of qp2p instead of git branch ([c9b2392](https://github.com/maidsafe/safe_network/commit/c9b23920aa1acb13fc62c488f5d4c1b1fb82bf53))
* **error:** use correct address for DataNotFound error ([1518e57](https://github.com/maidsafe/safe_network/commit/1518e57e77bb9a43dceab4f51bd6a98033844a40))
* **full_nodes:** retain members only on full_node db during splits and churns ([2ff4ff8](https://github.com/maidsafe/safe_network/commit/2ff4ff8c93631adb823ac5539e6545615f03c17d))
* **messaging:** remove all non-bootstrapped elders from local list of elders ([275c353](https://github.com/maidsafe/safe_network/commit/275c353b5fb3595aa7812c3f6ab2066577d63288))
* **storage:** do storage checks on writes at adults as well ([42b9b78](https://github.com/maidsafe/safe_network/commit/42b9b78007b6c87edc64443e6a5410fd8ccacd42))
* add GetChunk variant under NodeSystemQueryResponse fixing chunk-replication ([cf6f5d3](https://github.com/maidsafe/safe_network/commit/cf6f5d3c2287e1272c0b330a128e70806cf6372e))
* fix unresolved import ([a325254](https://github.com/maidsafe/safe_network/commit/a325254422a6fb6a27cdf1b96ad17c44dc0a35e4))
* post-rebase issues ([b55744d](https://github.com/maidsafe/safe_network/commit/b55744d090c19e4a1e3e899b1155a91fd98f68ea))
* re-add necessary msgs that were removed ([9643da6](https://github.com/maidsafe/safe_network/commit/9643da687b31cae189bfc714641012128e4ab3ac))
* **full-adults:** use BTreeSet to hold full adult information for ([85ecb70](https://github.com/maidsafe/safe_network/commit/85ecb70e1210f808ac51b9d219e17fdea810a002))
* **msgs:** add aggregation scheme to SendToNodes ([ca19c7d](https://github.com/maidsafe/safe_network/commit/ca19c7d8dec0b4b89dd24f923a8d3cac4092de27))
* **node_msg:** remove unused target_section_pk field ([898c1a9](https://github.com/maidsafe/safe_network/commit/898c1a95e6d0e74d2179e5921340fc9b479899e1))
* **node_msg:** rename and expose id function ([8deb221](https://github.com/maidsafe/safe_network/commit/8deb221298c4ce4a9345946f74cbd32f5544e90d))
* resolve failing tests after SectionAuthProvider refactor ([99d5d28](https://github.com/maidsafe/safe_network/commit/99d5d283f5977f1d1d16a8789290c454cce1f49a))
* **queries:** restore client as recipient of chunk query response ([113daee](https://github.com/maidsafe/safe_network/commit/113daee9f8551a4c5a2c50c2eafb3c8a7a873dae))
* **query-listener:** listen to query responses from any of the elders ([b157eee](https://github.com/maidsafe/safe_network/commit/b157eeee20e27db68ccbb0b5ee07c10fc7baf37d))
* **replication:** copy and filter data before clearing ([5d6b110](https://github.com/maidsafe/safe_network/commit/5d6b11008dbd464a1622ae4d6d2330b6d0761aae))
* **section_funds:** fix unfinished loop when dropping wallets ([bae1cc9](https://github.com/maidsafe/safe_network/commit/bae1cc9d461cd5240b0abff9d4fad0cbd0fb8954))
* change stack size to 8mb for all windows builds ([27b4c4e](https://github.com/maidsafe/safe_network/commit/27b4c4ea9b3d37bb947961229c3b29e48b014586))
* do not overwrite existing reward key ([eee066b](https://github.com/maidsafe/safe_network/commit/eee066b3b300162deabff1bf48958713fbb3fb0d))
* enable relocation again ([f9fde30](https://github.com/maidsafe/safe_network/commit/f9fde30572e19e2ef50cb3f75a47714f8670332a))
* genesis elder count must be at least 5 ([7ff3703](https://github.com/maidsafe/safe_network/commit/7ff3703393f2e378d11881d6636af8e0238dca71))
* handle from_history error in transfers ([50c5c39](https://github.com/maidsafe/safe_network/commit/50c5c39e5692a298ad2394e2b12294344591d7da))
* handle nothing to sync error ([e20d437](https://github.com/maidsafe/safe_network/commit/e20d437912650d408863d76f33dcaab90a1b38cd))
* include remainder when splitting section funds ([f8094bb](https://github.com/maidsafe/safe_network/commit/f8094bb7975124c0138665a53cb55f2c843e300b))
* initiate elder change after becoming elder ([015fdf1](https://github.com/maidsafe/safe_network/commit/015fdf10bc8f5ddf32e08cc094f7a2d60ed008e9))
* no split in first section ([81a716f](https://github.com/maidsafe/safe_network/commit/81a716fce20da6a2521c0a41f2133d6704568d28))
* notification only about live adults ([01a8524](https://github.com/maidsafe/safe_network/commit/01a8524db851cf120d338347b2e1435976a4f8ba))
* only send adult list notification when no elder changing ([4964a20](https://github.com/maidsafe/safe_network/commit/4964a20fab42e78b6a1cab0951dcbdde7bc53449))
* only send to client directly when it belongs to self section ([b8ddc1b](https://github.com/maidsafe/safe_network/commit/b8ddc1b86f728c08734bbf06d4a5c63ee63a4f4b))
* pending transitions ([d30920a](https://github.com/maidsafe/safe_network/commit/d30920a0807e4a1555d51e64ff360a1f5a622fff))
* post-rebase issues ([ab40c41](https://github.com/maidsafe/safe_network/commit/ab40c41cad0385e36508cae829f4109451051240))
* post-rebase issues + clippy ([dffd5c3](https://github.com/maidsafe/safe_network/commit/dffd5c332ad3fc7658c1744b8359812a34456943))
* **bootstrap:** reverting previous change to again wrap endpoint.connect_to() with a timeout ([0d51c57](https://github.com/maidsafe/safe_network/commit/0d51c57bba21b5ef914576d537db5ba3ac6fddc7))
* **bootstrap:** wrap endpoint.connect_to() with a timeout ([7ea75b1](https://github.com/maidsafe/safe_network/commit/7ea75b13fcdad521ab49854f964694bc58d85227))
* **chunks:** add in missing capacity reached check ([2da76af](https://github.com/maidsafe/safe_network/commit/2da76af4fb375b409159115fd263e9e0977f3fe7))
* **config:** set connection info when genesis ([9857435](https://github.com/maidsafe/safe_network/commit/9857435f96e9fae834d96cd0be717efdb3b5210b))
* **connection_manager:** set forward-port to true to use public address ([0e5a21f](https://github.com/maidsafe/safe_network/commit/0e5a21f0642952390982d69d25e6c2781c039c04))
* **join:** automatic retry when RoutingError::TryJoinLater is ([3ce6180](https://github.com/maidsafe/safe_network/commit/3ce6180785ac110557a143fc2100649c78acbb49))
* **propagation:** rely on routing aggregation ([f9840a3](https://github.com/maidsafe/safe_network/commit/f9840a324d3714035afa2159303d1dfb0480160a))
* **replication:** update holders ([aff4370](https://github.com/maidsafe/safe_network/commit/aff4370746e851bd27c6d504485e238388ee5eb1))
* **rewards:** distribute according to work ([62ff7dc](https://github.com/maidsafe/safe_network/commit/62ff7dca6dcc5cb722287d7bb716f7b6229de243))
* **rewards:** improve distribution ([4927633](https://github.com/maidsafe/safe_network/commit/492763350d68ef34abea8c019f0a1541523788df))
* adults not able to send non-aggregated messages ([9248cd0](https://github.com/maidsafe/safe_network/commit/9248cd071a7c5f2a7dd95ac201a40267c3ac3e6a))
* bounce untrusted messages directly to the original sender ([1bed232](https://github.com/maidsafe/safe_network/commit/1bed232ba085aa46fd71b3469366f5ab029aab8c))
* clarify api ([6db2924](https://github.com/maidsafe/safe_network/commit/6db29249540c3329a7e92c2f3187933ffa5f7159))
* create listeners before sending queries ([2651bfb](https://github.com/maidsafe/safe_network/commit/2651bfb9715ebf86cce7f683b4fe27eb7dadba1b))
* don't send OtherSection or vote TheirKnowledge to our section ([95f14d8](https://github.com/maidsafe/safe_network/commit/95f14d8ef869d263cc782e7faf91e7bc160dcf16))
* dont connect to elders until we have had a full section response/pk ([a3ec50e](https://github.com/maidsafe/safe_network/commit/a3ec50e1be7110995e65234fa4f7888e9aac712e))
* expose create_processing_error_msg ([0070d89](https://github.com/maidsafe/safe_network/commit/0070d899cee49a18be5477f2585a0283d8c02c08))
* Logging during conn attempts fixed ([fdeb84f](https://github.com/maidsafe/safe_network/commit/fdeb84f3c125d5774f77d59293f4d1ff64e7e6e3))
* missing export of NodeSystemQueryResponse ([1883557](https://github.com/maidsafe/safe_network/commit/1883557ff72acc72eeaf9c33425d7be722f4f08e))
* no router startup fixed w/qp2p udpate ([29b98ea](https://github.com/maidsafe/safe_network/commit/29b98eabe87921377b605c86ac8724453b55ba8f))
* post-rebase issues ([1350573](https://github.com/maidsafe/safe_network/commit/13505732ab1911a53ec08f48a88c1447e66f2b67))
* post-rebase issues ([ce4f194](https://github.com/maidsafe/safe_network/commit/ce4f19451ed37622f2cafe9da012b0968b6ae8a6))
* propagate only once per wallet churn ([f072e89](https://github.com/maidsafe/safe_network/commit/f072e891e71a3374db84be6556edebcd54c2850e))
* reduce unneeded lazy messages ([0498f24](https://github.com/maidsafe/safe_network/commit/0498f2447cc7cee91b1a897b0227d806861782a3))
* relocated allowed to join with own age ([018a9b8](https://github.com/maidsafe/safe_network/commit/018a9b8d1ae3e189f6381641f7721e419d5d13a7))
* relocated node can have higher age to join after first section split ([68b3e1e](https://github.com/maidsafe/safe_network/commit/68b3e1e1335b2ca6b23c2779fef013793d694e3d))
* remove potential panic in SignedRelocateDetails ([23d0936](https://github.com/maidsafe/safe_network/commit/23d09363211fc6d957f74ef85bc103c27685644b))
* remove some unnecessary logs ([7ef9542](https://github.com/maidsafe/safe_network/commit/7ef9542c819829752ec93ca1b5d0d144fe13e13d))
* restore EldersChange to a previous version ([0a85b87](https://github.com/maidsafe/safe_network/commit/0a85b879d5a2173daf24f49fbb3e106ecc0a0f5d))
* send register payout to section instead of nodes ([fc6ed33](https://github.com/maidsafe/safe_network/commit/fc6ed33cb67cf007fba31f2f028274132d2ea87a))
* send wallet history query to our section ([ca58521](https://github.com/maidsafe/safe_network/commit/ca58521e8b24514f58923af93b072292b90b8d4d))
* TEMP_HACK: use random port to not throw conn pool out of whack ([311fa30](https://github.com/maidsafe/safe_network/commit/311fa301b5b932b5ed5ec03ff216360742b8624b))
* unimplemented error handling of transfer error ([e21aa51](https://github.com/maidsafe/safe_network/commit/e21aa51f4aea36c3a5afcb276c4a373b8a032a85))
* **bootstrap:** avoid duplicate GetSectionRequest ([84327e2](https://github.com/maidsafe/safe_network/commit/84327e2521dfcace503886e3d4b79c3118cc4464))
* **bootstrap:** connect to all nodes and dont overwrite qp2p ([bcb31bd](https://github.com/maidsafe/safe_network/commit/bcb31bd410172c9f8c1245a9389b70776f9b7d6a))
* **bootstrap:** fix bootstrap logic when we get SectionInfo::Redirect ([cd6a24e](https://github.com/maidsafe/safe_network/commit/cd6a24ef46936fde1879bbb6df7b4feeb3ade24d))
* **bootstrap:** require GetSectionResponse to match our destination, not name ([4f484f1](https://github.com/maidsafe/safe_network/commit/4f484f1ea93f5d83180d5c77fcb5b3a680322d31))
* **churn:** swarm wallet when created ([401d04d](https://github.com/maidsafe/safe_network/commit/401d04d8146d02eb666c27b0348b700ad38fff0f))
* **dkg:** allow multiple pending key shares ([92dfc70](https://github.com/maidsafe/safe_network/commit/92dfc70a8bd18108f0c3a2f6d657b1e72e0a76cd))
* **dkg:** avoid mixing DKG messages from different generations ([e68ba2a](https://github.com/maidsafe/safe_network/commit/e68ba2aad975285c3968a67b38040d110c4f7d78))
* **elder:** query old key when new ([9a48bc2](https://github.com/maidsafe/safe_network/commit/9a48bc2f4112c91eb9a0bba51a65be34deae8c5d))
* **genesis:** init full section funds at completed ([108265e](https://github.com/maidsafe/safe_network/commit/108265ec4b6c5859df3a7e40cc6869e501a5d841))
* **genesis:** proper check if genesis ([fee55b8](https://github.com/maidsafe/safe_network/commit/fee55b8a39d244dc3139cd34933057465779abe5))
* **genesis:** propose also when still elder ([b8c790c](https://github.com/maidsafe/safe_network/commit/b8c790c06e129b0c5367ed748566282dcbb8671b))
* **metadata:** stop tracking adults when promoted ([44a4a19](https://github.com/maidsafe/safe_network/commit/44a4a19b64443bb210127af6ff38a83414a155f5))
* **rewards:** sync wallets in a better way ([090344f](https://github.com/maidsafe/safe_network/commit/090344fe089248c508587773e1d41ab2a5cc4607))
* use routing 0.57 for qp2p wan fixes ([e08dde7](https://github.com/maidsafe/safe_network/commit/e08dde78e5b727e0c9a7162b64f2021e6983457c))
* **section_funds:** reset transition after completed ([eba03da](https://github.com/maidsafe/safe_network/commit/eba03da2ac4af5f5d6c7ce97f07df13d7f5292ff))
* **storecost:** storecost always at least 1 ([df887e3](https://github.com/maidsafe/safe_network/commit/df887e3eb46b00cf023ad5929ac5cfba20fbd50e))
* **test:** fix assertion in blob_deletions test ([ad7d2ab](https://github.com/maidsafe/safe_network/commit/ad7d2ab7d46fc114856be799f7914ed4d640ce3c))
* **test:** increase the number of nodes in the drop test from 3 to 4 ([9ce0ec7](https://github.com/maidsafe/safe_network/commit/9ce0ec7da7483eacd9f5941bad470a4a821d0fd3))
* **tests:** refactor delivery group tests ([6437b76](https://github.com/maidsafe/safe_network/commit/6437b76bdc632366a71f00d0fdb55fc3947f44ab))
* **transfers:** update replica info on churn ([a052cd6](https://github.com/maidsafe/safe_network/commit/a052cd6f53e3a5ccff66cfdd146cf4032d211da4))
* send OtherSection to src of the original message, not src section ([cd3e382](https://github.com/maidsafe/safe_network/commit/cd3e38226af242950ce06797ea2ebf308b9cea31))
* transfer listener cleanup should happen only once ([66454f7](https://github.com/maidsafe/safe_network/commit/66454f72e675e57d208688068c2f87d00c61fb03))
* **churn:** send prop and acc to our elders ([e0055c2](https://github.com/maidsafe/safe_network/commit/e0055c2d0b3497797257fbe894523b30aedc369b))
* **data:** redirect data requests to the correct section if it does not ([5c3c195](https://github.com/maidsafe/safe_network/commit/5c3c195d60d2d2d302a4a1513a5f715324ca4128))
* process resulting duties ([13f9590](https://github.com/maidsafe/safe_network/commit/13f9590f06a9d64b4e6672e9aea35fa1847914ee))
* remove redundant origin field ([b341b00](https://github.com/maidsafe/safe_network/commit/b341b0002821cb1eb149b6b22e37f01083b0f768))
* ugly temp fix for lagging dkg outcome.. ([5673093](https://github.com/maidsafe/safe_network/commit/5673093cdd76388c8b608fa5e27bbd17b82500f4))
* wire up last stage of creating wallet ([4c01773](https://github.com/maidsafe/safe_network/commit/4c0177366c1b59eeac4f5b5e7bb67d1fe95e2773))
* **transition:** derive Clone for multiple structs ([335b93e](https://github.com/maidsafe/safe_network/commit/335b93e89156b53046963994030c34b513d0c86c))
* **transition:** start transition after getting Section Wallet history ([f49ed54](https://github.com/maidsafe/safe_network/commit/f49ed54233a70d848051f21eb9ce6c8f2bc25983))
* add missing routing to client of relayed client message ([fbde5b1](https://github.com/maidsafe/safe_network/commit/fbde5b10d734fcf5037b0d37767ba5093376e46e))
* dont use bincode for envelope deserialization ([818f75b](https://github.com/maidsafe/safe_network/commit/818f75b3a405d9b80e403d8d9f21e6e2803b332b))
* post-rebase issues ([906ef03](https://github.com/maidsafe/safe_network/commit/906ef031585f3db19a546928e76c8304a7f3c7f3))
* set response elected flag after electing best of the rest ([27726ee](https://github.com/maidsafe/safe_network/commit/27726eeb063500b48116d680659434429771045d))
* **connection_manager:** remove incorrect cloning of session ([67060d1](https://github.com/maidsafe/safe_network/commit/67060d1cb3d67f53d7d3653328ee3acd954deea0))
* add string payload to invalid bootstrap error ([23ed16c](https://github.com/maidsafe/safe_network/commit/23ed16c2dd68f75c2554f7cc4d54c6fd6a9c7522))
* address and remove comments ([a5017c2](https://github.com/maidsafe/safe_network/commit/a5017c22c57a504404d472a1e578216bd2d344fa))
* addresss review comments ([27dcac5](https://github.com/maidsafe/safe_network/commit/27dcac57b78daa9b41c481ed3970453d172720b4))
* adds used_space.reset() and reset() on age-up ([1267872](https://github.com/maidsafe/safe_network/commit/1267872c2e73fc9c440c612c14819850ca303df6))
* adjust for dt updates ([19d77a9](https://github.com/maidsafe/safe_network/commit/19d77a9b35afb9e31a0c42a0bbb0694934fdeac5))
* allow only one relocation at the time per node ([0e4d05f](https://github.com/maidsafe/safe_network/commit/0e4d05f7f06349512a63a912a832cbab0631e429))
* always send their Offline vote to relocated elders ([7f77e97](https://github.com/maidsafe/safe_network/commit/7f77e970adaf30c4653a60299829501e90453a4e))
* avoid invalidating signature when resending bounced Sync message ([d482dab](https://github.com/maidsafe/safe_network/commit/d482dab96b8e2bdd5d49aa1579b50bce8f459e64))
* botched conversion ([f681c24](https://github.com/maidsafe/safe_network/commit/f681c2422ac1e9c9e27121383fb1d50499683384))
* check for is_section not is_elder in msg_analysis ([fc9841b](https://github.com/maidsafe/safe_network/commit/fc9841b39ce62fabceed1d64c191bb0203ba6753))
* clippy ([d060aac](https://github.com/maidsafe/safe_network/commit/d060aac99ab1e4d209e43f5d3f67f2c3b4883aa3))
* clippy warnings ([3b667ef](https://github.com/maidsafe/safe_network/commit/3b667ef9e2ffe91d7c03d8af609e4e52d545ec52))
* doc tests ([e70db31](https://github.com/maidsafe/safe_network/commit/e70db31856d3951a6127ac46e73d8ae754bee112))
* doc tests ([deffab3](https://github.com/maidsafe/safe_network/commit/deffab301c2c18f02c8ce283ffac415cc5fca745))
* don't fail in update_state if secret key share is missing ([97d8266](https://github.com/maidsafe/safe_network/commit/97d8266042d1c21c02b8015aa5be38ad009c8224))
* double serialization bug ([7ef69a6](https://github.com/maidsafe/safe_network/commit/7ef69a6c224f3a5d25a366f0975acabecf83c662))
* enqueue elder ops while assuming elder duties ([88ed190](https://github.com/maidsafe/safe_network/commit/88ed19073620ce1882863f9323cccb0797ea84be))
* ignore elders update with incorrect prefix ([dfc9c60](https://github.com/maidsafe/safe_network/commit/dfc9c60278fe78fdd5fbb4de14b5cc2721dbf570))
* ignore invalid bootstrap response ([3d8cfd5](https://github.com/maidsafe/safe_network/commit/3d8cfd583c16cff7c25b82c142c80aa6348852e3))
* improve debug msg ([17daa0b](https://github.com/maidsafe/safe_network/commit/17daa0bca15a1d48a8869642969eca213196312e))
* NoOp when elder change has occured for various stages ([5fd50e2](https://github.com/maidsafe/safe_network/commit/5fd50e2951398fb4b9b6b88e06d95219073a52a1))
* post-rebase issues ([93578c7](https://github.com/maidsafe/safe_network/commit/93578c7b60fe8ec30dfbfd8d4371e8566284a9c1))
* redirect to our elders on mismatching GetSectionRequest as adult ([22c4745](https://github.com/maidsafe/safe_network/commit/22c47453a50d79b47771c9afb682d3cac88aeb12))
* remove offline elder from vote recipients ([3bcea21](https://github.com/maidsafe/safe_network/commit/3bcea21ceb00feacc52843cc435fe875d3ed3f84))
* remove println ([62b1c07](https://github.com/maidsafe/safe_network/commit/62b1c070fa295211cd565b78454e9417e93e80f4))
* remove unimplemented for match wildcard ([ea22dc8](https://github.com/maidsafe/safe_network/commit/ea22dc80365abee3e164531e18f40d194b30c0b6))
* save received transfer propagation ([e72ba81](https://github.com/maidsafe/safe_network/commit/e72ba8177866c53bc31964cf7e834e77665c36d4))
* unifies used space across node ([942c7f8](https://github.com/maidsafe/safe_network/commit/942c7f80f7002d95a33d88dfbdb4b143d43442e8))
* use msg id new fn for random id ([103beb5](https://github.com/maidsafe/safe_network/commit/103beb584fac08d9e58cd638c54206406eabe72a))
* **adult:** fix adults overwriting their blob chunkstore on churns ([f823ee8](https://github.com/maidsafe/safe_network/commit/f823ee85001a04a2d40054b79b98f0997f17b33e))
* **adult:** instantiate new adult state ([bd805f2](https://github.com/maidsafe/safe_network/commit/bd805f243e9e498ccd2a7bb951336a926f9f4ff2))
* **clippy:** remove clone (undetected in local clippy check) ([da6cbc7](https://github.com/maidsafe/safe_network/commit/da6cbc7af578649907202d74676caee1623af278))
* **clippy:** remove conversion to same type ([237d791](https://github.com/maidsafe/safe_network/commit/237d791b6d70bfc6c3166fc64685e892fa7ebded))
* **comm:** dont hold on to messages sent on a channel that is unused ([92856cd](https://github.com/maidsafe/safe_network/commit/92856cd8daf51af109405d1b9b58b7fa0a5f2d9c))
* **genesis:** use sn_transfer genesis ([f14b376](https://github.com/maidsafe/safe_network/commit/f14b376beea6f3ea6c8ed4f04624f6d65c29ed95))
* **hack:** connection lag via lowering qp2p timeouts ([e6e1375](https://github.com/maidsafe/safe_network/commit/e6e137585f4f6726123a63d94ed981d35614f4c1))
* **init:** process results at assuming duties ([e1a85d6](https://github.com/maidsafe/safe_network/commit/e1a85d603b8baac2c843245365b9c0537bde7811))
* **msganalysis:** expect validation from transfers ([a5f96fc](https://github.com/maidsafe/safe_network/commit/a5f96fc768e6366cc9ab51130c9cbaf41ea89981))
* **rewards:** return error when deactivation fails ([452a458](https://github.com/maidsafe/safe_network/commit/452a4582f8abd52193d1e515caa775f520a6786a))
* **stress test:** ignore InvalidSource errors when sending probes ([adabf82](https://github.com/maidsafe/safe_network/commit/adabf82f7da0f8b6669e8f28cc9fb7fca02f67b2))
* **test:** impl proper sig verification for test signing ([a7fd147](https://github.com/maidsafe/safe_network/commit/a7fd1479f33fe281625404a676c2a7ac285b2e6c))
* **used_space:** set the local used_space to zero instead of clearing ([1594b86](https://github.com/maidsafe/safe_network/commit/1594b8624228249862a6d70089298b3d49d1859a))
* remove unused method ([ae2453e](https://github.com/maidsafe/safe_network/commit/ae2453e4b91a6972218ac9a393865d7224b9ad5e))
* send messages with correct MessageKind byte ([6756b43](https://github.com/maidsafe/safe_network/commit/6756b43d969d26afe9305ee4ff2851c6e9193495))
* trust check failure of Sync message sent to non-elders ([5520c18](https://github.com/maidsafe/safe_network/commit/5520c182e1f3c29ce560d3fbb6e1e7e74324ac47))
* use keys not key indices to check whether elders changed ([a99a07f](https://github.com/maidsafe/safe_network/commit/a99a07f80b706d7fe84b5d970ee10999910db395))
* **blob:** fix blob msg accumulation ([4becc9d](https://github.com/maidsafe/safe_network/commit/4becc9defc54dbadabe8c297d61811e9a795bf9f))
* **blob:** fix verification of blob replication messages ([201f9e8](https://github.com/maidsafe/safe_network/commit/201f9e8046c0eefed14d974987bd8a2acd2a1d71))
* **blob:** short circuit blob query messaging ([4b39dc8](https://github.com/maidsafe/safe_network/commit/4b39dc87aafcb8172366303f29e6b5db66fd9161))
* **deps:** use correct deps ([ca66a89](https://github.com/maidsafe/safe_network/commit/ca66a89ec42f431dad57c9f6086ccff8ca4d5af3))
* **messagning:** fix msg wrapping at adults and elders ([0aa3b70](https://github.com/maidsafe/safe_network/commit/0aa3b708c9ae10f320bf2e86cebb5b14fca9b655))
* **msg_analysis:** accumulate node queries + resp ([9fc4363](https://github.com/maidsafe/safe_network/commit/9fc436365ceaa1f9d9c09e388d0d2fcca314d0ee))
* **msg_analysis:** remove incorrect accumulation ([e270455](https://github.com/maidsafe/safe_network/commit/e270455083894d3a5ab1cf3ff6453ebd03a47dcf))
* **rate_limit tests:** use u64 instead of f64.. ([56db5ab](https://github.com/maidsafe/safe_network/commit/56db5abbeedcf5bd0820bd2a18e5810f51c05225))
* **serialisation:** minor refactor and fix to Msgpack deserialisation logic ([d7c84e6](https://github.com/maidsafe/safe_network/commit/d7c84e6e1dd4f594613dac54ed2cc0ae0e958849))
* **sn_node:** set sn_node thread stack size ([435b50b](https://github.com/maidsafe/safe_network/commit/435b50bfd64526484d0f9d0e56d3263fa0266991))
* **storage:** fix storage calculation and improve logging ([77fb9f6](https://github.com/maidsafe/safe_network/commit/77fb9f667a10b3b092897a2cee142ceb96675fe4))
* **storage:** increase default maximum capacity ([8dfc35c](https://github.com/maidsafe/safe_network/commit/8dfc35c0c385b489b9482f46103b6c89347f2fd0))
* **stress test:** fix log to file and probe message destination ([c933605](https://github.com/maidsafe/safe_network/commit/c933605df7847b05afd5a0b497cc315381e99955))
* **stress test:** fix sent probe messages counter ([b9b7530](https://github.com/maidsafe/safe_network/commit/b9b7530fe383142541da5164e46455fe84287565))
* **verification:** verification of message_env checks underlying message, not itself ([ee90aef](https://github.com/maidsafe/safe_network/commit/ee90aef0c3164db4d57aba0022a6b82b941eec1b))
* avoid over relocation ([989529c](https://github.com/maidsafe/safe_network/commit/989529cafd1903e9009f4f66b1d111819d89be9c))
* bug in SectionChain::minimize ([0eef78e](https://github.com/maidsafe/safe_network/commit/0eef78e2f7d3a729e38c2421a54be58ff07ff5d4))
* check trust with all known keys, not just the src matching ones ([2c9a1b2](https://github.com/maidsafe/safe_network/commit/2c9a1b280cee471514f8254cd82cf19deb1383b5))
* choose longest history w/ simu payout responses ([d22aa01](https://github.com/maidsafe/safe_network/commit/d22aa0153518d9588b229234f634042baa4c22b4))
* CmdError handling for validation reqs ([a7a7ad4](https://github.com/maidsafe/safe_network/commit/a7a7ad4e732b2d6e8b6fa24397ada97c4593eb8d))
* compile + clippy errors ([d6a51a4](https://github.com/maidsafe/safe_network/commit/d6a51a44a157f256837e21db2fb2d21f87124194))
* config init ([2348a8d](https://github.com/maidsafe/safe_network/commit/2348a8dd64b8a07be0db2a3e66b0c728e1a6e082))
* consider also relocated current elders for elder candidates ([fffc946](https://github.com/maidsafe/safe_network/commit/fffc94696fc82b73711b48c1ba4d83d21e2dd09b))
* correctly handle section chain extend edge case ([cae05ba](https://github.com/maidsafe/safe_network/commit/cae05bab837e66e2a6f7754133f4937451e0bbe0))
* cover all cases of RelocatePromise handling ([5966d3d](https://github.com/maidsafe/safe_network/commit/5966d3db21045d7e56851bd22c2d46e9ebdf50bb))
* db format ([c79bda5](https://github.com/maidsafe/safe_network/commit/c79bda5fb68db5553c1110be71a6da6d19fd9876))
* disregard startup_relocation ([5117e30](https://github.com/maidsafe/safe_network/commit/5117e30b0b1b3d7dc2efdb0ce676559176a66728))
* do not accumulate node query ([7b3c0f0](https://github.com/maidsafe/safe_network/commit/7b3c0f0529a26aac5d3801d35ca381da9b6f1a15))
* do not require resource proof for relocated node + test ([667e1fb](https://github.com/maidsafe/safe_network/commit/667e1fb156aa1dfad68388f95082d87807898a3f))
* don't apply transfers to store if already seen. ([9f895ad](https://github.com/maidsafe/safe_network/commit/9f895ad22b9996844cde9e7552033812f45aec37))
* ensure section elders info is always signed with the last chain key ([82fad1a](https://github.com/maidsafe/safe_network/commit/82fad1aca7d55e5b83b3af3658ffeae5f3873581))
* Ensure to store TransferStore in lock ([5172011](https://github.com/maidsafe/safe_network/commit/51720117ac7723dd1354141f87218c439c1a8828))
* forward ResurceChallenge to the bootstrap task ([2552f06](https://github.com/maidsafe/safe_network/commit/2552f0631e32a8e442ef1291b537c1ef969bca6d))
* handle message send to self ([a1c26ff](https://github.com/maidsafe/safe_network/commit/a1c26ff62d2dfd0bab12e35b266ce46eee024b77))
* hex encode serialised key ([8bbc235](https://github.com/maidsafe/safe_network/commit/8bbc2352c46abd80ea4e047ab878ffa9fcd6806b))
* ignore Sync messages not for our section ([6d90fcf](https://github.com/maidsafe/safe_network/commit/6d90fcff2b2a1873b56915b8f3dc202b5394681b))
* issues pointed out in review comments ([d9a986e](https://github.com/maidsafe/safe_network/commit/d9a986e5f4df278dd87cf08cf6b77ab725a70455))
* logic errors, logging ([5b205c4](https://github.com/maidsafe/safe_network/commit/5b205c46a906edb2d2229416ae1a33a1a66bd0cd))
* make SectionChain::check_trust more strict ([8dcd021](https://github.com/maidsafe/safe_network/commit/8dcd0215ba4cb8d8d2d57c3782fbdd267a278065))
* make sure sibling section info is valid and trusted ([2044b11](https://github.com/maidsafe/safe_network/commit/2044b1106397071b581af8a64fae453d78f4ab3b))
* pre aggregation commit changes ([5a3014f](https://github.com/maidsafe/safe_network/commit/5a3014f3feac4165ae4780994d82e14f71a812ed))
* prevent creating Section with elders info signed with wrong key ([f0f839c](https://github.com/maidsafe/safe_network/commit/f0f839cb124c94fede41c9a21882e6b00c5743de))
* re-add disabled match branch ([4fe82ec](https://github.com/maidsafe/safe_network/commit/4fe82ec8f6edf01292e81e4c8feb5c97fc00f2d9))
* reimplement overwritten hex encode fix ([aa50061](https://github.com/maidsafe/safe_network/commit/aa50061efe35d2069a9ac4612513dd7d23a56a96))
* reject SectionInfo votes not voted by a participant ([c40dc12](https://github.com/maidsafe/safe_network/commit/c40dc12bb1f64f6b22695c0541479c2dbc26fd8f))
* responses require threshold again ([d86bce4](https://github.com/maidsafe/safe_network/commit/d86bce44b2f59a76942b50d50439f2322a450641))
* return Balance(0) when no db found ([99f7308](https://github.com/maidsafe/safe_network/commit/99f73087777498bbae3b2522e5f2c0cf993589d3))
* return empty vec when key's transfer db doesn't exist ([05fb09e](https://github.com/maidsafe/safe_network/commit/05fb09e85f89ad9cb5462b022d7f0e4d56b2a6f6))
* send Sync messages on split even when demoted ([5f42b78](https://github.com/maidsafe/safe_network/commit/5f42b78c4bcd68720399d9553512057d8b7d4d0d))
* use age assigned by section ([4db6351](https://github.com/maidsafe/safe_network/commit/4db63514d4b4f8fe226bc76d97ca33f0e646165a))
* use latest qp2p & so dont remove good connections from pool ([fa8fbd2](https://github.com/maidsafe/safe_network/commit/fa8fbd2573840533f23186f7b9764150863027d4))
* **all:** remove unused dependency and fix clippy issues ([4ed5a73](https://github.com/maidsafe/safe_network/commit/4ed5a73e3e43a2be96f0d12b58ec86d2094385fb))
* **blob:** fix blob flows and simplify the self_encryption process for ([423002e](https://github.com/maidsafe/safe_network/commit/423002edb99691f8b32b91d4441b1869644d92ff))
* **blob:** fix bug in blog storage ([d7dec70](https://github.com/maidsafe/safe_network/commit/d7dec705f63221faff2f538263eb87962291885f))
* **blob:** fix writing blob with always our PublicKey ([ae2aea4](https://github.com/maidsafe/safe_network/commit/ae2aea4150d7f33959e08dfb29626ec42ef2ed5c))
* **blob:** rebase atop latest master ([74a88dc](https://github.com/maidsafe/safe_network/commit/74a88dc513d8fb4a0c1f90f493e30fa9c89f9d61))
* **blob:** verify unpub blob owner ([0a4b5c7](https://github.com/maidsafe/safe_network/commit/0a4b5c748260b465015dd28c69901eca187cfaf1))
* **blob:** verify unpub blob owner ([36318be](https://github.com/maidsafe/safe_network/commit/36318be0b6e53e63cd98a7cf2fc59401563aac2d))
* **client response:** add missing await for message matching ([7019fa6](https://github.com/maidsafe/safe_network/commit/7019fa6ebea8447b4c1dd4ff82f2fd9ce1bd0e83))
* **clippy:** some refactors in tests to make clippy happy ([1bc59ca](https://github.com/maidsafe/safe_network/commit/1bc59caa038736d26cd22ee8eba2018ecdeaa8b2))
* **comm:** try to re-connect after previously failed send ([08d9410](https://github.com/maidsafe/safe_network/commit/08d9410b575cfb26f80cc3efe896a73da432f98d))
* **comms:** add flag to communicate with a Section as a Node ([d648ad3](https://github.com/maidsafe/safe_network/commit/d648ad3b712e88da6de00b10f3ed24412c62bd4e))
* **connections:** Dont fail when one elder cannot connect ([cf4db2c](https://github.com/maidsafe/safe_network/commit/cf4db2c465aade7ab45443758bd2ae0ebc2a5ed9))
* **data:** verify owner before writing/deleting new data ([88addf9](https://github.com/maidsafe/safe_network/commit/88addf9e70888afaf937c8a06e17d548b500a06e))
* **data_types:** update to new MsgEnvelope ([4d53f14](https://github.com/maidsafe/safe_network/commit/4d53f147c026015fe5c6092695edf75f38b46378))
* **dependencies:** update bls_signature_aggregator ([6688efd](https://github.com/maidsafe/safe_network/commit/6688efd922b4c81d101dbbf53993678bf92b6e46))
* **dirs:** replace dirs_next with directories to set project paths ([d636426](https://github.com/maidsafe/safe_network/commit/d636426927c7f20e726abf14ee7bbdfb41292ab4))
* **dkg:** backlog messages with unknown DKG key ([03873c1](https://github.com/maidsafe/safe_network/commit/03873c11224d26bf587a4b3366d51e6847b91f06))
* **dkg:** handle delayed DKG outcome ([c58611b](https://github.com/maidsafe/safe_network/commit/c58611b5bc8343bffef08f3a5464bed3109380f8))
* **dkg:** handle DKG with single participant ([00c2efa](https://github.com/maidsafe/safe_network/commit/00c2efa6fb042a2e97008713f10a28e9b27a62e7))
* **docs:** update docs to reflect recent changes ([ae5c63a](https://github.com/maidsafe/safe_network/commit/ae5c63ac59b9c92c766cd3e429829da01fb1dad6))
* **duplication:** fix message parsing for chunk duplication at adults ([5ea395f](https://github.com/maidsafe/safe_network/commit/5ea395ff1b63e8f08be92e76f84f355117f37d45))
* **events:** fix adult promotion process ([015a013](https://github.com/maidsafe/safe_network/commit/015a0134e534c44336fdb57e704ddbadf0cb596c))
* **msg_analysis:** various bugs ([aabaeec](https://github.com/maidsafe/safe_network/commit/aabaeec2c0e6d772497a8419953f94c0e7575f56))
* **msg_sending:** use correct ids and addresses ([858722a](https://github.com/maidsafe/safe_network/commit/858722a74eb1ea0de328cfcc5b60adddf8dc0c6c))
* **promotion:** update to latest routing and fix promoting node to adult ([5528b09](https://github.com/maidsafe/safe_network/commit/5528b098751391a540bc7673c5c5c0687ca4b43e))
* **responses:** Remove unneeded shortcircuit and clarify logs in repsonse handling ([a86bbcd](https://github.com/maidsafe/safe_network/commit/a86bbcda6517a25b2080696b0890cf826d86fd65))
* **sn_node:** set sn_node thread stack size ([9a42cd9](https://github.com/maidsafe/safe_network/commit/9a42cd9e829551a643e93a0616e03a2913b23db4))
* **test:** account for relocations in test_startup_section_bootstrapping ([53196a5](https://github.com/maidsafe/safe_network/commit/53196a5ef8a82a073383f56f1f58ac84dbf28b9f))
* **test:** dont assert new joining node is not instantly relocated ([9a18b4c](https://github.com/maidsafe/safe_network/commit/9a18b4c0230142dda3a3c64a5fd8aaa0c67fc3b6))
* **tests:** config obj expected size ([c44c137](https://github.com/maidsafe/safe_network/commit/c44c137cebb81818dfa16a5e110f44561df40b31))
* **tests:** make tests compile and run ([c8b6037](https://github.com/maidsafe/safe_network/commit/c8b60370e3b03b152f85bd6847e3093be1633057))
* **tests:** remove unnecessary assertion on size ([26b21ad](https://github.com/maidsafe/safe_network/commit/26b21ad9893cc4b45407726f471a7c22e2a44102))
* **transfers:** fix genesis, sigs and store keys ([194a9a3](https://github.com/maidsafe/safe_network/commit/194a9a317b0ed0880ba74f136a3e3898db7a949c))
* clippy warnings ([24145f5](https://github.com/maidsafe/safe_network/commit/24145f5cf28616b4ca1f38604b614ed7c17e368f))
* pre-reserve space in case of write fail ([f040acd](https://github.com/maidsafe/safe_network/commit/f040acdd3ee6269fe223bc7b7c808a6e4de1181c))
* temp convert name + top lvl err handle method ([8b415c7](https://github.com/maidsafe/safe_network/commit/8b415c78bf4d9a30a979b36a062ff27b45aa596c))
* **connections:** retry failed connections ([f14ebe6](https://github.com/maidsafe/safe_network/commit/f14ebe6b6bee0e035bb0e485753cc8810ce54c53))
* **docs:** Update duty config docs. ([40c4765](https://github.com/maidsafe/safe_network/commit/40c47652b74b9de6a8619f7dee37b849768644e2))
* **event:** export qp2p SendStream and RecvStream for consumers to use ([65af16f](https://github.com/maidsafe/safe_network/commit/65af16fd62055999460dd7aeec91b2e0eaab6c68))
* **genesis:** pass in "ghost" source key ([1f582ea](https://github.com/maidsafe/safe_network/commit/1f582eaf8b27f405fba25480a90d444e8114341f))
* **process_while_any:** don't drop any from `ops` ([a992f5f](https://github.com/maidsafe/safe_network/commit/a992f5f078bbb41e5b6e9651a3f20c73d8b51897))
* **proxy_handling:** fix proxy_handling for section-to-section messaging ([1543014](https://github.com/maidsafe/safe_network/commit/154301424427bb430680abbb9bc5a720138d667b))
* **seq:** sign op.crdt_op ([a584ef0](https://github.com/maidsafe/safe_network/commit/a584ef0c3ed672cce2cfc5bfee980c681d00d0f6))
* prevent losing incoming messages during bootstrapping ([3c9357e](https://github.com/maidsafe/safe_network/commit/3c9357e9cc9d77d5da35df5fb856b08f3ac674b3))
* remove duplicate cargo.lock version line ([e749a5f](https://github.com/maidsafe/safe_network/commit/e749a5f2dc342e0ad484607d85d719fb4cbbe939))
* remove leftover GetReplicaKeys refs ([ecde8bc](https://github.com/maidsafe/safe_network/commit/ecde8bc2d2c1b078579026f01d472feb5f32fe36))
* remove redundant origin field ([21bf1cb](https://github.com/maidsafe/safe_network/commit/21bf1cb88531f5268a0808ae558fb0609aa784e2))
* remove unnecessary error mapping ([0f3418b](https://github.com/maidsafe/safe_network/commit/0f3418b2ea4d66f438604ded2682d76f95e70d6f))
* remove use of wildcard match and unimplemented macro ([84c53d8](https://github.com/maidsafe/safe_network/commit/84c53d8db16f1ab237c46ad5e8221b2a80758d54))
* resolve a doc failure ([d51f0c6](https://github.com/maidsafe/safe_network/commit/d51f0c62534fe03add884bdb060105c1bd7c394b))
* respond with GetSectionResponse::Redirect on missing pk set ([69a1fb8](https://github.com/maidsafe/safe_network/commit/69a1fb840cbbb54b8ccb5af8856e3991d3ac46dd))
* threshold and error text ([11151d8](https://github.com/maidsafe/safe_network/commit/11151d8b448f3dede5e52cb9dd7b3f674cc348dd))
* update message bytes directly for dest change ([d253690](https://github.com/maidsafe/safe_network/commit/d2536909c25f1981a31d47eea9cd8016ed5a012a))
* use chain main branch length as the DKG generation ([ed3a54e](https://github.com/maidsafe/safe_network/commit/ed3a54e635661f6bb59d968a8a4c3d091f2a8587))
* **build:** fix conflicts after rebase, remove deprecated API use ([d7ae205](https://github.com/maidsafe/safe_network/commit/d7ae20597666be98a90cef253e721dbff5661df4))
* **ci:** fix coveralls failure in CI ([c92a6cc](https://github.com/maidsafe/safe_network/commit/c92a6cc58ef8fe5eeda044b2723a78172888f5a9))
* **ci:** fix dependency graph script ([0e178e2](https://github.com/maidsafe/safe_network/commit/0e178e267a726a9d293dcd8b6a8e777dc9a3e8db))
* **clippy:** Clippy enum fixes ([0554b4f](https://github.com/maidsafe/safe_network/commit/0554b4f8b86867a2e41fdf02b2b2452b4d8d1149))
* **clippy:** fix last clippy warnings ([83b64ab](https://github.com/maidsafe/safe_network/commit/83b64ab4dfe52951f402d64d4dc7cd5e107bc618))
* **clippy:** fix warnings after clippy update ([f2e25c2](https://github.com/maidsafe/safe_network/commit/f2e25c2c746b0bd1073f662cc7c4492af9a8f9b1))
* **clippy:** some clippy fixes (not all) ([4d0cba1](https://github.com/maidsafe/safe_network/commit/4d0cba1d03be051cd7c2a9bda34202846ffc1543))
* **config:** put correct wallet test value ([16ef078](https://github.com/maidsafe/safe_network/commit/16ef078cef0fa387ef3730400de7d720da1bc40c))
* **config:** reenable writing to disk ([79f59b5](https://github.com/maidsafe/safe_network/commit/79f59b503c90c5d5414b8a7271cf75d39ab9bd85))
* **config_file:** remove remaining occurrences of clear and fresh ([124ed70](https://github.com/maidsafe/safe_network/commit/124ed70f98cab343455348eb894f64df356bfc5c))
* **dependencies:** update temp dependency switch ([bc18408](https://github.com/maidsafe/safe_network/commit/bc18408f1668dd1d3673ca9831a3ed1ea651cdd7))
* **dkg:** detect corrupted DKG outcome ([ec53c63](https://github.com/maidsafe/safe_network/commit/ec53c63a78e5cf776219b75cb2c678710f9b34ae))
* **dst-accumulation:** verify aggregated signature with proof chain ([bd99595](https://github.com/maidsafe/safe_network/commit/bd99595379307f0f6b19bccaac0b3b8e145e0fcf))
* **from_db_key:** missing option wrap ([fc489f5](https://github.com/maidsafe/safe_network/commit/fc489f5e7d8f80293cff82b1ac2408407fd6a794))
* **gateway:** add missing client event processing ([7ab3b17](https://github.com/maidsafe/safe_network/commit/7ab3b175739d8bb0db9bf85f204f95973ebfb226))
* **gateway:** process transfer msgs ([21dad58](https://github.com/maidsafe/safe_network/commit/21dad58a0b32119d333c4e40277139c18cb4cdd1))
* **gateway:** votefor process locally, not forward ([2016df6](https://github.com/maidsafe/safe_network/commit/2016df6f2538ce5b271db7dbf415f65ed47ba32b))
* **minting:** velocity < 1 at < 50% supply ([e507ce5](https://github.com/maidsafe/safe_network/commit/e507ce58a655ef13246cb1de291645245f52eb46))
* **minting_velocity:** don't stop at 50% minted ([578c431](https://github.com/maidsafe/safe_network/commit/578c43166b4fc01ab094121e6b11f2c0a70d6176))
* **msg_analysis:** try all match methods for a msg ([fcadb77](https://github.com/maidsafe/safe_network/commit/fcadb773d879200c313c224471e073436cbe3334))
* **msganalysis:** correctly identify msg to client ([f111567](https://github.com/maidsafe/safe_network/commit/f111567ecac260d2763984135903efbac0b8d50b))
* **msgs:** fix random msg id generation ([624a5b0](https://github.com/maidsafe/safe_network/commit/624a5b058d4930f9e417ef33e603373e715d7378))
* **msgs:** updates to use qp2p streams ([814668b](https://github.com/maidsafe/safe_network/commit/814668b0d1102b410d15b33eae51303f2fdbbdd2))
* **new:** Fix simulated payout dot and related tests ([a795d4a](https://github.com/maidsafe/safe_network/commit/a795d4a02fd6c2258534f5b635b8d88a7793f2b9))
* **node:** create vault's root directory before writing to it ([513cfc1](https://github.com/maidsafe/safe_network/commit/513cfc1bead7c50c28579ec40ba046dc59347d3c))
* **node:** use node keypairs generated locally WIP ([4c520b5](https://github.com/maidsafe/safe_network/commit/4c520b56ffee9213224275a0ccd7abff3c1e2c0f))
* **node_ops:** add none to break infinite loop ([2dcc7f1](https://github.com/maidsafe/safe_network/commit/2dcc7f15e279cfe1095b0f61db433a92e3ca4dfd))
* **nodeduties:** set index of bls share ([8b85082](https://github.com/maidsafe/safe_network/commit/8b85082ec730eea676ac1ccc1809f03d5be3fb09))
* **onbarding:** only check clients w contains qry ([045d3dd](https://github.com/maidsafe/safe_network/commit/045d3ddae7453a62583fa89552cb41706ff419b1))
* **onboarding:** check if already accepted on ([eae22b3](https://github.com/maidsafe/safe_network/commit/eae22b384ea5135ec1d4a2f88a22ed8dbc80c088))
* **onboarding:** faulty elder address retreival ([eb38b78](https://github.com/maidsafe/safe_network/commit/eb38b7804d5fba057c5a88dbe215c48ab1258d0b))
* **onboarding:** idempotency check of bootstrap ([48c561a](https://github.com/maidsafe/safe_network/commit/48c561a1112a00b073d9c9b91582d49d156f0b4a))
* **onboarding:** return same challenge on repeated ([bf33bff](https://github.com/maidsafe/safe_network/commit/bf33bff27fd7d28f4ab777998c518bd70f090711))
* **qp2p:** Fixes for latest qp2p master ([0a5001b](https://github.com/maidsafe/safe_network/commit/0a5001b2fa21e22513a37621e7f35636fe6d840e))
* **rate_limit:** query network for all adults ([f428f17](https://github.com/maidsafe/safe_network/commit/f428f175ed33f87f88f90d9a382ba9aeb81e27e4))
* **replica_init:** clear init flag also when first ([d1765ca](https://github.com/maidsafe/safe_network/commit/d1765cabad62f0baf8528c88c85d338b28b13073))
* **replica_init:** have genesis node init replica ([cb61ef3](https://github.com/maidsafe/safe_network/commit/cb61ef35695f74f8fea909a974c55986150ec349))
* **reward_cfg:** register on connected to network ([a1e976f](https://github.com/maidsafe/safe_network/commit/a1e976f7f16c4173844e2e36803bbe98403ef06a))
* **routing:** remove unused is_genesis func ([6407959](https://github.com/maidsafe/safe_network/commit/6407959f80f1abc8aad98b524d86981cec3312c3))
* **storecost:** div section balan by allnodes sqrd ([74814d3](https://github.com/maidsafe/safe_network/commit/74814d3f87f2ed7606e2cf2bc8b44fd93d45c009))
* **stress-test:** fix probe message sending ([a8a184c](https://github.com/maidsafe/safe_network/commit/a8a184c70f57801140d4fb521b230485ab353727))
* **test:** don't hide exported tests behind #[cfg(test)] ([40d0766](https://github.com/maidsafe/safe_network/commit/40d0766efa2e57013c117e565c01f149dc455d4d))
* **test:** final fixes for test suite ([2ab562b](https://github.com/maidsafe/safe_network/commit/2ab562b6730193d96bfa45925d20c852757e8e4e))
* **test:** update name and assert correct value ([d929c8f](https://github.com/maidsafe/safe_network/commit/d929c8fc3d7286bf62933ba52175edc157094f6b))
* **tests:** add missing calls to start_network ([57751bd](https://github.com/maidsafe/safe_network/commit/57751bdb43f7ec51c144cf453bf14580d415e248))
* **tests:** fix actor and Money transfer tests ([ad67e08](https://github.com/maidsafe/safe_network/commit/ad67e08ebdb981c9558d6b37c39503641defdbd0))
* **tests:** update references to scl ([1efc59b](https://github.com/maidsafe/safe_network/commit/1efc59be105a0fc8097b34df9b94502c6263cf43))
* **transfer store:** Check for lists existence. ([618d33d](https://github.com/maidsafe/safe_network/commit/618d33d6ec69186ede6626b1f3c2ba140fbd8add))
* **transfers:** fix sending dst on registering a transfers ([1fccf16](https://github.com/maidsafe/safe_network/commit/1fccf160942b02621642013003e1f62d566fa596))
* **transfers:** send to client ([c1f5b52](https://github.com/maidsafe/safe_network/commit/c1f5b524de7e4ae825984c1f620caee1be7eb6df))
* add testing feature flag again ([21412e0](https://github.com/maidsafe/safe_network/commit/21412e09563daca70585d731ef8cdec9d941ab01))
* add visibility modifiers ([4d335a8](https://github.com/maidsafe/safe_network/commit/4d335a8dcf2cf8ac02be52ec3f08e0872849694a))
* bounce DKG message only if node has no ongoing session ([350b75d](https://github.com/maidsafe/safe_network/commit/350b75db30fbdec86e14d48ff4f1740be39ddc00))
* clear peer_mgr candidate post pfx change. ([57cd490](https://github.com/maidsafe/safe_network/commit/57cd490069c961098e3a242fcf439ab2f1631bb5))
* disable one missing validation of duplication ([2ecc390](https://github.com/maidsafe/safe_network/commit/2ecc3903f617fbaad9fd351442e7f78521463ebb))
* don't ack hop messages in Client state ([9539c05](https://github.com/maidsafe/safe_network/commit/9539c05f3133a487dd5f0806418283a880eb411e))
* expand ConnInfoReq handling conditions. ([d081800](https://github.com/maidsafe/safe_network/commit/d0818004f90d5f67e5d03f974967ba8829ae2a6a))
* handle invalid bootstrap response by retuning error ([d5ee338](https://github.com/maidsafe/safe_network/commit/d5ee338bf79c21d7e136bd8becb84d49fd3a2997))
* lost peer handling ([1d95194](https://github.com/maidsafe/safe_network/commit/1d95194f7a074d0561a4199cf106cca541af70f4))
* no longer use serde macro derive ([2116420](https://github.com/maidsafe/safe_network/commit/2116420e2d205499c3c030acafa036df73c9664c))
* remove non-existing field ([aeee3b8](https://github.com/maidsafe/safe_network/commit/aeee3b82f9cde660f62d1cd2ac914f1fd407f503))
* Remove old compatible neighbour pfx not restricted to a strict parent/child prefix in Chain on updating neighbour_infos. ([#1579](https://github.com/maidsafe/safe_network/issues/1579)) ([6d23fa3](https://github.com/maidsafe/safe_network/commit/6d23fa3390cac5462988ac069e93ad5199dcc57f))
* rename mock/quick_p2p to mock/quic_p2p ([067fab0](https://github.com/maidsafe/safe_network/commit/067fab09f2e2dcf185dd8bd5987bf8c99c88029d))
* resolve clippy errors of non-mock tests ([94eda60](https://github.com/maidsafe/safe_network/commit/94eda60e3eae1fd033903038e4271a955c729112))
* resolve failing example ([121ce95](https://github.com/maidsafe/safe_network/commit/121ce952993ad7e29e055d27b33f164331cd9252))
* send Event::Connected only after transition to Approved ([dbe0593](https://github.com/maidsafe/safe_network/commit/dbe059361876c09f00323b7eb7fd8d95bcb151ee))
* take ages into account when calculating DkgKey ([824d229](https://github.com/maidsafe/safe_network/commit/824d2293f17e3d64a6282544556d0ffec3d5e744))
* use the latest section key when updating non-elders ([219f98d](https://github.com/maidsafe/safe_network/commit/219f98d9b3e1a51e5c7eb32fd3857a5de592081f))
* **tests:** add RUST_FLAGS -D to test scripts ([83e12e4](https://github.com/maidsafe/safe_network/commit/83e12e4a857be7c48a1d12d71a59b7ad2ea5c21a))
* **transfers:** get history requests now return history. ([7590bd0](https://github.com/maidsafe/safe_network/commit/7590bd0ef746f74af60a92859be1cd06c5e8457b))
* **transfers:** xpect client as most recent sender ([61593e4](https://github.com/maidsafe/safe_network/commit/61593e4b0cc43972571deb742f39211f5dca7ce3))
* **wallet:** lock over the db on write ([a6f5127](https://github.com/maidsafe/safe_network/commit/a6f5127f0130c56fdac4ce0429ff3ebedbae5995))
* **walletstage:** actually add the signatures ([12cc467](https://github.com/maidsafe/safe_network/commit/12cc4673b0d77fb10db371a7fcba54a94f365460))
* vote for sibling knowledge after parsec reset ([090663f](https://github.com/maidsafe/safe_network/commit/090663f24dcb165b98d0ccb16b1f5d32614f3b91))


### update

* **deps:** update to the latest version of sn_messaging ([4882ad0](https://github.com/maidsafe/safe_network/commit/4882ad0986c186d7c7d539b2fb1fb9f5fe73dce2))


### api

* AE work ([3bb0c88](https://github.com/maidsafe/safe_network/commit/3bb0c88bbf789bf43998c709098ec5205ebb03bf))
* for aggregate_at_src message, notify sn_node with proof as well ([8a39aaa](https://github.com/maidsafe/safe_network/commit/8a39aaa936ea6e478bb1d96bd49ca390d62297c0))
* re-enable aggregate at source ([2074770](https://github.com/maidsafe/safe_network/commit/207477079622c6fdc9e0ee44ba58d694bf9b3fe6))
* remove NetworkParams ([686c248](https://github.com/maidsafe/safe_network/commit/686c2482358e03b94779c0cde9a61af2b83b6575))
* remove size fields within routing::Config ([9dfb935](https://github.com/maidsafe/safe_network/commit/9dfb935afd9bdfe4dcc65d37e1cdbb93ac21fa06))
* rename Proven to SectionSigned, MemberInfo to NodeState, PeerState ([08d6929](https://github.com/maidsafe/safe_network/commit/08d69293713cb8caea4449dbca4d119961403b92))
* SAP refactor ([402fd3f](https://github.com/maidsafe/safe_network/commit/402fd3fd8fa1cf9721135f17c1ccb5ed35f6f081))


* **chunk:** rename blob to chunk ([0249c7d](https://github.com/maidsafe/safe_network/commit/0249c7d4f8bb56d756651eff4081088f186844a5))
* rename `dest` to `dst` to align with `src` ([26f4af3](https://github.com/maidsafe/safe_network/commit/26f4af3df13b7dee79ed3519066766500cbd5430))
* **adult_ack:** use ChunkOpHandled in NodeEvent ([9b1ba47](https://github.com/maidsafe/safe_network/commit/9b1ba477a5f951ab9a1fe18a9906773478e2e231))
* **all:** refactor and remove messages no longer needed ([1b15dd6](https://github.com/maidsafe/safe_network/commit/1b15dd66d49b7197c5b00e92105f778ef5029855))
* **api:** return only SectionAuthorityProvider for matching_section ([eaea2bc](https://github.com/maidsafe/safe_network/commit/eaea2bcedfe5a5425acc4b266ab75e45a780c268))
* **api-usage:** use Routing::matching_section() API to get our ([c37fe53](https://github.com/maidsafe/safe_network/commit/c37fe538fb524990f05d8fa9d74dc8b9715491dd))
* **chunk-replication:** allow Adults to replicate chunks by ([2e7537c](https://github.com/maidsafe/safe_network/commit/2e7537c7131e1d62f1bc6f6d4f115e8d84b1855b))
* **chunks:** turn around the replication flow ([cc1e5e8](https://github.com/maidsafe/safe_network/commit/cc1e5e8901cf10c89c2fc0db31adc51e6968d4ee))
* **data_sync:** follow up PR to the data loss PR ([2cb863f](https://github.com/maidsafe/safe_network/commit/2cb863f271ef33655ebe752b6156f2cd40d2d74e))
* **deps:** update sn_messaging ([76e733b](https://github.com/maidsafe/safe_network/commit/76e733b627901c207a7f3c955cf9bd467b678873))
* **deps:** update sn_messaging ([c7c4108](https://github.com/maidsafe/safe_network/commit/c7c410895fb95f561a5e207017d2cacc9b25a3ef))
* **deps:** update sn_messaging and sn_routing ([0913819](https://github.com/maidsafe/safe_network/commit/09138195c2ca3f5962a351bfbaa0268d07ac2132))
* **deps:** update sn_messaging to 0.19.0 ([c79313d](https://github.com/maidsafe/safe_network/commit/c79313d69406abec71290266cb63fd01cb70575f))
* **deps:** update sn_messaging to 19.0.1 and sn_data_types to ([91709e5](https://github.com/maidsafe/safe_network/commit/91709e555c9747629d3cacc3b1b9e91246b244e7))
* **deps:** update sn_messaging to v20.0.0 ([2417d53](https://github.com/maidsafe/safe_network/commit/2417d5338244d6ad76865c0dc670875efff5cd12))
* **deps:** update sn_messaging to v29 ([b360807](https://github.com/maidsafe/safe_network/commit/b3608074dba4a931111a6cbb53184a7cd86f7b5b))
* **join:** re-organising all messages related to node joining flow ([2a94692](https://github.com/maidsafe/safe_network/commit/2a94692e03d5bcfe04f700cf6b7f0090af5cdbaf))
* **relocate:** simplifying relocation flow and logic ([d06c7b0](https://github.com/maidsafe/safe_network/commit/d06c7b0bb384e85da651c70ab3122f47298e8426))
* changes to upgrade sn_messaging to v34.0 ([3d405a7](https://github.com/maidsafe/safe_network/commit/3d405a73bf555e0d9aef32ae71c3cd92d322e52a))
* **bootstrap:** changes to new messaging flow for peers joining the network ([0ebb7c0](https://github.com/maidsafe/safe_network/commit/0ebb7c086f42712a219c920e560646837d0ee579))
* **deps:** update sn_messaging version to 20.0.1 ([4c8249d](https://github.com/maidsafe/safe_network/commit/4c8249d22e5cbd2b424dc76bed833656cf39915e))
* **event:** add SectionSplit, increase granularity ([4766067](https://github.com/maidsafe/safe_network/commit/47660678f765268cc32bf2d44cb427deaab42486))
* **messages:** add GetSectionResponse::NotAllowed variant ([846e39e](https://github.com/maidsafe/safe_network/commit/846e39edc1beef57cd39d563d2ed86b17791235c))
* **section_info:** use SectionAuthorityProvider for SectionInfo messages payload ([5436fbf](https://github.com/maidsafe/safe_network/commit/5436fbfe545bbc5f723db93b82291ace36cdcca5))
* removing unnecessary message definitions ([6997fc9](https://github.com/maidsafe/safe_network/commit/6997fc9d3fa5a3e7b4fd0cca50d6dd5100282b67))
* upgrading sn_messaging to v24.0.0 ([81907b5](https://github.com/maidsafe/safe_network/commit/81907b508db7daa9c61aaf68cb326db706098a40))
* upgrading sn_messaging to v25.0.0 ([7acb16a](https://github.com/maidsafe/safe_network/commit/7acb16addfb921f8997a134e15cf26e7e2907dd9))
* upgrading sn_routing to v74.0 ([c501650](https://github.com/maidsafe/safe_network/commit/c501650c4a3c730f4614aa2336694ec5026eb417))
* **deps:** cargo update ([ccc5e19](https://github.com/maidsafe/safe_network/commit/ccc5e191a5671659583e25c172876fd69192a620))
* **deps:** update sn_messaging ([ecc376d](https://github.com/maidsafe/safe_network/commit/ecc376d0199cdfa6191acfe943fe01ec67f2df91))
* **deps:** update sn_messaging ([1b1fdf7](https://github.com/maidsafe/safe_network/commit/1b1fdf7756bb287bd5c2b4c7637febf6a03e5a58))
* **err:** rename NoSuchData to DataNotFound and add the address ([463c6c7](https://github.com/maidsafe/safe_network/commit/463c6c7235e16aa5e52224ad7371a4c1b74f002a))
* **err:** return chunk address along with DataNotFound error ([74e2b3e](https://github.com/maidsafe/safe_network/commit/74e2b3e73ab1a3100c4794ea048d66bd416883a1))
* **msgs:** move all routing message definitions out to sn_messaging ([2259b7b](https://github.com/maidsafe/safe_network/commit/2259b7b7b120c92f3f8e441ddd694dc5b2d43386))
* breaking version change ([0fb090c](https://github.com/maidsafe/safe_network/commit/0fb090cd0136661adad0b6e7f37bba5ae4858a87))
* minor refactor to adapt to new sn_messaging v24.0.0 ([fb7ae79](https://github.com/maidsafe/safe_network/commit/fb7ae79a19e6a39dddcd2767cdaef63210c5e535))
* minor refactor to adapt to new sn_messaging v25.0.0 ([082b50e](https://github.com/maidsafe/safe_network/commit/082b50e4d493829b76e15b8ba9a5a0debdfd4569))
* **config:** refactor command line arguments to improve user ([11850c8](https://github.com/maidsafe/safe_network/commit/11850c8b4a9607ed8d9b07b6758aa84cb4678966))
* **dep:** update qp2p dependency ([3efb8c5](https://github.com/maidsafe/safe_network/commit/3efb8c54906397a5dd676cfb835eb22e3d453e40))
* **deps:** update qp2p version ([c91c555](https://github.com/maidsafe/safe_network/commit/c91c555a3fe3e4a2faf543134aa1ee322fbde158))
* **deps:** update sn_data_types ([2597df9](https://github.com/maidsafe/safe_network/commit/2597df926eef60298314d79764f5d0153c4225b6))
* **deps:** update sn_messaging ([d75c343](https://github.com/maidsafe/safe_network/commit/d75c343099e2c864bcc54aaaa73f53d639dc2ae7))
* **deps:** update sn_messaging ([8d61421](https://github.com/maidsafe/safe_network/commit/8d61421e1d8b92c0d52d2bdb964bee4095b70084))
* **deps:** update sn_routing ([ddaa1ce](https://github.com/maidsafe/safe_network/commit/ddaa1ce618ad2fd8f7ce76a49d01196011f4aa23))
* **messaging:** remove the RegisterEndUser messaging handling and flows ([fa88047](https://github.com/maidsafe/safe_network/commit/fa88047e9e53b244905963d1ab09e5900a5c0b1e))
* **section_info:** remove RegisterEndUser related messages ([da66f58](https://github.com/maidsafe/safe_network/commit/da66f58262f476e8732ea6955d60dee9737c618b))
* DT dep update ([7fb8a4a](https://github.com/maidsafe/safe_network/commit/7fb8a4a6ebed7e1990de4acdd38feca89cb52d1a))
* DT dep update ([ded2602](https://github.com/maidsafe/safe_network/commit/ded260297119a6025b9dcac92889ca3ebf09afc5))
* DT dep update ([b82b223](https://github.com/maidsafe/safe_network/commit/b82b2237fe67cc72d294d94f9cb61f31c2ee6ef6))
* fix clippy errors with version 1.50.0 of rust ([b6b385a](https://github.com/maidsafe/safe_network/commit/b6b385aa1d05a8ac908f568d4537bb64589cd470))
* remove unused scheme variant ([27c7d86](https://github.com/maidsafe/safe_network/commit/27c7d86ebce2a734183aa0136478ea6f13c7a511))
* Revert release "10.0.0" ([158e3a3](https://github.com/maidsafe/safe_network/commit/158e3a312b4461d1d6a23aaefd3e426a17c6b503))
* dep updates and changes for split ([10a291d](https://github.com/maidsafe/safe_network/commit/10a291d66856b914e6738d5e2e3d87374858ac82))
* messaging dep updates for ProcessMsg ([7935639](https://github.com/maidsafe/safe_network/commit/79356390e8640fe881fedfb56c4a5403c7cc6b8f))
* **data-types:** upgrading data-types to v0.16.0 and sn_messaging to v8.0.0 ([5e39755](https://github.com/maidsafe/safe_network/commit/5e397559e7f4b907276f2a2f689cb519d304b8be))
* **deps:** update sn_data_types ([555e4fb](https://github.com/maidsafe/safe_network/commit/555e4fbb3222ba0a46fd189c9c62bfd8052d9d19))
* **deps:** update sn_messaging, sn_data_types ([367b673](https://github.com/maidsafe/safe_network/commit/367b6731b90b7211679282b2fcaa8852f3449ccd))
* **deps:** update sn_routing, sn_messaging, sn_transfers ([2916764](https://github.com/maidsafe/safe_network/commit/291676482aa2f33b85732183438a13a6acec224a))
* **deps:** update sn_transfers, sn_messaging, sn_data_types ([4b5d876](https://github.com/maidsafe/safe_network/commit/4b5d876aea68f6252c100f13c6766ea38e67d2d4))
* **messaging:** add expected aggregation scheme, and use an itinerary ([6d3d970](https://github.com/maidsafe/safe_network/commit/6d3d97025332a522bc0e2b0b94945406a358d7e0))
* **messaging:** add expected aggregation scheme, and use an itinerary ([a79d2d0](https://github.com/maidsafe/safe_network/commit/a79d2d0f837354c46282410d387b2276af525848))
* **routing:** upgrading sn_routing to 0.48.1 ([8659be7](https://github.com/maidsafe/safe_network/commit/8659be7ca580b5a62a0e0bd4c5f701cf51e244da))
* **tokio:** upgrade qp2p to v0.10.1 and tokio to v1.3.0 ([07ce604](https://github.com/maidsafe/safe_network/commit/07ce6045f371b3cdef7c8f23c027b304b506cb2a))
* **tokio:** upgrade tokio to v1.2.0 and qp2p 0.10.0 ([e5adc1a](https://github.com/maidsafe/safe_network/commit/e5adc1a6e21c4b7f3aa62497535b7740cd08a3f3))
* **tokio:** upgrade tokio to v1.3.0 ([ffb74f9](https://github.com/maidsafe/safe_network/commit/ffb74f9976172d49b92b42f51c1eaef6129e391f))
* **version:** remove commented code ([8a2b058](https://github.com/maidsafe/safe_network/commit/8a2b058260df807a66ddaf2d649cb5bd145cdbf9))
* remove `Error::UntrustedMessage` ([dbcf0db](https://github.com/maidsafe/safe_network/commit/dbcf0db471f2b234342fdeeb68d5cf7aaff50846))
* remove Error::BadLocation ([3391c7f](https://github.com/maidsafe/safe_network/commit/3391c7f1d49e050ae2fe580816a10add68388d14))
* remove the Routing state machine ([cfa19ff](https://github.com/maidsafe/safe_network/commit/cfa19ff2151976996d425a3a10e863b03abf6331))
* rename Instance to Routing ([a227e3f](https://github.com/maidsafe/safe_network/commit/a227e3fe03894545956c7899d8b120b375065281))
* rename money to token ([e3d699c](https://github.com/maidsafe/safe_network/commit/e3d699cce291f9172b79d698cc7edeb3845690ab))
* rename money to token ([62f816a](https://github.com/maidsafe/safe_network/commit/62f816a5552b09822745c7f50b4d9b9c73824aca))
* rename money to token ([eb53ef5](https://github.com/maidsafe/safe_network/commit/eb53ef577da48c9850e8997fcb91ebc6ae9fefd2))
* rename Node to Instance and NodeConfig to Config ([d8d6314](https://github.com/maidsafe/safe_network/commit/d8d63149fce5742af1d2151b91ee974c24ada269))
