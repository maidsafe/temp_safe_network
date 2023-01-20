# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.15.0 (2023-01-20)

<csr-id-5e81ac4fb8a2312eb546a4b86e71be05df7c4e26/>
<csr-id-6cf816f4e3ce1e81af614ba84de83ccf13e8e402/>
<csr-id-5cbae9c10e9f1d4302d041a864bfee83d47834e0/>
<csr-id-b6474691ea6af5ee441b02f6cb9c3cf2b8f97459/>
<csr-id-a973b62a8ef48acc92af8735e7e7bcac94e0092f/>

### Chore

 - <csr-id-5e81ac4fb8a2312eb546a4b86e71be05df7c4e26/> remove unusued `Error` module
 - <csr-id-6cf816f4e3ce1e81af614ba84de83ccf13e8e402/> remove unusued `Result` from methods
 - <csr-id-5cbae9c10e9f1d4302d041a864bfee83d47834e0/> remove dashmap dep in sn_{dysfunction, interface}

### Test

 - <csr-id-852716257efac7453f01e7404e254dd5481a40e4/> increase msg counts and rate of good failure
 - <csr-id-2b6c4a8fcd2cd7b22e6a4b20f1218c859110be62/> add ElderVoting to the startup msg count
   Adds ElderVoting as a msg that can fail to aid detection of bad elders.
 - <csr-id-c503d722b9d3bf3325d564bd28d7df695dd70e95/> update to take only one node per fault
   4 was an assumption of data msgs, but many of those even are not going to
   data_copy_count, so we can relax this and just pass more messages for non
   probe/dkg test msgs
   
   This also removes invalid portion of tests w/ less than 7 elders in DKG, as
   that's a startup case where all nodes should be trusted, and only serves to
   complicate the tests

### Bug Fixes

 - <csr-id-3d0a359004de49d95378929903aeb71fa12d83c0/> ensure std dev cannot be rounded down

### New Features

 - <csr-id-74b5e7794a213b70db7231c31b68cee340976119/> check elder scores against elders only, non elders against non elders
 - <csr-id-f072aae9155cf833ce3a0f304496f43f6862dff4/> add ElderVoting issue type and track on section proposal votes outgoing
 - <csr-id-addd2f806e81be1f04599fa556216b61ee5b8138/> enable removing knowledge issues
   This should allow us to track voted proposals more easily
   and weed out nodes that consistently dont vote
 - <csr-id-2862f1cb9a60dcf4b4d22349c90d303bbb1e8305/> update values based on latest network
   remove threshold concepts and use std_dev only
   remove bogus tests where we rely on unreal network assumptions
   (ie, we wont have only one AE probe msg failing... and if we do, then we should start consider
   ing it a failure...

### Chore

 - <csr-id-8d2ef1a0f298ef010f478fcd59c5b6c437b7b62f/> clarify comments and ensure sorted output of faulty nodes
   other misc cleanup
 - <csr-id-87cb70eefdc63f80942a2c87ecc3790f76105b91/> clarify naming of elders/non elders
 - <csr-id-dc16323849e425e2ca2511f095caee5b0a4af1ab/> store elders in fault detection
 - <csr-id-38d85b391a72a3ee71f705d9b89d6dbc74c041e1/> rename Knowledge -> NetworkKnowledge for clarity
 - <csr-id-7674b9f5609384a35072043a777ac07b18c12bb3/> sort faulty nodes by fault level
 - <csr-id-b13f1d7e5e84e42ed076654a159418563a9a1a35/> remove OpId for failed request tracking
   We use bidi now, so we can report after any failure, no need for
   double accounting
 - <csr-id-21af053a5be2317be356e760c2b581c0f870a396/> happy new year 2023

### Other

 - <csr-id-b6474691ea6af5ee441b02f6cb9c3cf2b8f97459/> sn_dkg integration

### Refactor (BREAKING)

 - <csr-id-a973b62a8ef48acc92af8735e7e7bcac94e0092f/> removing op id from query response
   - Use the query msg id to generate the operation id to track the response from Adults
   - Remove peers from pending data queries when response was obtained from Adults
   - Removing correlation id from SystemMsg node query/response
   - Redefine system::NodeQueryResponse type just as an alias to data::QueryResponse

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 23 commits contributed to the release over the course of 23 calendar days.
 - 15 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge #1987 ([`1bf3c65`](https://github.com/maidsafe/safe_network/commit/1bf3c65dda02489297e98fb27ce3cf4a241ebf48))
    - increase msg counts and rate of good failure ([`8527162`](https://github.com/maidsafe/safe_network/commit/852716257efac7453f01e7404e254dd5481a40e4))
    - clarify comments and ensure sorted output of faulty nodes ([`8d2ef1a`](https://github.com/maidsafe/safe_network/commit/8d2ef1a0f298ef010f478fcd59c5b6c437b7b62f))
    - check elder scores against elders only, non elders against non elders ([`74b5e77`](https://github.com/maidsafe/safe_network/commit/74b5e7794a213b70db7231c31b68cee340976119))
    - clarify naming of elders/non elders ([`87cb70e`](https://github.com/maidsafe/safe_network/commit/87cb70eefdc63f80942a2c87ecc3790f76105b91))
    - store elders in fault detection ([`dc16323`](https://github.com/maidsafe/safe_network/commit/dc16323849e425e2ca2511f095caee5b0a4af1ab))
    - ensure std dev cannot be rounded down ([`3d0a359`](https://github.com/maidsafe/safe_network/commit/3d0a359004de49d95378929903aeb71fa12d83c0))
    - Merge #1984 ([`dd07ad0`](https://github.com/maidsafe/safe_network/commit/dd07ad03a6112504c65c52a39aba0379b19c886c))
    - add ElderVoting to the startup msg count ([`2b6c4a8`](https://github.com/maidsafe/safe_network/commit/2b6c4a8fcd2cd7b22e6a4b20f1218c859110be62))
    - add ElderVoting issue type and track on section proposal votes outgoing ([`f072aae`](https://github.com/maidsafe/safe_network/commit/f072aae9155cf833ce3a0f304496f43f6862dff4))
    - rename Knowledge -> NetworkKnowledge for clarity ([`38d85b3`](https://github.com/maidsafe/safe_network/commit/38d85b391a72a3ee71f705d9b89d6dbc74c041e1))
    - enable removing knowledge issues ([`addd2f8`](https://github.com/maidsafe/safe_network/commit/addd2f806e81be1f04599fa556216b61ee5b8138))
    - Merge #1961 ([`7da114b`](https://github.com/maidsafe/safe_network/commit/7da114b75cdb2a919506b0800ece860cb3e6df3e))
    - update to take only one node per fault ([`c503d72`](https://github.com/maidsafe/safe_network/commit/c503d722b9d3bf3325d564bd28d7df695dd70e95))
    - update values based on latest network ([`2862f1c`](https://github.com/maidsafe/safe_network/commit/2862f1cb9a60dcf4b4d22349c90d303bbb1e8305))
    - sort faulty nodes by fault level ([`7674b9f`](https://github.com/maidsafe/safe_network/commit/7674b9f5609384a35072043a777ac07b18c12bb3))
    - Merge #1958 ([`d3355bc`](https://github.com/maidsafe/safe_network/commit/d3355bc3c47e3f68517dfc62c01f647571bd1f73))
    - remove OpId for failed request tracking ([`b13f1d7`](https://github.com/maidsafe/safe_network/commit/b13f1d7e5e84e42ed076654a159418563a9a1a35))
    - Merge #1951 ([`24ca31f`](https://github.com/maidsafe/safe_network/commit/24ca31fd53c570c7c97849b74ded850c05273353))
    - happy new year 2023 ([`21af053`](https://github.com/maidsafe/safe_network/commit/21af053a5be2317be356e760c2b581c0f870a396))
    - Merge branch 'main' into proposal_refactor ([`0bc7f94`](https://github.com/maidsafe/safe_network/commit/0bc7f94c72c374d667a9b455c4f4f1830366e4a4))
    - Merge #1873 ([`8be1563`](https://github.com/maidsafe/safe_network/commit/8be1563fcddde2323ae2f892687dc76f253f3fb2))
    - chore(naming): rename dysfunction - Uses the more common vocabulary in fault tolerance area. ([`f68073f`](https://github.com/maidsafe/safe_network/commit/f68073f2897894375f5a09b870e2bfe4e03c3b10))
</details>

## v0.14.0 (2022-09-19)

<csr-id-a8a9fb90791b29496e8559090dca4161e04054da/>

### Chore

 - <csr-id-a8a9fb90791b29496e8559090dca4161e04054da/> sn_interface-0.15.0/sn_dysfunction-0.14.0/sn_client-0.76.0/sn_node-0.71.0/sn_api-0.74.0/sn_cli-0.67.0

## v0.13.0 (2022-09-09)

<csr-id-448694176dd3b40a12bd8ecc16d9bb66fd171a37/>

### Chore

 - <csr-id-448694176dd3b40a12bd8ecc16d9bb66fd171a37/> sn_interface-0.14.0/sn_dysfunction-0.13.0/sn_client-0.75.0/sn_node-0.70.0/sn_api-0.73.0/sn_cli-0.66.0

## v0.12.0 (2022-09-07)

<csr-id-fe659c5685289fe0071b54298dcac394e83c0dce/>

### Chore

 - <csr-id-fe659c5685289fe0071b54298dcac394e83c0dce/> sn_interface-0.13.0/sn_dysfunction-0.12.0/sn_client-0.74.0/sn_node-0.69.0/sn_api-0.72.0/sn_cli-0.65.0

## v0.11.0 (2022-09-06)

<csr-id-d28fdf3ddd0a39f7bbc6426e1e71d990319b0ec7/>
<csr-id-dd89cac97da96ffe26ae78c4b7b62aa952ec53fc/>
<csr-id-921438659ccaf65b2ea8cc00efb61d8146ef71ef/>
<csr-id-f5d436fba99e0e9c258c7ab3c3a256be3be58f84/>
<csr-id-3a718d8c0957957a75250b044c9d1ad1b5874ab0/>
<csr-id-1b9e0a6564e9564201ef3a3e04adb0bfbef6ac14/>

### Chore

 - <csr-id-d28fdf3ddd0a39f7bbc6426e1e71d990319b0ec7/> sn_interface-0.11.0/sn_dysfunction-0.10.0/sn_client-0.72.0/sn_node-0.67.0/sn_api-0.70.0/sn_cli-0.63.0
 - <csr-id-dd89cac97da96ffe26ae78c4b7b62aa952ec53fc/> replace implicit clones with clone
 - <csr-id-921438659ccaf65b2ea8cc00efb61d8146ef71ef/> unneeded iter methods removal
 - <csr-id-f5d436fba99e0e9c258c7ab3c3a256be3be58f84/> applied use_self lint
 - <csr-id-3a718d8c0957957a75250b044c9d1ad1b5874ab0/> switch on clippy::unwrap_used as a warning


### Chore

 - <csr-id-1b9e0a6564e9564201ef3a3e04adb0bfbef6ac14/> sn_interface-0.12.0/sn_dysfunction-0.11.0/sn_client-0.73.0/sn_node-0.68.0/sn_api-0.71.0/sn_cli-0.64.0

## v0.10.0 (2022-09-04)

<csr-id-dd89cac97da96ffe26ae78c4b7b62aa952ec53fc/>
<csr-id-921438659ccaf65b2ea8cc00efb61d8146ef71ef/>
<csr-id-f5d436fba99e0e9c258c7ab3c3a256be3be58f84/>
<csr-id-3a718d8c0957957a75250b044c9d1ad1b5874ab0/>

### Chore

 - <csr-id-dd89cac97da96ffe26ae78c4b7b62aa952ec53fc/> replace implicit clones with clone
 - <csr-id-921438659ccaf65b2ea8cc00efb61d8146ef71ef/> unneeded iter methods removal
 - <csr-id-f5d436fba99e0e9c258c7ab3c3a256be3be58f84/> applied use_self lint
 - <csr-id-3a718d8c0957957a75250b044c9d1ad1b5874ab0/> switch on clippy::unwrap_used as a warning


## v0.9.0 (2022-08-23)

<csr-id-93a13d896343f746718be228c46a37b03d6618bb/>
<csr-id-43fcc7c517f95eab0e27ddc79cd9c6de3631c7c6/>

### Chore

 - <csr-id-93a13d896343f746718be228c46a37b03d6618bb/> run periodic checks on time

### Chore

 - <csr-id-43fcc7c517f95eab0e27ddc79cd9c6de3631c7c6/> sn_interface-0.10.0/sn_dysfunction-0.9.0/sn_client-0.70.0/sn_node-0.66.0/sn_api-0.68.0/sn_cli-0.61.0

## v0.8.0 (2022-08-14)

<csr-id-a4a39b421103af7c143280ad3860b3cbd3016386/>
<csr-id-3cf903367bfcd805ceff2f2508cd2b12eddc3ca5/>
<csr-id-29de67f1e3583eab867d517cb50ed2e404bd63fd/>
<csr-id-db22c6c8c1aedb347bea52199a5673695eff86f8/>
<csr-id-7c109a0e22b2032ad5ad3b10f828f855091bec67/>
<csr-id-2f38be726cf493c89d452b6faa50ab8284048798/>
<csr-id-bbb77f0c34e9d4c263be1c5362f1115ecee1da57/>
<csr-id-31d9f9f99b4e166986b8e51c3d41e0eac55621a4/>
<csr-id-dedec486f85c1cf6cf2d538238f32e826e08da0a/>
<csr-id-e39917d0635a071625f7961ce6d40cb44cc65da0/>
<csr-id-9fde534277f359dfa0a1d91d917864776edb5138/>
<csr-id-53f60c2327f8a69f0b2ef6d1a4e96644c10aa358/>

### Chore

 - <csr-id-a4a39b421103af7c143280ad3860b3cbd3016386/> further tweak dysf, reduce score by std dev for better avg.
   Also adjusts tests to this, which now feels a bit saner too
 - <csr-id-3cf903367bfcd805ceff2f2508cd2b12eddc3ca5/> remove unused severity; refactor weighted score
   Prev weighted score related everything to the std_deviation, but this
   has the effect of nullifying outliers and decreasing the impact of
   weighting.
   
   Instead we opt for a simple "threshold" score, above which, we're
   dysfunctional. So the sum of all issues tracked is used, and if
   we reach above this point, our node is deemed dysfunctional.
 - <csr-id-29de67f1e3583eab867d517cb50ed2e404bd63fd/> serialize NetworkPrefixMap into JSON
 - <csr-id-db22c6c8c1aedb347bea52199a5673695eff86f8/> cleanup unnecessary options and results
 - <csr-id-7c109a0e22b2032ad5ad3b10f828f855091bec67/> rename DysfunctionDetection::adults to nodes
 - <csr-id-2f38be726cf493c89d452b6faa50ab8284048798/> relax knowledge penalty.
   We've seen some CI nodes being booted due to knowledge issues, so relaxing
   this should help there'
 - <csr-id-bbb77f0c34e9d4c263be1c5362f1115ecee1da57/> relax knowledge penalty.
   We've seen some CI nodes being booted due to knowledge issues, so relaxing
   this should help there'
 - <csr-id-31d9f9f99b4e166986b8e51c3d41e0eac55621a4/> remove awaits from tests as well
 - <csr-id-dedec486f85c1cf6cf2d538238f32e826e08da0a/> remove unused async
 - <csr-id-e39917d0635a071625f7961ce6d40cb44cc65da0/> Tweak dysf interval, reducing to report on issues more rapidly
   If not, we can only ever propose one node for a membership change
   (lost), every 30s... which may not succeed under churn...

### Chore

 - <csr-id-53f60c2327f8a69f0b2ef6d1a4e96644c10aa358/> sn_interface-0.9.0/sn_dysfunction-0.8.0/sn_client-0.69.0/sn_node-0.65.0/sn_api-0.67.0/sn_cli-0.60.0

### New Features

 - <csr-id-b2c6b2164fbf6679edea0157217dc946d5f9d318/> add AeProbe dysfunction. Refactor score calculation

### Bug Fixes

 - <csr-id-4a17a1dcf858b5daf96e5b9f69ac33c10a988c27/> make the diff proportional to mean to be reported
 - <csr-id-3befae39e3dbc93c4187092e7abe3c6e21893184/> newly inserted operation shall not count towards issue
 - <csr-id-4773e185302ada27cd08c8dfd04582e7fdaf42aa/> removed unused async at dysfunction

### Refactor

 - <csr-id-9fde534277f359dfa0a1d91d917864776edb5138/> reissuing DBCs for all sn_cli tests only once as a setup stage

## v0.7.1 (2022-07-07)

<csr-id-46262268fc167c05963e5b7bd6261310496e2379/>
<csr-id-6b574bd53f7e51839380b7be914dbab015726d1e/>
<csr-id-2f6fff23a29cc4f04415a9a606fec88167551268/>
<csr-id-2b00cec961561281f6b927e13e501342843f6a0f/>

### Chore

 - <csr-id-46262268fc167c05963e5b7bd6261310496e2379/> `try!` macro is deprecated
   No need for rustfmt to check/replace this, as the compiler will already
   warn for this. Deprecated since 1.39.
   
   Removing the option seems to trigger a couple of formatting changes that
   rustfmt did not seem to pick on before.
 - <csr-id-6b574bd53f7e51839380b7be914dbab015726d1e/> Remove registerStorage cache
 - <csr-id-2f6fff23a29cc4f04415a9a606fec88167551268/> remove dysfunction arc/rwlock

### Chore

 - <csr-id-2b00cec961561281f6b927e13e501342843f6a0f/> sn_interface-0.8.1/sn_dysfunction-0.7.1/sn_client-0.68.1/sn_node-0.64.1/sn_api-0.66.1/sn_cli-0.59.1

## v0.7.0 (2022-07-04)

<csr-id-9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7/>
<csr-id-ddb7798a7b0c5e60960e123414277d58f3da27eb/>
<csr-id-e4e2eb56611a328806c59ed8bc80ca2567206bbb/>

### Chore

 - <csr-id-9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7/> Docs - put symbols in backticks
 - <csr-id-ddb7798a7b0c5e60960e123414277d58f3da27eb/> remove let bindings for unit returns

### Chore

 - <csr-id-e4e2eb56611a328806c59ed8bc80ca2567206bbb/> sn_interface-0.8.0/sn_dysfunction-0.7.0/sn_client-0.68.0/sn_node-0.64.0/sn_api-0.66.0/sn_cli-0.59.0

## v0.6.1 (2022-06-28)

<csr-id-eebbc30f5dd449b786115c37813a4554309875e0/>
<csr-id-58890e5c919ada30f27d4e80c6b5e7291b99ed5c/>

### Test

 - <csr-id-eebbc30f5dd449b786115c37813a4554309875e0/> adding new dysf test for DKG rounds

### Chore

 - <csr-id-58890e5c919ada30f27d4e80c6b5e7291b99ed5c/> sn_interface-0.7.1/sn_dysfunction-0.6.1/sn_client-0.67.1/sn_node-0.63.1

## v0.6.0 (2022-06-26)

<csr-id-243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e/>

### Chore

 - <csr-id-243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e/> sn_interface-0.7.0/sn_dysfunction-0.6.0/sn_client-0.67.0/sn_node-0.63.0/sn_api-0.65.0/sn_cli-0.58.0

## v0.5.3 (2022-06-24)

<csr-id-b433a23b2f661ad3ac0ebc290f457f1c64e04471/>
<csr-id-dc69a62eec590b2d621ab2cbc3009cb052955e66/>

### Test

 - <csr-id-b433a23b2f661ad3ac0ebc290f457f1c64e04471/> improving dysf test, reproducible issues
   we add defined strategy for zornames, use those for nodes, and add an
   address to the IssueTypes generated, so they can be reliably routed to
   the same nodes.
   
   We also adjust our assert to tolerate finding _less_ than the required
   bad nodes... but do not allow false positives

### Chore

 - <csr-id-dc69a62eec590b2d621ab2cbc3009cb052955e66/> sn_interface-0.6.5/sn_dysfunction-0.5.3/sn_client-0.66.5/sn_node-0.62.8/sn_cli-0.57.6

## v0.5.2 (2022-06-21)

<csr-id-c038635cf88d32c52da89d11a8532e6c91c8bf38/>
<csr-id-d526e0a32d3f09a788899d82db4fe6f13258568c/>

### Chore

 - <csr-id-c038635cf88d32c52da89d11a8532e6c91c8bf38/> misc cleanup

### Chore

 - <csr-id-d526e0a32d3f09a788899d82db4fe6f13258568c/> sn_interface-0.6.4/sn_dysfunction-0.5.2/sn_client-0.66.4/sn_node-0.62.7/sn_api-0.64.4

## v0.5.1 (2022-06-15)

<csr-id-26e35cc2d1c5aab81c3479dd7948f7a7e586f817/>
<csr-id-537b6c08447c15a056d8c79c8592106d9a40b672/>
<csr-id-f599c5973d50324aad1720166156666d5db1ed3d/>

### Chore

 - <csr-id-26e35cc2d1c5aab81c3479dd7948f7a7e586f817/> adjust some dysfunction weighting. decreas dkg
 - <csr-id-537b6c08447c15a056d8c79c8592106d9a40b672/> reduce comm error weighting

### Chore

 - <csr-id-f599c5973d50324aad1720166156666d5db1ed3d/> sn_interface-0.6.3/sn_dysfunction-0.5.1/sn_client-0.66.3/sn_api-0.64.3/sn_cli-0.57.4

### New Features

 - <csr-id-7ccb02a7ded7579bb8645c918b9a6108b1b585af/> enable tracking of Dkg issues

## v0.5.0 (2022-06-05)

<csr-id-1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9/>

### Chore

 - <csr-id-1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9/> sn_interface-0.6.0/sn_dysfunction-0.5.0/sn_client-0.66.0/sn_node-0.62.0/sn_api-0.64.0/sn_cli-0.57.0

## v0.4.0 (2022-05-27)

<csr-id-e5fcd032e1dd904e05bc23e119af1d06e3b85a06/>

### Chore

 - <csr-id-e5fcd032e1dd904e05bc23e119af1d06e3b85a06/> sn_interface-0.5.0/sn_dysfunction-0.4.0/sn_client-0.65.0/sn_node-0.61.0/sn_api-0.63.0/sn_cli-0.56.0

## v0.3.0 (2022-05-25)

<csr-id-ef56cf9cf8de45a9f13c2510c63de245b12aeae8/>

### Chore

 - <csr-id-ef56cf9cf8de45a9f13c2510c63de245b12aeae8/> sn_interface-0.4.0/sn_dysfunction-0.3.0/sn_client-0.64.0/sn_node-0.60.0/sn_api-0.62.0/sn_cli-0.55.0

## v0.2.0 (2022-05-21)

<csr-id-cf21d66b9b726123e0a4320cd68481b67f7af03d/>
<csr-id-24125eb3603a14c22e208964cbecac16915161ae/>
<csr-id-ef798150deb88efac1dcfe9a3cd0f2cebe1e4682/>

### Chore

 - <csr-id-cf21d66b9b726123e0a4320cd68481b67f7af03d/> sn_dysfunction-0.2.0/sn_client-0.63.0/sn_node-0.59.0/sn_api-0.61.0/sn_cli-0.54.0
 - <csr-id-24125eb3603a14c22e208964cbecac16915161ae/> ensure op writes use mut
   Dysfunction op writes were pulling from the dashmap but not using mut
   which could perhaps lead to a deadlock as we lock over the entry value

### Chore (BREAKING)

 - <csr-id-ef798150deb88efac1dcfe9a3cd0f2cebe1e4682/> add Display for OperationId


## v0.1.3 (2022-05-11)

<csr-id-66638f508ad4df12b757672df589ba8ad09fbdfc/>

### Chore

 - <csr-id-66638f508ad4df12b757672df589ba8ad09fbdfc/> sn_dysfunction-0.1.3/sn_node-0.58.17

### Bug Fixes

 - <csr-id-ddb939d5831b2f0d66fa2e0954b62e5e22a3ee69/> relax dysfunction for knowledge and conn issues
   Increases 10x the amount of conn or knowledge issues. We've been voting
   off nodes far too quickly, even on droplet testnets

## v0.1.2 (2022-04-23)

<csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/>
<csr-id-6452690c1b75bb8804c1f9de19c394a83f178acb/>
<csr-id-08385f4e03cd43b94f15523597f90f1cc9977a87/>
<csr-id-66901bcb3b68d3fbe84bfde915bb80ae1b562347/>
<csr-id-1f3af46aea59ebeb1b6a4b736e80e86ce2f724d8/>
<csr-id-2f4e7e6305ba387f2e28945aee71df650ac1d3eb/>

### Chore

 - <csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/> tidy references in cargo manifests
   All references are organised alphabetically, and random usage of long-form references are removed in
   favour of the short-form version, unless the long-form style is justified, e.g., when lots of
   features are being used.
 - <csr-id-6452690c1b75bb8804c1f9de19c394a83f178acb/> remove modules that only contained tests
   Due to refactoring the issue tracking into a single `track_issue` function, these modules didn't end
   up having any code, just tests.
   
   The tests were moved to separate testing modules in the `detection` module.
 - <csr-id-08385f4e03cd43b94f15523597f90f1cc9977a87/> move request_operation_fulfilled
   This function is moved from the `operations` module to the top level module, since the `operations`
   and other modules that now only contain tests, will be getting removed. The tests in the modules
   being removed will be moved into the `detection` module.
   
   Some unit test coverage was added for this function, and a new `get_unfulfilled_ops` function was
   added to facilitate easier testing, but it could also be used by callers of the API. It made testing
   easier because it wraps the code that reads values from the concurrent data structures, which can be
   quite verbose.
 - <csr-id-66901bcb3b68d3fbe84bfde915bb80ae1b562347/> remove unused dep

### Chore

 - <csr-id-2f4e7e6305ba387f2e28945aee71df650ac1d3eb/> sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0

### New Features

 - <csr-id-5df610c93b76cfc3a6f09734476240313b16bee6/> compare against all nodes in section
   When calculating scores, we compare against the average of all the nodes in the section, rather than
   the 'neighbours' of the node. As a consequence, the `DysfunctionDetection` struct becomes simpler,
   as we don't need to keep track of what nodes are 'closer' to others as the set of nodes changes.
   
   In the course of this change, the Dysfunction API was updated.
   
   First, the `ScoreType` enum was renamed `IssueType`, which now details the three issue types, rather
   than have timed versus operational. Secondly, one `track_issue` function merges three
   `track_x_issue` functions for adding issues to the tracker.
   
   Both these decisions were influenced by property testing: there were properties that should have
   been true for each issue type, and it was easier to write tests for those properties if they all
   used a single API.
   
   It's worth noting, the `track_issue` function provides an `Option` argument for supplying an
   `op_id`. This value only applies when a `PendingRequestOperation` type is used. At first, the
   pending request entry was declared as `PendingRequestOperation([u8; 32])`, which makes sense
   initially. For adding the issue, you just pass the op ID along with the issue type. However, the
   problem comes when you later want to select the issues of this type. For example:
   ```
   let _ = op_scores.insert(
   *node,
   self.calculate_node_score(
   node,
   adults.clone(),
   IssueType::PendingRequestOperation,
   )
   .await,
   );
   ```
   
   If you have the `op_id` parameter on the enum entry, the code becomes very clunky:
   ```
   let _ = op_scores.insert(
   *node,
   self.calculate_node_score(
   node,
   adults.clone(),
   IssueType::PendingRequestOperation([1; 32])
   )
   .await,
   );
   ```
   
   As can be seen, you need to supply an ugly placeholder value that has no effect. For this reason, I
   decided to just supply the `op_id` as an optional argument on `track_issue`. Neither solution is
   completely ideal, but I prefer this one.
   
   Remaining changes were a bit more superficial. A `ScoreResults` helper struct was introduced to
   return all three types of scores from the refactored `calculate_scores` function, rather than
   returning a tuple of three hash tables, which would have been a bit cumbersome.

### Refactor

 - <csr-id-1f3af46aea59ebeb1b6a4b736e80e86ce2f724d8/> remove op_id arg from track_issue
   Based on PR feedback, Yogesh pointed out we could change the `PendingRequestOperation` to use an
   `Option<OperationId>`. This solved the problem when performing a selection, because you can use
   `PendingRequestOperation(None)`. That's a lot better than using some placeholder value for the
   operation ID. This also tidies up `track_issue` to remove the optional `op_id` argument.

## v0.1.1 (2022-03-26)

<csr-id-df66875627aa41d06b7613085f05a97187c7175d/>
<csr-id-2e6d78c13c137e422d3714e8c113aeb4c0b597a3/>
<csr-id-b471b5c9f539933dd12de7af3473d2b0f61d7f28/>
<csr-id-1aa331daa42ef306728fc99e612fbddeed1501d7/>
<csr-id-52c218861044a46bf4e1666188dc58de232bde60/>
<csr-id-c9f27640d3b1c62bdf88ec954a395e09e799a181/>
<csr-id-907c7d3ef4f65df5566627938154dfca1e2fdc05/>
<csr-id-15a0d354fd804f8f44735b09c22f9e456211c067/>
<csr-id-aafb6d2a458fc4e2dc94ea3a08cb519fe52bc131/>

### Other

 - <csr-id-df66875627aa41d06b7613085f05a97187c7175d/> add sn_dysfunction to release process
   The release workflow is extended to include the dysfunction crate. Add dysfunction to:
   
   * Version bumping
   * Version outputs for release process
   * The release changelog
   * Publishing as the first crate
   
   A basic README was also added to the dysfunction crate as this seems to be a prerequisite for a
   publish.
 - <csr-id-2e6d78c13c137e422d3714e8c113aeb4c0b597a3/> add dysfunction tests to ci

### Chore

 - <csr-id-b471b5c9f539933dd12de7af3473d2b0f61d7f28/> sn_dysfunction-/safe_network-0.58.9
 - <csr-id-1aa331daa42ef306728fc99e612fbddeed1501d7/> sn_dysfunction-0.1.0/safe_network-0.58.8/sn_api-0.58.0/sn_cli-0.51.0
   This is a manually generated commit to try and get the first release and publish of dysfunction.
 - <csr-id-52c218861044a46bf4e1666188dc58de232bde60/> sn_dysfunction-0.1.0/safe_network-0.58.8/sn_api-0.58.0/sn_cli-0.51.0
   This is a manually generated commit to try and get the first release and publish of dysfunction.
 - <csr-id-c9f27640d3b1c62bdf88ec954a395e09e799a181/> use time::Instant in place of SystemTime
   This simplifies the duration checks during cleanup
 - <csr-id-907c7d3ef4f65df5566627938154dfca1e2fdc05/> safe_network-0.58.8/sn_api-0.58.0/sn_cli-0.51.0
 - <csr-id-15a0d354fd804f8f44735b09c22f9e456211c067/> update readme
   Arbitrary update to kick GHA off
 - <csr-id-aafb6d2a458fc4e2dc94ea3a08cb519fe52bc131/> rename dysfunction -> sn_dysfunction

### Bug Fixes

 - <csr-id-52aaf595293f2f0d3dd234907134bc624703a3ca/> ensure we have at least 1 when calculating each score

## v0.1.0 (2022-03-25)

This first version is being edited manually to trigger a release and publish of the first crate.

Inserting another manual change for testing purposes.

### Bug Fixes

 - <csr-id-52aaf595293f2f0d3dd234907134bc624703a3ca/> ensure we have at least 1 when calculating each score

