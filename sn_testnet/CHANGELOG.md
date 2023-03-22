# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.3 (2023-03-22)

### Chore

 - <csr-id-b0627339e2458fd762084cc4805d7adedfd8c05e/> sn_testnet-0.1.3/sn_interface-0.20.7/sn_comms-0.6.4/sn_client-0.82.4/sn_node-0.80.1/sn_api-0.80.3/sn_cli-0.74.2
 - <csr-id-c9f3e7ccad8836c609193f1c6b53f351e5705805/> sn_node-0.80.0
 - <csr-id-50f6ede2104025bd79de8922ca7f27c742cf52bb/> sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1
 - <csr-id-807d69ef609decfe94230e2086144afc5cc56d7b/> sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1
 - <csr-id-1a8b9c9ba5b98c0f1176a0ccbce53d4acea8c84c/> safenode renaming

### Refactor

 - <csr-id-d3c6c9727a69389f4204b746c54a537cd783232c/> remove unused wiremsg-debuginfo ft

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 6 calendar days.
 - 6 days passed between releases.
 - 6 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Revert "chore(release): sn_testnet-0.1.3/sn_interface-0.20.7/sn_comms-0.6.4/sn_client-0.82.4/sn_node-0.80.1/sn_api-0.80.3/sn_cli-0.74.2" ([`2e25949`](https://github.com/maidsafe/safe_network/commit/2e25949f685b0b805d8866527232c010380573ce))
    - Sn_testnet-0.1.3/sn_interface-0.20.7/sn_comms-0.6.4/sn_client-0.82.4/sn_node-0.80.1/sn_api-0.80.3/sn_cli-0.74.2 ([`b062733`](https://github.com/maidsafe/safe_network/commit/b0627339e2458fd762084cc4805d7adedfd8c05e))
    - Remove unused wiremsg-debuginfo ft ([`d3c6c97`](https://github.com/maidsafe/safe_network/commit/d3c6c9727a69389f4204b746c54a537cd783232c))
    - Fix(testnet): move os cond. to if-clause body - Running cargo clippy on win machine was borked with the conditional over an error in a fn, as the following code became unreachable. - Placing it in the if clause body removes this clippy error, as the rest of the code in that can still be executed if the flame arg was not set. ([`022fae6`](https://github.com/maidsafe/safe_network/commit/022fae6616a4dbf13d01e53ada76ebf1c9dab7ad))
    - Sn_node-0.80.0 ([`c9f3e7c`](https://github.com/maidsafe/safe_network/commit/c9f3e7ccad8836c609193f1c6b53f351e5705805))
    - Sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1 ([`50f6ede`](https://github.com/maidsafe/safe_network/commit/50f6ede2104025bd79de8922ca7f27c742cf52bb))
    - Revert "chore(release): sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1" ([`a24dca6`](https://github.com/maidsafe/safe_network/commit/a24dca63d1fde8c5e13fa7bbfadf71cda15af5c5))
    - Sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1 ([`807d69e`](https://github.com/maidsafe/safe_network/commit/807d69ef609decfe94230e2086144afc5cc56d7b))
    - Safenode renaming ([`1a8b9c9`](https://github.com/maidsafe/safe_network/commit/1a8b9c9ba5b98c0f1176a0ccbce53d4acea8c84c))
</details>

## v0.1.2 (2023-03-16)

<csr-id-50f6ede2104025bd79de8922ca7f27c742cf52bb/>
<csr-id-807d69ef609decfe94230e2086144afc5cc56d7b/>
<csr-id-1a8b9c9ba5b98c0f1176a0ccbce53d4acea8c84c/>

### Chore

 - <csr-id-50f6ede2104025bd79de8922ca7f27c742cf52bb/> sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1
 - <csr-id-807d69ef609decfe94230e2086144afc5cc56d7b/> sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1
 - <csr-id-1a8b9c9ba5b98c0f1176a0ccbce53d4acea8c84c/> safenode renaming

## v0.1.1 (2023-03-16)

<csr-id-807d69ef609decfe94230e2086144afc5cc56d7b/>
<csr-id-1a8b9c9ba5b98c0f1176a0ccbce53d4acea8c84c/>

### Chore

 - <csr-id-807d69ef609decfe94230e2086144afc5cc56d7b/> sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1
 - <csr-id-1a8b9c9ba5b98c0f1176a0ccbce53d4acea8c84c/> safenode renaming

## v0.1.0 (2023-03-16)

<csr-id-4f04bd1a5d1c747bfc6b5d39824dd108f8546b7b/>
<csr-id-1c621d13b5edfc21ed85da7498d24c5db038795a/>

### Chore

 - <csr-id-4f04bd1a5d1c747bfc6b5d39824dd108f8546b7b/> rename testnet crate to sn_testnet
   Even though the `testnet` crate name is not taken on crates.io, I think it makes sense to prefix
   this crate with `sn_`, as per our other crates. The name of the binary does not change. This crate
   needs to be published because `sn_client` has a dependency on it.
   
   This also provides a README for the crate, which was necessary to have it published.

### Other

 - <csr-id-1c621d13b5edfc21ed85da7498d24c5db038795a/> temporarily prevent workflows running
   I want to temporarily disable the version bump and release workflows from running so that I can
   manually publish the new testnet crate and delete the tags from the last bad release.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Temporarily prevent workflows running ([`1c621d1`](https://github.com/maidsafe/safe_network/commit/1c621d13b5edfc21ed85da7498d24c5db038795a))
    - Rename testnet crate to sn_testnet ([`4f04bd1`](https://github.com/maidsafe/safe_network/commit/4f04bd1a5d1c747bfc6b5d39824dd108f8546b7b))
</details>

