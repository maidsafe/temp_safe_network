# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.58.14 (2022-04-25)

### Chore

 - <csr-id-2f4e7e6305ba387f2e28945aee71df650ac1d3eb/> sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0
 - <csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/> tidy references in cargo manifests
   All references are organised alphabetically, and random usage of long-form references are removed in
   favour of the short-form version, unless the long-form style is justified, e.g., when lots of
   features are being used.
 - <csr-id-826dfa48cc7c73f19adcd67bb06c7464dba4921d/> fix sn/interface dep version
 - <csr-id-8d041a80b75bc773fcbe0e4c88940ade9bda4b9d/> remove unused deps after node/client split
 - <csr-id-2a731b990dbe67a700468865288585ee8dff0d71/> move examples/bench -> sn_client where appropriate
 - <csr-id-aad69387240b067604a3d54bcf631a726c9d0956/> safe_network->sn_node
 - <csr-id-0fc38442008ff62a6bf5398ff36cd67f99a6e172/> rename sn->sn_node now we have client extracted
 - <csr-id-6383f038449ebba5e7c5dec1d3f8cc1f7deca581/> remove olde node github workflows

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

### Bug Fixes

 - <csr-id-9aa65d92e1d806150401f8bdefa1ead2e3cafd42/> use the config verbosity if no env var present
 - <csr-id-1e7c4ab6d56304f99d11396e0eee5109eb4dda04/> update some instances of safe_network->sn_node
 - <csr-id-ae4ee5c1a37dab7b5ca132d96d026bcdbac50be0/> use supported referencing style
   Currently smart-release doesn't support the `~` style of reference; the `^` style must be used. This
   caused the last nightly run to fail at version bumping.

### Other

 - <csr-id-5580cac3d7aeab7e809729697753a9a38e8f2270/> test valid nonce signature
 - <csr-id-a6cb9e6c5bd63d61c4114afdcc632532f48ba208/> remove test-publish step entirely.
   It doesnt buy us much and may fail if any dep of  has changed.
   Better to work on checking what we want (for git deps eg) rather than breaking CI
 - <csr-id-9945bf8fb5981c1a64b23d6ea1afba5089aa5c3a/> fix benchmark workflow for sn_node dir
 - <csr-id-54000b43cdd3688e6c691bef9dedc299da3c22aa/> test updates for sn_node and sn_client

### Refactor

 - <csr-id-1f3af46aea59ebeb1b6a4b736e80e86ce2f724d8/> remove op_id arg from track_issue
   Based on PR feedback, Yogesh pointed out we could change the `PendingRequestOperation` to use an
   `Option<OperationId>`. This solved the problem when performing a selection, because you can use
   `PendingRequestOperation(None)`. That's a lot better than using some placeholder value for the
   operation ID. This also tidies up `track_issue` to remove the optional `op_id` argument.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 19 commits contributed to the release over the course of 314 calendar days.
 - 17 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - use the config verbosity if no env var present ([`9aa65d9`](https://github.com/maidsafe/safe_network/commit/9aa65d92e1d806150401f8bdefa1ead2e3cafd42))
    - update some instances of safe_network->sn_node ([`1e7c4ab`](https://github.com/maidsafe/safe_network/commit/1e7c4ab6d56304f99d11396e0eee5109eb4dda04))
    - Merge #1128 ([`e49d382`](https://github.com/maidsafe/safe_network/commit/e49d38239b3a8c468616ad3782e1208316e9b5e0))
    - sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0 ([`2f4e7e6`](https://github.com/maidsafe/safe_network/commit/2f4e7e6305ba387f2e28945aee71df650ac1d3eb))
    - test valid nonce signature ([`5580cac`](https://github.com/maidsafe/safe_network/commit/5580cac3d7aeab7e809729697753a9a38e8f2270))
    - tidy references in cargo manifests ([`318ee1d`](https://github.com/maidsafe/safe_network/commit/318ee1d22970b5f06e93a99b6e8fff6da638c589))
    - use supported referencing style ([`ae4ee5c`](https://github.com/maidsafe/safe_network/commit/ae4ee5c1a37dab7b5ca132d96d026bcdbac50be0))
    - Merge #1122 ([`f359a45`](https://github.com/maidsafe/safe_network/commit/f359a45971a5b42a6f174536475f47b8ab076901))
    - remove op_id arg from track_issue ([`1f3af46`](https://github.com/maidsafe/safe_network/commit/1f3af46aea59ebeb1b6a4b736e80e86ce2f724d8))
    - fix sn/interface dep version ([`826dfa4`](https://github.com/maidsafe/safe_network/commit/826dfa48cc7c73f19adcd67bb06c7464dba4921d))
    - remove test-publish step entirely. ([`a6cb9e6`](https://github.com/maidsafe/safe_network/commit/a6cb9e6c5bd63d61c4114afdcc632532f48ba208))
    - compare against all nodes in section ([`5df610c`](https://github.com/maidsafe/safe_network/commit/5df610c93b76cfc3a6f09734476240313b16bee6))
    - remove unused deps after node/client split ([`8d041a8`](https://github.com/maidsafe/safe_network/commit/8d041a80b75bc773fcbe0e4c88940ade9bda4b9d))
    - fix benchmark workflow for sn_node dir ([`9945bf8`](https://github.com/maidsafe/safe_network/commit/9945bf8fb5981c1a64b23d6ea1afba5089aa5c3a))
    - move examples/bench -> sn_client where appropriate ([`2a731b9`](https://github.com/maidsafe/safe_network/commit/2a731b990dbe67a700468865288585ee8dff0d71))
    - test updates for sn_node and sn_client ([`54000b4`](https://github.com/maidsafe/safe_network/commit/54000b43cdd3688e6c691bef9dedc299da3c22aa))
    - safe_network->sn_node ([`aad6938`](https://github.com/maidsafe/safe_network/commit/aad69387240b067604a3d54bcf631a726c9d0956))
    - rename sn->sn_node now we have client extracted ([`0fc3844`](https://github.com/maidsafe/safe_network/commit/0fc38442008ff62a6bf5398ff36cd67f99a6e172))
    - remove olde node github workflows ([`6383f03`](https://github.com/maidsafe/safe_network/commit/6383f038449ebba5e7c5dec1d3f8cc1f7deca581))
</details>

## v0.58.13 (2022-04-23)

<csr-id-7b8ce1c9d980015768a300ac99d07f69cc1f5ae3/>
<csr-id-bc6d861706e57d6cc80bfde2b876ba9ce57efb09/>
<csr-id-8494a01d9be3dddc0d0f4c2c37cdc4d6c3e54521/>
<csr-id-b62ad80298eb4b3e2f9810d20dd553aaf802408b/>
<csr-id-86ce41ca31508dbaf2de56fc81e1ca3146f863dc/>
<csr-id-1f3af46aea59ebeb1b6a4b736e80e86ce2f724d8/>
<csr-id-a6cb9e6c5bd63d61c4114afdcc632532f48ba208/>
<csr-id-9945bf8fb5981c1a64b23d6ea1afba5089aa5c3a/>
<csr-id-54000b43cdd3688e6c691bef9dedc299da3c22aa/>
<csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/>
<csr-id-826dfa48cc7c73f19adcd67bb06c7464dba4921d/>
<csr-id-8d041a80b75bc773fcbe0e4c88940ade9bda4b9d/>
<csr-id-2a731b990dbe67a700468865288585ee8dff0d71/>
<csr-id-aad69387240b067604a3d54bcf631a726c9d0956/>
<csr-id-0fc38442008ff62a6bf5398ff36cd67f99a6e172/>
<csr-id-6383f038449ebba5e7c5dec1d3f8cc1f7deca581/>

### Chore

 - <csr-id-7b8ce1c9d980015768a300ac99d07f69cc1f5ae3/> remove unused sn_interface deps
 - <csr-id-bc6d861706e57d6cc80bfde2b876ba9ce57efb09/> fix bench types dep -> sn_interface
 - <csr-id-8494a01d9be3dddc0d0f4c2c37cdc4d6c3e54521/> split put messaging and types into top level crate

### Refactor

 - <csr-id-1f3af46aea59ebeb1b6a4b736e80e86ce2f724d8/> remove op_id arg from track_issue
   Based on PR feedback, Yogesh pointed out we could change the `PendingRequestOperation` to use an
   `Option<OperationId>`. This solved the problem when performing a selection, because you can use
   `PendingRequestOperation(None)`. That's a lot better than using some placeholder value for the
   operation ID. This also tidies up `track_issue` to remove the optional `op_id` argument.

### Other

 - <csr-id-a6cb9e6c5bd63d61c4114afdcc632532f48ba208/> remove test-publish step entirely.
   It doesnt buy us much and may fail if any dep of  has changed.
   Better to work on checking what we want (for git deps eg) rather than breaking CI
 - <csr-id-9945bf8fb5981c1a64b23d6ea1afba5089aa5c3a/> fix benchmark workflow for sn_node dir
 - <csr-id-54000b43cdd3688e6c691bef9dedc299da3c22aa/> test updates for sn_node and sn_client

### Bug Fixes

 - <csr-id-ae4ee5c1a37dab7b5ca132d96d026bcdbac50be0/> use supported referencing style
   Currently smart-release doesn't support the `~` style of reference; the `^` style must be used. This
   caused the last nightly run to fail at version bumping.

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

### Chore

 - <csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/> tidy references in cargo manifests
   All references are organised alphabetically, and random usage of long-form references are removed in
   favour of the short-form version, unless the long-form style is justified, e.g., when lots of
   features are being used.
 - <csr-id-826dfa48cc7c73f19adcd67bb06c7464dba4921d/> fix sn/interface dep version
 - <csr-id-8d041a80b75bc773fcbe0e4c88940ade9bda4b9d/> remove unused deps after node/client split
 - <csr-id-2a731b990dbe67a700468865288585ee8dff0d71/> move examples/bench -> sn_client where appropriate
 - <csr-id-aad69387240b067604a3d54bcf631a726c9d0956/> safe_network->sn_node
 - <csr-id-0fc38442008ff62a6bf5398ff36cd67f99a6e172/> rename sn->sn_node now we have client extracted
 - <csr-id-6383f038449ebba5e7c5dec1d3f8cc1f7deca581/> remove olde node github workflows

### Other

 - <csr-id-b62ad80298eb4b3e2f9810d20dd553aaf802408b/> add test-utils feat to bench

### Test

 - <csr-id-86ce41ca31508dbaf2de56fc81e1ca3146f863dc/> adding more unit tests to wallet APIs

## v0.58.12 (2022-04-09)

<csr-id-0a719147ae567b41ba2fcbf4c3c0b44e6d1955d1/>
<csr-id-03258b3777644de7799e9563df35afc5e8531be2/>
<csr-id-c8f4eed0406253cc4c253292bc82e7320fdcbf70/>
<csr-id-3dac350598f863fc3d66669c9f8789db51573b96/>
<csr-id-c45f6e362257ba9378547a8f1fd508a5e680cb0a/>
<csr-id-c4e3de1d9715c6e3618a763fa857feca4258248f/>

### Chore

 - <csr-id-0a719147ae567b41ba2fcbf4c3c0b44e6d1955d1/> sn_cli-0.51.1
 - <csr-id-03258b3777644de7799e9563df35afc5e8531be2/> only log SendSus once per batch

### Chore

 - <csr-id-c4e3de1d9715c6e3618a763fa857feca4258248f/> safe_network-0.58.12/sn_api-0.58.1/sn_cli-0.51.2

### New Features

 - <csr-id-951d1bd87490ad8b3c3747cba952424416da013f/> watch status of individual msgs
 - <csr-id-20c33249fceea1c3d085de048f13388187a77ea5/> individual send rates

### Bug Fixes

 - <csr-id-351450ec6794d7933fd2b29359f820e87c0486db/> adult shall update to new SAP
 - <csr-id-14725d0353d797b9437033781e8ff295a7eacc34/> allow update message pass through
 - <csr-id-bd488cbc9d24324bec730f85bdcebccaec2e75c4/> exclude suspects from current
 - <csr-id-bbd6a2370a809a4d23a1df0a813cac2809e06690/> bump node suspicion event prio
 - <csr-id-63c5deae1a4c31c681c019846da95105c0ef7733/> improve success rate

### Other

 - <csr-id-c8f4eed0406253cc4c253292bc82e7320fdcbf70/> debug promotion, rename workflow

### Refactor

 - <csr-id-3dac350598f863fc3d66669c9f8789db51573b96/> use msgs per s

### Test

 - <csr-id-c45f6e362257ba9378547a8f1fd508a5e680cb0a/> run 40mb test as normal

## v0.58.11 (2022-04-01)

### New Features

 - <csr-id-951d1bd87490ad8b3c3747cba952424416da013f/> watch status of individual msgs
 - <csr-id-20c33249fceea1c3d085de048f13388187a77ea5/> individual send rates

### Bug Fixes

 - <csr-id-14725d0353d797b9437033781e8ff295a7eacc34/> allow update message pass through
 - <csr-id-bd488cbc9d24324bec730f85bdcebccaec2e75c4/> exclude suspects from current
 - <csr-id-bbd6a2370a809a4d23a1df0a813cac2809e06690/> bump node suspicion event prio
 - <csr-id-63c5deae1a4c31c681c019846da95105c0ef7733/> improve success rate

## v0.58.10 (2022-03-31)

### New Features

 - <csr-id-951d1bd87490ad8b3c3747cba952424416da013f/> watch status of individual msgs
 - <csr-id-20c33249fceea1c3d085de048f13388187a77ea5/> individual send rates

### Bug Fixes

 - <csr-id-bd488cbc9d24324bec730f85bdcebccaec2e75c4/> exclude suspects from current
 - <csr-id-bbd6a2370a809a4d23a1df0a813cac2809e06690/> bump node suspicion event prio
 - <csr-id-63c5deae1a4c31c681c019846da95105c0ef7733/> improve success rate
 - <csr-id-14725d0353d797b9437033781e8ff295a7eacc34/> allow update message pass through

## v0.58.9 (2022-03-26)

<csr-id-b471b5c9f539933dd12de7af3473d2b0f61d7f28/>
<csr-id-c0806e384d99b94480e8f8e0322a6f5a6bd3636a/>
<csr-id-dabdc555d70be79b910c5fe2b2647ca85f2319f9/>

### Chore

 - <csr-id-b471b5c9f539933dd12de7af3473d2b0f61d7f28/> sn_dysfunction-/safe_network-0.58.9
 - <csr-id-c0806e384d99b94480e8f8e0322a6f5a6bd3636a/> safe_network-0.58.8/sn_api-0.58.0/sn_cli-0.51.0
 - <csr-id-dabdc555d70be79b910c5fe2b2647ca85f2319f9/> check sus nodes periodically and not on query

## v0.58.8 (2022-03-25)

<csr-id-aa82ab7d691cca3990ece4e56d75cd63fda2fe15/>
<csr-id-196b32f348d3f5ce2a63c24877acb1a0a7f449e3/>
<csr-id-907c7d3ef4f65df5566627938154dfca1e2fdc05/>
<csr-id-90712c91368b4d88537acc65a3ccc5478fe38d2c/>
<csr-id-51d534d39caf3a4c029f7d1a9c87a3edf3192b2f/>
<csr-id-b239e6b38a99afda7a945a51d3f6e00841730a8f/>
<csr-id-aafb6d2a458fc4e2dc94ea3a08cb519fe52bc131/>
<csr-id-6e897d0bc93256f5ab72350c9774f9a33937da1b/>
<csr-id-224079f66832c0a914fd20af4fc2f9e90dc9c9c9/>
<csr-id-453b246c002f9e964896876c254e6c31f1f6045d/>
<csr-id-0cad4c981dbdf9eee58fc28b4637f136b817c384/>
<csr-id-43489f6170ce13ea05148a52422fbff6bdb91f19/>
<csr-id-1ea4b4413fee11fde2b69086caeb69cc191fe277/>
<csr-id-2ac9edab88602fa2aeb148baf6566cab876dc2af/>
<csr-id-e4a7c564180cb3b81601950c4f7623dfcdcd7650/>
<csr-id-80c2ea0c2863ba8cc3500a1880337285e40fdf4c/>
<csr-id-a49ea81658b15280c23e93b1945f67aeb43a5962/>
<csr-id-6b83f38f17c241c00b70480a18a47b04d9a51ee1/>

### Other

 - <csr-id-aa82ab7d691cca3990ece4e56d75cd63fda2fe15/> remove unneeded retry_backoff in reg tests
 - <csr-id-196b32f348d3f5ce2a63c24877acb1a0a7f449e3/> re-ignore 40/100mb tests for basic ci

### Chore

 - <csr-id-907c7d3ef4f65df5566627938154dfca1e2fdc05/> safe_network-0.58.8/sn_api-0.58.0/sn_cli-0.51.0
 - <csr-id-90712c91368b4d88537acc65a3ccc5478fe38d2c/> update deps
 - <csr-id-51d534d39caf3a4c029f7d1a9c87a3edf3192b2f/> add LogMarker for dysfunction agreement
 - <csr-id-b239e6b38a99afda7a945a51d3f6e00841730a8f/> remove unnecessary map_err call
   Removes a map_err which isn't needed for
 - <csr-id-aafb6d2a458fc4e2dc94ea3a08cb519fe52bc131/> rename dysfunction -> sn_dysfunction
 - <csr-id-6e897d0bc93256f5ab72350c9774f9a33937da1b/> remove retry_loop! from nrs tests
 - <csr-id-224079f66832c0a914fd20af4fc2f9e90dc9c9c9/> tweaking values and thresholds
   Client tests should not trigger suspect or dysfunctional nodes.
   Prior to this, we can see that heppening regularly in testnets
 - <csr-id-453b246c002f9e964896876c254e6c31f1f6045d/> refactor NodeQueryResponse handling at elder
 - <csr-id-0cad4c981dbdf9eee58fc28b4637f136b817c384/> use one func to detect nodes beyond a severity
 - <csr-id-43489f6170ce13ea05148a52422fbff6bdb91f19/> remove _ from dys_function crate name
 - <csr-id-1ea4b4413fee11fde2b69086caeb69cc191fe277/> enable 100mb test
 - <csr-id-2ac9edab88602fa2aeb148baf6566cab876dc2af/> reduce node cleanup frequency
 - <csr-id-e4a7c564180cb3b81601950c4f7623dfcdcd7650/> different logs for different data failures
 - <csr-id-80c2ea0c2863ba8cc3500a1880337285e40fdf4c/> log ProposeOffline
 - <csr-id-a49ea81658b15280c23e93b1945f67aeb43a5962/> split out deviant/unresponsive calculations
   This should just keep things a bit tidier
 - <csr-id-6b83f38f17c241c00b70480a18a47b04d9a51ee1/> deps, remove ~ restriction on major versioned deps
   tilde w/ a major version restricts us to path udpats only.
   we want caret, which is implicit frm v 1

### New Features

 - <csr-id-5d80122d51dcbe8241e85f27d23e34b053b77651/> optimise DataQuery cache
   Move the pending_requests to storing an Arc<DashSet<Peers>>
   
   This is due to cache providing a clone when we , which means
   we may not always be setting pending peers correctly.
   
   Now we also use the DATA_QUERY_TIMEOUT, set to 15s to decide if we should
   re-send a query. (Previously it was only sent if no peer was waiting; ).
   
   This means we have a hard limit on how often we may ask for a chunk if peers are waiting for
   that chunk.
 - <csr-id-8168be4ef68f467f1c4b35943442fd30102a34f2/> record time of conn/knowledge issues
   Remove older issues (currently set to 15 mins) from the tracker to keep
   node dysfunction detection relevant to current circumstance
 - <csr-id-73fcad5160127c9e4d3a808df229947fa6163ae0/> add knowledge dysfunction tracking and weighting
 - <csr-id-88e1800103ac8465bbfc377719ed5804853d833a/> use weighted averages for dysfunctional detection
 - <csr-id-1d390ee298293ffe7533b4f1a3b47cc1d11a2198/> initial move of liveness->dysfunctional detection crate in root

### Bug Fixes

 - <csr-id-a45a3bda7044f07b6ecd99569ec4c043330d7160/> Improve query handling, keep peers on DataNotFound
   Previously if we had DataNotFound, any waiting peers are not requeued.
   So this means that one bogus adult can effectively fail a query which
   might otherwise succeed.
   
   In doing so, we need to always resend queries to adults if clients retry
   as otherwise they may remain queued but wont actually requery adults
   ever again.
   
   This appears to improve test time, even after reducing the RETRY_COUNT
   on messages.
 - <csr-id-2619ca4b399cf8156b9fdf2e33abe2708e6673b9/> re-queue waiting peers on dud responses
 - <csr-id-6915006f070811eccc7aea075a707d15a20abb93/> add new querying peer to wait list if that already exists
 - <csr-id-dbcffd94114e36ac763e5718937d1228ba882baa/> throttle suspect node's preemptive replication.
   Previous every query response could trigger this flow.
   Now we only do it once every X time. (25 mins atm)

## v0.58.7 (2022-03-22)

<csr-id-a6e2e0c5eec5c2e88842d18167128991b76ecbe8/>
<csr-id-d3989bdd95129999996e58736ec2553242697f2c/>

### Chore

 - <csr-id-a6e2e0c5eec5c2e88842d18167128991b76ecbe8/> safe_network-0.58.7/sn_api-0.57.3/sn_cli-0.50.5
 - <csr-id-d3989bdd95129999996e58736ec2553242697f2c/> bump bls_dkg, self_encryption, xor_name
   This is a step towards integrating sn_dbc into safe_network.

## v0.58.6 (2022-03-17)

<csr-id-a741d930b906054d09f1311ddcf35479aa1aa3ee/>
<csr-id-20c057c72fdcacc0bcda6da1f97eb6fab1a0cf4c/>

### Chore

 - <csr-id-a741d930b906054d09f1311ddcf35479aa1aa3ee/> safe_network-0.58.6/sn_api-0.57.2
 - <csr-id-20c057c72fdcacc0bcda6da1f97eb6fab1a0cf4c/> make resource logging consistent


### New Features

 - <csr-id-d6e601a3c18dc2b7f60c297f5c794883952e1d14/> prune parent whenever a child inserted

## v0.58.5 (2022-03-09)

<csr-id-df330fa0de1e334e55863828fb743131ab629a18/>
<csr-id-784870abfdd5620da6839e7fd7df80702e0f3afa/>
<csr-id-df398e1017221c0027542b597c8f7c38c1828723/>
<csr-id-8c3bcc2bb64063e646d368d90fff98420ab22dce/>

### Other

 - <csr-id-df330fa0de1e334e55863828fb743131ab629a18/> discard DataReplicator module
   - DataReplicator was used to track replication flows in order to delete data that was not
   in a node's range after replication completes. This can now be removed since we hold on to data
   that is not in our range, even after replication ends.

### Chore

 - <csr-id-784870abfdd5620da6839e7fd7df80702e0f3afa/> safe_network-0.58.5
 - <csr-id-df398e1017221c0027542b597c8f7c38c1828723/> clarify comment regarding bootstrap
 - <csr-id-8c3bcc2bb64063e646d368d90fff98420ab22dce/> remove unneeded HandleSystemMsg command

### New Features

 - <csr-id-7a1065c46f5d72f6997a504c984a70493e197a5b/> impl throttled message sending
   - adds a new command that given sends messages in a throttled fashion

## v0.58.4 (2022-03-04)

<csr-id-96bf83f8635004fa23e6b5b870cac27bd5b42332/>
<csr-id-98a0eb988ab819ffce4727a9b823709efdc18dab/>
<csr-id-bf7de428cd1e091ef2a6e616c07fd6596d711992/>
<csr-id-ba0a26db12823bd2de51a4c75c5e7c4875c8f3f5/>
<csr-id-94d58d565d9a6870e334bb2ca249f26ac3f8a327/>
<csr-id-7cf7fd675beec5e7aa122f0f127402b636e659b7/>
<csr-id-43a42657ff18409dd83bba03135cd013c1298dc3/>
<csr-id-7cb5ffd03f9bfce6ebe74f66dcabddef661cf94d/>
<csr-id-000fc62a87dac3cd41bb6ced59596635f056ff29/>
<csr-id-0ff76c44aeddedd765fa4933c0841539feabaae5/>
<csr-id-6970533f288ff867a702d3dbc5424314b3639674/>

### Test

 - <csr-id-96bf83f8635004fa23e6b5b870cac27bd5b42332/> add unit tests to data storage and replicator modules
   - renames some of the internal APIs
   - fixed a bug in replicator module

### Refactor

 - <csr-id-98a0eb988ab819ffce4727a9b823709efdc18dab/> limit batch size to 50 chunks and remove ack message
 - <csr-id-bf7de428cd1e091ef2a6e616c07fd6596d711992/> do not delete data after finishing replication
 - <csr-id-ba0a26db12823bd2de51a4c75c5e7c4875c8f3f5/> remove entries from replicator if target already has data
 - <csr-id-94d58d565d9a6870e334bb2ca249f26ac3f8a327/> do not cache to-be-replicated data; fetch from storage

### Chore

 - <csr-id-7cf7fd675beec5e7aa122f0f127402b636e659b7/> safe_network-0.58.4/sn_cli-0.50.4
 - <csr-id-43a42657ff18409dd83bba03135cd013c1298dc3/> remove unneeded logs
 - <csr-id-7cb5ffd03f9bfce6ebe74f66dcabddef661cf94d/> dont delete chunks on republish
 - <csr-id-000fc62a87dac3cd41bb6ced59596635f056ff29/> only add liveness check when we're sending out reqs.
   Previously the liveness add was before we checked if this client was already awaiting this same data req.
   So one slow req could compound liveness checks
 - <csr-id-0ff76c44aeddedd765fa4933c0841539feabaae5/> rename we_are_not_a_holder for clarity
 - <csr-id-6970533f288ff867a702d3dbc5424314b3639674/> tweak logging msg prio and removed unneeded msgs

### New Features

 - <csr-id-076d5e3cfd64d7bd677c9e6d34baf93f2eb49b4a/> clients should retry message sending when Conn errors encountered
 - <csr-id-5b530fec3cd6d182f4dba89e2144826977404aa9/> add basic PeerLink cleanup
   prior to this client Peer Links were held forever. Now we trigger
   a Cmd to clean them up if they're not waiting on any other response from
   the network.
   
   We do this every X time
 - <csr-id-070c146ea287db9bf708b11fbaaa177422cea73d/> nodes retry link.send_with once blindly
 - <csr-id-65c3adc8dd45368260bf60ef83d44a99eb6ee7ca/> batch-up data for replication
 - <csr-id-d3fd698b756d100daa12230fb944129529b773fb/> impl pull model data replication
 - <csr-id-a3a28bd7c816582218bea5dbf0f7e141b3ae2c76/> init data replicator module

## v0.58.3 (2022-03-01)

<csr-id-a3a2bbfbaf3be846a1c1f22a74aee2f961341685/>
<csr-id-51b3d75fc7389de647f6df230bff32e8c7d7267c/>

### Other

 - <csr-id-a3a2bbfbaf3be846a1c1f22a74aee2f961341685/> use split example to start network for e2e-split run
   We use the split test to check data integrity, before continuing
   with standard client checks

### Chore

 - <csr-id-51b3d75fc7389de647f6df230bff32e8c7d7267c/> safe_network-0.58.3/sn_cli-0.50.3

## v0.58.2 (2022-02-27)

<csr-id-d8c57e082b52196cc538271bc25a88e3efd2a97c/>
<csr-id-634010fd79ce1487abbff5adf3d15da59709dd95/>
<csr-id-85e513670aa61f8acb3e3302ee4b39763ade036e/>
<csr-id-705995ef67b3d4c45c95689c4a675e1063467ec9/>
<csr-id-f8bfe9efe68593ceb4f968a6d2a396c431ad6429/>
<csr-id-61068aaf3e9cd1c7513b58c073c55004697fdf6f/>
<csr-id-ea2ba0afa036b6abab35db9a76488d052e7682d6/>
<csr-id-912a2f8d2da4159fcf40567666b3b14024e8c0da/>
<csr-id-d5e6f462615de830cd9c27dba49a34ba2da13b81/>
<csr-id-35a46f06e6233aff25d03350abaefacbe57ad25c/>
<csr-id-fc074ab28d3c8c011016e6598cf840fc38026418/>
<csr-id-b44ac353e254d8d67996c3185dc40e5e99c0e4c7/>
<csr-id-222742f7c57a4b451af354d33015974d0d7a3561/>
<csr-id-c086db96c09a77c43777783f614ca6a43eff7cdd/>

### Other

 - <csr-id-d8c57e082b52196cc538271bc25a88e3efd2a97c/> ignore cargo husky in udeps checks

### Chore

 - <csr-id-634010fd79ce1487abbff5adf3d15da59709dd95/> safe_network-0.58.2/sn_api-0.57.1/sn_cli-0.50.2
 - <csr-id-85e513670aa61f8acb3e3302ee4b39763ade036e/> remove pointless and long checks in reg batch
   Previously we needlessly attempt to check for a reg which
   didnt exist, to confirm that it didnt.
   
   This took a QUERY_TIMEOUT length of time, and was obviously slow.
   
   This speeds up the test from ~4 mins to 7secs
 - <csr-id-705995ef67b3d4c45c95689c4a675e1063467ec9/> changes to appease clippy 1.59
 - <csr-id-f8bfe9efe68593ceb4f968a6d2a396c431ad6429/> update deps.
 - <csr-id-61068aaf3e9cd1c7513b58c073c55004697fdf6f/> Add testing documentation
 - <csr-id-ea2ba0afa036b6abab35db9a76488d052e7682d6/> add read timing logs into put_get example
 - <csr-id-912a2f8d2da4159fcf40567666b3b14024e8c0da/> add put timing logs into put_get example
 - <csr-id-d5e6f462615de830cd9c27dba49a34ba2da13b81/> more general dep updates
 - <csr-id-35a46f06e6233aff25d03350abaefacbe57ad25c/> add put_get chunk soak example
   Adds an example to launch many clients uploading many files, and then
   verify those uploads.
   
   This is handy to test node load and client throughput.
   
   Currently it's not used in CI anywhere.
 - <csr-id-fc074ab28d3c8c011016e6598cf840fc38026418/> sn_cli-0.50.1
 - <csr-id-b44ac353e254d8d67996c3185dc40e5e99c0e4c7/> move testnet bin into its own crate
 - <csr-id-222742f7c57a4b451af354d33015974d0d7a3561/> update qp2p
 - <csr-id-c086db96c09a77c43777783f614ca6a43eff7cdd/> more log cmd inspector into its own crate.
   This removes some deps from sn, and log cmd inspector should work just
   fine there

### Bug Fixes

 - <csr-id-38fb057da44a0e243186410df0c39361a21ec46e/> introduce cohesive conn handling
 - <csr-id-ddd45b7cc73bbacea19f5c93519ae16a74cc01cc/> add MIN_PENDING_OPS threshold for liveness checks

## v0.58.1 (2022-02-20)

<csr-id-ea0387b43233f95d10f19d403d289f272f42336f/>

### Chore

 - <csr-id-ea0387b43233f95d10f19d403d289f272f42336f/> safe_network-0.58.1

### Bug Fixes

 - <csr-id-ddd45b7cc73bbacea19f5c93519ae16a74cc01cc/> add MIN_PENDING_OPS threshold for liveness checks

## v0.58.0 (2022-02-17)

<csr-id-7b213d7bb1eaf208f35fc4c0c3f4a71a1da5aca3/>
<csr-id-149665a53c00f62be0e8c8ec340b951a06346848/>
<csr-id-9fb73957fbba99929911296305bfd66dbaa4d15a/>
<csr-id-3a427eb33dd0a81f8dc77521a88eba2112bec778/>
<csr-id-c184c49cd9d71e657a4b5349940eb23810749132/>
<csr-id-ceaf43916dfac62213ea23bde35c3f931cdd8c37/>
<csr-id-b161e038bf30e0a50a7103eaad80ef2368b6689d/>
<csr-id-7e15db2de4d99d84126e7087d838047bae7a009b/>
<csr-id-bd19e9e6eae4ce6068e1bee2d89528d36fce5329/>
<csr-id-a56399864e18f2b2de7ba033497c5bbbe3e5394e/>

### Refactor

 - <csr-id-7b213d7bb1eaf208f35fc4c0c3f4a71a1da5aca3/> remove MIN_PENDING_OPS and always check for unresponsive nodes
   - also refactors the previous inversely proportionate ratio check to directly proportionate

### Chore

 - <csr-id-149665a53c00f62be0e8c8ec340b951a06346848/> safe_network-0.58.0/sn_api-0.57.0/sn_cli-0.50.0
 - <csr-id-9fb73957fbba99929911296305bfd66dbaa4d15a/> update NEIGHBOUR_COUNT to DEFAULT_ELDER_SIZE
   - NEIGHBOUR_COUNT must always be set relevant to section size
 - <csr-id-3a427eb33dd0a81f8dc77521a88eba2112bec778/> document Liveness consts
 - <csr-id-c184c49cd9d71e657a4b5349940eb23810749132/> minor fixes and refactor to active data republishing
 - <csr-id-ceaf43916dfac62213ea23bde35c3f931cdd8c37/> update dashmap for security adv -> 5.1.0
 - <csr-id-b161e038bf30e0a50a7103eaad80ef2368b6689d/> log insufficent adult err
 - <csr-id-7e15db2de4d99d84126e7087d838047bae7a009b/> log adults known about during storage calcs
 - <csr-id-bd19e9e6eae4ce6068e1bee2d89528d36fce5329/> easier to read logging for saps
 - <csr-id-a56399864e18f2b2de7ba033497c5bbbe3e5394e/> tweak command ack wait and interval looping.
   Increases wait for acks slightly.

### New Features

 - <csr-id-928600b13fdb1a31498cc27cba968a8e3eba598c/> republish data actively when deviant nodes are detected

### Bug Fixes

 - <csr-id-a787b7c003f0604383a3c47d731e6c2fc84f7583/> update MIN_PENDING_OPS value and liveness tests
 - <csr-id-5d1bae8cf0fa1aeb8d48c24bb4fc82bd705bc32e/> fixes data republish flow by routing messages via elders

### New Features (BREAKING)

 - <csr-id-4e7bc8bbb3f521324edf0e4a6e329271b7f854f6/> remove always-joinable feature flag

## v0.57.1 (2022-02-15)

<csr-id-07dc30b281f3c67cb5598aaaf72ba5c668353bf7/>
<csr-id-03cf556d1b73565d520e2c5b82ab1482b076e639/>
<csr-id-7068445cad25b0273c841c5072055833df9b8229/>
<csr-id-d3dd663cfd6f3d0b0943ff49b1eed8c6b37d6263/>
<csr-id-5fe289271fb0da28935b0b578928687be0dc4665/>
<csr-id-d6e788221f332c3ff3a22eec39af428eebf5e75f/>
<csr-id-283ff34f03dbf783b2299a3c7eb7a183c1e3c0a6/>
<csr-id-e9e249a674d4a64078a519b7e20baf6f0759c1c9/>

### Chore

 - <csr-id-07dc30b281f3c67cb5598aaaf72ba5c668353bf7/> safe_network-0.57.1/sn_cli-0.49.1
 - <csr-id-03cf556d1b73565d520e2c5b82ab1482b076e639/> begin sorting out messaging
 - <csr-id-7068445cad25b0273c841c5072055833df9b8229/> move capacity and liveness into data mod
 - <csr-id-d3dd663cfd6f3d0b0943ff49b1eed8c6b37d6263/> move chunk store to dbs
 - <csr-id-5fe289271fb0da28935b0b578928687be0dc4665/> co-locate related modules
 - <csr-id-d6e788221f332c3ff3a22eec39af428eebf5e75f/> rename core to node
 - <csr-id-283ff34f03dbf783b2299a3c7eb7a183c1e3c0a6/> rename node to nodeinfo
 - <csr-id-e9e249a674d4a64078a519b7e20baf6f0759c1c9/> move peer and prefix map into types

## v0.57.0 (2022-02-12)

<csr-id-a398c4f8d72828db0fc8c6d5825ead62ba85db64/>
<csr-id-8f580200ba5b8b67f36977fb59d6d48e7613e176/>
<csr-id-1f57e30a206998a03b2201f4b57b372ebe9ae828/>
<csr-id-1dfb651e72f38743645afbbee62c7e6e7fbb0fb2/>
<csr-id-5869662a29c4bd8dfe0d7bf07a30f10d89b450ad/>
<csr-id-74fc4ddf31af51e0036eb164e6b5c4f0864bd08c/>

### Chore

 - <csr-id-a398c4f8d72828db0fc8c6d5825ead62ba85db64/> safe_network-0.57.0/sn_api-0.56.0/sn_cli-0.49.0
 - <csr-id-8f580200ba5b8b67f36977fb59d6d48e7613e176/> Batch pending requests together per name
   Previously we were only allowing one operation id, which could be overwritten by new clients.
   This allows client requests to await one response together
 - <csr-id-1f57e30a206998a03b2201f4b57b372ebe9ae828/> only prioritise service cmd/query with permits.
   Previously we were holding permits in a RwLock struct for _any_ command.
   
   Now we remove both the conecpt of permits here, removing one potential
   locking loction, and the unused message prioritisation is removed to
   clean that up.
   
   Now we simply check if we have max allowed pending queries at any one
   time, and if so, we drop inbound queries.
 - <csr-id-1dfb651e72f38743645afbbee62c7e6e7fbb0fb2/> rename standard_wait to cmd_ack_wait, set default to 5s
   Previously we've had standard wait be more AE centric. But it was actually
   only being used to determine how long to wait for Cmd Acks.
   
   Here that is clarified, and a none-zero time set to be more useful
 - <csr-id-5869662a29c4bd8dfe0d7bf07a30f10d89b450ad/> simplify ae msg handling at clients
   Previously AE-retry/redirect did not both have a SAP. Now
   they do we can simplify that and directly use it in either case.
   
   We also can remove the AE caches, and just send on each incoming AE-msg
   to one elder (as opposed to `count` elders). This is deterministic based
   on sender name. Which means we can simply do it as each one comes in. No
   tracking of prior messages needed.
   
   We also remove the `make_contact` exemptions in case this interferes
   with initial network contact.
   
   Overall simpler and cleaner.

### New Features

 - <csr-id-0bc50ae33ccb934016ac425e7bb2eca90a4b06e3/> resolve nrs map container content
   The resolver can now return `NrsMapContainer` content, which can then be displayed by the CLI with
   the `cat` and `dog` commands. This functionality was unintentionally broken at some point.
   
   The first change introduced an `NrsEntry` field in `SafeData`, and modified the `NrsMapContainer` to
   remove its `resolve_into` and `public_name` fields. The intention is for the resolver to return
   `NrsMapContainer` data when a container XOR-URL is used, but when using an NRS URL, an `NrsEntry`
   will be returned. The `NrsMapContainer` data will have the NRS map, whereas the `NrsEntry` will only
   contain the target link and subname version. It's worth noting, the `NrsEntry` doesn't have an
   XOR-URL because the entries are still stored in the map. An NRS URL still has an `NrsMapContainer`
   content type and that content is retrieved during the resolution process.
   
   This brings us to the next change. The `nrs_get` API was modified to return an `Option<SafeUrl>`,
   where `None` will now be returned if the container XOR-URL is used. In this case, the resolver will
   know to return `NrsMapContainer` data, otherwise, it will return the `NrsEntry` with the target URL.
   One exception is worth mentioning: if the NRS URL uses the registered topname and that topname
   *doesn't* link to anything, `NrsMapContainer` data will also be returned. To make these extensions,
   small unit tests were added to the `NrsMap` and several tests were added to the resolver to cover
   these scenarios.
   
   With these changes in place, the CLI could then be updated. The `cat` and `dog` commands were
   modified to print the NRS map when `NrsMapContainer` data was returned. Previously, the map was
   printed as a table, but this isn't really suitable for presentation because the table crate doesn't
   have the ability to use multi-line cells and the target links are too large, so I changed it to
   print a list. Test cases were added for both commands, which should hopefully prevent us breaking
   the feature again.
   
   Finally, some usability changes were also made to `nrs` commands to give the user the XOR-URL of the
   container. This can be useful to them if they want to list all the entries in a map.

### Refactor (BREAKING)

 - <csr-id-74fc4ddf31af51e0036eb164e6b5c4f0864bd08c/> reorganise internal node messaging mod and remove redundant private APIs
   - Fixing missing AE update to be sent to sibling new Elders upon split.

## v0.56.0 (2022-02-08)

<csr-id-3f75bf8da770a6167c396080b3ad8b54cfeb27e2/>
<csr-id-471d910f2b6d8952569c3dc4b2dd31fe7aa30dfa/>
<csr-id-f1e0e4564c2b581352f6dc6a0ba32259452494c5/>

### Chore

 - <csr-id-3f75bf8da770a6167c396080b3ad8b54cfeb27e2/> safe_network-0.56.0/sn_api-0.55.0/sn_cli-0.48.0
 - <csr-id-471d910f2b6d8952569c3dc4b2dd31fe7aa30dfa/> improve acronym consistency
 - <csr-id-f1e0e4564c2b581352f6dc6a0ba32259452494c5/> ensure to use backoff.reset() before use
   otherwise the default settings appear to be used

### New Features (BREAKING)

 - <csr-id-0b5ebece1240deb56360b238b96e2aece4a6d314/> fix typo

## v0.55.5 (2022-02-07)

<csr-id-d56f3c7c0bfc7bd6d045eb80a68a885615e73115/>
<csr-id-473e4b0ca27767f4e1326629670f43ff5de5bc86/>
<csr-id-dab972dccdf968e706f0c7599154e188dc74bf48/>

### Chore

 - <csr-id-d56f3c7c0bfc7bd6d045eb80a68a885615e73115/> safe_network-0.55.5
 - <csr-id-473e4b0ca27767f4e1326629670f43ff5de5bc86/> tweak client commend retries and backoff
 - <csr-id-dab972dccdf968e706f0c7599154e188dc74bf48/> adding a trace log when updating a section member state

### New Features

 - <csr-id-0c04071e2184c4e74fa8c9f264a380585e84369e/> add `flame` arg to use cargo flamegraph
   uses the launcher option to enable flamegraph generation per node

## v0.55.4 (2022-02-05)

<csr-id-a0e9c52951e81018249a3bcf3b6300b3ad592136/>
<csr-id-1c78a27d31bb6530274d4ac5cecbce817fad5313/>

### Chore

 - <csr-id-a0e9c52951e81018249a3bcf3b6300b3ad592136/> safe_network-0.55.4
 - <csr-id-1c78a27d31bb6530274d4ac5cecbce817fad5313/> client errors when a SAP has less than expected elders

### New Features (BREAKING)

 - <csr-id-0b5ebece1240deb56360b238b96e2aece4a6d314/> fix typo

### Bug Fixes

 - <csr-id-e0e085273524f8e457887548cd68238060590a7b/> only send reg commands to a subset of elders

## v0.55.3 (2022-02-04)

<csr-id-86975f228f31303597a707e158005e44c86de1cc/>
<csr-id-064a7ed00ca84edd5b5f86640ae868c1cb202590/>

### Chore

 - <csr-id-86975f228f31303597a707e158005e44c86de1cc/> safe_network-0.55.3/sn_api-0.54.1/sn_cli-0.47.1
 - <csr-id-064a7ed00ca84edd5b5f86640ae868c1cb202590/> one more error msg on client send

### New Features

 - <csr-id-0c04071e2184c4e74fa8c9f264a380585e84369e/> add `flame` arg to use cargo flamegraph
   uses the launcher option to enable flamegraph generation per node

### New Features (BREAKING)

 - <csr-id-208e73c732ae5bac184bf5848c0490b98c9a0364/> move to q2p 0.28 / quinn 0.8

### Bug Fixes

 - <csr-id-df808e1d3c559408f5704590493d0aa97d9c2a19/> section_peers refactored to avoid dashmap deadlock.
   Also updates dashmap to latest version, which uses parking lot for locks,
   which is theoretically faster too

## v0.55.2 (2022-02-02)

<csr-id-b1d35af9c2ae6cb386c6726a432c02b9d44973c2/>
<csr-id-637ec03f9922e5d3dd0c8703eba5019256f4ec06/>
<csr-id-d42b34d54adfc16b7947fa5728fcde80ad4f49c7/>
<csr-id-7cd63377de16dd2962961f2dd936df3276fe8d6d/>
<csr-id-f73e00cc67fae6090b9c991ac4e06999ea28f22e/>
<csr-id-0c4c7d8c973bdb1e1d055798c17be2065f0334e2/>
<csr-id-2479daea6b05c7b680bf6062407f507ad8692d57/>
<csr-id-7f01106c8266abec07c9522cb69e4fbbb493f386/>

### Refactor

 - <csr-id-b1d35af9c2ae6cb386c6726a432c02b9d44973c2/> use the churn signature for deciding both the destination and members to relocate

### Chore

 - <csr-id-637ec03f9922e5d3dd0c8703eba5019256f4ec06/> safe_network-0.55.2
 - <csr-id-d42b34d54adfc16b7947fa5728fcde80ad4f49c7/> log chain update logic
 - <csr-id-7cd63377de16dd2962961f2dd936df3276fe8d6d/> all env var to override cmd_timeout
 - <csr-id-f73e00cc67fae6090b9c991ac4e06999ea28f22e/> session records all failed connection ids
 - <csr-id-0c4c7d8c973bdb1e1d055798c17be2065f0334e2/> minor reorganisation of code
 - <csr-id-2479daea6b05c7b680bf6062407f507ad8692d57/> failcmd with any error after retries
 - <csr-id-7f01106c8266abec07c9522cb69e4fbbb493f386/> simplify data replication cmds and flow

## v0.55.1 (2022-02-01)

<csr-id-25259b221120c8c9258ffdfae8883c65ead38677/>
<csr-id-2ec86e28246031084d603768ffa1fddf320a10a2/>
<csr-id-81298bb8ce1d93d7a418ff340d866fc00d9f60a4/>
<csr-id-fa6014e581f52615971701572a1635dfde922fb6/>

### Test

 - <csr-id-25259b221120c8c9258ffdfae8883c65ead38677/> unit test liveness tracker basics

### Chore

 - <csr-id-2ec86e28246031084d603768ffa1fddf320a10a2/> safe_network-0.55.1/sn_api-0.54.0/sn_cli-0.47.0
 - <csr-id-81298bb8ce1d93d7a418ff340d866fc00d9f60a4/> log data reorganisation

### Refactor (BREAKING)

 - <csr-id-fa6014e581f52615971701572a1635dfde922fb6/> remove the relocation promise step for Elders
   - Both Adults and Elders now receive the `Relocate` message with all the relocation details
   without any previous or additional steps required to proceed with their relocation.

### New Features

 - <csr-id-b2b0520630774d935aca1f2b602a1de9479ba6f9/> enable cmd retries
   Previously a command error would simply error out and fail.
   Now we use an exponential backoff to retry incase errors
   can be overcome

### Bug Fixes

 - <csr-id-16bd75af79708f88dcb7086d04fac8475f3b3190/> fix liveness tracking logic and keep track of newly joining adults
 - <csr-id-6aae745a4ffd7302fc74d5d548ca464066d5203f/> make log files rotate properly

## v0.55.0 (2022-01-28)

<csr-id-c52de7a75ce4bbf872b215a14258b5e7778bfb34/>
<csr-id-366eee25f4b982d5a20d90168368a1aa14aa3181/>
<csr-id-2cc7323ec8b62f89f2e4247c5d6ab56f78eda2ce/>
<csr-id-2746cff087fd33aff523bc2df07a3462d05c6de1/>
<csr-id-fa6014e581f52615971701572a1635dfde922fb6/>

### Refactor

 - <csr-id-c52de7a75ce4bbf872b215a14258b5e7778bfb34/> keep section Left/Relocated members in a separate archive container
   - Also remove an archived section member when its state is previous to last five Elder churn events

### Chore

 - <csr-id-366eee25f4b982d5a20d90168368a1aa14aa3181/> safe_network-0.55.0/sn_api-0.53.0/sn_cli-0.46.0
 - <csr-id-2cc7323ec8b62f89f2e4247c5d6ab56f78eda2ce/> clarify sap switch logic
 - <csr-id-2746cff087fd33aff523bc2df07a3462d05c6de1/> renaming apis for clarity of purpose
   Prior we had many 'send to target' type namings for send apis in node
   it wasn't always clear who/what could be sent throuh this and we've seen some enduser msgs
   being dropped there. (Perhaps only Backpressure reports... but still).
   
   This aims to clarify those APIs a bit, and provides a simple method to check
   if the src

### Bug Fixes

 - <csr-id-6aae745a4ffd7302fc74d5d548ca464066d5203f/> make log files rotate properly

### Refactor (BREAKING)

 - <csr-id-fa6014e581f52615971701572a1635dfde922fb6/> remove the relocation promise step for Elders
   - Both Adults and Elders now receive the `Relocate` message with all the relocation details
   without any previous or additional steps required to proceed with their relocation.

## v0.54.2 (2022-01-25)

<csr-id-77d45d7498facb18a611a8edcf608a3c0ff0b2c8/>
<csr-id-9d3a164efa7f38b7eeafc3936160458871956f5b/>
<csr-id-90cdbce964c8fd293d85b9a28f1b0d2d2a046b08/>
<csr-id-99df748624a36e1d2457cb81f74e8cd8accb8633/>

### Other

 - <csr-id-77d45d7498facb18a611a8edcf608a3c0ff0b2c8/> run many client test on ci

### Chore

 - <csr-id-9d3a164efa7f38b7eeafc3936160458871956f5b/> safe_network-0.54.2
 - <csr-id-90cdbce964c8fd293d85b9a28f1b0d2d2a046b08/> send queries to random elders in a section
   data holders are deterministic, which means any elder could
   successfully route to the relevant adults.
   
   This change should spread the load of popular data amongst
   all elders
 - <csr-id-99df748624a36e1d2457cb81f74e8cd8accb8633/> keep all logs by default

### Bug Fixes

 - <csr-id-501ce9838371b3fdcff6439a4a4f13a70423988f/> avoid infinite loop
 - <csr-id-16bd75af79708f88dcb7086d04fac8475f3b3190/> fix liveness tracking logic and keep track of newly joining adults

### New Features

 - <csr-id-67a3313105b31cee9ddd58de6e510384b8ae1397/> randomize elder contact on cmd.
   Priously we always took the first 3 from the SAP's elders vec. This should spread
   commands out over all elders more fairly.

## v0.54.1 (2022-01-24)

<csr-id-5a90ec673edcb36ae954b0af6144bae7d8243cd7/>
<csr-id-1ff0ab735994f8cbfae6abd58fea0e71a90f742c/>

### Chore

 - <csr-id-5a90ec673edcb36ae954b0af6144bae7d8243cd7/> safe_network-0.54.1
 - <csr-id-1ff0ab735994f8cbfae6abd58fea0e71a90f742c/> dont keep wiremsg in mem while sendin bytes

### Bug Fixes

 - <csr-id-151993ac6442224079dc02bfe476d2dfbc1a411b/> put connection ensurance and sending within the same spawned thread
 - <csr-id-39c8f5dceed5639daa226432f3d6f3c9bf19a852/> check Joined status on Info level

## v0.54.0 (2022-01-22)

<csr-id-8f58c5d900ce00d90bf7421f34122f4ca5ff5601/>
<csr-id-3b5ce194213a7090ee83c02b0043700cda230796/>
<csr-id-d84ac520676b83f39d5cf356e89ac3ec2d6478cc/>
<csr-id-0190f0305980bdaee30f9f2ab5eb5510149916db/>
<csr-id-d59999adcecd36380ed3a6855fdedfbef8107914/>
<csr-id-4e517a1fd57e63a3b5381ecd0cdf9db4f762e03f/>
<csr-id-3dc23278c6a4fabc250b27f4312f5c51f0f271a4/>
<csr-id-274cc12ca37da5ff536a7b4ab59f803546ccefe9/>
<csr-id-1b3c0eb4443cff3f4575164907a7a1fbf92cebe2/>
<csr-id-d96ded11ca74e75dde3dcc0d0b865712895d3dcc/>
<csr-id-7a7752f830785ec39d301e751dc75f228d43d595/>
<csr-id-959e909d8c61230ea143702858cb7b4c42caffbf/>

### chore (BREAKING)

 - <csr-id-8f58c5d900ce00d90bf7421f34122f4ca5ff5601/> making section members list non optional in AntiEntropyUpdate message type


### Refactor

 - <csr-id-3b5ce194213a7090ee83c02b0043700cda230796/> remove one layer of indirection

### Other

 - <csr-id-d84ac520676b83f39d5cf356e89ac3ec2d6478cc/> ignore many_client tests

### Chore

 - <csr-id-0190f0305980bdaee30f9f2ab5eb5510149916db/> safe_network-0.54.0/sn_api-0.52.0/sn_cli-0.45.0
 - <csr-id-d59999adcecd36380ed3a6855fdedfbef8107914/> make op_id u64, and use this for pending_query tracking
   Previously we were tracking pending queries using the data address, which with many queries from many clients for the same data would obviously be problematic.
   
   Here we use the operation_id And the peername to track returned data...
 - <csr-id-4e517a1fd57e63a3b5381ecd0cdf9db4f762e03f/> address some review comments
 - <csr-id-3dc23278c6a4fabc250b27f4312f5c51f0f271a4/> update remaining places
 - <csr-id-274cc12ca37da5ff536a7b4ab59f803546ccefe9/> resolve clippy failures after rebase
 - <csr-id-1b3c0eb4443cff3f4575164907a7a1fbf92cebe2/> wait before attempting to make_contact again
   Prior contact rounds did not between attempts. This gives the prev attempts more chance
   to succeed before we give up on them
 - <csr-id-d96ded11ca74e75dde3dcc0d0b865712895d3dcc/> dont fail client send w/ closed conn
   Closing a connection would cause a client error in send before this.
   Now we check if we had an application layer close... something intentional.
   If so we do not consider this a failure at the client and can
   continue thereafter.
   
   qp2p is upgraded to send a reason on close that we can check against.
 - <csr-id-7a7752f830785ec39d301e751dc75f228d43d595/> update year on files modified 2022

### Bug Fixes

 - <csr-id-8f5f7fd0c57db4a2cd29dfe2dc617b796fea97f4/> make client sending message non-blocking
 - <csr-id-36daaebd84e07c3ff55477e24598e6b9d0c0b314/> correct correlation_id for query response
 - <csr-id-56b6b11447010485c860a55e6dbd035d826091d1/> client wait longer for AE update got triggered
 - <csr-id-10123e6a3d18129fcb5c0090d409a02b2f762139/> using try_send to avoid blocking
 - <csr-id-1e4049fc944f8bcab0e51671219896854a3bffaa/> return CmdError to the caller
 - <csr-id-c0cc9807df988ca3a871720edadc4ff19e028f63/> sign to client msg with node's Ed key
 - <csr-id-cc38b212179645d99f5cf4f26cc5c43393d9db72/> JoinResponse::Redirect doesn't update prefix_map

### New Features

 - <csr-id-61e87de077bd47b3dd106dc1e1dd65aa1f5b2d0e/> retry intial client contact if it fails
   Previously we tried to connect to all known nodes, but if we reached the end of the list, we'd keep hitting the same nodes over and over.
   Now we fail after trying all candidates, and retry with a new xorname to get new candidates (if we know a prefixmap).
   We also mark the attempted connections as failed, so the client attempts to create
   fresh new connections instead of using possibly dead connections
 - <csr-id-a02f5f0de50c7c58b1cda55a14cb63d805c772c5/> clients mark closed connections as such
   And now check that the current connection is valid before trying to use it.
   Otherwise they reconnect
 - <csr-id-d88890261c2ca06498914714eefb56aff61673d5/> counting ACK/Err cmd response together and notify error to upper layer
 - <csr-id-939f353ed399d0148ba52225ca6d1676d3d7c04b/> timeout on waiting CmdAcks
 - <csr-id-b33b7dcb7fa6eb976bff7f38ac3d6ddc62e1977a/> send CmdAck on client cmd
 - <csr-id-65282654943bdf156f144e0449a85dc4e2956f58/> randomize elder contact on cmd.
   Priously we always took the first 3 from the SAP's elders vec. This should spread
   commands out over all elders more fairly.

### Refactor (BREAKING)

 - <csr-id-959e909d8c61230ea143702858cb7b4c42caffbf/> removing the internal routing layer

## v0.53.0 (2022-01-20)

<csr-id-8f58c5d900ce00d90bf7421f34122f4ca5ff5601/>
<csr-id-37d2522619e49212572aa37034e0ba1857679dc7/>
<csr-id-565711d4bc58f18181d279e5f29d9adb3acb051a/>
<csr-id-4eef4d3a9356d6a0724c0c670809e8d9fd1f11c0/>
<csr-id-c222370b52befaedebe895fcf7feb2c1da99aeaa/>
<csr-id-a018022e4eb298a9a66513c770c0edc1b415a886/>
<csr-id-893df83ec9a1c4231f09b4e7bad9c15ef2944036/>
<csr-id-08cde8586e018e19df6b35270bd999f45d30596e/>
<csr-id-fdda641b3874c425616352daf7db7429219bb858/>
<csr-id-923930acb3769cfa7047954a1fee1853ec9e3062/>
<csr-id-82f39977ec965429269a31639ce83be9749b5d80/>
<csr-id-b06bc6eb0fd210b6abc9039d1f20ab8d93befc16/>
<csr-id-f7f59369d09a4d25a94328d031f00cc61b187eed/>
<csr-id-96a955e5b124db3250c3d0fd09926cec10322632/>
<csr-id-57749b7d0671423fe205447bc84d9f8bfc99f54b/>
<csr-id-252f81e155dce2b8d774c8999b730e763674d93f/>
<csr-id-40b00cbd899c72889439b9f94b34b173ff3af837/>
<csr-id-941b83f3960c84cfee86a8c818233fbbc403c189/>
<csr-id-56bed01344ca5ec74a49c2a41116ef76fb33e3b4/>
<csr-id-026458c6afa9848bb58a694da6ae4b81196b8f19/>
<csr-id-f22e4dce22492079170cbaeb8c29b3911faaf89a/>

### Documentation

 - <csr-id-1d36f019159296db6d5a6bb4ebeec8699ea39f96/> add doc for usage of DKG in safe_network
   - also moves README from the .github folder so it doesn't show up in the
   repo root

### New Features

 - <csr-id-f81ade56b2b43cfbf81c6112d4c687d5b540b101/> verify blobs QueryResponse from adults and penalise if faulty
 - <csr-id-5cf1dc87b67e9af9e50299a75cdaf9d571408e10/> Add jitter into client query request timing
   This should help prevent nodes being hammered over the same period.
   This PR also increases the default client query time to 500s.

### Bug Fixes

 - <csr-id-5102396bd1cc1bc8c7765a9c8da74abe8ff02a1e/> new rust v and new clippy warnings
 - <csr-id-d4634f0e5e89e22d6ab3c70ec9135bfb80360c7e/> make owner semantics unbreakable
 - <csr-id-5bd1198aa8bd7f1b78282fb02b262e38f9121d78/> include ServiceAuth in DataQuery to Adults
 - <csr-id-42d3b932f69c3680e8aaff488395406983728ed6/> only send data cmd to 3 elders
 - <csr-id-a1de919c3d96c3ea5f566178989c9db9af01e468/> Store logging guard outwith of optional feat block.
   A code block was added to not log when using tokio console.
   This was resulting in our  being dropped and no node logging!
   This stores the gaurd outwith this block.
 - <csr-id-39f2cc24c1ad706cb7700b8a89585c2098f589c4/> on ae update, use pre update adults for checks
   Previously we'd had the  checking current
   against current adults. Which obviously did nothing.
   
   Now we grab adults before we update from AE flows. So we _should_ see
   changes, and initiate reorganisational flows for chunks etc.
 - <csr-id-2d339a094f3ccf589304dcddf6a16956280ca7a4/> use async/await over BoxFuture
   this seems to affect the regular functioning of the node
 - <csr-id-841b917977a484568615b540b267e3192cc95ed5/> dont .await on the root task
 - <csr-id-b55a7044fae9e578cb10b82a3b1db2ab6400b3db/> revert to original way of creating the Runtime
   at first tokio-console didn't work without the use of #[tokio::main]
   turns out it was something else, preventing the console from starting up
 - <csr-id-0f0ef819f4b36e5056dfa7f1983238bd4752caba/> read from disk on cache miss
 - <csr-id-e18c88019d37ab4f7618dde1a90e19ddf94db1c7/> VersioinHash use Display for encode
 - <csr-id-edefd2d7860e6f79c07060e078cdaa433da9e804/> use latest PK to send DataExchange messages
 - <csr-id-687434b5cccc3b796e18e4356ce2114b72c2c1ad/> update config smoke test expected size
 - <csr-id-4542a579ac4baadaeb06c10592f6d65fe148bafc/> make register query op-ids unique
 - <csr-id-83ef7a66bb245e2303b80d98d6b8fa888b93d6ba/> make use of all the queries
 - <csr-id-d080586074dea44b53a9901cb2e85599cc997379/> wait for a full section's info
   Before a client can perform put/query we shoud have at least one full
   section's info to work properly
 - <csr-id-06f980874357872e16535b0371c7e0e15a8d0a1c/> set DEFAULT_CHUNK_COPY_COUNT to 4 once again

### chore (BREAKING)

 - <csr-id-8f58c5d900ce00d90bf7421f34122f4ca5ff5601/> making section members list non optional in AntiEntropyUpdate message type


### Test

 - <csr-id-37d2522619e49212572aa37034e0ba1857679dc7/> send CmdAck for client cmd during test

### Refactor

 - <csr-id-565711d4bc58f18181d279e5f29d9adb3acb051a/> moving logic to check for split into a helper function
 - <csr-id-4eef4d3a9356d6a0724c0c670809e8d9fd1f11c0/> re-enable init with populated register
 - <csr-id-c222370b52befaedebe895fcf7feb2c1da99aeaa/> store cap within the register
 - <csr-id-a018022e4eb298a9a66513c770c0edc1b415a886/> only check permissions once

### Other

 - <csr-id-893df83ec9a1c4231f09b4e7bad9c15ef2944036/> fix client config query timeout check
 - <csr-id-08cde8586e018e19df6b35270bd999f45d30596e/> cleanup and cover both public/private
 - <csr-id-fdda641b3874c425616352daf7db7429219bb858/> use AE on all client tests

### Chore

 - <csr-id-923930acb3769cfa7047954a1fee1853ec9e3062/> safe_network-0.53.0/sn_api-0.51.0/sn_cli-0.44.0
 - <csr-id-82f39977ec965429269a31639ce83be9749b5d80/> moving relocation module within routing::Core where it belongs
 - <csr-id-b06bc6eb0fd210b6abc9039d1f20ab8d93befc16/> qp2p update/ clippy fixes and code comment cleanup
 - <csr-id-f7f59369d09a4d25a94328d031f00cc61b187eed/> allow 1gb uncompressed logs by default
 - <csr-id-96a955e5b124db3250c3d0fd09926cec10322632/> lessen nesting and indentation
 - <csr-id-57749b7d0671423fe205447bc84d9f8bfc99f54b/> solving new clippy findings
 - <csr-id-252f81e155dce2b8d774c8999b730e763674d93f/> Don't tie idle_timeout to query_timeout
 - <csr-id-40b00cbd899c72889439b9f94b34b173ff3af837/> remove unused old file
 - <csr-id-941b83f3960c84cfee86a8c818233fbbc403c189/> fix additional wrongly setup test cases
 - <csr-id-56bed01344ca5ec74a49c2a41116ef76fb33e3b4/> removing unused routing Command and API
 - <csr-id-026458c6afa9848bb58a694da6ae4b81196b8f19/> add checks to ensure all nodes have joined

### Refactor (BREAKING)

 - <csr-id-f22e4dce22492079170cbaeb8c29b3911faaf89a/> removing MIN_AGE and the concept of mature node
   - Also refactoring routing internal API for querying section peers from its network knowledge.

## v0.52.13 (2022-01-06)

<csr-id-155ee032ee56cbbb34928f2d14529273ccb69559/>

### Chore

 - <csr-id-155ee032ee56cbbb34928f2d14529273ccb69559/> safe_network-0.52.13/sn_api-0.50.6

### Bug Fixes

 - <csr-id-edefd2d7860e6f79c07060e078cdaa433da9e804/> use latest PK to send DataExchange messages

## v0.52.12 (2022-01-06)

<csr-id-81282318ae2da1793e66f28f0c8b3c0b2272a529/>
<csr-id-7928ce6411d237078f7ed3ba83823f438f3a991f/>
<csr-id-f1afc5933dc782bc6a7840cd12cebb32a189a5df/>
<csr-id-4f29c285a0b48220df1f1c6c52c4b487350eae08/>
<csr-id-db515397771f117b3bf095e1a4afb897eb4acafe/>
<csr-id-bebdae9d52d03bd13b679ee19446452990d1e2cf/>

### Refactor

 - <csr-id-81282318ae2da1793e66f28f0c8b3c0b2272a529/> clients and node read/write to a global prefix_map
 - <csr-id-7928ce6411d237078f7ed3ba83823f438f3a991f/> move to adults

### Other

 - <csr-id-f1afc5933dc782bc6a7840cd12cebb32a189a5df/> increase testnet startup interval on CI only.
   Lower the default value for smoother development networks

### Chore

 - <csr-id-4f29c285a0b48220df1f1c6c52c4b487350eae08/> safe_network-0.52.12
 - <csr-id-db515397771f117b3bf095e1a4afb897eb4acafe/> sn_cli-0.43.1
 - <csr-id-bebdae9d52d03bd13b679ee19446452990d1e2cf/> rename dest to dst

### Bug Fixes

 - <csr-id-06f980874357872e16535b0371c7e0e15a8d0a1c/> set DEFAULT_CHUNK_COPY_COUNT to 4 once again
 - <csr-id-112d0b474b8dd141d0daf1302b80055482d65a15/> unnecessary fetch_add for loading

### New Features

 - <csr-id-878e4bb43865933502a22f7cefb861bb6d72195c/> locally track query listeners to avoid overwrite/removal
   Previously each query for an op_id overwrote the listener.
   This could lead to odd behaviour for parallel requests in the same client.
   Now we only remove the listener for our msg_id even if it's the same
   operation_id
 - <csr-id-fff2d52b700dfe7ec9a8909a0d5adf176de4c5c7/> substract space from used_space on register delete
 - <csr-id-1b8681838d810aa2b4ef0abfaf9106678ff7cebb/> impl Ordering for NetworkPrefixMap based on it's length

## v0.52.11 (2022-01-06)

<csr-id-99d012ef529df78ef4c84f5e6ea99d3a77414797/>

### Chore

 - <csr-id-99d012ef529df78ef4c84f5e6ea99d3a77414797/> safe_network-0.52.11/sn_api-0.50.5/sn_cli-0.43.2

### Bug Fixes

 - <csr-id-42d3b932f69c3680e8aaff488395406983728ed6/> only send data cmd to 3 elders

### Documentation

 - <csr-id-1d36f019159296db6d5a6bb4ebeec8699ea39f96/> add doc for usage of DKG in safe_network
   - also moves README from the .github folder so it doesn't show up in the
   repo root

## v0.52.10 (2022-01-05)

<csr-id-9c9a537ad12cc809540df321297c8552c52a8648/>
<csr-id-7b0cd4d183a9f82d1d78dbb56572e5bf36714a0a/>
<csr-id-233e64af3b2af63bbba06ba8f43d0a7becece913/>
<csr-id-9f1bd81b269ec06fd5d379ab4b07f18f814da865/>
<csr-id-bf16c5ea7051386064233443921438cbbd79d907/>
<csr-id-012e91a60e01cd2ce9155d5c56045f211865ff2c/>

### Refactor

 - <csr-id-9c9a537ad12cc809540df321297c8552c52a8648/> ties up the loose ends in unified data flow

### Chore

 - <csr-id-7b0cd4d183a9f82d1d78dbb56572e5bf36714a0a/> safe_network-0.52.10
 - <csr-id-233e64af3b2af63bbba06ba8f43d0a7becece913/> ignore GA failing test
 - <csr-id-9f1bd81b269ec06fd5d379ab4b07f18f814da865/> misc fixes
 - <csr-id-bf16c5ea7051386064233443921438cbbd79d907/> log EntryHash human readable
 - <csr-id-012e91a60e01cd2ce9155d5c56045f211865ff2c/> remove unused dep async recursion

### Bug Fixes

 - <csr-id-ad202c9d24da5a5e47d93a3793d665e1b844b38d/> dont .await on threads spawned for subtasks
 - <csr-id-e18c88019d37ab4f7618dde1a90e19ddf94db1c7/> VersioinHash use Display for encode

### New Features

 - <csr-id-c47aeca2618d54a8b3d7b21c82f6ac6e62acd10c/> refactor node to support tokio-console and resolve issues

## v0.52.9 (2022-01-04)

<csr-id-a64c7e0414b77f545cb8cdbf64af0fb7212d1f2e/>

### Chore

 - <csr-id-a64c7e0414b77f545cb8cdbf64af0fb7212d1f2e/> safe_network-0.52.9/sn_api-0.50.4

### Bug Fixes

 - <csr-id-edefd2d7860e6f79c07060e078cdaa433da9e804/> use latest PK to send DataExchange messages

### New Features

 - <csr-id-fff2d52b700dfe7ec9a8909a0d5adf176de4c5c7/> substract space from used_space on register delete

## v0.52.8 (2022-01-04)

<csr-id-c790077bebca691f974000278d5525f4b011b8a7/>
<csr-id-5214d5e7f84a3c1cf213097a5d55bfb293f03324/>
<csr-id-6ccb792c18481ffd8218cd7c27b28d8a10d1f528/>

### Refactor

 - <csr-id-c790077bebca691f974000278d5525f4b011b8a7/> rename blob to file

### Chore

 - <csr-id-5214d5e7f84a3c1cf213097a5d55bfb293f03324/> safe_network-0.52.8
 - <csr-id-6ccb792c18481ffd8218cd7c27b28d8a10d1f528/> some detailed logging for debugging

### New Features

 - <csr-id-a7531b434591c39fc91bdffc413fcb2d6ed47e7a/> restore NotEnoughSpace checks and reunite reg and chunk storage tracking
 - <csr-id-19d7d3ad04a428485738ffc916b4f14388ad10d5/> optimise disk space checks by doing them less often
 - <csr-id-c47aeca2618d54a8b3d7b21c82f6ac6e62acd10c/> refactor node to support tokio-console and resolve issues
 - <csr-id-878e4bb43865933502a22f7cefb861bb6d72195c/> locally track query listeners to avoid overwrite/removal
   Previously each query for an op_id overwrote the listener.
   This could lead to odd behaviour for parallel requests in the same client.
   Now we only remove the listener for our msg_id even if it's the same
   operation_id

### Bug Fixes

 - <csr-id-112d0b474b8dd141d0daf1302b80055482d65a15/> unnecessary fetch_add for loading
 - <csr-id-ad202c9d24da5a5e47d93a3793d665e1b844b38d/> dont .await on threads spawned for subtasks
 - <csr-id-9f89966e02c3e0ba0297377b4efdf88a31ec1e87/> restore original behavior

## v0.52.7 (2022-01-04)

<csr-id-da3bbe16084b71ec42343035087848c8f6996ec4/>
<csr-id-b35d0ccd0305e3e87a9070bc2a57287dbe6b2633/>
<csr-id-2ebd1b4fcab47bc86980860379891bb041ff2aa4/>
<csr-id-40d1844e0b28578e8b8c6b270151dbb86961a766/>
<csr-id-bd0382fef77947935584418ee91720001f5f269c/>

### Refactor

 - <csr-id-da3bbe16084b71ec42343035087848c8f6996ec4/> record it in an atomic usize

### Other

 - <csr-id-b35d0ccd0305e3e87a9070bc2a57287dbe6b2633/> reorder network startup
   - Remove unverifiable asserts around query msg counts
   * Alters order of testnet startup vs building tests. As yet unverified,
   but building the tests is CPU intensive and could possibly slow down
   section startup leading to some test failures as we dont have all elders
   yet... Just a theory. We shall see...
 - <csr-id-2ebd1b4fcab47bc86980860379891bb041ff2aa4/> use zsh to repeat e2e tests
   We've seen occasionally flakey e2e tests. This sets the basic e2e tests to run 5 times back to back, which should increase confidence in each PR being merged and reduce fluke ci-passes

### Chore

 - <csr-id-40d1844e0b28578e8b8c6b270151dbb86961a766/> safe_network-0.52.7
 - <csr-id-bd0382fef77947935584418ee91720001f5f269c/> remove unneeded retry_loop from reg tests
   Splits out batching into its own tests that takes significantly longer than the rest

### New Features

 - <csr-id-19d7d3ad04a428485738ffc916b4f14388ad10d5/> optimise disk space checks by doing them less often

## v0.52.6 (2022-01-04)

<csr-id-838c495c8858b85c693da1a3e45baafa57ba03ea/>
<csr-id-36ba7d5e85e304d6d0ff3210429923beed77d25b/>
<csr-id-0a70425fb314de4c165da54fdc29a127ae900d81/>
<csr-id-1caad35e0e744e50b2bd15dda8dbd3adbacb87c7/>
<csr-id-78e41a3d8387f9b53bfd5e078ae7aa44fe1ea6d4/>
<csr-id-04ee3e74e5573f903be29cd89416ce9e5758cf00/>
<csr-id-a9913a0f7140d302fcaf24264fc1982f2ad3d06b/>
<csr-id-97171142548772a466188f2e6d9f24072f28640d/>
<csr-id-a7e7908537d63e4071323a59cbbd036edcff41ab/>
<csr-id-18cee44f08aa4f83ad477cc82a29525e9d339e0c/>
<csr-id-1d19c02668dfa3739a350b15c7310daec93d9837/>
<csr-id-2492ea84e9fcba5d19022e171ec6b60c341ee59b/>
<csr-id-8884f9453a859bd63b378337aab326889d153768/>
<csr-id-690b24f14c3183640d04d048c2a7f4ac79f6e6c7/>
<csr-id-3b3a38130bd6943b7c53b1cf74321d89dd4af1da/>
<csr-id-8effd08c16cfd2e0715ee0d00092e267f72a8cf0/>

### Refactor

 - <csr-id-838c495c8858b85c693da1a3e45baafa57ba03ea/> rename + use glob const
   - Adds max_num_faulty_elders based on const assumption for
   max number of faulty Elders as 1/3.
   - Adds at_least_one_correct_elder based on max_num_faulty_elders.
   
   These numbers were made dependent on global consts, as was asked for in
   the todo comment prior to this change.
   
   Also, the number of Elders to whom a client sends a chunk is not based
   on the chunk copy count, but on our assumptions on max number of faulty
   Elders, hence renaming the variable.

### Other

 - <csr-id-36ba7d5e85e304d6d0ff3210429923beed77d25b/> add 5mb many client test
   This adds a test to check if many many client connections will result in high mem usage at nodes.

### Chore

 - <csr-id-0a70425fb314de4c165da54fdc29a127ae900d81/> safe_network-0.52.6/sn_api-0.50.2
 - <csr-id-1caad35e0e744e50b2bd15dda8dbd3adbacb87c7/> refactor wait until higher_prio
   Moves wait into its own function and keeps semaphore acquisition separate
 - <csr-id-78e41a3d8387f9b53bfd5e078ae7aa44fe1ea6d4/> reduce semaphore wait timeout
 - <csr-id-04ee3e74e5573f903be29cd89416ce9e5758cf00/> make unstable-command-prioritisation required for wait
   Otherwise no waits are performed and we just need to acquire the semaphore to proceed
 - <csr-id-a9913a0f7140d302fcaf24264fc1982f2ad3d06b/> disable waiting for higher prio messages before continuing
 - <csr-id-97171142548772a466188f2e6d9f24072f28640d/> change dkg interval
 - <csr-id-a7e7908537d63e4071323a59cbbd036edcff41ab/> improve formatting of priority match statements
 - <csr-id-18cee44f08aa4f83ad477cc82a29525e9d339e0c/> limit time waiting to acquire priority permit
 - <csr-id-1d19c02668dfa3739a350b15c7310daec93d9837/> tidy up some log messages
 - <csr-id-2492ea84e9fcba5d19022e171ec6b60c341ee59b/> put command prioritisation behind a feature flag.
   It has been noted that enabling prioritisation flat out may result in obscuring underlying bugs
   (such as those hidden behind the DkgUnderway flag). This way we can keep code and work out
   where/when to activate this thereafter (perhaps when ndoes are under stress eg).
 - <csr-id-8884f9453a859bd63b378337aab326889d153768/> add PermitInfo type
   This more clearly states what we're storing as we track Permit use for prioritisation.
 - <csr-id-690b24f14c3183640d04d048c2a7f4ac79f6e6c7/> stop everything if something more important is going on
 - <csr-id-3b3a38130bd6943b7c53b1cf74321d89dd4af1da/> ensure child commands dont remove root permit early
 - <csr-id-8effd08c16cfd2e0715ee0d00092e267f72a8cf0/> use constants for cmd priorities

### New Features

 - <csr-id-cc2c55a24bbe3edc63fd6a3a8553b10330960495/> discard JoinResponse messages in dispatcher
   Dispatcher is only created when we have joined the network already. Therefore we can safely ignore JoinResponses there.
   
   We now error out of the msg handling flow and so will log this unexpected message at the node.
 - <csr-id-158043cba04983a35f66df825dc803c68f3ea454/> Move command limit to service msgs only.
   Require a permit or drop a service msg level command. This should _hopefully_ stop mem leak due to waiting to handle messages coming in from clients
 - <csr-id-01ea1017749a1644737ff8654378f9db70b8a988/> Add hard limit to concurrent commands
   It may be that nodes can be overwhelmed when too many messages come in.
   
   Here there's a naiive impl to drop commands (and msgs) from being handled when we're overwhelmed
 - <csr-id-3058bf1a50be8a88ac0c8cb4a66278db7e186957/> reenable using constants for message priority
   and wait on higher priority message completion before spawning new
   Command tasks
   
   This reverts commit 6d1cdc64078de06a43281d924f58d01b615e9268.

## v0.52.5 (2022-01-04)

<csr-id-ab00eca916d6ed8a0a137004a6b9fd24e7217a70/>

### Chore

 - <csr-id-ab00eca916d6ed8a0a137004a6b9fd24e7217a70/> safe_network-0.52.5

### Bug Fixes

 - <csr-id-9f89966e02c3e0ba0297377b4efdf88a31ec1e87/> restore original behavior
 - <csr-id-22f6dbcba8067ef777b7bea0393673bb669893b4/> make all atomic ops relaxed

## v0.52.4 (2022-01-04)

<csr-id-4bb2adf52efdac6187fffc299018bf13f3398e14/>
<csr-id-0a5fca96af4d2627b842591775e77c09201ed655/>
<csr-id-3af8ddbee91f3403b86914d352a970e366d1fa40/>

### Chore

 - <csr-id-4bb2adf52efdac6187fffc299018bf13f3398e14/> safe_network-0.52.4/sn_api-0.50.1
 - <csr-id-0a5fca96af4d2627b842591775e77c09201ed655/> do not skip AE check for AE-Probe messages
 - <csr-id-3af8ddbee91f3403b86914d352a970e366d1fa40/> set testnet interval to 10s by default once again

### Bug Fixes

 - <csr-id-9f89966e02c3e0ba0297377b4efdf88a31ec1e87/> restore original behavior
 - <csr-id-a9e753de1465d3f6abeb4ccf9a5b31fc3a2172f5/> add decrease fn

### New Features

 - <csr-id-cc2c55a24bbe3edc63fd6a3a8553b10330960495/> discard JoinResponse messages in dispatcher
   Dispatcher is only created when we have joined the network already. Therefore we can safely ignore JoinResponses there.
   
   We now error out of the msg handling flow and so will log this unexpected message at the node.
 - <csr-id-158043cba04983a35f66df825dc803c68f3ea454/> Move command limit to service msgs only.
   Require a permit or drop a service msg level command. This should _hopefully_ stop mem leak due to waiting to handle messages coming in from clients
 - <csr-id-01ea1017749a1644737ff8654378f9db70b8a988/> Add hard limit to concurrent commands
   It may be that nodes can be overwhelmed when too many messages come in.
   
   Here there's a naiive impl to drop commands (and msgs) from being handled when we're overwhelmed
 - <csr-id-3058bf1a50be8a88ac0c8cb4a66278db7e186957/> reenable using constants for message priority
   and wait on higher priority message completion before spawning new
   Command tasks
   
   This reverts commit 6d1cdc64078de06a43281d924f58d01b615e9268.

## v0.52.3 (2022-01-03)

<csr-id-6b9b1590bce9423130210007ddd6e9c14b51819d/>
<csr-id-be8989971c68ac5aee43380223000a1f400252fc/>
<csr-id-292466119e2d99c36043e7f2247b1bde9ec9ced9/>
<csr-id-d54c955aa768ab08ef8193b7e36cb96822bc6cb8/>
<csr-id-72f79d46fc56fdda9215f8a9d6f95bcdf323a66f/>
<csr-id-e9a9cc1096e025d88f19390ad6ba7398f71bc800/>

### Refactor

 - <csr-id-6b9b1590bce9423130210007ddd6e9c14b51819d/> do not check if space available
 - <csr-id-be8989971c68ac5aee43380223000a1f400252fc/> remove repetitive code

### Chore

 - <csr-id-292466119e2d99c36043e7f2247b1bde9ec9ced9/> safe_network-0.52.3
 - <csr-id-d54c955aa768ab08ef8193b7e36cb96822bc6cb8/> fmt
 - <csr-id-72f79d46fc56fdda9215f8a9d6f95bcdf323a66f/> typos and shortcuts

### Refactor (BREAKING)

 - <csr-id-e9a9cc1096e025d88f19390ad6ba7398f71bc800/> removing unnecessary Error type definitions

### New Features

 - <csr-id-5fadc027b4b7dd942275ef6041b2bdb92b062bed/> remove unused kv store, cleanup chunk store
 - <csr-id-7f1ead2f0f33558583989a7314d2c121a6f1280a/> disk chunk store
 - <csr-id-eb30133e773124b46cdad6e6fa7f3c65f066946a/> make read and delete async

## v0.52.2 (2022-01-03)

<csr-id-d490127b17d53a7648f9e97aae690b232188b034/>
<csr-id-9d7e6843701465a13de7b528768273bad02e920e/>
<csr-id-e9a9cc1096e025d88f19390ad6ba7398f71bc800/>

### Chore

 - <csr-id-d490127b17d53a7648f9e97aae690b232188b034/> safe_network-0.52.2
 - <csr-id-9d7e6843701465a13de7b528768273bad02e920e/> fix docs to match parameters

### Refactor (BREAKING)

 - <csr-id-e9a9cc1096e025d88f19390ad6ba7398f71bc800/> removing unnecessary Error type definitions

## v0.52.1 (2022-01-03)

<csr-id-e38925e07d69432db310fc8ec9803200ea964ab2/>
<csr-id-36dbea03d879c07f922be36a124ad8d44c3c2d0e/>
<csr-id-48ef44e9db01d74119a2b1c9f7e7dae4ce988c57/>
<csr-id-619d142de8999d536e41ac5fe402a94d934689fb/>

### Chore

 - <csr-id-e38925e07d69432db310fc8ec9803200ea964ab2/> safe_network-0.52.1/sn_api-0.48.0/sn_cli-0.41.0
 - <csr-id-36dbea03d879c07f922be36a124ad8d44c3c2d0e/> further integrate and organize dir
 - <csr-id-48ef44e9db01d74119a2b1c9f7e7dae4ce988c57/> move routing into node dir
 - <csr-id-619d142de8999d536e41ac5fe402a94d934689fb/> remove expired queries from pending queries cache

### Bug Fixes

 - <csr-id-f00de3a9cbc43eabeb0d46804a92b88204a48ea4/> respond only once to client for every chunk query

## v0.52.0 (2021-12-22)

<csr-id-6b59ad852f89f033caf2b3c7dfcfa3019f8129e8/>
<csr-id-1188ed58eed443b4b8c65b591376f2f9a21acc0d/>

### Chore

 - <csr-id-6b59ad852f89f033caf2b3c7dfcfa3019f8129e8/> safe_network-0.52.0/sn_api-0.47.0/sn_cli-0.40.0

### Bug Fixes

 - <csr-id-f00de3a9cbc43eabeb0d46804a92b88204a48ea4/> respond only once to client for every chunk query

### Refactor (BREAKING)

 - <csr-id-1188ed58eed443b4b8c65b591376f2f9a21acc0d/> minor refactor to error types definitions

## v0.51.7 (2021-12-20)

<csr-id-c76c3ab638188cba38911f037829c209fcc45fc3/>
<csr-id-07e19b53cd8eaa777f4c83369d2ee1076c75fe4f/>
<csr-id-069013b032b7fd8d8a58ca0d75f6ea357abf5593/>

### Chore

 - <csr-id-c76c3ab638188cba38911f037829c209fcc45fc3/> safe_network-0.51.7
 - <csr-id-07e19b53cd8eaa777f4c83369d2ee1076c75fe4f/> reduce query attempts
 - <csr-id-069013b032b7fd8d8a58ca0d75f6ea357abf5593/> removing exponential backoff when retrying queries

### Bug Fixes

 - <csr-id-44e57667f48b1fd9bce154652ae2108603a35c11/> send_query_with_retry_count now uses that retry count in its calculations

## v0.51.6 (2021-12-17)

<csr-id-79b2d0a3f52de0335323773936dee9bdbe12a0cf/>

### Chore

 - <csr-id-79b2d0a3f52de0335323773936dee9bdbe12a0cf/> safe_network-0.51.6

### New Features

 - <csr-id-1078e59be3a58ffedcd3c1460385b4bf00f18f6b/> use upload_and_verify by default in safe_client

### Bug Fixes

 - <csr-id-f083ea9200a1fccfc7bd21117f34c118702a7a70/> fix: adult choice per chunk and handling db errors

## v0.51.5 (2021-12-16)

<csr-id-45df3d71cc4b3185602b9d27b8cb0f5bf65a4b43/>
<csr-id-9cf7c72a94386f2cbe6f803be970c6debfbcb99b/>

### Chore

 - <csr-id-45df3d71cc4b3185602b9d27b8cb0f5bf65a4b43/> safe_network-0.51.5
 - <csr-id-9cf7c72a94386f2cbe6f803be970c6debfbcb99b/> attempt to retrieve all bytes during upload_and_verify

### New Features

 - <csr-id-1078e59be3a58ffedcd3c1460385b4bf00f18f6b/> use upload_and_verify by default in safe_client
 - <csr-id-3e91a43676e1252a872a24872db8f91c729bfb15/> keeping Chunks wich were retrieved in a local client cache

### Bug Fixes

 - <csr-id-d8ad6a9b1d6a530b7f597bccf6a6bed6d8546ac0/> populate client sender success counter

## v0.51.4 (2021-12-16)

<csr-id-17d7906656bec401d6b39cc3551141112a3d77c4/>

### Chore

 - <csr-id-17d7906656bec401d6b39cc3551141112a3d77c4/> safe_network-0.51.4

### New Features

 - <csr-id-3e91a43676e1252a872a24872db8f91c729bfb15/> keeping Chunks wich were retrieved in a local client cache

### Bug Fixes

 - <csr-id-d8ad6a9b1d6a530b7f597bccf6a6bed6d8546ac0/> populate client sender success counter

## v0.51.3 (2021-12-16)

<csr-id-92cdb53391652651bfe9a47c5a0261ba10f38148/>
<csr-id-9be440b36db07e1c04ab688b44ef91e4a56ed576/>

### Refactor

 - <csr-id-92cdb53391652651bfe9a47c5a0261ba10f38148/> defining a const for Chunks cache size

### Chore

 - <csr-id-9be440b36db07e1c04ab688b44ef91e4a56ed576/> safe_network-0.51.3/sn_api-0.46.1

### New Features

 - <csr-id-3e91a43676e1252a872a24872db8f91c729bfb15/> keeping Chunks wich were retrieved in a local client cache

## v0.51.2 (2021-12-16)

<csr-id-595541b83284a5c5b60fbc00e47b1146117d7613/>
<csr-id-f522d9aad0071cd3b47b1a3e4c178b0100ec71d6/>

### Chore

 - <csr-id-595541b83284a5c5b60fbc00e47b1146117d7613/> safe_network-0.51.2
 - <csr-id-f522d9aad0071cd3b47b1a3e4c178b0100ec71d6/> add more logs regarding storage levels

## v0.51.1 (2021-12-15)

<csr-id-dcbb67fc699d7cb1f3a2c4632bcb8a5738916091/>
<csr-id-e52a0c063f747e0be1525f07f8f759f4b9d042a7/>
<csr-id-df87fcf42c46dc28e6926394c120fbf2c715e54a/>

### Chore

 - <csr-id-dcbb67fc699d7cb1f3a2c4632bcb8a5738916091/> safe_network-0.51.1
 - <csr-id-e52a0c063f747e0be1525f07f8f759f4b9d042a7/> add recursion limit for clippy
 - <csr-id-df87fcf42c46dc28e6926394c120fbf2c715e54a/> move Peer to root of sn
   This will let us use the same struct across routing and clients

### New Features

 - <csr-id-591ce5f6dfa14143114fbd16c1d632a8dbe4a2d1/> Use Peers to ensure connection reuse

## v0.51.0 (2021-12-15)

<csr-id-c685838d8f9c10b0f4e7541fe201862bb84e8555/>
<csr-id-7c34940401b0115105d9b818b9f93c39d7669eed/>

### Chore

 - <csr-id-c685838d8f9c10b0f4e7541fe201862bb84e8555/> safe_network-0.51.0
 - <csr-id-7c34940401b0115105d9b818b9f93c39d7669eed/> add in distinct client errors for comms failure
   Some situations were mislabelled as InsuffcientElderConnection, implying failed connections, but it was actually a failure in our network knowledge.
   
   This commit adds clarifying error types for this situation

### New Features (BREAKING)

 - <csr-id-134dfa29b3698fb233095194305d9bbbba2875c7/> different flavors of upload data API, with/without verifying successful upload

## v0.50.0 (2021-12-14)

<csr-id-653f653a775a101679904ab75c8012a72dfdedfb/>

### Chore

 - <csr-id-653f653a775a101679904ab75c8012a72dfdedfb/> safe_network-0.50.0

### New Features (BREAKING)

 - <csr-id-134dfa29b3698fb233095194305d9bbbba2875c7/> different flavors of upload data API, with/without verifying successful upload

## v0.49.3 (2021-12-14)

<csr-id-edb8de8b4d923e97d68eed40a7953f38461b0281/>
<csr-id-36ca20e606899ecbdea24d845c34ba11ab889cf7/>

### Test

 - <csr-id-edb8de8b4d923e97d68eed40a7953f38461b0281/> adding a test for retrieving Blob with range over data length

### Chore

 - <csr-id-36ca20e606899ecbdea24d845c34ba11ab889cf7/> safe_network-0.49.3

## v0.49.2 (2021-12-14)

<csr-id-62d747969b739172910aabca6fcb273d2827fc8a/>
<csr-id-dd50d1d860aa4ca60b6c0d5a525b45d88ddf432e/>

### Chore

 - <csr-id-62d747969b739172910aabca6fcb273d2827fc8a/> safe_network-0.49.2
 - <csr-id-dd50d1d860aa4ca60b6c0d5a525b45d88ddf432e/> set default AE_WAIT to be 0

### New Features

 - <csr-id-86ba4234a29137518c73b18becbf018993e104a8/> on initial contact put all known elders into the contact pool.
   Previously if we knew a sap, we only took 3 nodes
 - <csr-id-99add55c5ca5a3e3da2130797083dd449da2f7cd/> make contact via register get
   We previously use a chunk get, but this in itself will cause more network messaging than a simple register get which can be dealt with by elders only
 - <csr-id-2bdc03578f3d9144a097a947ab44d0c1286f6180/> use backoff during make contact instead of standard_wait.
   This should help aleviate any pressure on already struggling nodes, especially if a low wait was set to make tests run faster eg (where ae may not always be needed

## v0.49.1 (2021-12-13)

<csr-id-69ae8c20e91dd9959ebfa5456efdf9c218a9d66f/>
<csr-id-2e7bc0b782da6231f54edc440fa555fa754d294c/>

### Chore

 - <csr-id-69ae8c20e91dd9959ebfa5456efdf9c218a9d66f/> safe_network-0.49.1
 - <csr-id-2e7bc0b782da6231f54edc440fa555fa754d294c/> set DEFAULT_QUERY_TIMEOUT to 120s

### New Features

 - <csr-id-86ba4234a29137518c73b18becbf018993e104a8/> on initial contact put all known elders into the contact pool.
   Previously if we knew a sap, we only took 3 nodes
 - <csr-id-99add55c5ca5a3e3da2130797083dd449da2f7cd/> make contact via register get
   We previously use a chunk get, but this in itself will cause more network messaging than a simple register get which can be dealt with by elders only
 - <csr-id-2bdc03578f3d9144a097a947ab44d0c1286f6180/> use backoff during make contact instead of standard_wait.
   This should help aleviate any pressure on already struggling nodes, especially if a low wait was set to make tests run faster eg (where ae may not always be needed

## v0.49.0 (2021-12-13)

<csr-id-88c78e8129e5092bd120d0fc6c9696673550be9d/>
<csr-id-6f5516d8bb677462ea6def46aa65a1094767d68c/>
<csr-id-1a81c8f04f947d2b83d3cd726c00ad66927f5225/>
<csr-id-a569474a8be9c11ab73ec7ad1ad157f69827b4d3/>

### New Features

 - <csr-id-35467d3e886d2824c4f9e4586666cab7a7960e54/> limit the relocations at one time

### Bug Fixes

 - <csr-id-8955fcf9d69e869725177340d1de6b6b1e7a203b/> read_from client API was incorrectly using provided length value as an end index
   - Minor refactoring in sn_api moving the SafeData struct into its own file.

### chore (BREAKING)

 - <csr-id-88c78e8129e5092bd120d0fc6c9696673550be9d/> rename enum variants for improved clarity on flows

### Chore

 - <csr-id-6f5516d8bb677462ea6def46aa65a1094767d68c/> safe_network-0.49.0
 - <csr-id-1a81c8f04f947d2b83d3cd726c00ad66927f5225/> replace use of deprecated dalek function
   See the comment on the function for an explanation as to why this was changed.
 - <csr-id-a569474a8be9c11ab73ec7ad1ad157f69827b4d3/> tidy up some log messages

## v0.48.0 (2021-12-10)

<csr-id-7eda2760da82b3079d5eee2f97e2d15ac8da0d57/>
<csr-id-9eff03598ba09aa339180d7ecd57b50174180095/>
<csr-id-58632a27d271140fc4d777f25a76b0daea582426/>
<csr-id-0d4343c8fa56749d3ec9390e298d1d6384573a67/>
<csr-id-85709655b0ce38246515658b956aa9b8f67cb55a/>

### Test

 - <csr-id-7eda2760da82b3079d5eee2f97e2d15ac8da0d57/> checking Relocation during CI

### Chore

 - <csr-id-9eff03598ba09aa339180d7ecd57b50174180095/> safe_network-0.48.0
 - <csr-id-58632a27d271140fc4d777f25a76b0daea582426/> minor improvement to client log msgs related to configured timeouts
 - <csr-id-0d4343c8fa56749d3ec9390e298d1d6384573a67/> remove Generic client error
 - <csr-id-85709655b0ce38246515658b956aa9b8f67cb55a/> safe_network-0.47.0

### New Features

 - <csr-id-fd31bd2ef5ccc9149f8f0a2844c52af60bff3840/> use Vec of DataCmd instead of wrapper struct
 - <csr-id-84ebec0e1b2b3e1107d673734125aadbdb108472/> upgrade batch to write ahead log with index

### Bug Fixes

 - <csr-id-661e994cb452242a1d7c831ab88b9a66a244faff/> avoid change name by mistake when Join
 - <csr-id-58bd78e8a4fc34e7f76a9f449301b9216cb6a1d4/> complete the relocation flow

### New Features (BREAKING)

 - <csr-id-565e57619557e2b63f028eb214b59fc69b77fc37/> register op batching

## v0.47.0 (2021-12-08)

### New Features

 - <csr-id-fd31bd2ef5ccc9149f8f0a2844c52af60bff3840/> use Vec of DataCmd instead of wrapper struct
 - <csr-id-84ebec0e1b2b3e1107d673734125aadbdb108472/> upgrade batch to write ahead log with index

### Bug Fixes

 - <csr-id-58bd78e8a4fc34e7f76a9f449301b9216cb6a1d4/> complete the relocation flow

### New Features (BREAKING)

 - <csr-id-565e57619557e2b63f028eb214b59fc69b77fc37/> register op batching

## v0.46.6 (2021-12-07)

<csr-id-b5e9dcc5b13b1eda711d4760d9feb8dc929a0c43/>
<csr-id-05f6d98cf21f0158f4b5161484c7c15a0561b6f4/>

### Chore

 - <csr-id-b5e9dcc5b13b1eda711d4760d9feb8dc929a0c43/> safe_network-0.46.6
 - <csr-id-05f6d98cf21f0158f4b5161484c7c15a0561b6f4/> clippy tidyup for rust 1.57

### Bug Fixes

 - <csr-id-ddde0d6767f5230eca7a760423538fa5e4f640a2/> order of target length in ElderConn error corrected
 - <csr-id-1ea33e0c63fead5097b57496b2fd997201a2c531/> keep trying even with connection error w/ always-joinable flag
 - <csr-id-8e480e5eb5cd38f9bfb3b723aa1d0dd5e4be3122/> aovid overwrite pervious session by  accident
 - <csr-id-fdc4ba6bb36dff85126c8273cdfc67f9b07e2175/> multiple fixes for DKG-AE
   - discard DKG session info for older sessions

## v0.46.5 (2021-12-02)

<csr-id-86577846e845c110c49e15c95c6bd5595db51773/>

### Chore

 - <csr-id-86577846e845c110c49e15c95c6bd5595db51773/> safe_network-0.46.5

### Bug Fixes

<csr-id-01a5c961fc02f9ca8d6f60286306d5efba460e4e/>

 - <csr-id-fdc4ba6bb36dff85126c8273cdfc67f9b07e2175/> multiple fixes for DKG-AE
   - discard DKG session info for older sessions

## v0.46.4 (2021-12-02)

<csr-id-de3051e7e809a8f75507c54f3cf053a4244fdf19/>

### Chore

 - <csr-id-de3051e7e809a8f75507c54f3cf053a4244fdf19/> safe_network-0.46.4

### Bug Fixes

 - <csr-id-01a5c961fc02f9ca8d6f60286306d5efba460e4e/> avoid network_knowledge loopup when sending DkgRetry or DkgSessionInfo
 - <csr-id-27868bc51ba2dd12cc584396c53a158706d0c07b/> avoid network_knowledge lookup when sending DKGNotReady or DkgSessionUnknown

## v0.46.3 (2021-12-02)

<csr-id-69e9be2a1567bfa211af7e9d7595381d9a0a3b38/>

### Chore

 - <csr-id-69e9be2a1567bfa211af7e9d7595381d9a0a3b38/> safe_network-0.46.3

### New Features

 - <csr-id-5b003745ce3421b0554f35a6198d58cf67a1f4c6/> aggregate joins per section key
   Previously we only did one aggregation. But errors could arise if we had to resend and scrap aggregation.
   
   This means we can aggregate any number of churning section keys.

### Bug Fixes

 - <csr-id-27868bc51ba2dd12cc584396c53a158706d0c07b/> avoid network_knowledge lookup when sending DKGNotReady or DkgSessionUnknown

## v0.46.2 (2021-12-01)

<csr-id-260eaabd2d1b0c26dec9febc963929e65d7ec912/>
<csr-id-b4f0306fb945cce096de7f68c3cf6ece6905786d/>
<csr-id-204f927220c8bd1829ac89feaed1c48a8034e80b/>
<csr-id-2ef8e45f53ff6925e56321ed0ebc922d2d4dd9b9/>
<csr-id-1eea84476294c1dbfbfe72fe0e9acdb997762595/>

### Chore

 - <csr-id-260eaabd2d1b0c26dec9febc963929e65d7ec912/> safe_network-0.46.2
 - <csr-id-b4f0306fb945cce096de7f68c3cf6ece6905786d/> remove sig check on join ApprovalShare receipt
   Now we have a map of keys and aggregators, we dont need this check here.
   If keys dont match, we'll get an error and initiate join sending again below
 - <csr-id-204f927220c8bd1829ac89feaed1c48a8034e80b/> remove Joinrejection DkgUnderway
 - <csr-id-2ef8e45f53ff6925e56321ed0ebc922d2d4dd9b9/> leave longer wait on testnet startup
 - <csr-id-1eea84476294c1dbfbfe72fe0e9acdb997762595/> dont send dkg underway

### New Features

 - <csr-id-5b003745ce3421b0554f35a6198d58cf67a1f4c6/> aggregate joins per section key
   Previously we only did one aggregation. But errors could arise if we had to resend and scrap aggregation.
   
   This means we can aggregate any number of churning section keys.

### Bug Fixes

 - <csr-id-8de5ecf50008793546760621d906f23e8fe9791f/> make has_split.sh detect SplitSuccess correctly
 - <csr-id-8cfc5a4adf8a31bf6916273597cf4c18aa5adc46/> reduce backoff time; avoid start DKG shrink elders

## v0.46.1 (2021-12-01)

<csr-id-bf55f9b7e3b96319de4423e19333bf3b16fd1c78/>

### Chore

 - <csr-id-bf55f9b7e3b96319de4423e19333bf3b16fd1c78/> safe_network-0.46.1

### Bug Fixes

 - <csr-id-8de5ecf50008793546760621d906f23e8fe9791f/> make has_split.sh detect SplitSuccess correctly
 - <csr-id-8cfc5a4adf8a31bf6916273597cf4c18aa5adc46/> reduce backoff time; avoid start DKG shrink elders

## v0.46.0 (2021-12-01)

<csr-id-f3d3ab2b059040ff08b6239c8a6583c64eac160e/>
<csr-id-8ea94983b37b1d559358a62d6ca075b97c193f0d/>
<csr-id-3f77429e8bd659a5b2e7aa377437fac1b3d709c0/>
<csr-id-a7e058536ae6ae27228bd2254ea6465c5eface35/>

### New Features

 - <csr-id-24f1aa0208e3e474862c18b21ea9f048cb6abf25/> expose API for calculate Blob/Spot address without a network connection

### chore (BREAKING)

 - <csr-id-f3d3ab2b059040ff08b6239c8a6583c64eac160e/> include remote addrs in listerners threads log entries


### Chore

 - <csr-id-8ea94983b37b1d559358a62d6ca075b97c193f0d/> safe_network-0.46.0
 - <csr-id-3f77429e8bd659a5b2e7aa377437fac1b3d709c0/> unify skip port forwarding arguments
   We decided to rename the `--skip-igd` argument to `--skip-auto-port-forwarding`. I had initially
   renamed the arguments on the CLI and `sn_launch_tool` to `--disable-port-forwarding`, but it turned
   out that wasn't a completely accurate description of what's happening.
   
   We are now uniformly referring to this argument as `--skip-auto-port-forwarding`, which is quite an
   accurate description, as it skips (but not disables) the software-based port forwarding, leaving you
   to 'manually' setup port forwarding on your router, if need be.
   
   Also fixed some clippy warnings.
 - <csr-id-a7e058536ae6ae27228bd2254ea6465c5eface35/> safe_network-0.45.0

## v0.45.0 (2021-11-30)

<csr-id-f3d3ab2b059040ff08b6239c8a6583c64eac160e/>

### New Features

 - <csr-id-24f1aa0208e3e474862c18b21ea9f048cb6abf25/> expose API for calculate Blob/Spot address without a network connection

### chore (BREAKING)

 - <csr-id-f3d3ab2b059040ff08b6239c8a6583c64eac160e/> include remote addrs in listerners threads log entries


## v0.44.5 (2021-11-30)

<csr-id-d364a7e364827b2d71f196a9f897b7d613bbab94/>
<csr-id-0da3c99785e026075214b7cfa7933f64420aa00f/>
<csr-id-aaab10b3a5a44d9ec844757c71ac091016f51fd1/>
<csr-id-9deb9e7e2ae5d909eeb745e9021cf7660ed55dc3/>

### Other

 - <csr-id-d364a7e364827b2d71f196a9f897b7d613bbab94/> only check split count for health check for now
 - <csr-id-0da3c99785e026075214b7cfa7933f64420aa00f/> log desired elder count during health check

### Chore

 - <csr-id-aaab10b3a5a44d9ec844757c71ac091016f51fd1/> safe_network-0.44.5
 - <csr-id-9deb9e7e2ae5d909eeb745e9021cf7660ed55dc3/> add SN_AE_WAIT env var
   This unhooks the post-Put wait for AE messages from the query timeout.
   This should keep PUTs fast while allowing longer query timeouts as necessary, while
   also allowing it to be easily configured

## v0.44.4 (2021-11-30)

<csr-id-984c5f83e3f4d889dc4e0583b09571e540357cf9/>
<csr-id-e841ae0cf3caf852e8a48b559dcffb64b7fcecad/>

### Chore

 - <csr-id-984c5f83e3f4d889dc4e0583b09571e540357cf9/> safe_network-0.44.4
 - <csr-id-e841ae0cf3caf852e8a48b559dcffb64b7fcecad/> spawn writing prefix_map to disk as separate thread

### New Features

 - <csr-id-d2c352198c0e421cf4b7327a17c4343dc693b2dd/> start a fresh round of joins when aggregation is erroring.
   Aggregation may error, perhaps due to section churn. If we half or more bad signaturs that would prevent aggregation. We clean the slate and start a fresh round of aggregation and join messaging.

### Bug Fixes

 - <csr-id-758661d87edfef0fee6676993f1e96b3fd2d26bb/> don't crash node on failed signature aggregation on join
 - <csr-id-b79e8d6dbd18a4a0787d326a6b10146fa6b90f30/> retry joins on section key mismatch.
   We could sometimes get a key sent from nodes with a newer section_pk
   than we were expecting.
   
   We should then retry

## v0.44.3 (2021-11-30)

<csr-id-ec3dd4991535bb22235e2d1d413dd93489b8aedf/>
<csr-id-9885536afe337bf109e3ab18eec054100bf3fd82/>
<csr-id-cf67a374ae5aa5a9f95d39357fdbd937eb5e1a1a/>

### Chore

 - <csr-id-ec3dd4991535bb22235e2d1d413dd93489b8aedf/> safe_network-0.44.3
 - <csr-id-9885536afe337bf109e3ab18eec054100bf3fd82/> increase join node aggregatin expiration.
   Could be that some shares take time to come in under load/membership changes
 - <csr-id-cf67a374ae5aa5a9f95d39357fdbd937eb5e1a1a/> improve node logging

### New Features

 - <csr-id-d2c352198c0e421cf4b7327a17c4343dc693b2dd/> start a fresh round of joins when aggregation is erroring.
   Aggregation may error, perhaps due to section churn. If we half or more bad signaturs that would prevent aggregation. We clean the slate and start a fresh round of aggregation and join messaging.

### Bug Fixes

 - <csr-id-758661d87edfef0fee6676993f1e96b3fd2d26bb/> don't crash node on failed signature aggregation on join
 - <csr-id-b79e8d6dbd18a4a0787d326a6b10146fa6b90f30/> retry joins on section key mismatch.
   We could sometimes get a key sent from nodes with a newer section_pk
   than we were expecting.
   
   We should then retry
 - <csr-id-5523d0fcca7bbbbd05b6d125692f5fbd1a8f50d7/> avoid use outdated keyshare status

## v0.44.2 (2021-11-30)

<csr-id-51b0f0068c9a279da9a1edf45509cf80a90e663d/>
<csr-id-a3c5ef63f37ec0d2f98d45176e27da7de31baabc/>

### Chore

 - <csr-id-51b0f0068c9a279da9a1edf45509cf80a90e663d/> safe_network-0.44.2
 - <csr-id-a3c5ef63f37ec0d2f98d45176e27da7de31baabc/> move key_share check into network_knowledge_update

### Bug Fixes

 - <csr-id-5523d0fcca7bbbbd05b6d125692f5fbd1a8f50d7/> avoid use outdated keyshare status

## v0.44.1 (2021-11-29)

<csr-id-14c84c9db23557626e4889eff0ff403a574dccad/>
<csr-id-10e135b4f77dcd30de99eb7bc370ba1e15bbd148/>

### Chore

 - <csr-id-14c84c9db23557626e4889eff0ff403a574dccad/> safe_network-0.44.1
 - <csr-id-10e135b4f77dcd30de99eb7bc370ba1e15bbd148/> remove Core::is_dkg_underway flag which is not necessary

### Bug Fixes

 - <csr-id-0e505be2f57ab427cd3ed8c9564fd8b84909f6f3/> restore `ServiceMsg` authority check
   The `AuthorityProof` struct is designed to be a proof of valid
   authority, by ensuring all possible constructors either generate or
   validate a signature. This can only be guaranteed if the field remains
   module-private. At some point it seems the field was made `pub(crate)`,
   which meant we were missing an authority check for some `ServiceMsg`s,

## v0.44.0 (2021-11-26)

<csr-id-75a4b537573d4e5e8767e38fa7d1b1126dffe148/>

### Chore

 - <csr-id-75a4b537573d4e5e8767e38fa7d1b1126dffe148/> safe_network-0.44.0

### New Features

 - <csr-id-cc256bdf3f493f8841be07b9d7634c486e21a1cf/> avoid broadcasting DKG messages
 - <csr-id-60e6a5b1c1db4011c6bcdb473be3dbfea8858d6a/> revamp joins/rejoins to follow BRB

### Bug Fixes

 - <csr-id-0e505be2f57ab427cd3ed8c9564fd8b84909f6f3/> restore `ServiceMsg` authority check
   The `AuthorityProof` struct is designed to be a proof of valid
   authority, by ensuring all possible constructors either generate or
   validate a signature. This can only be guaranteed if the field remains
   module-private. At some point it seems the field was made `pub(crate)`,
   which meant we were missing an authority check for some `ServiceMsg`s,
 - <csr-id-23bd9b23243da1d20cee8e7f06adc90f0894c2e7/> fix node joining age for genesis section Adults
 - <csr-id-5dd0961c5172ed890b2453eb33332380e4757ad3/> refactor join flows to use the new Peer type
 - <csr-id-1822857e35b888df75bea652a2858cfd34b8c2fc/> remove redundant send_node_approval

### Bug Fixes (BREAKING)

 - <csr-id-94c91e16135e524202608b2dc6312102890eab94/> include the original message when requesting DkgSessionInfo
   - also early-returns when signature verification fails / the key is
   outdated

## v0.43.0 (2021-11-26)

<csr-id-0f2786483016adb2f44199cd4e5bf55e8c54adc3/>
<csr-id-ee6431e9509dd5b02ec2eca4c10ea17c7dddcfc9/>
<csr-id-d73deebbf94e65898d29455d6d4ed951ca59b814/>
<csr-id-3fa2957fe4f4a1f14b9001e101270dec4e386c14/>
<csr-id-612f8437429d4d9852f54fa0e297b059b8e86827/>
<csr-id-0ab3c35e424e60ffbed0edf8c1f71da48daa0978/>
<csr-id-c7b9d748b12e6d1e23d2a2972f2553af50343b22/>
<csr-id-d85d896ceaa68785797b1c525ae8653ef26f2708/>
<csr-id-1c5929bcc0638e8ff05dd495b3cedfdeff7540aa/>
<csr-id-e6df2ec1d2c8ab02bdaa95ed4c3288cfc0e532cc/>
<csr-id-ddd3726a0a2a131ae7c05a4190edb8bdc7fa3665/>
<csr-id-52a39b4c41b09ec33c60bc09c55fc63484e524cf/>
<csr-id-f69c0e319b639b678b46b73724a77ed7172cfec3/>
<csr-id-7ae741f93f39c437618bde150458eedd7663b512/>
<csr-id-c78f4703a970e8b7466b091ad331d0f2233aa9a3/>
<csr-id-2003e34cc26c450486151fa1df74ffea61c8c9ae/>
<csr-id-c051e02afdcbd2c21deec19b1baebfb8293d515c/>
<csr-id-016b7d47f708308490c1883db4ba239fc2add59b/>
<csr-id-580f2e26baf787bdee64c093a03ae9470668619d/>
<csr-id-da39113a7f97f6881643e66a48e19f2e06368e8d/>

### Refactor

 - <csr-id-0f2786483016adb2f44199cd4e5bf55e8c54adc3/> prefer existing connection when merging `Peer`s
   This is a probably-premature optimisation to minimise when we take write
   locks on `Peer::connection`. Previously when merging connections we
   would prioritise the source connection (e.g. we would overwrite an
   existing connection in the target). In particular, this would cause
   write-locking even if it was the same connection, which could lead to
   needless contention when trying to read connections.
   
   This commit changes to instead retain the existing connection if we have
   one, and does so in a way that minimises the likelihood that we need to
   take a write lock at all.
 - <csr-id-ee6431e9509dd5b02ec2eca4c10ea17c7dddcfc9/> prevent concurrent sends from opening many connections
   This is an optimisation in favour of reducing the number of connections
   we open, at the cost of reduced parallelism when sending multiple
   messages to the same unconnected `Peer` (which is fairly common when
   performing DKG rounds with new peers, e.g.).
   
   The implementation is based on giving `Peer` the responsibility of
   creating new connections if necessary, meaning it can take the write
   lock and see if any competing attempt already set a new connection - and
   if so, avoid creating another one. A 'fast path' is included that will
   return immediately if there's an existing valid connection.
 - <csr-id-d73deebbf94e65898d29455d6d4ed951ca59b814/> only recreate failed connections in `Comm::send`
   `Peer`s may hold connections that have been lost due to timeouts etc. As
   such, we want to handle connection loss when sending by trying again and
   forcing a reconnection. We would previously do this by passing a
   `force_reconnection` flag, however in the event of concurrent sends this
   might lead to more connections being opened than necessary.
   
   The new approach tests the existing connection to see if it's the same
   one that previously failed, and will only reconnect if so.
   
   This does not fully protect from 'stampedes' of new connections, since
   the same issue exists when the `Peer` has no connection (e.g. many
   concurrent sends for the same peer may lead to many connections being
   made), but this will be followed up separately.
 - <csr-id-3fa2957fe4f4a1f14b9001e101270dec4e386c14/> simplify `Comm::send` error handling
   There's now a dedicated error type for `Comm::send_to_one` that better
   represents the possible states. In particular, a connection error could
   never be from a reused connection, and we avoid having to wrap errors as
   much.
 - <csr-id-612f8437429d4d9852f54fa0e297b059b8e86827/> replace `Comm::send` closure with normal `fn`
   The closure was rather massive. Separating out makes it easier to see
   what's going on.
 - <csr-id-0ab3c35e424e60ffbed0edf8c1f71da48daa0978/> reuse system message sender connections
   Whenever we receive a valid system message, we now merge the sender's
   connection into our network knowledge, if applicable.
   
   An example of when this is useful is when a node uses their SAP to cold-
   call an elder who they have not yet spoken to. Without this change,
   there would be additional connections since the elder would reply over a
   new connection. With this change in place, they will instead be able to
   reply over the same sending connection.
 - <csr-id-c7b9d748b12e6d1e23d2a2972f2553af50343b22/> merge sender connection into SAP when joining
   This sets up joining nodes to reuse the connection they already have
   with the approving elder.
 - <csr-id-d85d896ceaa68785797b1c525ae8653ef26f2708/> remove connection pooling
   This completely removes `ConnectedPeers`. Prior changes ensure that
   connection usage remains fairly nominal (though still higher than
   optimal), and that clients are still reachable without reconnection.
 - <csr-id-1c5929bcc0638e8ff05dd495b3cedfdeff7540aa/> reuse more `Peer`s in DKG sessions
   We previously started to check our knowledge of section members for
   existing `Peer`s, however in some cases it seems that elders are not
   present in `members` (perhaps deliberately?). To ensure maximum reuse,
   we now also check the SAP for reusable connections.
 - <csr-id-e6df2ec1d2c8ab02bdaa95ed4c3288cfc0e532cc/> get section name from prefix and get section key from our
 - <csr-id-ddd3726a0a2a131ae7c05a4190edb8bdc7fa3665/> replace dkg message backlog with message exchange
   when a node has not received enough sig. shares to start processing a
 - <csr-id-52a39b4c41b09ec33c60bc09c55fc63484e524cf/> send a retry join message if DKG is underway
 - <csr-id-f69c0e319b639b678b46b73724a77ed7172cfec3/> adults join with age lesser than the youngest elder in genesis section

### Other

 - <csr-id-7ae741f93f39c437618bde150458eedd7663b512/> fixes node join/rejoin tests to support the BRB join process

### Chore

 - <csr-id-c78f4703a970e8b7466b091ad331d0f2233aa9a3/> safe_network-0.43.0
 - <csr-id-2003e34cc26c450486151fa1df74ffea61c8c9ae/> use bls-dkg 0.9.0
 - <csr-id-c051e02afdcbd2c21deec19b1baebfb8293d515c/> reduce the testnet node numbers and make name deduce more
 - <csr-id-016b7d47f708308490c1883db4ba239fc2add59b/> fixes to node joining and DKG flows
 - <csr-id-580f2e26baf787bdee64c093a03ae9470668619d/> add comments and use backoff for DKGUnderway on joining
 - <csr-id-da39113a7f97f6881643e66a48e19f2e06368e8d/> backoff BRB join response

### New Features

 - <csr-id-cc256bdf3f493f8841be07b9d7634c486e21a1cf/> avoid broadcasting DKG messages
 - <csr-id-60e6a5b1c1db4011c6bcdb473be3dbfea8858d6a/> revamp joins/rejoins to follow BRB

### Bug Fixes

 - <csr-id-23bd9b23243da1d20cee8e7f06adc90f0894c2e7/> fix node joining age for genesis section Adults
 - <csr-id-5dd0961c5172ed890b2453eb33332380e4757ad3/> refactor join flows to use the new Peer type
 - <csr-id-1822857e35b888df75bea652a2858cfd34b8c2fc/> remove redundant send_node_approval

### New Features (BREAKING)

 - <csr-id-3fe9d7a6624fe5503f80395f6ed11426b131d3b1/> move Url to sn_api

### Bug Fixes (BREAKING)

 - <csr-id-94c91e16135e524202608b2dc6312102890eab94/> include the original message when requesting DkgSessionInfo
   - also early-returns when signature verification fails / the key is
   outdated

## v0.42.0 (2021-11-25)

<csr-id-ca21d1e97fcd28ca351887636affffff78e3aeb3/>
<csr-id-bad083332fdcc3a0bc3c3f13628c00315f1b2519/>

### Chore

 - <csr-id-ca21d1e97fcd28ca351887636affffff78e3aeb3/> safe_network-0.42.0/sn_api-0.43.0
 - <csr-id-bad083332fdcc3a0bc3c3f13628c00315f1b2519/> remove url deps from sn

### New Features

 - <csr-id-a8c0e645d9bf11834557049045cca95f8d715a77/> deduce expected_prefix from expected_age

### New Features (BREAKING)

 - <csr-id-3fe9d7a6624fe5503f80395f6ed11426b131d3b1/> move Url to sn_api

## v0.41.4 (2021-11-25)

<csr-id-8b8a3616673405005d77868dc397bd7542ab3ea7/>

### Chore

 - <csr-id-8b8a3616673405005d77868dc397bd7542ab3ea7/> safe_network-0.41.4/sn_api-0.42.0

### New Features

 - <csr-id-a8c0e645d9bf11834557049045cca95f8d715a77/> deduce expected_prefix from expected_age
 - <csr-id-b96034796d479352340caaf3d6e6b6d7e6e425ad/> adapt registers to hold vec u8 instead of Url in sn

## v0.41.3 (2021-11-24)

<csr-id-4b72bfc9a6c3a0db4821e7ebf1f4b5daa7cc56d1/>
<csr-id-df25e4920c570771f6813ca03da02f6dfc8e59fb/>
<csr-id-9d657249f3d0391e7c3bbeae6f81909993802cc7/>

### Chore

 - <csr-id-4b72bfc9a6c3a0db4821e7ebf1f4b5daa7cc56d1/> safe_network-0.41.3
 - <csr-id-df25e4920c570771f6813ca03da02f6dfc8e59fb/> sn_api-0.41.0
 - <csr-id-9d657249f3d0391e7c3bbeae6f81909993802cc7/> add comment about vec size

### New Features

 - <csr-id-b96034796d479352340caaf3d6e6b6d7e6e425ad/> adapt registers to hold vec u8 instead of Url in sn

## v0.41.2 (2021-11-24)

<csr-id-bf0488d239fc52ce03c1f380ae0986810d753007/>
<csr-id-aca0fb7a451ffc25c6e34479cc9201fef42796be/>
<csr-id-c34c28dd8776196a6c1c475b7f3ec3be709c0b6d/>
<csr-id-a973039178af33b859d421cf36571de49cceff17/>

### Refactor

 - <csr-id-bf0488d239fc52ce03c1f380ae0986810d753007/> simplify `Proposal::as_signable_bytes`
   The `SignableView` struct isn't used anywhere else, and although it's
   annoying having to write `bincode::serialize` a bunch of times, it is
   simpler with less indirection.
 - <csr-id-aca0fb7a451ffc25c6e34479cc9201fef42796be/> replace `ProposalUtils` with inherent impl
   Now that `Proposal` lives outside of `messaging`, there's no point in
   having the functions in a trait.
 - <csr-id-c34c28dd8776196a6c1c475b7f3ec3be709c0b6d/> duplicate `Proposal` in `routing`
   This simplifies a bunch of conversion to/from the `messaging` and
   `routing` variants of `NodeState` and `SectionAuthorityProvider`.
   Additionally, this lets us reuse more connections when we handle a
   proposal ourselves, such as when responding to a `JoinRequest`.
   
   This had a surprisingly high impact on connection use, shaving off 500
   or so when forming an 11 node network.

### Chore

 - <csr-id-a973039178af33b859d421cf36571de49cceff17/> safe_network-0.41.2/sn_api-0.40.1

### New Features

 - <csr-id-b96034796d479352340caaf3d6e6b6d7e6e425ad/> adapt registers to hold vec u8 instead of Url in sn

## v0.41.1 (2021-11-23)

<csr-id-b456f2f610ea57e5d8a4811fbca5a26175434645/>
<csr-id-950a4eece1a11f70b2aef71cc11404603bb2ec5c/>
<csr-id-f73364efa66718b92e04f24d4546c1e248198ce8/>
<csr-id-ad1617f96954a810898484e5b00b5b8b12495f4e/>
<csr-id-d8ec5a81ae566e8d7068592e01cff4e808b1cad1/>
<csr-id-62aa668d5777058ae617f8952cfcb62be002abf3/>
<csr-id-63432eb2e528401ae67da8eea0c82837ab42fc18/>
<csr-id-7a875a88c911b5a9db59a55b81c94b995e2b95ae/>
<csr-id-5514d9c06ee400e983a258ba2eb37bff1abf0dd0/>
<csr-id-1848189aa780f2f6aabddb3564b816c72f9bee6e/>

### Refactor

 - <csr-id-b456f2f610ea57e5d8a4811fbca5a26175434645/> renaming some variables and functions for better readability
 - <csr-id-950a4eece1a11f70b2aef71cc11404603bb2ec5c/> reuse connections during join flow
   This only saves a few connections, but it proves out the machinery for
   carrying them around. Once additional mechanism was added,
 - <csr-id-f73364efa66718b92e04f24d4546c1e248198ce8/> replace `Sender` with `UnknownPeer`
   The `UnknownPeer` struct is more general, and will make it easier to
   propagate connections into more places (e.g. this already propagates
   connections into `HandleSystemMsg`). It's also generally easier to use
   than `Sender`, and doesn't need any special casing for test-only
   purposes.
   
   The downside is that we lose the 'sending to ourself' representation.
   This wasn't being used so far though, so it's only a hypothetical. The
   comment on `UnknownPeer` attempts to document that 'rough edge' (since
   we would otherwise expect to only know peers either through
   advertisement, where the name would be known, or through direct
   connection, where the connection would be present).

### Other

 - <csr-id-ad1617f96954a810898484e5b00b5b8b12495f4e/> revert "chore(release):"
   This reverts commit d794bc6862692004432699b8deae1a52a1ae1207.
   
   This release was a mistake and modified the changelog in an incorrect way.
 - <csr-id-d8ec5a81ae566e8d7068592e01cff4e808b1cad1/> revert "chore(release): safe_network-0.42.0/sn_api-0.41.0"
   This reverts commit 63432eb2e528401ae67da8eea0c82837ab42fc18.
   
   This release was duplicating everything that was in 0.41.0, probably because the tags weren't
   correct.

### Chore

 - <csr-id-62aa668d5777058ae617f8952cfcb62be002abf3/> safe_network-0.41.1
 - <csr-id-63432eb2e528401ae67da8eea0c82837ab42fc18/> safe_network-0.42.0/sn_api-0.41.0
 - <csr-id-7a875a88c911b5a9db59a55b81c94b995e2b95ae/> add 'connectedness' to `Peer`'s `Display`
 - <csr-id-5514d9c06ee400e983a258ba2eb37bff1abf0dd0/> inherit span on `Comm::send` incoming message listener
   Since we spawn a detached future, we need to explicitly inherit the
   span.
 - <csr-id-1848189aa780f2f6aabddb3564b816c72f9bee6e/> add a feature for increased `WireMsg` debug info
   When we serialize to a `WireMsg`, we lose visibility of the exact
   contents of the message. Serialization is necessary in order to obtain a
   signature for the message, so we can't simply hold the message
   unserialised until send time (at least, not without passing around a
   keypair, which we don't want to do).
   
   As a hacky alternative, an `unstable-wiremsg-debuginfo` feature has been
   added. When enabled, this adds a `payload_debug` field to `WireMsg`
   which can be set to contain a debug representation of the message
   contents. Since `WireMsg` is constructed with `Bytes`, its necessary for
   callers to set this themselves.

## v0.41.0 (2021-11-23)

<csr-id-b1297f2e42cdd7f7945c9fd5b4012086f8298b85/>
<csr-id-680f9431bc75efea455dd7c6985e41c11740dc3e/>
<csr-id-eb0291082e49d1114c853b214f55b8a25e18d1e1/>
<csr-id-93bffbf1343621a8d5187649d1bb6d2a81cba793/>
<csr-id-1268fb6e46e4ba061ba7f724917371d8e0db0092/>
<csr-id-b70a1053adc267780215636dd80a759c45d533d5/>
<csr-id-1d62fcefd5d44ef0df84b8126ba88de1127bd2bd/>
<csr-id-20c1d7bf22907a44be2cc9585dd2ac55dd2985bf/>
<csr-id-182c4db4531c77fa6c67b8267cd7895b49f26ae1/>
<csr-id-02f1325c18779ff02e6bb7d903fda4519ca87231/>
<csr-id-1bf17662ecae4e1f3e9969ac03a8f543f57f2cd0/>
<csr-id-90f2979a5bcb7f8e1786b5bdd868793d2fe924b4/>
<csr-id-14328b3d5b2579e4f038624c2353ec36c45fa9ed/>
<csr-id-4f5e84da96d7277dfc4e385ff03edf6c1d84991e/>
<csr-id-14fdaa6537619483e94424ead5751d5ab41c8a01/>
<csr-id-dbdda1392325e8a08a6a28988208769552d5e543/>
<csr-id-e8e44fb69dae7fe1cae3bbf39f7b03a90df44dd7/>
<csr-id-86c83bedf061b285b0d4f3effb54cc89a9fbd84d/>
<csr-id-590bfea538bb78fd20bb9574c3ea9ed6dcceced0/>
<csr-id-85f2b5fd2adbf1f57e184c9e687fd14929b911e7/>
<csr-id-14c6ab95da62a144a798665ade6da626fb5ee6f2/>
<csr-id-30d220c180c7586de56282fafe52b54c8b619860/>
<csr-id-6c0209187797bb27fa1529dd2f0ee8116975a02d/>
<csr-id-a328d70720a795b62ac05a68454abf1034e97b9d/>
<csr-id-a98cc810409a4ccc78e3c7a36b38b5f00cfcb23a/>
<csr-id-da1fe8dda6325e05b4d976d2b77d084c06d98cb0/>
<csr-id-078f45148af2da8d1690490720ef5f95555d42cd/>
<csr-id-4c9b20ee58be9ca3579c7d1f4edd09452d59d38c/>
<csr-id-49d3a298be19a46dcb6232ee8087c9bf0381db23/>
<csr-id-cdc31cb44af8b6dc2393a0f15c98c0be364db6ff/>
<csr-id-bc717073fde5fef59cbbe670ab6b7a9d66253d3b/>
<csr-id-189aeb6941d277f94e47ade0e851696206aaa146/>
<csr-id-9d5baf83c630e597407e5c5d4a47fc7f34805696/>
<csr-id-9aaccb8784afd03454cd69f599dc2b7822ca130c/>
<csr-id-8f1cc32b87a9e333f32399020339dd41e2d0b049/>
<csr-id-1fbc9b24d14460c6a88a064297c3a327ca9182aa/>
<csr-id-4e7da0aeddba7287c1556ca318d9cdd8459000d2/>
<csr-id-456c022ddf66af5f991e6aca2240d2ab73f8241d/>
<csr-id-ccb7efda1ee0c6303ac537ca9edd4b6c5cfcc5f2/>
<csr-id-d55978b38c9f69c0e31b335909ca650758c522c1/>
<csr-id-9a149113d6ffa4d4e5564ad8ba274117a4522685/>
<csr-id-f8de28a8e040439fcf83f217261facb934cacd51/>
<csr-id-231bde07064d6597867b5668a1e5ff319de3a202/>
<csr-id-3b7091bd8b45fc5bbfa6f1714ec56e123d76c647/>
<csr-id-3b4e2e33285f4b1a53bd52b147dc85c570ed2f6e/>
<csr-id-345033abe74a86c212e3b38fc0f6b655216b63b0/>
<csr-id-4a1349c5e99fdc45c038afdd2dac24486ee625ea/>
<csr-id-d6732833db0bb13d8414f99758e8767ab8ab2bdd/>
<csr-id-d75e5c65d04c60dca477cca4e704308198cfd6cf/>

### Refactor

 - <csr-id-b1297f2e42cdd7f7945c9fd5b4012086f8298b85/> store `Peer`s in `SectionAuthorityProvider`
   `SectionAuthorityProvider` contains details of the elders of a section,
   and is often used as the source of recipients when sending messages to
   elders. By storing a `Peer` inside the SAP, we ensure that connections
   will be reused if possible when messaging elders.
   
   Note that we wholesale replace the SAP in a few cases, which will
   immediately cause us to drop connections. In future we can be more
   careful to retain or selectively update existing `Peer`s to get more
   reuse.
 - <csr-id-680f9431bc75efea455dd7c6985e41c11740dc3e/> reuse known `Peer`s in DKG sessions
   Now that `NodeState` contains a `Peer`, which itself may contain an
   existing `Connection`, it makes sense to try and reuse the `Peer`s in
   network knowledge when initialising DKG sessions.
   
   This doesn't have much impact on the total number of connections yet,
   probably because we don't always maintain the `Peer`s in network
   knowledge.
 - <csr-id-eb0291082e49d1114c853b214f55b8a25e18d1e1/> clarify `connection` in `Peer`'s `Debug` output
   This makes it easier to see in logs whether a `Peer` was connected or
   not, which is useful for identifying where connections are not being
   maintained.
 - <csr-id-93bffbf1343621a8d5187649d1bb6d2a81cba793/> store a `Peer` in `NodeState`
   Since `Peer` can now hold a `connection`, we'd like to get `Peer`
   instances into as many places as possible to enable connection reuse.
   `NodeState` records are held by `SectionPeers`, which contains all known
   members of a node's section. If we can get a connection in there, we
   should be able to reuse it for most node<->node communication.
   
   The change was largely mechanical. The only interesting thing is that
 - <csr-id-1268fb6e46e4ba061ba7f724917371d8e0db0092/> set `Peer::connection` on send
   This changes the `Peer::connection` field to an `Arc<RwLock<...>>` so
   that `routing::Comm` can set the connection if one is opened. This
   shaves off ~ 4k `ConnectionOpened` log markers when forming an 11 node
   network.
   
   The main concern with this approach is that any existing connection is
   simply overwrote, and so will be disassociated from the `Peer`. This
   should currently be fine, since we do not rely on messages coming from
   particular connections.
 - <csr-id-b70a1053adc267780215636dd80a759c45d533d5/> correlate client connections without `ConnectedPeers`
   This is a first pass at introducing a mechanism to correlate client
   connections without using the `ConnectedPeers` cache/pool. The approach
   is centred on passing along the connected `Peer` that originated the
   request, which in most cases is enough to get the response to reply
   directly to the same `Peer` instance.
   
   Chunk queries need an additional mechanism, since the elder cannot reply
   right away, and instead must forward the query to nodes, and can only
   reply to the client once the nodes have themselves replied. For this we
   introduce a `pending_chunk_queries` field to `routing::Core`. When a
   chunk query is received from a client, a random `XorName` is generated
   as a correlation ID, and the client `Peer` is stored in the
   `pending_chunk_queries` cache under that `XorName`. The `XorName` is
   then passed as the 'end user' for the message to nodes, who send it back
   in their reply, allowing the receiving elder to correlate the response
   with the origin client `Peer`, and forward the response to the client
   over the `Peer`'s connection.
   
   There are some minor behavioural changes with this approach:
   
   1. Unlike the connection pooling approach, the client's exact connection
      must remain valid (e.g. if they reconnect while waiting for a
      response, they will never receive it). There is no way around this
      without a general-purpose connection cache, since we do not know on
      connection whether a peer has ever sent a client request. Thankfully
      it's not a big deal, since client operations are always idempotent,
      so client logic can handle any connection interruption by sending the
      same request(s) over a new connection.
   
   2. The `pending_chunk_queries` cache uses duration-based expiration,
      currently hard-coded to 5 minutes. Previously there was no timeout
      for this operation, and it would just depend on the connection
      remaining open in the pool. Since we cannot easily detect connection
      closure in `routing::Core`, and 5 mins is longer than the default
      client query timeout, this seems reasonable for now.
 - <csr-id-1d62fcefd5d44ef0df84b8126ba88de1127bd2bd/> use `qp2p::Connection` for connected senders
   This gets us closer to being able to reuse incoming connections, without
   `ConnectedPeers` (though there's no functional change yet).
   
   Sadly, a `cfg(test)` variant was necessary to satisfy routing tests,
   which don't make actual connections but do make assertions on the sender
   address.
 - <csr-id-20c1d7bf22907a44be2cc9585dd2ac55dd2985bf/> represent `HandleMessage::sender` as an enum
   This is a precursor to having `HandleMessage::sender` contain the
   originating `Connection` for connected peers. Since we use
   `HandleMessage` also when 'sending' messages to ourselves, we have to
   deal with the possibility that there may not be a connection. We could
   simply use `Option` for this, however the dedicated enum makes it
   clearer exactly what situation is represented.
   
   To facilitate later plumbing, `ConnectionEvent` now also uses `Sender`
   to represent the message sender.
   
   It's possible that we should make more use of the knowledge that a
   message was sent to ourselves, e.g. to short-circuit AE checks. For now,
   we more-or-less immediately lookup our own address, and proceed as if
   the message was actually sent from there.
 - <csr-id-182c4db4531c77fa6c67b8267cd7895b49f26ae1/> add an `Option<qp2p::Connection>` to `Peer`
   This allows the `Peer` struct to carry an existing connection to the
   peer, if one exists. Currently, the connection must be set on
   construction with `Peer::connected`. A reference to the connection can
   be retrieved with `Peer::connection`. When sending a message using
 - <csr-id-02f1325c18779ff02e6bb7d903fda4519ca87231/> add a log marker for reusing a connection
   This is mostly informational, but it gives a good indication that
   connection reuse is occurring, and how many new connection attempts have
   been saved.
 - <csr-id-1bf17662ecae4e1f3e9969ac03a8f543f57f2cd0/> add a feature to disable connection pooling
   The feature is named with `unstable-` to indicate that it may be removed
   at any time. This allows us to better test changes to connection
   management, since we know that all reuse must be coming from connection
   management rather than pooling.
 - <csr-id-90f2979a5bcb7f8e1786b5bdd868793d2fe924b4/> use a DAG to keep track of all sections chains within our NetworkKnowledge

### Other

 - <csr-id-14328b3d5b2579e4f038624c2353ec36c45fa9ed/> set node count to be 33
 - <csr-id-4f5e84da96d7277dfc4e385ff03edf6c1d84991e/> increase timeout for network forming

### Chore

 - <csr-id-14fdaa6537619483e94424ead5751d5ab41c8a01/> safe_network-0.41.0
 - <csr-id-dbdda1392325e8a08a6a28988208769552d5e543/> Remove always sending a message to ourself when we can handle it directly
 - <csr-id-e8e44fb69dae7fe1cae3bbf39f7b03a90df44dd7/> move ELDER_COUNT to root of sn
 - <csr-id-86c83bedf061b285b0d4f3effb54cc89a9fbd84d/> depend on ELDER_SIZE where it should be used
   Changes certain instances of  ">= 7" sort of thing, when we actually want to use ELDER_SIZE
 - <csr-id-590bfea538bb78fd20bb9574c3ea9ed6dcceced0/> increase join timeout when always-joinable set
 - <csr-id-85f2b5fd2adbf1f57e184c9e687fd14929b911e7/> increase join timeout before waiting to retry at bin/node
 - <csr-id-14c6ab95da62a144a798665ade6da626fb5ee6f2/> bye bye so much olde node code
 - <csr-id-30d220c180c7586de56282fafe52b54c8b619860/> readd routing member joined event for tests
 - <csr-id-6c0209187797bb27fa1529dd2f0ee8116975a02d/> log node age on relocation in routing
 - <csr-id-a328d70720a795b62ac05a68454abf1034e97b9d/> set joins allowed in routing code
 - <csr-id-a98cc810409a4ccc78e3c7a36b38b5f00cfcb23a/> burn down more of the /node dir
 - <csr-id-da1fe8dda6325e05b4d976d2b77d084c06d98cb0/> add StillElderAfterSplit LogMarker
   This makes it easier to distinguish nodes that continued being an aleder after split
 - <csr-id-078f45148af2da8d1690490720ef5f95555d42cd/> remove AdultsChanged event and ahndle everything inside routing
 - <csr-id-4c9b20ee58be9ca3579c7d1f4edd09452d59d38c/> update client listener error handling
 - <csr-id-49d3a298be19a46dcb6232ee8087c9bf0381db23/> cleanup test clippy
 - <csr-id-cdc31cb44af8b6dc2393a0f15c98c0be364db6ff/> dont always exit on client listener loop errors
 - <csr-id-bc717073fde5fef59cbbe670ab6b7a9d66253d3b/> fail query if we cannot send to more than 1 elder
 - <csr-id-189aeb6941d277f94e47ade0e851696206aaa146/> fix routing tests failing after we no longer emit MemerLeft event
 - <csr-id-9d5baf83c630e597407e5c5d4a47fc7f34805696/> simplify msg handling for NodeCmds
 - <csr-id-9aaccb8784afd03454cd69f599dc2b7822ca130c/> more node plumbing cleanup
 - <csr-id-8f1cc32b87a9e333f32399020339dd41e2d0b049/> handle set storage level in routing directly
 - <csr-id-1fbc9b24d14460c6a88a064297c3a327ca9182aa/> handle member left directly in routing code
   Remove related /node plumbing
 - <csr-id-4e7da0aeddba7287c1556ca318d9cdd8459000d2/> handle receiving dataExchange packet in routing.
   Removes more flows from the node code
 - <csr-id-456c022ddf66af5f991e6aca2240d2ab73f8241d/> remove node plumbing
 - <csr-id-ccb7efda1ee0c6303ac537ca9edd4b6c5cfcc5f2/> perform data exchange on split in routing
 - <csr-id-d55978b38c9f69c0e31b335909ca650758c522c1/> on split retain adult members only
 - <csr-id-9a149113d6ffa4d4e5564ad8ba274117a4522685/> fix demotion sendmessage test
 - <csr-id-f8de28a8e040439fcf83f217261facb934cacd51/> only fire SplitSuccess if elder
 - <csr-id-231bde07064d6597867b5668a1e5ff319de3a202/> send data updates to new elders only
 - <csr-id-3b7091bd8b45fc5bbfa6f1714ec56e123d76c647/> dont AE-update sibling elders on split.
   They cannot do anything with this info until they have their secret key, and with that they should get the SAP anyway. Or another AE flow can correct as needed
 - <csr-id-3b4e2e33285f4b1a53bd52b147dc85c570ed2f6e/> remove node plumbing
 - <csr-id-345033abe74a86c212e3b38fc0f6b655216b63b0/> perform data exchange on split in routing
 - <csr-id-4a1349c5e99fdc45c038afdd2dac24486ee625ea/> stop sending unnecessary AE-Update msgs to to-be-promoted candidates
 - <csr-id-d6732833db0bb13d8414f99758e8767ab8ab2bdd/> log an error when failing to add latest sibling key to proof chain
 - <csr-id-d75e5c65d04c60dca477cca4e704308198cfd6cf/> update launch tool and secured linked list deps

### New Features

 - <csr-id-f29c94d7dfab2e82b9db70fcfeddc4a71d987abb/> use AE to progress various intra-DKG phases
   sometimes a DKG session might receive DKG messages from a further phase
   that it has not reached yet. eg. it might receive Proposal messages
   while in the Initialization phase. When this happens, the DKG can
   respond with a DkgNotReady message to the message source which will sent
   a list of 'DKG Messages' that can be applied to an existing DKG process
   so it can progress to the next phase.
   
   note that this does not solve the case where the DKG session has not yet
   started. that will need to be handled separately. they are currently
   pushed to the backlog and handled later
 - <csr-id-46ea542731fa3e2cede4ce9357783d3681434643/> allow ELDER_COUNT to be overridden by env var SN_ELDER_COUNT
 - <csr-id-f6a3f78156b9cc0f934c19e5d4c0004238a593e4/> verify and use SAP received in AE-Redirect to update client's network knowledge
 - <csr-id-ae5fc40e29161652306b2e42b92d2a80fc746708/> verify and use SAP received in AE-Redirect to update our network knowledge
 - <csr-id-20bda03e07985114b8a54c866755be8a240c2504/> retry client queires if elder conns fail

### Bug Fixes

<csr-id-d3b88f749ca6ee53f65e200105aeeea691581e83/>
<csr-id-35a8fdb9dca60ca268d536958a9d32ce0a876792/>
<csr-id-c27d7e997c5a7812f995f113f31edf30c2c21272/>
<csr-id-55eb0f259a83faff470dbfdeb9365d314ed6a697/>
<csr-id-42d90b763606e2d324c5ce1235fc801105c07acb/>
<csr-id-302ce4e605d72a0925509bfe3220c2b1ddac677d/>
<csr-id-c78513903457a701096b5c542f15012e71d33c46/>

 - <csr-id-9a82649f0ca01c6d2eae57f260d2f98246724556/> multiples fixes for unit tests
   - error instead of panicking if logger is already initialized

### New Features (BREAKING)

 - <csr-id-3a59ee3b532bbc26388780ddc2f5b51ddae61d4c/> include section chain in AE-Redirect messages

## v0.40.0 (2021-11-15)

<csr-id-1389ffa00762e126047a206abc475599c277930c/>
<csr-id-70015730c3e08881f803e9ce59be7ca16185ae11/>
<csr-id-ee165d41ca40be378423394b6422570d1d47727c/>
<csr-id-00acf0c0d8a65bdd5355ba909d73e74729a27044/>
<csr-id-7d5d5e11fef39a6dc1b89c972e42772db807374c/>
<csr-id-855b8ff87217e92a5f7d55fb78ab73c9d81f75a2/>
<csr-id-23b8a08e97fa415b9216caac5da18cb97ede980f/>

### Refactor

 - <csr-id-1389ffa00762e126047a206abc475599c277930c/> retry on connection loss of reused connection
   The `routing::Comm::send` method uses an instance of `ConnectedPeers` to
   reuse existing connections. This saves a lot of connections during busy
   exchanges between few nodes, such as DKG. However, should the connection
   retrieved for reuse have become invalid (e.g. closed by the peer or
   timed out), that recipient would be recorded as failed.
   
   Since connection timeouts are to be expected, particularly for stored
   connections with keep-alives disabled as in `ConnectedPeers`, it makes
   sense to retry the recipient with a fresh connection before declaring
   it failed.
   
   The implementation has been bolted on to the existing `send` mechanism.
   The `send` closure will return an additional boolean indicating whether
   or not the connection was reused. When evaluating the result, if it
   failed due to connection loss on a reused connection, we crate a new
   `send` task for the same recipient, but pass an additional flag to force
   a new connection. Thus, if the peer fails *again* for connection loss,
   the connection will not have been reused and the peer will be recorded
   as a failed recipient as normal.

### Chore

 - <csr-id-70015730c3e08881f803e9ce59be7ca16185ae11/> safe_network v0.40.0/sn_api v0.39.0

### New Features

 - <csr-id-b8b0097ac86645f0c7a7352f2bf220279068228c/> support `--json-logs` in `testnet`
   This makes it easier to enable JSON logs for local testnets.
 - <csr-id-044fd61950be76e3207694094dcec81313937403/> allow to change the default interval for testnet nodes

### Bug Fixes

 - <csr-id-ad633e1b6882db1aac0cb1a530300d9e4d666fd8/> avoid holding write lock on ae_backoff_cache during sleep
   This could inadvertently be slowing down other joins if we're holding the write lock on the cache while we backoff
 - <csr-id-366ad9a42034fe450c30908d372fede0ff92f655/> draining backlog before process DKG message
 - <csr-id-c4a42fb5965c79aaae2a0ef2ae62fcb987c37525/> less DKG progression interval; switch SAP on DKGcompletion when OurElder already received
 - <csr-id-adf9feac964fb7f690bd43aeef3270c82fde419c/> fix `Eq` for `SectionPeers`
   This would have always returned true, since it was comparing with
   itself...

### New Features (BREAKING)

 - <csr-id-9b8ddfde0287e47b6f18a77a8e8847d80ee84bcd/> provide the SAP proof chain in JoinResponse::Retry msgs
   - Joining node now makes use of the NetworkPrefixMap to validate and
   accept new SAPs using the proof chain provided in JoinResponse::Retry.

### Refactor (BREAKING)

 - <csr-id-ee165d41ca40be378423394b6422570d1d47727c/> duplicate `SectionAuthorityProvider` in `routing`
   This is part of the move towards passing connection handles through
   network knowledge. Whilst it would be possible to put a connection
   handle in the `messaging` struct, and just skip it when (de)serialising,
   this seems less clean than having a separate struct.
 - <csr-id-00acf0c0d8a65bdd5355ba909d73e74729a27044/> move `SectionPeers` from `messaging` to `routing`
   `SectionPeers` was only used in the `SystemMsg::AntiEntropyUpdate`
   message. Rather than simply inlining the `DashMap`, it's been replaced
   by `BTreeSet<SectionAuth<NodeState>>` since the map keys are redundant
   with the `name` field of `NodeState` (and `BTreeSet` specifically for
   deterministic ordering).
   
   With the move, it's also become `pub(crate)`, so from the perspective of
   the public API this type has been removed.
 - <csr-id-7d5d5e11fef39a6dc1b89c972e42772db807374c/> move `ElderCandidates` to `routing`
   The `ElderCandidate` type was in `messaging`, and was only used in the
   `SystemMsg::DkStart` message. It was more commonly used as network
   state, so homing it in `routing::network_knowledge` makes more sense.
   This also means the type no longer needs to be public, and doesn't need
   to be (de)serialisable, which opens up the possibility of putting more
   stuff in it in future (e.g. connection handles...).
   
   As a first step down that road, the `elders` field now contains `Peer`s
   rather than merely `SocketAddr`s. The majority of reads of the field
   ultimately want either name-only or `Peer`s, so this seems reasonable
   regardless.
 - <csr-id-855b8ff87217e92a5f7d55fb78ab73c9d81f75a2/> make `SectionAuthorityProviderUtils` `pub(crate)`
   This shouldn't really be part of the public API, and having it such
   transitively grows the visibility of its collaborators (e.g.
   `ElderCandidates`) which also don't need to be in the public API.
 - <csr-id-23b8a08e97fa415b9216caac5da18cb97ede980f/> remove `Copy` from `Peer`
   Eventually we may want to put a `Connection` into `Peer`, which is not
   `Copy`. As such, `Peer` itself will be unable to derive `Copy`. Although
   this might not happen for a while, removing dependence on `Copy` now
   will make it smoother when the time comes.

## v0.39.0 (2021-11-12)

<csr-id-0a5027cd9b2c62833ccf70e2bcca5ab22625a840/>
<csr-id-645047b231eaf69f1299dee22ff2079feb7f5a95/>
<csr-id-0e2b01bb7e9c32f0bc5c1c6fd616acfcc5c45020/>
<csr-id-ab00cf08d217654c57449437348b73576a65e89f/>
<csr-id-5a3b70e9721fcdfdd809d2a6bd85968446b4e9a3/>
<csr-id-08629e583a03852d9f02b0e7ff66a829e3056d9b/>
<csr-id-5a81d874cf8040ff81a602bc8c5707a4ba6c64ff/>
<csr-id-8aa27b01f8043e98953971d1623aaf8b07d1596e/>
<csr-id-48a7d0062f259b362eb745e1bd59e77df514f1b8/>
<csr-id-008f36317f440579fe952325ccb97c9b5a547165/>
<csr-id-213cb39be8fbfdf614f3eb6248b14fe161927a14/>
<csr-id-ee165d41ca40be378423394b6422570d1d47727c/>
<csr-id-00acf0c0d8a65bdd5355ba909d73e74729a27044/>
<csr-id-7d5d5e11fef39a6dc1b89c972e42772db807374c/>
<csr-id-855b8ff87217e92a5f7d55fb78ab73c9d81f75a2/>
<csr-id-23b8a08e97fa415b9216caac5da18cb97ede980f/>

### Refactor

 - <csr-id-0a5027cd9b2c62833ccf70e2bcca5ab22625a840/> make `NodeState` not `Copy`
   At some point we might stick a connection handle in there, which would
   not be `Copy`, so unimplementing it now may save some changed lines in
   future.
 - <csr-id-645047b231eaf69f1299dee22ff2079feb7f5a95/> duplicate `NodeState` in `routing`
   This is part of the move towards passing connection handles through
   network knowledge. Whilst it would be possible to put a connection
   handle in the `messaging` struct, and just skip it when (de)serialising,
   this seems less clean than having a separate struct.
 - <csr-id-0e2b01bb7e9c32f0bc5c1c6fd616acfcc5c45020/> move `routing::Peer` into `network_knowledge`
   This seems like a better home for it. It's still re-exported from
   `routing` to minimise disruption.

### Chore

 - <csr-id-ab00cf08d217654c57449437348b73576a65e89f/> safe_network v0.39.0
 - <csr-id-5a3b70e9721fcdfdd809d2a6bd85968446b4e9a3/> remove unnecessary peer lagging check
   - All 'Proposal' messages are sent with current SAP's section key, same key
     as the Proposal's signature share public key, thus earlier AntiEntropy check
     would have caught a Proposal msg signed with an old share and this check
     is never reached.
 - <csr-id-08629e583a03852d9f02b0e7ff66a829e3056d9b/> update testnet default interval
 - <csr-id-5a81d874cf8040ff81a602bc8c5707a4ba6c64ff/> rename forced to skip section info agreement, tidy assignment
 - <csr-id-8aa27b01f8043e98953971d1623aaf8b07d1596e/> update bls_dkg to 0.7.1
 - <csr-id-48a7d0062f259b362eb745e1bd59e77df514f1b8/> stepped age with long interval
 - <csr-id-008f36317f440579fe952325ccb97c9b5a547165/> add more tracing around connections
   This is mostly peppering `.in_current_span` to some spawned futures.
   Additionally the traces for `ConnectionOpened` and `ConnectionClosed`
   now use fields for their context, since it makes JSON processing easier.
 - <csr-id-213cb39be8fbfdf614f3eb6248b14fe161927a14/> update bls_dkg and blsttc to 0.7 and 0.3.4 respectively

### New Features

 - <csr-id-b8b0097ac86645f0c7a7352f2bf220279068228c/> support `--json-logs` in `testnet`
   This makes it easier to enable JSON logs for local testnets.
 - <csr-id-044fd61950be76e3207694094dcec81313937403/> allow to change the default interval for testnet nodes

### Bug Fixes

 - <csr-id-366ad9a42034fe450c30908d372fede0ff92f655/> draining backlog before process DKG message
 - <csr-id-c4a42fb5965c79aaae2a0ef2ae62fcb987c37525/> less DKG progression interval; switch SAP on DKGcompletion when OurElder already received
 - <csr-id-adf9feac964fb7f690bd43aeef3270c82fde419c/> fix `Eq` for `SectionPeers`
   This would have always returned true, since it was comparing with
   itself...

### New Features (BREAKING)

 - <csr-id-9b8ddfde0287e47b6f18a77a8e8847d80ee84bcd/> provide the SAP proof chain in JoinResponse::Retry msgs
   - Joining node now makes use of the NetworkPrefixMap to validate and
   accept new SAPs using the proof chain provided in JoinResponse::Retry.

### Refactor (BREAKING)

 - <csr-id-ee165d41ca40be378423394b6422570d1d47727c/> duplicate `SectionAuthorityProvider` in `routing`
   This is part of the move towards passing connection handles through
   network knowledge. Whilst it would be possible to put a connection
   handle in the `messaging` struct, and just skip it when (de)serialising,
   this seems less clean than having a separate struct.
 - <csr-id-00acf0c0d8a65bdd5355ba909d73e74729a27044/> move `SectionPeers` from `messaging` to `routing`
   `SectionPeers` was only used in the `SystemMsg::AntiEntropyUpdate`
   message. Rather than simply inlining the `DashMap`, it's been replaced
   by `BTreeSet<SectionAuth<NodeState>>` since the map keys are redundant
   with the `name` field of `NodeState` (and `BTreeSet` specifically for
   deterministic ordering).
   
   With the move, it's also become `pub(crate)`, so from the perspective of
   the public API this type has been removed.
 - <csr-id-7d5d5e11fef39a6dc1b89c972e42772db807374c/> move `ElderCandidates` to `routing`
   The `ElderCandidate` type was in `messaging`, and was only used in the
   `SystemMsg::DkStart` message. It was more commonly used as network
   state, so homing it in `routing::network_knowledge` makes more sense.
   This also means the type no longer needs to be public, and doesn't need
   to be (de)serialisable, which opens up the possibility of putting more
   stuff in it in future (e.g. connection handles...).
   
   As a first step down that road, the `elders` field now contains `Peer`s
   rather than merely `SocketAddr`s. The majority of reads of the field
   ultimately want either name-only or `Peer`s, so this seems reasonable
   regardless.
 - <csr-id-855b8ff87217e92a5f7d55fb78ab73c9d81f75a2/> make `SectionAuthorityProviderUtils` `pub(crate)`
   This shouldn't really be part of the public API, and having it such
   transitively grows the visibility of its collaborators (e.g.
   `ElderCandidates`) which also don't need to be in the public API.
 - <csr-id-23b8a08e97fa415b9216caac5da18cb97ede980f/> remove `Copy` from `Peer`
   Eventually we may want to put a `Connection` into `Peer`, which is not
   `Copy`. As such, `Peer` itself will be unable to derive `Copy`. Although
   this might not happen for a while, removing dependence on `Copy` now
   will make it smoother when the time comes.

## v0.38.0 (2021-11-10)

<csr-id-1568adb28a2a6a3cdf8a9737e098a5ea7bb2c419/>
<csr-id-cbdf0236f7cd241e7addb1afe75ed5e5cfac00ab/>
<csr-id-a7968dd17927e346b3d32bb5971ed6457aea6606/>

### Refactor

 - <csr-id-1568adb28a2a6a3cdf8a9737e098a5ea7bb2c419/> insert a connected peer for outgoing connections
   The `ConnectedPeers` struct serves as something of a 'cache' of existing
   connections with peers. It was introduced when connection pooling was
   removed from `qp2p`, in order to preserve connections from clients (who
   cannot be reconnected to). Since we can't discriminate between clients
   and nodes when they connect to us, we ended up inserting *all* incoming
   connections into `ConnectedPeers`. This is relatively harmless, since
   node<->node connections will eventually timeout (no keep-alive) and
   connections are removed from `ConnectedPeers` when they close for any
   reason.
   
   Unlike the `qp2p` connection pool that it replaces, outgoing connections
   were not added to `ConnectedPeers`. This was deliberate, on the basis
   that we would prefer more precise connection management, e.g. by passing
   connection handles around as part of commands/state. Sadly there are a
   few challenges to doing this:
   
   - The code paths are complicated and generally divergent. Ensuring
     connection reuse would likely involve plumbing a connection through a
     significant portion of the codebase.
   - Network state is often held in `messaging` types (e.g.
     `ElderCandidates`, `SectionPeers`, `NodeState`, etc.). Polluting the
     messaging types with connection handles that cannot be (de)serialised
     is undesirable, but keeping them out of state can involve non-trivial
     refactoring.
   - More generally, 'inertia' from having gone for a very long time with
     messaging being entirely address-based. This is closely related to the
     first point – most of the codebase is connection-unaware, and plumbing
     a connection through is slow work and typically only provides reuse
     for a single message flow.
   
   By contrast, inserting outgoing connections into `ConnectedPeers` is
   trivial, and massively cuts down the number of opened connections (from
   ~2.5k down to ~150 when forming an 11 node network, which is still far
   more connections than the minimum of one connection between each node
   and every other, 55, but better than nothing).
   
   It probably still makes sense to aim for tigher connection management,
   but some refactoring groundwork would make this a smoother process, and
   with everything else currently in flight it seems like a lower priority.

### Chore

 - <csr-id-cbdf0236f7cd241e7addb1afe75ed5e5cfac00ab/> safe_network v0.38.0
 - <csr-id-a7968dd17927e346b3d32bb5971ed6457aea6606/> reuse join permits for relocated nodes too.
   Also increases concurrent joins allowed

### New Features

 - <csr-id-044fd61950be76e3207694094dcec81313937403/> allow to change the default interval for testnet nodes

### Bug Fixes

 - <csr-id-adf9feac964fb7f690bd43aeef3270c82fde419c/> fix `Eq` for `SectionPeers`
   This would have always returned true, since it was comparing with
   itself...

### New Features (BREAKING)

 - <csr-id-9b8ddfde0287e47b6f18a77a8e8847d80ee84bcd/> provide the SAP proof chain in JoinResponse::Retry msgs
   - Joining node now makes use of the NetworkPrefixMap to validate and
   accept new SAPs using the proof chain provided in JoinResponse::Retry.

## v0.37.0 (2021-11-09)

<csr-id-1b7f5d6e198eb286034e12eed3fa4fc63b927e0f/>
<csr-id-9581fcf27765752f6b556d632e8396e93d0f15e0/>
<csr-id-a03d2cef55665759ddfeb40972676c87b17ccfa8/>
<csr-id-bde3cd5eac75cc41a3a9ffefb091584273575f68/>
<csr-id-e24763e604e6a25ee1b211af6cf3cdd388ec0978/>
<csr-id-7d5212a2b916a8e540403346e0770cee1a446884/>
<csr-id-109843c67ffd4ac4675cdfe56d7fcdaf97664007/>
<csr-id-6bb5e64b3b0eb518719581278551272ae8f2b2ed/>
<csr-id-e47e9ecb0c7f23209a9c1eb58c248fdf2facfd4a/>
<csr-id-3b728b65a07a33019e06dd6f3da9fd334e6da9e1/>
<csr-id-87c62d39d240afc01118135bc18d22fe23fc421c/>
<csr-id-d7972da6abd3001e75019bf72aae6a98919ed1db/>
<csr-id-49e2d14a1c5f8fd83aa4b9a5abe67e23fca9f966/>
<csr-id-5326de5f658dfe75d1f5c44224d8623123848b08/>
<csr-id-9a3cffc52589e8adf6dac75ae6aab4c184118648/>
<csr-id-c2df9db2fe9e99450cc16060ff034289ab683783/>
<csr-id-2fc2f0e6fbb5f7b05e61281e90992053ef5f0f5d/>
<csr-id-432e36de012695c6c5e20bd704dc184db9c5c4d6/>
<csr-id-9cc16296db7241819e17dd2673c7b3cb9fe2ead8/>
<csr-id-c49f9a16c9e0912bf581a2afef22ac4806898ade/>
<csr-id-9cd2d37dafb95a2765d5c7801a7bb0c58286c47c/>
<csr-id-b719b74abdd1cd84b3813ec7046f4fdf99cde6a2/>
<csr-id-3703819c7f0da220c8ff21169ca1e8161a20157b/>
<csr-id-d0134e870bb097e095e1c8a33e607cf7994e6491/>
<csr-id-b66444c7d7a10c9a26db30fe5ce8e91984031624/>
<csr-id-c5aae59a4eac6c3285bc45845433d23fc96d155f/>
<csr-id-723618d0e3ad77eb26a3c698f9545490166b7ee0/>
<csr-id-80ce4ca9376b99037b9377425c477ea3a7493e54/>
<csr-id-616c024f2280fc99de063026756a9c938f78b885/>
<csr-id-cf5a041742844b8b3d5e71c3e895018367b76013/>
<csr-id-ed6d2f09c12b72446f0f55ecacb5fc4f35278575/>
<csr-id-9350f273a5afec88120fe79fe85ceaf8027d691e/>
<csr-id-a54c4426caccf71d28b2697094135892ec4a5e16/>
<csr-id-8eedb660461b13b236545643fa21868eb6613826/>
<csr-id-abfe7378604a74119accd7b9f86bef5682b0784a/>
<csr-id-0399a9e76547a03f1e3902aec30ecbd57ed437c7/>
<csr-id-23f53a339a5fe0d2f2e5d415bfc01e646a81a5c8/>
<csr-id-5232bf51c738197339ac70ee4f46adee4fa87179/>
<csr-id-f9cd39b6d3b1877fa0f3fa73bd6e3a796cae3b08/>
<csr-id-208abdd108654ab83e8bee763793201e6f7b5eb2/>
<csr-id-fc10d037d64efc86796f1b1c6f255a4c7f91d3e1/>
<csr-id-12c673e7f52ed1c4cbe15c14fe6eb4e68b986e18/>
<csr-id-9d27749bc7d59ee499980044f57eab86d2e63d04/>
<csr-id-aaa0af4d1685c65a3c166070a590c10e9fd54765/>
<csr-id-73b405d1228244a2b984e1294d5e8542f8691cef/>
<csr-id-0c9c1f2edd9e872b9ba1642ac50f59a63f68b488/>
<csr-id-9bdd68f313b3b7881cb39db02026184bbab0bfb0/>
<csr-id-8b7b66450349b669e023539e92c77f9a3b830948/>
<csr-id-7fc29f8187a29c2eeff8bbb5e09f068414bb8b93/>
<csr-id-4415c9b1d166f7e53032a0100d829e8581255a1e/>
<csr-id-8eb1877effb8cf0bfc8986c23d49d727500087dd/>
<csr-id-0b8413998f018d7d577f2248e36e21f6c2744116/>
<csr-id-aaa6903e612178ce59481b2e81fe3bd0d1cc2617/>
<csr-id-2651423c61b160841557d279c9b706abdaab4cdf/>
<csr-id-20461a84dbc0fc373b184a3982a79affad0544f6/>
<csr-id-6a8f5a1f41e5a0f4c0cce6914d4b330b68f5e5d8/>
<csr-id-da70738ff0e24827b749a970f466f3983b70442c/>
<csr-id-7ddf2f850441e6596133ab7a596eb766380008c3/>
<csr-id-0387123114ff6ae42920577706497319c8a888cb/>
<csr-id-aa17309092211e8d1ba36d4aaf50c4677a461594/>
<csr-id-225432908839359800d301d9e5aa8274e4652ee1/>
<csr-id-7b430a54f50846a8475cec804bc24552043558b7/>
<csr-id-fe62ad9f590f151dd20a6832dfab81d34fc9c020/>
<csr-id-9e3cb4e7e4dc3d1b8fa23455151de60d5ea03d4d/>
<csr-id-37aec09a555d428c730cdd982d06cf5cb58b60b1/>
<csr-id-810fe0797a4f23a6bb2586d901ccac9272a9beb2/>
<csr-id-ca074b92b31add1ad6d0db50f2ba3b3d1ae25d5a/>
<csr-id-993d564f0ad5d8cac8d9a32b8c6c8d1dd00c3fd9/>
<csr-id-fd8e8f3fb22f8592a45db33e75f237ef22cd1f5b/>
<csr-id-c1c2bcac021dda4bf18a4d80ad2f86370d56efa7/>
<csr-id-e2727b91c836619dadf2e464ee7c57b338427f22/>
<csr-id-148140f1d932e6c4b30122ebcca3450ab6c84544/>
<csr-id-61dec0fd90b4df6b0695a7ba46da86999d199d4a/>
<csr-id-49c76d13a91474038bd8cb005959a37a7d4c6603/>
<csr-id-c5d2e31a5f0cea381bb60dc1f896dbbda5038506/>
<csr-id-0cb790f5c3712e357f685bfb88cd237c5b5f76c5/>
<csr-id-b3ce84012e6cdf4c87d6d4a3137ab6506264e949/>
<csr-id-ecf0bc9a9736167edb15db7ff4e3cf5dc388dd22/>

### Test

 - <csr-id-1b7f5d6e198eb286034e12eed3fa4fc63b927e0f/> fix healthcheck 0/1 summation.
 - <csr-id-9581fcf27765752f6b556d632e8396e93d0f15e0/> adapt routing unit tests to new network knowledge handling logic
 - <csr-id-a03d2cef55665759ddfeb40972676c87b17ccfa8/> update elder count check to be general one split health check
 - <csr-id-bde3cd5eac75cc41a3a9ffefb091584273575f68/> ignore demotion test for now
 - <csr-id-e24763e604e6a25ee1b211af6cf3cdd388ec0978/> add network elder count test, using testnet grep

### Refactor

 - <csr-id-7d5212a2b916a8e540403346e0770cee1a446884/> use the prefix map updating outcome to decide if current SAP and chain shall be updated
 - <csr-id-109843c67ffd4ac4675cdfe56d7fcdaf97664007/> drop JoinRetry responses for SAPs we've already resent join reqs to
 - <csr-id-6bb5e64b3b0eb518719581278551272ae8f2b2ed/> thread a `Peer` through more places
   Rather than using separate `XorName` and `SocketAddr` arguments/fields,
   we can use `Peer`. This doesn't buy us much currently, but in future we
   could store an open `Connection` to the peer inside the struct, letting
   us reuse connections to reply to incoming messages etc.
 - <csr-id-e47e9ecb0c7f23209a9c1eb58c248fdf2facfd4a/> simplify some iterator chains
 - <csr-id-3b728b65a07a33019e06dd6f3da9fd334e6da9e1/> use `peers()` instead of `Peer::new` when available
   This was only useful in a couple of places, but it makes sense to use
   the `peers` method when it exists.
 - <csr-id-87c62d39d240afc01118135bc18d22fe23fc421c/> replace `(XorName, SocketAddr)` with `Peer`
   In many places, we pass around `Peer`s which were then converted to
   `(XorName, SocketAddr)`. Most of the time this ends up at the
 - <csr-id-d7972da6abd3001e75019bf72aae6a98919ed1db/> storing all sections chains in routing network knowledge
 - <csr-id-49e2d14a1c5f8fd83aa4b9a5abe67e23fca9f966/> refactoring NetworkKnowledge private API
 - <csr-id-5326de5f658dfe75d1f5c44224d8623123848b08/> moving to a unified network knowledge for updating SAPs and section chain
 - <csr-id-9a3cffc52589e8adf6dac75ae6aab4c184118648/> make clients read prefix_map from disk
 - <csr-id-c2df9db2fe9e99450cc16060ff034289ab683783/> add `KvStore::flush` to avoid waiting in tests
   Some `KvStore` tests verify that used space adjusts appropriately when
   data is written (also when data is removed, but those tests are disabled
   since it's not predictable when sled will compact its log). We measure
   used space by measuring the size of the database files.
   
   sled flushes to disk asynchronously by default, and the tests were
   accounting for this by looping until the size changed. This could lead
   to hung tests in the case of bugs with used space calculation, which
   would be harder to debug than a concrete failure. As such, a test-only
   `flush` method has been added to `KvStore`, which is now used by the
   tests to force a flush to disk.
 - <csr-id-2fc2f0e6fbb5f7b05e61281e90992053ef5f0f5d/> remove redundant `K` type parameter from `KvStore`
   The `Value` trait already included an associated type for the key type,
   and this can be referenced directly in signatures without the need for
   an additional generic type parameter.
 - <csr-id-432e36de012695c6c5e20bd704dc184db9c5c4d6/> update `qp2p`, which removes `ConnectionPool`
   The latest release of `qp2p` has removed the `ConnectionPool`, meaning
   connections now close immediately when both sides of it (`Connection`
   and `ConnectionIncoming`). It's intended that we apply more deliberate
   connection management as a substitute, but as a stop-gap this commit
   implements a `ConnectedPeers` type, which behaves much like the `qp2p`
   `ConnectionPool`.
 - <csr-id-9cc16296db7241819e17dd2673c7b3cb9fe2ead8/> moving Proposal utilities into routing:Core
 - <csr-id-c49f9a16c9e0912bf581a2afef22ac4806898ade/> simplifying key shares cache
 - <csr-id-9cd2d37dafb95a2765d5c7801a7bb0c58286c47c/> encapsulate Section info to reduce SAP and chain cloning
 - <csr-id-b719b74abdd1cd84b3813ec7046f4fdf99cde6a2/> moving Section definition our of messaging onto routing

### Other

 - <csr-id-3703819c7f0da220c8ff21169ca1e8161a20157b/> update actions workflows for workspace refactor
   Now that we have multiple crates in the same repository, if we only want to do things when subsets
   of the source tree have been modified (e.g. only one crate particular crate has changed), it does
   make the CI situation a bit more cumbersome. We need to make much more frequent use the path filter
   action and only do things if certain files have been changed.
   
   It's also useful for the workflows to indicate the crate they relate to, so they've been renamed or
   added to accommodate that.
   
   Certain things have been moved to the level of the workspace because with these, you can't really
   discriminate based on the crate that's changed.
   
   An unused dependency for sn_api was also taken out here.
 - <csr-id-d0134e870bb097e095e1c8a33e607cf7994e6491/> update actions workflows for workspace refactor
   Now that we have multiple crates in the same repository, if we only want to do things when subsets
   of the source tree have been modified (e.g. only one crate particular crate has changed), it does
   make the CI situation a bit more cumbersome. We need to make much more frequent use the path filter
   action and only do things if certain files have been changed.
   
   It's also useful for the workflows to indicate the crate they relate to, so they've been renamed or
   added to accommodate that.
   
   Certain things have been moved to the level of the workspace because with these, you can't really
   discriminate based on the crate that's changed.
   
   An unused dependency for sn_api was also taken out here.

### Chore

 - <csr-id-b66444c7d7a10c9a26db30fe5ce8e91984031624/> some variables renaming and improving some comments
 - <csr-id-c5aae59a4eac6c3285bc45845433d23fc96d155f/> avoid rebuild on windows for testnet
   Windows OS has an issue of re-building as unable to remove testnet.exe.
 - <csr-id-723618d0e3ad77eb26a3c698f9545490166b7ee0/> add more log markers.
   For adding SKs to our cache as an elder
   For handlingElderAgreement
 - <csr-id-80ce4ca9376b99037b9377425c477ea3a7493e54/> update log messages and test errors for clarity
 - <csr-id-616c024f2280fc99de063026756a9c938f78b885/> remove blocking/non blocking msg handling distinction
 - <csr-id-cf5a041742844b8b3d5e71c3e895018367b76013/> clarify elder counts in health check test
 - <csr-id-ed6d2f09c12b72446f0f55ecacb5fc4f35278575/> reduce join backoff times
 - <csr-id-9350f273a5afec88120fe79fe85ceaf8027d691e/> change client timeouts
 - <csr-id-a54c4426caccf71d28b2697094135892ec4a5e16/> reorder blob tst log initialisation to _after_ initial AE triggering msgs sent
 - <csr-id-8eedb660461b13b236545643fa21868eb6613826/> update health check to test for Prefix(1/0), use this in CI
 - <csr-id-abfe7378604a74119accd7b9f86bef5682b0784a/> add env var for client query timeout
   rename ClientConfig
 - <csr-id-0399a9e76547a03f1e3902aec30ecbd57ed437c7/> reduce clone and lock during message handling
 - <csr-id-23f53a339a5fe0d2f2e5d415bfc01e646a81a5c8/> implement `Deref` for `SectionAuth`
   This lets us drop a lot of explicit `value` accessed in favour of
   automatic dereferencing. This may make future changes lower impact.
 - <csr-id-5232bf51c738197339ac70ee4f46adee4fa87179/> renaming section mod to network_knowledge
 - <csr-id-f9cd39b6d3b1877fa0f3fa73bd6e3a796cae3b08/> Section->NetworkKnowledge name change
 - <csr-id-208abdd108654ab83e8bee763793201e6f7b5eb2/> reduce the report check interval
 - <csr-id-fc10d037d64efc86796f1b1c6f255a4c7f91d3e1/> bump rust edition
   The few breaking changes in this edition did not affect us.
 - <csr-id-12c673e7f52ed1c4cbe15c14fe6eb4e68b986e18/> remove unnecessary `allow(unused)`
 - <csr-id-9d27749bc7d59ee499980044f57eab86d2e63d04/> remove unused `KvStore::store_batch`
   Although this was commented to indicate it will be used again soon, it
   is trivial to recover code from git history, but it may be non-trivial
   to maintain or fix unused code. That's the case for trying to fix used
   space handling in `store_batch`, since `sled::Tree::apply_batch` cannot
   perform compare-and-swap operations. As such, the whole operation
   probably needs a rethink, and since it's not currently used we can defer
   that until it's needed.
 - <csr-id-aaa0af4d1685c65a3c166070a590c10e9fd54765/> unignore routing demote test
 - <csr-id-73b405d1228244a2b984e1294d5e8542f8691cef/> tweak demotion test
 - <csr-id-0c9c1f2edd9e872b9ba1642ac50f59a63f68b488/> use prefixmap update as opposed to verify against redundant same chain
 - <csr-id-9bdd68f313b3b7881cb39db02026184bbab0bfb0/> trust any valid key for chain updates
   But only return if it was successfully inserted into prefixmap
 - <csr-id-8b7b66450349b669e023539e92c77f9a3b830948/> move known prefix log to verify and udpate
 - <csr-id-7fc29f8187a29c2eeff8bbb5e09f068414bb8b93/> add check to received sap on ae update
   other cleanup
 - <csr-id-4415c9b1d166f7e53032a0100d829e8581255a1e/> use constants for message priority
 - <csr-id-8eb1877effb8cf0bfc8986c23d49d727500087dd/> remove unused data exchange structs
 - <csr-id-0b8413998f018d7d577f2248e36e21f6c2744116/> minor refactor to prefix_map reading
 - <csr-id-aaa6903e612178ce59481b2e81fe3bd0d1cc2617/> try to update Section before updating Node info when relocating
 - <csr-id-2651423c61b160841557d279c9b706abdaab4cdf/> increase node resource proof difficulty
 - <csr-id-20461a84dbc0fc373b184a3982a79affad0544f6/> tweak join backoff
 - <csr-id-6a8f5a1f41e5a0f4c0cce6914d4b330b68f5e5d8/> remove core RwLock
 - <csr-id-da70738ff0e24827b749a970f466f3983b70442c/> upgrade `sn_launch_tool`
   The new version uses the upgraded versions of tracing which no longer
   depend on `chrono`.
 - <csr-id-7ddf2f850441e6596133ab7a596eb766380008c3/> tweak join retry backoff timing
 - <csr-id-0387123114ff6ae42920577706497319c8a888cb/> upgrade `tracing-appender` and `tracing-subscriber`
   These new versions have dropped their dependence on `chrono`, which has
   an active security advisory against it (RUSTSEC-2020-0159) which seems
   unlikely to be resolved.
   
   `chrono` is still being pulled in by `qp2p` (via `rcgen`), `sn_api`, and
   `sn_launch_tool`. This will be fixed in future commits.
 - <csr-id-aa17309092211e8d1ba36d4aaf50c4677a461594/> tweak join retry backoff timing
 - <csr-id-225432908839359800d301d9e5aa8274e4652ee1/> move safe_network code into sn directory
   Due to the fact that we're now using multiple crates, the safe_network code is moved into the `sn`
   directory.
   
   A Cargo.toml is added to the root directory to establish this repository as a workspace, currently
   with 2 members, sn and sn_api. If you now run a `cargo build` at the root directory, it will build
   both of these crates.
   
   The Github Actions workflows that were brought in from the `sn_api` merge were also removed here.
 - <csr-id-7b430a54f50846a8475cec804bc24552043558b7/> don't backoff when sending join resource challenge responses
 - <csr-id-fe62ad9f590f151dd20a6832dfab81d34fc9c020/> remove extraneous comment
 - <csr-id-9e3cb4e7e4dc3d1b8fa23455151de60d5ea03d4d/> cleanup comments
 - <csr-id-37aec09a555d428c730cdd982d06cf5cb58b60b1/> some clippy cleanup
   excuse the nonsense commit!
 - <csr-id-810fe0797a4f23a6bb2586d901ccac9272a9beb2/> remove unused debug
 - <csr-id-ca074b92b31add1ad6d0db50f2ba3b3d1ae25d5a/> add some DKG related markers
 - <csr-id-993d564f0ad5d8cac8d9a32b8c6c8d1dd00c3fd9/> move AE updat resend after retry to only happen for validated saps
 - <csr-id-fd8e8f3fb22f8592a45db33e75f237ef22cd1f5b/> add LogMarker for all different send msg types
 - <csr-id-c1c2bcac021dda4bf18a4d80ad2f86370d56efa7/> make elder agreemeent its own blocking command
   make more commands non blocking
 - <csr-id-e2727b91c836619dadf2e464ee7c57b338427f22/> flesh out far too many 'let _ = '
 - <csr-id-148140f1d932e6c4b30122ebcca3450ab6c84544/> make section peers concurrent by using Arc<DashMap>

### New Features

<csr-id-1e92fa5a2ae4931f6265d82af121125495f58655/>
<csr-id-56f3b514fceccbc1cc47256410b4f2119bb8affd/>
<csr-id-ddad0b8ce37d3537a9e9ed66da18758b6b3ace68/>
<csr-id-cfaed1ece5120d60e9f352b4e9ef70448e2ed3f2/>
<csr-id-60239655dba08940bd293b3c9243ac732923acfe/>
<csr-id-4cafe78e3fdb60d144e8cf810788116ce01de025/>
<csr-id-d1ecf96a6d965928d434810ccc9c89d1bc7fac4e/>
<csr-id-958e38ecd3b4e4dc908913192a1d43b83e082d08/>

 - <csr-id-a3552ae2dd0f727a71505d832c1ed2520283e8c8/> add network health check script to easily wait until we're ready+healthy
 - <csr-id-ba5f28475048bfaebcc37c660bec65644e4e52fe/> cache prefix_map for clients
   - also refactors methods for writing data to disk

### Bug Fixes

 - <csr-id-e406a9a61bb313dcd445d639e82fa8ae1ff99964/> refactor the Join process regarding used_recipients
 - <csr-id-db76c5ee5b2efc35214d12df0a2aa4e137231fa6/> if attempting to send a msg fails, continue with the rest of cmd processing instead of bailing out
 - <csr-id-5b596a444d24f7021db9d3c322bafa33d49dcfae/> client config test timeout/keepalive needed updating
 - <csr-id-d44fd8d5888c554bf1aa0d56e08471cfe90bd988/> routing test tweaks
 - <csr-id-a927d9e27d831a48348eab6d41e0f4231b0e62c7/> make health check tolerante of possible demotions
 - <csr-id-42a3123b0aa4f01daedc2faebd3fa3430dbc1618/> tesnet grep is_log_file check
 - <csr-id-9214dff1541b28577c44d8cbbebeec0b80598653/> node tests after removing blocking cmd layer
 - <csr-id-bc47eed7e327a8e02fff1d385648c752ad33b8f1/> fix windows grep lock error: dont search non log files
 - <csr-id-27e8d895983d236a0949354219c4adbe3f1e22a0/> readd keep alive for client
 - <csr-id-e45dd10ac770ed41431f0e8758a6228fc4dfbe3c/> update logs during AEUpdate
 - <csr-id-a0806aa624384ccb437bc8b4b2d108523ea5c068/> bring back elders len check during chain updates
 - <csr-id-57b736a8901ed76c402460fcf1799162cfdb3c37/> match on name
 - <csr-id-6eb389acfb3adcec0be07ee990106ed19a7f78f5/> make `KvStore::store` perform atomic immutable writes
   `KvStore` is the database abstraction we use to persist chunks. This is
   itself backed by `sled`. `sled` uses a log as one of the underlying
   storage primitives, which means that all inserts (even overwrites) will
   consume additional space (until the log is compacted, which may happen
   irregularly).
   
   However, chunks themselves are write-once which means that we would only
   ever need to write a value once. As such, we can use `compare_and_swap`
   to write the value only if it's not currently set. Since chunks are
   content-addressable, we furthermore don't need to do anything if the
   value did already exist and wasn't written, since there is only one
   possible value for a given key.
 - <csr-id-b53a86b9ba0bfb33ab9cccaecb383bcce0acf233/> fix test for `GetChunk` 'data not found' failures
   `QueryResponse::failed_with_data_not_found` is used to prevent sending
   'data not found' errors to clients. Clients, in turn, do not retry
   errors, only timeouts.
   
   Recently (c0b1770154) the error type returned when a chunk is not found
   was changed from `NoSuchData` to `ChunkNotFound`, but the check in
   `failed_with_data_not_found` was not updated. This could lead to
   spurious client errors when they attempt to read data before it is
   available (which would otherwise be covered by the retry on timeout,
   since nodes would not forward the data-not-found response).
 - <csr-id-5f985342239b74963f89e627e28031f8817c0d3d/> typo
 - <csr-id-a1910095d4b7a749186d614e04dad2b64a1eabff/> spot and blob management inconsistency add some comments
 - <csr-id-ab98ec2076c1f6c60899c505ed85aaefa5359278/> replace prefixmap check to ensure last proof chainkey is present on provided sap
 - <csr-id-96519e26ef9adf6360b6a8f79ca73ab4bc7b627b/> elders update uses correct key for prefixmap updates
 - <csr-id-3569939ff83978ba50039588eb87d4e6da4fedd2/> updated routing untrusted ae test for new error
 - <csr-id-f615aacdb4cd972545ba88c927cfb5a9b357fb9a/> provide valid proof chain to update prefix map during elders agreement handling
 - <csr-id-51ea358eff0edee0b27c5e21af4f38a7ee09422c/> keep section in sync on ae retry recevial at nodes
 - <csr-id-0df6615df2fdb88d7fd89f9751c6b88e1b7ebb5f/> ae blob tests dont have standard wait
 - <csr-id-b0319628d587a2caeb1aa5be52505cbb6ede40d3/> use client query in routing tests instead of bytes
 - <csr-id-20a69ccb6c6b1f466b519318dd85ef11e7700027/> don't bail out when handling ElderAgreement if prefix-map is not updated
 - <csr-id-0ea37af54f761b2a1a5137463637736c6be25206/> enough having enough elders before split attempt
 - <csr-id-0d3fe1a6e02f6d83115e9098c706a98cb688d41d/> allow to set the genesis key for Core and Section structs of first node
 - <csr-id-c360dc61bd7d4ce1745ba7bfbe9d032fedac067a/> prevent deadlock by dropping write locks in relocation
 - <csr-id-084926e045952670295eb666c82f1d77ff88f1be/> fix a possible panic when creating client
   `client::connections::Session::make_contact_with_nodes` would slice the
   known initial contacts, but this would panic if fewer than
   `NODES_TO_CONTACT_PER_STARTUP_BATCH` (currently: 3) nodes are known.
   It's not clear that this is a precondition or requirement, so for now we
   simply take as many as we can up to
   `NODES_TO_CONTACT_PER_STARTUP_BATCH`. If it *is* a requirement, we
   should return an `Err` rather than panic.
 - <csr-id-3a19e42770a12885659d653b98511378dad8015f/> improve join req handling for elders
 - <csr-id-2d807cb4b75bfd52d9dde7b91214bd3dc5a6d992/> maintain lexicographical order
 - <csr-id-c765828018158bda663f937a73bcc47ab358884c/> write to correct source
 - <csr-id-ba7c4c609150839825a4992c08e3bdd00b698269/> get more splits (at the cost of more join msgs)
 - <csr-id-104ed366dbaafd98fd3ef67899e27540894a959c/> dont wait before first join request
 - <csr-id-0033fc65db266d92623f53628f9f5b6e6069920d/> fix elders agreement routing tests using wrong command type
 - <csr-id-ec3f16a8c33e8afbdfffb392466ba422216d3f68/> missing post rebase async_trait dep
 - <csr-id-73c5baf94dd37346c6c9987aa51ca26f3a2fea1f/> only resend if we've had an update
 - <csr-id-6aa1130a7b380dc3d1ad12f3054cda3a390ff20d/> client config test
 - <csr-id-c88a1bc5fda40093bb129b4351eef73d2eb7c041/> resolution test except for the blob range tests
 - <csr-id-fd4513f054e282218797208dcac1de6903e94f2c/> build `sn_node` from inside `sn` directory
   Since we moved to a workspace layout, the `testnet` binary has been
   building more than necessary since it runs `cargo build` from the
   workspace root. Additionally, cargo's feature resolution may behave
   unexpectedly when executed from the workspace directory, so rather than
   using `-p safe_network` we rather run `cargo build` from the `sn`
   directory.
 - <csr-id-7ff7557850460d98b526646b21da635381a70e2a/> exit with error if `sn_node` build fails
   We would previously ignore the result of trying to build the `sn_node`
   binary, which would lead to either using the incorrect binary if one
   existed, or a later error because the binary doesn't exist.

### New Features (BREAKING)

 - <csr-id-06b57d587da4882bfce1b0acd09faf9129306ab2/> add log markers for connection open/close
   We can detect connection open/close easily in the connection listener
   tasks, since these are started as soon as a connection is opened, and
   finish when there are no more incoming connections (e.g. connection has
   closed).
 - <csr-id-20895dd4326341de4d44547861ac4a57ae8531cf/> the JoinResponse::Retry message now provides the expected age for the joining node

### Refactor (BREAKING)

 - <csr-id-61dec0fd90b4df6b0695a7ba46da86999d199d4a/> remove `SectionAuthorityProviderUtils::elders`
   Since the `elders` field of `SectionAuthorityProvider` is public, this
   method is essentially equivalent to `sap.elders.clone()`. Furthermore,
   it was barely used (5 call sites).
 - <csr-id-49c76d13a91474038bd8cb005959a37a7d4c6603/> tweak `Peer` API
   `Peer`'s fields are no longer public, and the `name` and `addr` methods
   return owned (copied) values. There's no point in having both a public
   field and a getter, and the getters were used far more often. The
   getters have been changed to return owned values, since `XorName` and
   `SocketAddr` are both `Copy`, so it seems reasonable to avoid the
   indirection of a borrow.
 - <csr-id-c5d2e31a5f0cea381bb60dc1f896dbbda5038506/> remove `PeerUtils` trait
   This trait has been redundant since the `sn_messaging` and `sn_routing`
   crate were merged, but it's doubly so when the `Peer` struct is now
   itself in `routing`.
 - <csr-id-0cb790f5c3712e357f685bfb88cd237c5b5f76c5/> move `Peer` from `messaging` to `routing`
   The `Peer` struct no longer appears in any message definitions, and as
   such doesn't belong in `messaging`. It has been moved to `routing`,
   which is the only place it's now used.
   
   The derive for `Deserialize` and `Serialize` have also been removed from
   `Peer`, since they're no longer needed.
 - <csr-id-b3ce84012e6cdf4c87d6d4a3137ab6506264e949/> remove `Peer` from `NodeState`
   Syntactially, having a "peer" struct inside `NodeState` doesn't make a
   lot of sense, vs. including the name and address directly in the state.
   This also unblocks moving `Peer` out of `messaging`, which will remove
   the requirement for it to be serialisable etc.
   
   To make migration easier, an `age` method was added to the
   `NodeStateUtils` trait.
 - <csr-id-ecf0bc9a9736167edb15db7ff4e3cf5dc388dd22/> remove `reachable` from `messaging::system::Peer`
   Although the field is `false` on initialisation, and it is checked in a
   couple of places, in every case as far as I can tell it is set to `true`
   right after construction. As such, it's not really doing anything for us
   and we can remove it.

