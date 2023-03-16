# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.1 (2023-03-16)

### Chore

 - <csr-id-807d69ef609decfe94230e2086144afc5cc56d7b/> sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1
 - <csr-id-1a8b9c9ba5b98c0f1176a0ccbce53d4acea8c84c/> safenode renaming

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Revert "chore(release): sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1" ([`a24dca6`](https://github.com/maidsafe/safe_network/commit/a24dca63d1fde8c5e13fa7bbfadf71cda15af5c5))
    - Sn_interface-0.20.6/sn_comms-0.6.3/sn_client-0.82.3/sn_node-0.79.0/sn_cli-0.74.1 ([`807d69e`](https://github.com/maidsafe/safe_network/commit/807d69ef609decfe94230e2086144afc5cc56d7b))
    - Safenode renaming ([`1a8b9c9`](https://github.com/maidsafe/safe_network/commit/1a8b9c9ba5b98c0f1176a0ccbce53d4acea8c84c))
</details>

## v0.1.0 (2023-03-16)

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

