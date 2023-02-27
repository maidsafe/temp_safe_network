# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.5.0 (2023-02-27)

## v0.4.0 (2023-02-24)

### Chore

 - <csr-id-444e4e5969be16129fb87ae42927e183ac41982b/> sn_interface-0.18.0/sn_comms-0.4.0/sn_client-0.80.0/sn_node-0.76.0/sn_api-0.78.0/sn_cli-0.71.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.18.0/sn_comms-0.4.0/sn_client-0.80.0/sn_node-0.76.0/sn_api-0.78.0/sn_cli-0.71.0 ([`444e4e5`](https://github.com/maidsafe/safe_network/commit/444e4e5969be16129fb87ae42927e183ac41982b))
</details>

## v0.3.5 (2023-02-24)

<csr-id-c112a24847ec6800da444192b35a78a75af65de1/>
<csr-id-5a29d0d8ad6853e4bb46bd4c122a8fe80dd2cde2/>
<csr-id-67867b1379b9225f4be3d584ea2df5c3b0afca3a/>
<csr-id-a6a04247785e973a639ed2a4ccc385d941d65756/>
<csr-id-d682cef91723d778501323aef1f03818d0425ee7/>
<csr-id-e513ab35693a86393925ed5e529dcede1bdbe8b3/>
<csr-id-679591e53ed65fa3f0d78f15b5054cd05085e8d9/>
<csr-id-0d55437399624692c6e5cfc8363a6a630ed13019/>

### Chore

 - <csr-id-c112a24847ec6800da444192b35a78a75af65de1/> allow processing to backpressure down to qp2p
 - <csr-id-5a29d0d8ad6853e4bb46bd4c122a8fe80dd2cde2/> update for qp2p recv stream ownership changes
 - <csr-id-67867b1379b9225f4be3d584ea2df5c3b0afca3a/> sn_interface-0.17.10/sn_comms-0.3.5/sn_client-0.79.0/sn_node-0.75.0/sn_api-0.77.0/sn_cli-0.70.0
 - <csr-id-a6a04247785e973a639ed2a4ccc385d941d65756/> remove unnecessary clippy lints

### Chore

 - <csr-id-0d55437399624692c6e5cfc8363a6a630ed13019/> sn_interface-0.17.10/sn_comms-0.3.5/sn_client-0.79.0/sn_node-0.75.0/sn_api-0.77.0/sn_cli-0.70.0

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 18 commits contributed to the release over the course of 8 calendar days.
 - 10 days passed between releases.
 - 8 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.17.10/sn_comms-0.3.5/sn_client-0.79.0/sn_node-0.75.0/sn_api-0.77.0/sn_cli-0.70.0 ([`0d55437`](https://github.com/maidsafe/safe_network/commit/0d55437399624692c6e5cfc8363a6a630ed13019))
    - Revert "chore(release): sn_interface-0.17.10/sn_comms-0.3.5/sn_client-0.79.0/sn_node-0.75.0/sn_api-0.77.0/sn_cli-0.70.0" ([`7d41c76`](https://github.com/maidsafe/safe_network/commit/7d41c763221d52e44e0f3faefbbb0a4d4aeca0a2))
    - Merge #2065 #2117 ([`7f4f814`](https://github.com/maidsafe/safe_network/commit/7f4f8144f68ea235e2508699a7e843d0004028e1))
    - Merge branch 'main' into UseParkingLotDeadlockDetection ([`4eddb41`](https://github.com/maidsafe/safe_network/commit/4eddb41639b8845ed7567d8518199944de62f907))
    - Allow processing to backpressure down to qp2p ([`c112a24`](https://github.com/maidsafe/safe_network/commit/c112a24847ec6800da444192b35a78a75af65de1))
    - Update for qp2p recv stream ownership changes ([`5a29d0d`](https://github.com/maidsafe/safe_network/commit/5a29d0d8ad6853e4bb46bd4c122a8fe80dd2cde2))
    - Pass public addr in --first ([`d682cef`](https://github.com/maidsafe/safe_network/commit/d682cef91723d778501323aef1f03818d0425ee7))
    - Pass public addr in --first ([`e513ab3`](https://github.com/maidsafe/safe_network/commit/e513ab35693a86393925ed5e529dcede1bdbe8b3))
    - Merge #2087 #2107 ([`be64f75`](https://github.com/maidsafe/safe_network/commit/be64f75991cbe72899dd7bde6aab8c1ed66aaae9))
    - Merge branch 'main' into dbc_without_ringct ([`803b158`](https://github.com/maidsafe/safe_network/commit/803b1581880f24267f5b7389cac9e268d42c5702))
    - Chore(general): renaming variants and types - This better reflects the domain. ([`9d126b6`](https://github.com/maidsafe/safe_network/commit/9d126b60e2ac72b7bce0baa0de9b68f2f85e5e56))
    - Merge #2092 ([`82057ec`](https://github.com/maidsafe/safe_network/commit/82057ecb0875217efa47f0bcfaad48b43d29d8aa))
    - Merge branch 'main' into dbc_without_ringct ([`ca4781b`](https://github.com/maidsafe/safe_network/commit/ca4781b551fb40edc71f199c00097eb83ef7cb7b))
    - Split out ae ([`679591e`](https://github.com/maidsafe/safe_network/commit/679591e53ed65fa3f0d78f15b5054cd05085e8d9))
    - Sn_interface-0.17.10/sn_comms-0.3.5/sn_client-0.79.0/sn_node-0.75.0/sn_api-0.77.0/sn_cli-0.70.0 ([`67867b1`](https://github.com/maidsafe/safe_network/commit/67867b1379b9225f4be3d584ea2df5c3b0afca3a))
    - Merge branch 'main' into dbc_without_ringct ([`f4bfef2`](https://github.com/maidsafe/safe_network/commit/f4bfef20db8c718aacef188f0150e07673eba1b0))
    - Merge #2105 ([`33a6f3f`](https://github.com/maidsafe/safe_network/commit/33a6f3f5bc18708ad1027f71f4de5fefe26bd209))
    - Remove unnecessary clippy lints ([`a6a0424`](https://github.com/maidsafe/safe_network/commit/a6a04247785e973a639ed2a4ccc385d941d65756))
</details>

## v0.3.4 (2023-02-14)

<csr-id-b9478767acab3f612d97647384c171837cb15811/>

### Chore

 - <csr-id-b9478767acab3f612d97647384c171837cb15811/> sn_comms-0.3.4

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 3 calendar days.
 - 4 days passed between releases.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_comms-0.3.4 ([`b947876`](https://github.com/maidsafe/safe_network/commit/b9478767acab3f612d97647384c171837cb15811))
    - Merge #2093 ([`2bb2b21`](https://github.com/maidsafe/safe_network/commit/2bb2b2154fc4f26202f216bd42f3faf88a13b876))
    - Refactor(comms): simplify peer_session - Removes some unnecessary spawning and channel faff, since we're already on a new thread. ([`a4103fe`](https://github.com/maidsafe/safe_network/commit/a4103fe2f7124188fd17ba3d5f54ac8f6132529b))
</details>

## v0.3.3 (2023-02-09)

<csr-id-17ca03749dc6eed8922cde5a7c3ef8a8e5505483/>
<csr-id-23903c3e7ee462676bd25e3618a4b313c7b5cf3b/>

### Chore

 - <csr-id-17ca03749dc6eed8922cde5a7c3ef8a8e5505483/> sn_interface-0.17.9/sn_comms-0.3.3/sn_node-0.74.11
 - <csr-id-23903c3e7ee462676bd25e3618a4b313c7b5cf3b/> minor cleanup, docs and log fixes

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.17.9/sn_comms-0.3.3/sn_node-0.74.11 ([`17ca037`](https://github.com/maidsafe/safe_network/commit/17ca03749dc6eed8922cde5a7c3ef8a8e5505483))
    - Merge #2043 ([`bef44e5`](https://github.com/maidsafe/safe_network/commit/bef44e53b55ddbe957b77755ad97f561b35f69e7))
    - Refactor(comms): send single response to client - All results from storage nodes are awaited, and a single response returned to client. ([`39d5574`](https://github.com/maidsafe/safe_network/commit/39d55747512fc233a92c1e8ab310f984aebc1d4f))
    - Minor cleanup, docs and log fixes ([`23903c3`](https://github.com/maidsafe/safe_network/commit/23903c3e7ee462676bd25e3618a4b313c7b5cf3b))
    - Refactor(comms): send to nodes in parallel - Previous commit had the msgs go out sequentially to the nodes. - Also, we didn't send any response to client until all nodes had responded. ([`e4318d1`](https://github.com/maidsafe/safe_network/commit/e4318d14381f71ea7b46957f99dde4ba736774c9))
    - Refactor(comms): don't use dashmap for sessions - This allows a lockfree access to sessions in comms. ([`15670eb`](https://github.com/maidsafe/safe_network/commit/15670eba1ab9afeaebd5abd91551834e59b4c14d))
</details>

## v0.3.2 (2023-02-07)

<csr-id-6cb1f9548ce44aaf04c9d6c64364ca1c8b344470/>
<csr-id-b7a6024af9e777473615cddfd5940f84fda4bb6b/>

### Chore

 - <csr-id-6cb1f9548ce44aaf04c9d6c64364ca1c8b344470/> remove unused async

### Chore

 - <csr-id-b7a6024af9e777473615cddfd5940f84fda4bb6b/> sn_comms-0.3.2/sn_node-0.74.4

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_comms-0.3.2/sn_node-0.74.4 ([`b7a6024`](https://github.com/maidsafe/safe_network/commit/b7a6024af9e777473615cddfd5940f84fda4bb6b))
    - Merge #2073 ([`732621c`](https://github.com/maidsafe/safe_network/commit/732621c1261dd24eb58f38b49ba25c78af272c69))
    - Remove unused async ([`6cb1f95`](https://github.com/maidsafe/safe_network/commit/6cb1f9548ce44aaf04c9d6c64364ca1c8b344470))
</details>

## v0.3.1 (2023-02-07)

<csr-id-677ef5cc8b1935b94641c61c53429faf2c58c261/>
<csr-id-3c34a731eca9d5b37d2574e3e16c7f089c7cc8b2/>

### Chore

 - <csr-id-677ef5cc8b1935b94641c61c53429faf2c58c261/> update various log levels

### Chore

 - <csr-id-3c34a731eca9d5b37d2574e3e16c7f089c7cc8b2/> sn_interface-0.17.4/sn_fault_detection-0.15.4/sn_comms-0.3.1/sn_node-0.74.2

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.17.4/sn_fault_detection-0.15.4/sn_comms-0.3.1/sn_node-0.74.2 ([`3c34a73`](https://github.com/maidsafe/safe_network/commit/3c34a731eca9d5b37d2574e3e16c7f089c7cc8b2))
    - Merge #2066 ([`023ed9b`](https://github.com/maidsafe/safe_network/commit/023ed9b313d050bd43869eb871697716fea888d8))
    - Update various log levels ([`677ef5c`](https://github.com/maidsafe/safe_network/commit/677ef5cc8b1935b94641c61c53429faf2c58c261))
</details>

## v0.3.0 (2023-02-06)

<csr-id-e967cc4d827c460bb47748decdf564c9cf7e1e6d/>
<csr-id-58a0608722ca8f4a8363c1f9221ac47f5838d158/>

### Chore

 - <csr-id-e967cc4d827c460bb47748decdf564c9cf7e1e6d/> sn_interface-0.17.3/sn_comms-0.3.0/sn_client-0.78.4/sn_node-0.74.0/sn_cli-0.69.2

### New Features

 - <csr-id-7a5d6975153f9d78e742e0a799919852bcfc33ab/> pass client msgs onwards with no deserialisation

### Chore (BREAKING)

 - <csr-id-58a0608722ca8f4a8363c1f9221ac47f5838d158/> remove unused NodeDataResponse

### New Features (BREAKING)

 - <csr-id-af38f56c7e76a076f0accca7d44a74c055dd74e1/> remove DataQueryVariant

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 4 calendar days.
 - 3 days passed between releases.
 - 4 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.17.3/sn_comms-0.3.0/sn_client-0.78.4/sn_node-0.74.0/sn_cli-0.69.2 ([`e967cc4`](https://github.com/maidsafe/safe_network/commit/e967cc4d827c460bb47748decdf564c9cf7e1e6d))
    - Merge #2048 ([`ae06c94`](https://github.com/maidsafe/safe_network/commit/ae06c9458ad904863a925f1d2b2e253a67456298))
    - Merge branch 'main' into sap_change_force_dkg_termination ([`876d78a`](https://github.com/maidsafe/safe_network/commit/876d78a911e852b8cc1c33b2130e4cf9b28dd510))
    - Remove unused NodeDataResponse ([`58a0608`](https://github.com/maidsafe/safe_network/commit/58a0608722ca8f4a8363c1f9221ac47f5838d158))
    - Remove DataQueryVariant ([`af38f56`](https://github.com/maidsafe/safe_network/commit/af38f56c7e76a076f0accca7d44a74c055dd74e1))
    - Pass client msgs onwards with no deserialisation ([`7a5d697`](https://github.com/maidsafe/safe_network/commit/7a5d6975153f9d78e742e0a799919852bcfc33ab))
</details>

## v0.2.1 (2023-02-02)

<csr-id-e706848522d6c52d6ed5eddf638376996cc947a9/>
<csr-id-3831dae3e34623ef252298645a43cbafcc923a13/>

### Chore

 - <csr-id-e706848522d6c52d6ed5eddf638376996cc947a9/> add clippy check for unused async

### Chore

 - <csr-id-3831dae3e34623ef252298645a43cbafcc923a13/> sn_interface-0.17.1/sn_fault_detection-0.15.3/sn_comms-0.2.1/sn_client-0.78.2/sn_node-0.73.3/sn_api-0.76.1

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 1 day passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.17.1/sn_fault_detection-0.15.3/sn_comms-0.2.1/sn_client-0.78.2/sn_node-0.73.3/sn_api-0.76.1 ([`3831dae`](https://github.com/maidsafe/safe_network/commit/3831dae3e34623ef252298645a43cbafcc923a13))
    - Merge #2061 ([`bab8208`](https://github.com/maidsafe/safe_network/commit/bab82087260ac4f1f44e688db2e67ca2387a7175))
    - Add clippy check for unused async ([`e706848`](https://github.com/maidsafe/safe_network/commit/e706848522d6c52d6ed5eddf638376996cc947a9))
    - Merge branch 'main' into sap_change_force_dkg_termination ([`7d3665b`](https://github.com/maidsafe/safe_network/commit/7d3665bfe05f61d170229df9f4424c5663b116d5))
</details>

## v0.2.0 (2023-02-01)

<csr-id-69f8ade1ea8bb3e77c169b17ae21a40370bfab58/>
<csr-id-cee5c65a1a099606d5430452995d26edfd1f6bfc/>
<csr-id-f779144986a6b2b06f550d3a2a4cbc39c64af83d/>
<csr-id-47e0f87d5ccad33cfa82ef80f3648fe8270acaaa/>
<csr-id-9ef9a2f2c8711895b62b82d25cb9d208c464cad6/>

### Chore

 - <csr-id-69f8ade1ea8bb3e77c169b17ae21a40370bfab58/> sn_interface-0.17.0/sn_comms-0.2.0/sn_client-0.78.0/sn_node-0.73.0/sn_api-0.76.0/sn_cli-0.69.0

### Refactor

 - <csr-id-cee5c65a1a099606d5430452995d26edfd1f6bfc/> leave out reachability check for join
   This reachability check tries to reply/reach out to the node that could
   not connect, but we only save incoming connections, so this will not
   make it out to the node.
   
   Also, the reachability check was left out of the last release of qp2p,
   as the network should not only rely on these reachability checks, but
   the nodes should be unable to join the network anyway if they're not
   reachable in the first place.
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

 - 6 commits contributed to the release.
 - 4 days passed between releases.
 - 5 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.17.0/sn_comms-0.2.0/sn_client-0.78.0/sn_node-0.73.0/sn_api-0.76.0/sn_cli-0.69.0 ([`69f8ade`](https://github.com/maidsafe/safe_network/commit/69f8ade1ea8bb3e77c169b17ae21a40370bfab58))
    - Merge #1996 ([`bb7b2db`](https://github.com/maidsafe/safe_network/commit/bb7b2dbcae9c0a67fc0a23c279537df49d88a07a))
    - Leave out reachability check for join ([`cee5c65`](https://github.com/maidsafe/safe_network/commit/cee5c65a1a099606d5430452995d26edfd1f6bfc))
    - Idle_timeout from 10s to 70s ([`f779144`](https://github.com/maidsafe/safe_network/commit/f779144986a6b2b06f550d3a2a4cbc39c64af83d))
    - Remove passing parameters to qp2p ([`47e0f87`](https://github.com/maidsafe/safe_network/commit/47e0f87d5ccad33cfa82ef80f3648fe8270acaaa))
    - Implement new qp2p version ([`9ef9a2f`](https://github.com/maidsafe/safe_network/commit/9ef9a2f2c8711895b62b82d25cb9d208c464cad6))
</details>

## v0.1.7 (2023-01-27)

<csr-id-4fa50e710c65dc4298f85f6eb01a3575155417d6/>
<csr-id-acfc8c88d5fbc15f46d76535c058c60b6d20433a/>

### Refactor

 - <csr-id-4fa50e710c65dc4298f85f6eb01a3575155417d6/> removing unnecessary SendStatus and SendWatcher

### Chore

 - <csr-id-acfc8c88d5fbc15f46d76535c058c60b6d20433a/> sn_comms-0.1.7/sn_node-0.72.38

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_comms-0.1.7/sn_node-0.72.38 ([`acfc8c8`](https://github.com/maidsafe/safe_network/commit/acfc8c88d5fbc15f46d76535c058c60b6d20433a))
    - Merge #2038 ([`8f1c443`](https://github.com/maidsafe/safe_network/commit/8f1c443a29f794da6a0412eab87672281c3a4d4b))
    - Removing unnecessary SendStatus and SendWatcher ([`4fa50e7`](https://github.com/maidsafe/safe_network/commit/4fa50e710c65dc4298f85f6eb01a3575155417d6))
</details>

## v0.1.6 (2023-01-27)

<csr-id-0304e4904dd901cbf24643a5803a190c87c2048d/>
<csr-id-e990f883bec55e5e3c73a3b074428c42d2538785/>

### Refactor

 - <csr-id-0304e4904dd901cbf24643a5803a190c87c2048d/> simplifying complexity by removing Link mod

### Chore

 - <csr-id-e990f883bec55e5e3c73a3b074428c42d2538785/> sn_comms-0.1.6/sn_node-0.72.37

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_comms-0.1.6/sn_node-0.72.37 ([`e990f88`](https://github.com/maidsafe/safe_network/commit/e990f883bec55e5e3c73a3b074428c42d2538785))
    - Merge #2013 #2029 ([`3456929`](https://github.com/maidsafe/safe_network/commit/3456929564e00303315da6b458d5fc4f97422301))
    - Simplifying complexity by removing Link mod ([`0304e49`](https://github.com/maidsafe/safe_network/commit/0304e4904dd901cbf24643a5803a190c87c2048d))
</details>

## v0.1.5 (2023-01-27)

<csr-id-12f9f764dc821d78b39073fe007a3a6ac32d70cb/>

### Chore

 - <csr-id-12f9f764dc821d78b39073fe007a3a6ac32d70cb/> sn_comms-0.1.5/sn_node-0.72.36

### Bug Fixes

 - <csr-id-a450e56be02410e00521afa1b4070f0de014c0ab/> merge issues

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_comms-0.1.5/sn_node-0.72.36 ([`12f9f76`](https://github.com/maidsafe/safe_network/commit/12f9f764dc821d78b39073fe007a3a6ac32d70cb))
    - Merge #2031 ([`96d8799`](https://github.com/maidsafe/safe_network/commit/96d8799cf10510c9d3514fdd9f6fdfc628568da3))
    - Merge issues ([`a450e56`](https://github.com/maidsafe/safe_network/commit/a450e56be02410e00521afa1b4070f0de014c0ab))
</details>

## v0.1.4 (2023-01-27)

<csr-id-6b923515f0f0cd1b0d1a4ed51d3ca418e41d9916/>
<csr-id-01ff2ccf45dfc9d45c5ad540144d7a4a640830fc/>

### Chore

 - <csr-id-6b923515f0f0cd1b0d1a4ed51d3ca418e41d9916/> fix issues reported by new clippy

### Chore

 - <csr-id-01ff2ccf45dfc9d45c5ad540144d7a4a640830fc/> sn_interface-0.16.18/sn_comms-0.1.4/sn_client-0.77.9/sn_node-0.72.34/sn_api-0.75.5/sn_cli-0.68.6

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
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

## v0.1.3 (2023-01-26)

<csr-id-d51dd695437dac1695447491d4f298334b7e0fd1/>

### Chore

 - <csr-id-d51dd695437dac1695447491d4f298334b7e0fd1/> sn_comms-0.1.3/sn_node-0.72.33

### Bug Fixes

 - <csr-id-42f2c3709af96207b10b711878d03d42781bfdba/> send_out_bytes was not reporting send failures
   - sn_comms::Comm::send_out_bytes was spawning a task when sending a msg,
   now it's the caller's duty to do so if ever required.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_comms-0.1.3/sn_node-0.72.33 ([`d51dd69`](https://github.com/maidsafe/safe_network/commit/d51dd695437dac1695447491d4f298334b7e0fd1))
    - Merge #2025 ([`4baaae3`](https://github.com/maidsafe/safe_network/commit/4baaae3022d0295715e58f6f74bac3c6b2547be1))
    - Send_out_bytes was not reporting send failures ([`42f2c37`](https://github.com/maidsafe/safe_network/commit/42f2c3709af96207b10b711878d03d42781bfdba))
    - Chore(comm): remove unused async - Also shortens fn name and clarifies docs a bit. ([`3eced25`](https://github.com/maidsafe/safe_network/commit/3eced25805febe313d3d612756931fd52b0d67b0))
</details>

## v0.1.2 (2023-01-25)

<csr-id-a4d295ccdddea3d4d11bca5eb0236a5447c75633/>
<csr-id-6ba7b5a12ed8d15fb807524ee90dc250068c1004/>

### Chore

 - <csr-id-a4d295ccdddea3d4d11bca5eb0236a5447c75633/> sn_interface-0.16.17/sn_comms-0.1.2/sn_node-0.72.30

### Refactor

 - <csr-id-6ba7b5a12ed8d15fb807524ee90dc250068c1004/> removing Comm::members and unnecessary private types
   - We now use Comm::sessions as the list of members we keep sessions with,
   which is updated only when the user wants to update the set of known peers.
   - Removing unnecessary PeerSession's SessionStatus, disconnect fn, and SessionCmd.
   - Never remove sessions from Comm::sessions unless the set of known members is
   updated/changed by the user. Even if we failed to send using all peer session's connections,
   we keep the session since it's been set as a known and connectable peer.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 2 days passed between releases.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_interface-0.16.17/sn_comms-0.1.2/sn_node-0.72.30 ([`a4d295c`](https://github.com/maidsafe/safe_network/commit/a4d295ccdddea3d4d11bca5eb0236a5447c75633))
    - Merge #2022 ([`3a99b2b`](https://github.com/maidsafe/safe_network/commit/3a99b2b616cfd3a90d271868e502d795790b2af0))
    - Removing Comm::members and unnecessary private types ([`6ba7b5a`](https://github.com/maidsafe/safe_network/commit/6ba7b5a12ed8d15fb807524ee90dc250068c1004))
    - Merge #2016 #2019 #2023 ([`c8e5746`](https://github.com/maidsafe/safe_network/commit/c8e574687ea74ed1adb69a722afe6bff734c19ad))
</details>

## v0.1.1 (2023-01-23)

<csr-id-b6ee82b6f5abf129e3e0d84e60e15272479e0db2/>
<csr-id-61d2bc2d35e3f829d58af736a722d01cd86864b6/>
<csr-id-783d62461a65eb7c06b0d4f399b97216b6c75519/>
<csr-id-95ae6f9e8e30184a24465a35626288af64d7995e/>
<csr-id-b8dbee25acfd5b0f348f06419f8058742f575953/>
<csr-id-dbfa4ac0dd23e76060b8df44c4666a30bb9b317f/>
<csr-id-a86d5ad1f352c9000488197ece8edb716941d601/>
<csr-id-9aaf91bfedd9fcf040f18e0762ff2cbbc03b4d6f/>
<csr-id-0ab0c302dcc6ce32b0b71d696b0707a2c50cfa3a/>
<csr-id-12a6620525a5767d906037a74caf0e38af3da596/>

### Chore

 - <csr-id-b6ee82b6f5abf129e3e0d84e60e15272479e0db2/> remove ConnectionEvents and listener
   They are no longer needed.
   Instead we listen to incoming msgs only, but do not keep
   any incoming connections at all. So nothing to clean up
   there now.
 - <csr-id-61d2bc2d35e3f829d58af736a722d01cd86864b6/> only cache connections created by us
 - <csr-id-783d62461a65eb7c06b0d4f399b97216b6c75519/> sn_interface-0.16.14/sn_client-0.77.8/sn_node-0.72.25/sn_api-0.75.4/sn_cli-0.68.5
 - <csr-id-95ae6f9e8e30184a24465a35626288af64d7995e/> add Peer to UnknownPeer conn Err
 - <csr-id-b8dbee25acfd5b0f348f06419f8058742f575953/> remove Option around send watcher
 - <csr-id-dbfa4ac0dd23e76060b8df44c4666a30bb9b317f/> replace node comms with sn_comms
 - <csr-id-a86d5ad1f352c9000488197ece8edb716941d601/> make a new crate for comm

### Chore

 - <csr-id-0ab0c302dcc6ce32b0b71d696b0707a2c50cfa3a/> sn_comms-0.1.1/sn_node-0.72.26
 - <csr-id-12a6620525a5767d906037a74caf0e38af3da596/> rename update_members to update_valid_comm_targets for clarity

### New Features

 - <csr-id-b5e57d5b3d91849074a90f5ba671d9b19b7e4461/> prohibit creating PeerSession to unkown node
 - <csr-id-3d8070155bb88b7403ae97730b33510b8c3af685/> introducing Cmd::SendNodeMsgResponse for NodeMsg responses to nodes over streams
   - Having this internal sn_node::Cmd to handle sending msg responses to nodes over
   a response bi-stream allows us to decouple such logic from the rest, but it also
   allows us to have unit tests within sn_node which verify the outcome of processing
   Cmds without sending any msg over the wire.

### Bug Fixes

 - <csr-id-28cdebb4b05c5d64dcbe8dfb39a72c88fd2c28bd/> update comm members on AE msg in
 - <csr-id-0cbcc1dddf7db229b7fb81328108a076263343d2/> handle sending msgs off the incoming msg loop
   This should prevent us blocking qp2p should a channel itself be blocked

### Refactor

 - <csr-id-9aaf91bfedd9fcf040f18e0762ff2cbbc03b4d6f/> forward client data cmds/queries to holders through Cmd::SendMsgAndAwaitResponse
   - Unifying and simplifying logic to send client data cmds and queries to holders so in both
   cases the sn_node `Cmd::SendMsgAndAwaitResponse` is used.
   - Renaming `sn_comms::Error::CmdSendError` to `SendError` since it's not specific for
   cmds but for any msg.
   - Some internal sn_node helper functions were moved to different files/mods so they are closer
   to the logic making use of them.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 22 commits contributed to the release over the course of 13 calendar days.
 - 14 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Sn_comms-0.1.1/sn_node-0.72.26 ([`0ab0c30`](https://github.com/maidsafe/safe_network/commit/0ab0c302dcc6ce32b0b71d696b0707a2c50cfa3a))
    - Merge #2009 ([`83448f4`](https://github.com/maidsafe/safe_network/commit/83448f43dace53b3357796bf177edb98c3d5803d))
    - Rename update_members to update_valid_comm_targets for clarity ([`12a6620`](https://github.com/maidsafe/safe_network/commit/12a6620525a5767d906037a74caf0e38af3da596))
    - Remove ConnectionEvents and listener ([`b6ee82b`](https://github.com/maidsafe/safe_network/commit/b6ee82b6f5abf129e3e0d84e60e15272479e0db2))
    - Only cache connections created by us ([`61d2bc2`](https://github.com/maidsafe/safe_network/commit/61d2bc2d35e3f829d58af736a722d01cd86864b6))
    - Sn_interface-0.16.14/sn_client-0.77.8/sn_node-0.72.25/sn_api-0.75.4/sn_cli-0.68.5 ([`783d624`](https://github.com/maidsafe/safe_network/commit/783d62461a65eb7c06b0d4f399b97216b6c75519))
    - Merge #2008 ([`ffac6c6`](https://github.com/maidsafe/safe_network/commit/ffac6c68dc0612a41aa74c533231a63006c22b22))
    - Update comm members on AE msg in ([`28cdebb`](https://github.com/maidsafe/safe_network/commit/28cdebb4b05c5d64dcbe8dfb39a72c88fd2c28bd))
    - Merge #1997 #1998 #2002 ([`0c968ad`](https://github.com/maidsafe/safe_network/commit/0c968ad50d9e9dada3f5f5488bd1708fddadef72))
    - Add Peer to UnknownPeer conn Err ([`95ae6f9`](https://github.com/maidsafe/safe_network/commit/95ae6f9e8e30184a24465a35626288af64d7995e))
    - Merge #2001 ([`b306800`](https://github.com/maidsafe/safe_network/commit/b30680008443dbb89d68567f54cb789c72423af2))
    - Remove Option around send watcher ([`b8dbee2`](https://github.com/maidsafe/safe_network/commit/b8dbee25acfd5b0f348f06419f8058742f575953))
    - Prohibit creating PeerSession to unkown node ([`b5e57d5`](https://github.com/maidsafe/safe_network/commit/b5e57d5b3d91849074a90f5ba671d9b19b7e4461))
    - Handle sending msgs off the incoming msg loop ([`0cbcc1d`](https://github.com/maidsafe/safe_network/commit/0cbcc1dddf7db229b7fb81328108a076263343d2))
    - Merge #1978 ([`fde6710`](https://github.com/maidsafe/safe_network/commit/fde67106242ad3d47f04ce99261a1e6299e94047))
    - Forward client data cmds/queries to holders through Cmd::SendMsgAndAwaitResponse ([`9aaf91b`](https://github.com/maidsafe/safe_network/commit/9aaf91bfedd9fcf040f18e0762ff2cbbc03b4d6f))
    - Merge #1975 ([`635a1b2`](https://github.com/maidsafe/safe_network/commit/635a1b29c9f8be3f708c6670de51ce68c0d34663))
    - Introducing Cmd::SendNodeMsgResponse for NodeMsg responses to nodes over streams ([`3d80701`](https://github.com/maidsafe/safe_network/commit/3d8070155bb88b7403ae97730b33510b8c3af685))
    - Merge #1973 ([`f308b44`](https://github.com/maidsafe/safe_network/commit/f308b44fbc8cb0b669ed129727e638285ba65f1d))
    - Fix(tests): add feat flag to call test fn from ext - As we test comms in sn_node, now when in another crate, cfg(test) is not detected, and we solve that by adding the dev-dep with a feat flag. ([`76b5e75`](https://github.com/maidsafe/safe_network/commit/76b5e75af26e4a25dcc7f8e0b58e842350339b02))
    - Replace node comms with sn_comms ([`dbfa4ac`](https://github.com/maidsafe/safe_network/commit/dbfa4ac0dd23e76060b8df44c4666a30bb9b317f))
    - Make a new crate for comm ([`a86d5ad`](https://github.com/maidsafe/safe_network/commit/a86d5ad1f352c9000488197ece8edb716941d601))
</details>

## v0.1.0 (2023-01-20)

<csr-id-95ae6f9e8e30184a24465a35626288af64d7995e/>
<csr-id-b8dbee25acfd5b0f348f06419f8058742f575953/>
<csr-id-dbfa4ac0dd23e76060b8df44c4666a30bb9b317f/>
<csr-id-a86d5ad1f352c9000488197ece8edb716941d601/>
<csr-id-9aaf91bfedd9fcf040f18e0762ff2cbbc03b4d6f/>

### Chore

 - <csr-id-95ae6f9e8e30184a24465a35626288af64d7995e/> add Peer to UnknownPeer conn Err
 - <csr-id-b8dbee25acfd5b0f348f06419f8058742f575953/> remove Option around send watcher
 - <csr-id-dbfa4ac0dd23e76060b8df44c4666a30bb9b317f/> replace node comms with sn_comms
 - <csr-id-a86d5ad1f352c9000488197ece8edb716941d601/> make a new crate for comm

### New Features

 - <csr-id-b5e57d5b3d91849074a90f5ba671d9b19b7e4461/> prohibit creating PeerSession to unkown node
 - <csr-id-3d8070155bb88b7403ae97730b33510b8c3af685/> introducing Cmd::SendNodeMsgResponse for NodeMsg responses to nodes over streams
   - Having this internal sn_node::Cmd to handle sending msg responses to nodes over
   a response bi-stream allows us to decouple such logic from the rest, but it also
   allows us to have unit tests within sn_node which verify the outcome of processing
   Cmds without sending any msg over the wire.

### Bug Fixes

 - <csr-id-28cdebb4b05c5d64dcbe8dfb39a72c88fd2c28bd/> update comm members on AE msg in
 - <csr-id-0cbcc1dddf7db229b7fb81328108a076263343d2/> handle sending msgs off the incoming msg loop
   This should prevent us blocking qp2p should a channel itself be blocked

### Refactor

 - <csr-id-9aaf91bfedd9fcf040f18e0762ff2cbbc03b4d6f/> forward client data cmds/queries to holders through Cmd::SendMsgAndAwaitResponse
   - Unifying and simplifying logic to send client data cmds and queries to holders so in both
   cases the sn_node `Cmd::SendMsgAndAwaitResponse` is used.
   - Renaming `sn_comms::Error::CmdSendError` to `SendError` since it's not specific for
   cmds but for any msg.
   - Some internal sn_node helper functions were moved to different files/mods so they are closer
   to the logic making use of them.

<csr-unknown>
We are here also changing Cmd::SendMsg to make/restricting it exclusively forsending msgs to nodes over uni-streams.<csr-unknown/>
<csr-unknown/>

