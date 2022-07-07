# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## v0.8.1 (2022-07-07)

### Chore

 - <csr-id-c79e2aac378b28b373fd7c18c4b9006348960071/> bit more low hanging clippy fruit
 - <csr-id-46262268fc167c05963e5b7bd6261310496e2379/> `try!` macro is deprecated
   No need for rustfmt to check/replace this, as the compiler will already
   warn for this. Deprecated since 1.39.
   
   Removing the option seems to trigger a couple of formatting changes that
   rustfmt did not seem to pick on before.
 - <csr-id-8dccb7f1fc81385f9f5f25e6c354ad1d35759528/> clippy runs cargo check already

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release.
 - 2 days passed between releases.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge #1315 ([`67686f7`](https://github.com/maidsafe/safe_network/commit/67686f73f9e7b18bb6fbf1eadc3fd3a256285396))
    - Merge #1313 ([`7fe7be3`](https://github.com/maidsafe/safe_network/commit/7fe7be336799dec811c5b17e6d753ebe31e625f1))
    - Merge branch 'main' into cargo-husky-tweaks ([`6881855`](https://github.com/maidsafe/safe_network/commit/688185573bb71cc44a7103df17f3fbeea6740247))
    - bit more low hanging clippy fruit ([`c79e2aa`](https://github.com/maidsafe/safe_network/commit/c79e2aac378b28b373fd7c18c4b9006348960071))
    - Merge branch 'main' into cargo-husky-tweaks ([`52dd02e`](https://github.com/maidsafe/safe_network/commit/52dd02e45ab4e160b0a26498919a79ce1aefb1bd))
    - `try!` macro is deprecated ([`4626226`](https://github.com/maidsafe/safe_network/commit/46262268fc167c05963e5b7bd6261310496e2379))
    - clippy runs cargo check already ([`8dccb7f`](https://github.com/maidsafe/safe_network/commit/8dccb7f1fc81385f9f5f25e6c354ad1d35759528))
</details>

## v0.8.0 (2022-07-04)

<csr-id-9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7/>
<csr-id-ddb7798a7b0c5e60960e123414277d58f3da27eb/>
<csr-id-4e04a2b0acc79140bf1d0aefd82c0ad5b046a3cd/>
<csr-id-6a2553a11b1404ad404e67df29bf3ec535d1b954/>
<csr-id-2aae965ca2fdd4ff59034547b5ee8dcef0b7253e/>
<csr-id-068327834c8d07ada6bf42cf78d6f7a117715466/>
<csr-id-976e8c3d8c610d2a34c1bfa6678132a1bad234e8/>
<csr-id-849dfba283362d8fbdddd92be1078c3a963fb564/>
<csr-id-91da4d4ac7aab039853b0651e5aafd9cdd31b9c4/>

### Chore

 - <csr-id-9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7/> Docs - put symbols in backticks
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
 - <csr-id-2aae965ca2fdd4ff59034547b5ee8dcef0b7253e/> use hardlink instead of symlink
 - <csr-id-068327834c8d07ada6bf42cf78d6f7a117715466/> sn_cli modify tests
 - <csr-id-976e8c3d8c610d2a34c1bfa6678132a1bad234e8/> sn_cli uses NetworkPrefixMap instead of node_conn_info.config
 - <csr-id-849dfba283362d8fbdddd92be1078c3a963fb564/> update PrefixMap symlink if incorrect
 - <csr-id-91da4d4ac7aab039853b0651e5aafd9cdd31b9c4/> remove node_connection_info.config from sn_node, sn_interface, sn_client

### New Features (BREAKING)

 - <csr-id-5dad80d3f239f5844243fedb89f8d4baaee3b640/> have the nodes to attach valid Commitments to signed SpentProofShares

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 6 calendar days.
 - 6 days passed between releases.
 - 11 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.0/sn_dysfunction-0.7.0/sn_client-0.68.0/sn_node-0.64.0/sn_api-0.66.0/sn_cli-0.59.0 ([`e4e2eb5`](https://github.com/maidsafe/safe_network/commit/e4e2eb56611a328806c59ed8bc80ca2567206bbb))
    - remove NetworkInfo::GenesisKey variant ([`6a2553a`](https://github.com/maidsafe/safe_network/commit/6a2553a11b1404ad404e67df29bf3ec535d1b954))
    - use hardlink instead of symlink ([`2aae965`](https://github.com/maidsafe/safe_network/commit/2aae965ca2fdd4ff59034547b5ee8dcef0b7253e))
    - sn_cli modify tests ([`0683278`](https://github.com/maidsafe/safe_network/commit/068327834c8d07ada6bf42cf78d6f7a117715466))
    - sn_cli uses NetworkPrefixMap instead of node_conn_info.config ([`976e8c3`](https://github.com/maidsafe/safe_network/commit/976e8c3d8c610d2a34c1bfa6678132a1bad234e8))
    - update PrefixMap symlink if incorrect ([`849dfba`](https://github.com/maidsafe/safe_network/commit/849dfba283362d8fbdddd92be1078c3a963fb564))
    - remove node_connection_info.config from sn_node, sn_interface, sn_client ([`91da4d4`](https://github.com/maidsafe/safe_network/commit/91da4d4ac7aab039853b0651e5aafd9cdd31b9c4))
    - Docs - put symbols in backticks ([`9314a2d`](https://github.com/maidsafe/safe_network/commit/9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7))
    - remove let bindings for unit returns ([`ddb7798`](https://github.com/maidsafe/safe_network/commit/ddb7798a7b0c5e60960e123414277d58f3da27eb))
    - have the nodes to attach valid Commitments to signed SpentProofShares ([`5dad80d`](https://github.com/maidsafe/safe_network/commit/5dad80d3f239f5844243fedb89f8d4baaee3b640))
    - remove unused asyncs (clippy) ([`4e04a2b`](https://github.com/maidsafe/safe_network/commit/4e04a2b0acc79140bf1d0aefd82c0ad5b046a3cd))
</details>

## v0.7.1 (2022-06-28)

<csr-id-8c69306dc86a99a8be443ab8213253983540f1cf/>
<csr-id-58890e5c919ada30f27d4e80c6b5e7291b99ed5c/>

### New Features

 - <csr-id-6bfd101ed12a16f3f6a9a0b55252d45d200af7c6/> Select which adult to query
   Let the client pick the adult to query, based on the XOR distance.

### Bug Fixes

 - <csr-id-752824774884ef77616d26734517c58530cdae1f/> resend last vote if nothing received after an interval.
   We were seeing stalled membership, perhaps due to dropped packages. This means we don't rest
   and if after an interval we haven't seen anything new, we trigger nodes to resend their votes out, which
   should hopefully complete the current gen

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
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.7.1/sn_dysfunction-0.6.1/sn_client-0.67.1/sn_node-0.63.1 ([`58890e5`](https://github.com/maidsafe/safe_network/commit/58890e5c919ada30f27d4e80c6b5e7291b99ed5c))
    - resend last vote if nothing received after an interval. ([`7528247`](https://github.com/maidsafe/safe_network/commit/752824774884ef77616d26734517c58530cdae1f))
    - Select which adult to query ([`6bfd101`](https://github.com/maidsafe/safe_network/commit/6bfd101ed12a16f3f6a9a0b55252d45d200af7c6))
    - Rename DataQuery with suffix Variant ([`8c69306`](https://github.com/maidsafe/safe_network/commit/8c69306dc86a99a8be443ab8213253983540f1cf))
</details>

## v0.7.0 (2022-06-26)

<csr-id-243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e/>
<csr-id-3f3c39a14987910bb424df51f89d948333ca3e87/>
<csr-id-5ea4c3d60bf84384ed37b5dde25ac4dc26147c24/>

### Chore

 - <csr-id-243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e/> sn_interface-0.7.0/sn_dysfunction-0.6.0/sn_client-0.67.0/sn_node-0.63.0/sn_api-0.65.0/sn_cli-0.58.0
 - <csr-id-3f3c39a14987910bb424df51f89d948333ca3e87/> changes based on review feedback
   * Prefer `map_err` in various places rather than a full `match`.
   * Change key serialization utility functions to static rather than instance.
   * Change `dog` command to print non-support of `SafeKey` data type rather than panic.
   * Remove unnecessary clone on `public_key_hex`.
   * Remove unnecessary match in various tests.
   * Ignore wallet CLI tests that deleted the credentials file. They are problematic when running in
     parallel with other tests. We need better isolated testing mechanisms for these. Will address in a
     separate PR.
   * Use different deposit names in wallet tests where multiple DBCs are deposited.
 - <csr-id-5ea4c3d60bf84384ed37b5dde25ac4dc26147c24/> changes based on review feedback
   * Prefer `map_err` in various places rather than a full `match`.
   * Change key serialization utility functions to static rather than instance.
   * Change `dog` command to print non-support of `SafeKey` data type rather than panic.
   * Remove unnecessary clone on `public_key_hex`.
   * Remove unnecessary match in various tests.
   * Ignore wallet CLI tests that deleted the credentials file. They are problematic when running in
     parallel with other tests. We need better isolated testing mechanisms for these. Will address in a
     separate PR.
   * Use different deposit names in wallet tests where multiple DBCs are deposited.

### New Features (BREAKING)

 - <csr-id-5577695b5d3291c46cd475df8c0933a067b4cfc5/> serialize to bls keys in util functions
   Utility functions were recently added to the API for serializing to the `Keypair` type. This was
   changed to serialize directly to BLS to make it easier for the CLI to deal directly with BLS keys.
   Soon we will be refactoring the `Keypair` type to have a different use case and things like
   `sn_client` would be refactored to directly work with BLS keys. This is a little step in that
   direction.
   
   There was a utility function added to `sn_interface` to create a `Keypair` from a hex-based BLS key
   because we still need to use the `Keypair` at this point in time.
 - <csr-id-67006eb2e84b750a6b9b03d04aafdcfc85b38955/> serialize to bls keys in util functions
   Utility functions were recently added to the API for serializing to the `Keypair` type. This was
   changed to serialize directly to BLS to make it easier for the CLI to deal directly with BLS keys.
   Soon we will be refactoring the `Keypair` type to have a different use case and things like
   `sn_client` would be refactored to directly work with BLS keys. This is a little step in that
   direction.
   
   There was a utility function added to `sn_interface` to create a `Keypair` from a hex-based BLS key
   because we still need to use the `Keypair` at this point in time.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 5 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.7.0/sn_dysfunction-0.6.0/sn_client-0.67.0/sn_node-0.63.0/sn_api-0.65.0/sn_cli-0.58.0 ([`243cfc4`](https://github.com/maidsafe/safe_network/commit/243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e))
    - changes based on review feedback ([`3f3c39a`](https://github.com/maidsafe/safe_network/commit/3f3c39a14987910bb424df51f89d948333ca3e87))
    - serialize to bls keys in util functions ([`5577695`](https://github.com/maidsafe/safe_network/commit/5577695b5d3291c46cd475df8c0933a067b4cfc5))
    - changes based on review feedback ([`5ea4c3d`](https://github.com/maidsafe/safe_network/commit/5ea4c3d60bf84384ed37b5dde25ac4dc26147c24))
    - serialize to bls keys in util functions ([`67006eb`](https://github.com/maidsafe/safe_network/commit/67006eb2e84b750a6b9b03d04aafdcfc85b38955))
    - Merge #1268 ([`e9adc0d`](https://github.com/maidsafe/safe_network/commit/e9adc0d3ba2f33fe0b4590a5fe11fea56bd4bda9))
</details>

## v0.6.5 (2022-06-24)

<csr-id-d7a831329ad600ad7b5a1b6b68841f96b8ef8cfa/>
<csr-id-1fbc762305a581680b52e2cbdaa7aea2feaf05ab/>
<csr-id-dc69a62eec590b2d621ab2cbc3009cb052955e66/>

### Chore

 - <csr-id-d7a831329ad600ad7b5a1b6b68841f96b8ef8cfa/> misc cleanup and fixes

### Chore

 - <csr-id-dc69a62eec590b2d621ab2cbc3009cb052955e66/> sn_interface-0.6.5/sn_dysfunction-0.5.3/sn_client-0.66.5/sn_node-0.62.8/sn_cli-0.57.6

### New Features

 - <csr-id-71eb46e47032074cdca678783e815b8d55ae39a0/> organize internal work

### Refactor

 - <csr-id-1fbc762305a581680b52e2cbdaa7aea2feaf05ab/> move it to its own file

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 3 calendar days.
 - 3 days passed between releases.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.5/sn_dysfunction-0.5.3/sn_client-0.66.5/sn_node-0.62.8/sn_cli-0.57.6 ([`dc69a62`](https://github.com/maidsafe/safe_network/commit/dc69a62eec590b2d621ab2cbc3009cb052955e66))
    - misc cleanup and fixes ([`d7a8313`](https://github.com/maidsafe/safe_network/commit/d7a831329ad600ad7b5a1b6b68841f96b8ef8cfa))
    - organize internal work ([`71eb46e`](https://github.com/maidsafe/safe_network/commit/71eb46e47032074cdca678783e815b8d55ae39a0))
    - Merge #1255 #1258 ([`ed0b5d8`](https://github.com/maidsafe/safe_network/commit/ed0b5d890e8404a59c25f8131eab5d23ce12eb7d))
    - Merge #1257 #1260 ([`19d89df`](https://github.com/maidsafe/safe_network/commit/19d89dfbbf8ac8ab2b08380ce9b4bed58a5dc0d9))
    - move it to its own file ([`1fbc762`](https://github.com/maidsafe/safe_network/commit/1fbc762305a581680b52e2cbdaa7aea2feaf05ab))
    - Merge branch 'main' into refactor-event-channel ([`024883e`](https://github.com/maidsafe/safe_network/commit/024883e9a1b853c02c29daa5c447b03570af2473))
</details>

## v0.6.4 (2022-06-21)

<csr-id-1574b495f17d25af2ed9dd017ccf8dce715a8b28/>
<csr-id-fd7f845f7d1534cf9ff93ee9dc9f3009ab7e5664/>
<csr-id-d204cffdc25a08f604f3a7b97dd74c0f4181b696/>
<csr-id-c038635cf88d32c52da89d11a8532e6c91c8bf38/>
<csr-id-d26d26df6ddd0321555fa3653be966fe91e2dca4/>
<csr-id-d526e0a32d3f09a788899d82db4fe6f13258568c/>

### Chore

 - <csr-id-1574b495f17d25af2ed9dd017ccf8dce715a8b28/> avoid another chain borrow/drop, use cloning api
 - <csr-id-fd7f845f7d1534cf9ff93ee9dc9f3009ab7e5664/> make NetworkKnowledge single threaded
 - <csr-id-d204cffdc25a08f604f3a7b97dd74c0f4181b696/> remove unused deps and enum variants
 - <csr-id-c038635cf88d32c52da89d11a8532e6c91c8bf38/> misc cleanup

### Refactor

 - <csr-id-d26d26df6ddd0321555fa3653be966fe91e2dca4/> cleanup and restructure of enum

### Chore

 - <csr-id-d526e0a32d3f09a788899d82db4fe6f13258568c/> sn_interface-0.6.4/sn_dysfunction-0.5.2/sn_client-0.66.4/sn_node-0.62.7/sn_api-0.64.4

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 5 days passed between releases.
 - 6 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.4/sn_dysfunction-0.5.2/sn_client-0.66.4/sn_node-0.62.7/sn_api-0.64.4 ([`d526e0a`](https://github.com/maidsafe/safe_network/commit/d526e0a32d3f09a788899d82db4fe6f13258568c))
    - cleanup and restructure of enum ([`d26d26d`](https://github.com/maidsafe/safe_network/commit/d26d26df6ddd0321555fa3653be966fe91e2dca4))
    - avoid another chain borrow/drop, use cloning api ([`1574b49`](https://github.com/maidsafe/safe_network/commit/1574b495f17d25af2ed9dd017ccf8dce715a8b28))
    - make NetworkKnowledge single threaded ([`fd7f845`](https://github.com/maidsafe/safe_network/commit/fd7f845f7d1534cf9ff93ee9dc9f3009ab7e5664))
    - remove unused deps and enum variants ([`d204cff`](https://github.com/maidsafe/safe_network/commit/d204cffdc25a08f604f3a7b97dd74c0f4181b696))
    - misc cleanup ([`c038635`](https://github.com/maidsafe/safe_network/commit/c038635cf88d32c52da89d11a8532e6c91c8bf38))
</details>

## v0.6.3 (2022-06-15)

<csr-id-4eb43fa884d7b047febb18c067ae905969a113bf/>
<csr-id-537b6c08447c15a056d8c79c8592106d9a40b672/>
<csr-id-f599c5973d50324aad1720166156666d5db1ed3d/>

### Chore

 - <csr-id-4eb43fa884d7b047febb18c067ae905969a113bf/> upgrade blsttc to 6.0.0
   There were various other crates that had to be upgraded in this process:
   * secured_linked_list to v0.5.2 because it was also upgraded to reference v6.0.0 of blsttc
   * bls_dkg to v0.10.3 because it was also upgraded to reference v6.0.0 of blsttc
   * sn_consensus to v2.1.1 because it was also upgraded to reference v6.0.0 of blsttc
   * sn_dbc to v4.0.0 because it was also upgraded to reference v6.0.0 of blsttc
 - <csr-id-537b6c08447c15a056d8c79c8592106d9a40b672/> reduce comm error weighting

### Chore

 - <csr-id-f599c5973d50324aad1720166156666d5db1ed3d/> sn_interface-0.6.3/sn_dysfunction-0.5.1/sn_client-0.66.3/sn_api-0.64.3/sn_cli-0.57.4

### New Features

 - <csr-id-7ccb02a7ded7579bb8645c918b9a6108b1b585af/> enable tracking of Dkg issues

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.3/sn_dysfunction-0.5.1/sn_client-0.66.3/sn_api-0.64.3/sn_cli-0.57.4 ([`f599c59`](https://github.com/maidsafe/safe_network/commit/f599c5973d50324aad1720166156666d5db1ed3d))
    - upgrade blsttc to 6.0.0 ([`4eb43fa`](https://github.com/maidsafe/safe_network/commit/4eb43fa884d7b047febb18c067ae905969a113bf))
    - reduce comm error weighting ([`537b6c0`](https://github.com/maidsafe/safe_network/commit/537b6c08447c15a056d8c79c8592106d9a40b672))
    - enable tracking of Dkg issues ([`7ccb02a`](https://github.com/maidsafe/safe_network/commit/7ccb02a7ded7579bb8645c918b9a6108b1b585af))
</details>

## v0.6.2 (2022-06-15)

<csr-id-b818c3fd10a4e3304b2c5f84dac843397873cba6/>
<csr-id-46246f155ab65f3fcd61381345f1a7f747dfe957/>

### New Features

 - <csr-id-1b1cb77df6c2805ecfa741bb824b359214558929/> remove private registers
 - <csr-id-f1829f99ef1415a83731f855757fbce9970fa4f0/> remove private data addresses
 - <csr-id-8be2f2c9efac1623ea95ff1641c6b9bc22fad455/> remove private safe key addresses

### Bug Fixes

 - <csr-id-616d8cb12bfc257f9b3609239790065ebced8fe3/> replace at_least_one_elders with supermajority for sending cmd
 - <csr-id-60f5a68a1df6114b65d7c57099fea0347ba3d1dd/> some changes I missed in the initial private removal

### Test

 - <csr-id-b818c3fd10a4e3304b2c5f84dac843397873cba6/> cmd sent to all elders

### Chore

 - <csr-id-46246f155ab65f3fcd61381345f1a7f747dfe957/> sn_interface-0.6.2/sn_client-0.66.2/sn_node-0.62.4/sn_api-0.64.2/sn_cli-0.57.3

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 2 calendar days.
 - 8 days passed between releases.
 - 7 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.2/sn_client-0.66.2/sn_node-0.62.4/sn_api-0.64.2/sn_cli-0.57.3 ([`46246f1`](https://github.com/maidsafe/safe_network/commit/46246f155ab65f3fcd61381345f1a7f747dfe957))
    - Merge remote-tracking branch 'origin/main' into drusu/remove-private-data ([`2057273`](https://github.com/maidsafe/safe_network/commit/2057273509c2488cafc7f6db2ae69a99efc3b350))
    - Merge branch 'main' into drusu/remove-private-data ([`0cd2007`](https://github.com/maidsafe/safe_network/commit/0cd2007e442086d6eb2a39ad1f452e590fad46a9))
    - replace at_least_one_elders with supermajority for sending cmd ([`616d8cb`](https://github.com/maidsafe/safe_network/commit/616d8cb12bfc257f9b3609239790065ebced8fe3))
    - some changes I missed in the initial private removal ([`60f5a68`](https://github.com/maidsafe/safe_network/commit/60f5a68a1df6114b65d7c57099fea0347ba3d1dd))
    - remove private registers ([`1b1cb77`](https://github.com/maidsafe/safe_network/commit/1b1cb77df6c2805ecfa741bb824b359214558929))
    - remove private data addresses ([`f1829f9`](https://github.com/maidsafe/safe_network/commit/f1829f99ef1415a83731f855757fbce9970fa4f0))
    - remove private safe key addresses ([`8be2f2c`](https://github.com/maidsafe/safe_network/commit/8be2f2c9efac1623ea95ff1641c6b9bc22fad455))
    - cmd sent to all elders ([`b818c3f`](https://github.com/maidsafe/safe_network/commit/b818c3fd10a4e3304b2c5f84dac843397873cba6))
</details>

## v0.6.1 (2022-06-07)

<csr-id-24299786ba730e467c10946c8c152936b96148f8/>
<csr-id-489904e325cfb8efca4289b05125904ad4029f3b/>

### Chore

 - <csr-id-24299786ba730e467c10946c8c152936b96148f8/> address some review comments

### Chore

 - <csr-id-489904e325cfb8efca4289b05125904ad4029f3b/> sn_interface-0.6.1/sn_client-0.66.1/sn_node-0.62.1/sn_api-0.64.1

### New Features

 - <csr-id-dbda86be03f912079776be514828ff5fd034830c/> first version of Spentbook messaging, storage, and client API
   - Storage is implemented using Register as the underlying data type. To be changed when
   actual SpentBook native data type is put in place.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.1/sn_client-0.66.1/sn_node-0.62.1/sn_api-0.64.1 ([`489904e`](https://github.com/maidsafe/safe_network/commit/489904e325cfb8efca4289b05125904ad4029f3b))
    - Merge branch 'main' into Gabriel_Spentbook_PR1143 ([`0eda02a`](https://github.com/maidsafe/safe_network/commit/0eda02ac126be4f088af6bf9e7247c8496a389ba))
    - address some review comments ([`2429978`](https://github.com/maidsafe/safe_network/commit/24299786ba730e467c10946c8c152936b96148f8))
    - first version of Spentbook messaging, storage, and client API ([`dbda86b`](https://github.com/maidsafe/safe_network/commit/dbda86be03f912079776be514828ff5fd034830c))
    - Merge #1217 ([`2f26043`](https://github.com/maidsafe/safe_network/commit/2f2604325d533357bad7d917315cf4cba0b2d3c0))
</details>

## v0.6.0 (2022-06-05)

<csr-id-1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9/>

### Chore

 - <csr-id-1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9/> sn_interface-0.6.0/sn_dysfunction-0.5.0/sn_client-0.66.0/sn_node-0.62.0/sn_api-0.64.0/sn_cli-0.57.0

### New Features

 - <csr-id-95de2ffe6f57ae0e6cebf123da3e9b6c3ad84aaf/> handover sap elder checks with membership knowledge
 - <csr-id-e3169f385c795ada14fde85a88aa04399934b9d7/> add bls type to keypair enum
   Extends the `sn_interface::types::keys::Keypair` enum to support the BLS keypair type. We need this
   because we want to change the CLI to use BLS rather than Ed25519 keys, so we can support signing
   output DBCs with the same keypair we use to sign commands sent from the CLI.
   
   I modified the tests that check each keypair type can be serialized/deserialized. Previously there
   was one test case looping over each type of keypair, but I think it's better style to define each
   test case explicitly: you are calling out what scenarios you want to support and it makes the cases
   easier to understand at a glance, even if there is a small bit of repetition between them.

### New Features (BREAKING)

 - <csr-id-f03fb7e35319dbb9e4745e3cb36c7913c4f220ac/> cli will now use bls keys

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 4 calendar days.
 - 8 days passed between releases.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.0/sn_dysfunction-0.5.0/sn_client-0.66.0/sn_node-0.62.0/sn_api-0.64.0/sn_cli-0.57.0 ([`1bf7dfb`](https://github.com/maidsafe/safe_network/commit/1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9))
    - Merge branch 'main' into handover_byz_sap_check_squashed ([`6769996`](https://github.com/maidsafe/safe_network/commit/6769996e3ea78a6be306437193687b422a21ce80))
    - handover sap elder checks with membership knowledge ([`95de2ff`](https://github.com/maidsafe/safe_network/commit/95de2ffe6f57ae0e6cebf123da3e9b6c3ad84aaf))
    - cli will now use bls keys ([`f03fb7e`](https://github.com/maidsafe/safe_network/commit/f03fb7e35319dbb9e4745e3cb36c7913c4f220ac))
    - add bls type to keypair enum ([`e3169f3`](https://github.com/maidsafe/safe_network/commit/e3169f385c795ada14fde85a88aa04399934b9d7))
    - Merge #1192 ([`f9fc2a7`](https://github.com/maidsafe/safe_network/commit/f9fc2a76f083ba5161c8c4eef9013c53586b4693))
</details>

## v0.5.0 (2022-05-27)

<csr-id-e5fcd032e1dd904e05bc23e119af1d06e3b85a06/>

### Chore

 - <csr-id-e5fcd032e1dd904e05bc23e119af1d06e3b85a06/> sn_interface-0.5.0/sn_dysfunction-0.4.0/sn_client-0.65.0/sn_node-0.61.0/sn_api-0.63.0/sn_cli-0.56.0

### New Features

 - <csr-id-0c449a731b22eb25e616d83182899e12aba95d7f/> handover AE, empty consensus handling, generations

### New Features (BREAKING)

 - <csr-id-294549ebc998d11a2f3621e2a9fd20a0dd9bcce5/> remove sus node flows, replicate data per data

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 2 calendar days.
 - 2 days passed between releases.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.5.0/sn_dysfunction-0.4.0/sn_client-0.65.0/sn_node-0.61.0/sn_api-0.63.0/sn_cli-0.56.0 ([`e5fcd03`](https://github.com/maidsafe/safe_network/commit/e5fcd032e1dd904e05bc23e119af1d06e3b85a06))
    - Merge #1202 ([`e42a2e3`](https://github.com/maidsafe/safe_network/commit/e42a2e3c212597e68238451a5bb4a8725c4761be))
    - handover AE, empty consensus handling, generations ([`0c449a7`](https://github.com/maidsafe/safe_network/commit/0c449a731b22eb25e616d83182899e12aba95d7f))
    - Merge #1208 ([`6c9b851`](https://github.com/maidsafe/safe_network/commit/6c9b851dd5bab8b2f5d9b3ef1db72d198706ac9d))
    - remove sus node flows, replicate data per data ([`294549e`](https://github.com/maidsafe/safe_network/commit/294549ebc998d11a2f3621e2a9fd20a0dd9bcce5))
    - Merge #1198 #1204 ([`5e82ef3`](https://github.com/maidsafe/safe_network/commit/5e82ef3d0e78898f9ffac8bebe4970c4d26e608f))
    - Merge branch 'main' into bump-consensus-2.0.0 ([`a1c592a`](https://github.com/maidsafe/safe_network/commit/a1c592a71247660e7372e019e5f9a6ea23299e0f))
</details>

## v0.4.0 (2022-05-25)

<csr-id-ef56cf9cf8de45a9f13c2510c63de245b12aeae8/>
<csr-id-392e522c69803fddbeb3cd9e1cbae8060188578f/>
<csr-id-cf21d66b9b726123e0a4320cd68481b67f7af03d/>
<csr-id-8e2731d8b7923a9050451b31ef3a92f892d2d6d3/>
<csr-id-f2742d92b3c3b56ed80732aa1d6943885fcd4317/>
<csr-id-cb733fd4b1ed642da73f1e9db4fc3d8a1ec49a2b/>
<csr-id-ef798150deb88efac1dcfe9a3cd0f2cebe1e4682/>

### Chore

 - <csr-id-ef56cf9cf8de45a9f13c2510c63de245b12aeae8/> sn_interface-0.4.0/sn_dysfunction-0.3.0/sn_client-0.64.0/sn_node-0.60.0/sn_api-0.62.0/sn_cli-0.55.0
 - <csr-id-392e522c69803fddbeb3cd9e1cbae8060188578f/> bump consensus 1.16.0 -> 2.0.0
 - <csr-id-cf21d66b9b726123e0a4320cd68481b67f7af03d/> sn_dysfunction-0.2.0/sn_client-0.63.0/sn_node-0.59.0/sn_api-0.61.0/sn_cli-0.54.0

### New Features

 - <csr-id-941703e23a53d8d894d5a9a7a253ad1735e900e0/> error triggering on churn join miss
 - <csr-id-fe073bc0674c2099b7cd3f30ac744ea6172e24c2/> section probing for all nodes every 120s

### Refactor

 - <csr-id-8e2731d8b7923a9050451b31ef3a92f892d2d6d3/> de-dupe init_test_logger
 - <csr-id-f2742d92b3c3b56ed80732aa1d6943885fcd4317/> cargo test works without feature flag
 - <csr-id-cb733fd4b1ed642da73f1e9db4fc3d8a1ec49a2b/> move NodeState validations to NodeState struct

### Chore (BREAKING)

 - <csr-id-ef798150deb88efac1dcfe9a3cd0f2cebe1e4682/> add Display for OperationId


### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 5 calendar days.
 - 7 days passed between releases.
 - 9 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.4.0/sn_dysfunction-0.3.0/sn_client-0.64.0/sn_node-0.60.0/sn_api-0.62.0/sn_cli-0.55.0 ([`ef56cf9`](https://github.com/maidsafe/safe_network/commit/ef56cf9cf8de45a9f13c2510c63de245b12aeae8))
    - bump consensus 1.16.0 -> 2.0.0 ([`392e522`](https://github.com/maidsafe/safe_network/commit/392e522c69803fddbeb3cd9e1cbae8060188578f))
    - Merge #1195 ([`c6e6e32`](https://github.com/maidsafe/safe_network/commit/c6e6e324164028c6c15a78643783a9f86679f39e))
    - sn_dysfunction-0.2.0/sn_client-0.63.0/sn_node-0.59.0/sn_api-0.61.0/sn_cli-0.54.0 ([`cf21d66`](https://github.com/maidsafe/safe_network/commit/cf21d66b9b726123e0a4320cd68481b67f7af03d))
    - add Display for OperationId ([`ef79815`](https://github.com/maidsafe/safe_network/commit/ef798150deb88efac1dcfe9a3cd0f2cebe1e4682))
    - de-dupe init_test_logger ([`8e2731d`](https://github.com/maidsafe/safe_network/commit/8e2731d8b7923a9050451b31ef3a92f892d2d6d3))
    - cargo test works without feature flag ([`f2742d9`](https://github.com/maidsafe/safe_network/commit/f2742d92b3c3b56ed80732aa1d6943885fcd4317))
    - Merge branch 'main' into move-membership-history-to-network-knowledge ([`57de06b`](https://github.com/maidsafe/safe_network/commit/57de06b828191e093de06750f94fe6f500890112))
    - move NodeState validations to NodeState struct ([`cb733fd`](https://github.com/maidsafe/safe_network/commit/cb733fd4b1ed642da73f1e9db4fc3d8a1ec49a2b))
    - error triggering on churn join miss ([`941703e`](https://github.com/maidsafe/safe_network/commit/941703e23a53d8d894d5a9a7a253ad1735e900e0))
    - section probing for all nodes every 120s ([`fe073bc`](https://github.com/maidsafe/safe_network/commit/fe073bc0674c2099b7cd3f30ac744ea6172e24c2))
</details>

## v0.2.4 (2022-05-18)

<csr-id-07504faeda6cbfd0b27abea25facde992398ecf9/>
<csr-id-e25fb53a299dd5daa755799c36a316e4b011f4d7/>
<csr-id-9b06304f46e1a1bda90a0fc6ff82edc928c2529d/>

### Chore

 - <csr-id-07504faeda6cbfd0b27abea25facde992398ecf9/> upgrade blsttc to v5.2.0 and rand to v0.8
 - <csr-id-e25fb53a299dd5daa755799c36a316e4b011f4d7/> rename DkgId generation to section chain len

### Chore

 - <csr-id-9b06304f46e1a1bda90a0fc6ff82edc928c2529d/> sn_interface-0.2.4/sn_client-0.62.3/sn_api-0.60.2/sn_cli-0.53.1

### New Features

 - <csr-id-2b18ba8a1b0e8342af176bb78dba08f3e7b65d26/> add membership generation to DKG and SectionInfo agreement
   This prevents bogus DKG failure when two generations (of same prefix)
   may crossover under heavy churn

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 1 calendar day.
 - 5 days passed between releases.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.4/sn_client-0.62.3/sn_api-0.60.2/sn_cli-0.53.1 ([`9b06304`](https://github.com/maidsafe/safe_network/commit/9b06304f46e1a1bda90a0fc6ff82edc928c2529d))
    - upgrade blsttc to v5.2.0 and rand to v0.8 ([`07504fa`](https://github.com/maidsafe/safe_network/commit/07504faeda6cbfd0b27abea25facde992398ecf9))
    - Merge #1189 ([`00f41b4`](https://github.com/maidsafe/safe_network/commit/00f41b4a96bcc172d91620aa0da0cb799db5483c))
    - Merge branch 'main' into Handover ([`7734f36`](https://github.com/maidsafe/safe_network/commit/7734f36ce326277647ac2b680a2d3f562d92917b))
    - rename DkgId generation to section chain len ([`e25fb53`](https://github.com/maidsafe/safe_network/commit/e25fb53a299dd5daa755799c36a316e4b011f4d7))
    - add membership generation to DKG and SectionInfo agreement ([`2b18ba8`](https://github.com/maidsafe/safe_network/commit/2b18ba8a1b0e8342af176bb78dba08f3e7b65d26))
</details>

## v0.2.3 (2022-05-12)

<csr-id-00dc9c0ba9afb6de038dda9e20a10e6727a0b0e6/>
<csr-id-a49a007ef8fde53a346403824f09eb0fd25e1109/>

### Chore

 - <csr-id-00dc9c0ba9afb6de038dda9e20a10e6727a0b0e6/> reduce handover logging

### Chore

 - <csr-id-a49a007ef8fde53a346403824f09eb0fd25e1109/> sn_interface-0.2.3/sn_node-0.58.18

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.3/sn_node-0.58.18 ([`a49a007`](https://github.com/maidsafe/safe_network/commit/a49a007ef8fde53a346403824f09eb0fd25e1109))
    - reduce handover logging ([`00dc9c0`](https://github.com/maidsafe/safe_network/commit/00dc9c0ba9afb6de038dda9e20a10e6727a0b0e6))
</details>

## v0.2.2 (2022-05-10)

<csr-id-61ba367c308a846cb3f1ae065b1fbbdfb85838e4/>
<csr-id-ae9aeeb94f55f29849c8c5fe1b05419b96fac6e9/>

### Chore

 - <csr-id-61ba367c308a846cb3f1ae065b1fbbdfb85838e4/> sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1

### Chore

 - <csr-id-ae9aeeb94f55f29849c8c5fe1b05419b96fac6e9/> add ProposalAgreed log marker

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 1 calendar day.
 - 3 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1 ([`61ba367`](https://github.com/maidsafe/safe_network/commit/61ba367c308a846cb3f1ae065b1fbbdfb85838e4))
    - Merge #1171 ([`06b4433`](https://github.com/maidsafe/safe_network/commit/06b4433f199ba7c622ad57e767d80f58f0b50a69))
    - Merge #1140 ([`459b641`](https://github.com/maidsafe/safe_network/commit/459b641f22b488f33825777b974da80512eabed5))
    - Merge #1165 ([`e08096d`](https://github.com/maidsafe/safe_network/commit/e08096d37dfab490f22ae9786a006aa3f9f630c1))
    - Merge #1167 ([`5b21c66`](https://github.com/maidsafe/safe_network/commit/5b21c663c7f11124f0ed2f330b2f8687745f7da7))
    - Merge #1169 ([`e5d0c17`](https://github.com/maidsafe/safe_network/commit/e5d0c17c335a3a25ee0bb4c81906fa176abeb7f5))
    - add ProposalAgreed log marker ([`ae9aeeb`](https://github.com/maidsafe/safe_network/commit/ae9aeeb94f55f29849c8c5fe1b05419b96fac6e9))
</details>

## v0.2.1 (2022-05-06)

<csr-id-155d62257546868513627709742215c0c8f9574f/>
<csr-id-e8f4fbca2acb81b3ddc1b275140e5f7b1b56f9a9/>
<csr-id-7766e7d20b392cf5b8563d1dbc9560254b44e756/>
<csr-id-737d906a61f772593ac7df755d995d66059e8b5e/>
<csr-id-ae9aeeb94f55f29849c8c5fe1b05419b96fac6e9/>

### Chore

 - <csr-id-155d62257546868513627709742215c0c8f9574f/> check and log for shrinking SAP on verify_with_chain
 - <csr-id-e8f4fbca2acb81b3ddc1b275140e5f7b1b56f9a9/> make client targets relative to sap size
   The proivided sap could be different from expected, but
   we should be able to trust if if it's valid... As such
   we base target counts off of the provided SAP
 - <csr-id-7766e7d20b392cf5b8563d1dbc9560254b44e756/> rename MsgKind -> AuthKind
   This feels more correct given that the kind is actually about the authority that
   the message carries.

### Chore

 - <csr-id-737d906a61f772593ac7df755d995d66059e8b5e/> sn_interface-0.2.1/sn_client-0.62.1/sn_node-0.58.15/sn_api-0.60.0/sn_cli-0.53.0
 - <csr-id-ae9aeeb94f55f29849c8c5fe1b05419b96fac6e9/> add ProposalAgreed log marker

### New Features

 - <csr-id-0d5cdf940afc390de22d94e91621e76d45a9eaad/> handover integration squashed
 - <csr-id-696414ac858795628872a594268517e99a671b00/> add separate feature flags for register/chunk messages
 - <csr-id-c08f05537b70f2d6e0759a39b3f917c0e305e734/> add service-msg feature flag to messaging
   This allows us to more easily separate out what kind of messaging
   interface we want ndoes to be able to accept.
   
   Eg. Removing all service messages means we can focus on only the
   infrastructure layer..

### Bug Fixes

 - <csr-id-dd353b969ace383c3e89c94f7f242b84b6aee89f/> early return when AE required from a vote batch
   With latest changes we can have vote batches, and if for some reason
   we were not up to speed with the provided info, we were requesting AE
   updates for _every_ vote in the batch.
   
   Here we change that to request only one AE for the first one that fails.
 - <csr-id-9f4c3a523212c41079afcde8052a0891f3895f3b/> client knowledge could not update
   adds network knowledge storage to clients.
   Previously we were seeing issues where knowledge could not be
   updated after receiving one of two sibling saps after split.
   
   now we store the whole knowledge and validate against this chain
 - <csr-id-829eb33184c6012faa2020333e72a7c811fdb660/> batch MembershipVotes in order to ensure that order is preserved.
   Membership AE could trigger looping if response messages were processed in a bad
   order, so now we just send all the votes in a oner, in order, and those will be handled
   in the correct order. Hopefully cutting down on potential AE looping.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 14 commits contributed to the release over the course of 11 calendar days.
 - 13 days passed between releases.
 - 10 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.1/sn_client-0.62.1/sn_node-0.58.15/sn_api-0.60.0/sn_cli-0.53.0 ([`737d906`](https://github.com/maidsafe/safe_network/commit/737d906a61f772593ac7df755d995d66059e8b5e))
    - early return when AE required from a vote batch ([`dd353b9`](https://github.com/maidsafe/safe_network/commit/dd353b969ace383c3e89c94f7f242b84b6aee89f))
    - batch MembershipVotes in order to ensure that order is preserved. ([`829eb33`](https://github.com/maidsafe/safe_network/commit/829eb33184c6012faa2020333e72a7c811fdb660))
    - client knowledge could not update ([`9f4c3a5`](https://github.com/maidsafe/safe_network/commit/9f4c3a523212c41079afcde8052a0891f3895f3b))
    - check and log for shrinking SAP on verify_with_chain ([`155d622`](https://github.com/maidsafe/safe_network/commit/155d62257546868513627709742215c0c8f9574f))
    - make client targets relative to sap size ([`e8f4fbc`](https://github.com/maidsafe/safe_network/commit/e8f4fbca2acb81b3ddc1b275140e5f7b1b56f9a9))
    - handover integration squashed ([`0d5cdf9`](https://github.com/maidsafe/safe_network/commit/0d5cdf940afc390de22d94e91621e76d45a9eaad))
    - Merge #1141 ([`865f244`](https://github.com/maidsafe/safe_network/commit/865f24477244155528583afa5a3655690e4b7093))
    - add separate feature flags for register/chunk messages ([`696414a`](https://github.com/maidsafe/safe_network/commit/696414ac858795628872a594268517e99a671b00))
    - add service-msg feature flag to messaging ([`c08f055`](https://github.com/maidsafe/safe_network/commit/c08f05537b70f2d6e0759a39b3f917c0e305e734))
    - Merge branch 'main' into Feat-InterfaceAuthKind ([`df40fb9`](https://github.com/maidsafe/safe_network/commit/df40fb94f6847b31aec730eb7cbc6c0b97fe9a0e))
    - Merge branch 'main' into Feat-InterfaceAuthKind ([`5db6533`](https://github.com/maidsafe/safe_network/commit/5db6533b2151e2377299a0be11e513210adfabd4))
    - rename MsgKind -> AuthKind ([`7766e7d`](https://github.com/maidsafe/safe_network/commit/7766e7d20b392cf5b8563d1dbc9560254b44e756))
    - Merge #1128 ([`e49d382`](https://github.com/maidsafe/safe_network/commit/e49d38239b3a8c468616ad3782e1208316e9b5e0))
</details>

## v0.2.0 (2022-04-23)

<csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/>
<csr-id-88421d9cb7872b6397283a0035130bc14de6d4ff/>
<csr-id-9ea06ffe9339d3927897f010314b1be1bf7026bf/>
<csr-id-f37582288da65f27f53eb27453a4693166821064/>
<csr-id-7b8ce1c9d980015768a300ac99d07f69cc1f5ae3/>
<csr-id-8494a01d9be3dddc0d0f4c2c37cdc4d6c3e54521/>
<csr-id-ad7aa2d27c1eeeb11734f5cc2712383a36343d54/>
<csr-id-2f4e7e6305ba387f2e28945aee71df650ac1d3eb/>

### Chore

 - <csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/> tidy references in cargo manifests
   All references are organised alphabetically, and random usage of long-form references are removed in
   favour of the short-form version, unless the long-form style is justified, e.g., when lots of
   features are being used.
 - <csr-id-88421d9cb7872b6397283a0035130bc14de6d4ff/> pull sn_client out of the node codebase
 - <csr-id-9ea06ffe9339d3927897f010314b1be1bf7026bf/> sn_dysfunction-0.1.1/safe_network-0.58.13/sn_api-0.58.2/sn_cli-0.51.3
 - <csr-id-f37582288da65f27f53eb27453a4693166821064/> add changelog/readme for sn_interface publishing
 - <csr-id-7b8ce1c9d980015768a300ac99d07f69cc1f5ae3/> remove unused sn_interface deps
 - <csr-id-8494a01d9be3dddc0d0f4c2c37cdc4d6c3e54521/> split put messaging and types into top level crate

### Chore

 - <csr-id-2f4e7e6305ba387f2e28945aee71df650ac1d3eb/> sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0

### Other

 - <csr-id-ad7aa2d27c1eeeb11734f5cc2712383a36343d54/> create separate sn_interface unit test step

### New Features (BREAKING)

 - <csr-id-c1ee1dbb50fb8128776b4ba0a821e23056801201/> integrate membership into safe-network

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 8 calendar days.
 - 9 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0 ([`2f4e7e6`](https://github.com/maidsafe/safe_network/commit/2f4e7e6305ba387f2e28945aee71df650ac1d3eb))
    - tidy references in cargo manifests ([`318ee1d`](https://github.com/maidsafe/safe_network/commit/318ee1d22970b5f06e93a99b6e8fff6da638c589))
    - pull sn_client out of the node codebase ([`88421d9`](https://github.com/maidsafe/safe_network/commit/88421d9cb7872b6397283a0035130bc14de6d4ff))
    - integrate membership into safe-network ([`c1ee1db`](https://github.com/maidsafe/safe_network/commit/c1ee1dbb50fb8128776b4ba0a821e23056801201))
    - sn_dysfunction-0.1.1/safe_network-0.58.13/sn_api-0.58.2/sn_cli-0.51.3 ([`9ea06ff`](https://github.com/maidsafe/safe_network/commit/9ea06ffe9339d3927897f010314b1be1bf7026bf))
    - add changelog/readme for sn_interface publishing ([`f375822`](https://github.com/maidsafe/safe_network/commit/f37582288da65f27f53eb27453a4693166821064))
    - remove unused sn_interface deps ([`7b8ce1c`](https://github.com/maidsafe/safe_network/commit/7b8ce1c9d980015768a300ac99d07f69cc1f5ae3))
    - create separate sn_interface unit test step ([`ad7aa2d`](https://github.com/maidsafe/safe_network/commit/ad7aa2d27c1eeeb11734f5cc2712383a36343d54))
    - split put messaging and types into top level crate ([`8494a01`](https://github.com/maidsafe/safe_network/commit/8494a01d9be3dddc0d0f4c2c37cdc4d6c3e54521))
</details>

## v0.1.1 (2022-04-14)

<csr-id-f37582288da65f27f53eb27453a4693166821064/>
<csr-id-7b8ce1c9d980015768a300ac99d07f69cc1f5ae3/>
<csr-id-8494a01d9be3dddc0d0f4c2c37cdc4d6c3e54521/>
<csr-id-ad7aa2d27c1eeeb11734f5cc2712383a36343d54/>

### Chore

 - <csr-id-f37582288da65f27f53eb27453a4693166821064/> add changelog/readme for sn_interface publishing
 - <csr-id-7b8ce1c9d980015768a300ac99d07f69cc1f5ae3/> remove unused sn_interface deps
 - <csr-id-8494a01d9be3dddc0d0f4c2c37cdc4d6c3e54521/> split put messaging and types into top level crate

### Other

 - <csr-id-ad7aa2d27c1eeeb11734f5cc2712383a36343d54/> create separate sn_interface unit test step

## v0.1.0 (2022-04-14)

This first version is being edited manually to trigger a release and publish of the first crate.

Inserting another manual change for testing purposes.

