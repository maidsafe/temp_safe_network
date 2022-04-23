# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.1.2 (2022-04-23)

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

 - 7 commits contributed to the release over the course of 2 calendar days.
 - 27 days passed between releases.
 - 6 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - tidy references in cargo manifests ([`318ee1d`](https://github.com/maidsafe/safe_network/commit/318ee1d22970b5f06e93a99b6e8fff6da638c589))
    - Merge #1122 ([`f359a45`](https://github.com/maidsafe/safe_network/commit/f359a45971a5b42a6f174536475f47b8ab076901))
    - remove modules that only contained tests ([`6452690`](https://github.com/maidsafe/safe_network/commit/6452690c1b75bb8804c1f9de19c394a83f178acb))
    - move request_operation_fulfilled ([`08385f4`](https://github.com/maidsafe/safe_network/commit/08385f4e03cd43b94f15523597f90f1cc9977a87))
    - remove op_id arg from track_issue ([`1f3af46`](https://github.com/maidsafe/safe_network/commit/1f3af46aea59ebeb1b6a4b736e80e86ce2f724d8))
    - remove unused dep ([`66901bc`](https://github.com/maidsafe/safe_network/commit/66901bcb3b68d3fbe84bfde915bb80ae1b562347))
    - compare against all nodes in section ([`5df610c`](https://github.com/maidsafe/safe_network/commit/5df610c93b76cfc3a6f09734476240313b16bee6))
</details>

## v0.1.1 (2022-03-26)

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

