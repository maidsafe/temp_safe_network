# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## v0.2.3 (2022-05-12)

### Chore

 - <csr-id-00dc9c0ba9afb6de038dda9e20a10e6727a0b0e6/> reduce handover logging

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 day passed between releases.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - reduce handover logging ([`00dc9c0`](https://github.com/maidsafe/safe_network/commit/00dc9c0ba9afb6de038dda9e20a10e6727a0b0e6))
</details>

## v0.2.2 (2022-05-10)

### Chore

 - <csr-id-61ba367c308a846cb3f1ae065b1fbbdfb85838e4/> sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 3 days passed between releases.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1 ([`61ba367`](https://github.com/maidsafe/safe_network/commit/61ba367c308a846cb3f1ae065b1fbbdfb85838e4))
    - Merge #1171 ([`06b4433`](https://github.com/maidsafe/safe_network/commit/06b4433f199ba7c622ad57e767d80f58f0b50a69))
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

 - 19 commits contributed to the release over the course of 11 calendar days.
 - 13 days passed between releases.
 - 11 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.1/sn_client-0.62.1/sn_node-0.58.15/sn_api-0.60.0/sn_cli-0.53.0 ([`737d906`](https://github.com/maidsafe/safe_network/commit/737d906a61f772593ac7df755d995d66059e8b5e))
    - Merge #1140 ([`459b641`](https://github.com/maidsafe/safe_network/commit/459b641f22b488f33825777b974da80512eabed5))
    - Merge #1165 ([`e08096d`](https://github.com/maidsafe/safe_network/commit/e08096d37dfab490f22ae9786a006aa3f9f630c1))
    - check and log for shrinking SAP on verify_with_chain ([`155d622`](https://github.com/maidsafe/safe_network/commit/155d62257546868513627709742215c0c8f9574f))
    - Merge #1167 ([`5b21c66`](https://github.com/maidsafe/safe_network/commit/5b21c663c7f11124f0ed2f330b2f8687745f7da7))
    - Merge #1169 ([`e5d0c17`](https://github.com/maidsafe/safe_network/commit/e5d0c17c335a3a25ee0bb4c81906fa176abeb7f5))
    - early return when AE required from a vote batch ([`dd353b9`](https://github.com/maidsafe/safe_network/commit/dd353b969ace383c3e89c94f7f242b84b6aee89f))
    - client knowledge could not update ([`9f4c3a5`](https://github.com/maidsafe/safe_network/commit/9f4c3a523212c41079afcde8052a0891f3895f3b))
    - make client targets relative to sap size ([`e8f4fbc`](https://github.com/maidsafe/safe_network/commit/e8f4fbca2acb81b3ddc1b275140e5f7b1b56f9a9))
    - add ProposalAgreed log marker ([`ae9aeeb`](https://github.com/maidsafe/safe_network/commit/ae9aeeb94f55f29849c8c5fe1b05419b96fac6e9))
    - batch MembershipVotes in order to ensure that order is preserved. ([`829eb33`](https://github.com/maidsafe/safe_network/commit/829eb33184c6012faa2020333e72a7c811fdb660))
    - handover integration squashed ([`0d5cdf9`](https://github.com/maidsafe/safe_network/commit/0d5cdf940afc390de22d94e91621e76d45a9eaad))
    - Merge #1141 ([`865f244`](https://github.com/maidsafe/safe_network/commit/865f24477244155528583afa5a3655690e4b7093))
    - add separate feature flags for register/chunk messages ([`696414a`](https://github.com/maidsafe/safe_network/commit/696414ac858795628872a594268517e99a671b00))
    - Merge branch 'main' into Feat-InterfaceAuthKind ([`df40fb9`](https://github.com/maidsafe/safe_network/commit/df40fb94f6847b31aec730eb7cbc6c0b97fe9a0e))
    - add service-msg feature flag to messaging ([`c08f055`](https://github.com/maidsafe/safe_network/commit/c08f05537b70f2d6e0759a39b3f917c0e305e734))
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

