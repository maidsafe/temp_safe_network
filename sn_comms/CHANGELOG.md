# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.3 (2023-01-26)

### Bug Fixes

 - <csr-id-42f2c3709af96207b10b711878d03d42781bfdba/> send_out_bytes was not reporting send failures
   - sn_comms::Comm::send_out_bytes was spawning a task when sending a msg,
   now it's the caller's duty to do so if ever required.
   - Run sn_comms unit tests in CI/Bors.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 1 commit was understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Merge #2025 ([`4baaae3`](https://github.com/maidsafe/safe_network/commit/4baaae3022d0295715e58f6f74bac3c6b2547be1))
    - send_out_bytes was not reporting send failures ([`42f2c37`](https://github.com/maidsafe/safe_network/commit/42f2c3709af96207b10b711878d03d42781bfdba))
</details>

## v0.1.2 (2023-01-25)

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
    - sn_interface-0.16.17/sn_comms-0.1.2/sn_node-0.72.30 ([`a4d295c`](https://github.com/maidsafe/safe_network/commit/a4d295ccdddea3d4d11bca5eb0236a5447c75633))
    - Merge #2022 ([`3a99b2b`](https://github.com/maidsafe/safe_network/commit/3a99b2b616cfd3a90d271868e502d795790b2af0))
    - removing Comm::members and unnecessary private types ([`6ba7b5a`](https://github.com/maidsafe/safe_network/commit/6ba7b5a12ed8d15fb807524ee90dc250068c1004))
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
- We are here also changing Cmd::SendMsg to make/restricting it exclusively for
   sending msgs to nodes over uni-streams.

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
    - sn_comms-0.1.1/sn_node-0.72.26 ([`0ab0c30`](https://github.com/maidsafe/safe_network/commit/0ab0c302dcc6ce32b0b71d696b0707a2c50cfa3a))
    - Merge #2009 ([`83448f4`](https://github.com/maidsafe/safe_network/commit/83448f43dace53b3357796bf177edb98c3d5803d))
    - rename update_members to update_valid_comm_targets for clarity ([`12a6620`](https://github.com/maidsafe/safe_network/commit/12a6620525a5767d906037a74caf0e38af3da596))
    - remove ConnectionEvents and listener ([`b6ee82b`](https://github.com/maidsafe/safe_network/commit/b6ee82b6f5abf129e3e0d84e60e15272479e0db2))
    - only cache connections created by us ([`61d2bc2`](https://github.com/maidsafe/safe_network/commit/61d2bc2d35e3f829d58af736a722d01cd86864b6))
    - sn_interface-0.16.14/sn_client-0.77.8/sn_node-0.72.25/sn_api-0.75.4/sn_cli-0.68.5 ([`783d624`](https://github.com/maidsafe/safe_network/commit/783d62461a65eb7c06b0d4f399b97216b6c75519))
    - Merge #2008 ([`ffac6c6`](https://github.com/maidsafe/safe_network/commit/ffac6c68dc0612a41aa74c533231a63006c22b22))
    - update comm members on AE msg in ([`28cdebb`](https://github.com/maidsafe/safe_network/commit/28cdebb4b05c5d64dcbe8dfb39a72c88fd2c28bd))
    - Merge #1997 #1998 #2002 ([`0c968ad`](https://github.com/maidsafe/safe_network/commit/0c968ad50d9e9dada3f5f5488bd1708fddadef72))
    - add Peer to UnknownPeer conn Err ([`95ae6f9`](https://github.com/maidsafe/safe_network/commit/95ae6f9e8e30184a24465a35626288af64d7995e))
    - Merge #2001 ([`b306800`](https://github.com/maidsafe/safe_network/commit/b30680008443dbb89d68567f54cb789c72423af2))
    - remove Option around send watcher ([`b8dbee2`](https://github.com/maidsafe/safe_network/commit/b8dbee25acfd5b0f348f06419f8058742f575953))
    - prohibit creating PeerSession to unkown node ([`b5e57d5`](https://github.com/maidsafe/safe_network/commit/b5e57d5b3d91849074a90f5ba671d9b19b7e4461))
    - handle sending msgs off the incoming msg loop ([`0cbcc1d`](https://github.com/maidsafe/safe_network/commit/0cbcc1dddf7db229b7fb81328108a076263343d2))
    - Merge #1978 ([`fde6710`](https://github.com/maidsafe/safe_network/commit/fde67106242ad3d47f04ce99261a1e6299e94047))
    - forward client data cmds/queries to holders through Cmd::SendMsgAndAwaitResponse ([`9aaf91b`](https://github.com/maidsafe/safe_network/commit/9aaf91bfedd9fcf040f18e0762ff2cbbc03b4d6f))
    - Merge #1975 ([`635a1b2`](https://github.com/maidsafe/safe_network/commit/635a1b29c9f8be3f708c6670de51ce68c0d34663))
    - introducing Cmd::SendNodeMsgResponse for NodeMsg responses to nodes over streams ([`3d80701`](https://github.com/maidsafe/safe_network/commit/3d8070155bb88b7403ae97730b33510b8c3af685))
    - Merge #1973 ([`f308b44`](https://github.com/maidsafe/safe_network/commit/f308b44fbc8cb0b669ed129727e638285ba65f1d))
    - fix(tests): add feat flag to call test fn from ext - As we test comms in sn_node, now when in another crate, cfg(test) is not detected, and we solve that by adding the dev-dep with a feat flag. ([`76b5e75`](https://github.com/maidsafe/safe_network/commit/76b5e75af26e4a25dcc7f8e0b58e842350339b02))
    - replace node comms with sn_comms ([`dbfa4ac`](https://github.com/maidsafe/safe_network/commit/dbfa4ac0dd23e76060b8df44c4666a30bb9b317f))
    - make a new crate for comm ([`a86d5ad`](https://github.com/maidsafe/safe_network/commit/a86d5ad1f352c9000488197ece8edb716941d601))
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

