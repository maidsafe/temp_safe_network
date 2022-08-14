# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.8.0 (2022-08-14)

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

### New Features

 - <csr-id-b2c6b2164fbf6679edea0157217dc946d5f9d318/> add AeProbe dysfunction. Refactor score calculation

### Bug Fixes

 - <csr-id-4a17a1dcf858b5daf96e5b9f69ac33c10a988c27/> make the diff proportional to mean to be reported
 - <csr-id-3befae39e3dbc93c4187092e7abe3c6e21893184/> newly inserted operation shall not count towards issue
 - <csr-id-4773e185302ada27cd08c8dfd04582e7fdaf42aa/> removed unused async at dysfunction

### Refactor

 - <csr-id-9fde534277f359dfa0a1d91d917864776edb5138/> reissuing DBCs for all sn_cli tests only once as a setup stage

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 15 commits contributed to the release over the course of 29 calendar days.
 - 36 days passed between releases.
 - 15 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - further tweak dysf, reduce score by std dev for better avg. ([`a4a39b4`](https://github.com/maidsafe/safe_network/commit/a4a39b421103af7c143280ad3860b3cbd3016386))
    - remove unused severity; refactor weighted score ([`3cf9033`](https://github.com/maidsafe/safe_network/commit/3cf903367bfcd805ceff2f2508cd2b12eddc3ca5))
    - add AeProbe dysfunction. Refactor score calculation ([`b2c6b21`](https://github.com/maidsafe/safe_network/commit/b2c6b2164fbf6679edea0157217dc946d5f9d318))
    - serialize NetworkPrefixMap into JSON ([`29de67f`](https://github.com/maidsafe/safe_network/commit/29de67f1e3583eab867d517cb50ed2e404bd63fd))
    - cleanup unnecessary options and results ([`db22c6c`](https://github.com/maidsafe/safe_network/commit/db22c6c8c1aedb347bea52199a5673695eff86f8))
    - make the diff proportional to mean to be reported ([`4a17a1d`](https://github.com/maidsafe/safe_network/commit/4a17a1dcf858b5daf96e5b9f69ac33c10a988c27))
    - rename DysfunctionDetection::adults to nodes ([`7c109a0`](https://github.com/maidsafe/safe_network/commit/7c109a0e22b2032ad5ad3b10f828f855091bec67))
    - newly inserted operation shall not count towards issue ([`3befae3`](https://github.com/maidsafe/safe_network/commit/3befae39e3dbc93c4187092e7abe3c6e21893184))
    - reissuing DBCs for all sn_cli tests only once as a setup stage ([`9fde534`](https://github.com/maidsafe/safe_network/commit/9fde534277f359dfa0a1d91d917864776edb5138))
    - relax knowledge penalty. ([`2f38be7`](https://github.com/maidsafe/safe_network/commit/2f38be726cf493c89d452b6faa50ab8284048798))
    - relax knowledge penalty. ([`bbb77f0`](https://github.com/maidsafe/safe_network/commit/bbb77f0c34e9d4c263be1c5362f1115ecee1da57))
    - removed unused async at dysfunction ([`4773e18`](https://github.com/maidsafe/safe_network/commit/4773e185302ada27cd08c8dfd04582e7fdaf42aa))
    - remove awaits from tests as well ([`31d9f9f`](https://github.com/maidsafe/safe_network/commit/31d9f9f99b4e166986b8e51c3d41e0eac55621a4))
    - remove unused async ([`dedec48`](https://github.com/maidsafe/safe_network/commit/dedec486f85c1cf6cf2d538238f32e826e08da0a))
    - Tweak dysf interval, reducing to report on issues more rapidly ([`e39917d`](https://github.com/maidsafe/safe_network/commit/e39917d0635a071625f7961ce6d40cb44cc65da0))
</details>

## v0.7.1 (2022-07-07)

<csr-id-46262268fc167c05963e5b7bd6261310496e2379/>
<csr-id-6b574bd53f7e51839380b7be914dbab015726d1e/>
<csr-id-2f6fff23a29cc4f04415a9a606fec88167551268/>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.1/sn_dysfunction-0.7.1/sn_client-0.68.1/sn_node-0.64.1/sn_api-0.66.1/sn_cli-0.59.1 ([`2b00cec`](https://github.com/maidsafe/safe_network/commit/2b00cec961561281f6b927e13e501342843f6a0f))
    - Merge #1315 ([`67686f7`](https://github.com/maidsafe/safe_network/commit/67686f73f9e7b18bb6fbf1eadc3fd3a256285396))
    - Merge #1313 ([`7fe7be3`](https://github.com/maidsafe/safe_network/commit/7fe7be336799dec811c5b17e6d753ebe31e625f1))
    - `try!` macro is deprecated ([`4626226`](https://github.com/maidsafe/safe_network/commit/46262268fc167c05963e5b7bd6261310496e2379))
    - Remove registerStorage cache ([`6b574bd`](https://github.com/maidsafe/safe_network/commit/6b574bd53f7e51839380b7be914dbab015726d1e))
    - remove dysfunction arc/rwlock ([`2f6fff2`](https://github.com/maidsafe/safe_network/commit/2f6fff23a29cc4f04415a9a606fec88167551268))
</details>

## v0.7.0 (2022-07-04)

<csr-id-9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7/>
<csr-id-ddb7798a7b0c5e60960e123414277d58f3da27eb/>
<csr-id-e4e2eb56611a328806c59ed8bc80ca2567206bbb/>

### Chore

 - <csr-id-9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7/> Docs - put symbols in backticks
 - <csr-id-ddb7798a7b0c5e60960e123414277d58f3da27eb/> remove let bindings for unit returns

### Chore

 - <csr-id-e4e2eb56611a328806c59ed8bc80ca2567206bbb/> sn_interface-0.8.0/sn_dysfunction-0.7.0/sn_client-0.68.0/sn_node-0.64.0/sn_api-0.66.0/sn_cli-0.59.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 3 calendar days.
 - 6 days passed between releases.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.0/sn_dysfunction-0.7.0/sn_client-0.68.0/sn_node-0.64.0/sn_api-0.66.0/sn_cli-0.59.0 ([`e4e2eb5`](https://github.com/maidsafe/safe_network/commit/e4e2eb56611a328806c59ed8bc80ca2567206bbb))
    - Docs - put symbols in backticks ([`9314a2d`](https://github.com/maidsafe/safe_network/commit/9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7))
    - remove let bindings for unit returns ([`ddb7798`](https://github.com/maidsafe/safe_network/commit/ddb7798a7b0c5e60960e123414277d58f3da27eb))
</details>

## v0.6.1 (2022-06-28)

<csr-id-eebbc30f5dd449b786115c37813a4554309875e0/>
<csr-id-58890e5c919ada30f27d4e80c6b5e7291b99ed5c/>

### Test

 - <csr-id-eebbc30f5dd449b786115c37813a4554309875e0/> adding new dysf test for DKG rounds

### Chore

 - <csr-id-58890e5c919ada30f27d4e80c6b5e7291b99ed5c/> sn_interface-0.7.1/sn_dysfunction-0.6.1/sn_client-0.67.1/sn_node-0.63.1

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.7.1/sn_dysfunction-0.6.1/sn_client-0.67.1/sn_node-0.63.1 ([`58890e5`](https://github.com/maidsafe/safe_network/commit/58890e5c919ada30f27d4e80c6b5e7291b99ed5c))
    - adding new dysf test for DKG rounds ([`eebbc30`](https://github.com/maidsafe/safe_network/commit/eebbc30f5dd449b786115c37813a4554309875e0))
</details>

## v0.6.0 (2022-06-26)

<csr-id-243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e/>

### Chore

 - <csr-id-243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e/> sn_interface-0.7.0/sn_dysfunction-0.6.0/sn_client-0.67.0/sn_node-0.63.0/sn_api-0.65.0/sn_cli-0.58.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.7.0/sn_dysfunction-0.6.0/sn_client-0.67.0/sn_node-0.63.0/sn_api-0.65.0/sn_cli-0.58.0 ([`243cfc4`](https://github.com/maidsafe/safe_network/commit/243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e))
    - Merge #1268 ([`e9adc0d`](https://github.com/maidsafe/safe_network/commit/e9adc0d3ba2f33fe0b4590a5fe11fea56bd4bda9))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 1 calendar day.
 - 3 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.5/sn_dysfunction-0.5.3/sn_client-0.66.5/sn_node-0.62.8/sn_cli-0.57.6 ([`dc69a62`](https://github.com/maidsafe/safe_network/commit/dc69a62eec590b2d621ab2cbc3009cb052955e66))
    - improving dysf test, reproducible issues ([`b433a23`](https://github.com/maidsafe/safe_network/commit/b433a23b2f661ad3ac0ebc290f457f1c64e04471))
</details>

## v0.5.2 (2022-06-21)

<csr-id-c038635cf88d32c52da89d11a8532e6c91c8bf38/>
<csr-id-d526e0a32d3f09a788899d82db4fe6f13258568c/>

### Chore

 - <csr-id-c038635cf88d32c52da89d11a8532e6c91c8bf38/> misc cleanup

### Chore

 - <csr-id-d526e0a32d3f09a788899d82db4fe6f13258568c/> sn_interface-0.6.4/sn_dysfunction-0.5.2/sn_client-0.66.4/sn_node-0.62.7/sn_api-0.64.4

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 5 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.4/sn_dysfunction-0.5.2/sn_client-0.66.4/sn_node-0.62.7/sn_api-0.64.4 ([`d526e0a`](https://github.com/maidsafe/safe_network/commit/d526e0a32d3f09a788899d82db4fe6f13258568c))
    - misc cleanup ([`c038635`](https://github.com/maidsafe/safe_network/commit/c038635cf88d32c52da89d11a8532e6c91c8bf38))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 9 calendar days.
 - 10 days passed between releases.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.3/sn_dysfunction-0.5.1/sn_client-0.66.3/sn_api-0.64.3/sn_cli-0.57.4 ([`f599c59`](https://github.com/maidsafe/safe_network/commit/f599c5973d50324aad1720166156666d5db1ed3d))
    - adjust some dysfunction weighting. decreas dkg ([`26e35cc`](https://github.com/maidsafe/safe_network/commit/26e35cc2d1c5aab81c3479dd7948f7a7e586f817))
    - reduce comm error weighting ([`537b6c0`](https://github.com/maidsafe/safe_network/commit/537b6c08447c15a056d8c79c8592106d9a40b672))
    - enable tracking of Dkg issues ([`7ccb02a`](https://github.com/maidsafe/safe_network/commit/7ccb02a7ded7579bb8645c918b9a6108b1b585af))
    - Merge #1217 ([`2f26043`](https://github.com/maidsafe/safe_network/commit/2f2604325d533357bad7d917315cf4cba0b2d3c0))
</details>

## v0.5.0 (2022-06-05)

<csr-id-1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9/>

### Chore

 - <csr-id-1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9/> sn_interface-0.6.0/sn_dysfunction-0.5.0/sn_client-0.66.0/sn_node-0.62.0/sn_api-0.64.0/sn_cli-0.57.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 4 calendar days.
 - 8 days passed between releases.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.0/sn_dysfunction-0.5.0/sn_client-0.66.0/sn_node-0.62.0/sn_api-0.64.0/sn_cli-0.57.0 ([`1bf7dfb`](https://github.com/maidsafe/safe_network/commit/1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9))
    - Merge #1192 ([`f9fc2a7`](https://github.com/maidsafe/safe_network/commit/f9fc2a76f083ba5161c8c4eef9013c53586b4693))
</details>

## v0.4.0 (2022-05-27)

<csr-id-e5fcd032e1dd904e05bc23e119af1d06e3b85a06/>

### Chore

 - <csr-id-e5fcd032e1dd904e05bc23e119af1d06e3b85a06/> sn_interface-0.5.0/sn_dysfunction-0.4.0/sn_client-0.65.0/sn_node-0.61.0/sn_api-0.63.0/sn_cli-0.56.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 2 days passed between releases.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.5.0/sn_dysfunction-0.4.0/sn_client-0.65.0/sn_node-0.61.0/sn_api-0.63.0/sn_cli-0.56.0 ([`e5fcd03`](https://github.com/maidsafe/safe_network/commit/e5fcd032e1dd904e05bc23e119af1d06e3b85a06))
</details>

## v0.3.0 (2022-05-25)

<csr-id-ef56cf9cf8de45a9f13c2510c63de245b12aeae8/>

### Chore

 - <csr-id-ef56cf9cf8de45a9f13c2510c63de245b12aeae8/> sn_interface-0.4.0/sn_dysfunction-0.3.0/sn_client-0.64.0/sn_node-0.60.0/sn_api-0.62.0/sn_cli-0.55.0

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
    - sn_interface-0.4.0/sn_dysfunction-0.3.0/sn_client-0.64.0/sn_node-0.60.0/sn_api-0.62.0/sn_cli-0.55.0 ([`ef56cf9`](https://github.com/maidsafe/safe_network/commit/ef56cf9cf8de45a9f13c2510c63de245b12aeae8))
    - Merge #1195 ([`c6e6e32`](https://github.com/maidsafe/safe_network/commit/c6e6e324164028c6c15a78643783a9f86679f39e))
</details>

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


### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 3 calendar days.
 - 10 days passed between releases.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_dysfunction-0.2.0/sn_client-0.63.0/sn_node-0.59.0/sn_api-0.61.0/sn_cli-0.54.0 ([`cf21d66`](https://github.com/maidsafe/safe_network/commit/cf21d66b9b726123e0a4320cd68481b67f7af03d))
    - add Display for OperationId ([`ef79815`](https://github.com/maidsafe/safe_network/commit/ef798150deb88efac1dcfe9a3cd0f2cebe1e4682))
    - ensure op writes use mut ([`24125eb`](https://github.com/maidsafe/safe_network/commit/24125eb3603a14c22e208964cbecac16915161ae))
</details>

## v0.1.3 (2022-05-11)

<csr-id-66638f508ad4df12b757672df589ba8ad09fbdfc/>

### Chore

 - <csr-id-66638f508ad4df12b757672df589ba8ad09fbdfc/> sn_dysfunction-0.1.3/sn_node-0.58.17

### Bug Fixes

 - <csr-id-ddb939d5831b2f0d66fa2e0954b62e5e22a3ee69/> relax dysfunction for knowledge and conn issues
   Increases 10x the amount of conn or knowledge issues. We've been voting
   off nodes far too quickly, even on droplet testnets

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 15 calendar days.
 - 18 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_dysfunction-0.1.3/sn_node-0.58.17 ([`66638f5`](https://github.com/maidsafe/safe_network/commit/66638f508ad4df12b757672df589ba8ad09fbdfc))
    - relax dysfunction for knowledge and conn issues ([`ddb939d`](https://github.com/maidsafe/safe_network/commit/ddb939d5831b2f0d66fa2e0954b62e5e22a3ee69))
    - Merge #1128 ([`e49d382`](https://github.com/maidsafe/safe_network/commit/e49d38239b3a8c468616ad3782e1208316e9b5e0))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 2 calendar days.
 - 27 days passed between releases.
 - 7 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0 ([`2f4e7e6`](https://github.com/maidsafe/safe_network/commit/2f4e7e6305ba387f2e28945aee71df650ac1d3eb))
    - tidy references in cargo manifests ([`318ee1d`](https://github.com/maidsafe/safe_network/commit/318ee1d22970b5f06e93a99b6e8fff6da638c589))
    - Merge #1122 ([`f359a45`](https://github.com/maidsafe/safe_network/commit/f359a45971a5b42a6f174536475f47b8ab076901))
    - remove modules that only contained tests ([`6452690`](https://github.com/maidsafe/safe_network/commit/6452690c1b75bb8804c1f9de19c394a83f178acb))
    - move request_operation_fulfilled ([`08385f4`](https://github.com/maidsafe/safe_network/commit/08385f4e03cd43b94f15523597f90f1cc9977a87))
    - remove op_id arg from track_issue ([`1f3af46`](https://github.com/maidsafe/safe_network/commit/1f3af46aea59ebeb1b6a4b736e80e86ce2f724d8))
    - remove unused dep ([`66901bc`](https://github.com/maidsafe/safe_network/commit/66901bcb3b68d3fbe84bfde915bb80ae1b562347))
    - compare against all nodes in section ([`5df610c`](https://github.com/maidsafe/safe_network/commit/5df610c93b76cfc3a6f09734476240313b16bee6))
</details>

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 1 calendar day.
 - 10 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_dysfunction-/safe_network-0.58.9 ([`b471b5c`](https://github.com/maidsafe/safe_network/commit/b471b5c9f539933dd12de7af3473d2b0f61d7f28))
    - sn_dysfunction-0.1.0/safe_network-0.58.8/sn_api-0.58.0/sn_cli-0.51.0 ([`1aa331d`](https://github.com/maidsafe/safe_network/commit/1aa331daa42ef306728fc99e612fbddeed1501d7))
    - sn_dysfunction-0.1.0/safe_network-0.58.8/sn_api-0.58.0/sn_cli-0.51.0 ([`52c2188`](https://github.com/maidsafe/safe_network/commit/52c218861044a46bf4e1666188dc58de232bde60))
    - add sn_dysfunction to release process ([`df66875`](https://github.com/maidsafe/safe_network/commit/df66875627aa41d06b7613085f05a97187c7175d))
    - use time::Instant in place of SystemTime ([`c9f2764`](https://github.com/maidsafe/safe_network/commit/c9f27640d3b1c62bdf88ec954a395e09e799a181))
    - safe_network-0.58.8/sn_api-0.58.0/sn_cli-0.51.0 ([`907c7d3`](https://github.com/maidsafe/safe_network/commit/907c7d3ef4f65df5566627938154dfca1e2fdc05))
    - ensure we have at least 1 when calculating each score ([`52aaf59`](https://github.com/maidsafe/safe_network/commit/52aaf595293f2f0d3dd234907134bc624703a3ca))
    - update readme ([`15a0d35`](https://github.com/maidsafe/safe_network/commit/15a0d354fd804f8f44735b09c22f9e456211c067))
    - add dysfunction tests to ci ([`2e6d78c`](https://github.com/maidsafe/safe_network/commit/2e6d78c13c137e422d3714e8c113aeb4c0b597a3))
    - rename dysfunction -> sn_dysfunction ([`aafb6d2`](https://github.com/maidsafe/safe_network/commit/aafb6d2a458fc4e2dc94ea3a08cb519fe52bc131))
</details>

## v0.1.0 (2022-03-25)

This first version is being edited manually to trigger a release and publish of the first crate.

Inserting another manual change for testing purposes.

### Bug Fixes

 - <csr-id-52aaf595293f2f0d3dd234907134bc624703a3ca/> ensure we have at least 1 when calculating each score

