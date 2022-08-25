# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.69.0 (2022-08-25)

## v0.68.0 (2022-08-23)

<csr-id-c8517a481e39bf688041cd8f8661bc663ee7bce7/>
<csr-id-589f03ce8670544285f329fe35c19897d4bfced8/>
<csr-id-ddbbb53d61d6c94b00a47dc2b708a2aeda870d96/>
<csr-id-1618cf6a93117942946d152efee24fe3c7020e55/>
<csr-id-63172ab4ab9fc87bc17b09c6fd384679a37a40f0/>
<csr-id-f0fbe5fd9bec0b2865271bb139c9fcb4ec225884/>

### Chore

 - <csr-id-c8517a481e39bf688041cd8f8661bc663ee7bce7/> fix clippy some/none issues
 - <csr-id-589f03ce8670544285f329fe35c19897d4bfced8/> upgrading sn_dbc to v8.0
 - <csr-id-ddbbb53d61d6c94b00a47dc2b708a2aeda870d96/> leave out unnecessary Arc<RwLock>

### Chore

 - <csr-id-43fcc7c517f95eab0e27ddc79cd9c6de3631c7c6/> sn_interface-0.10.0/sn_dysfunction-0.9.0/sn_client-0.70.0/sn_node-0.66.0/sn_api-0.68.0/sn_cli-0.61.0

### Bug Fixes

 - <csr-id-43ecab2dda52cb0ede7c0d4b6e48eaffe1fb6b75/> reintroduce Arc<RwLock> for section tree
   The RwLock was mistakenly removed by me. This meant that network updates
   to the section tree were not propagated back to the client's session.

### Refactor

 - <csr-id-1618cf6a93117942946d152efee24fe3c7020e55/> expose serialisation/deserialisation utilities as public methods instead
   - Also include the genesis key of each network in the list shown by CLI networks cmd.
 - <csr-id-63172ab4ab9fc87bc17b09c6fd384679a37a40f0/> circumvent clone to use reference

### Refactor (BREAKING)

 - <csr-id-f0fbe5fd9bec0b2865271bb139c9fcb4ec225884/> renaming NetworkPrefixMap to SectionTree
   - Changing CLI and sn_client default path for network contacts to `$HOME/.safe/network_contacts`.
   - Renaming variables and functions referring to "prefix map" to now refer to "network contacts".

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 8 commits contributed to the release over the course of 8 calendar days.
 - 9 days passed between releases.
 - 8 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.10.0/sn_dysfunction-0.9.0/sn_client-0.70.0/sn_node-0.66.0/sn_api-0.68.0/sn_cli-0.61.0 ([`43fcc7c`](https://github.com/maidsafe/safe_network/commit/43fcc7c517f95eab0e27ddc79cd9c6de3631c7c6))
    - fix clippy some/none issues ([`c8517a4`](https://github.com/maidsafe/safe_network/commit/c8517a481e39bf688041cd8f8661bc663ee7bce7))
    - reintroduce Arc<RwLock> for section tree ([`43ecab2`](https://github.com/maidsafe/safe_network/commit/43ecab2dda52cb0ede7c0d4b6e48eaffe1fb6b75))
    - upgrading sn_dbc to v8.0 ([`589f03c`](https://github.com/maidsafe/safe_network/commit/589f03ce8670544285f329fe35c19897d4bfced8))
    - renaming NetworkPrefixMap to SectionTree ([`f0fbe5f`](https://github.com/maidsafe/safe_network/commit/f0fbe5fd9bec0b2865271bb139c9fcb4ec225884))
    - expose serialisation/deserialisation utilities as public methods instead ([`1618cf6`](https://github.com/maidsafe/safe_network/commit/1618cf6a93117942946d152efee24fe3c7020e55))
    - circumvent clone to use reference ([`63172ab`](https://github.com/maidsafe/safe_network/commit/63172ab4ab9fc87bc17b09c6fd384679a37a40f0))
    - leave out unnecessary Arc<RwLock> ([`ddbbb53`](https://github.com/maidsafe/safe_network/commit/ddbbb53d61d6c94b00a47dc2b708a2aeda870d96))
</details>

## v0.67.0 (2022-08-14)

<csr-id-de57210562e1e3a637564332e081514dabb177ab/>
<csr-id-29de67f1e3583eab867d517cb50ed2e404bd63fd/>
<csr-id-afcf083469c732f10c7c80f4a45e4c33ab111101/>
<csr-id-06c3859cf739487b9c27de6fcdf5078f82403b4f/>
<csr-id-5050522c85a3430ee017db3215aad21619bf7796/>
<csr-id-b98e46116628c62b71e7cc4171aeda86b05b2b99/>
<csr-id-6f03b93bd2d02f0ffe54b69fbf25070fbe64eab0/>
<csr-id-27ba2a63dcfa272cf7ef8c5301987fc6bfe18ed0/>
<csr-id-ed37bb56e5e17d4cba7c1b2165746c193241d618/>
<csr-id-14ea6c7f4bbaee9c2ac4a30fba938ef2de2f77e5/>
<csr-id-e0fb940b24e87d86fe920095176362f73503ce79/>
<csr-id-ca32230926e5a435d90694df8fbce1218ea397f0/>
<csr-id-9fde534277f359dfa0a1d91d917864776edb5138/>
<csr-id-5c82df633e7c062fdf761a8e6e0a7ae8d26cc73a/>
<csr-id-93614b18b4316af04ab8c74358a5c86510590b85/>
<csr-id-f5af444b8ac37d2debfbe5e1d4dcdc48de963694/>
<csr-id-9e87a00f8749de236cd9722b22936ae86cfdcf4e/>
<csr-id-d4be0cc431947b035046cc4d56642a81c0880924/>
<csr-id-db7dcdc7968d1d7e946274650d5a0c48719b4955/>
<csr-id-d3a05a728be8752ea9ebff4e38e7c4c85e5db09b/>
<csr-id-96da1171d0cac240f772e5d6a15c56f63441b4b3/>
<csr-id-dd2eb21352223f6340064e0021f4a7df402cd5c9/>
<csr-id-53f60c2327f8a69f0b2ef6d1a4e96644c10aa358/>

### Chore

 - <csr-id-de57210562e1e3a637564332e081514dabb177ab/> let client builder do env overrides
   The CLI/api had its own env vars to set timeout; delegate this to the
   client builder
 - <csr-id-29de67f1e3583eab867d517cb50ed2e404bd63fd/> serialize NetworkPrefixMap into JSON
 - <csr-id-afcf083469c732f10c7c80f4a45e4c33ab111101/> remove RwLock from NetworkPrefixMap
 - <csr-id-06c3859cf739487b9c27de6fcdf5078f82403b4f/> additional error information within a wallet_reissue error msg
 - <csr-id-5050522c85a3430ee017db3215aad21619bf7796/> increase the DBC_REISSUE_ATTEMPTS
 - <csr-id-b98e46116628c62b71e7cc4171aeda86b05b2b99/> perform up to three attempts to reissue DBCs for tests from genesis DBC
 - <csr-id-6f03b93bd2d02f0ffe54b69fbf25070fbe64eab0/> upgrade blsttc to 7.0.0
   This version has a more helpful error message for the shares interpolation problem.

### Chore

 - <csr-id-53f60c2327f8a69f0b2ef6d1a4e96644c10aa358/> sn_interface-0.9.0/sn_dysfunction-0.8.0/sn_client-0.69.0/sn_node-0.65.0/sn_api-0.67.0/sn_cli-0.60.0

### New Features

<csr-id-ba97ca06b67cd6e5de8e1c910b396fbe44f40fd7/>
<csr-id-1b3f0516cf899c2fc0d101ce9cf0079c95bbfd7b/>

 - <csr-id-796b9e640ddafcbc804cd4792a867143422cf4f5/> expose a public API to reissue many output DBCs from a wallet
 - <csr-id-005b84cab0ca91762cbedd208b022d4c4983fe26/> retry twice if it fails spending inputs when reissuing DBCs
 - <csr-id-c46dd0737779c8ee515ee037add54ce049448ea7/> expose a public API which allows users to check if a DBC's `KeyImage` has been already spent on the network
   - Expose a public `is_dbc_spent` API which allows users to check if a DBC's KeyImage has
   been already spent on the network.

### Bug Fixes

 - <csr-id-f4b89d390eaeae0ab6dd329c1a0e9bbc65ec28a6/> update prefixmap getter call after name change

### Refactor

 - <csr-id-27ba2a63dcfa272cf7ef8c5301987fc6bfe18ed0/> sn_client to only read a default prefix map file, updates to be cached on disk by user
   - CLI to cache the up to date PrefixMap after all commands were executed and right before exiting.
   - Refactoring sn_cli::Config to remove some redundant code.
 - <csr-id-ed37bb56e5e17d4cba7c1b2165746c193241d618/> move SectionChain into NetworkPrefixMap
 - <csr-id-14ea6c7f4bbaee9c2ac4a30fba938ef2de2f77e5/> use builder to instantiate
 - <csr-id-e0fb940b24e87d86fe920095176362f73503ce79/> use sn_dbc::SpentProof API for verifying SpentProofShares
 - <csr-id-ca32230926e5a435d90694df8fbce1218ea397f0/> remove unused storage path
 - <csr-id-9fde534277f359dfa0a1d91d917864776edb5138/> reissuing DBCs for all sn_cli tests only once as a setup stage
 - <csr-id-5c82df633e7c062fdf761a8e6e0a7ae8d26cc73a/> setup step for tests to reissue a set of DBCs from genesis only once
 - <csr-id-93614b18b4316af04ab8c74358a5c86510590b85/> make chunk_store accept all datatypes
 - <csr-id-f5af444b8ac37d2debfbe5e1d4dcdc48de963694/> removing hard-coded test DBC from sn_api Wallet unit tests

### Test

 - <csr-id-9e87a00f8749de236cd9722b22936ae86cfdcf4e/> fix test helper for reissuing dbcs
 - <csr-id-d4be0cc431947b035046cc4d56642a81c0880924/> additional tests in sn-api for DBC verification failures

### Chore (BREAKING)

 - <csr-id-db7dcdc7968d1d7e946274650d5a0c48719b4955/> remove providing path to qp2p cfg
   This configuration seems never to be provided or stored anyway. It looks
   like some code was also taking this parameter to be the client config,
   not the qp2p config, which is a source of confusion.
 - <csr-id-d3a05a728be8752ea9ebff4e38e7c4c85e5db09b/> having spent proofs and Txs within SpentbookCmd::Send msg to be a set instead of a vec

### New Features (BREAKING)

 - <csr-id-f666204febb1044980412345236ce0cb8377b162/> return reference instead of clone
   Let the end user decide on wether to clone a value that is taken from
   the struct.

### Refactor (BREAKING)

 - <csr-id-96da1171d0cac240f772e5d6a15c56f63441b4b3/> nodes to cache their own individual prefix map file on disk
 - <csr-id-dd2eb21352223f6340064e0021f4a7df402cd5c9/> removing Token from sn_interfaces::type as it is now exposed by sn_dbc

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 35 commits contributed to the release over the course of 33 calendar days.
 - 34 days passed between releases.
 - 30 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.9.0/sn_dysfunction-0.8.0/sn_client-0.69.0/sn_node-0.65.0/sn_api-0.67.0/sn_cli-0.60.0 ([`53f60c2`](https://github.com/maidsafe/safe_network/commit/53f60c2327f8a69f0b2ef6d1a4e96644c10aa358))
    - update prefixmap getter call after name change ([`f4b89d3`](https://github.com/maidsafe/safe_network/commit/f4b89d390eaeae0ab6dd329c1a0e9bbc65ec28a6))
    - sn_client to only read a default prefix map file, updates to be cached on disk by user ([`27ba2a6`](https://github.com/maidsafe/safe_network/commit/27ba2a63dcfa272cf7ef8c5301987fc6bfe18ed0))
    - let client builder do env overrides ([`de57210`](https://github.com/maidsafe/safe_network/commit/de57210562e1e3a637564332e081514dabb177ab))
    - serialize NetworkPrefixMap into JSON ([`29de67f`](https://github.com/maidsafe/safe_network/commit/29de67f1e3583eab867d517cb50ed2e404bd63fd))
    - nodes to cache their own individual prefix map file on disk ([`96da117`](https://github.com/maidsafe/safe_network/commit/96da1171d0cac240f772e5d6a15c56f63441b4b3))
    - removing Token from sn_interfaces::type as it is now exposed by sn_dbc ([`dd2eb21`](https://github.com/maidsafe/safe_network/commit/dd2eb21352223f6340064e0021f4a7df402cd5c9))
    - remove RwLock from NetworkPrefixMap ([`afcf083`](https://github.com/maidsafe/safe_network/commit/afcf083469c732f10c7c80f4a45e4c33ab111101))
    - move SectionChain into NetworkPrefixMap ([`ed37bb5`](https://github.com/maidsafe/safe_network/commit/ed37bb56e5e17d4cba7c1b2165746c193241d618))
    - expose a public API to reissue many output DBCs from a wallet ([`796b9e6`](https://github.com/maidsafe/safe_network/commit/796b9e640ddafcbc804cd4792a867143422cf4f5))
    - use builder to instantiate ([`14ea6c7`](https://github.com/maidsafe/safe_network/commit/14ea6c7f4bbaee9c2ac4a30fba938ef2de2f77e5))
    - return reference instead of clone ([`f666204`](https://github.com/maidsafe/safe_network/commit/f666204febb1044980412345236ce0cb8377b162))
    - additional error information within a wallet_reissue error msg ([`06c3859`](https://github.com/maidsafe/safe_network/commit/06c3859cf739487b9c27de6fcdf5078f82403b4f))
    - remove providing path to qp2p cfg ([`db7dcdc`](https://github.com/maidsafe/safe_network/commit/db7dcdc7968d1d7e946274650d5a0c48719b4955))
    - Merge #1427 ([`949ee11`](https://github.com/maidsafe/safe_network/commit/949ee111717c8f07487f3f4db6fbc0043583916d))
    - increase the DBC_REISSUE_ATTEMPTS ([`5050522`](https://github.com/maidsafe/safe_network/commit/5050522c85a3430ee017db3215aad21619bf7796))
    - use sn_dbc::SpentProof API for verifying SpentProofShares ([`e0fb940`](https://github.com/maidsafe/safe_network/commit/e0fb940b24e87d86fe920095176362f73503ce79))
    - remove unused storage path ([`ca32230`](https://github.com/maidsafe/safe_network/commit/ca32230926e5a435d90694df8fbce1218ea397f0))
    - retry twice if it fails spending inputs when reissuing DBCs ([`005b84c`](https://github.com/maidsafe/safe_network/commit/005b84cab0ca91762cbedd208b022d4c4983fe26))
    - fix test helper for reissuing dbcs ([`9e87a00`](https://github.com/maidsafe/safe_network/commit/9e87a00f8749de236cd9722b22936ae86cfdcf4e))
    - perform up to three attempts to reissue DBCs for tests from genesis DBC ([`b98e461`](https://github.com/maidsafe/safe_network/commit/b98e46116628c62b71e7cc4171aeda86b05b2b99))
    - having spent proofs and Txs within SpentbookCmd::Send msg to be a set instead of a vec ([`d3a05a7`](https://github.com/maidsafe/safe_network/commit/d3a05a728be8752ea9ebff4e38e7c4c85e5db09b))
    - upgrade blsttc to 7.0.0 ([`6f03b93`](https://github.com/maidsafe/safe_network/commit/6f03b93bd2d02f0ffe54b69fbf25070fbe64eab0))
    - expose a public API which allows users to check if a DBC's `KeyImage` has been already spent on the network ([`c46dd07`](https://github.com/maidsafe/safe_network/commit/c46dd0737779c8ee515ee037add54ce049448ea7))
    - additional tests in sn-api for DBC verification failures ([`d4be0cc`](https://github.com/maidsafe/safe_network/commit/d4be0cc431947b035046cc4d56642a81c0880924))
    - reissuing DBCs for all sn_cli tests only once as a setup stage ([`9fde534`](https://github.com/maidsafe/safe_network/commit/9fde534277f359dfa0a1d91d917864776edb5138))
    - perform verification of input TX and spentproofs when depositing or reissuing a DBC ([`ba97ca0`](https://github.com/maidsafe/safe_network/commit/ba97ca06b67cd6e5de8e1c910b396fbe44f40fd7))
    - setup step for tests to reissue a set of DBCs from genesis only once ([`5c82df6`](https://github.com/maidsafe/safe_network/commit/5c82df633e7c062fdf761a8e6e0a7ae8d26cc73a))
    - make chunk_store accept all datatypes ([`93614b1`](https://github.com/maidsafe/safe_network/commit/93614b18b4316af04ab8c74358a5c86510590b85))
    - removing hard-coded test DBC from sn_api Wallet unit tests ([`f5af444`](https://github.com/maidsafe/safe_network/commit/f5af444b8ac37d2debfbe5e1d4dcdc48de963694))
    - Merge branch 'main' into feat-cat-wallet-improvements ([`08a3b85`](https://github.com/maidsafe/safe_network/commit/08a3b85ae73b2360e63f9d4fbdec23e349dc0626))
    - Merge branch 'main' into feat-cat-wallet-improvements ([`e2e89e6`](https://github.com/maidsafe/safe_network/commit/e2e89e6b061ae0827cdeeb1d8b17e702d2f3607a))
    - Merge branch 'main' into feat-cat-wallet-improvements ([`9409bf4`](https://github.com/maidsafe/safe_network/commit/9409bf42e99b4eb3da883f76c802e7dc6ea1a4a0))
    - Merge branch 'main' into feat-cat-wallet-improvements ([`8e6eecf`](https://github.com/maidsafe/safe_network/commit/8e6eecf0da8df5cdac55bbf1f81d00bcb19558b4))
    - show the DBC owner in the wallet displayed by cat cmd ([`1b3f051`](https://github.com/maidsafe/safe_network/commit/1b3f0516cf899c2fc0d101ce9cf0079c95bbfd7b))
</details>

<csr-unknown>
Have the CLI wallet deposit command to perform a verification is the supplied DBC has beenalready spent before depositing into a wallet.Allow users to provide a --force flag with the CLI wallet deposit command to skip theverification of DBC already spent and force the deposit into the wallet.Display the owner of each DBC when cat-ing a wallet.Align to the right the balance of each DBC when cat-ing a wallet.Shorten the default name set to DBC when deposited in a wallet.Make the name of the change DBC automatically deposited in the wallet unique.<csr-unknown/>

## v0.66.3 (2022-07-10)

<csr-id-49e223e2c07695b4c63e253ba19ce43ec24d7112/>
<csr-id-34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8/>

### Chore

 - <csr-id-49e223e2c07695b4c63e253ba19ce43ec24d7112/> move more deps to clap-v3; rm some deps on rand

### Chore

 - <csr-id-34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8/> sn_interface-0.8.2/sn_client-0.68.2/sn_node-0.64.2/sn_api-0.66.3/sn_cli-0.59.3

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.2/sn_client-0.68.2/sn_node-0.64.2/sn_api-0.66.3/sn_cli-0.59.3 ([`34bd9bd`](https://github.com/maidsafe/safe_network/commit/34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8))
    - move more deps to clap-v3; rm some deps on rand ([`49e223e`](https://github.com/maidsafe/safe_network/commit/49e223e2c07695b4c63e253ba19ce43ec24d7112))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`45418f2`](https://github.com/maidsafe/safe_network/commit/45418f2f9b5cc58f2a153bf40966beb2bf36a62a))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`94be181`](https://github.com/maidsafe/safe_network/commit/94be181789b0010f83ed5e89341f3f347575e37f))
</details>

## v0.66.2 (2022-07-08)

<csr-id-b478314f331382229c9fb235dab0198f5203f509/>

### Chore

 - <csr-id-b478314f331382229c9fb235dab0198f5203f509/> sn_api-0.66.2/sn_cli-0.59.2

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.66.2/sn_cli-0.59.2 ([`b478314`](https://github.com/maidsafe/safe_network/commit/b478314f331382229c9fb235dab0198f5203f509))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`44411d5`](https://github.com/maidsafe/safe_network/commit/44411d511a496b13893670c8bc7d9f43f0ce9073))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`45309c4`](https://github.com/maidsafe/safe_network/commit/45309c4c0463dd9198a49537187417bf4bfdb847))
    - Merge branch 'main' into feat-cli-wallet-show-deposited-amount ([`6268fe7`](https://github.com/maidsafe/safe_network/commit/6268fe76e9dd81d291492b4611094273f8d1e223))
</details>

## v0.66.1 (2022-07-07)

<csr-id-c79e2aac378b28b373fd7c18c4b9006348960071/>
<csr-id-2b00cec961561281f6b927e13e501342843f6a0f/>

### Chore

 - <csr-id-c79e2aac378b28b373fd7c18c4b9006348960071/> bit more low hanging clippy fruit

### New Features (BREAKING)

 - <csr-id-79a53b0d1df5a9377cfe7a9d70480ed1fa31bacc/> wallet_deposit API to also return the amount desposited

### Chore

 - <csr-id-2b00cec961561281f6b927e13e501342843f6a0f/> sn_interface-0.8.1/sn_dysfunction-0.7.1/sn_client-0.68.1/sn_node-0.64.1/sn_api-0.66.1/sn_cli-0.59.1

### New Features

 - <csr-id-57f635fbe80392574f7f122a9d157fbb6320c4cc/> generate the genesis DBC when launching first node and write it to disk
 - <csr-id-8313ed8d5b45b7f4ed3b36ada231e74c49c9f9e6/> perform signature verifications on input DBC SpentProof before signing new spent proof share

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 1 calendar day.
 - 2 days passed between releases.
 - 5 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.1/sn_dysfunction-0.7.1/sn_client-0.68.1/sn_node-0.64.1/sn_api-0.66.1/sn_cli-0.59.1 ([`2b00cec`](https://github.com/maidsafe/safe_network/commit/2b00cec961561281f6b927e13e501342843f6a0f))
    - Merge branch 'main' into feat-cli-wallet-show-deposited-amount ([`7e2a25a`](https://github.com/maidsafe/safe_network/commit/7e2a25ae31ead0fae7824ca794b6c407695080cd))
    - Merge branch 'main' into feat-cli-wallet-show-deposited-amount ([`7cf2eb6`](https://github.com/maidsafe/safe_network/commit/7cf2eb64e1176d2b23d63091f6f459d92bdccb57))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`f83724c`](https://github.com/maidsafe/safe_network/commit/f83724cff1e63b35f1612fc82dffdefbeaab6cc1))
    - Merge branch 'main' into feat-cli-wallet-show-deposited-amount ([`cd2f9aa`](https://github.com/maidsafe/safe_network/commit/cd2f9aa2f7001ae779273745f9ac78fc289525e3))
    - Merge branch 'main' into feat-cli-wallet-show-deposited-amount ([`39bd5b4`](https://github.com/maidsafe/safe_network/commit/39bd5b471b6b3acb6ebe90489335c995b0aca82f))
    - perform signature verifications on input DBC SpentProof before signing new spent proof share ([`8313ed8`](https://github.com/maidsafe/safe_network/commit/8313ed8d5b45b7f4ed3b36ada231e74c49c9f9e6))
    - wallet_deposit API to also return the amount desposited ([`79a53b0`](https://github.com/maidsafe/safe_network/commit/79a53b0d1df5a9377cfe7a9d70480ed1fa31bacc))
    - bit more low hanging clippy fruit ([`c79e2aa`](https://github.com/maidsafe/safe_network/commit/c79e2aac378b28b373fd7c18c4b9006348960071))
    - generate the genesis DBC when launching first node and write it to disk ([`57f635f`](https://github.com/maidsafe/safe_network/commit/57f635fbe80392574f7f122a9d157fbb6320c4cc))
</details>

## v0.66.0 (2022-07-04)

<csr-id-9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7/>
<csr-id-ddb7798a7b0c5e60960e123414277d58f3da27eb/>
<csr-id-2aae965ca2fdd4ff59034547b5ee8dcef0b7253e/>
<csr-id-5dbf50d92bf7e93acbb00e85f51910f32ac4a124/>
<csr-id-068327834c8d07ada6bf42cf78d6f7a117715466/>
<csr-id-976e8c3d8c610d2a34c1bfa6678132a1bad234e8/>
<csr-id-91da4d4ac7aab039853b0651e5aafd9cdd31b9c4/>
<csr-id-e4e2eb56611a328806c59ed8bc80ca2567206bbb/>

### Chore

 - <csr-id-9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7/> Docs - put symbols in backticks
 - <csr-id-ddb7798a7b0c5e60960e123414277d58f3da27eb/> remove let bindings for unit returns

### Chore

 - <csr-id-e4e2eb56611a328806c59ed8bc80ca2567206bbb/> sn_interface-0.8.0/sn_dysfunction-0.7.0/sn_client-0.68.0/sn_node-0.64.0/sn_api-0.66.0/sn_cli-0.59.0

### Refactor

 - <csr-id-2aae965ca2fdd4ff59034547b5ee8dcef0b7253e/> use hardlink instead of symlink
 - <csr-id-5dbf50d92bf7e93acbb00e85f51910f32ac4a124/> remove NodeConfig from sn_api::ipc, add sn_cli tests
 - <csr-id-068327834c8d07ada6bf42cf78d6f7a117715466/> sn_cli modify tests
 - <csr-id-976e8c3d8c610d2a34c1bfa6678132a1bad234e8/> sn_cli uses NetworkPrefixMap instead of node_conn_info.config
 - <csr-id-91da4d4ac7aab039853b0651e5aafd9cdd31b9c4/> remove node_connection_info.config from sn_node, sn_interface, sn_client

### New Features (BREAKING)

 - <csr-id-5dad80d3f239f5844243fedb89f8d4baaee3b640/> have the nodes to attach valid Commitments to signed SpentProofShares

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 4 calendar days.
 - 8 days passed between releases.
 - 9 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.0/sn_dysfunction-0.7.0/sn_client-0.68.0/sn_node-0.64.0/sn_api-0.66.0/sn_cli-0.59.0 ([`e4e2eb5`](https://github.com/maidsafe/safe_network/commit/e4e2eb56611a328806c59ed8bc80ca2567206bbb))
    - use hardlink instead of symlink ([`2aae965`](https://github.com/maidsafe/safe_network/commit/2aae965ca2fdd4ff59034547b5ee8dcef0b7253e))
    - remove NodeConfig from sn_api::ipc, add sn_cli tests ([`5dbf50d`](https://github.com/maidsafe/safe_network/commit/5dbf50d92bf7e93acbb00e85f51910f32ac4a124))
    - sn_cli modify tests ([`0683278`](https://github.com/maidsafe/safe_network/commit/068327834c8d07ada6bf42cf78d6f7a117715466))
    - sn_cli uses NetworkPrefixMap instead of node_conn_info.config ([`976e8c3`](https://github.com/maidsafe/safe_network/commit/976e8c3d8c610d2a34c1bfa6678132a1bad234e8))
    - remove node_connection_info.config from sn_node, sn_interface, sn_client ([`91da4d4`](https://github.com/maidsafe/safe_network/commit/91da4d4ac7aab039853b0651e5aafd9cdd31b9c4))
    - Docs - put symbols in backticks ([`9314a2d`](https://github.com/maidsafe/safe_network/commit/9314a2db5dc1ae91bc4d80a65c1a8825492fc7c7))
    - remove let bindings for unit returns ([`ddb7798`](https://github.com/maidsafe/safe_network/commit/ddb7798a7b0c5e60960e123414277d58f3da27eb))
    - have the nodes to attach valid Commitments to signed SpentProofShares ([`5dad80d`](https://github.com/maidsafe/safe_network/commit/5dad80d3f239f5844243fedb89f8d4baaee3b640))
</details>

## v0.65.0 (2022-06-26)

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

### New Features

 - <csr-id-341306acd1e16290fe9a8ec676499feec8ef7639/> extend cli wallet deposit for owned dbcs
   The CLI is now extended to support the deposit of owned DBCs.
   
   The `deposit` command will check if the supplied DBC is owned, and if it is, it will check to see if
   the `--secret-key` argument is present and use that. If that argument isn't present, it will attempt
   to use the secret key that's configured for use with the CLI, i.e., the `keys create --for-cli`
   command.
   
   The `reissue` command was also extended to provide an `--owned` flag, which when used, will reissue
   an owned DBC using the public key configured for use with the CLI. This argument is mutually
   exclusive with the `--public-key` argument, which will reissue the DBC using a specified key.
   
   So we could offer the user a suggestion when a supplied secret key didn't match, this also involved
   making a little extension to the API, to return a specific type of error. We will need to modify
   `sn_dbc` to return a specific error type for this too, so we can avoid checking the string content
   of the error message, but this will be covered on a separate PR.
 - <csr-id-69079d698a539a6fe42e87ab1603bbb41ac91f50/> extend cli wallet deposit for owned dbcs
   The CLI is now extended to support the deposit of owned DBCs.
   
   The `deposit` command will check if the supplied DBC is owned, and if it is, it will check to see if
   the `--secret-key` argument is present and use that. If that argument isn't present, it will attempt
   to use the secret key that's configured for use with the CLI, i.e., the `keys create --for-cli`
   command.
   
   The `reissue` command was also extended to provide an `--owned` flag, which when used, will reissue
   an owned DBC using the public key configured for use with the CLI. This argument is mutually
   exclusive with the `--public-key` argument, which will reissue the DBC using a specified key.
   
   So we could offer the user a suggestion when a supplied secret key didn't match, this also involved
   making a little extension to the API, to return a specific type of error. We will need to modify
   `sn_dbc` to return a specific error type for this too, so we can avoid checking the string content
   of the error message, but this will be covered on a separate PR.

### New Features (BREAKING)

 - <csr-id-5577695b5d3291c46cd475df8c0933a067b4cfc5/> serialize to bls keys in util functions
   Utility functions were recently added to the API for serializing to the `Keypair` type. This was
   changed to serialize directly to BLS to make it easier for the CLI to deal directly with BLS keys.
   Soon we will be refactoring the `Keypair` type to have a different use case and things like
   `sn_client` would be refactored to directly work with BLS keys. This is a little step in that
   direction.
   
   There was a utility function added to `sn_interface` to create a `Keypair` from a hex-based BLS key
   because we still need to use the `Keypair` at this point in time.
 - <csr-id-3e757bb626d71c03608a625fa435a312b8fc0beb/> extend wallet_deposit for owned dbcs
 - <csr-id-67006eb2e84b750a6b9b03d04aafdcfc85b38955/> serialize to bls keys in util functions
   Utility functions were recently added to the API for serializing to the `Keypair` type. This was
   changed to serialize directly to BLS to make it easier for the CLI to deal directly with BLS keys.
   Soon we will be refactoring the `Keypair` type to have a different use case and things like
   `sn_client` would be refactored to directly work with BLS keys. This is a little step in that
   direction.
   
   There was a utility function added to `sn_interface` to create a `Keypair` from a hex-based BLS key
   because we still need to use the `Keypair` at this point in time.
 - <csr-id-23802f8e357831b0166307934ca19658d9107039/> extend wallet_deposit for owned dbcs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 1 calendar day.
 - 5 days passed between releases.
 - 9 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.7.0/sn_dysfunction-0.6.0/sn_client-0.67.0/sn_node-0.63.0/sn_api-0.65.0/sn_cli-0.58.0 ([`243cfc4`](https://github.com/maidsafe/safe_network/commit/243cfc48a7f4a9b60b5b7f1fdd609c02197aba5e))
    - changes based on review feedback ([`3f3c39a`](https://github.com/maidsafe/safe_network/commit/3f3c39a14987910bb424df51f89d948333ca3e87))
    - extend cli wallet deposit for owned dbcs ([`341306a`](https://github.com/maidsafe/safe_network/commit/341306acd1e16290fe9a8ec676499feec8ef7639))
    - serialize to bls keys in util functions ([`5577695`](https://github.com/maidsafe/safe_network/commit/5577695b5d3291c46cd475df8c0933a067b4cfc5))
    - extend wallet_deposit for owned dbcs ([`3e757bb`](https://github.com/maidsafe/safe_network/commit/3e757bb626d71c03608a625fa435a312b8fc0beb))
    - changes based on review feedback ([`5ea4c3d`](https://github.com/maidsafe/safe_network/commit/5ea4c3d60bf84384ed37b5dde25ac4dc26147c24))
    - extend cli wallet deposit for owned dbcs ([`69079d6`](https://github.com/maidsafe/safe_network/commit/69079d698a539a6fe42e87ab1603bbb41ac91f50))
    - serialize to bls keys in util functions ([`67006eb`](https://github.com/maidsafe/safe_network/commit/67006eb2e84b750a6b9b03d04aafdcfc85b38955))
    - extend wallet_deposit for owned dbcs ([`23802f8`](https://github.com/maidsafe/safe_network/commit/23802f8e357831b0166307934ca19658d9107039))
</details>

## v0.64.4 (2022-06-21)

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

## v0.64.3 (2022-06-15)

<csr-id-4eb43fa884d7b047febb18c067ae905969a113bf/>
<csr-id-f599c5973d50324aad1720166156666d5db1ed3d/>

### Chore

 - <csr-id-4eb43fa884d7b047febb18c067ae905969a113bf/> upgrade blsttc to 6.0.0
   There were various other crates that had to be upgraded in this process:
   * secured_linked_list to v0.5.2 because it was also upgraded to reference v6.0.0 of blsttc
   * bls_dkg to v0.10.3 because it was also upgraded to reference v6.0.0 of blsttc
   * sn_consensus to v2.1.1 because it was also upgraded to reference v6.0.0 of blsttc
   * sn_dbc to v4.0.0 because it was also upgraded to reference v6.0.0 of blsttc

### Chore

 - <csr-id-f599c5973d50324aad1720166156666d5db1ed3d/> sn_interface-0.6.3/sn_dysfunction-0.5.1/sn_client-0.66.3/sn_api-0.64.3/sn_cli-0.57.4

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.3/sn_dysfunction-0.5.1/sn_client-0.66.3/sn_api-0.64.3/sn_cli-0.57.4 ([`f599c59`](https://github.com/maidsafe/safe_network/commit/f599c5973d50324aad1720166156666d5db1ed3d))
    - upgrade blsttc to 6.0.0 ([`4eb43fa`](https://github.com/maidsafe/safe_network/commit/4eb43fa884d7b047febb18c067ae905969a113bf))
</details>

## v0.64.2 (2022-06-15)

<csr-id-0f00c8cf7caae190716c8fd57addd38b18a3a49b/>
<csr-id-46246f155ab65f3fcd61381345f1a7f747dfe957/>

### New Features

 - <csr-id-1b1cb77df6c2805ecfa741bb824b359214558929/> remove private registers
 - <csr-id-f1829f99ef1415a83731f855757fbce9970fa4f0/> remove private data addresses
 - <csr-id-8be2f2c9efac1623ea95ff1641c6b9bc22fad455/> remove private safe key addresses

### Bug Fixes

 - <csr-id-426ad4a0c15d0ef3554ec098081c118759bf44fb/> retry_loop on wallet tests
 - <csr-id-d7bcc1012d81d2d73b35e59d636630fe77f532ab/> update test vectors now that private scope is gone
 - <csr-id-fcec8ffaaf7cfb827db5338428b38a7b29cc67af/> add retry loop to wallet tests
 - <csr-id-4cb31ffe40ac60ca4ce6233b7d61ddcc93d455a7/> hack: ignore private wallet test until encryption impl.
 - <csr-id-d6c7887631eab05a1f423d7a136feee814318329/> fix skipped byte in SafeUrl parsing without scope

### Refactor

 - <csr-id-0f00c8cf7caae190716c8fd57addd38b18a3a49b/> add from_safekey, from_register, from_bytes

### Chore

 - <csr-id-46246f155ab65f3fcd61381345f1a7f747dfe957/> sn_interface-0.6.2/sn_client-0.66.2/sn_node-0.62.4/sn_api-0.64.2/sn_cli-0.57.3

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 13 commits contributed to the release over the course of 1 calendar day.
 - 8 days passed between releases.
 - 10 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.2/sn_client-0.66.2/sn_node-0.62.4/sn_api-0.64.2/sn_cli-0.57.3 ([`46246f1`](https://github.com/maidsafe/safe_network/commit/46246f155ab65f3fcd61381345f1a7f747dfe957))
    - Merge remote-tracking branch 'origin/main' into drusu/remove-private-data ([`2057273`](https://github.com/maidsafe/safe_network/commit/2057273509c2488cafc7f6db2ae69a99efc3b350))
    - Merge branch 'main' into simplify_safeurl ([`a0175ab`](https://github.com/maidsafe/safe_network/commit/a0175abfa15e558e54fbb25dc3baf49343f040ac))
    - Merge branch 'main' into drusu/remove-private-data ([`0cd2007`](https://github.com/maidsafe/safe_network/commit/0cd2007e442086d6eb2a39ad1f452e590fad46a9))
    - add from_safekey, from_register, from_bytes ([`0f00c8c`](https://github.com/maidsafe/safe_network/commit/0f00c8cf7caae190716c8fd57addd38b18a3a49b))
    - retry_loop on wallet tests ([`426ad4a`](https://github.com/maidsafe/safe_network/commit/426ad4a0c15d0ef3554ec098081c118759bf44fb))
    - update test vectors now that private scope is gone ([`d7bcc10`](https://github.com/maidsafe/safe_network/commit/d7bcc1012d81d2d73b35e59d636630fe77f532ab))
    - add retry loop to wallet tests ([`fcec8ff`](https://github.com/maidsafe/safe_network/commit/fcec8ffaaf7cfb827db5338428b38a7b29cc67af))
    - hack: ignore private wallet test until encryption impl. ([`4cb31ff`](https://github.com/maidsafe/safe_network/commit/4cb31ffe40ac60ca4ce6233b7d61ddcc93d455a7))
    - fix skipped byte in SafeUrl parsing without scope ([`d6c7887`](https://github.com/maidsafe/safe_network/commit/d6c7887631eab05a1f423d7a136feee814318329))
    - remove private registers ([`1b1cb77`](https://github.com/maidsafe/safe_network/commit/1b1cb77df6c2805ecfa741bb824b359214558929))
    - remove private data addresses ([`f1829f9`](https://github.com/maidsafe/safe_network/commit/f1829f99ef1415a83731f855757fbce9970fa4f0))
    - remove private safe key addresses ([`8be2f2c`](https://github.com/maidsafe/safe_network/commit/8be2f2c9efac1623ea95ff1641c6b9bc22fad455))
</details>

## v0.64.1 (2022-06-07)

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

## v0.64.0 (2022-06-05)

<csr-id-1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9/>
<csr-id-c12e2269e3a537d96422bed96a4459a0add07deb/>
<csr-id-e548388c693cfb71b270cf9e370b2f9b463044c5/>

### Chore

 - <csr-id-1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9/> sn_interface-0.6.0/sn_dysfunction-0.5.0/sn_client-0.66.0/sn_node-0.62.0/sn_api-0.64.0/sn_cli-0.57.0
 - <csr-id-c12e2269e3a537d96422bed96a4459a0add07deb/> upgrade sn_dbc to 3.2.0
   This new release has utilities for serializing/deserializing `Dbc` to/from hex.
 - <csr-id-e548388c693cfb71b270cf9e370b2f9b463044c5/> upgrade sn_dbc to 3.2.0
   This new release has utilities for serializing/deserializing `Dbc` to/from hex.

### New Features

<csr-id-0e9980f5358a0aca5d40d607dfdc6de120e6412b/>
<csr-id-95de2ffe6f57ae0e6cebf123da3e9b6c3ad84aaf/>
<csr-id-1048c5e3d2196aed7de89a7938d6fc01c1843502/>

 - <csr-id-4c6e6cff474d306e6632f004c6cf05729c7ced16/> add public key argument for owned dbcs
   The `wallet reissue` command now has an additional optional argument, `--public-key`, which allows
   the user to reissue a DBC to be owned by the holder of that public key. The key should be BLS
   hex-encoded.
   
   The `wallet deposit` command will now require extension to provide the secret key when depositing an
   owned DBC. This will be done as a separate piece of work.
   
   Some additional changes were made in support or to tidy CLI-related code:
   * The conversion of DBCs to/from hex were removed from the CLI since this is now done on the `Dbc`
   type.
* A CLI test that existed to test the above conversion code was removed since it's no longer
     necessary.
* The naming scheme for the CLI wallet tests were elaborated and the redundant "calling_safe"
     prefixes were removed.
* The conversion of DBCs to/from hex were removed from the CLI since this is now done on the `Dbc`
     type.
* A CLI test that existed to test the above conversion code was removed since it's no longer
     necessary.
* The naming scheme for the CLI wallet tests were elaborated and the redundant "calling_safe"
     prefixes were removed.

### New Features (BREAKING)

 - <csr-id-92c53f186d2a63c6333b4d7b1016bb55edf74e42/> reissue dbc to a particular owner
 - <csr-id-cd85844f9f6402aba02f28fbedf92c7ee234e315/> reissue dbc to a particular owner
 - <csr-id-f03fb7e35319dbb9e4745e3cb36c7913c4f220ac/> cli will now use bls keys
 - <csr-id-48006b73547778bc08b077717e04fd5efb562eaf/> extend client with dbc owner field

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 13 commits contributed to the release over the course of 4 calendar days.
 - 8 days passed between releases.
 - 11 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.6.0/sn_dysfunction-0.5.0/sn_client-0.66.0/sn_node-0.62.0/sn_api-0.64.0/sn_cli-0.57.0 ([`1bf7dfb`](https://github.com/maidsafe/safe_network/commit/1bf7dfb3ce8b14cbed7a4a8ed98c8310653a2da9))
    - add public key argument for owned dbcs ([`4c6e6cf`](https://github.com/maidsafe/safe_network/commit/4c6e6cff474d306e6632f004c6cf05729c7ced16))
    - upgrade sn_dbc to 3.2.0 ([`c12e226`](https://github.com/maidsafe/safe_network/commit/c12e2269e3a537d96422bed96a4459a0add07deb))
    - reissue dbc to a particular owner ([`92c53f1`](https://github.com/maidsafe/safe_network/commit/92c53f186d2a63c6333b4d7b1016bb55edf74e42))
    - add public key argument for owned dbcs ([`0e9980f`](https://github.com/maidsafe/safe_network/commit/0e9980f5358a0aca5d40d607dfdc6de120e6412b))
    - upgrade sn_dbc to 3.2.0 ([`e548388`](https://github.com/maidsafe/safe_network/commit/e548388c693cfb71b270cf9e370b2f9b463044c5))
    - Merge branch 'main' into handover_byz_sap_check_squashed ([`6769996`](https://github.com/maidsafe/safe_network/commit/6769996e3ea78a6be306437193687b422a21ce80))
    - handover sap elder checks with membership knowledge ([`95de2ff`](https://github.com/maidsafe/safe_network/commit/95de2ffe6f57ae0e6cebf123da3e9b6c3ad84aaf))
    - reissue dbc to a particular owner ([`cd85844`](https://github.com/maidsafe/safe_network/commit/cd85844f9f6402aba02f28fbedf92c7ee234e315))
    - cli will now use bls keys ([`f03fb7e`](https://github.com/maidsafe/safe_network/commit/f03fb7e35319dbb9e4745e3cb36c7913c4f220ac))
    - use persistent dbc owner in sn_api ([`1048c5e`](https://github.com/maidsafe/safe_network/commit/1048c5e3d2196aed7de89a7938d6fc01c1843502))
    - extend client with dbc owner field ([`48006b7`](https://github.com/maidsafe/safe_network/commit/48006b73547778bc08b077717e04fd5efb562eaf))
    - Merge #1192 ([`f9fc2a7`](https://github.com/maidsafe/safe_network/commit/f9fc2a76f083ba5161c8c4eef9013c53586b4693))
</details>

## v0.63.0 (2022-05-27)

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

## v0.62.0 (2022-05-25)

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

## v0.61.0 (2022-05-21)

<csr-id-cf21d66b9b726123e0a4320cd68481b67f7af03d/>

### Chore

 - <csr-id-cf21d66b9b726123e0a4320cd68481b67f7af03d/> sn_dysfunction-0.2.0/sn_client-0.63.0/sn_node-0.59.0/sn_api-0.61.0/sn_cli-0.54.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 3 days passed between releases.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_dysfunction-0.2.0/sn_client-0.63.0/sn_node-0.59.0/sn_api-0.61.0/sn_cli-0.54.0 ([`cf21d66`](https://github.com/maidsafe/safe_network/commit/cf21d66b9b726123e0a4320cd68481b67f7af03d))
</details>

## v0.60.2 (2022-05-18)

<csr-id-07504faeda6cbfd0b27abea25facde992398ecf9/>
<csr-id-9b06304f46e1a1bda90a0fc6ff82edc928c2529d/>

### Chore

 - <csr-id-07504faeda6cbfd0b27abea25facde992398ecf9/> upgrade blsttc to v5.2.0 and rand to v0.8

### Chore

 - <csr-id-9b06304f46e1a1bda90a0fc6ff82edc928c2529d/> sn_interface-0.2.4/sn_client-0.62.3/sn_api-0.60.2/sn_cli-0.53.1

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 8 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.4/sn_client-0.62.3/sn_api-0.60.2/sn_cli-0.53.1 ([`9b06304`](https://github.com/maidsafe/safe_network/commit/9b06304f46e1a1bda90a0fc6ff82edc928c2529d))
    - upgrade blsttc to v5.2.0 and rand to v0.8 ([`07504fa`](https://github.com/maidsafe/safe_network/commit/07504faeda6cbfd0b27abea25facde992398ecf9))
</details>

## v0.60.1 (2022-05-10)

<csr-id-61ba367c308a846cb3f1ae065b1fbbdfb85838e4/>

### Chore

 - <csr-id-61ba367c308a846cb3f1ae065b1fbbdfb85838e4/> sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1

### Bug Fixes

 - <csr-id-ae4156228a4bb684ff10ac8c98917dd4dae434ea/> check Register permissions on ops locally to prevent failures when broadcasted to the network

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release.
 - 3 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.2/sn_client-0.62.2/sn_node-0.58.16/sn_api-0.60.1 ([`61ba367`](https://github.com/maidsafe/safe_network/commit/61ba367c308a846cb3f1ae065b1fbbdfb85838e4))
    - Merge #1171 ([`06b4433`](https://github.com/maidsafe/safe_network/commit/06b4433f199ba7c622ad57e767d80f58f0b50a69))
    - check Register permissions on ops locally to prevent failures when broadcasted to the network ([`ae41562`](https://github.com/maidsafe/safe_network/commit/ae4156228a4bb684ff10ac8c98917dd4dae434ea))
    - Merge #1140 ([`459b641`](https://github.com/maidsafe/safe_network/commit/459b641f22b488f33825777b974da80512eabed5))
    - Merge #1169 ([`e5d0c17`](https://github.com/maidsafe/safe_network/commit/e5d0c17c335a3a25ee0bb4c81906fa176abeb7f5))
</details>

## v0.60.0 (2022-05-06)

<csr-id-737d906a61f772593ac7df755d995d66059e8b5e/>

### Bug Fixes

 - <csr-id-ae4156228a4bb684ff10ac8c98917dd4dae434ea/> check Register permissions on ops locally to prevent failures when broadcasted to the network

### Chore

 - <csr-id-737d906a61f772593ac7df755d995d66059e8b5e/> sn_interface-0.2.1/sn_client-0.62.1/sn_node-0.58.15/sn_api-0.60.0/sn_cli-0.53.0

### New Features (BREAKING)

 - <csr-id-681457a75e818beb30401154f336383507acd935/> return a Token value from wallet balance API instead of a string
   - Additionally add support to the cat and dog commands for Wallets.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 11 calendar days.
 - 13 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.1/sn_client-0.62.1/sn_node-0.58.15/sn_api-0.60.0/sn_cli-0.53.0 ([`737d906`](https://github.com/maidsafe/safe_network/commit/737d906a61f772593ac7df755d995d66059e8b5e))
    - return a Token value from wallet balance API instead of a string ([`681457a`](https://github.com/maidsafe/safe_network/commit/681457a75e818beb30401154f336383507acd935))
    - Merge #1128 ([`e49d382`](https://github.com/maidsafe/safe_network/commit/e49d38239b3a8c468616ad3782e1208316e9b5e0))
</details>

## v0.59.0 (2022-04-23)

<csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/>
<csr-id-e3dca8b07441c86744b091fe883d16a9c750f702/>
<csr-id-ad7d340720f0737f502b0d55023a15461dded91d/>
<csr-id-88421d9cb7872b6397283a0035130bc14de6d4ff/>
<csr-id-d0913293df42e73716fabb3460ae6dbd4ddf5a1b/>
<csr-id-91606f631a211d959364cab1e428d1ac895d3dca/>
<csr-id-bda0ea00e2e5a258e02a91d12dcd1e480dfff17c/>
<csr-id-2f4e7e6305ba387f2e28945aee71df650ac1d3eb/>

### Chore

 - <csr-id-318ee1d22970b5f06e93a99b6e8fff6da638c589/> tidy references in cargo manifests
   All references are organised alphabetically, and random usage of long-form references are removed in
   favour of the short-form version, unless the long-form style is justified, e.g., when lots of
   features are being used.
 - <csr-id-e3dca8b07441c86744b091fe883d16a9c750f702/> set sn_client version to be new release
   previously sn_client was its own repo and crate, we havent published under this name in some time. This will bring us back into this namespace ad on crates.io, but at a new updated version
 - <csr-id-ad7d340720f0737f502b0d55023a15461dded91d/> update sn_cli and api readme for sn_client extraction
 - <csr-id-88421d9cb7872b6397283a0035130bc14de6d4ff/> pull sn_client out of the node codebase
 - <csr-id-d0913293df42e73716fabb3460ae6dbd4ddf5a1b/> update proptest dep to v1 in sn_api

### Chore

 - <csr-id-2f4e7e6305ba387f2e28945aee71df650ac1d3eb/> sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0

### New Features

<csr-id-adb085e98b00ec6cd0d670bf665009d6e93e2514/>

 - <csr-id-e662317e93b3247a1afd9970587ea7241a9b5619/> first and basic implementation of Wallet reissue API and related CLI cmds
   - Generate output DBCs with sn_dbc::TransactionBuilder

### Other

 - <csr-id-91606f631a211d959364cab1e428d1ac895d3dca/> additional wallet API test cases
 - <csr-id-bda0ea00e2e5a258e02a91d12dcd1e480dfff17c/> additional wallet API test cases

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release over the course of 7 calendar days.
 - 8 days passed between releases.
 - 10 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.2.0/sn_dysfunction-0.1.2/sn_api-0.59.0/sn_cli-0.52.0 ([`2f4e7e6`](https://github.com/maidsafe/safe_network/commit/2f4e7e6305ba387f2e28945aee71df650ac1d3eb))
    - tidy references in cargo manifests ([`318ee1d`](https://github.com/maidsafe/safe_network/commit/318ee1d22970b5f06e93a99b6e8fff6da638c589))
    - additional wallet API test cases ([`91606f6`](https://github.com/maidsafe/safe_network/commit/91606f631a211d959364cab1e428d1ac895d3dca))
    - first and basic implementation of Wallet reissue API and related CLI cmds ([`e662317`](https://github.com/maidsafe/safe_network/commit/e662317e93b3247a1afd9970587ea7241a9b5619))
    - additional wallet API test cases ([`bda0ea0`](https://github.com/maidsafe/safe_network/commit/bda0ea00e2e5a258e02a91d12dcd1e480dfff17c))
    - first and basic implementation of Wallet reissue API and related CLI cmds ([`adb085e`](https://github.com/maidsafe/safe_network/commit/adb085e98b00ec6cd0d670bf665009d6e93e2514))
    - set sn_client version to be new release ([`e3dca8b`](https://github.com/maidsafe/safe_network/commit/e3dca8b07441c86744b091fe883d16a9c750f702))
    - update sn_cli and api readme for sn_client extraction ([`ad7d340`](https://github.com/maidsafe/safe_network/commit/ad7d340720f0737f502b0d55023a15461dded91d))
    - pull sn_client out of the node codebase ([`88421d9`](https://github.com/maidsafe/safe_network/commit/88421d9cb7872b6397283a0035130bc14de6d4ff))
    - update proptest dep to v1 in sn_api ([`d091329`](https://github.com/maidsafe/safe_network/commit/d0913293df42e73716fabb3460ae6dbd4ddf5a1b))
</details>

## v0.58.2 (2022-04-14)

<csr-id-8494a01d9be3dddc0d0f4c2c37cdc4d6c3e54521/>
<csr-id-b62ad80298eb4b3e2f9810d20dd553aaf802408b/>
<csr-id-2c557b5d5b5e21882ea3bf1cf904103576363603/>
<csr-id-86ce41ca31508dbaf2de56fc81e1ca3146f863dc/>
<csr-id-9ea06ffe9339d3927897f010314b1be1bf7026bf/>

### Chore

 - <csr-id-8494a01d9be3dddc0d0f4c2c37cdc4d6c3e54521/> split put messaging and types into top level crate

### Chore

 - <csr-id-9ea06ffe9339d3927897f010314b1be1bf7026bf/> sn_dysfunction-0.1.1/safe_network-0.58.13/sn_api-0.58.2/sn_cli-0.51.3

### New Features

 - <csr-id-842c77a5fe1c4f13e9a9f37b3b5dea974c0f5a82/> adding first set of basic wallet APIs and CLI commands

### Other

 - <csr-id-b62ad80298eb4b3e2f9810d20dd553aaf802408b/> add test-utils feat to bench

### Test

 - <csr-id-2c557b5d5b5e21882ea3bf1cf904103576363603/> adding CLI tests for Wallet commands
 - <csr-id-86ce41ca31508dbaf2de56fc81e1ca3146f863dc/> adding more unit tests to wallet APIs

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
    - sn_dysfunction-0.1.1/safe_network-0.58.13/sn_api-0.58.2/sn_cli-0.51.3 ([`9ea06ff`](https://github.com/maidsafe/safe_network/commit/9ea06ffe9339d3927897f010314b1be1bf7026bf))
    - add test-utils feat to bench ([`b62ad80`](https://github.com/maidsafe/safe_network/commit/b62ad80298eb4b3e2f9810d20dd553aaf802408b))
    - split put messaging and types into top level crate ([`8494a01`](https://github.com/maidsafe/safe_network/commit/8494a01d9be3dddc0d0f4c2c37cdc4d6c3e54521))
    - adding CLI tests for Wallet commands ([`2c557b5`](https://github.com/maidsafe/safe_network/commit/2c557b5d5b5e21882ea3bf1cf904103576363603))
    - adding more unit tests to wallet APIs ([`86ce41c`](https://github.com/maidsafe/safe_network/commit/86ce41ca31508dbaf2de56fc81e1ca3146f863dc))
    - adding first set of basic wallet APIs and CLI commands ([`842c77a`](https://github.com/maidsafe/safe_network/commit/842c77a5fe1c4f13e9a9f37b3b5dea974c0f5a82))
</details>

## v0.58.1 (2022-04-09)

<csr-id-c4e3de1d9715c6e3618a763fa857feca4258248f/>

### Chore

 - <csr-id-c4e3de1d9715c6e3618a763fa857feca4258248f/> safe_network-0.58.12/sn_api-0.58.1/sn_cli-0.51.2

### Bug Fixes

 - <csr-id-4303aec7813f235234022be43e2b3adb4528da57/> files API to use the Scope encoded in the input Urls

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 2 calendar days.
 - 14 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.58.12/sn_api-0.58.1/sn_cli-0.51.2 ([`c4e3de1`](https://github.com/maidsafe/safe_network/commit/c4e3de1d9715c6e3618a763fa857feca4258248f))
    - files API to use the Scope encoded in the input Urls ([`4303aec`](https://github.com/maidsafe/safe_network/commit/4303aec7813f235234022be43e2b3adb4528da57))
</details>

## v0.58.0 (2022-03-25)

<csr-id-907c7d3ef4f65df5566627938154dfca1e2fdc05/>
<csr-id-90712c91368b4d88537acc65a3ccc5478fe38d2c/>
<csr-id-6e897d0bc93256f5ab72350c9774f9a33937da1b/>
<csr-id-453b246c002f9e964896876c254e6c31f1f6045d/>
<csr-id-6b83f38f17c241c00b70480a18a47b04d9a51ee1/>

### Chore

 - <csr-id-907c7d3ef4f65df5566627938154dfca1e2fdc05/> safe_network-0.58.8/sn_api-0.58.0/sn_cli-0.51.0
 - <csr-id-90712c91368b4d88537acc65a3ccc5478fe38d2c/> update deps
 - <csr-id-6e897d0bc93256f5ab72350c9774f9a33937da1b/> remove retry_loop! from nrs tests
 - <csr-id-453b246c002f9e964896876c254e6c31f1f6045d/> refactor NodeQueryResponse handling at elder
 - <csr-id-6b83f38f17c241c00b70480a18a47b04d9a51ee1/> deps, remove ~ restriction on major versioned deps
   tilde w/ a major version restricts us to path udpats only.
   we want caret, which is implicit frm v 1

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

### New Features (BREAKING)

 - <csr-id-eaeca4223c4e35884bfd1129832b890e70b6ef5e/> upgrade to new version of qjsonrpc
   The `sn_api` crate is updated to use the new version of `qjsonrpc`.
   
   The `qjsonrpc` library had breaking changes to pass cert and key paths rather than passing a
   directory that was assumed to contain them. This change caused `sn_api` to do the same. There is an
   unfortunate consequence here: to use `SafeAuthdClient` you need to pass a certificate, along with
   another certificate and a private key for starting an endpoint that deals with notifications for
   approving authentication requests. The previous setup was just making the assumption that both keys
   and certificates were in the same directory. Since this change led to an odd interface for
   `SafeAuthdClient` we had some discussion around using a different mechanism for notifications, but
   we decided to come back to it later with a wider review.
   
   The API change forced an update to some code in the CLI, even though the authd system isn't really
   being used at the moment. The self-signed certificates are now being generated in the CLI. Since the
   CLI is a specific application, I think this is a more appropriate place for that to happen.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 1 calendar day.
 - 3 days passed between releases.
 - 7 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.58.8/sn_api-0.58.0/sn_cli-0.51.0 ([`907c7d3`](https://github.com/maidsafe/safe_network/commit/907c7d3ef4f65df5566627938154dfca1e2fdc05))
    - update deps ([`90712c9`](https://github.com/maidsafe/safe_network/commit/90712c91368b4d88537acc65a3ccc5478fe38d2c))
    - Improve query handling, keep peers on DataNotFound ([`a45a3bd`](https://github.com/maidsafe/safe_network/commit/a45a3bda7044f07b6ecd99569ec4c043330d7160))
    - remove retry_loop! from nrs tests ([`6e897d0`](https://github.com/maidsafe/safe_network/commit/6e897d0bc93256f5ab72350c9774f9a33937da1b))
    - refactor NodeQueryResponse handling at elder ([`453b246`](https://github.com/maidsafe/safe_network/commit/453b246c002f9e964896876c254e6c31f1f6045d))
    - deps, remove ~ restriction on major versioned deps ([`6b83f38`](https://github.com/maidsafe/safe_network/commit/6b83f38f17c241c00b70480a18a47b04d9a51ee1))
    - upgrade to new version of qjsonrpc ([`eaeca42`](https://github.com/maidsafe/safe_network/commit/eaeca4223c4e35884bfd1129832b890e70b6ef5e))
</details>

## v0.57.3 (2022-03-22)

<csr-id-a6e2e0c5eec5c2e88842d18167128991b76ecbe8/>
<csr-id-d3989bdd95129999996e58736ec2553242697f2c/>

### Chore

 - <csr-id-a6e2e0c5eec5c2e88842d18167128991b76ecbe8/> safe_network-0.58.7/sn_api-0.57.3/sn_cli-0.50.5
 - <csr-id-d3989bdd95129999996e58736ec2553242697f2c/> bump bls_dkg, self_encryption, xor_name
   This is a step towards integrating sn_dbc into safe_network.

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
    - safe_network-0.58.7/sn_api-0.57.3/sn_cli-0.50.5 ([`a6e2e0c`](https://github.com/maidsafe/safe_network/commit/a6e2e0c5eec5c2e88842d18167128991b76ecbe8))
    - bump bls_dkg, self_encryption, xor_name ([`d3989bd`](https://github.com/maidsafe/safe_network/commit/d3989bdd95129999996e58736ec2553242697f2c))
</details>

## v0.57.2 (2022-03-17)

<csr-id-a741d930b906054d09f1311ddcf35479aa1aa3ee/>
<csr-id-6ca81812df56858c789353383e018fcee8b4c297/>

### Chore

 - <csr-id-a741d930b906054d09f1311ddcf35479aa1aa3ee/> safe_network-0.58.6/sn_api-0.57.2
 - <csr-id-6ca81812df56858c789353383e018fcee8b4c297/> tidy up some new test/clippy issues

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release over the course of 5 calendar days.
 - 17 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.58.6/sn_api-0.57.2 ([`a741d93`](https://github.com/maidsafe/safe_network/commit/a741d930b906054d09f1311ddcf35479aa1aa3ee))
    - tidy up some new test/clippy issues ([`6ca8181`](https://github.com/maidsafe/safe_network/commit/6ca81812df56858c789353383e018fcee8b4c297))
</details>

## v0.57.1 (2022-02-27)

<csr-id-634010fd79ce1487abbff5adf3d15da59709dd95/>
<csr-id-705995ef67b3d4c45c95689c4a675e1063467ec9/>
<csr-id-f95ef3d1cdc5d588e99c343470b8f45aedda70b4/>
<csr-id-d5e6f462615de830cd9c27dba49a34ba2da13b81/>
<csr-id-7b4672dfb6ae305221018e7eab090deabe4d6739/>

### Chore

 - <csr-id-634010fd79ce1487abbff5adf3d15da59709dd95/> safe_network-0.58.2/sn_api-0.57.1/sn_cli-0.50.2
 - <csr-id-705995ef67b3d4c45c95689c4a675e1063467ec9/> changes to appease clippy 1.59
 - <csr-id-f95ef3d1cdc5d588e99c343470b8f45aedda70b4/> more dep updates
 - <csr-id-d5e6f462615de830cd9c27dba49a34ba2da13b81/> more general dep updates
 - <csr-id-7b4672dfb6ae305221018e7eab090deabe4d6739/> update multibase to be inline across codebase

### Bug Fixes

 - <csr-id-38fb057da44a0e243186410df0c39361a21ec46e/> introduce cohesive conn handling

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 2 calendar days.
 - 9 days passed between releases.
 - 6 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.58.2/sn_api-0.57.1/sn_cli-0.50.2 ([`634010f`](https://github.com/maidsafe/safe_network/commit/634010fd79ce1487abbff5adf3d15da59709dd95))
    - Merge #1038 #1042 ([`08da844`](https://github.com/maidsafe/safe_network/commit/08da8440f9acd2eb8b2494ca7d1c2f8f3f8f631e))
    - more general dep updates ([`d5e6f46`](https://github.com/maidsafe/safe_network/commit/d5e6f462615de830cd9c27dba49a34ba2da13b81))
    - update multibase to be inline across codebase ([`7b4672d`](https://github.com/maidsafe/safe_network/commit/7b4672dfb6ae305221018e7eab090deabe4d6739))
    - changes to appease clippy 1.59 ([`705995e`](https://github.com/maidsafe/safe_network/commit/705995ef67b3d4c45c95689c4a675e1063467ec9))
    - introduce cohesive conn handling ([`38fb057`](https://github.com/maidsafe/safe_network/commit/38fb057da44a0e243186410df0c39361a21ec46e))
    - more dep updates ([`f95ef3d`](https://github.com/maidsafe/safe_network/commit/f95ef3d1cdc5d588e99c343470b8f45aedda70b4))
</details>

## v0.57.0 (2022-02-17)

<csr-id-149665a53c00f62be0e8c8ec340b951a06346848/>

### Chore

 - <csr-id-149665a53c00f62be0e8c8ec340b951a06346848/> safe_network-0.58.0/sn_api-0.57.0/sn_cli-0.50.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 5 days passed between releases.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.58.0/sn_api-0.57.0/sn_cli-0.50.0 ([`149665a`](https://github.com/maidsafe/safe_network/commit/149665a53c00f62be0e8c8ec340b951a06346848))
</details>

## v0.56.0 (2022-02-12)

<csr-id-f558b5c60df64dd349158a327bec945321937cf3/>
<csr-id-b9ceb229091ca29f5dcc675d66a0d9ff46a60427/>
<csr-id-a398c4f8d72828db0fc8c6d5825ead62ba85db64/>

### Refactor

 - <csr-id-f558b5c60df64dd349158a327bec945321937cf3/> make nrs url validation private
   We made URL validation public to share code during NRS resolution, but there's no need for
   validation during resolution because the URLs have already been validated at the point of
   association.
   
   Also fix some clippy warnings.

### Other

 - <csr-id-b9ceb229091ca29f5dcc675d66a0d9ff46a60427/> update nrs tests to use helper
   The helper was moved into the test_helpers module and renamed to TestDataFilesContainer, since this
   seems more accurate.
   
   A test was added for retrieving a topname link. This is arguably covered by the `nrs_associate`
   tests, but I think it's good to have this test to call it out as functionality we don't want to
   break.

### Chore

 - <csr-id-a398c4f8d72828db0fc8c6d5825ead62ba85db64/> safe_network-0.57.0/sn_api-0.56.0/sn_cli-0.49.0

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
 - <csr-id-ced0f6631ac5e8d24546841ba15b8cb7b43929c4/> retrieve subname version
   The `nrs_get` function is changed to make use of the version argument to retrieve a subname at a
   specific version.
   
   A helper struct was created to reduce test verbosity. It uploads a file container and stores file
   path/URL pairs in a hash table.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 3 calendar days.
 - 3 days passed between releases.
 - 5 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.57.0/sn_api-0.56.0/sn_cli-0.49.0 ([`a398c4f`](https://github.com/maidsafe/safe_network/commit/a398c4f8d72828db0fc8c6d5825ead62ba85db64))
    - Merge branch 'main' into resolve_nrs_map_container_content ([`1631737`](https://github.com/maidsafe/safe_network/commit/1631737769f0d1a3cd2740af6d835479daafe1a7))
    - make nrs url validation private ([`f558b5c`](https://github.com/maidsafe/safe_network/commit/f558b5c60df64dd349158a327bec945321937cf3))
    - resolve nrs map container content ([`0bc50ae`](https://github.com/maidsafe/safe_network/commit/0bc50ae33ccb934016ac425e7bb2eca90a4b06e3))
    - update nrs tests to use helper ([`b9ceb22`](https://github.com/maidsafe/safe_network/commit/b9ceb229091ca29f5dcc675d66a0d9ff46a60427))
    - retrieve subname version ([`ced0f66`](https://github.com/maidsafe/safe_network/commit/ced0f6631ac5e8d24546841ba15b8cb7b43929c4))
    - Merge #995 ([`5176b3a`](https://github.com/maidsafe/safe_network/commit/5176b3a72e2f5f3f1dfc21116a6bf3ffa3893830))
</details>

## v0.55.0 (2022-02-08)

<csr-id-3f75bf8da770a6167c396080b3ad8b54cfeb27e2/>
<csr-id-471d910f2b6d8952569c3dc4b2dd31fe7aa30dfa/>

### Chore

 - <csr-id-3f75bf8da770a6167c396080b3ad8b54cfeb27e2/> safe_network-0.56.0/sn_api-0.55.0/sn_cli-0.48.0
 - <csr-id-471d910f2b6d8952569c3dc4b2dd31fe7aa30dfa/> improve acronym consistency

### Bug Fixes

 - <csr-id-e867b1f5aa290823e77eff95f0846f00d7c0416c/> CLI shell was creating a new Safe API instance, and connecting to the net, for every command

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 4 days passed between releases.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.56.0/sn_api-0.55.0/sn_cli-0.48.0 ([`3f75bf8`](https://github.com/maidsafe/safe_network/commit/3f75bf8da770a6167c396080b3ad8b54cfeb27e2))
    - Merge branch 'main' into fix-cli-shell-api-instances ([`5fe7e54`](https://github.com/maidsafe/safe_network/commit/5fe7e54874e5d665fd10906c4c973f24d613aeba))
    - CLI shell was creating a new Safe API instance, and connecting to the net, for every command ([`e867b1f`](https://github.com/maidsafe/safe_network/commit/e867b1f5aa290823e77eff95f0846f00d7c0416c))
    - improve acronym consistency ([`471d910`](https://github.com/maidsafe/safe_network/commit/471d910f2b6d8952569c3dc4b2dd31fe7aa30dfa))
</details>

## v0.54.1 (2022-02-04)

<csr-id-86975f228f31303597a707e158005e44c86de1cc/>
<csr-id-9af70e7785c9329d8262de99bda68c4ad79d5154/>
<csr-id-58bf678793cbf474751c7bccc80e08fe3cd2d192/>
<csr-id-a58f6c5019e73ffbfa0f29965aa0fa62b026ece7/>
<csr-id-effc6fa5a035f8d88b7537eca304e4b0d6de29a3/>

### Chore

 - <csr-id-86975f228f31303597a707e158005e44c86de1cc/> safe_network-0.55.3/sn_api-0.54.1/sn_cli-0.47.1

### Refactor

 - <csr-id-9af70e7785c9329d8262de99bda68c4ad79d5154/> remove get_target_url function
   This function contained an unnecessary check to see if the `safe://` prefix wasn't present in top
   name or public names, which we wouldn't expect it to be. I'm not sure what I was thinking with this.
   
   Also made some more readability changes based on PR feedback.
 - <csr-id-58bf678793cbf474751c7bccc80e08fe3cd2d192/> subname -> public name
   Since we're now storing the full public name in the NRS map, the key should be referred to as the
   public name rather than the subname. Error messages were also updated.
 - <csr-id-a58f6c5019e73ffbfa0f29965aa0fa62b026ece7/> remove url sanitisation from api
   The `Safe::parse_url` function was removed from the API. This function 'sanitised' a URL by applying
   a `safe://` prefix to a URL string if the caller hadn't specified it.
   
   Initially, it was done to tidy up NRS code that was calling this function, but the same code was
   also calling a private function `parse_url` was making use of, so effectively the code was being
   called twice. More generally, we decided callers of the API should be responsible for passing a
   valid URL.
   
   The function was being called by various other parts of the API and also in the CLI, so these were
   changed to call `SafeUrl::from_url` directly.
   
   Some code was added to CLI commands to apply the `safe://` prefix if the user omitted it, so no
   functionality  was broken. A few test cases were added to cover it. A couple of NRS test cases for
   validating URLs were also removed as they no longer apply. This behaviour may actually have been
   incorrect in the first place.
   
   Also apply various clippy fixes.

### Other

 - <csr-id-effc6fa5a035f8d88b7537eca304e4b0d6de29a3/> rework nrs tests and provide more coverage
   Added more coverage for code paths that weren't being tested:
   * `nrs_create` duplicate topname
   * `nrs_associate` with non-versioned files container link
   * `nrs_associate` with non-versioned NRS map container link
   * `nrs_associate` with register link
   * `nrs_associate` with invalid URL
   
   Some NRS tests were reworked as follows:
   * Make them single purpose. Some tests were testing a few things, which makes it a bit harder to
     identify what's failed.
   * Trim down. Some tests had assertions and setup that didn't seem quite relevant, like checking and
     assigning versions.
   * Test `nrs_associate` without calling `nrs_add`. This muddies the water a little bit because
     `nrs_add` calls associate.
   * Separate `nrs_associate` tests for topnames and subnames.

### New Features

 - <csr-id-1115093a4cb4a0c7ed9f8d2b846aa435a7026b2e/> store full public name in nrs map
   Entries will now use the full public name reference, rather than the just subname. Like so:
   * example -> link
* a.example -> link
* b.example -> link
* a.b.example -> link
 - <csr-id-b2b0520630774d935aca1f2b602a1de9479ba6f9/> enable cmd retries
   Previously a command error would simply error out and fail.
   Now we use an exponential backoff to retry incase errors
   can be overcome

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 2 calendar days.
 - 2 days passed between releases.
 - 7 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.55.3/sn_api-0.54.1/sn_cli-0.47.1 ([`86975f2`](https://github.com/maidsafe/safe_network/commit/86975f228f31303597a707e158005e44c86de1cc))
    - Merge #993 ([`303e856`](https://github.com/maidsafe/safe_network/commit/303e856346dd1d4e5544c9ceae6d571c54cfb84e))
    - remove get_target_url function ([`9af70e7`](https://github.com/maidsafe/safe_network/commit/9af70e7785c9329d8262de99bda68c4ad79d5154))
    - subname -> public name ([`58bf678`](https://github.com/maidsafe/safe_network/commit/58bf678793cbf474751c7bccc80e08fe3cd2d192))
    - remove url sanitisation from api ([`a58f6c5`](https://github.com/maidsafe/safe_network/commit/a58f6c5019e73ffbfa0f29965aa0fa62b026ece7))
    - rework nrs tests and provide more coverage ([`effc6fa`](https://github.com/maidsafe/safe_network/commit/effc6fa5a035f8d88b7537eca304e4b0d6de29a3))
    - store full public name in nrs map ([`1115093`](https://github.com/maidsafe/safe_network/commit/1115093a4cb4a0c7ed9f8d2b846aa435a7026b2e))
    - enable cmd retries ([`b2b0520`](https://github.com/maidsafe/safe_network/commit/b2b0520630774d935aca1f2b602a1de9479ba6f9))
    - Merge #985 ([`ba572d5`](https://github.com/maidsafe/safe_network/commit/ba572d5f909f5c1dc389b9affadffec39a4e0369))
</details>

## v0.54.0 (2022-02-01)

<csr-id-889e0d99a6f096329e875c812a29ec165e61f5ae/>
<csr-id-2ec86e28246031084d603768ffa1fddf320a10a2/>

### Refactor

 - <csr-id-889e0d99a6f096329e875c812a29ec165e61f5ae/> nrs map fetch and rename multimap
   The `MultimapKeyValues` type alias was renamed to `Multimap` to make it easier to work with. Some
   functions were also renamed to the same effect, e.g., `fetch_multimap_values` -> `fetch_multimap`.
   
   The function for retrieving the NRS map was also refactored. Some aspects were broken down into
   smaller functions to reduce some complex if/else cases. Things were also setup for the version in
   the last entry in the Multimap to be collected. This is going to be used with the returned
   NrsMapContainer, to facilitate returning links to immutable content, which doesn't have a version.
   Currently the code isn't setup to support that scenario.

### Chore

 - <csr-id-2ec86e28246031084d603768ffa1fddf320a10a2/> safe_network-0.55.1/sn_api-0.54.0/sn_cli-0.47.0

### New Features

 - <csr-id-b2b0520630774d935aca1f2b602a1de9479ba6f9/> enable cmd retries
   Previously a command error would simply error out and fail.
   Now we use an exponential backoff to retry incase errors
   can be overcome
 - <csr-id-3d73dd03a7a6913a248e5cca7d714f8b8e4c0d01/> retrieve immutable content via nrs
   Changes the resolver to enable retrieval of NRS links to immutable content. Previously, the
   NRS `target_url` was incorrectly being checked to see if it contained a version. We now validate
   this URL using the same process used by NRS, which won't assert that a file link has a version. To
   reduce duplication, the `validate_nrs_url` function in the NRS module was changed to public.
   
   Tests were added to both the API and CLI to cover the scenario.
   
   The NrsMap struct was extended to include a subname_version field. This is going to be used to
   request the map with a particular version of a subname. If no version was specified when the
   container was retrieved, then the field won't be set. This is why it's an Option. Since we're
   storing this field on the NrsMap, the `version` field on the `NrsMapContainer` was removed.
   
   A couple of CLI NRS tests were also re-enabled, one of which happened to be related to immutable
   content.

### Bug Fixes (BREAKING)

 - <csr-id-e0885987742226f72ed761e7b78b86e2fa72e256/> dry-runner was making a connection to the network
   - Removing unnecessary mutability in many Safe API.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 3 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.55.1/sn_api-0.54.0/sn_cli-0.47.0 ([`2ec86e2`](https://github.com/maidsafe/safe_network/commit/2ec86e28246031084d603768ffa1fddf320a10a2))
    - dry-runner was making a connection to the network ([`e088598`](https://github.com/maidsafe/safe_network/commit/e0885987742226f72ed761e7b78b86e2fa72e256))
</details>

## v0.53.0 (2022-01-28)

<csr-id-9af70e7785c9329d8262de99bda68c4ad79d5154/>
<csr-id-58bf678793cbf474751c7bccc80e08fe3cd2d192/>
<csr-id-a58f6c5019e73ffbfa0f29965aa0fa62b026ece7/>
<csr-id-effc6fa5a035f8d88b7537eca304e4b0d6de29a3/>
<csr-id-366eee25f4b982d5a20d90168368a1aa14aa3181/>
<csr-id-889e0d99a6f096329e875c812a29ec165e61f5ae/>

### Refactor

 - <csr-id-9af70e7785c9329d8262de99bda68c4ad79d5154/> remove get_target_url function
   This function contained an unnecessary check to see if the `safe://` prefix wasn't present in top
   name or public names, which we wouldn't expect it to be. I'm not sure what I was thinking with this.
   
   Also made some more readability changes based on PR feedback.
 - <csr-id-58bf678793cbf474751c7bccc80e08fe3cd2d192/> subname -> public name
   Since we're now storing the full public name in the NRS map, the key should be referred to as the
   public name rather than the subname. Error messages were also updated.
 - <csr-id-a58f6c5019e73ffbfa0f29965aa0fa62b026ece7/> remove url sanitisation from api
   The `Safe::parse_url` function was removed from the API. This function 'sanitised' a URL by applying
   a `safe://` prefix to a URL string if the caller hadn't specified it.
   
   Initially, it was done to tidy up NRS code that was calling this function, but the same code was
   also calling a private function `parse_url` was making use of, so effectively the code was being
   called twice. More generally, we decided callers of the API should be responsible for passing a
   valid URL.
   
   The function was being called by various other parts of the API and also in the CLI, so these were
   changed to call `SafeUrl::from_url` directly.
   
   Some code was added to CLI commands to apply the `safe://` prefix if the user omitted it, so no
   functionality  was broken. A few test cases were added to cover it. A couple of NRS test cases for
   validating URLs were also removed as they no longer apply. This behaviour may actually have been
   incorrect in the first place.
   
   Also apply various clippy fixes.

### Refactor

 - <csr-id-889e0d99a6f096329e875c812a29ec165e61f5ae/> nrs map fetch and rename multimap
   The `MultimapKeyValues` type alias was renamed to `Multimap` to make it easier to work with. Some
   functions were also renamed to the same effect, e.g., `fetch_multimap_values` -> `fetch_multimap`.
   
   The function for retrieving the NRS map was also refactored. Some aspects were broken down into
   smaller functions to reduce some complex if/else cases. Things were also setup for the version in
   the last entry in the Multimap to be collected. This is going to be used with the returned
   NrsMapContainer, to facilitate returning links to immutable content, which doesn't have a version.
   Currently the code isn't setup to support that scenario.

### Other

 - <csr-id-effc6fa5a035f8d88b7537eca304e4b0d6de29a3/> rework nrs tests and provide more coverage
   Added more coverage for code paths that weren't being tested:
   * `nrs_create` duplicate topname
   * `nrs_associate` with non-versioned files container link
   * `nrs_associate` with non-versioned NRS map container link
   * `nrs_associate` with register link
   * `nrs_associate` with invalid URL
   
   Some NRS tests were reworked as follows:
   * Make them single purpose. Some tests were testing a few things, which makes it a bit harder to
     identify what's failed.
   * Trim down. Some tests had assertions and setup that didn't seem quite relevant, like checking and
     assigning versions.
   * Test `nrs_associate` without calling `nrs_add`. This muddies the water a little bit because
     `nrs_add` calls associate.
   * Separate `nrs_associate` tests for topnames and subnames.

### Chore

 - <csr-id-366eee25f4b982d5a20d90168368a1aa14aa3181/> safe_network-0.55.0/sn_api-0.53.0/sn_cli-0.46.0

### New Features

 - <csr-id-3d73dd03a7a6913a248e5cca7d714f8b8e4c0d01/> retrieve immutable content via nrs
   Changes the resolver to enable retrieval of NRS links to immutable content. Previously, the
   NRS `target_url` was incorrectly being checked to see if it contained a version. We now validate
   this URL using the same process used by NRS, which won't assert that a file link has a version. To
   reduce duplication, the `validate_nrs_url` function in the NRS module was changed to public.
   
   Tests were added to both the API and CLI to cover the scenario.
   
   The NrsMap struct was extended to include a subname_version field. This is going to be used to
   request the map with a particular version of a subname. If no version was specified when the
   container was retrieved, then the field won't be set. This is why it's an Option. Since we're
   storing this field on the NrsMap, the `version` field on the `NrsMapContainer` was removed.
   
   A couple of CLI NRS tests were also re-enabled, one of which happened to be related to immutable
   content.
 - <csr-id-1115093a4cb4a0c7ed9f8d2b846aa435a7026b2e/> store full public name in nrs map
   Entries will now use the full public name reference, rather than the just subname. Like so:
   * example -> link
* a.example -> link
* b.example -> link
* a.b.example -> link

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 2 calendar days.
 - 6 days passed between releases.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.55.0/sn_api-0.53.0/sn_cli-0.46.0 ([`366eee2`](https://github.com/maidsafe/safe_network/commit/366eee25f4b982d5a20d90168368a1aa14aa3181))
    - Merge branch 'main' into nrs_resolve_immutable_content ([`099bf22`](https://github.com/maidsafe/safe_network/commit/099bf224714e667bf998de80099eeeabfd869d8b))
    - retrieve immutable content via nrs ([`3d73dd0`](https://github.com/maidsafe/safe_network/commit/3d73dd03a7a6913a248e5cca7d714f8b8e4c0d01))
    - nrs map fetch and rename multimap ([`889e0d9`](https://github.com/maidsafe/safe_network/commit/889e0d99a6f096329e875c812a29ec165e61f5ae))
</details>

## v0.52.0 (2022-01-22)

<csr-id-3b5ce194213a7090ee83c02b0043700cda230796/>
<csr-id-0190f0305980bdaee30f9f2ab5eb5510149916db/>
<csr-id-3dc23278c6a4fabc250b27f4312f5c51f0f271a4/>
<csr-id-20f416cb7d0960a1d8d6f167a1ad1eed33ed6a7b/>
<csr-id-7a7752f830785ec39d301e751dc75f228d43d595/>

### Refactor

 - <csr-id-3b5ce194213a7090ee83c02b0043700cda230796/> remove one layer of indirection

### Chore

 - <csr-id-0190f0305980bdaee30f9f2ab5eb5510149916db/> safe_network-0.54.0/sn_api-0.52.0/sn_cli-0.45.0
 - <csr-id-3dc23278c6a4fabc250b27f4312f5c51f0f271a4/> update remaining places
 - <csr-id-20f416cb7d0960a1d8d6f167a1ad1eed33ed6a7b/> update from MIT/BSD3 to GPL3
 - <csr-id-7a7752f830785ec39d301e751dc75f228d43d595/> update year on files modified 2022

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.54.0/sn_api-0.52.0/sn_cli-0.45.0 ([`0190f03`](https://github.com/maidsafe/safe_network/commit/0190f0305980bdaee30f9f2ab5eb5510149916db))
    - update remaining places ([`3dc2327`](https://github.com/maidsafe/safe_network/commit/3dc23278c6a4fabc250b27f4312f5c51f0f271a4))
    - Merge #958 ([`437a113`](https://github.com/maidsafe/safe_network/commit/437a113e6e5736e4eb4287f41228806678a9762e))
    - Merge branch 'main' into simplify-sn-api ([`33ef052`](https://github.com/maidsafe/safe_network/commit/33ef0524ae238391f25c8fb340627c34ea79fcb2))
    - Merge #962 ([`29d01da`](https://github.com/maidsafe/safe_network/commit/29d01da5233fd2a10b30699b555a0d85d7a7409a))
</details>

## v0.51.0 (2022-01-20)

<csr-id-923930acb3769cfa7047954a1fee1853ec9e3062/>
<csr-id-57749b7d0671423fe205447bc84d9f8bfc99f54b/>
<csr-id-941b83f3960c84cfee86a8c818233fbbc403c189/>
<csr-id-3b5ce194213a7090ee83c02b0043700cda230796/>
<csr-id-9c9a537ad12cc809540df321297c8552c52a8648/>
<csr-id-20f416cb7d0960a1d8d6f167a1ad1eed33ed6a7b/>
<csr-id-7a7752f830785ec39d301e751dc75f228d43d595/>

### Chore

 - <csr-id-923930acb3769cfa7047954a1fee1853ec9e3062/> safe_network-0.53.0/sn_api-0.51.0/sn_cli-0.44.0
 - <csr-id-57749b7d0671423fe205447bc84d9f8bfc99f54b/> solving new clippy findings
 - <csr-id-941b83f3960c84cfee86a8c818233fbbc403c189/> fix additional wrongly setup test cases

### Refactor

 - <csr-id-3b5ce194213a7090ee83c02b0043700cda230796/> remove one layer of indirection
 - <csr-id-9c9a537ad12cc809540df321297c8552c52a8648/> ties up the loose ends in unified data flow

### Chore

 - <csr-id-20f416cb7d0960a1d8d6f167a1ad1eed33ed6a7b/> update from MIT/BSD3 to GPL3
 - <csr-id-7a7752f830785ec39d301e751dc75f228d43d595/> update year on files modified 2022

### Bug Fixes

 - <csr-id-e18c88019d37ab4f7618dde1a90e19ddf94db1c7/> VersioinHash use Display for encode
 - <csr-id-83ef7a66bb245e2303b80d98d6b8fa888b93d6ba/> make use of all the queries

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 16 commits contributed to the release over the course of 13 calendar days.
 - 13 days passed between releases.
 - 8 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.53.0/sn_api-0.51.0/sn_cli-0.44.0 ([`923930a`](https://github.com/maidsafe/safe_network/commit/923930acb3769cfa7047954a1fee1853ec9e3062))
    - remove one layer of indirection ([`3b5ce19`](https://github.com/maidsafe/safe_network/commit/3b5ce194213a7090ee83c02b0043700cda230796))
    - update from MIT/BSD3 to GPL3 ([`20f416c`](https://github.com/maidsafe/safe_network/commit/20f416cb7d0960a1d8d6f167a1ad1eed33ed6a7b))
    - update year on files modified 2022 ([`7a7752f`](https://github.com/maidsafe/safe_network/commit/7a7752f830785ec39d301e751dc75f228d43d595))
    - Merge node-logrotate origin for rebase with main ([`6df7f6f`](https://github.com/maidsafe/safe_network/commit/6df7f6fec3ee9d37b44db188fd670e4b65796e8c))
    - fix additional wrongly setup test cases ([`941b83f`](https://github.com/maidsafe/safe_network/commit/941b83f3960c84cfee86a8c818233fbbc403c189))
    - make use of all the queries ([`83ef7a6`](https://github.com/maidsafe/safe_network/commit/83ef7a66bb245e2303b80d98d6b8fa888b93d6ba))
    - ties up the loose ends in unified data flow ([`9c9a537`](https://github.com/maidsafe/safe_network/commit/9c9a537ad12cc809540df321297c8552c52a8648))
    - solving new clippy findings ([`57749b7`](https://github.com/maidsafe/safe_network/commit/57749b7d0671423fe205447bc84d9f8bfc99f54b))
    - Merge #885 ([`72a3f12`](https://github.com/maidsafe/safe_network/commit/72a3f1269c9c38add9b88455837655f2bc33b551))
    - Merge branch 'main' into kill-the-blob ([`5a055ba`](https://github.com/maidsafe/safe_network/commit/5a055ba679e6a4f2cd92700af68f8b36ac12a544))
    - Merge branch 'main' into kill-the-blob ([`411ce5b`](https://github.com/maidsafe/safe_network/commit/411ce5b9d4c396484d2384324ae09d346c79013f))
    - Merge branch 'main' into kill-the-blob ([`9c5cd80`](https://github.com/maidsafe/safe_network/commit/9c5cd80c286308c6d075c5418d8a1650e87fddd5))
    - Merge branch 'main' into kill-the-blob ([`fe814a6`](https://github.com/maidsafe/safe_network/commit/fe814a69e5ef5fbe4c62a056498ef88ce5897fef))
    - Merge #917 ([`0eb6439`](https://github.com/maidsafe/safe_network/commit/0eb643910098ab6021561e5b997b6289be9e2c57))
    - Merge #916 #918 #919 ([`5c4d3a9`](https://github.com/maidsafe/safe_network/commit/5c4d3a92ff28126468f07d599c6caf416661aba2))
</details>

## v0.50.6 (2022-01-06)

<csr-id-9c9a537ad12cc809540df321297c8552c52a8648/>
<csr-id-155ee032ee56cbbb34928f2d14529273ccb69559/>
<csr-id-7b0cd4d183a9f82d1d78dbb56572e5bf36714a0a/>
<csr-id-bf16c5ea7051386064233443921438cbbd79d907/>
<csr-id-4f29c285a0b48220df1f1c6c52c4b487350eae08/>

### Refactor

 - <csr-id-9c9a537ad12cc809540df321297c8552c52a8648/> ties up the loose ends in unified data flow

### Chore

 - <csr-id-4f29c285a0b48220df1f1c6c52c4b487350eae08/> safe_network-0.52.12

### Chore

 - <csr-id-155ee032ee56cbbb34928f2d14529273ccb69559/> safe_network-0.52.13/sn_api-0.50.6
 - <csr-id-7b0cd4d183a9f82d1d78dbb56572e5bf36714a0a/> safe_network-0.52.10
 - <csr-id-bf16c5ea7051386064233443921438cbbd79d907/> log EntryHash human readable

### Bug Fixes

 - <csr-id-e18c88019d37ab4f7618dde1a90e19ddf94db1c7/> VersioinHash use Display for encode

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.13/sn_api-0.50.6 ([`155ee03`](https://github.com/maidsafe/safe_network/commit/155ee032ee56cbbb34928f2d14529273ccb69559))
    - safe_network-0.52.12 ([`4f29c28`](https://github.com/maidsafe/safe_network/commit/4f29c285a0b48220df1f1c6c52c4b487350eae08))
    - VersioinHash use Display for encode ([`e18c880`](https://github.com/maidsafe/safe_network/commit/e18c88019d37ab4f7618dde1a90e19ddf94db1c7))
</details>

## v0.50.5 (2022-01-06)

<csr-id-99d012ef529df78ef4c84f5e6ea99d3a77414797/>
<csr-id-bf16c5ea7051386064233443921438cbbd79d907/>
<csr-id-7b0cd4d183a9f82d1d78dbb56572e5bf36714a0a/>

### Chore

 - <csr-id-99d012ef529df78ef4c84f5e6ea99d3a77414797/> safe_network-0.52.11/sn_api-0.50.5/sn_cli-0.43.2

### Chore

 - <csr-id-bf16c5ea7051386064233443921438cbbd79d907/> log EntryHash human readable
 - <csr-id-7b0cd4d183a9f82d1d78dbb56572e5bf36714a0a/> safe_network-0.52.10

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 5 commits contributed to the release over the course of 1 calendar day.
 - 1 day passed between releases.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.11/sn_api-0.50.5/sn_cli-0.43.2 ([`99d012e`](https://github.com/maidsafe/safe_network/commit/99d012ef529df78ef4c84f5e6ea99d3a77414797))
    - log EntryHash human readable ([`bf16c5e`](https://github.com/maidsafe/safe_network/commit/bf16c5ea7051386064233443921438cbbd79d907))
    - safe_network-0.52.10 ([`7b0cd4d`](https://github.com/maidsafe/safe_network/commit/7b0cd4d183a9f82d1d78dbb56572e5bf36714a0a))
    - Merge branch 'main' into kill-the-blob ([`40268a5`](https://github.com/maidsafe/safe_network/commit/40268a598aea8d14c1dbeb1c00712b9f9a664ef8))
    - Merge branch 'main' into kill-the-blob ([`6f89f12`](https://github.com/maidsafe/safe_network/commit/6f89f129ece75dee45f311d30e52ca71b6b7bc98))
</details>

## v0.50.4 (2022-01-04)

<csr-id-a64c7e0414b77f545cb8cdbf64af0fb7212d1f2e/>
<csr-id-4f29c285a0b48220df1f1c6c52c4b487350eae08/>
<csr-id-bebdae9d52d03bd13b679ee19446452990d1e2cf/>
<csr-id-ab8109cf5aede62596abfdeb813a019d03201f96/>
<csr-id-5214d5e7f84a3c1cf213097a5d55bfb293f03324/>
<csr-id-c790077bebca691f974000278d5525f4b011b8a7/>

### Chore

 - <csr-id-a64c7e0414b77f545cb8cdbf64af0fb7212d1f2e/> safe_network-0.52.9/sn_api-0.50.4
 - <csr-id-4f29c285a0b48220df1f1c6c52c4b487350eae08/> safe_network-0.52.12
 - <csr-id-bebdae9d52d03bd13b679ee19446452990d1e2cf/> rename dest to dst
 - <csr-id-ab8109cf5aede62596abfdeb813a019d03201f96/> revert change of fn name
 - <csr-id-5214d5e7f84a3c1cf213097a5d55bfb293f03324/> safe_network-0.52.8

### Refactor

 - <csr-id-c790077bebca691f974000278d5525f4b011b8a7/> rename blob to file

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 5 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.9/sn_api-0.50.4 ([`a64c7e0`](https://github.com/maidsafe/safe_network/commit/a64c7e0414b77f545cb8cdbf64af0fb7212d1f2e))
    - rename dest to dst ([`bebdae9`](https://github.com/maidsafe/safe_network/commit/bebdae9d52d03bd13b679ee19446452990d1e2cf))
    - safe_network-0.52.8 ([`5214d5e`](https://github.com/maidsafe/safe_network/commit/5214d5e7f84a3c1cf213097a5d55bfb293f03324))
    - Merge branch 'main' into kill-the-blob ([`7d38c3d`](https://github.com/maidsafe/safe_network/commit/7d38c3df14d03c042b645ad05be6cd3cc540d631))
    - revert change of fn name ([`ab8109c`](https://github.com/maidsafe/safe_network/commit/ab8109cf5aede62596abfdeb813a019d03201f96))
    - rename blob to file ([`c790077`](https://github.com/maidsafe/safe_network/commit/c790077bebca691f974000278d5525f4b011b8a7))
</details>

## v0.50.3 (2022-01-04)

<csr-id-c790077bebca691f974000278d5525f4b011b8a7/>
<csr-id-5f7000c5ec5895fb3f4c4a17a74ada52bb873fc7/>
<csr-id-ab00eca916d6ed8a0a137004a6b9fd24e7217a70/>
<csr-id-40d1844e0b28578e8b8c6b270151dbb86961a766/>

### Refactor

 - <csr-id-c790077bebca691f974000278d5525f4b011b8a7/> rename blob to file

### Chore

 - <csr-id-40d1844e0b28578e8b8c6b270151dbb86961a766/> safe_network-0.52.7

### Chore

 - <csr-id-5f7000c5ec5895fb3f4c4a17a74ada52bb873fc7/> sn_api-0.50.3
 - <csr-id-ab00eca916d6ed8a0a137004a6b9fd24e7217a70/> safe_network-0.52.5

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.50.3 ([`5f7000c`](https://github.com/maidsafe/safe_network/commit/5f7000c5ec5895fb3f4c4a17a74ada52bb873fc7))
    - safe_network-0.52.7 ([`40d1844`](https://github.com/maidsafe/safe_network/commit/40d1844e0b28578e8b8c6b270151dbb86961a766))
</details>

## v0.50.2 (2022-01-04)

<csr-id-0a70425fb314de4c165da54fdc29a127ae900d81/>
<csr-id-292466119e2d99c36043e7f2247b1bde9ec9ced9/>
<csr-id-ab00eca916d6ed8a0a137004a6b9fd24e7217a70/>

### Chore

 - <csr-id-0a70425fb314de4c165da54fdc29a127ae900d81/> safe_network-0.52.6/sn_api-0.50.2
 - <csr-id-292466119e2d99c36043e7f2247b1bde9ec9ced9/> safe_network-0.52.3

### Chore

 - <csr-id-ab00eca916d6ed8a0a137004a6b9fd24e7217a70/> safe_network-0.52.5

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.6/sn_api-0.50.2 ([`0a70425`](https://github.com/maidsafe/safe_network/commit/0a70425fb314de4c165da54fdc29a127ae900d81))
    - safe_network-0.52.5 ([`ab00eca`](https://github.com/maidsafe/safe_network/commit/ab00eca916d6ed8a0a137004a6b9fd24e7217a70))
</details>

## v0.50.1 (2022-01-04)

<csr-id-4bb2adf52efdac6187fffc299018bf13f3398e14/>
<csr-id-40d1844e0b28578e8b8c6b270151dbb86961a766/>
<csr-id-292466119e2d99c36043e7f2247b1bde9ec9ced9/>

### Chore

 - <csr-id-4bb2adf52efdac6187fffc299018bf13f3398e14/> safe_network-0.52.4/sn_api-0.50.1
 - <csr-id-40d1844e0b28578e8b8c6b270151dbb86961a766/> safe_network-0.52.7

### Chore

 - <csr-id-292466119e2d99c36043e7f2247b1bde9ec9ced9/> safe_network-0.52.3

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.4/sn_api-0.50.1 ([`4bb2adf`](https://github.com/maidsafe/safe_network/commit/4bb2adf52efdac6187fffc299018bf13f3398e14))
    - safe_network-0.52.3 ([`2924661`](https://github.com/maidsafe/safe_network/commit/292466119e2d99c36043e7f2247b1bde9ec9ced9))
</details>

## v0.50.0 (2022-01-03)

<csr-id-ee86dc7ab1781731d3be19f9d7f414f157a91edb/>
<csr-id-d490127b17d53a7648f9e97aae690b232188b034/>
<csr-id-715a154fe7448cd18decd0a666ae11fb02eadedb/>

### Chore

 - <csr-id-ee86dc7ab1781731d3be19f9d7f414f157a91edb/> sn_api-0.50.0/sn_cli-0.43.0
 - <csr-id-d490127b17d53a7648f9e97aae690b232188b034/> safe_network-0.52.2

### Refactor (BREAKING)

 - <csr-id-715a154fe7448cd18decd0a666ae11fb02eadedb/> remove dry-run as arg from all APIs and make it a Safe instance mode

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.50.0/sn_cli-0.43.0 ([`ee86dc7`](https://github.com/maidsafe/safe_network/commit/ee86dc7ab1781731d3be19f9d7f414f157a91edb))
    - remove dry-run as arg from all APIs and make it a Safe instance mode ([`715a154`](https://github.com/maidsafe/safe_network/commit/715a154fe7448cd18decd0a666ae11fb02eadedb))
    - safe_network-0.52.2 ([`d490127`](https://github.com/maidsafe/safe_network/commit/d490127b17d53a7648f9e97aae690b232188b034))
</details>

## v0.49.0 (2022-01-03)

<csr-id-4f600e179bfbf6ac018876cca6f7fc193f5b5f1e/>

### Chore

 - <csr-id-4f600e179bfbf6ac018876cca6f7fc193f5b5f1e/> sn_api-0.49.0/sn_cli-0.42.0

### Bug Fixes (BREAKING)

 - <csr-id-fe13166b6dc4ae0fdd96b20a135baf7ebef3647b/> properly handle scenarios when retrieving empty FilesContainers
   - Also removing the Default impl for VersioHash as it's meaningless, and invalid content version hash.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.49.0/sn_cli-0.42.0 ([`4f600e1`](https://github.com/maidsafe/safe_network/commit/4f600e179bfbf6ac018876cca6f7fc193f5b5f1e))
    - properly handle scenarios when retrieving empty FilesContainers ([`fe13166`](https://github.com/maidsafe/safe_network/commit/fe13166b6dc4ae0fdd96b20a135baf7ebef3647b))
</details>

## v0.48.0 (2022-01-03)

<csr-id-e38925e07d69432db310fc8ec9803200ea964ab2/>
<csr-id-f1bb1909f3fb506c1b7ec9b660ad533b7b8b9044/>
<csr-id-ff1dd477aaea2a4dda6c9c15b5822b1b3a7514b7/>

### Chore

 - <csr-id-e38925e07d69432db310fc8ec9803200ea964ab2/> safe_network-0.52.1/sn_api-0.48.0/sn_cli-0.41.0
 - <csr-id-f1bb1909f3fb506c1b7ec9b660ad533b7b8b9044/> minor refactor and changes to CLI report errors

### Bug Fixes

 - <csr-id-19e8f70c3f4369fae3a80d5de5e56161c5fa0258/> enable logging from api tests; resolve one failing test
 - <csr-id-7ba567f7f491836961e769c836226ebc9a4731f8/> when in dry-run was still requiring a connection by some APIs

### New Features (BREAKING)

 - <csr-id-4adaeaff4f07871840397adc3371ec8b3436e7ce/> change files APIs to accept std::Path for path args rather than only &str
   - Changed the files_container_create API to now create just an empty FilesContainer

### Refactor (BREAKING)

 - <csr-id-ff1dd477aaea2a4dda6c9c15b5822b1b3a7514b7/> ProcessedFiles redefined on more specific data types instead of simply Strings

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release over the course of 11 calendar days.
 - 11 days passed between releases.
 - 6 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.1/sn_api-0.48.0/sn_cli-0.41.0 ([`e38925e`](https://github.com/maidsafe/safe_network/commit/e38925e07d69432db310fc8ec9803200ea964ab2))
    - change files APIs to accept std::Path for path args rather than only &str ([`4adaeaf`](https://github.com/maidsafe/safe_network/commit/4adaeaff4f07871840397adc3371ec8b3436e7ce))
    - minor refactor and changes to CLI report errors ([`f1bb190`](https://github.com/maidsafe/safe_network/commit/f1bb1909f3fb506c1b7ec9b660ad533b7b8b9044))
    - ProcessedFiles redefined on more specific data types instead of simply Strings ([`ff1dd47`](https://github.com/maidsafe/safe_network/commit/ff1dd477aaea2a4dda6c9c15b5822b1b3a7514b7))
    - enable logging from api tests; resolve one failing test ([`19e8f70`](https://github.com/maidsafe/safe_network/commit/19e8f70c3f4369fae3a80d5de5e56161c5fa0258))
    - when in dry-run was still requiring a connection by some APIs ([`7ba567f`](https://github.com/maidsafe/safe_network/commit/7ba567f7f491836961e769c836226ebc9a4731f8))
</details>

## v0.47.0 (2021-12-22)

<csr-id-1d1487f1c8e8f3e33df626af1ff027eea653f84c/>
<csr-id-6b59ad852f89f033caf2b3c7dfcfa3019f8129e8/>
<csr-id-c76c3ab638188cba38911f037829c209fcc45fc3/>
<csr-id-79b2d0a3f52de0335323773936dee9bdbe12a0cf/>
<csr-id-45df3d71cc4b3185602b9d27b8cb0f5bf65a4b43/>
<csr-id-1188ed58eed443b4b8c65b591376f2f9a21acc0d/>

### Other

 - <csr-id-1d1487f1c8e8f3e33df626af1ff027eea653f84c/> calm down the retry loop tests

### Chore

 - <csr-id-6b59ad852f89f033caf2b3c7dfcfa3019f8129e8/> safe_network-0.52.0/sn_api-0.47.0/sn_cli-0.40.0
 - <csr-id-c76c3ab638188cba38911f037829c209fcc45fc3/> safe_network-0.51.7
 - <csr-id-79b2d0a3f52de0335323773936dee9bdbe12a0cf/> safe_network-0.51.6
 - <csr-id-45df3d71cc4b3185602b9d27b8cb0f5bf65a4b43/> safe_network-0.51.5

### New Features

 - <csr-id-1078e59be3a58ffedcd3c1460385b4bf00f18f6b/> use upload_and_verify by default in safe_client

### Refactor (BREAKING)

 - <csr-id-1188ed58eed443b4b8c65b591376f2f9a21acc0d/> minor refactor to error types definitions

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 7 commits contributed to the release over the course of 6 calendar days.
 - 6 days passed between releases.
 - 7 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.52.0/sn_api-0.47.0/sn_cli-0.40.0 ([`6b59ad8`](https://github.com/maidsafe/safe_network/commit/6b59ad852f89f033caf2b3c7dfcfa3019f8129e8))
    - minor refactor to error types definitions ([`1188ed5`](https://github.com/maidsafe/safe_network/commit/1188ed58eed443b4b8c65b591376f2f9a21acc0d))
    - safe_network-0.51.7 ([`c76c3ab`](https://github.com/maidsafe/safe_network/commit/c76c3ab638188cba38911f037829c209fcc45fc3))
    - calm down the retry loop tests ([`1d1487f`](https://github.com/maidsafe/safe_network/commit/1d1487f1c8e8f3e33df626af1ff027eea653f84c))
    - safe_network-0.51.6 ([`79b2d0a`](https://github.com/maidsafe/safe_network/commit/79b2d0a3f52de0335323773936dee9bdbe12a0cf))
    - safe_network-0.51.5 ([`45df3d7`](https://github.com/maidsafe/safe_network/commit/45df3d71cc4b3185602b9d27b8cb0f5bf65a4b43))
    - use upload_and_verify by default in safe_client ([`1078e59`](https://github.com/maidsafe/safe_network/commit/1078e59be3a58ffedcd3c1460385b4bf00f18f6b))
</details>

## v0.46.2 (2021-12-16)

<csr-id-6df94b1d1fb017c9b02e566ca22a518f885397c8/>
<csr-id-17d7906656bec401d6b39cc3551141112a3d77c4/>

### Chore

 - <csr-id-6df94b1d1fb017c9b02e566ca22a518f885397c8/> sn_api-0.46.2
 - <csr-id-17d7906656bec401d6b39cc3551141112a3d77c4/> safe_network-0.51.4

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.46.2 ([`6df94b1`](https://github.com/maidsafe/safe_network/commit/6df94b1d1fb017c9b02e566ca22a518f885397c8))
    - safe_network-0.51.4 ([`17d7906`](https://github.com/maidsafe/safe_network/commit/17d7906656bec401d6b39cc3551141112a3d77c4))
</details>

## v0.46.1 (2021-12-16)

<csr-id-9be440b36db07e1c04ab688b44ef91e4a56ed576/>
<csr-id-595541b83284a5c5b60fbc00e47b1146117d7613/>

### Chore

 - <csr-id-9be440b36db07e1c04ab688b44ef91e4a56ed576/> safe_network-0.51.3/sn_api-0.46.1
 - <csr-id-595541b83284a5c5b60fbc00e47b1146117d7613/> safe_network-0.51.2

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.51.3/sn_api-0.46.1 ([`9be440b`](https://github.com/maidsafe/safe_network/commit/9be440b36db07e1c04ab688b44ef91e4a56ed576))
    - safe_network-0.51.2 ([`595541b`](https://github.com/maidsafe/safe_network/commit/595541b83284a5c5b60fbc00e47b1146117d7613))
</details>

## v0.46.0 (2021-12-16)

<csr-id-edb8de8b4d923e97d68eed40a7953f38461b0281/>
<csr-id-634a8f9f307598c51305067444514b43c85f196d/>
<csr-id-dcbb67fc699d7cb1f3a2c4632bcb8a5738916091/>
<csr-id-c685838d8f9c10b0f4e7541fe201862bb84e8555/>
<csr-id-653f653a775a101679904ab75c8012a72dfdedfb/>
<csr-id-36ca20e606899ecbdea24d845c34ba11ab889cf7/>
<csr-id-d30aa0cb7440b9f3a44fefc3b6b9f7855480958c/>
<csr-id-62d747969b739172910aabca6fcb273d2827fc8a/>
<csr-id-69ae8c20e91dd9959ebfa5456efdf9c218a9d66f/>
<csr-id-6f5516d8bb677462ea6def46aa65a1094767d68c/>

### Test

 - <csr-id-edb8de8b4d923e97d68eed40a7953f38461b0281/> adding a test for retrieving Blob with range over data length

### Chore

 - <csr-id-634a8f9f307598c51305067444514b43c85f196d/> sn_api-0.46.0
 - <csr-id-dcbb67fc699d7cb1f3a2c4632bcb8a5738916091/> safe_network-0.51.1
 - <csr-id-c685838d8f9c10b0f4e7541fe201862bb84e8555/> safe_network-0.51.0
 - <csr-id-653f653a775a101679904ab75c8012a72dfdedfb/> safe_network-0.50.0
 - <csr-id-36ca20e606899ecbdea24d845c34ba11ab889cf7/> safe_network-0.49.3
 - <csr-id-d30aa0cb7440b9f3a44fefc3b6b9f7855480958c/> fmt
 - <csr-id-62d747969b739172910aabca6fcb273d2827fc8a/> safe_network-0.49.2
 - <csr-id-69ae8c20e91dd9959ebfa5456efdf9c218a9d66f/> safe_network-0.49.1
 - <csr-id-6f5516d8bb677462ea6def46aa65a1094767d68c/> safe_network-0.49.0

### New Features (BREAKING)

 - <csr-id-18879590ddfcf125133a6b2b8f3f372e8683be42/> rename Url to SafeUrl

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 2 calendar days.
 - 5 days passed between releases.
 - 11 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.46.0 ([`634a8f9`](https://github.com/maidsafe/safe_network/commit/634a8f9f307598c51305067444514b43c85f196d))
    - safe_network-0.51.1 ([`dcbb67f`](https://github.com/maidsafe/safe_network/commit/dcbb67fc699d7cb1f3a2c4632bcb8a5738916091))
    - safe_network-0.51.0 ([`c685838`](https://github.com/maidsafe/safe_network/commit/c685838d8f9c10b0f4e7541fe201862bb84e8555))
    - safe_network-0.50.0 ([`653f653`](https://github.com/maidsafe/safe_network/commit/653f653a775a101679904ab75c8012a72dfdedfb))
    - safe_network-0.49.3 ([`36ca20e`](https://github.com/maidsafe/safe_network/commit/36ca20e606899ecbdea24d845c34ba11ab889cf7))
    - adding a test for retrieving Blob with range over data length ([`edb8de8`](https://github.com/maidsafe/safe_network/commit/edb8de8b4d923e97d68eed40a7953f38461b0281))
    - fmt ([`d30aa0c`](https://github.com/maidsafe/safe_network/commit/d30aa0cb7440b9f3a44fefc3b6b9f7855480958c))
    - rename Url to SafeUrl ([`1887959`](https://github.com/maidsafe/safe_network/commit/18879590ddfcf125133a6b2b8f3f372e8683be42))
    - safe_network-0.49.2 ([`62d7479`](https://github.com/maidsafe/safe_network/commit/62d747969b739172910aabca6fcb273d2827fc8a))
    - safe_network-0.49.1 ([`69ae8c2`](https://github.com/maidsafe/safe_network/commit/69ae8c20e91dd9959ebfa5456efdf9c218a9d66f))
    - safe_network-0.49.0 ([`6f5516d`](https://github.com/maidsafe/safe_network/commit/6f5516d8bb677462ea6def46aa65a1094767d68c))
</details>

## v0.44.0 (2021-12-10)

<csr-id-2b7c72c3f87f34051ea301250d1af258f9d310cb/>
<csr-id-bb7abce7330a432884e80baa2aa158409e9734d0/>
<csr-id-9eff03598ba09aa339180d7ecd57b50174180095/>
<csr-id-85709655b0ce38246515658b956aa9b8f67cb55a/>
<csr-id-66dc0682edb53c64a0660b3622bdc1a646114dee/>
<csr-id-b5e9dcc5b13b1eda711d4760d9feb8dc929a0c43/>
<csr-id-1120c99868a0a97e8e25a7611fea31838fe9f6f6/>
<csr-id-9afcb8b2776c39de1925742ccb19e36e9f3fec55/>
<csr-id-05f6d98cf21f0158f4b5161484c7c15a0561b6f4/>
<csr-id-86577846e845c110c49e15c95c6bd5595db51773/>
<csr-id-de3051e7e809a8f75507c54f3cf053a4244fdf19/>
<csr-id-69e9be2a1567bfa211af7e9d7595381d9a0a3b38/>
<csr-id-260eaabd2d1b0c26dec9febc963929e65d7ec912/>
<csr-id-bf55f9b7e3b96319de4423e19333bf3b16fd1c78/>
<csr-id-8ea94983b37b1d559358a62d6ca075b97c193f0d/>
<csr-id-a7e058536ae6ae27228bd2254ea6465c5eface35/>
<csr-id-aaab10b3a5a44d9ec844757c71ac091016f51fd1/>
<csr-id-984c5f83e3f4d889dc4e0583b09571e540357cf9/>
<csr-id-ec3dd4991535bb22235e2d1d413dd93489b8aedf/>
<csr-id-51b0f0068c9a279da9a1edf45509cf80a90e663d/>
<csr-id-14c84c9db23557626e4889eff0ff403a574dccad/>
<csr-id-75a4b537573d4e5e8767e38fa7d1b1126dffe148/>
<csr-id-c78f4703a970e8b7466b091ad331d0f2233aa9a3/>

### Test

 - <csr-id-2b7c72c3f87f34051ea301250d1af258f9d310cb/> adapting test_files_container_remove_path test to new behavior

### Refactor

 - <csr-id-bb7abce7330a432884e80baa2aa158409e9734d0/> refactoring files API tests
   - Also removing Url resolver recursive implementation (in favor of an iteration) since it's memory-wise
     inneficient creating new Vec for each resolution step and not providing any benefits.
   - Removing the restriction in files API which was preventing input Urls to contain a version/hash.

### Chore

 - <csr-id-9eff03598ba09aa339180d7ecd57b50174180095/> safe_network-0.48.0
 - <csr-id-85709655b0ce38246515658b956aa9b8f67cb55a/> safe_network-0.47.0
 - <csr-id-66dc0682edb53c64a0660b3622bdc1a646114dee/> re-enable fetch API tests to run in CI
 - <csr-id-b5e9dcc5b13b1eda711d4760d9feb8dc929a0c43/> safe_network-0.46.6
 - <csr-id-1120c99868a0a97e8e25a7611fea31838fe9f6f6/> fmt
 - <csr-id-9afcb8b2776c39de1925742ccb19e36e9f3fec55/> fmt clippy
 - <csr-id-05f6d98cf21f0158f4b5161484c7c15a0561b6f4/> clippy tidyup for rust 1.57
 - <csr-id-86577846e845c110c49e15c95c6bd5595db51773/> safe_network-0.46.5
 - <csr-id-de3051e7e809a8f75507c54f3cf053a4244fdf19/> safe_network-0.46.4
 - <csr-id-69e9be2a1567bfa211af7e9d7595381d9a0a3b38/> safe_network-0.46.3
 - <csr-id-260eaabd2d1b0c26dec9febc963929e65d7ec912/> safe_network-0.46.2
 - <csr-id-bf55f9b7e3b96319de4423e19333bf3b16fd1c78/> safe_network-0.46.1
 - <csr-id-8ea94983b37b1d559358a62d6ca075b97c193f0d/> safe_network-0.46.0
 - <csr-id-a7e058536ae6ae27228bd2254ea6465c5eface35/> safe_network-0.45.0
 - <csr-id-aaab10b3a5a44d9ec844757c71ac091016f51fd1/> safe_network-0.44.5
 - <csr-id-984c5f83e3f4d889dc4e0583b09571e540357cf9/> safe_network-0.44.4
 - <csr-id-ec3dd4991535bb22235e2d1d413dd93489b8aedf/> safe_network-0.44.3
 - <csr-id-51b0f0068c9a279da9a1edf45509cf80a90e663d/> safe_network-0.44.2
 - <csr-id-14c84c9db23557626e4889eff0ff403a574dccad/> safe_network-0.44.1
 - <csr-id-75a4b537573d4e5e8767e38fa7d1b1126dffe148/> safe_network-0.44.0
 - <csr-id-c78f4703a970e8b7466b091ad331d0f2233aa9a3/> safe_network-0.43.0

### New Features

 - <csr-id-fd31bd2ef5ccc9149f8f0a2844c52af60bff3840/> use Vec of DataCmd instead of wrapper struct
 - <csr-id-544cfd21b2ede036615ac00673470f43f0399526/> dry run for NRS Multimaps Registers and FileContainers
 - <csr-id-da5136e3995307d71c11192769ad167b56962f26/> return nrs map on register fork, ignore fork errors of get for another subname
 - <csr-id-e2869c21b5b6c1c93f1a11cbad5c40b94b5e04ea/> improve register fork, fix dups issue, cleanup
 - <csr-id-9428700e5dec7aaabeaf078a8537d678aa0e5c4c/> Nrs with multimaps, resurected multimap tombstones
 - <csr-id-24f1aa0208e3e474862c18b21ea9f048cb6abf25/> expose API for calculate Blob/Spot address without a network connection

### Bug Fixes

<csr-id-b26833d80cf88b9a2dc1bb8478e74d9e37d6dc51/>
<csr-id-15dc3bd46686d25f679c21f85de091578d5f42eb/>

 - <csr-id-8955fcf9d69e869725177340d1de6b6b1e7a203b/> read_from client API was incorrectly using provided length value as an end index
   - Minor refactoring in sn_api moving the SafeData struct into its own file.

### New Features (BREAKING)

 - <csr-id-3fe9d7a6624fe5503f80395f6ed11426b131d3b1/> move Url to sn_api
 - <csr-id-c284f0787afe0d079e53b79b3a9d74cad04c4b0e/> `nrs create` only creates topnames

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 33 commits contributed to the release over the course of 14 calendar days.
 - 15 days passed between releases.
 - 33 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - `nrs create` only creates topnames ([`c284f07`](https://github.com/maidsafe/safe_network/commit/c284f0787afe0d079e53b79b3a9d74cad04c4b0e))
    - read_from client API was incorrectly using provided length value as an end index ([`8955fcf`](https://github.com/maidsafe/safe_network/commit/8955fcf9d69e869725177340d1de6b6b1e7a203b))
    - safe_network-0.48.0 ([`9eff035`](https://github.com/maidsafe/safe_network/commit/9eff03598ba09aa339180d7ecd57b50174180095))
    - files_container_create API was trying to send Register op to the network in dry-run ([`b26833d`](https://github.com/maidsafe/safe_network/commit/b26833d80cf88b9a2dc1bb8478e74d9e37d6dc51))
    - safe_network-0.47.0 ([`8570965`](https://github.com/maidsafe/safe_network/commit/85709655b0ce38246515658b956aa9b8f67cb55a))
    - re-enable fetch API tests to run in CI ([`66dc068`](https://github.com/maidsafe/safe_network/commit/66dc0682edb53c64a0660b3622bdc1a646114dee))
    - adapting test_files_container_remove_path test to new behavior ([`2b7c72c`](https://github.com/maidsafe/safe_network/commit/2b7c72c3f87f34051ea301250d1af258f9d310cb))
    - files container resolver was resolving path even when signaled to not to ([`15dc3bd`](https://github.com/maidsafe/safe_network/commit/15dc3bd46686d25f679c21f85de091578d5f42eb))
    - use Vec of DataCmd instead of wrapper struct ([`fd31bd2`](https://github.com/maidsafe/safe_network/commit/fd31bd2ef5ccc9149f8f0a2844c52af60bff3840))
    - dry run for NRS Multimaps Registers and FileContainers ([`544cfd2`](https://github.com/maidsafe/safe_network/commit/544cfd21b2ede036615ac00673470f43f0399526))
    - safe_network-0.46.6 ([`b5e9dcc`](https://github.com/maidsafe/safe_network/commit/b5e9dcc5b13b1eda711d4760d9feb8dc929a0c43))
    - refactoring files API tests ([`bb7abce`](https://github.com/maidsafe/safe_network/commit/bb7abce7330a432884e80baa2aa158409e9734d0))
    - return nrs map on register fork, ignore fork errors of get for another subname ([`da5136e`](https://github.com/maidsafe/safe_network/commit/da5136e3995307d71c11192769ad167b56962f26))
    - fmt ([`1120c99`](https://github.com/maidsafe/safe_network/commit/1120c99868a0a97e8e25a7611fea31838fe9f6f6))
    - improve register fork, fix dups issue, cleanup ([`e2869c2`](https://github.com/maidsafe/safe_network/commit/e2869c21b5b6c1c93f1a11cbad5c40b94b5e04ea))
    - fmt clippy ([`9afcb8b`](https://github.com/maidsafe/safe_network/commit/9afcb8b2776c39de1925742ccb19e36e9f3fec55))
    - Nrs with multimaps, resurected multimap tombstones ([`9428700`](https://github.com/maidsafe/safe_network/commit/9428700e5dec7aaabeaf078a8537d678aa0e5c4c))
    - clippy tidyup for rust 1.57 ([`05f6d98`](https://github.com/maidsafe/safe_network/commit/05f6d98cf21f0158f4b5161484c7c15a0561b6f4))
    - safe_network-0.46.5 ([`8657784`](https://github.com/maidsafe/safe_network/commit/86577846e845c110c49e15c95c6bd5595db51773))
    - safe_network-0.46.4 ([`de3051e`](https://github.com/maidsafe/safe_network/commit/de3051e7e809a8f75507c54f3cf053a4244fdf19))
    - safe_network-0.46.3 ([`69e9be2`](https://github.com/maidsafe/safe_network/commit/69e9be2a1567bfa211af7e9d7595381d9a0a3b38))
    - safe_network-0.46.2 ([`260eaab`](https://github.com/maidsafe/safe_network/commit/260eaabd2d1b0c26dec9febc963929e65d7ec912))
    - safe_network-0.46.1 ([`bf55f9b`](https://github.com/maidsafe/safe_network/commit/bf55f9b7e3b96319de4423e19333bf3b16fd1c78))
    - safe_network-0.46.0 ([`8ea9498`](https://github.com/maidsafe/safe_network/commit/8ea94983b37b1d559358a62d6ca075b97c193f0d))
    - safe_network-0.45.0 ([`a7e0585`](https://github.com/maidsafe/safe_network/commit/a7e058536ae6ae27228bd2254ea6465c5eface35))
    - expose API for calculate Blob/Spot address without a network connection ([`24f1aa0`](https://github.com/maidsafe/safe_network/commit/24f1aa0208e3e474862c18b21ea9f048cb6abf25))
    - safe_network-0.44.5 ([`aaab10b`](https://github.com/maidsafe/safe_network/commit/aaab10b3a5a44d9ec844757c71ac091016f51fd1))
    - safe_network-0.44.4 ([`984c5f8`](https://github.com/maidsafe/safe_network/commit/984c5f83e3f4d889dc4e0583b09571e540357cf9))
    - safe_network-0.44.3 ([`ec3dd49`](https://github.com/maidsafe/safe_network/commit/ec3dd4991535bb22235e2d1d413dd93489b8aedf))
    - safe_network-0.44.2 ([`51b0f00`](https://github.com/maidsafe/safe_network/commit/51b0f0068c9a279da9a1edf45509cf80a90e663d))
    - safe_network-0.44.1 ([`14c84c9`](https://github.com/maidsafe/safe_network/commit/14c84c9db23557626e4889eff0ff403a574dccad))
    - safe_network-0.44.0 ([`75a4b53`](https://github.com/maidsafe/safe_network/commit/75a4b537573d4e5e8767e38fa7d1b1126dffe148))
    - safe_network-0.43.0 ([`c78f470`](https://github.com/maidsafe/safe_network/commit/c78f4703a970e8b7466b091ad331d0f2233aa9a3))
</details>

## v0.43.0 (2021-11-25)

<csr-id-ca21d1e97fcd28ca351887636affffff78e3aeb3/>

### Chore

 - <csr-id-ca21d1e97fcd28ca351887636affffff78e3aeb3/> safe_network-0.42.0/sn_api-0.43.0

### New Features (BREAKING)

 - <csr-id-3fe9d7a6624fe5503f80395f6ed11426b131d3b1/> move Url to sn_api

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.42.0/sn_api-0.43.0 ([`ca21d1e`](https://github.com/maidsafe/safe_network/commit/ca21d1e97fcd28ca351887636affffff78e3aeb3))
    - move Url to sn_api ([`3fe9d7a`](https://github.com/maidsafe/safe_network/commit/3fe9d7a6624fe5503f80395f6ed11426b131d3b1))
</details>

## v0.42.0 (2021-11-25)

<csr-id-8b8a3616673405005d77868dc397bd7542ab3ea7/>
<csr-id-4b72bfc9a6c3a0db4821e7ebf1f4b5daa7cc56d1/>

### Chore

 - <csr-id-8b8a3616673405005d77868dc397bd7542ab3ea7/> safe_network-0.41.4/sn_api-0.42.0
 - <csr-id-4b72bfc9a6c3a0db4821e7ebf1f4b5daa7cc56d1/> safe_network-0.41.3

### New Features (BREAKING)

 - <csr-id-11750ed18391c7e8cb112d4a34a19f15eedaed1d/> propagate registers (with vec u8 instead of Url) change to sn_api

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.41.4/sn_api-0.42.0 ([`8b8a361`](https://github.com/maidsafe/safe_network/commit/8b8a3616673405005d77868dc397bd7542ab3ea7))
    - safe_network-0.41.3 ([`4b72bfc`](https://github.com/maidsafe/safe_network/commit/4b72bfc9a6c3a0db4821e7ebf1f4b5daa7cc56d1))
</details>

## v0.41.0 (2021-11-24)

<csr-id-df25e4920c570771f6813ca03da02f6dfc8e59fb/>

### Chore

 - <csr-id-df25e4920c570771f6813ca03da02f6dfc8e59fb/> sn_api-0.41.0

### New Features (BREAKING)

 - <csr-id-11750ed18391c7e8cb112d4a34a19f15eedaed1d/> propagate registers (with vec u8 instead of Url) change to sn_api

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 2 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_api-0.41.0 ([`df25e49`](https://github.com/maidsafe/safe_network/commit/df25e4920c570771f6813ca03da02f6dfc8e59fb))
    - propagate registers (with vec u8 instead of Url) change to sn_api ([`11750ed`](https://github.com/maidsafe/safe_network/commit/11750ed18391c7e8cb112d4a34a19f15eedaed1d))
</details>

## v0.40.1 (2021-11-24)

<csr-id-d8ec5a81ae566e8d7068592e01cff4e808b1cad1/>
<csr-id-a973039178af33b859d421cf36571de49cceff17/>
<csr-id-62aa668d5777058ae617f8952cfcb62be002abf3/>
<csr-id-63432eb2e528401ae67da8eea0c82837ab42fc18/>

### Other

 - <csr-id-d8ec5a81ae566e8d7068592e01cff4e808b1cad1/> revert "chore(release): safe_network-0.42.0/sn_api-0.41.0"
   This reverts commit 63432eb2e528401ae67da8eea0c82837ab42fc18.
   
   This release was duplicating everything that was in 0.41.0, probably because the tags weren't
   correct.

### Chore

 - <csr-id-a973039178af33b859d421cf36571de49cceff17/> safe_network-0.41.2/sn_api-0.40.1
 - <csr-id-62aa668d5777058ae617f8952cfcb62be002abf3/> safe_network-0.41.1
 - <csr-id-63432eb2e528401ae67da8eea0c82837ab42fc18/> safe_network-0.42.0/sn_api-0.41.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.41.2/sn_api-0.40.1 ([`a973039`](https://github.com/maidsafe/safe_network/commit/a973039178af33b859d421cf36571de49cceff17))
    - safe_network-0.41.1 ([`62aa668`](https://github.com/maidsafe/safe_network/commit/62aa668d5777058ae617f8952cfcb62be002abf3))
    - revert "chore(release): safe_network-0.42.0/sn_api-0.41.0" ([`d8ec5a8`](https://github.com/maidsafe/safe_network/commit/d8ec5a81ae566e8d7068592e01cff4e808b1cad1))
    - safe_network-0.42.0/sn_api-0.41.0 ([`63432eb`](https://github.com/maidsafe/safe_network/commit/63432eb2e528401ae67da8eea0c82837ab42fc18))
</details>

## v0.40.0 (2021-11-23)

<csr-id-14fdaa6537619483e94424ead5751d5ab41c8a01/>

### Chore

 - <csr-id-14fdaa6537619483e94424ead5751d5ab41c8a01/> safe_network-0.41.0

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 8 days passed between releases.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.41.0 ([`14fdaa6`](https://github.com/maidsafe/safe_network/commit/14fdaa6537619483e94424ead5751d5ab41c8a01))
</details>

## v0.39.0 (2021-11-15)

<csr-id-60cc59ce18406609f36a37861afa920b96dcac99/>
<csr-id-6f0501699b0d0620a7c9d2b013944f90884ca1c3/>
<csr-id-6a8a173acf96d98d00548659b4641488f5fec2ee/>
<csr-id-7d00d0ad089924915aa2bf564b5b925825aa4880/>
<csr-id-472df87678c07624d6163b8105f110cd22e4e3c9/>
<csr-id-7fb6bd96a8bdaaee64592b5dc02596b9f6220165/>
<csr-id-774694795114dc392db5219393fa63f204fcc905/>
<csr-id-e3235b0329c74b20092278c628a2115d50e206d7/>
<csr-id-a4b7d9ba415831308c5ec6d902722843715a2d97/>
<csr-id-86910340897256bb4df77b6edaa0f2c9584d6dce/>
<csr-id-c0ac51ae4bf4dbd9df3dd39700887df439eec4f6/>
<csr-id-5780fd1d6ba480cb775fd66e53e41f02d97b3a94/>
<csr-id-afd5422945fd1fc4ac509713e72471076ea4aee0/>
<csr-id-a6107accfc6950b4beb2dcb84dfaa3c0e18bbd5d/>
<csr-id-d910e8607c8898bb8d8ccb0c824d8a5401a2c938/>
<csr-id-c2c6716d29e56f387776202dad94ddda9b8fe2b2/>
<csr-id-f6f07349a7524304b3d6b1c22db65d77be519f4c/>
<csr-id-e692becbbf09e2500284cb1507916fac56149f02/>
<csr-id-ae026bb9ce91b1373b8b300c41bfef0c3f295c7a/>
<csr-id-f9e07293ea1f8cd5e4428d95a299ba06c0f30a20/>
<csr-id-5ac36cc64566561f4d442058c91b9857622e6f26/>
<csr-id-b916d7bdc83e9a02fd29e9cbd6623fc922066a6c/>
<csr-id-b6eddcbf5d272e6a4430cfd6488f5236bef92a5d/>
<csr-id-e814ff3b8c58ae7741938a1c73a22c87ed602883/>
<csr-id-3da9b26f467cb8c468ffb0a319559a5a0f60e86e/>
<csr-id-3703819c7f0da220c8ff21169ca1e8161a20157b/>
<csr-id-d0134e870bb097e095e1c8a33e607cf7994e6491/>
<csr-id-712d9a4e72f62725a0b0ac5526bc68abe8ca503f/>
<csr-id-67b746f607501511c38fe752f64119a12985ab72/>
<csr-id-40bcd0f46dad6177b0052b73393d7789fd559b33/>
<csr-id-e107e1314957053db2d71357450cac65cba52a68/>
<csr-id-4db2c27badcccac6d7368566ee6f483613c3aa93/>
<csr-id-cfc35809120cdf2144c8df5dc509eb316bcb0068/>
<csr-id-9272bb55edf690ccd33de5904530e7ff8036c0fe/>
<csr-id-70015730c3e08881f803e9ce59be7ca16185ae11/>
<csr-id-ab00cf08d217654c57449437348b73576a65e89f/>
<csr-id-213cb39be8fbfdf614f3eb6248b14fe161927a14/>
<csr-id-cbdf0236f7cd241e7addb1afe75ed5e5cfac00ab/>
<csr-id-abfe7378604a74119accd7b9f86bef5682b0784a/>
<csr-id-d9c35b79a6d0aa787e627b747e665d96bb110c13/>
<csr-id-fc10d037d64efc86796f1b1c6f255a4c7f91d3e1/>
<csr-id-407efd15e0b4854864b83ccdb7d2c3adbb0a02e2/>
<csr-id-442c1bdd3f9c22424aa9642d71d4016c868b0b58/>
<csr-id-0387123114ff6ae42920577706497319c8a888cb/>
<csr-id-225432908839359800d301d9e5aa8274e4652ee1/>
<csr-id-2b5d17740ca74fc379cab89cb95683e200589148/>
<csr-id-01183625d7a1a60b652b1a295a908fa8ba04f6f7/>
<csr-id-335f9dcfc4588624728b4b10c576953d51a08e1a/>
<csr-id-49d04e1414bf517cc76ebe2c6b86e0b3dd48e47a/>
<csr-id-91e1f0cefe1eb6a6d70e0df3d4d3b6f97e76ecef/>
<csr-id-7ceab22ae72535159db7fbfdc5832b1aea891388/>
<csr-id-ce56a3504c8f27bfeb13bdf9051c2e91409230ea/>
<csr-id-0d4755ed64a65c223bad253d9d7a03980ec12e8d/>
<csr-id-b9bae620533559c5671075fb7a3fe576fe14431f/>
<csr-id-d77859a8138de0ddcd6b121b928efe13e0254e81/>
<csr-id-6cf56022cd6beec4254d0b1667c4d654f87e6a5a/>
<csr-id-4518154481dbd3aeb397353d4ec296ea98ee3e9a/>
<csr-id-590c4c634046ab655a84093c2ed60e8289415d44/>
<csr-id-a09c4c26230b0cf60d7d792d89f0e275f2b64bc2/>
<csr-id-de482a5611333d069076d7da1b7c5a6017db65eb/>
<csr-id-e3c6da38f92c354c560bd6b555d76f698779ebcf/>
<csr-id-6f672547ee9a55b241aa6004b353d6919fb0e8cb/>
<csr-id-792bce8dd94192f17c51d6a1c0b63c7c214ad7c3/>
<csr-id-b61e83716cce00c0ba02f3d50bf060cfc095051a/>
<csr-id-6d909431d6a7164a06f2322d188c1d4764e5d0b8/>
<csr-id-43c675ee514aa73fb5192717dae58c97587521e7/>
<csr-id-f0bef9e5e79381abf27eec7fae28f4ce0fecb370/>
<csr-id-4ba83c720fabcace7a2859ad308be5922a6597c0/>
<csr-id-b38d840320d65b09ce85db9074f7b7a9487f83df/>
<csr-id-b04c1ca2090a32c423ffe2c23ac22be9f5ebbcf3/>
<csr-id-2dff02dc71bc3574763906c8592d32bde64337c9/>
<csr-id-5ba32bd9a86e9c44cf3d59f54be899dc3e2ca861/>
<csr-id-4f89812ed5ca3394d2cd7b93e3c79aac2929d11d/>
<csr-id-6e4ea368fdcedb10042b5d8dc94ab02eece47003/>
<csr-id-84260fc32473c9a84f5c3e6fd54564a865f9d7fe/>
<csr-id-77e1c693735760cf1c24c9a4552c9104e52cada0/>
<csr-id-422547f9081de77538f2241c727ac55b00e1e48b/>
<csr-id-7b6445a5b9903b1704c45759878bced097bcb82c/>
<csr-id-b261640cebdaf4f7fb1b9c13e911bf82bb46f33a/>
<csr-id-274de45a63122c9685617c97272deb603dd9a15b/>
<csr-id-f9c19faa33db0fb76d392d8cbcd7d910131fedb2/>
<csr-id-e892a99f2feb0c36204cfff103b13ca2e5e96388/>
<csr-id-9c3adffd0889f045ac19110072a194072d294705/>
<csr-id-eb88c0cc1a8e7a21d7af4004c1e1a0a49297f22c/>
<csr-id-19ca515ac84d8cf5d99c4b2cba25561248597f13/>
<csr-id-52c8c8d7d2f5a5f9c7d2862bcf3fca1902048695/>
<csr-id-eb2364c77d6e755d6e184735e50db366faf266d2/>
<csr-id-0e6ffb0a3c216d891e6a60ac162d733d2fa28690/>
<csr-id-2de7f66f3f732b9dae55dad50f15888513b5a125/>
<csr-id-b5a6d8115ad3975a17dd973430480adf6c483490/>
<csr-id-29a978f3047464ad8014817e331218372b53c06c/>
<csr-id-77805418d129cb2924dc35b6a88f704771ef6e5c/>
<csr-id-dfabea0a26f97f420f47ba314cae0882aae47dca/>
<csr-id-34ef8e8c31de6e013d3539a0ec595b32f9301194/>
<csr-id-106407e8125cc003794ba6249158aa1a655d3357/>
<csr-id-c56e26502ee44af179bfa65c8a194a2190b16842/>
<csr-id-e0a4de8de20e5023b83256d39beca19f759ba129/>
<csr-id-5f8d61cf41eb547c13b65d2030beefd235b75820/>
<csr-id-4905fae6259063411c5e4ef5fd2afb531980630c/>
<csr-id-ee05ed31cb12d8e1d8bac7569beec90db52a5840/>
<csr-id-b99e7dee3e72e703b47888e3ff03d2baa933b408/>
<csr-id-e389ab24f2186fc515b115e736a06d20756ae031/>
<csr-id-d3d04e36b8fb52ce070aecc2b1f77eb158983427/>
<csr-id-d5c1cd2808f9844b06b846ec10dfe05146137023/>
<csr-id-8f309dada1517afa10c263a52f5597429f764890/>
<csr-id-8b9b481df5d124857abb02158739a6ded8f02af7/>
<csr-id-6f2dd39d783812a9b3abd774b6bebd4cde2d5a1e/>
<csr-id-08f5fec47809bbf0aea61a939dfb3c909043703f/>
<csr-id-95200df5f310911294ee72153d10d13f2e4fb737/>
<csr-id-019370cfd0ace44c656caf45c17248f2a547dbbf/>
<csr-id-873fe29ac9042b7ad28a29630d2c048bde3a7634/>
<csr-id-9e191132a4281c53bd4872a756888234adfc0e2a/>
<csr-id-cb66e8f5a89872d018e48311738d96173ae8274c/>
<csr-id-4320a059224ef6018f7fc067f4a40a6534beeebb/>
<csr-id-426158fcbb6d7c1fe44755c138bba1ac825a0a0c/>
<csr-id-0b17fddbe3418531df1d03a82d4eb510b819b811/>
<csr-id-532aed1ed8e6b3957627ff2cc2f9d10d87fe3cb2/>
<csr-id-5a1cd2790b159e35c734dfb1fe64a43ea4409dfc/>
<csr-id-0a5c18d115820f7124050bc0a246503b5cc63fd9/>
<csr-id-b863e7eb299472b0c9dbd633b1b892cc221efb46/>
<csr-id-371e7f00e6463063c99beb9823d8684355359d2a/>
<csr-id-4466c48a3fcec76f6c90cf6fcf1f28b177978c90/>
<csr-id-1e4c47aed1aeed3488e370ab0c33a7b5519e40f5/>

### New Features

 - <csr-id-9098cff4c4d2ba15321dd072c970a18781a04a49/> add an example app showcasing how to fetch a Blob from Safe using sn_api
 - <csr-id-c5b0f0b630f673697367361508a30caf7ad787bd/> fixing multimap_remove and adding multimap_get_by_hash API
 - <csr-id-129f83f9b3a22669bf0fe0b5787546e4e3a924a0/> adding Register and Multimap public APIs
 - <csr-id-0beee4f8a1e876c1a7f482ba3be554e5b1654a42/> allow immutable reference to fetch method
 - <csr-id-cd3e1b71ef70bb7a4e5a786bdd337c2797250fdf/> allow aliases of Safe (immutable references)
   Tweaked a few method signatures to leave out the mutability binding.
   This means there can be multiple (immutable) references, instead of
   one unique reference to the Safe instance.
 - <csr-id-4f6c526e194f1b949c1b5b126442150157b7b0ba/> support transfers to BLS public keys
 - <csr-id-0f2f3c74dc81fefbc719e79f41af434023ac0462/> re-enabling dry-run feature for Blob API as well as for CLI commands
 - <csr-id-cb8ea4d6740f0f5e69be941608bcd620ed94e549/> when the --to value of keys transfer command doesn't start with safe:// and cannot be decoded as PK fallback to assume it's a URL
 - <csr-id-5499aeb2f755ce363b709c5379b860048c92ce5a/> pass SecretKey by reference
 - <csr-id-85462b6fb58f16f4797d7ef2816e96a287af7ad9/> adapting to new Seq type where Policy is immutable
 - <csr-id-45f11ae22df242e229d01bfc5dc2b6ac9de8536d/> customise the error message displayed when a panic occurred
 - <csr-id-ddb318d96c8a9bf8299bd14aa6824252296bde27/> store serialised NrsMap in a Blob and only its XOR-URL in the NrsMapContainer's underlying Sequence
 - <csr-id-1f377038ea693d5253f658d480885662df5b0542/> store serialised FilesMap in a Blob and only its XOR-URL in the FilesContainer's underlying Sequence
 - <csr-id-6c9cf24df423abae568fab63fc6615d9f7a3df68/> update sn_client and data types
 - <csr-id-2a43ca8fb10dcdbf085890643c673491399b1a8b/> command to configure networks in the config by providing a list of IP and ports
 - <csr-id-e366c878da84d2cf051bbc692e6b80c675ef8393/> add auth version subcommand which prints out the authd binary version
 - <csr-id-7a77ef4e3f3730d2f033d0365b2446bd560b1e21/> default to check balance of key assigned to CLI when no sk is provided to keys balance command
 - <csr-id-c20932f5c15fa16ccad907208522b9c9b52bb062/> support transfers to Ed25519 public keys in addition to using a Wallet or SafeKey URLs
 - <csr-id-b2e3faf9b0943ec779b1e513c76179048dbb0db3/> re-enable implementation for coins transfers with Ed25519 keys
 - <csr-id-d8186c309a7e6ca4862fb8855da6636a9f0d84c6/> insert and retrieve authorised apps keypairs into/from the Safe
 - <csr-id-d64d0fd9c0d6cd9002405985ff03f6d9ff7aa695/> reenable use of connect from auth responses
 - <csr-id-58ecf7fbae95dee0a103ce39d61efaac6e2cf550/> adapt authd client api and service names to new terminology of Safe creation and unlocking
 - <csr-id-805298fa5f4d1455015d2aa248143f460b4057ba/> store generated keypairs on the network
 - <csr-id-0cd75fbd9aceace5fbfc4b726a48d652177d61ce/> loop seq gets w/ timeout
   Add a timeout to sequence retrieval from the network to aid in
   convergence of data (ie. retry when get fails incase the responding
   nodes havent had time to update their own data yet).
   
   Timeout is configurable per safe app api instance, sop could be exposed
   for apps.
 - <csr-id-3f23687471b846e3ad1e2492c237a21f212b753f/> reenable decoding auth reqs and basic app key generation
 - <csr-id-b994b8d6ec1fcfc540e91aa9df79ba849aee7647/> setting up IPC for auth
 - <csr-id-07dd1956d4a53e2f4d09310b48e2a43a3a10e795/> moving in basics of auth func into the repo
 - <csr-id-de64d44d336611f4f7f427fd6b627a8c0880fd80/> remove sn_url crate form this repo and depend on the one just published on crates.io
 - <csr-id-7d2bd2f7ad27724866d04b7e10d69418f6decad9/> check for existing balance + err when creating acct

### Bug Fixes

 - <csr-id-d0cb7674071694885ebf9620a8cefcc4e62ecfcc/> NRS remove test
 - <csr-id-5582e995193ba17de3a3d0837271b405957d28ee/> NRS register failures
 - <csr-id-c88a1bc5fda40093bb129b4351eef73d2eb7c041/> resolution test except for the blob range tests
 - <csr-id-2fe38d13bc74a7c7cede96340d275ef3f94e1427/> NRS all tests except for the register issue
 - <csr-id-6c61764c7479405d2978bec1bbc5cbd11ca5e7c8/> parsing Url as xorurl
 - <csr-id-441be176521c20f9372b86d6daffc44bb97207fc/> add feature gate for app-specific error
 - <csr-id-d91150d8f46003cc0fa7813e4ae907b187379de8/> change node_path arg of 'node bin-path' command to node-path
 - <csr-id-8d0ac669b27247fed193cdc0543ab5c892bd7c0d/> styling by cargo fmt and clippy
 - <csr-id-e6b5cb012a8684c87c574ce98e55ad038d573d49/> removed more mut bindings to self methods
 - <csr-id-419bc87f345b4790a880b5def3b5b791d920a679/> fix set_content_version api in SafeUrl API to update its internal state
 - <csr-id-ce1823a437ffe59db01cfb412821fc7becd4634a/> pass approriate type
 - <csr-id-bfb09f28d9ff304b80354b971f8f02ffbf2a60c5/> adapt to new error returned by sn-client when Map entry is not found
 - <csr-id-f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0/> keypair API now returns a new randomly create Ed25519 key pair
 - <csr-id-9f18fb51a8c173ebb07e3ab9cea7a8f66b373fc5/> allocate testcoin to keypair generated for app upon being authorised the very first time
 - <csr-id-3694efb16b16fcfc0d34db51187c043d0e24f09c/> store Blob API was returning the wrong xorname
 - <csr-id-5a30bf331242ba8dd9b3189dc255b134fdf24587/> keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey
 - <csr-id-01cc2894b37908377eb822a826f46c7fef39347e/> ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes
 - <csr-id-87d64dc3fe865b68f014da373aa253caf50d6512/> remove repeated data storing in SafeAppClient::store_sequence()
 - <csr-id-4bb48397cbe321bd551ec9530ca0325208dd138f/> set `formatting` feature for `time` dependency
   We use time formatting, so this feature should be necessary for the
   crate to compile. Weirdly, compilation only fails when publishing, not
   when checking or building. It's assumed this has something to do with
   feature resolution in workspaces 🤷

### chore (BREAKING)

 - <csr-id-60cc59ce18406609f36a37861afa920b96dcac99/> adapt test


### Test

 - <csr-id-6f0501699b0d0620a7c9d2b013944f90884ca1c3/> adding first set of tests for Multimap API
 - <csr-id-6a8a173acf96d98d00548659b4641488f5fec2ee/> init logger in sn_api tests
 - <csr-id-7d00d0ad089924915aa2bf564b5b925825aa4880/> add a simple example app which uploads a file using the API
 - <csr-id-472df87678c07624d6163b8105f110cd22e4e3c9/> connect to port 12000 by default in local testnets
 - <csr-id-7fb6bd96a8bdaaee64592b5dc02596b9f6220165/> adapt wallet tests and minor refactoring
 - <csr-id-774694795114dc392db5219393fa63f204fcc905/> adapt tests to new transfer costs and minor refactor to transfers errors handling
 - <csr-id-e3235b0329c74b20092278c628a2115d50e206d7/> minor adjustments on cli test utilities

### Style

 - <csr-id-a4b7d9ba415831308c5ec6d902722843715a2d97/> fix styling by cargo fmt

### Refactor

 - <csr-id-86910340897256bb4df77b6edaa0f2c9584d6dce/> update cli to use new nrs api
   There are changes here to update the NRS command handlers to use the new API for NRS, and other
   smaller changes for referencing types that had been moved to new modules. This commit also restores
   a function, `get_map_summary`, that was deleted from the `NrsMap` struct. The CLI `cat` and `dog`
   commands still relied on this functionality, so it's been provided as a placeholder for now, just to
   get the CLI to compile.
   
   At this point, the NRS test suite is not passing, because there have been some behavioural changes
   as part of the refactor. Unfortunately, because of the state of the rest of the network just now,
   it's hard to isolate these specific changes to get the tests passing again, as other functionality
   like the `cat` command appear to be broken just now too.
   
   I think it's worth committing this as it is now, with the CLI compiling, then we can come back and
   address the other issues that will get the test suite passing again.
 - <csr-id-c0ac51ae4bf4dbd9df3dd39700887df439eec4f6/> nrs and resolver
 - <csr-id-5780fd1d6ba480cb775fd66e53e41f02d97b3a94/> moving out safe_url mod as a standalone sn_url crate
 - <csr-id-afd5422945fd1fc4ac509713e72471076ea4aee0/> re-organising files, nrs and xorurl files into their own mod folders
 - <csr-id-a6107accfc6950b4beb2dcb84dfaa3c0e18bbd5d/> reorganise files module
 - <csr-id-d910e8607c8898bb8d8ccb0c824d8a5401a2c938/> use get_sequence_entry
 - <csr-id-c2c6716d29e56f387776202dad94ddda9b8fe2b2/> migrating to use anyhow for CLI errors and use thiserror for sn_api error types
 - <csr-id-f6f07349a7524304b3d6b1c22db65d77be519f4c/> return anyhow::Result/Error from all CLI tests
 - <csr-id-e692becbbf09e2500284cb1507916fac56149f02/> remove Error::Unexpected and Error::Unknown errors from API
 - <csr-id-ae026bb9ce91b1373b8b300c41bfef0c3f295c7a/> properly serialise key pairs in CLI commands output
 - <csr-id-f9e07293ea1f8cd5e4428d95a299ba06c0f30a20/> minor reorganisation to cli test scripts
 - <csr-id-5ac36cc64566561f4d442058c91b9857622e6f26/> minor renamings in authd status report with new terminology
 - <csr-id-b916d7bdc83e9a02fd29e9cbd6623fc922066a6c/> populate each keypair generated for authorised apps with testcoins
 - <csr-id-b6eddcbf5d272e6a4430cfd6488f5236bef92a5d/> adapt to latest sn-client api changes and further simplification of auth messages
 - <csr-id-e814ff3b8c58ae7741938a1c73a22c87ed602883/> simplify authd messages format and serialisation

### Other

 - <csr-id-3da9b26f467cb8c468ffb0a319559a5a0f60e86e/> fix api tests.
   Removes retry_loop as it's no longer needed for general queries, clients retry in the core lib.
   
   Set the query timeout correctly and so reduces test time.
   Removes api_tests TEST_AUTH setting to a cli dependency auth variable which may not exist and isn't needed. Falls back to random keys.
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
 - <csr-id-712d9a4e72f62725a0b0ac5526bc68abe8ca503f/> adding a README.md file for sn_api crate with a description of the current APIs
 - <csr-id-67b746f607501511c38fe752f64119a12985ab72/> fix all clippy issues after updating to rust 1.49
 - <csr-id-40bcd0f46dad6177b0052b73393d7789fd559b33/> updates and enhancements to the User Guide, and to some commands help messages
 - <csr-id-e107e1314957053db2d71357450cac65cba52a68/> updating CLI User Guide
 - <csr-id-4db2c27badcccac6d7368566ee6f483613c3aa93/> fix proptest regex
 - <csr-id-cfc35809120cdf2144c8df5dc509eb316bcb0068/> update pass/phrase derivation proptest
 - <csr-id-9272bb55edf690ccd33de5904530e7ff8036c0fe/> add login/creation seedable pk tests

### Chore

 - <csr-id-70015730c3e08881f803e9ce59be7ca16185ae11/> safe_network v0.40.0/sn_api v0.39.0
 - <csr-id-ab00cf08d217654c57449437348b73576a65e89f/> safe_network v0.39.0
 - <csr-id-213cb39be8fbfdf614f3eb6248b14fe161927a14/> update bls_dkg and blsttc to 0.7 and 0.3.4 respectively
 - <csr-id-cbdf0236f7cd241e7addb1afe75ed5e5cfac00ab/> safe_network v0.38.0
 - <csr-id-abfe7378604a74119accd7b9f86bef5682b0784a/> add env var for client query timeout
   rename ClientConfig
 - <csr-id-d9c35b79a6d0aa787e627b747e665d96bb110c13/> enable tests
 - <csr-id-fc10d037d64efc86796f1b1c6f255a4c7f91d3e1/> bump rust edition
   The few breaking changes in this edition did not affect us.
 - <csr-id-407efd15e0b4854864b83ccdb7d2c3adbb0a02e2/> appease clippy
 - <csr-id-442c1bdd3f9c22424aa9642d71d4016c868b0b58/> switch from `chrono` to `time`
   Due to the active security advisory against chrono (RUSTSEC-2020-0159)
   is makes sense to switch to a library that is not affected (though we
   are not using the affected APIs in `chrono`).
 - <csr-id-0387123114ff6ae42920577706497319c8a888cb/> upgrade `tracing-appender` and `tracing-subscriber`
   These new versions have dropped their dependence on `chrono`, which has
   an active security advisory against it (RUSTSEC-2020-0159) which seems
   unlikely to be resolved.
   
   `chrono` is still being pulled in by `qp2p` (via `rcgen`), `sn_api`, and
   `sn_launch_tool`. This will be fixed in future commits.
 - <csr-id-225432908839359800d301d9e5aa8274e4652ee1/> move safe_network code into sn directory
   Due to the fact that we're now using multiple crates, the safe_network code is moved into the `sn`
   directory.
   
   A Cargo.toml is added to the root directory to establish this repository as a workspace, currently
   with 2 members, sn and sn_api. If you now run a `cargo build` at the root directory, it will build
   both of these crates.
   
   The Github Actions workflows that were brought in from the `sn_api` merge were also removed here.
 - <csr-id-2b5d17740ca74fc379cab89cb95683e200589148/> move sn_api code into an sn_api directory
   The initial merge of the `sn_api` repository put the code for `sn_api` into the `src` directory at
   the root of the repository, which has merged it in alongside the code for `safe_network`. We now
   create an `sn_api` directory at the root of the repository and move the `sn_api` code in here. This
   completely isolates the code for `sn_api` in its own crate, with the same Cargo.toml from the old
   repository.
   
   As things are, the crate builds fine.
 - <csr-id-01183625d7a1a60b652b1a295a908fa8ba04f6f7/> upgrade sn_api to use 0.36.x of safe_network
   The only code breaking change here was that the `Config::new` function had been extended to use a
   new `query_timeout` parameter. For now, the API is opting not to pass a value for this optional
   parameter.
   
   Also remove unused dependency `async-trait`.
 - <csr-id-335f9dcfc4588624728b4b10c576953d51a08e1a/> various renamings and changes matching PR comments
 - <csr-id-49d04e1414bf517cc76ebe2c6b86e0b3dd48e47a/> fmt
 - <csr-id-91e1f0cefe1eb6a6d70e0df3d4d3b6f97e76ecef/> update sn_client to 0.54.4
 - <csr-id-7ceab22ae72535159db7fbfdc5832b1aea891388/> minor changes to rebase with master
 - <csr-id-ce56a3504c8f27bfeb13bdf9051c2e91409230ea/> removing unused files
 - <csr-id-0d4755ed64a65c223bad253d9d7a03980ec12e8d/> update sn_client and sn_data_types to latest
 - <csr-id-b9bae620533559c5671075fb7a3fe576fe14431f/> fixing files and NRS APIs tests
 - <csr-id-d77859a8138de0ddcd6b121b928efe13e0254e81/> upgrade tokio to v1.3.0 and quinn to v0.10.1
 - <csr-id-6cf56022cd6beec4254d0b1667c4d654f87e6a5a/> re-enabling wallet APIs tests in CI
 - <csr-id-4518154481dbd3aeb397353d4ec296ea98ee3e9a/> re-enabling xorurl & keys APIs tests in CI
 - <csr-id-590c4c634046ab655a84093c2ed60e8289415d44/> re-enabling sequence & fetch APIs tests in CI
 - <csr-id-a09c4c26230b0cf60d7d792d89f0e275f2b64bc2/> remove unused utility function
 - <csr-id-de482a5611333d069076d7da1b7c5a6017db65eb/> adding a step to check for unused dependencies
 - <csr-id-e3c6da38f92c354c560bd6b555d76f698779ebcf/> upgrade sn_client to v0.46.12 and most of all dependencies to their latest published version
 - <csr-id-6f672547ee9a55b241aa6004b353d6919fb0e8cb/> Remove unused ed25519 dependency
 - <csr-id-792bce8dd94192f17c51d6a1c0b63c7c214ad7c3/> update tiny-keccak from 1.5.0 to 2.0.2
   keyword was used interchangeably with passphrase so remove keyword.
   pin was used interchangeably with salt so remove pin.
   passphrase, password ordering is always the same (alphabetical).
 - <csr-id-b61e83716cce00c0ba02f3d50bf060cfc095051a/> upgrade sn_client to v0.46.9 and solve clippy issues
 - <csr-id-6d909431d6a7164a06f2322d188c1d4764e5d0b8/> set bootstrapping contacts for api-tests
 - <csr-id-43c675ee514aa73fb5192717dae58c97587521e7/> provide bootstrapping contacts list to sn_client as required by new sn_client API
 - <csr-id-f0bef9e5e79381abf27eec7fae28f4ce0fecb370/> upgrading sn_client to v0.46.2
 - <csr-id-4ba83c720fabcace7a2859ad308be5922a6597c0/> changes to remove any use of Arc for keypairs and secret keys
 - <csr-id-b38d840320d65b09ce85db9074f7b7a9487f83df/> update sn_client and dts
 - <csr-id-b04c1ca2090a32c423ffe2c23ac22be9f5ebbcf3/> Money -> Token
 - <csr-id-2dff02dc71bc3574763906c8592d32bde64337c9/> do not attempt to retry fetching a Sequence entry if not found the first time
 - <csr-id-5ba32bd9a86e9c44cf3d59f54be899dc3e2ca861/> minor logging improvements and upgrade sn_client to v0.44.16
 - <csr-id-4f89812ed5ca3394d2cd7b93e3c79aac2929d11d/> upgrade sn_client to v0.44.15
 - <csr-id-6e4ea368fdcedb10042b5d8dc94ab02eece47003/> minor change to error returned when parsing pk from hex
 - <csr-id-84260fc32473c9a84f5c3e6fd54564a865f9d7fe/> minor enhancement to logs when instantiating safe client
 - <csr-id-77e1c693735760cf1c24c9a4552c9104e52cada0/> temporarily disable running tests in CI
 - <csr-id-422547f9081de77538f2241c727ac55b00e1e48b/> remove unwrap instances from prod and test code
 - <csr-id-7b6445a5b9903b1704c45759878bced097bcb82c/> update credentials location
 - <csr-id-b261640cebdaf4f7fb1b9c13e911bf82bb46f33a/> fmt
 - <csr-id-274de45a63122c9685617c97272deb603dd9a15b/> simplify loop on err
 - <csr-id-f9c19faa33db0fb76d392d8cbcd7d910131fedb2/> tidying up some logs
 - <csr-id-e892a99f2feb0c36204cfff103b13ca2e5e96388/> unwrap_or_else -> _or_ for empty funcs
 - <csr-id-9c3adffd0889f045ac19110072a194072d294705/> fix lint issues
 - <csr-id-eb88c0cc1a8e7a21d7af4004c1e1a0a49297f22c/> unused variable `address`
 - <csr-id-19ca515ac84d8cf5d99c4b2cba25561248597f13/> generate_random_ed_keypair
 - <csr-id-52c8c8d7d2f5a5f9c7d2862bcf3fca1902048695/> update sn_client
 - <csr-id-eb2364c77d6e755d6e184735e50db366faf266d2/> clippy
 - <csr-id-0e6ffb0a3c216d891e6a60ac162d733d2fa28690/> reliably derives seed
 - <csr-id-2de7f66f3f732b9dae55dad50f15888513b5a125/> clippy
 - <csr-id-b5a6d8115ad3975a17dd973430480adf6c483490/> setting up for no ClientId
   Remove PublicId refs too as data_type has had this removed
 - <csr-id-29a978f3047464ad8014817e331218372b53c06c/> Batch of changes for sk handling
 - <csr-id-77805418d129cb2924dc35b6a88f704771ef6e5c/> dep updates for dt and client
 - <csr-id-dfabea0a26f97f420f47ba314cae0882aae47dca/> converting to more generic data types for keypair sk pk
   W/ updated client and data_types deps
 - <csr-id-34ef8e8c31de6e013d3539a0ec595b32f9301194/> small tidy
 - <csr-id-106407e8125cc003794ba6249158aa1a655d3357/> clippy
 - <csr-id-c56e26502ee44af179bfa65c8a194a2190b16842/> remove merge added file
 - <csr-id-e0a4de8de20e5023b83256d39beca19f759ba129/> clippy
 - <csr-id-5f8d61cf41eb547c13b65d2030beefd235b75820/> update to reference renamed sn_client
 - <csr-id-4905fae6259063411c5e4ef5fd2afb531980630c/> tidying up
 - <csr-id-ee05ed31cb12d8e1d8bac7569beec90db52a5840/> update to reference renamed sn_node crate/repo
 - <csr-id-b99e7dee3e72e703b47888e3ff03d2baa933b408/> fix merge bugs and readd some shell completion logic
 - <csr-id-e389ab24f2186fc515b115e736a06d20756ae031/> rename artifacts and paths to match new naming convention
   safe-cli --> sn_cli
   safe-authd --> sn_authd
   safe-ffi --> sn_ffi
 - <csr-id-d3d04e36b8fb52ce070aecc2b1f77eb158983427/> reenabling more transfer funcs
 - <csr-id-d5c1cd2808f9844b06b846ec10dfe05146137023/> further ffi cleanup
 - <csr-id-8f309dada1517afa10c263a52f5597429f764890/> update safe-cmd-test-utilities name to
 - <csr-id-8b9b481df5d124857abb02158739a6ded8f02af7/> remove mock/ffi builds + files
 - <csr-id-6f2dd39d783812a9b3abd774b6bebd4cde2d5a1e/> update jsonrpc-quic crate name to qjsonrpc
 - <csr-id-08f5fec47809bbf0aea61a939dfb3c909043703f/> upgrade multibase to v0.8.0
 - <csr-id-95200df5f310911294ee72153d10d13f2e4fb737/> tests updated for wallet changes
 - <csr-id-019370cfd0ace44c656caf45c17248f2a547dbbf/> update safe-authd crate name to sn_authd
 - <csr-id-873fe29ac9042b7ad28a29630d2c048bde3a7634/> reenable wallet apis
 - <csr-id-9e191132a4281c53bd4872a756888234adfc0e2a/> reenabling map
 - <csr-id-cb66e8f5a89872d018e48311738d96173ae8274c/> update safe-api repo/crate name to sn_api
   Have not changed the S3 bucket names.
   Also included renaming SAFE to Safe in any documentation as I came
   across it.
   Cargo fmt fixes included.
 - <csr-id-4320a059224ef6018f7fc067f4a40a6534beeebb/> sn_client updated
 - <csr-id-426158fcbb6d7c1fe44755c138bba1ac825a0a0c/> use dirs_next for dir finding
 - <csr-id-0b17fddbe3418531df1d03a82d4eb510b819b811/> clippy
 - <csr-id-532aed1ed8e6b3957627ff2cc2f9d10d87fe3cb2/> getting tests compiling
 - <csr-id-5a1cd2790b159e35c734dfb1fe64a43ea4409dfc/> reenabling some authd functionality
 - <csr-id-0a5c18d115820f7124050bc0a246503b5cc63fd9/> reenabling some money apis
 - <csr-id-b863e7eb299472b0c9dbd633b1b892cc221efb46/> sn_data_type updates
 - <csr-id-371e7f00e6463063c99beb9823d8684355359d2a/> use core
 - <csr-id-4466c48a3fcec76f6c90cf6fcf1f28b177978c90/> safe_nd -> sn_data_types
 - <csr-id-1e4c47aed1aeed3488e370ab0c33a7b5519e40f5/> initial tweaks for app / auth changes

### New Features (BREAKING)

 - <csr-id-8787f07281e249a344a217d7d5b0e732a7dd7959/> easy to use nrs_add and rigorous nrs_create

### Bug Fixes (BREAKING)

 - <csr-id-7ffda3021fb36533f22538b1100acfa71b13cd81/> nrs get with versions, nrs_map always returned

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 205 commits contributed to the release over the course of 431 calendar days.
 - 173 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 1 unique issue was worked on: [#6](https://github.com/maidsafe/safe_network/issues/6)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **[#6](https://github.com/maidsafe/safe_network/issues/6)**
    - Drusu/review/feat multimap api ([`c63bb0f`](https://github.com/maidsafe/safe_network/commit/c63bb0fbfa8d661aef085dfc0582c6b537fd1349))
 * **Uncategorized**
    - safe_network v0.40.0/sn_api v0.39.0 ([`7001573`](https://github.com/maidsafe/safe_network/commit/70015730c3e08881f803e9ce59be7ca16185ae11))
    - safe_network v0.39.0 ([`ab00cf0`](https://github.com/maidsafe/safe_network/commit/ab00cf08d217654c57449437348b73576a65e89f))
    - update bls_dkg and blsttc to 0.7 and 0.3.4 respectively ([`213cb39`](https://github.com/maidsafe/safe_network/commit/213cb39be8fbfdf614f3eb6248b14fe161927a14))
    - safe_network v0.38.0 ([`cbdf023`](https://github.com/maidsafe/safe_network/commit/cbdf0236f7cd241e7addb1afe75ed5e5cfac00ab))
    - fix api tests. ([`3da9b26`](https://github.com/maidsafe/safe_network/commit/3da9b26f467cb8c468ffb0a319559a5a0f60e86e))
    - (cargo-release) version 0.37.0 ([`ef42af4`](https://github.com/maidsafe/safe_network/commit/ef42af483725286dd8a0cef6f922dfd1739412d8))
    - set `formatting` feature for `time` dependency ([`4bb4839`](https://github.com/maidsafe/safe_network/commit/4bb48397cbe321bd551ec9530ca0325208dd138f))
    - add env var for client query timeout ([`abfe737`](https://github.com/maidsafe/safe_network/commit/abfe7378604a74119accd7b9f86bef5682b0784a))
    - enable tests ([`d9c35b7`](https://github.com/maidsafe/safe_network/commit/d9c35b79a6d0aa787e627b747e665d96bb110c13))
    - NRS remove test ([`d0cb767`](https://github.com/maidsafe/safe_network/commit/d0cb7674071694885ebf9620a8cefcc4e62ecfcc))
    - NRS register failures ([`5582e99`](https://github.com/maidsafe/safe_network/commit/5582e995193ba17de3a3d0837271b405957d28ee))
    - bump rust edition ([`fc10d03`](https://github.com/maidsafe/safe_network/commit/fc10d037d64efc86796f1b1c6f255a4c7f91d3e1))
    - adapt test ([`60cc59c`](https://github.com/maidsafe/safe_network/commit/60cc59ce18406609f36a37861afa920b96dcac99))
    - easy to use nrs_add and rigorous nrs_create ([`8787f07`](https://github.com/maidsafe/safe_network/commit/8787f07281e249a344a217d7d5b0e732a7dd7959))
    - nrs get with versions, nrs_map always returned ([`7ffda30`](https://github.com/maidsafe/safe_network/commit/7ffda3021fb36533f22538b1100acfa71b13cd81))
    - appease clippy ([`407efd1`](https://github.com/maidsafe/safe_network/commit/407efd15e0b4854864b83ccdb7d2c3adbb0a02e2))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_cli_into_workspace ([`eea8307`](https://github.com/maidsafe/safe_network/commit/eea83074b9bbd334d80b80f12cfcce724d0e8ca3))
    - update cli to use new nrs api ([`8691034`](https://github.com/maidsafe/safe_network/commit/86910340897256bb4df77b6edaa0f2c9584d6dce))
    - switch from `chrono` to `time` ([`442c1bd`](https://github.com/maidsafe/safe_network/commit/442c1bdd3f9c22424aa9642d71d4016c868b0b58))
    - upgrade `tracing-appender` and `tracing-subscriber` ([`0387123`](https://github.com/maidsafe/safe_network/commit/0387123114ff6ae42920577706497319c8a888cb))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_api_into_workspace ([`8ed5aff`](https://github.com/maidsafe/safe_network/commit/8ed5aff8b30ce798f71eac22d66eb3aa9b0bdcdd))
    - upgrade sn_api to use 0.36.x of safe_network ([`0118362`](https://github.com/maidsafe/safe_network/commit/01183625d7a1a60b652b1a295a908fa8ba04f6f7))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_api_into_workspace ([`3a2817a`](https://github.com/maidsafe/safe_network/commit/3a2817a4c802d74b57d475d88d7bc23223994147))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_api_into_workspace ([`50f48ae`](https://github.com/maidsafe/safe_network/commit/50f48aefcba272345df7d4cd45a59071a5844932))
    - Merge branch 'merge_sn_api_into_workspace' into nrs_resolver_refactor ([`a273d97`](https://github.com/maidsafe/safe_network/commit/a273d9733b8d50b94b0ea3faec1d9e721d86aa27))
    - various renamings and changes matching PR comments ([`335f9dc`](https://github.com/maidsafe/safe_network/commit/335f9dcfc4588624728b4b10c576953d51a08e1a))
    - merge github.com:maidsafe/sn_cli into safe_network ([`414aca2`](https://github.com/maidsafe/safe_network/commit/414aca284b35f1bcb27e5d0cca2bfe451b69e27b))
    - fmt ([`49d04e1`](https://github.com/maidsafe/safe_network/commit/49d04e1414bf517cc76ebe2c6b86e0b3dd48e47a))
    - resolution test except for the blob range tests ([`c88a1bc`](https://github.com/maidsafe/safe_network/commit/c88a1bc5fda40093bb129b4351eef73d2eb7c041))
    - update actions workflows for workspace refactor ([`3703819`](https://github.com/maidsafe/safe_network/commit/3703819c7f0da220c8ff21169ca1e8161a20157b))
    - NRS all tests except for the register issue ([`2fe38d1`](https://github.com/maidsafe/safe_network/commit/2fe38d13bc74a7c7cede96340d275ef3f94e1427))
    - parsing Url as xorurl ([`6c61764`](https://github.com/maidsafe/safe_network/commit/6c61764c7479405d2978bec1bbc5cbd11ca5e7c8))
    - nrs and resolver ([`c0ac51a`](https://github.com/maidsafe/safe_network/commit/c0ac51ae4bf4dbd9df3dd39700887df439eec4f6))
    - update actions workflows for workspace refactor ([`d0134e8`](https://github.com/maidsafe/safe_network/commit/d0134e870bb097e095e1c8a33e607cf7994e6491))
    - move safe_network code into sn directory ([`2254329`](https://github.com/maidsafe/safe_network/commit/225432908839359800d301d9e5aa8274e4652ee1))
    - move sn_api code into an sn_api directory ([`2b5d177`](https://github.com/maidsafe/safe_network/commit/2b5d17740ca74fc379cab89cb95683e200589148))
    - add an example app showcasing how to fetch a Blob from Safe using sn_api ([`9098cff`](https://github.com/maidsafe/safe_network/commit/9098cff4c4d2ba15321dd072c970a18781a04a49))
    - Version change: sn_api v0.26.0; sn_cli v0.26.0; sn_authd v0.8.0 ([`3bcf8ef`](https://github.com/maidsafe/safe_network/commit/3bcf8efcee84c5fb45f5e03ec49d5a623147dc4d))
    - Version change: sn_api v0.25.3; sn_cli v0.25.3; sn_authd v0.7.3 ([`ab68342`](https://github.com/maidsafe/safe_network/commit/ab683420665c54df1ae3dae95055000518b543d1))
    - Version change: sn_api v0.25.2; sn_cli v0.25.2; sn_authd v0.7.2 ([`0282dd6`](https://github.com/maidsafe/safe_network/commit/0282dd6edfce91ac25314bda7b6d87fd1ae621fe))
    - update sn_client to 0.54.4 ([`91e1f0c`](https://github.com/maidsafe/safe_network/commit/91e1f0cefe1eb6a6d70e0df3d4d3b6f97e76ecef))
    - remove sn_url crate form this repo and depend on the one just published on crates.io ([`de64d44`](https://github.com/maidsafe/safe_network/commit/de64d44d336611f4f7f427fd6b627a8c0880fd80))
    - minor changes to rebase with master ([`7ceab22`](https://github.com/maidsafe/safe_network/commit/7ceab22ae72535159db7fbfdc5832b1aea891388))
    - moving out safe_url mod as a standalone sn_url crate ([`5780fd1`](https://github.com/maidsafe/safe_network/commit/5780fd1d6ba480cb775fd66e53e41f02d97b3a94))
    - Version change: sn_api v0.25.1; sn_cli v0.25.1; sn_authd v0.7.1 ([`7a8860d`](https://github.com/maidsafe/safe_network/commit/7a8860d71776958fb93e91fefe157b3de4277a8c))
    - add feature gate for app-specific error ([`441be17`](https://github.com/maidsafe/safe_network/commit/441be176521c20f9372b86d6daffc44bb97207fc))
    - Version change: sn_api v0.25.0; sn_cli v0.25.0; sn_authd v0.7.0 ([`60717f1`](https://github.com/maidsafe/safe_network/commit/60717f1a09aac06911f01cb3a811731721ae5708))
    - change node_path arg of 'node bin-path' command to node-path ([`d91150d`](https://github.com/maidsafe/safe_network/commit/d91150d8f46003cc0fa7813e4ae907b187379de8))
    - fixing multimap_remove and adding multimap_get_by_hash API ([`c5b0f0b`](https://github.com/maidsafe/safe_network/commit/c5b0f0b630f673697367361508a30caf7ad787bd))
    - adding first set of tests for Multimap API ([`6f05016`](https://github.com/maidsafe/safe_network/commit/6f0501699b0d0620a7c9d2b013944f90884ca1c3))
    - adding Register and Multimap public APIs ([`129f83f`](https://github.com/maidsafe/safe_network/commit/129f83f9b3a22669bf0fe0b5787546e4e3a924a0))
    - styling by cargo fmt and clippy ([`8d0ac66`](https://github.com/maidsafe/safe_network/commit/8d0ac669b27247fed193cdc0543ab5c892bd7c0d))
    - allow immutable reference to fetch method ([`0beee4f`](https://github.com/maidsafe/safe_network/commit/0beee4f8a1e876c1a7f482ba3be554e5b1654a42))
    - removed more mut bindings to self methods ([`e6b5cb0`](https://github.com/maidsafe/safe_network/commit/e6b5cb012a8684c87c574ce98e55ad038d573d49))
    - allow aliases of Safe (immutable references) ([`cd3e1b7`](https://github.com/maidsafe/safe_network/commit/cd3e1b71ef70bb7a4e5a786bdd337c2797250fdf))
    - fix set_content_version api in SafeUrl API to update its internal state ([`419bc87`](https://github.com/maidsafe/safe_network/commit/419bc87f345b4790a880b5def3b5b791d920a679))
    - Version change: sn_api v0.24.0; sn_cli v0.24.0; sn_authd v0.6.0 ([`47e7f0a`](https://github.com/maidsafe/safe_network/commit/47e7f0aea943a568d767a5226b0d8e71414508bc))
    - support transfers to BLS public keys ([`4f6c526`](https://github.com/maidsafe/safe_network/commit/4f6c526e194f1b949c1b5b126442150157b7b0ba))
    - removing unused files ([`ce56a35`](https://github.com/maidsafe/safe_network/commit/ce56a3504c8f27bfeb13bdf9051c2e91409230ea))
    - re-enabling dry-run feature for Blob API as well as for CLI commands ([`0f2f3c7`](https://github.com/maidsafe/safe_network/commit/0f2f3c74dc81fefbc719e79f41af434023ac0462))
    - re-organising files, nrs and xorurl files into their own mod folders ([`afd5422`](https://github.com/maidsafe/safe_network/commit/afd5422945fd1fc4ac509713e72471076ea4aee0))
    - when the --to value of keys transfer command doesn't start with safe:// and cannot be decoded as PK fallback to assume it's a URL ([`cb8ea4d`](https://github.com/maidsafe/safe_network/commit/cb8ea4d6740f0f5e69be941608bcd620ed94e549))
    - Version change: sn_api v0.23.2; sn_cli v0.23.2; sn_authd v0.5.2 ([`e939702`](https://github.com/maidsafe/safe_network/commit/e939702fdc1986c0021cf12223cbc707589b889f))
    - Version change: sn_api v0.23.1; sn_cli v0.23.1; sn_authd v0.5.1 ([`6aa920e`](https://github.com/maidsafe/safe_network/commit/6aa920e07a42b85ca8d081b8c93e7290553bb7ca))
    - Version change: sn_api v0.23.0; sn_cli v0.23.0; sn_authd v0.5.0 ([`e506e06`](https://github.com/maidsafe/safe_network/commit/e506e06acd50467834e80ebb15a3221261b45752))
    - fix styling by cargo fmt ([`a4b7d9b`](https://github.com/maidsafe/safe_network/commit/a4b7d9ba415831308c5ec6d902722843715a2d97))
    - pass approriate type ([`ce1823a`](https://github.com/maidsafe/safe_network/commit/ce1823a437ffe59db01cfb412821fc7becd4634a))
    - pass SecretKey by reference ([`5499aeb`](https://github.com/maidsafe/safe_network/commit/5499aeb2f755ce363b709c5379b860048c92ce5a))
    - Version change: sn_api v0.22.0; sn_cli v0.22.0; sn_authd v0.4.0 ([`fedab1b`](https://github.com/maidsafe/safe_network/commit/fedab1b7bc6c01b8be07ae2c54c034514bc70717))
    - update sn_client and sn_data_types to latest ([`0d4755e`](https://github.com/maidsafe/safe_network/commit/0d4755ed64a65c223bad253d9d7a03980ec12e8d))
    - init logger in sn_api tests ([`6a8a173`](https://github.com/maidsafe/safe_network/commit/6a8a173acf96d98d00548659b4641488f5fec2ee))
    - Version change: sn_api v0.21.0; sn_cli v0.21.0; sn_authd v0.3.0; qjsonrpc v0.2.0 ([`838238d`](https://github.com/maidsafe/safe_network/commit/838238d745a18aa28a8b366ab4adc62745656990))
    - fixing files and NRS APIs tests ([`b9bae62`](https://github.com/maidsafe/safe_network/commit/b9bae620533559c5671075fb7a3fe576fe14431f))
    - upgrade tokio to v1.3.0 and quinn to v0.10.1 ([`d77859a`](https://github.com/maidsafe/safe_network/commit/d77859a8138de0ddcd6b121b928efe13e0254e81))
    - adapting to new Seq type where Policy is immutable ([`85462b6`](https://github.com/maidsafe/safe_network/commit/85462b6fb58f16f4797d7ef2816e96a287af7ad9))
    - customise the error message displayed when a panic occurred ([`45f11ae`](https://github.com/maidsafe/safe_network/commit/45f11ae22df242e229d01bfc5dc2b6ac9de8536d))
    - re-enabling wallet APIs tests in CI ([`6cf5602`](https://github.com/maidsafe/safe_network/commit/6cf56022cd6beec4254d0b1667c4d654f87e6a5a))
    - re-enabling xorurl & keys APIs tests in CI ([`4518154`](https://github.com/maidsafe/safe_network/commit/4518154481dbd3aeb397353d4ec296ea98ee3e9a))
    - re-enabling sequence & fetch APIs tests in CI ([`590c4c6`](https://github.com/maidsafe/safe_network/commit/590c4c634046ab655a84093c2ed60e8289415d44))
    - Version change: sn_api v0.20.0; sn_cli v0.20.0; sn_authd v0.2.0; qjsonrpc v0.1.2 ([`a35ffb7`](https://github.com/maidsafe/safe_network/commit/a35ffb759bafd6e2b03d96bffa62747eb1965c89))
    - remove unused utility function ([`a09c4c2`](https://github.com/maidsafe/safe_network/commit/a09c4c26230b0cf60d7d792d89f0e275f2b64bc2))
    - store serialised NrsMap in a Blob and only its XOR-URL in the NrsMapContainer's underlying Sequence ([`ddb318d`](https://github.com/maidsafe/safe_network/commit/ddb318d96c8a9bf8299bd14aa6824252296bde27))
    - store serialised FilesMap in a Blob and only its XOR-URL in the FilesContainer's underlying Sequence ([`1f37703`](https://github.com/maidsafe/safe_network/commit/1f377038ea693d5253f658d480885662df5b0542))
    - reorganise files module ([`a6107ac`](https://github.com/maidsafe/safe_network/commit/a6107accfc6950b4beb2dcb84dfaa3c0e18bbd5d))
    - adding a README.md file for sn_api crate with a description of the current APIs ([`712d9a4`](https://github.com/maidsafe/safe_network/commit/712d9a4e72f62725a0b0ac5526bc68abe8ca503f))
    - Version change: sn_api v0.19.1; sn_cli v0.19.1 ([`edbdcb6`](https://github.com/maidsafe/safe_network/commit/edbdcb62c36a2998aab23dd3a4d0b13bae13b472))
    - update sn_client and data types ([`6c9cf24`](https://github.com/maidsafe/safe_network/commit/6c9cf24df423abae568fab63fc6615d9f7a3df68))
    - Version change: sn_api-v0.19.0; sn_cli-v0.19.0; sn_authd-v0.1.1; qjsonrpc-v0.1.1 ([`21f4733`](https://github.com/maidsafe/safe_network/commit/21f4733fbc32efd2c822337c7b3f077cca0f2992))
    - adding a step to check for unused dependencies ([`de482a5`](https://github.com/maidsafe/safe_network/commit/de482a5611333d069076d7da1b7c5a6017db65eb))
    - add a simple example app which uploads a file using the API ([`7d00d0a`](https://github.com/maidsafe/safe_network/commit/7d00d0ad089924915aa2bf564b5b925825aa4880))
    - use get_sequence_entry ([`d910e86`](https://github.com/maidsafe/safe_network/commit/d910e8607c8898bb8d8ccb0c824d8a5401a2c938))
    - upgrade sn_client to v0.46.12 and most of all dependencies to their latest published version ([`e3c6da3`](https://github.com/maidsafe/safe_network/commit/e3c6da38f92c354c560bd6b555d76f698779ebcf))
    - Remove unused ed25519 dependency ([`6f67254`](https://github.com/maidsafe/safe_network/commit/6f672547ee9a55b241aa6004b353d6919fb0e8cb))
    - update tiny-keccak from 1.5.0 to 2.0.2 ([`792bce8`](https://github.com/maidsafe/safe_network/commit/792bce8dd94192f17c51d6a1c0b63c7c214ad7c3))
    - connect to port 12000 by default in local testnets ([`472df87`](https://github.com/maidsafe/safe_network/commit/472df87678c07624d6163b8105f110cd22e4e3c9))
    - upgrade sn_client to v0.46.9 and solve clippy issues ([`b61e837`](https://github.com/maidsafe/safe_network/commit/b61e83716cce00c0ba02f3d50bf060cfc095051a))
    - set bootstrapping contacts for api-tests ([`6d90943`](https://github.com/maidsafe/safe_network/commit/6d909431d6a7164a06f2322d188c1d4764e5d0b8))
    - provide bootstrapping contacts list to sn_client as required by new sn_client API ([`43c675e`](https://github.com/maidsafe/safe_network/commit/43c675ee514aa73fb5192717dae58c97587521e7))
    - command to configure networks in the config by providing a list of IP and ports ([`2a43ca8`](https://github.com/maidsafe/safe_network/commit/2a43ca8fb10dcdbf085890643c673491399b1a8b))
    - add auth version subcommand which prints out the authd binary version ([`e366c87`](https://github.com/maidsafe/safe_network/commit/e366c878da84d2cf051bbc692e6b80c675ef8393))
    - migrating to use anyhow for CLI errors and use thiserror for sn_api error types ([`c2c6716`](https://github.com/maidsafe/safe_network/commit/c2c6716d29e56f387776202dad94ddda9b8fe2b2))
    - Version change: sn_api-v0.18.0; sn_cli--v0.18.0; sn_authd-v0.1.0; qjsonrpc-0.1.0 ([`fce96bf`](https://github.com/maidsafe/safe_network/commit/fce96bfb00279be41a139a360d1b2eac02d874cf))
    - upgrading sn_client to v0.46.2 ([`f0bef9e`](https://github.com/maidsafe/safe_network/commit/f0bef9e5e79381abf27eec7fae28f4ce0fecb370))
    - changes to remove any use of Arc for keypairs and secret keys ([`4ba83c7`](https://github.com/maidsafe/safe_network/commit/4ba83c720fabcace7a2859ad308be5922a6597c0))
    - update sn_client and dts ([`b38d840`](https://github.com/maidsafe/safe_network/commit/b38d840320d65b09ce85db9074f7b7a9487f83df))
    - Money -> Token ([`b04c1ca`](https://github.com/maidsafe/safe_network/commit/b04c1ca2090a32c423ffe2c23ac22be9f5ebbcf3))
    - Invalidate nrs names with troublesome characters ([`eabdd42`](https://github.com/maidsafe/safe_network/commit/eabdd4254e0725a8556a413d786966a00d9e6e3d))
    - Validate length of nrs name and subname ([`0f578dc`](https://github.com/maidsafe/safe_network/commit/0f578dc0079973ed9d1b49fa04c222d7f3460a6b))
    - Remove unused check for whitespace ([`71852f5`](https://github.com/maidsafe/safe_network/commit/71852f56e490cc3b0f31f15005c142eced0a72e1))
    - Invalidate public names containing slash char ([`4a33102`](https://github.com/maidsafe/safe_network/commit/4a331029c6d6f18fdf487f86bd6130a0455b0a4b))
    - Remove repeated 'cannot cannot' in error message ([`7cb0969`](https://github.com/maidsafe/safe_network/commit/7cb0969522f9845cddff8a4bd52725f500697975))
    - do not attempt to retry fetching a Sequence entry if not found the first time ([`2dff02d`](https://github.com/maidsafe/safe_network/commit/2dff02dc71bc3574763906c8592d32bde64337c9))
    - default to check balance of key assigned to CLI when no sk is provided to keys balance command ([`7a77ef4`](https://github.com/maidsafe/safe_network/commit/7a77ef4e3f3730d2f033d0365b2446bd560b1e21))
    - adapt to new error returned by sn-client when Map entry is not found ([`bfb09f2`](https://github.com/maidsafe/safe_network/commit/bfb09f28d9ff304b80354b971f8f02ffbf2a60c5))
    - minor logging improvements and upgrade sn_client to v0.44.16 ([`5ba32bd`](https://github.com/maidsafe/safe_network/commit/5ba32bd9a86e9c44cf3d59f54be899dc3e2ca861))
    - upgrade sn_client to v0.44.15 ([`4f89812`](https://github.com/maidsafe/safe_network/commit/4f89812ed5ca3394d2cd7b93e3c79aac2929d11d))
    - Version change: sn_api-v0.17.1; sn_cli-v0.17.1; sn_authd-v0.0.14 ([`3961969`](https://github.com/maidsafe/safe_network/commit/396196997f6d114b01e5b269447b3c4219250f35))
    - fix all clippy issues after updating to rust 1.49 ([`67b746f`](https://github.com/maidsafe/safe_network/commit/67b746f607501511c38fe752f64119a12985ab72))
    - minor change to error returned when parsing pk from hex ([`6e4ea36`](https://github.com/maidsafe/safe_network/commit/6e4ea368fdcedb10042b5d8dc94ab02eece47003))
    - support transfers to Ed25519 public keys in addition to using a Wallet or SafeKey URLs ([`c20932f`](https://github.com/maidsafe/safe_network/commit/c20932f5c15fa16ccad907208522b9c9b52bb062))
    - keypair API now returns a new randomly create Ed25519 key pair ([`f2589e0`](https://github.com/maidsafe/safe_network/commit/f2589e047a5ac68834f6d0d3ce49a32a0e7ddab0))
    - minor enhancement to logs when instantiating safe client ([`84260fc`](https://github.com/maidsafe/safe_network/commit/84260fc32473c9a84f5c3e6fd54564a865f9d7fe))
    - return anyhow::Result/Error from all CLI tests ([`f6f0734`](https://github.com/maidsafe/safe_network/commit/f6f07349a7524304b3d6b1c22db65d77be519f4c))
    - remove Error::Unexpected and Error::Unknown errors from API ([`e692bec`](https://github.com/maidsafe/safe_network/commit/e692becbbf09e2500284cb1507916fac56149f02))
    - Version change: sn_api-v0.17.0; sn_cli-v0.17.0; sn_authd-v0.0.13 ([`23365d4`](https://github.com/maidsafe/safe_network/commit/23365d409b1a538b2eb8c5138623a409e45f9601))
    - temporarily disable running tests in CI ([`77e1c69`](https://github.com/maidsafe/safe_network/commit/77e1c693735760cf1c24c9a4552c9104e52cada0))
    - allocate testcoin to keypair generated for app upon being authorised the very first time ([`9f18fb5`](https://github.com/maidsafe/safe_network/commit/9f18fb51a8c173ebb07e3ab9cea7a8f66b373fc5))
    - updates and enhancements to the User Guide, and to some commands help messages ([`40bcd0f`](https://github.com/maidsafe/safe_network/commit/40bcd0f46dad6177b0052b73393d7789fd559b33))
    - Update sn_api/src/api/authenticator/mod.rs ([`7209603`](https://github.com/maidsafe/safe_network/commit/72096033f5bacdfbeb5c4a5da0a4b52d74f499de))
    - check for existing balance + err when creating acct ([`7d2bd2f`](https://github.com/maidsafe/safe_network/commit/7d2bd2f7ad27724866d04b7e10d69418f6decad9))
    - updating CLI User Guide ([`e107e13`](https://github.com/maidsafe/safe_network/commit/e107e1314957053db2d71357450cac65cba52a68))
    - store Blob API was returning the wrong xorname ([`3694efb`](https://github.com/maidsafe/safe_network/commit/3694efb16b16fcfc0d34db51187c043d0e24f09c))
    - adapt wallet tests and minor refactoring ([`7fb6bd9`](https://github.com/maidsafe/safe_network/commit/7fb6bd96a8bdaaee64592b5dc02596b9f6220165))
    - adapt tests to new transfer costs and minor refactor to transfers errors handling ([`7746947`](https://github.com/maidsafe/safe_network/commit/774694795114dc392db5219393fa63f204fcc905))
    - re-enable implementation for coins transfers with Ed25519 keys ([`b2e3faf`](https://github.com/maidsafe/safe_network/commit/b2e3faf9b0943ec779b1e513c76179048dbb0db3))
    - keys_create_preload_test_coins was not triggering the simulated payout on the newly created SafeKey ([`5a30bf3`](https://github.com/maidsafe/safe_network/commit/5a30bf331242ba8dd9b3189dc255b134fdf24587))
    - ed_sk_from_hex was deserialising sk with bincode rather than just from raw bytes ([`01cc289`](https://github.com/maidsafe/safe_network/commit/01cc2894b37908377eb822a826f46c7fef39347e))
    - remove unwrap instances from prod and test code ([`422547f`](https://github.com/maidsafe/safe_network/commit/422547f9081de77538f2241c727ac55b00e1e48b))
    - minor adjustments on cli test utilities ([`e3235b0`](https://github.com/maidsafe/safe_network/commit/e3235b0329c74b20092278c628a2115d50e206d7))
    - properly serialise key pairs in CLI commands output ([`ae026bb`](https://github.com/maidsafe/safe_network/commit/ae026bb9ce91b1373b8b300c41bfef0c3f295c7a))
    - minor reorganisation to cli test scripts ([`f9e0729`](https://github.com/maidsafe/safe_network/commit/f9e07293ea1f8cd5e4428d95a299ba06c0f30a20))
    - insert and retrieve authorised apps keypairs into/from the Safe ([`d8186c3`](https://github.com/maidsafe/safe_network/commit/d8186c309a7e6ca4862fb8855da6636a9f0d84c6))
    - minor renamings in authd status report with new terminology ([`5ac36cc`](https://github.com/maidsafe/safe_network/commit/5ac36cc64566561f4d442058c91b9857622e6f26))
    - populate each keypair generated for authorised apps with testcoins ([`b916d7b`](https://github.com/maidsafe/safe_network/commit/b916d7bdc83e9a02fd29e9cbd6623fc922066a6c))
    - adapt to latest sn-client api changes and further simplification of auth messages ([`b6eddcb`](https://github.com/maidsafe/safe_network/commit/b6eddcbf5d272e6a4430cfd6488f5236bef92a5d))
    - update credentials location ([`7b6445a`](https://github.com/maidsafe/safe_network/commit/7b6445a5b9903b1704c45759878bced097bcb82c))
    - simplify authd messages format and serialisation ([`e814ff3`](https://github.com/maidsafe/safe_network/commit/e814ff3b8c58ae7741938a1c73a22c87ed602883))
    - reenable use of connect from auth responses ([`d64d0fd`](https://github.com/maidsafe/safe_network/commit/d64d0fd9c0d6cd9002405985ff03f6d9ff7aa695))
    - fmt ([`b261640`](https://github.com/maidsafe/safe_network/commit/b261640cebdaf4f7fb1b9c13e911bf82bb46f33a))
    - simplify loop on err ([`274de45`](https://github.com/maidsafe/safe_network/commit/274de45a63122c9685617c97272deb603dd9a15b))
    - tidying up some logs ([`f9c19fa`](https://github.com/maidsafe/safe_network/commit/f9c19faa33db0fb76d392d8cbcd7d910131fedb2))
    - unwrap_or_else -> _or_ for empty funcs ([`e892a99`](https://github.com/maidsafe/safe_network/commit/e892a99f2feb0c36204cfff103b13ca2e5e96388))
    - loop seq gets w/ timeout ([`0cd75fb`](https://github.com/maidsafe/safe_network/commit/0cd75fbd9aceace5fbfc4b726a48d652177d61ce))
    - fix lint issues ([`9c3adff`](https://github.com/maidsafe/safe_network/commit/9c3adffd0889f045ac19110072a194072d294705))
    - adapt authd client api and service names to new terminology of Safe creation and unlocking ([`58ecf7f`](https://github.com/maidsafe/safe_network/commit/58ecf7fbae95dee0a103ce39d61efaac6e2cf550))
    - store generated keypairs on the network ([`805298f`](https://github.com/maidsafe/safe_network/commit/805298fa5f4d1455015d2aa248143f460b4057ba))
    - unused variable `address` ([`eb88c0c`](https://github.com/maidsafe/safe_network/commit/eb88c0cc1a8e7a21d7af4004c1e1a0a49297f22c))
    - remove repeated data storing in SafeAppClient::store_sequence() ([`87d64dc`](https://github.com/maidsafe/safe_network/commit/87d64dc3fe865b68f014da373aa253caf50d6512))
    - generate_random_ed_keypair ([`19ca515`](https://github.com/maidsafe/safe_network/commit/19ca515ac84d8cf5d99c4b2cba25561248597f13))
    - fix proptest regex ([`4db2c27`](https://github.com/maidsafe/safe_network/commit/4db2c27badcccac6d7368566ee6f483613c3aa93))
    - update sn_client ([`52c8c8d`](https://github.com/maidsafe/safe_network/commit/52c8c8d7d2f5a5f9c7d2862bcf3fca1902048695))
    - update pass/phrase derivation proptest ([`cfc3580`](https://github.com/maidsafe/safe_network/commit/cfc35809120cdf2144c8df5dc509eb316bcb0068))
    - clippy ([`eb2364c`](https://github.com/maidsafe/safe_network/commit/eb2364c77d6e755d6e184735e50db366faf266d2))
    - add login/creation seedable pk tests ([`9272bb5`](https://github.com/maidsafe/safe_network/commit/9272bb55edf690ccd33de5904530e7ff8036c0fe))
    - reliably derives seed ([`0e6ffb0`](https://github.com/maidsafe/safe_network/commit/0e6ffb0a3c216d891e6a60ac162d733d2fa28690))
    - clippy ([`2de7f66`](https://github.com/maidsafe/safe_network/commit/2de7f66f3f732b9dae55dad50f15888513b5a125))
    - setting up for no ClientId ([`b5a6d81`](https://github.com/maidsafe/safe_network/commit/b5a6d8115ad3975a17dd973430480adf6c483490))
    - Batch of changes for sk handling ([`29a978f`](https://github.com/maidsafe/safe_network/commit/29a978f3047464ad8014817e331218372b53c06c))
    - dep updates for dt and client ([`7780541`](https://github.com/maidsafe/safe_network/commit/77805418d129cb2924dc35b6a88f704771ef6e5c))
    - converting to more generic data types for keypair sk pk ([`dfabea0`](https://github.com/maidsafe/safe_network/commit/dfabea0a26f97f420f47ba314cae0882aae47dca))
    - small tidy ([`34ef8e8`](https://github.com/maidsafe/safe_network/commit/34ef8e8c31de6e013d3539a0ec595b32f9301194))
    - clippy ([`106407e`](https://github.com/maidsafe/safe_network/commit/106407e8125cc003794ba6249158aa1a655d3357))
    - reenable decoding auth reqs and basic app key generation ([`3f23687`](https://github.com/maidsafe/safe_network/commit/3f23687471b846e3ad1e2492c237a21f212b753f))
    - remove merge added file ([`c56e265`](https://github.com/maidsafe/safe_network/commit/c56e26502ee44af179bfa65c8a194a2190b16842))
    - clippy ([`e0a4de8`](https://github.com/maidsafe/safe_network/commit/e0a4de8de20e5023b83256d39beca19f759ba129))
    - tidying up ([`4905fae`](https://github.com/maidsafe/safe_network/commit/4905fae6259063411c5e4ef5fd2afb531980630c))
    - fix merge bugs and readd some shell completion logic ([`b99e7de`](https://github.com/maidsafe/safe_network/commit/b99e7dee3e72e703b47888e3ff03d2baa933b408))
    - Merge branch 'master' into ExploreUpdatesForApis ([`34f9bc7`](https://github.com/maidsafe/safe_network/commit/34f9bc704f301ac903f768813fbd4140cd702f21))
    - reenabling more transfer funcs ([`d3d04e3`](https://github.com/maidsafe/safe_network/commit/d3d04e36b8fb52ce070aecc2b1f77eb158983427))
    - further ffi cleanup ([`d5c1cd2`](https://github.com/maidsafe/safe_network/commit/d5c1cd2808f9844b06b846ec10dfe05146137023))
    - remove mock/ffi builds + files ([`8b9b481`](https://github.com/maidsafe/safe_network/commit/8b9b481df5d124857abb02158739a6ded8f02af7))
    - upgrade multibase to v0.8.0 ([`08f5fec`](https://github.com/maidsafe/safe_network/commit/08f5fec47809bbf0aea61a939dfb3c909043703f))
    - tests updated for wallet changes ([`95200df`](https://github.com/maidsafe/safe_network/commit/95200df5f310911294ee72153d10d13f2e4fb737))
    - reenable wallet apis ([`873fe29`](https://github.com/maidsafe/safe_network/commit/873fe29ac9042b7ad28a29630d2c048bde3a7634))
    - reenabling map ([`9e19113`](https://github.com/maidsafe/safe_network/commit/9e191132a4281c53bd4872a756888234adfc0e2a))
    - sn_client updated ([`4320a05`](https://github.com/maidsafe/safe_network/commit/4320a059224ef6018f7fc067f4a40a6534beeebb))
    - use dirs_next for dir finding ([`426158f`](https://github.com/maidsafe/safe_network/commit/426158fcbb6d7c1fe44755c138bba1ac825a0a0c))
    - clippy ([`0b17fdd`](https://github.com/maidsafe/safe_network/commit/0b17fddbe3418531df1d03a82d4eb510b819b811))
    - getting tests compiling ([`532aed1`](https://github.com/maidsafe/safe_network/commit/532aed1ed8e6b3957627ff2cc2f9d10d87fe3cb2))
    - reenabling some authd functionality ([`5a1cd27`](https://github.com/maidsafe/safe_network/commit/5a1cd2790b159e35c734dfb1fe64a43ea4409dfc))
    - reenabling some money apis ([`0a5c18d`](https://github.com/maidsafe/safe_network/commit/0a5c18d115820f7124050bc0a246503b5cc63fd9))
    - setting up IPC for auth ([`b994b8d`](https://github.com/maidsafe/safe_network/commit/b994b8d6ec1fcfc540e91aa9df79ba849aee7647))
    - sn_data_type updates ([`b863e7e`](https://github.com/maidsafe/safe_network/commit/b863e7eb299472b0c9dbd633b1b892cc221efb46))
    - use core ([`371e7f0`](https://github.com/maidsafe/safe_network/commit/371e7f00e6463063c99beb9823d8684355359d2a))
    - moving in basics of auth func into the repo ([`07dd195`](https://github.com/maidsafe/safe_network/commit/07dd1956d4a53e2f4d09310b48e2a43a3a10e795))
    - safe_nd -> sn_data_types ([`4466c48`](https://github.com/maidsafe/safe_network/commit/4466c48a3fcec76f6c90cf6fcf1f28b177978c90))
    - initial tweaks for app / auth changes ([`1e4c47a`](https://github.com/maidsafe/safe_network/commit/1e4c47aed1aeed3488e370ab0c33a7b5519e40f5))
    - update to reference renamed sn_client ([`5f8d61c`](https://github.com/maidsafe/safe_network/commit/5f8d61cf41eb547c13b65d2030beefd235b75820))
    - update to reference renamed sn_node crate/repo ([`ee05ed3`](https://github.com/maidsafe/safe_network/commit/ee05ed31cb12d8e1d8bac7569beec90db52a5840))
    - rename artifacts and paths to match new naming convention ([`e389ab2`](https://github.com/maidsafe/safe_network/commit/e389ab24f2186fc515b115e736a06d20756ae031))
    - update safe-cmd-test-utilities name to ([`8f309da`](https://github.com/maidsafe/safe_network/commit/8f309dada1517afa10c263a52f5597429f764890))
    - update jsonrpc-quic crate name to qjsonrpc ([`6f2dd39`](https://github.com/maidsafe/safe_network/commit/6f2dd39d783812a9b3abd774b6bebd4cde2d5a1e))
    - update safe-authd crate name to sn_authd ([`019370c`](https://github.com/maidsafe/safe_network/commit/019370cfd0ace44c656caf45c17248f2a547dbbf))
    - update safe-api repo/crate name to sn_api ([`cb66e8f`](https://github.com/maidsafe/safe_network/commit/cb66e8f5a89872d018e48311738d96173ae8274c))
</details>

