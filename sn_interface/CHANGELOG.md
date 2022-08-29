# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## v0.10.2 (2022-08-28)

### New Features

 - <csr-id-7cc2a00907381e93db266f31545b12ff76907e5d/> implement `SecuredLinkedList` as a `MerkleRegister`
 - <csr-id-b87617e44e9b20b8a79864e30e29ecee86444352/> return error to client on unknown section key
   If one of the spent proofs sent by the client have been signed with a key this section is not
   currently aware of, return an error back to the client.
   
   This introduces a new SpentProofUnknownSectionKey variant to the messaging data errors, because none
   of the existing variants seemed appropriate for this scenario.

### Other

 - <csr-id-b587893737bc51aee483f7cd53da782036dd6c5e/> unit tests for spentbook handler
   Provide unit test coverage for the `SpentbookCmd::Spent` message handler.
   
   It's important to note that at this point, the failure cases only assert that no commands were
   returned from the handler, because this is the way we deal with failures at the moment.
   Unfortunately this means it's easy for there to be false positives because you can't check the error
   type or message. I will look into changing this as a separate PR.
   
   Most of the changes here are related to testing infrastructure:
   * Support setting a threshold when a secret key set is generated for the section. For use with the
     genesis DBC generation, the threshold had to be set to 0.
   * Support adults in the test section. The spent message generates data to be replicated on adults,
     so the mechanisms for creating a test section were extended for this. There are now
     `create_section` and `create_section_with_elders` functions, because some existing tests require
     the condition where only elders have been marked as members.
   * The genesis DBC is needed for these tests, so the scope of the function for generating it was
     changed to `pub(crate)`.
   * The `Cmd` struct was extended in the test module to provide utils to get at the content of
     messages, which are used for test verification.
   * Provide util function for wrapping a `ServiceMsg` inside a `WireMsg` and so on. Keeps the testing
     code cleaner.
   * Provide util function for extracting the spent proof share from the replicated data so that we can
     verify the message handler assigned the correct values to its fields.
   * Various util functions related to the use of DBCs were provided in a `dbc_utils` module. The doc
     comments on the functions should hopefully make clear what they are for.
   
   A couple of superficial changes were also made to the message handler code:
   * The key image sent by the client is validated (along with a test case for that).
   * Change the format of debugging messages and comments to be more uniform.
   * Move some code into functions scoped at `pub(crate)`. This is so they can be shared for use with
     test setup. For further explanation, see the doc comments on these functions in the diff.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 2 calendar days.
 - 2 days passed between releases.
 - 3 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - implement `SecuredLinkedList` as a `MerkleRegister` ([`7cc2a00`](https://github.com/maidsafe/safe_network/commit/7cc2a00907381e93db266f31545b12ff76907e5d))
    - return error to client on unknown section key ([`b87617e`](https://github.com/maidsafe/safe_network/commit/b87617e44e9b20b8a79864e30e29ecee86444352))
    - unit tests for spentbook handler ([`b587893`](https://github.com/maidsafe/safe_network/commit/b587893737bc51aee483f7cd53da782036dd6c5e))
</details>

## v0.10.1 (2022-08-25)

### Chore

 - <csr-id-401bc416c7aea65ae55e9adee2cbecf782c999cf/> sn_interface-0.10.1/sn_client-0.71.0/sn_node-0.66.1/sn_api-0.69.0/sn_cli-0.62.0
 - <csr-id-d58f1c55e9502fd6e8a99509f7ca30640835458b/> make RegisterCmdId a hex-encodedstring
 - <csr-id-fd6b97b37bb875404ef2ba7f5f35d5675c122ea0/> make RegisterCmds be stored under deterministic id

### Bug Fixes

 - <csr-id-175011ea4a14ef0ce2538ce9e69a6ffc8d47f2ac/> append RegsiterId as hex for storage folder
   Previously we used bitdepth which can clash for low depths, even for
   unique xornames.
   
   Now we also add the register folder id name, so we know all ops in a
   given folder are for that register.
 - <csr-id-604556e670d5fe0a9408bbd0d586363c7b4c0d6c/> Decode ReplicatedDataAddress from chunk filename
   We were previously encoding a ReplicatedDataAddress, but
   decoding as a ChunkAddress
 - <csr-id-4da782096826f2074dac2a5628f9c9d9a85fcf1f/> paths for read/write RegisterCmd ops and support any order for reading them

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 1 day passed between releases.
 - 6 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.10.1/sn_client-0.71.0/sn_node-0.66.1/sn_api-0.69.0/sn_cli-0.62.0 ([`401bc41`](https://github.com/maidsafe/safe_network/commit/401bc416c7aea65ae55e9adee2cbecf782c999cf))
    - append RegsiterId as hex for storage folder ([`175011e`](https://github.com/maidsafe/safe_network/commit/175011ea4a14ef0ce2538ce9e69a6ffc8d47f2ac))
    - Decode ReplicatedDataAddress from chunk filename ([`604556e`](https://github.com/maidsafe/safe_network/commit/604556e670d5fe0a9408bbd0d586363c7b4c0d6c))
    - make RegisterCmdId a hex-encodedstring ([`d58f1c5`](https://github.com/maidsafe/safe_network/commit/d58f1c55e9502fd6e8a99509f7ca30640835458b))
    - paths for read/write RegisterCmd ops and support any order for reading them ([`4da7820`](https://github.com/maidsafe/safe_network/commit/4da782096826f2074dac2a5628f9c9d9a85fcf1f))
    - make RegisterCmds be stored under deterministic id ([`fd6b97b`](https://github.com/maidsafe/safe_network/commit/fd6b97b37bb875404ef2ba7f5f35d5675c122ea0))
</details>

## v0.10.0 (2022-08-23)

<csr-id-2c8cbdf06993e86f7e5575c5dc856721a5ed08b7/>
<csr-id-c8517a481e39bf688041cd8f8661bc663ee7bce7/>
<csr-id-589f03ce8670544285f329fe35c19897d4bfced8/>
<csr-id-9f64d681e285de57a54f571e98ff68f1bf39b6f1/>
<csr-id-836c1ba8d17d380e8504325e14f46739e2688bb3/>
<csr-id-1618cf6a93117942946d152efee24fe3c7020e55/>
<csr-id-11b8182a3de636a760d899cb15d7184d8153545a/>
<csr-id-e52028f1e9d7fcf19962a7643b272ba3a786c7c4/>
<csr-id-28d95a2e959e32ee69a70bdc855cba1fff1fc8d8/>
<csr-id-d3f66d6cfa838a5c65fb8f31fa68d48794b33dea/>
<csr-id-f0fbe5fd9bec0b2865271bb139c9fcb4ec225884/>

### Chore

 - <csr-id-2c8cbdf06993e86f7e5575c5dc856721a5ed08b7/> update tokio
 - <csr-id-c8517a481e39bf688041cd8f8661bc663ee7bce7/> fix clippy some/none issues
 - <csr-id-589f03ce8670544285f329fe35c19897d4bfced8/> upgrading sn_dbc to v8.0
 - <csr-id-9f64d681e285de57a54f571e98ff68f1bf39b6f1/> increase data query limit
   Now we differentiate queries per adult/index, we may need more queries.
 - <csr-id-836c1ba8d17d380e8504325e14f46739e2688bb3/> check members need updating before verifying.
   During merge members we were spending a lot of CPU verifying, when we may not actually need the udpate at all

### Chore

 - <csr-id-43fcc7c517f95eab0e27ddc79cd9c6de3631c7c6/> sn_interface-0.10.0/sn_dysfunction-0.9.0/sn_client-0.70.0/sn_node-0.66.0/sn_api-0.68.0/sn_cli-0.61.0

### New Features

 - <csr-id-f0f860efcf89cb7bf51bddd6364a9bec33bbf3c3/> remove ConnectivityCheck
   Now we have periodic health checks and dysfunciton, this
   check should not be needed, and can cause network strain
   with the frequent DKG we have now
 - <csr-id-e97ab2220d150706741549944c6e4bf77f2a5bae/> new cmd to display detailed information about a configured network
 - <csr-id-438716cf9cfc11685968b10ccba8deffff96e56e/> include span in module path
 - <csr-id-108997c0ca4c291b8628b4a349732b3a23802d5a/> simplify log format to '<timestamp> [module] LEVEL <msg>'
 - <csr-id-1e2a0a122f8c53d669916cded16876aa16d8ebfb/> make AntiEntropyProbe carry a current known section key for response

### Bug Fixes

 - <csr-id-d529bd61de83795b2b10cce12549374cd9521a4f/> add fallback if only single prefix

### Refactor

 - <csr-id-1618cf6a93117942946d152efee24fe3c7020e55/> expose serialisation/deserialisation utilities as public methods instead
   - Also include the genesis key of each network in the list shown by CLI networks cmd.
 - <csr-id-11b8182a3de636a760d899cb15d7184d8153545a/> clean up unused functionality
   `closest` is a method that will find a prefix that is closest, but if
   not returning any, it means the set is empty. The `closest_or_opposite`
   used this function internally, but actually never got to the opposite,
   because `closest` would always return a SAP.
   
   This method was used in a few places where no exclusions were given, so
   it is clear in that case that it would always find a prefix. In a single
   case, it was called with an exclusion, where it would find a section
   closer than its own section.
 - <csr-id-e52028f1e9d7fcf19962a7643b272ba3a786c7c4/> SAP reference instead of clone

### New Features (BREAKING)

 - <csr-id-991ccd452119137d9da046b7f222f091177e28f1/> adding more context information to sn_client::Error types

### Refactor (BREAKING)

 - <csr-id-28d95a2e959e32ee69a70bdc855cba1fff1fc8d8/> removing unused CreateRegister::Populated msg type
 - <csr-id-d3f66d6cfa838a5c65fb8f31fa68d48794b33dea/> removing unused sn_node::dbs::Error variants and RegisterExtend cmd
 - <csr-id-f0fbe5fd9bec0b2865271bb139c9fcb4ec225884/> renaming NetworkPrefixMap to SectionTree
   - Changing CLI and sn_client default path for network contacts to `$HOME/.safe/network_contacts`.
   - Renaming variables and functions referring to "prefix map" to now refer to "network contacts".

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 19 commits contributed to the release over the course of 8 calendar days.
 - 9 days passed between releases.
 - 19 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.10.0/sn_dysfunction-0.9.0/sn_client-0.70.0/sn_node-0.66.0/sn_api-0.68.0/sn_cli-0.61.0 ([`43fcc7c`](https://github.com/maidsafe/safe_network/commit/43fcc7c517f95eab0e27ddc79cd9c6de3631c7c6))
    - remove ConnectivityCheck ([`f0f860e`](https://github.com/maidsafe/safe_network/commit/f0f860efcf89cb7bf51bddd6364a9bec33bbf3c3))
    - removing unused CreateRegister::Populated msg type ([`28d95a2`](https://github.com/maidsafe/safe_network/commit/28d95a2e959e32ee69a70bdc855cba1fff1fc8d8))
    - removing unused sn_node::dbs::Error variants and RegisterExtend cmd ([`d3f66d6`](https://github.com/maidsafe/safe_network/commit/d3f66d6cfa838a5c65fb8f31fa68d48794b33dea))
    - update tokio ([`2c8cbdf`](https://github.com/maidsafe/safe_network/commit/2c8cbdf06993e86f7e5575c5dc856721a5ed08b7))
    - fix clippy some/none issues ([`c8517a4`](https://github.com/maidsafe/safe_network/commit/c8517a481e39bf688041cd8f8661bc663ee7bce7))
    - new cmd to display detailed information about a configured network ([`e97ab22`](https://github.com/maidsafe/safe_network/commit/e97ab2220d150706741549944c6e4bf77f2a5bae))
    - adding more context information to sn_client::Error types ([`991ccd4`](https://github.com/maidsafe/safe_network/commit/991ccd452119137d9da046b7f222f091177e28f1))
    - upgrading sn_dbc to v8.0 ([`589f03c`](https://github.com/maidsafe/safe_network/commit/589f03ce8670544285f329fe35c19897d4bfced8))
    - renaming NetworkPrefixMap to SectionTree ([`f0fbe5f`](https://github.com/maidsafe/safe_network/commit/f0fbe5fd9bec0b2865271bb139c9fcb4ec225884))
    - include span in module path ([`438716c`](https://github.com/maidsafe/safe_network/commit/438716cf9cfc11685968b10ccba8deffff96e56e))
    - simplify log format to '<timestamp> [module] LEVEL <msg>' ([`108997c`](https://github.com/maidsafe/safe_network/commit/108997c0ca4c291b8628b4a349732b3a23802d5a))
    - expose serialisation/deserialisation utilities as public methods instead ([`1618cf6`](https://github.com/maidsafe/safe_network/commit/1618cf6a93117942946d152efee24fe3c7020e55))
    - increase data query limit ([`9f64d68`](https://github.com/maidsafe/safe_network/commit/9f64d681e285de57a54f571e98ff68f1bf39b6f1))
    - add fallback if only single prefix ([`d529bd6`](https://github.com/maidsafe/safe_network/commit/d529bd61de83795b2b10cce12549374cd9521a4f))
    - clean up unused functionality ([`11b8182`](https://github.com/maidsafe/safe_network/commit/11b8182a3de636a760d899cb15d7184d8153545a))
    - SAP reference instead of clone ([`e52028f`](https://github.com/maidsafe/safe_network/commit/e52028f1e9d7fcf19962a7643b272ba3a786c7c4))
    - check members need updating before verifying. ([`836c1ba`](https://github.com/maidsafe/safe_network/commit/836c1ba8d17d380e8504325e14f46739e2688bb3))
    - make AntiEntropyProbe carry a current known section key for response ([`1e2a0a1`](https://github.com/maidsafe/safe_network/commit/1e2a0a122f8c53d669916cded16876aa16d8ebfb))
</details>

## v0.9.0 (2022-08-14)

<csr-id-66c26782759be707edb922daa548e3f0a3f9be8c/>
<csr-id-6d60525874dc4efeb658433f1f253d54e0cba2d4/>
<csr-id-29de67f1e3583eab867d517cb50ed2e404bd63fd/>
<csr-id-8242f2f1035b1c0718e53954951badffa30f3393/>
<csr-id-848dba48e5959d0b9cfe182fde2f12ede71ba9c2/>
<csr-id-35483b3f322eeea2c10427e94e4750a8269811c0/>
<csr-id-820fcc9a77f756fca308f247c3ea1b82f65d30b9/>
<csr-id-afcf083469c732f10c7c80f4a45e4c33ab111101/>
<csr-id-17f0e8a08c9543d380c16a35d3d7bfe7834a9e5a/>
<csr-id-aafc560d3b3b1e375f7be224e0e63a3b567bbd86/>
<csr-id-7394030fe5aeeb88f4524d2da2a71e36334c831d/>
<csr-id-73dc9b4a1757393270e62d265328bab0c0aa3b35/>
<csr-id-0a653e4becc4a8e14ffd6d0752cf035430067ce9/>
<csr-id-9789797e3f773285f23bd22957fe45a67aabec24/>
<csr-id-08af2a6ac3485a696d2a1e799af588943f207e6b/>
<csr-id-702c33b0d78f4a459725ed0c4538819c949978ce/>
<csr-id-2ea069543dbe6ffebac663d4d8d7e0bc33cfc566/>
<csr-id-322c69845e2e14eb029fdbebb24e08063a2323b0/>
<csr-id-ea490ddf749ac9e0c7962c3c21c053663e6b6ee7/>
<csr-id-bf2902c18b900b8b4a8abae5f966d1e08d547910/>
<csr-id-6f03b93bd2d02f0ffe54b69fbf25070fbe64eab0/>
<csr-id-8b3c4eb06fa988dc97b0cb75ed615ec69af29a48/>
<csr-id-214adedc31bca576c7f28ff52a1f4ff0a2676757/>
<csr-id-39c3fdf4128462e5f7c5fec3c628d394f505e2f2/>
<csr-id-1e8180c23fab27ac92c93f201efd050cff00db10/>
<csr-id-44cea00f54b39eaea0936ec187a8fa9ccdb61661/>
<csr-id-847db2c487cd102af0cf9a477b4c1b65fc2c8aa6/>
<csr-id-0a5593b0512d6f059c6a8003634b07e7d2d3e514/>
<csr-id-707b80c3526ae727a7e91330dc386cdb41c51f4c/>
<csr-id-9bd6ae20c1207f99420093fd5c9f4eb53836e3c1/>
<csr-id-31d9f9f99b4e166986b8e51c3d41e0eac55621a4/>
<csr-id-30a7028dd702e2f6575e299a609a2416439cbaed/>
<csr-id-dedec486f85c1cf6cf2d538238f32e826e08da0a/>
<csr-id-879678e986a722d216ee9a4f37e8ae398221a394/>
<csr-id-629a5873dd3bdf138649360222c00e3e0a76e097/>
<csr-id-12360a6dcc204153a81adbf842a64dc018c750f9/>
<csr-id-27ba2a63dcfa272cf7ef8c5301987fc6bfe18ed0/>
<csr-id-6e65ed8e6c5872bd2c49a1ed2837b1fb16523af1/>
<csr-id-6b1fee8cf3d0b2995f4b81e59dd684547593b5fa/>
<csr-id-ed37bb56e5e17d4cba7c1b2165746c193241d618/>
<csr-id-a0c89ff0e451d2e5dd13fc29635075097f2c7b94/>
<csr-id-0f07efd9ef0b75de79f27772566b013bc886bcc8/>
<csr-id-db4f4d07b155d732ad76d263563d81b5fee535f7/>
<csr-id-ff1a10b4aa2b41b7028949101504a29b52927e71/>
<csr-id-e0fb940b24e87d86fe920095176362f73503ce79/>
<csr-id-35ebd8e872f9d9db16c42cbe8d61702f9660aece/>
<csr-id-3f577d2a6fe70792d7d02e231b599ca3d44a5ed2/>
<csr-id-5c82df633e7c062fdf761a8e6e0a7ae8d26cc73a/>
<csr-id-24676dadb771bbd966b6a3e1aa53d1c736c90627/>
<csr-id-93614b18b4316af04ab8c74358a5c86510590b85/>
<csr-id-116b109ecdaaf0f1f10ede96cd68977782ab9ea3/>
<csr-id-f5f73acb035159718780a08f3d10512b11558e59/>
<csr-id-d3a05a728be8752ea9ebff4e38e7c4c85e5db09b/>
<csr-id-96da1171d0cac240f772e5d6a15c56f63441b4b3/>
<csr-id-dd2eb21352223f6340064e0021f4a7df402cd5c9/>
<csr-id-53f60c2327f8a69f0b2ef6d1a4e96644c10aa358/>

### Chore

 - <csr-id-66c26782759be707edb922daa548e3f0a3f9be8c/> add partial eq for rust 1.63; dep updates
 - <csr-id-6d60525874dc4efeb658433f1f253d54e0cba2d4/> remove wiremsg.priority as uneeded
 - <csr-id-29de67f1e3583eab867d517cb50ed2e404bd63fd/> serialize NetworkPrefixMap into JSON
 - <csr-id-8242f2f1035b1c0718e53954951badffa30f3393/> organise usings, cleanup
 - <csr-id-848dba48e5959d0b9cfe182fde2f12ede71ba9c2/> use matches! macros, minor refactoring
 - <csr-id-35483b3f322eeea2c10427e94e4750a8269811c0/> remove unused async/await
 - <csr-id-820fcc9a77f756fca308f247c3ea1b82f65d30b9/> remove NetworkPrefxiMap::genesis_key, NetworkKnowledge::genesis_key
   Remove the feilds as they can be obtained from NetworkPrefixMap::sections_dag
 - <csr-id-afcf083469c732f10c7c80f4a45e4c33ab111101/> remove RwLock from NetworkPrefixMap
 - <csr-id-17f0e8a08c9543d380c16a35d3d7bfe7834a9e5a/> make NetowrkPrefixMap::sections_dag private
 - <csr-id-aafc560d3b3b1e375f7be224e0e63a3b567bbd86/> rename traceroute fns
 - <csr-id-7394030fe5aeeb88f4524d2da2a71e36334c831d/> traceroute update after cmds flow rebase
 - <csr-id-73dc9b4a1757393270e62d265328bab0c0aa3b35/> make traceroute a default feature
 - <csr-id-0a653e4becc4a8e14ffd6d0752cf035430067ce9/> improve Display, Debug impl for Traceroute
 - <csr-id-9789797e3f773285f23bd22957fe45a67aabec24/> improve traceroute readability and other improvements
   - simplfies creating identites for traceroute to avoid locking
   - implements Display and Debug for traceroute
   - add clearer logs for traceroute
 - <csr-id-08af2a6ac3485a696d2a1e799af588943f207e6b/> clarify fn signatures
 - <csr-id-702c33b0d78f4a459725ed0c4538819c949978ce/> cleanup cache interface let binding

 - <csr-id-2ea069543dbe6ffebac663d4d8d7e0bc33cfc566/> remove RwLock over Cache type
 - <csr-id-322c69845e2e14eb029fdbebb24e08063a2323b0/> remove write lock around non query service msg handling
 - <csr-id-ea490ddf749ac9e0c7962c3c21c053663e6b6ee7/> reflect the semantics not the type
 - <csr-id-bf2902c18b900b8b4a8abae5f966d1e08d547910/> whitespace + typo fix
 - <csr-id-6f03b93bd2d02f0ffe54b69fbf25070fbe64eab0/> upgrade blsttc to 7.0.0
   This version has a more helpful error message for the shares interpolation problem.
 - <csr-id-8b3c4eb06fa988dc97b0cb75ed615ec69af29a48/> add traceroute feature to testnet bin
 - <csr-id-214adedc31bca576c7f28ff52a1f4ff0a2676757/> improve traceroute redability and resolve clippy
 - <csr-id-39c3fdf4128462e5f7c5fec3c628d394f505e2f2/> remove unused console-subscriber
 - <csr-id-1e8180c23fab27ac92c93f201efd050cff00db10/> re-enable registers benchmark and tidy sled residue
 - <csr-id-44cea00f54b39eaea0936ec187a8fa9ccdb61661/> refactor WireMsg.serialize to use BytesMut and avoid some Vec allocations
 - <csr-id-847db2c487cd102af0cf9a477b4c1b65fc2c8aa6/> remove locking from section key cache
 - <csr-id-0a5593b0512d6f059c6a8003634b07e7d2d3e514/> remove refcell on NetworkKnowledge::all_section_chains
 - <csr-id-707b80c3526ae727a7e91330dc386cdb41c51f4c/> remove refcell around NetworkKnowledge::signed_sap
 - <csr-id-9bd6ae20c1207f99420093fd5c9f4eb53836e3c1/> remove refcell on NetworkKnowledge::chain
 - <csr-id-31d9f9f99b4e166986b8e51c3d41e0eac55621a4/> remove awaits from tests as well
 - <csr-id-30a7028dd702e2f6575e299a609a2416439cbaed/> remove locking around signature aggregation
 - <csr-id-dedec486f85c1cf6cf2d538238f32e826e08da0a/> remove unused async
 - <csr-id-879678e986a722d216ee9a4f37e8ae398221a394/> logging sn_consensus in CI, tweak section min and max elder age

### Chore

 - <csr-id-53f60c2327f8a69f0b2ef6d1a4e96644c10aa358/> sn_interface-0.9.0/sn_dysfunction-0.8.0/sn_client-0.69.0/sn_node-0.65.0/sn_api-0.67.0/sn_cli-0.60.0

### Documentation

 - <csr-id-e3c90998e1abd10768e861370a65a934f52e2ec3/> broken links

### New Features

 - <csr-id-df8ea32f9218d344cd1f291359969b38a05b4642/> join approval has membership decision
 - <csr-id-a6fb1fc516a9ef6dae7aa236f3dd440d50697ae2/> impl traceroute feature to trace a message's flow in the network
   - implements traceroute for Client queries and is logged at the client on return

### Bug Fixes

<csr-id-1694c1566ac562216447eb491cc3b2b00b0c5979/>
<csr-id-19bb0b99afee53dd7b6e109919249b25e0a55e48/>
<csr-id-f0d1abf6dd8731310b7749cd6cc7077886215997/>
<csr-id-f6ea1da4a57e40a051c7d1ee3b87fe9b442c537b/>
<csr-id-d2bc1167288724b05d70ae7a305c88c93eac611a/>

 - <csr-id-8bf0aeed0af193322f341bd718f7a5f84fa2d02f/> gossip all votes and start timer after first vote
 - <csr-id-0ed5075304b090597f7760fb51c4a33435a853f1/> fix deadlock introduced after removal of Arc from NetworkPrefixMap
   Removing the checks in compare_and_write_prefix_map and directly
   writing the prefix_map fixed the issue
 - <csr-id-cebf37ea8ef44c51ce84646a83a5d1e6dcab3e7a/> use correct data address type
 - <csr-id-a5028c551d1b3db2c4c912c2897490e9a4b34a0d/> disable rejoins
   It seems that we may have not fully thought through the implications
   of nodes rejoining.
   
   The flow was:
   1. a node leaves the section (reason is not tracked)
2. the node rejoins
3. the section accepts them back and attempts to relocate them

### Other

 - <csr-id-629a5873dd3bdf138649360222c00e3e0a76e097/> remove test files from /resources/test_prefixmap/
   Generate and write prefixmap for the tests instead of storing a copy

### Refactor

 - <csr-id-12360a6dcc204153a81adbf842a64dc018c750f9/> reorganise flow control unit tests
   The unit tests in the `flow_ctrl` module provide coverage for messaging handling in the node. To run
   each test, a `Node` must constructed, and this involves a lot of tedious setup code.
   
   A `network_utils` testing module is introduced to organise the code related to this setup and to
   also provide a `TestNodeBuilder` to reduce duplication and hopefully make it easier to provide more
   coverage for message handlers. The doc comments should hopefully make clear how the struct can be
   used in various different testing contexts. I will also be looking to extend its functionality a bit
   further when I come to unit testing the message handlers for the DBC spentbook commands.
   
   There were a few tests whose setup was too complex to use the builder because they require too much
   customisation and seem best left alone.
   
   A `cmd_utils` module is also introduced to organise the code for processing commands. I again also
   plan on extending this when considering the DBC tests.
 - <csr-id-27ba2a63dcfa272cf7ef8c5301987fc6bfe18ed0/> sn_client to only read a default prefix map file, updates to be cached on disk by user
   - CLI to cache the up to date PrefixMap after all commands were executed and right before exiting.
   - Refactoring sn_cli::Config to remove some redundant code.
 - <csr-id-6e65ed8e6c5872bd2c49a1ed2837b1fb16523af1/> remove NetworkKnowledge::chain
 - <csr-id-6b1fee8cf3d0b2995f4b81e59dd684547593b5fa/> reduce AE msgs to one msg with a kind field
 - <csr-id-ed37bb56e5e17d4cba7c1b2165746c193241d618/> move SectionChain into NetworkPrefixMap
 - <csr-id-a0c89ff0e451d2e5dd13fc29635075097f2c7b94/> do not require node write lock on query
 - <csr-id-0f07efd9ef0b75de79f27772566b013bc886bcc8/> remove optional field
 - <csr-id-db4f4d07b155d732ad76d263563d81b5fee535f7/> remove more unused code
 - <csr-id-ff1a10b4aa2b41b7028949101504a29b52927e71/> simplify send msg
 - <csr-id-e0fb940b24e87d86fe920095176362f73503ce79/> use sn_dbc::SpentProof API for verifying SpentProofShares
 - <csr-id-35ebd8e872f9d9db16c42cbe8d61702f9660aece/> expose known keys on network knowledge
 - <csr-id-3f577d2a6fe70792d7d02e231b599ca3d44a5ed2/> rename gen_section_authority_provider to random_sap
 - <csr-id-5c82df633e7c062fdf761a8e6e0a7ae8d26cc73a/> setup step for tests to reissue a set of DBCs from genesis only once
 - <csr-id-24676dadb771bbd966b6a3e1aa53d1c736c90627/> replace sled with filestore for storing registers
 - <csr-id-93614b18b4316af04ab8c74358a5c86510590b85/> make chunk_store accept all datatypes

### Test

 - <csr-id-116b109ecdaaf0f1f10ede96cd68977782ab9ea3/> add proptest to make sure NetworkPrefixMap fields stay in sync
 - <csr-id-f5f73acb035159718780a08f3d10512b11558e59/> generate a random prefix map

### Chore (BREAKING)

 - <csr-id-d3a05a728be8752ea9ebff4e38e7c4c85e5db09b/> having spent proofs and Txs within SpentbookCmd::Send msg to be a set instead of a vec

### Refactor (BREAKING)

 - <csr-id-96da1171d0cac240f772e5d6a15c56f63441b4b3/> nodes to cache their own individual prefix map file on disk
 - <csr-id-dd2eb21352223f6340064e0021f4a7df402cd5c9/> removing Token from sn_interfaces::type as it is now exposed by sn_dbc

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 70 commits contributed to the release over the course of 31 calendar days.
 - 34 days passed between releases.
 - 68 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.9.0/sn_dysfunction-0.8.0/sn_client-0.69.0/sn_node-0.65.0/sn_api-0.67.0/sn_cli-0.60.0 ([`53f60c2`](https://github.com/maidsafe/safe_network/commit/53f60c2327f8a69f0b2ef6d1a4e96644c10aa358))
    - add partial eq for rust 1.63; dep updates ([`66c2678`](https://github.com/maidsafe/safe_network/commit/66c26782759be707edb922daa548e3f0a3f9be8c))
    - reorganise flow control unit tests ([`12360a6`](https://github.com/maidsafe/safe_network/commit/12360a6dcc204153a81adbf842a64dc018c750f9))
    - sn_client to only read a default prefix map file, updates to be cached on disk by user ([`27ba2a6`](https://github.com/maidsafe/safe_network/commit/27ba2a63dcfa272cf7ef8c5301987fc6bfe18ed0))
    - remove wiremsg.priority as uneeded ([`6d60525`](https://github.com/maidsafe/safe_network/commit/6d60525874dc4efeb658433f1f253d54e0cba2d4))
    - gossip all votes and start timer after first vote ([`8bf0aee`](https://github.com/maidsafe/safe_network/commit/8bf0aeed0af193322f341bd718f7a5f84fa2d02f))
    - remove NetworkKnowledge::chain ([`6e65ed8`](https://github.com/maidsafe/safe_network/commit/6e65ed8e6c5872bd2c49a1ed2837b1fb16523af1))
    - serialize NetworkPrefixMap into JSON ([`29de67f`](https://github.com/maidsafe/safe_network/commit/29de67f1e3583eab867d517cb50ed2e404bd63fd))
    - nodes to cache their own individual prefix map file on disk ([`96da117`](https://github.com/maidsafe/safe_network/commit/96da1171d0cac240f772e5d6a15c56f63441b4b3))
    - reduce AE msgs to one msg with a kind field ([`6b1fee8`](https://github.com/maidsafe/safe_network/commit/6b1fee8cf3d0b2995f4b81e59dd684547593b5fa))
    - removing Token from sn_interfaces::type as it is now exposed by sn_dbc ([`dd2eb21`](https://github.com/maidsafe/safe_network/commit/dd2eb21352223f6340064e0021f4a7df402cd5c9))
    - organise usings, cleanup ([`8242f2f`](https://github.com/maidsafe/safe_network/commit/8242f2f1035b1c0718e53954951badffa30f3393))
    - use matches! macros, minor refactoring ([`848dba4`](https://github.com/maidsafe/safe_network/commit/848dba48e5959d0b9cfe182fde2f12ede71ba9c2))
    - remove unused async/await ([`35483b3`](https://github.com/maidsafe/safe_network/commit/35483b3f322eeea2c10427e94e4750a8269811c0))
    - remove test files from /resources/test_prefixmap/ ([`629a587`](https://github.com/maidsafe/safe_network/commit/629a5873dd3bdf138649360222c00e3e0a76e097))
    - remove NetworkPrefxiMap::genesis_key, NetworkKnowledge::genesis_key ([`820fcc9`](https://github.com/maidsafe/safe_network/commit/820fcc9a77f756fca308f247c3ea1b82f65d30b9))
    - fix deadlock introduced after removal of Arc from NetworkPrefixMap ([`0ed5075`](https://github.com/maidsafe/safe_network/commit/0ed5075304b090597f7760fb51c4a33435a853f1))
    - remove RwLock from NetworkPrefixMap ([`afcf083`](https://github.com/maidsafe/safe_network/commit/afcf083469c732f10c7c80f4a45e4c33ab111101))
    - add proptest to make sure NetworkPrefixMap fields stay in sync ([`116b109`](https://github.com/maidsafe/safe_network/commit/116b109ecdaaf0f1f10ede96cd68977782ab9ea3))
    - generate a random prefix map ([`f5f73ac`](https://github.com/maidsafe/safe_network/commit/f5f73acb035159718780a08f3d10512b11558e59))
    - make NetowrkPrefixMap::sections_dag private ([`17f0e8a`](https://github.com/maidsafe/safe_network/commit/17f0e8a08c9543d380c16a35d3d7bfe7834a9e5a))
    - move SectionChain into NetworkPrefixMap ([`ed37bb5`](https://github.com/maidsafe/safe_network/commit/ed37bb56e5e17d4cba7c1b2165746c193241d618))
    - rename traceroute fns ([`aafc560`](https://github.com/maidsafe/safe_network/commit/aafc560d3b3b1e375f7be224e0e63a3b567bbd86))
    - traceroute update after cmds flow rebase ([`7394030`](https://github.com/maidsafe/safe_network/commit/7394030fe5aeeb88f4524d2da2a71e36334c831d))
    - make traceroute a default feature ([`73dc9b4`](https://github.com/maidsafe/safe_network/commit/73dc9b4a1757393270e62d265328bab0c0aa3b35))
    - improve Display, Debug impl for Traceroute ([`0a653e4`](https://github.com/maidsafe/safe_network/commit/0a653e4becc4a8e14ffd6d0752cf035430067ce9))
    - improve traceroute readability and other improvements ([`9789797`](https://github.com/maidsafe/safe_network/commit/9789797e3f773285f23bd22957fe45a67aabec24))
    - use correct data address type ([`cebf37e`](https://github.com/maidsafe/safe_network/commit/cebf37ea8ef44c51ce84646a83a5d1e6dcab3e7a))
    - do not require node write lock on query ([`a0c89ff`](https://github.com/maidsafe/safe_network/commit/a0c89ff0e451d2e5dd13fc29635075097f2c7b94))
    - remove optional field ([`0f07efd`](https://github.com/maidsafe/safe_network/commit/0f07efd9ef0b75de79f27772566b013bc886bcc8))
    - remove more unused code ([`db4f4d0`](https://github.com/maidsafe/safe_network/commit/db4f4d07b155d732ad76d263563d81b5fee535f7))
    - clarify fn signatures ([`08af2a6`](https://github.com/maidsafe/safe_network/commit/08af2a6ac3485a696d2a1e799af588943f207e6b))
    - simplify send msg ([`ff1a10b`](https://github.com/maidsafe/safe_network/commit/ff1a10b4aa2b41b7028949101504a29b52927e71))
    - use sn_dbc::SpentProof API for verifying SpentProofShares ([`e0fb940`](https://github.com/maidsafe/safe_network/commit/e0fb940b24e87d86fe920095176362f73503ce79))
    - cleanup cache interface let binding ([`702c33b`](https://github.com/maidsafe/safe_network/commit/702c33b0d78f4a459725ed0c4538819c949978ce))
    - remove RwLock over Cache type ([`2ea0695`](https://github.com/maidsafe/safe_network/commit/2ea069543dbe6ffebac663d4d8d7e0bc33cfc566))
    - remove write lock around non query service msg handling ([`322c698`](https://github.com/maidsafe/safe_network/commit/322c69845e2e14eb029fdbebb24e08063a2323b0))
    - having spent proofs and Txs within SpentbookCmd::Send msg to be a set instead of a vec ([`d3a05a7`](https://github.com/maidsafe/safe_network/commit/d3a05a728be8752ea9ebff4e38e7c4c85e5db09b))
    - reflect the semantics not the type ([`ea490dd`](https://github.com/maidsafe/safe_network/commit/ea490ddf749ac9e0c7962c3c21c053663e6b6ee7))
    - Tweak docs of `known_keys` ([`f0b105f`](https://github.com/maidsafe/safe_network/commit/f0b105f48c10a42b3d8434f3900700cd2ec2e484))
    - expose known keys on network knowledge ([`35ebd8e`](https://github.com/maidsafe/safe_network/commit/35ebd8e872f9d9db16c42cbe8d61702f9660aece))
    - whitespace + typo fix ([`bf2902c`](https://github.com/maidsafe/safe_network/commit/bf2902c18b900b8b4a8abae5f966d1e08d547910))
    - disable rejoins ([`a5028c5`](https://github.com/maidsafe/safe_network/commit/a5028c551d1b3db2c4c912c2897490e9a4b34a0d))
    - prevent rejoins of archived nodes ([`1694c15`](https://github.com/maidsafe/safe_network/commit/1694c1566ac562216447eb491cc3b2b00b0c5979))
    - rename gen_section_authority_provider to random_sap ([`3f577d2`](https://github.com/maidsafe/safe_network/commit/3f577d2a6fe70792d7d02e231b599ca3d44a5ed2))
    - join approval has membership decision ([`df8ea32`](https://github.com/maidsafe/safe_network/commit/df8ea32f9218d344cd1f291359969b38a05b4642))
    - upgrade blsttc to 7.0.0 ([`6f03b93`](https://github.com/maidsafe/safe_network/commit/6f03b93bd2d02f0ffe54b69fbf25070fbe64eab0))
    - adds unique conn info validation to membership ([`19bb0b9`](https://github.com/maidsafe/safe_network/commit/19bb0b99afee53dd7b6e109919249b25e0a55e48))
    - add traceroute feature to testnet bin ([`8b3c4eb`](https://github.com/maidsafe/safe_network/commit/8b3c4eb06fa988dc97b0cb75ed615ec69af29a48))
    - improve traceroute redability and resolve clippy ([`214aded`](https://github.com/maidsafe/safe_network/commit/214adedc31bca576c7f28ff52a1f4ff0a2676757))
    - impl traceroute feature to trace a message's flow in the network ([`a6fb1fc`](https://github.com/maidsafe/safe_network/commit/a6fb1fc516a9ef6dae7aa236f3dd440d50697ae2))
    - broken links ([`e3c9099`](https://github.com/maidsafe/safe_network/commit/e3c90998e1abd10768e861370a65a934f52e2ec3))
    - remove redundant generation field ([`f0d1abf`](https://github.com/maidsafe/safe_network/commit/f0d1abf6dd8731310b7749cd6cc7077886215997))
    - Merge #1356 ([`d9b4608`](https://github.com/maidsafe/safe_network/commit/d9b46080ac849cac259983dc80b4b879e58c13ba))
    - remove unused console-subscriber ([`39c3fdf`](https://github.com/maidsafe/safe_network/commit/39c3fdf4128462e5f7c5fec3c628d394f505e2f2))
    - setup step for tests to reissue a set of DBCs from genesis only once ([`5c82df6`](https://github.com/maidsafe/safe_network/commit/5c82df633e7c062fdf761a8e6e0a7ae8d26cc73a))
    - re-enable registers benchmark and tidy sled residue ([`1e8180c`](https://github.com/maidsafe/safe_network/commit/1e8180c23fab27ac92c93f201efd050cff00db10))
    - replace sled with filestore for storing registers ([`24676da`](https://github.com/maidsafe/safe_network/commit/24676dadb771bbd966b6a3e1aa53d1c736c90627))
    - make chunk_store accept all datatypes ([`93614b1`](https://github.com/maidsafe/safe_network/commit/93614b18b4316af04ab8c74358a5c86510590b85))
    - refactor WireMsg.serialize to use BytesMut and avoid some Vec allocations ([`44cea00`](https://github.com/maidsafe/safe_network/commit/44cea00f54b39eaea0936ec187a8fa9ccdb61661))
    - cleanup unused async ([`f6ea1da`](https://github.com/maidsafe/safe_network/commit/f6ea1da4a57e40a051c7d1ee3b87fe9b442c537b))
    - box LRUCache to avoid stackoverflow ([`d2bc116`](https://github.com/maidsafe/safe_network/commit/d2bc1167288724b05d70ae7a305c88c93eac611a))
    - remove locking from section key cache ([`847db2c`](https://github.com/maidsafe/safe_network/commit/847db2c487cd102af0cf9a477b4c1b65fc2c8aa6))
    - remove refcell on NetworkKnowledge::all_section_chains ([`0a5593b`](https://github.com/maidsafe/safe_network/commit/0a5593b0512d6f059c6a8003634b07e7d2d3e514))
    - remove refcell around NetworkKnowledge::signed_sap ([`707b80c`](https://github.com/maidsafe/safe_network/commit/707b80c3526ae727a7e91330dc386cdb41c51f4c))
    - remove refcell on NetworkKnowledge::chain ([`9bd6ae2`](https://github.com/maidsafe/safe_network/commit/9bd6ae20c1207f99420093fd5c9f4eb53836e3c1))
    - remove awaits from tests as well ([`31d9f9f`](https://github.com/maidsafe/safe_network/commit/31d9f9f99b4e166986b8e51c3d41e0eac55621a4))
    - remove locking around signature aggregation ([`30a7028`](https://github.com/maidsafe/safe_network/commit/30a7028dd702e2f6575e299a609a2416439cbaed))
    - remove unused async ([`dedec48`](https://github.com/maidsafe/safe_network/commit/dedec486f85c1cf6cf2d538238f32e826e08da0a))
    - logging sn_consensus in CI, tweak section min and max elder age ([`879678e`](https://github.com/maidsafe/safe_network/commit/879678e986a722d216ee9a4f37e8ae398221a394))
</details>

## v0.8.2 (2022-07-10)

<csr-id-49e223e2c07695b4c63e253ba19ce43ec24d7112/>
<csr-id-dce3ba214354ad007900efce78273670cfb725d5/>
<csr-id-34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8/>

### Chore

 - <csr-id-49e223e2c07695b4c63e253ba19ce43ec24d7112/> move more deps to clap-v3; rm some deps on rand

### Chore

 - <csr-id-34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8/> sn_interface-0.8.2/sn_client-0.68.2/sn_node-0.64.2/sn_api-0.66.3/sn_cli-0.59.3

### Bug Fixes

 - <csr-id-64eb333d532694f46f1d0b9dd5109961b3551802/> for QueryResponse, set correlation_id to be the origin msg_id
 - <csr-id-3c383ccf9ad0ed77080fb3e3ec459e5b02158505/> passing the churn test
   This commit contains the work to passing the churn test.
   There are mainly two fixes:
   1, Only trigger data reorganization when there is membership update.
   Previously, data reorganzation get undertaken whenever there is
   incoming message. Which result in a looping of messaging among
   nodes.
   2, Only broadcast result when the QueryResponse is not an error.
   Previously, this will cause the client thinking the whole query
   is failed whenever an error response received.

### Refactor

 - <csr-id-dce3ba214354ad007900efce78273670cfb725d5/> move dkg util method definitions onto the DKG structs

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 9 commits contributed to the release over the course of 2 calendar days.
 - 2 days passed between releases.
 - 5 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.2/sn_client-0.68.2/sn_node-0.64.2/sn_api-0.66.3/sn_cli-0.59.3 ([`34bd9bd`](https://github.com/maidsafe/safe_network/commit/34bd9bd01a3f042c35e0432df2f0cfcebc32a8a8))
    - move dkg util method definitions onto the DKG structs ([`dce3ba2`](https://github.com/maidsafe/safe_network/commit/dce3ba214354ad007900efce78273670cfb725d5))
    - move more deps to clap-v3; rm some deps on rand ([`49e223e`](https://github.com/maidsafe/safe_network/commit/49e223e2c07695b4c63e253ba19ce43ec24d7112))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`45418f2`](https://github.com/maidsafe/safe_network/commit/45418f2f9b5cc58f2a153bf40966beb2bf36a62a))
    - for QueryResponse, set correlation_id to be the origin msg_id ([`64eb333`](https://github.com/maidsafe/safe_network/commit/64eb333d532694f46f1d0b9dd5109961b3551802))
    - passing the churn test ([`3c383cc`](https://github.com/maidsafe/safe_network/commit/3c383ccf9ad0ed77080fb3e3ec459e5b02158505))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`94be181`](https://github.com/maidsafe/safe_network/commit/94be181789b0010f83ed5e89341f3f347575e37f))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`44411d5`](https://github.com/maidsafe/safe_network/commit/44411d511a496b13893670c8bc7d9f43f0ce9073))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`45309c4`](https://github.com/maidsafe/safe_network/commit/45309c4c0463dd9198a49537187417bf4bfdb847))
</details>

## v0.8.1 (2022-07-07)

<csr-id-c79e2aac378b28b373fd7c18c4b9006348960071/>
<csr-id-46262268fc167c05963e5b7bd6261310496e2379/>
<csr-id-8dccb7f1fc81385f9f5f25e6c354ad1d35759528/>
<csr-id-2b00cec961561281f6b927e13e501342843f6a0f/>

### Chore

 - <csr-id-c79e2aac378b28b373fd7c18c4b9006348960071/> bit more low hanging clippy fruit
 - <csr-id-46262268fc167c05963e5b7bd6261310496e2379/> `try!` macro is deprecated
   No need for rustfmt to check/replace this, as the compiler will already
   warn for this. Deprecated since 1.39.
   
   Removing the option seems to trigger a couple of formatting changes that
   rustfmt did not seem to pick on before.
 - <csr-id-8dccb7f1fc81385f9f5f25e6c354ad1d35759528/> clippy runs cargo check already

### New Features

 - <csr-id-8313ed8d5b45b7f4ed3b36ada231e74c49c9f9e6/> perform signature verifications on input DBC SpentProof before signing new spent proof share

### Chore

 - <csr-id-2b00cec961561281f6b927e13e501342843f6a0f/> sn_interface-0.8.1/sn_dysfunction-0.7.1/sn_client-0.68.1/sn_node-0.64.1/sn_api-0.66.1/sn_cli-0.59.1

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 10 commits contributed to the release.
 - 2 days passed between releases.
 - 5 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - sn_interface-0.8.1/sn_dysfunction-0.7.1/sn_client-0.68.1/sn_node-0.64.1/sn_api-0.66.1/sn_cli-0.59.1 ([`2b00cec`](https://github.com/maidsafe/safe_network/commit/2b00cec961561281f6b927e13e501342843f6a0f))
    - Merge #1315 ([`67686f7`](https://github.com/maidsafe/safe_network/commit/67686f73f9e7b18bb6fbf1eadc3fd3a256285396))
    - Merge branch 'main' into feat-dbc-spent-proof-validations ([`f83724c`](https://github.com/maidsafe/safe_network/commit/f83724cff1e63b35f1612fc82dffdefbeaab6cc1))
    - Merge #1313 ([`7fe7be3`](https://github.com/maidsafe/safe_network/commit/7fe7be336799dec811c5b17e6d753ebe31e625f1))
    - Merge branch 'main' into cargo-husky-tweaks ([`6881855`](https://github.com/maidsafe/safe_network/commit/688185573bb71cc44a7103df17f3fbeea6740247))
    - perform signature verifications on input DBC SpentProof before signing new spent proof share ([`8313ed8`](https://github.com/maidsafe/safe_network/commit/8313ed8d5b45b7f4ed3b36ada231e74c49c9f9e6))
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
<csr-id-e4e2eb56611a328806c59ed8bc80ca2567206bbb/>

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

