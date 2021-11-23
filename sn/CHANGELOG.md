# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.42.0 (2021-11-23)

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

### Bug Fixes

 - <csr-id-9a82649f0ca01c6d2eae57f260d2f98246724556/> multiples fixes for unit tests
   - error instead of panicking if logger is already initialized
   - use unique socker addrs for nodes
   - print error returned from proptest
 - <csr-id-d3b88f749ca6ee53f65e200105aeeea691581e83/> send DkgRetry for DkgError::MissingPart as well
 - <csr-id-35a8fdb9dca60ca268d536958a9d32ce0a876792/> ensure client chunk count is 3 w/ 7 elders
 - <csr-id-c27d7e997c5a7812f995f113f31edf30c2c21272/> fix a compilation error introduced by merge

 - <csr-id-55eb0f259a83faff470dbfdeb9365d314ed6a697/> joining node age to genesis section shall be less than or equal to suggested age
 - <csr-id-42d90b763606e2d324c5ce1235fc801105c07acb/> use correct dst section key for AE-Update to siblings after split
 - <csr-id-302ce4e605d72a0925509bfe3220c2b1ddac677d/> raise SectionSplit event whenever prefix changed
 - <csr-id-c78513903457a701096b5c542f15012e71d33c46/> during bootstrap, handling the case prefix_map was loaded

### New Features (BREAKING)

 - <csr-id-3a59ee3b532bbc26388780ddc2f5b51ddae61d4c/> include section chain in AE-Redirect messages


### Commit Statistics

<csr-read-only-do-not-edit/>

 - 68 commits contributed to the release over the course of 8 calendar days.
 - 65 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network-0.41.0 ([`14fdaa6`](https://github.com/maidsafe/safe_network/commit/14fdaa6537619483e94424ead5751d5ab41c8a01))
    - reuse connections during join flow ([`950a4ee`](https://github.com/maidsafe/safe_network/commit/950a4eece1a11f70b2aef71cc11404603bb2ec5c))
    - add 'connectedness' to `Peer`'s `Display` ([`7a875a8`](https://github.com/maidsafe/safe_network/commit/7a875a88c911b5a9db59a55b81c94b995e2b95ae))
    - inherit span on `Comm::send` incoming message listener ([`5514d9c`](https://github.com/maidsafe/safe_network/commit/5514d9c06ee400e983a258ba2eb37bff1abf0dd0))
    - add a feature for increased `WireMsg` debug info ([`1848189`](https://github.com/maidsafe/safe_network/commit/1848189aa780f2f6aabddb3564b816c72f9bee6e))
    - replace `Sender` with `UnknownPeer` ([`f73364e`](https://github.com/maidsafe/safe_network/commit/f73364efa66718b92e04f24d4546c1e248198ce8))
    - Remove always sending a message to ourself when we can handle it directly ([`dbdda13`](https://github.com/maidsafe/safe_network/commit/dbdda1392325e8a08a6a28988208769552d5e543))
    - multiples fixes for unit tests ([`9a82649`](https://github.com/maidsafe/safe_network/commit/9a82649f0ca01c6d2eae57f260d2f98246724556))
    - send DkgRetry for DkgError::MissingPart as well ([`d3b88f7`](https://github.com/maidsafe/safe_network/commit/d3b88f749ca6ee53f65e200105aeeea691581e83))
    - use AE to progress various intra-DKG phases ([`f29c94d`](https://github.com/maidsafe/safe_network/commit/f29c94d7dfab2e82b9db70fcfeddc4a71d987abb))
    - ensure client chunk count is 3 w/ 7 elders ([`35a8fdb`](https://github.com/maidsafe/safe_network/commit/35a8fdb9dca60ca268d536958a9d32ce0a876792))
    - allow ELDER_COUNT to be overridden by env var SN_ELDER_COUNT ([`46ea542`](https://github.com/maidsafe/safe_network/commit/46ea542731fa3e2cede4ce9357783d3681434643))
    - move ELDER_COUNT to root of sn ([`e8e44fb`](https://github.com/maidsafe/safe_network/commit/e8e44fb69dae7fe1cae3bbf39f7b03a90df44dd7))
    - depend on ELDER_SIZE where it should be used ([`86c83be`](https://github.com/maidsafe/safe_network/commit/86c83bedf061b285b0d4f3effb54cc89a9fbd84d))
    - set node count to be 33 ([`14328b3`](https://github.com/maidsafe/safe_network/commit/14328b3d5b2579e4f038624c2353ec36c45fa9ed))
    - increase join timeout when always-joinable set ([`590bfea`](https://github.com/maidsafe/safe_network/commit/590bfea538bb78fd20bb9574c3ea9ed6dcceced0))
    - increase join timeout before waiting to retry at bin/node ([`85f2b5f`](https://github.com/maidsafe/safe_network/commit/85f2b5fd2adbf1f57e184c9e687fd14929b911e7))
    - fix a compilation error introduced by merge ([`c27d7e9`](https://github.com/maidsafe/safe_network/commit/c27d7e997c5a7812f995f113f31edf30c2c21272))
    - store `Peer`s in `SectionAuthorityProvider` ([`b1297f2`](https://github.com/maidsafe/safe_network/commit/b1297f2e42cdd7f7945c9fd5b4012086f8298b85))
    - reuse known `Peer`s in DKG sessions ([`680f943`](https://github.com/maidsafe/safe_network/commit/680f9431bc75efea455dd7c6985e41c11740dc3e))
    - clarify `connection` in `Peer`'s `Debug` output ([`eb02910`](https://github.com/maidsafe/safe_network/commit/eb0291082e49d1114c853b214f55b8a25e18d1e1))
    - store a `Peer` in `NodeState` ([`93bffbf`](https://github.com/maidsafe/safe_network/commit/93bffbf1343621a8d5187649d1bb6d2a81cba793))
    - set `Peer::connection` on send ([`1268fb6`](https://github.com/maidsafe/safe_network/commit/1268fb6e46e4ba061ba7f724917371d8e0db0092))
    - bye bye so much olde node code ([`14c6ab9`](https://github.com/maidsafe/safe_network/commit/14c6ab95da62a144a798665ade6da626fb5ee6f2))
    - readd routing member joined event for tests ([`30d220c`](https://github.com/maidsafe/safe_network/commit/30d220c180c7586de56282fafe52b54c8b619860))
    - log node age on relocation in routing ([`6c02091`](https://github.com/maidsafe/safe_network/commit/6c0209187797bb27fa1529dd2f0ee8116975a02d))
    - set joins allowed in routing code ([`a328d70`](https://github.com/maidsafe/safe_network/commit/a328d70720a795b62ac05a68454abf1034e97b9d))
    - add StillElderAfterSplit LogMarker ([`da1fe8d`](https://github.com/maidsafe/safe_network/commit/da1fe8dda6325e05b4d976d2b77d084c06d98cb0))
    - remove AdultsChanged event and ahndle everything inside routing ([`078f451`](https://github.com/maidsafe/safe_network/commit/078f45148af2da8d1690490720ef5f95555d42cd))
    - update client listener error handling ([`4c9b20e`](https://github.com/maidsafe/safe_network/commit/4c9b20ee58be9ca3579c7d1f4edd09452d59d38c))
    - cleanup test clippy ([`49d3a29`](https://github.com/maidsafe/safe_network/commit/49d3a298be19a46dcb6232ee8087c9bf0381db23))
    - feat(sn/client): retry client queires if elder conns fail ([`20bda03`](https://github.com/maidsafe/safe_network/commit/20bda03e07985114b8a54c866755be8a240c2504))
    - chore(sn/client): dont always exit on client listener loop errors ([`cdc31cb`](https://github.com/maidsafe/safe_network/commit/cdc31cb44af8b6dc2393a0f15c98c0be364db6ff))
    - fail query if we cannot send to more than 1 elder ([`bc71707`](https://github.com/maidsafe/safe_network/commit/bc717073fde5fef59cbbe670ab6b7a9d66253d3b))
    - burn down more of the /node dir ([`a98cc81`](https://github.com/maidsafe/safe_network/commit/a98cc810409a4ccc78e3c7a36b38b5f00cfcb23a))
    - joining node age to genesis section shall be less than or equal to suggested age ([`55eb0f2`](https://github.com/maidsafe/safe_network/commit/55eb0f259a83faff470dbfdeb9365d314ed6a697))
    - fix routing tests failing after we no longer emit MemerLeft event ([`189aeb6`](https://github.com/maidsafe/safe_network/commit/189aeb6941d277f94e47ade0e851696206aaa146))
    - chore(sn/routing): simplify msg handling for NodeCmds ([`9d5baf8`](https://github.com/maidsafe/safe_network/commit/9d5baf83c630e597407e5c5d4a47fc7f34805696))
    - more node plumbing cleanup ([`9aaccb8`](https://github.com/maidsafe/safe_network/commit/9aaccb8784afd03454cd69f599dc2b7822ca130c))
    - handle set storage level in routing directly ([`8f1cc32`](https://github.com/maidsafe/safe_network/commit/8f1cc32b87a9e333f32399020339dd41e2d0b049))
    - handle member left directly in routing code ([`1fbc9b2`](https://github.com/maidsafe/safe_network/commit/1fbc9b24d14460c6a88a064297c3a327ca9182aa))
    - handle receiving dataExchange packet in routing. ([`4e7da0a`](https://github.com/maidsafe/safe_network/commit/4e7da0aeddba7287c1556ca318d9cdd8459000d2))
    - remove node plumbing ([`456c022`](https://github.com/maidsafe/safe_network/commit/456c022ddf66af5f991e6aca2240d2ab73f8241d))
    - perform data exchange on split in routing ([`ccb7efd`](https://github.com/maidsafe/safe_network/commit/ccb7efda1ee0c6303ac537ca9edd4b6c5cfcc5f2))
    - correlate client connections without `ConnectedPeers` ([`b70a105`](https://github.com/maidsafe/safe_network/commit/b70a1053adc267780215636dd80a759c45d533d5))
    - use `qp2p::Connection` for connected senders ([`1d62fce`](https://github.com/maidsafe/safe_network/commit/1d62fcefd5d44ef0df84b8126ba88de1127bd2bd))
    - represent `HandleMessage::sender` as an enum ([`20c1d7b`](https://github.com/maidsafe/safe_network/commit/20c1d7bf22907a44be2cc9585dd2ac55dd2985bf))
    - add an `Option<qp2p::Connection>` to `Peer` ([`182c4db`](https://github.com/maidsafe/safe_network/commit/182c4db4531c77fa6c67b8267cd7895b49f26ae1))
    - add a log marker for reusing a connection ([`02f1325`](https://github.com/maidsafe/safe_network/commit/02f1325c18779ff02e6bb7d903fda4519ca87231))
    - add a feature to disable connection pooling ([`1bf1766`](https://github.com/maidsafe/safe_network/commit/1bf17662ecae4e1f3e9969ac03a8f543f57f2cd0))
    - stop sending unnecessary AE-Update msgs to to-be-promoted candidates ([`4a1349c`](https://github.com/maidsafe/safe_network/commit/4a1349c5e99fdc45c038afdd2dac24486ee625ea))
    - on split retain adult members only ([`d55978b`](https://github.com/maidsafe/safe_network/commit/d55978b38c9f69c0e31b335909ca650758c522c1))
    - fix demotion sendmessage test ([`9a14911`](https://github.com/maidsafe/safe_network/commit/9a149113d6ffa4d4e5564ad8ba274117a4522685))
    - only fire SplitSuccess if elder ([`f8de28a`](https://github.com/maidsafe/safe_network/commit/f8de28a8e040439fcf83f217261facb934cacd51))
    - send data updates to new elders only ([`231bde0`](https://github.com/maidsafe/safe_network/commit/231bde07064d6597867b5668a1e5ff319de3a202))
    - dont AE-update sibling elders on split. ([`3b7091b`](https://github.com/maidsafe/safe_network/commit/3b7091bd8b45fc5bbfa6f1714ec56e123d76c647))
    - remove node plumbing ([`3b4e2e3`](https://github.com/maidsafe/safe_network/commit/3b4e2e33285f4b1a53bd52b147dc85c570ed2f6e))
    - perform data exchange on split in routing ([`345033a`](https://github.com/maidsafe/safe_network/commit/345033abe74a86c212e3b38fc0f6b655216b63b0))
    - verify and use SAP received in AE-Redirect to update client's network knowledge ([`f6a3f78`](https://github.com/maidsafe/safe_network/commit/f6a3f78156b9cc0f934c19e5d4c0004238a593e4))
    - verify and use SAP received in AE-Redirect to update our network knowledge ([`ae5fc40`](https://github.com/maidsafe/safe_network/commit/ae5fc40e29161652306b2e42b92d2a80fc746708))
    - log an error when failing to add latest sibling key to proof chain ([`d673283`](https://github.com/maidsafe/safe_network/commit/d6732833db0bb13d8414f99758e8767ab8ab2bdd))
    - increase timeout for network forming ([`4f5e84d`](https://github.com/maidsafe/safe_network/commit/4f5e84da96d7277dfc4e385ff03edf6c1d84991e))
    - update launch tool and secured linked list deps ([`d75e5c6`](https://github.com/maidsafe/safe_network/commit/d75e5c65d04c60dca477cca4e704308198cfd6cf))
    - use correct dst section key for AE-Update to siblings after split ([`42d90b7`](https://github.com/maidsafe/safe_network/commit/42d90b763606e2d324c5ce1235fc801105c07acb))
    - include section chain in AE-Redirect messages ([`3a59ee3`](https://github.com/maidsafe/safe_network/commit/3a59ee3b532bbc26388780ddc2f5b51ddae61d4c))
    - use a DAG to keep track of all sections chains within our NetworkKnowledge ([`90f2979`](https://github.com/maidsafe/safe_network/commit/90f2979a5bcb7f8e1786b5bdd868793d2fe924b4))
    - raise SectionSplit event whenever prefix changed ([`302ce4e`](https://github.com/maidsafe/safe_network/commit/302ce4e605d72a0925509bfe3220c2b1ddac677d))
    - during bootstrap, handling the case prefix_map was loaded ([`c785139`](https://github.com/maidsafe/safe_network/commit/c78513903457a701096b5c542f15012e71d33c46))
</details>

## v0.41.0 (2021-11-23)

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
- use unique socker addrs for nodes
- print error returned from proptest

### New Features (BREAKING)

 - <csr-id-3a59ee3b532bbc26388780ddc2f5b51ddae61d4c/> include section chain in AE-Redirect messages

<csr-unknown>
 send DkgRetry for DkgError::MissingPart as well ensure client chunk count is 3 w/ 7 elders fix a compilation error introduced by merge joining node age to genesis section shall be less than or equal to suggested age use correct dst section key for AE-Update to siblings after split raise SectionSplit event whenever prefix changed during bootstrap, handling the case prefix_map was loaded<csr-unknown/>

## v0.40.0 (2021-11-15)

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

### refactor (BREAKING)

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 4 commits contributed to the release over the course of 3 calendar days.
 - 4 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network v0.40.0/sn_api v0.39.0 ([`7001573`](https://github.com/maidsafe/safe_network/commit/70015730c3e08881f803e9ce59be7ca16185ae11))
    - retry on connection loss of reused connection ([`1389ffa`](https://github.com/maidsafe/safe_network/commit/1389ffa00762e126047a206abc475599c277930c))
    - avoid holding write lock on ae_backoff_cache during sleep ([`ad633e1`](https://github.com/maidsafe/safe_network/commit/ad633e1b6882db1aac0cb1a530300d9e4d666fd8))
    - remove unnecessary peer lagging check ([`5a3b70e`](https://github.com/maidsafe/safe_network/commit/5a3b70e9721fcdfdd809d2a6bd85968446b4e9a3))
</details>

<csr-unknown>
Expected age of joining node for genesis section is now calculatedin a deterministic way using the peer’s address.<csr-unknown/>

## v0.39.0 (2021-11-12)

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

### refactor (BREAKING)

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 18 commits contributed to the release over the course of 1 calendar day.
 - 18 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network v0.39.0 ([`ab00cf0`](https://github.com/maidsafe/safe_network/commit/ab00cf08d217654c57449437348b73576a65e89f))
    - make `NodeState` not `Copy` ([`0a5027c`](https://github.com/maidsafe/safe_network/commit/0a5027cd9b2c62833ccf70e2bcca5ab22625a840))
    - duplicate `SectionAuthorityProvider` in `routing` ([`ee165d4`](https://github.com/maidsafe/safe_network/commit/ee165d41ca40be378423394b6422570d1d47727c))
    - duplicate `NodeState` in `routing` ([`645047b`](https://github.com/maidsafe/safe_network/commit/645047b231eaf69f1299dee22ff2079feb7f5a95))
    - move `SectionPeers` from `messaging` to `routing` ([`00acf0c`](https://github.com/maidsafe/safe_network/commit/00acf0c0d8a65bdd5355ba909d73e74729a27044))
    - update testnet default interval ([`08629e5`](https://github.com/maidsafe/safe_network/commit/08629e583a03852d9f02b0e7ff66a829e3056d9b))
    - rename forced to skip section info agreement, tidy assignment ([`5a81d87`](https://github.com/maidsafe/safe_network/commit/5a81d874cf8040ff81a602bc8c5707a4ba6c64ff))
    - update bls_dkg to 0.7.1 ([`8aa27b0`](https://github.com/maidsafe/safe_network/commit/8aa27b01f8043e98953971d1623aaf8b07d1596e))
    - draining backlog before process DKG message ([`366ad9a`](https://github.com/maidsafe/safe_network/commit/366ad9a42034fe450c30908d372fede0ff92f655))
    - less DKG progression interval; switch SAP on DKGcompletion when OurElder already received ([`c4a42fb`](https://github.com/maidsafe/safe_network/commit/c4a42fb5965c79aaae2a0ef2ae62fcb987c37525))
    - stepped age with long interval ([`48a7d00`](https://github.com/maidsafe/safe_network/commit/48a7d0062f259b362eb745e1bd59e77df514f1b8))
    - move `ElderCandidates` to `routing` ([`7d5d5e1`](https://github.com/maidsafe/safe_network/commit/7d5d5e11fef39a6dc1b89c972e42772db807374c))
    - make `SectionAuthorityProviderUtils` `pub(crate)` ([`855b8ff`](https://github.com/maidsafe/safe_network/commit/855b8ff87217e92a5f7d55fb78ab73c9d81f75a2))
    - move `routing::Peer` into `network_knowledge` ([`0e2b01b`](https://github.com/maidsafe/safe_network/commit/0e2b01bb7e9c32f0bc5c1c6fd616acfcc5c45020))
    - remove `Copy` from `Peer` ([`23b8a08`](https://github.com/maidsafe/safe_network/commit/23b8a08e97fa415b9216caac5da18cb97ede980f))
    - add more tracing around connections ([`008f363`](https://github.com/maidsafe/safe_network/commit/008f36317f440579fe952325ccb97c9b5a547165))
    - support `--json-logs` in `testnet` ([`b8b0097`](https://github.com/maidsafe/safe_network/commit/b8b0097ac86645f0c7a7352f2bf220279068228c))
    - update bls_dkg and blsttc to 0.7 and 0.3.4 respectively ([`213cb39`](https://github.com/maidsafe/safe_network/commit/213cb39be8fbfdf614f3eb6248b14fe161927a14))
</details>

## v0.38.0 (2021-11-10)

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 6 commits contributed to the release.
 - 6 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - safe_network v0.38.0 ([`cbdf023`](https://github.com/maidsafe/safe_network/commit/cbdf0236f7cd241e7addb1afe75ed5e5cfac00ab))
    - fix `Eq` for `SectionPeers` ([`adf9fea`](https://github.com/maidsafe/safe_network/commit/adf9feac964fb7f690bd43aeef3270c82fde419c))
    - reuse join permits for relocated nodes too. ([`a7968dd`](https://github.com/maidsafe/safe_network/commit/a7968dd17927e346b3d32bb5971ed6457aea6606))
    - insert a connected peer for outgoing connections ([`1568adb`](https://github.com/maidsafe/safe_network/commit/1568adb28a2a6a3cdf8a9737e098a5ea7bb2c419))
    - allow to change the default interval for testnet nodes ([`044fd61`](https://github.com/maidsafe/safe_network/commit/044fd61950be76e3207694094dcec81313937403))
    - provide the SAP proof chain in JoinResponse::Retry msgs ([`9b8ddfd`](https://github.com/maidsafe/safe_network/commit/9b8ddfde0287e47b6f18a77a8e8847d80ee84bcd))
</details>

## v0.37.0 (2021-11-09)

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

### New Features (BREAKING)

 - <csr-id-06b57d587da4882bfce1b0acd09faf9129306ab2/> add log markers for connection open/close
   We can detect connection open/close easily in the connection listener
   tasks, since these are started as soon as a connection is opened, and
   finish when there are no more incoming connections (e.g. connection has
   closed).
 - <csr-id-20895dd4326341de4d44547861ac4a57ae8531cf/> the JoinResponse::Retry message now provides the expected age for the joining node

### refactor (BREAKING)

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

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 140 commits contributed to the release over the course of 28 calendar days.
 - 128 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - (cargo-release) version 0.37.0 ([`ef42af4`](https://github.com/maidsafe/safe_network/commit/ef42af483725286dd8a0cef6f922dfd1739412d8))
    - refactor the Join process regarding used_recipients ([`e406a9a`](https://github.com/maidsafe/safe_network/commit/e406a9a61bb313dcd445d639e82fa8ae1ff99964))
    - if attempting to send a msg fails, continue with the rest of cmd processing instead of bailing out ([`db76c5e`](https://github.com/maidsafe/safe_network/commit/db76c5ee5b2efc35214d12df0a2aa4e137231fa6))
    - use the prefix map updating outcome to decide if current SAP and chain shall be updated ([`7d5212a`](https://github.com/maidsafe/safe_network/commit/7d5212a2b916a8e540403346e0770cee1a446884))
    - some variables renaming and improving some comments ([`b66444c`](https://github.com/maidsafe/safe_network/commit/b66444c7d7a10c9a26db30fe5ce8e91984031624))
    - drop JoinRetry responses for SAPs we've already resent join reqs to ([`109843c`](https://github.com/maidsafe/safe_network/commit/109843c67ffd4ac4675cdfe56d7fcdaf97664007))
    - avoid rebuild on windows for testnet ([`c5aae59`](https://github.com/maidsafe/safe_network/commit/c5aae59a4eac6c3285bc45845433d23fc96d155f))
    - client config test timeout/keepalive needed updating ([`5b596a4`](https://github.com/maidsafe/safe_network/commit/5b596a444d24f7021db9d3c322bafa33d49dcfae))
    - add more log markers. ([`723618d`](https://github.com/maidsafe/safe_network/commit/723618d0e3ad77eb26a3c698f9545490166b7ee0))
    - fix healthcheck 0/1 summation. ([`1b7f5d6`](https://github.com/maidsafe/safe_network/commit/1b7f5d6e198eb286034e12eed3fa4fc63b927e0f))
    - update log messages and test errors for clarity ([`80ce4ca`](https://github.com/maidsafe/safe_network/commit/80ce4ca9376b99037b9377425c477ea3a7493e54))
    - routing test tweaks ([`d44fd8d`](https://github.com/maidsafe/safe_network/commit/d44fd8d5888c554bf1aa0d56e08471cfe90bd988))
    - make health check tolerante of possible demotions ([`a927d9e`](https://github.com/maidsafe/safe_network/commit/a927d9e27d831a48348eab6d41e0f4231b0e62c7))
    - tesnet grep is_log_file check ([`42a3123`](https://github.com/maidsafe/safe_network/commit/42a3123b0aa4f01daedc2faebd3fa3430dbc1618))
    - node tests after removing blocking cmd layer ([`9214dff`](https://github.com/maidsafe/safe_network/commit/9214dff1541b28577c44d8cbbebeec0b80598653))
    - remove blocking/non blocking msg handling distinction ([`616c024`](https://github.com/maidsafe/safe_network/commit/616c024f2280fc99de063026756a9c938f78b885))
    - fix windows grep lock error: dont search non log files ([`bc47eed`](https://github.com/maidsafe/safe_network/commit/bc47eed7e327a8e02fff1d385648c752ad33b8f1))
    - clarify elder counts in health check test ([`cf5a041`](https://github.com/maidsafe/safe_network/commit/cf5a041742844b8b3d5e71c3e895018367b76013))
    - reduce join backoff times ([`ed6d2f0`](https://github.com/maidsafe/safe_network/commit/ed6d2f09c12b72446f0f55ecacb5fc4f35278575))
    - change client timeouts ([`9350f27`](https://github.com/maidsafe/safe_network/commit/9350f273a5afec88120fe79fe85ceaf8027d691e))
    - readd keep alive for client ([`27e8d89`](https://github.com/maidsafe/safe_network/commit/27e8d895983d236a0949354219c4adbe3f1e22a0))
    - reorder blob tst log initialisation to _after_ initial AE triggering msgs sent ([`a54c442`](https://github.com/maidsafe/safe_network/commit/a54c4426caccf71d28b2697094135892ec4a5e16))
    - update health check to test for Prefix(1/0), use this in CI ([`8eedb66`](https://github.com/maidsafe/safe_network/commit/8eedb660461b13b236545643fa21868eb6613826))
    - add env var for client query timeout ([`abfe737`](https://github.com/maidsafe/safe_network/commit/abfe7378604a74119accd7b9f86bef5682b0784a))
    - update logs during AEUpdate ([`e45dd10`](https://github.com/maidsafe/safe_network/commit/e45dd10ac770ed41431f0e8758a6228fc4dfbe3c))
    - reduce clone and lock during message handling ([`0399a9e`](https://github.com/maidsafe/safe_network/commit/0399a9e76547a03f1e3902aec30ecbd57ed437c7))
    - thread a `Peer` through more places ([`6bb5e64`](https://github.com/maidsafe/safe_network/commit/6bb5e64b3b0eb518719581278551272ae8f2b2ed))
    - simplify some iterator chains ([`e47e9ec`](https://github.com/maidsafe/safe_network/commit/e47e9ecb0c7f23209a9c1eb58c248fdf2facfd4a))
    - use `peers()` instead of `Peer::new` when available ([`3b728b6`](https://github.com/maidsafe/safe_network/commit/3b728b65a07a33019e06dd6f3da9fd334e6da9e1))
    - remove `SectionAuthorityProviderUtils::elders` ([`61dec0f`](https://github.com/maidsafe/safe_network/commit/61dec0fd90b4df6b0695a7ba46da86999d199d4a))
    - replace `(XorName, SocketAddr)` with `Peer` ([`87c62d3`](https://github.com/maidsafe/safe_network/commit/87c62d39d240afc01118135bc18d22fe23fc421c))
    - Fix crates.io badge ([`e764941`](https://github.com/maidsafe/safe_network/commit/e76494135d3c8f50b60d146d5c4dadc55248ba39))
    - tweak `Peer` API ([`49c76d1`](https://github.com/maidsafe/safe_network/commit/49c76d13a91474038bd8cb005959a37a7d4c6603))
    - remove `PeerUtils` trait ([`c5d2e31`](https://github.com/maidsafe/safe_network/commit/c5d2e31a5f0cea381bb60dc1f896dbbda5038506))
    - move `Peer` from `messaging` to `routing` ([`0cb790f`](https://github.com/maidsafe/safe_network/commit/0cb790f5c3712e357f685bfb88cd237c5b5f76c5))
    - remove `Peer` from `NodeState` ([`b3ce840`](https://github.com/maidsafe/safe_network/commit/b3ce84012e6cdf4c87d6d4a3137ab6506264e949))
    - remove `reachable` from `messaging::system::Peer` ([`ecf0bc9`](https://github.com/maidsafe/safe_network/commit/ecf0bc9a9736167edb15db7ff4e3cf5dc388dd22))
    - implement `Deref` for `SectionAuth` ([`23f53a3`](https://github.com/maidsafe/safe_network/commit/23f53a339a5fe0d2f2e5d415bfc01e646a81a5c8))
    - add network health check script to easily wait until we're ready+healthy ([`a3552ae`](https://github.com/maidsafe/safe_network/commit/a3552ae2dd0f727a71505d832c1ed2520283e8c8))
    - bring back elders len check during chain updates ([`a0806aa`](https://github.com/maidsafe/safe_network/commit/a0806aa624384ccb437bc8b4b2d108523ea5c068))
    - adapt routing unit tests to new network knowledge handling logic ([`9581fcf`](https://github.com/maidsafe/safe_network/commit/9581fcf27765752f6b556d632e8396e93d0f15e0))
    - renaming section mod to network_knowledge ([`5232bf5`](https://github.com/maidsafe/safe_network/commit/5232bf51c738197339ac70ee4f46adee4fa87179))
    - storing all sections chains in routing network knowledge ([`d7972da`](https://github.com/maidsafe/safe_network/commit/d7972da6abd3001e75019bf72aae6a98919ed1db))
    - refactoring NetworkKnowledge private API ([`49e2d14`](https://github.com/maidsafe/safe_network/commit/49e2d14a1c5f8fd83aa4b9a5abe67e23fca9f966))
    - moving to a unified network knowledge for updating SAPs and section chain ([`5326de5`](https://github.com/maidsafe/safe_network/commit/5326de5f658dfe75d1f5c44224d8623123848b08))
    - Section->NetworkKnowledge name change ([`f9cd39b`](https://github.com/maidsafe/safe_network/commit/f9cd39b6d3b1877fa0f3fa73bd6e3a796cae3b08))
    - match on name ([`57b736a`](https://github.com/maidsafe/safe_network/commit/57b736a8901ed76c402460fcf1799162cfdb3c37))
    - reduce the report check interval ([`208abdd`](https://github.com/maidsafe/safe_network/commit/208abdd108654ab83e8bee763793201e6f7b5eb2))
    - bump rust edition ([`fc10d03`](https://github.com/maidsafe/safe_network/commit/fc10d037d64efc86796f1b1c6f255a4c7f91d3e1))
    - make clients read prefix_map from disk ([`9a3cffc`](https://github.com/maidsafe/safe_network/commit/9a3cffc52589e8adf6dac75ae6aab4c184118648))
    - cache prefix_map for clients ([`ba5f284`](https://github.com/maidsafe/safe_network/commit/ba5f28475048bfaebcc37c660bec65644e4e52fe))
    - add `KvStore::flush` to avoid waiting in tests ([`c2df9db`](https://github.com/maidsafe/safe_network/commit/c2df9db2fe9e99450cc16060ff034289ab683783))
    - remove unnecessary `allow(unused)` ([`12c673e`](https://github.com/maidsafe/safe_network/commit/12c673e7f52ed1c4cbe15c14fe6eb4e68b986e18))
    - remove unused `KvStore::store_batch` ([`9d27749`](https://github.com/maidsafe/safe_network/commit/9d27749bc7d59ee499980044f57eab86d2e63d04))
    - make `KvStore::store` perform atomic immutable writes ([`6eb389a`](https://github.com/maidsafe/safe_network/commit/6eb389acfb3adcec0be07ee990106ed19a7f78f5))
    - remove redundant `K` type parameter from `KvStore` ([`2fc2f0e`](https://github.com/maidsafe/safe_network/commit/2fc2f0e6fbb5f7b05e61281e90992053ef5f0f5d))
    - update elder count check to be general one split health check ([`a03d2ce`](https://github.com/maidsafe/safe_network/commit/a03d2cef55665759ddfeb40972676c87b17ccfa8))
    - fix test for `GetChunk` 'data not found' failures ([`b53a86b`](https://github.com/maidsafe/safe_network/commit/b53a86b9ba0bfb33ab9cccaecb383bcce0acf233))
    - fix!(sn/node): disable keep-alive by default ([`eb90bb9`](https://github.com/maidsafe/safe_network/commit/eb90bb90db977ae6e368047e8a61efd6caba25bd))
    - add log markers for connection open/close ([`06b57d5`](https://github.com/maidsafe/safe_network/commit/06b57d587da4882bfce1b0acd09faf9129306ab2))
    - update `qp2p`, which removes `ConnectionPool` ([`432e36d`](https://github.com/maidsafe/safe_network/commit/432e36de012695c6c5e20bd704dc184db9c5c4d6))
    - typo ([`5f98534`](https://github.com/maidsafe/safe_network/commit/5f985342239b74963f89e627e28031f8817c0d3d))
    - spot and blob management inconsistency add some comments ([`a191009`](https://github.com/maidsafe/safe_network/commit/a1910095d4b7a749186d614e04dad2b64a1eabff))
    - unignore routing demote test ([`aaa0af4`](https://github.com/maidsafe/safe_network/commit/aaa0af4d1685c65a3c166070a590c10e9fd54765))
    - replace prefixmap check to ensure last proof chainkey is present on provided sap ([`ab98ec2`](https://github.com/maidsafe/safe_network/commit/ab98ec2076c1f6c60899c505ed85aaefa5359278))
    - Revert "feat(messages): add more prioritiy leves for different types of messages" ([`a0fd091`](https://github.com/maidsafe/safe_network/commit/a0fd09155c885dbfd6858a68805a8d4391284eb0))
    - Revert "chore: use constants for message priority" ([`6d1cdc6`](https://github.com/maidsafe/safe_network/commit/6d1cdc64078de06a43281d924f58d01b615e9268))
    - ignore demotion test for now ([`bde3cd5`](https://github.com/maidsafe/safe_network/commit/bde3cd5eac75cc41a3a9ffefb091584273575f68))
    - elders update uses correct key for prefixmap updates ([`96519e2`](https://github.com/maidsafe/safe_network/commit/96519e26ef9adf6360b6a8f79ca73ab4bc7b627b))
    - tweak demotion test ([`73b405d`](https://github.com/maidsafe/safe_network/commit/73b405d1228244a2b984e1294d5e8542f8691cef))
    - use prefixmap update as opposed to verify against redundant same chain ([`0c9c1f2`](https://github.com/maidsafe/safe_network/commit/0c9c1f2edd9e872b9ba1642ac50f59a63f68b488))
    - trust any valid key for chain updates ([`9bdd68f`](https://github.com/maidsafe/safe_network/commit/9bdd68f313b3b7881cb39db02026184bbab0bfb0))
    - move known prefix log to verify and udpate ([`8b7b664`](https://github.com/maidsafe/safe_network/commit/8b7b66450349b669e023539e92c77f9a3b830948))
    - updated routing untrusted ae test for new error ([`3569939`](https://github.com/maidsafe/safe_network/commit/3569939ff83978ba50039588eb87d4e6da4fedd2))
    - add check to received sap on ae update ([`7fc29f8`](https://github.com/maidsafe/safe_network/commit/7fc29f8187a29c2eeff8bbb5e09f068414bb8b93))
    - provide valid proof chain to update prefix map during elders agreement handling ([`f615aac`](https://github.com/maidsafe/safe_network/commit/f615aacdb4cd972545ba88c927cfb5a9b357fb9a))
    - keep section in sync on ae retry recevial at nodes ([`51ea358`](https://github.com/maidsafe/safe_network/commit/51ea358eff0edee0b27c5e21af4f38a7ee09422c))
    - ae blob tests dont have standard wait ([`0df6615`](https://github.com/maidsafe/safe_network/commit/0df6615df2fdb88d7fd89f9751c6b88e1b7ebb5f))
    - use constants for message priority ([`4415c9b`](https://github.com/maidsafe/safe_network/commit/4415c9b1d166f7e53032a0100d829e8581255a1e))
    - use client query in routing tests instead of bytes ([`b031962`](https://github.com/maidsafe/safe_network/commit/b0319628d587a2caeb1aa5be52505cbb6ede40d3))
    - add more prioritiy leves for different types of messages ([`1e92fa5`](https://github.com/maidsafe/safe_network/commit/1e92fa5a2ae4931f6265d82af121125495f58655))
    - remove unused data exchange structs ([`8eb1877`](https://github.com/maidsafe/safe_network/commit/8eb1877effb8cf0bfc8986c23d49d727500087dd))
    - the JoinResponse::Retry message now provides the expected age for the joining node ([`20895dd`](https://github.com/maidsafe/safe_network/commit/20895dd4326341de4d44547861ac4a57ae8531cf))
    - don't bail out when handling ElderAgreement if prefix-map is not updated ([`20a69cc`](https://github.com/maidsafe/safe_network/commit/20a69ccb6c6b1f466b519318dd85ef11e7700027))
    - stepped fixed age during first section ([`56f3b51`](https://github.com/maidsafe/safe_network/commit/56f3b514fceccbc1cc47256410b4f2119bb8affd))
    - enough having enough elders before split attempt ([`0ea37af`](https://github.com/maidsafe/safe_network/commit/0ea37af54f761b2a1a5137463637736c6be25206))
    - minor refactor to prefix_map reading ([`0b84139`](https://github.com/maidsafe/safe_network/commit/0b8413998f018d7d577f2248e36e21f6c2744116))
    - read prefix_map from disk if available ([`ddad0b8`](https://github.com/maidsafe/safe_network/commit/ddad0b8ce37d3537a9e9ed66da18758b6b3ace68))
    - try to update Section before updating Node info when relocating ([`aaa6903`](https://github.com/maidsafe/safe_network/commit/aaa6903e612178ce59481b2e81fe3bd0d1cc2617))
    - moving Proposal utilities into routing:Core ([`9cc1629`](https://github.com/maidsafe/safe_network/commit/9cc16296db7241819e17dd2673c7b3cb9fe2ead8))
    - simplifying key shares cache ([`c49f9a1`](https://github.com/maidsafe/safe_network/commit/c49f9a16c9e0912bf581a2afef22ac4806898ade))
    - allow to set the genesis key for Core and Section structs of first node ([`0d3fe1a`](https://github.com/maidsafe/safe_network/commit/0d3fe1a6e02f6d83115e9098c706a98cb688d41d))
    - prevent deadlock by dropping write locks in relocation ([`c360dc6`](https://github.com/maidsafe/safe_network/commit/c360dc61bd7d4ce1745ba7bfbe9d032fedac067a))
    - increase node resource proof difficulty ([`2651423`](https://github.com/maidsafe/safe_network/commit/2651423c61b160841557d279c9b706abdaab4cdf))
    - tweak join backoff ([`20461a8`](https://github.com/maidsafe/safe_network/commit/20461a84dbc0fc373b184a3982a79affad0544f6))
    - remove core RwLock ([`6a8f5a1`](https://github.com/maidsafe/safe_network/commit/6a8f5a1f41e5a0f4c0cce6914d4b330b68f5e5d8))
    - fix(sn/testnet): build `sn_node` from inside `sn` directory ([`fd4513f`](https://github.com/maidsafe/safe_network/commit/fd4513f054e282218797208dcac1de6903e94f2c))
    - fix(sn/testnet): exit with error if `sn_node` build fails ([`7ff7557`](https://github.com/maidsafe/safe_network/commit/7ff7557850460d98b526646b21da635381a70e2a))
    - upgrade `sn_launch_tool` ([`da70738`](https://github.com/maidsafe/safe_network/commit/da70738ff0e24827b749a970f466f3983b70442c))
    - upgrade `tracing-appender` and `tracing-subscriber` ([`0387123`](https://github.com/maidsafe/safe_network/commit/0387123114ff6ae42920577706497319c8a888cb))
    - encapsulate Section info to reduce SAP and chain cloning ([`9cd2d37`](https://github.com/maidsafe/safe_network/commit/9cd2d37dafb95a2765d5c7801a7bb0c58286c47c))
    - don't backoff when sending join resource challenge responses ([`7b430a5`](https://github.com/maidsafe/safe_network/commit/7b430a54f50846a8475cec804bc24552043558b7))
    - use tokio::semaphore for limiting concurrent joins ([`cfaed1e`](https://github.com/maidsafe/safe_network/commit/cfaed1ece5120d60e9f352b4e9ef70448e2ed3f2))
    - remove extraneous comment ([`fe62ad9`](https://github.com/maidsafe/safe_network/commit/fe62ad9f590f151dd20a6832dfab81d34fc9c020))
    - fix a possible panic when creating client ([`084926e`](https://github.com/maidsafe/safe_network/commit/084926e045952670295eb666c82f1d77ff88f1be))
    - improve join req handling for elders ([`3a19e42`](https://github.com/maidsafe/safe_network/commit/3a19e42770a12885659d653b98511378dad8015f))
    - add some DKG related markers ([`ca074b9`](https://github.com/maidsafe/safe_network/commit/ca074b92b31add1ad6d0db50f2ba3b3d1ae25d5a))
    - moving Section definition our of messaging onto routing ([`b719b74`](https://github.com/maidsafe/safe_network/commit/b719b74abdd1cd84b3813ec7046f4fdf99cde6a2))
    - maintain lexicographical order ([`2d807cb`](https://github.com/maidsafe/safe_network/commit/2d807cb4b75bfd52d9dde7b91214bd3dc5a6d992))
    - write to correct source ([`c765828`](https://github.com/maidsafe/safe_network/commit/c765828018158bda663f937a73bcc47ab358884c))
    - get more splits (at the cost of more join msgs) ([`ba7c4c6`](https://github.com/maidsafe/safe_network/commit/ba7c4c609150839825a4992c08e3bdd00b698269))
    - tweak join retry backoff timing ([`7ddf2f8`](https://github.com/maidsafe/safe_network/commit/7ddf2f850441e6596133ab7a596eb766380008c3))
    - tweak join retry backoff timing ([`aa17309`](https://github.com/maidsafe/safe_network/commit/aa17309092211e8d1ba36d4aaf50c4677a461594))
    - dont wait before first join request ([`104ed36`](https://github.com/maidsafe/safe_network/commit/104ed366dbaafd98fd3ef67899e27540894a959c))
    - fix elders agreement routing tests using wrong command type ([`0033fc6`](https://github.com/maidsafe/safe_network/commit/0033fc65db266d92623f53628f9f5b6e6069920d))
    - add network elder count test, using testnet grep ([`e24763e`](https://github.com/maidsafe/safe_network/commit/e24763e604e6a25ee1b211af6cf3cdd388ec0978))
    - cleanup comments ([`9e3cb4e`](https://github.com/maidsafe/safe_network/commit/9e3cb4e7e4dc3d1b8fa23455151de60d5ea03d4d))
    - some clippy cleanup ([`37aec09`](https://github.com/maidsafe/safe_network/commit/37aec09a555d428c730cdd982d06cf5cb58b60b1))
    - remove unused debug ([`810fe07`](https://github.com/maidsafe/safe_network/commit/810fe0797a4f23a6bb2586d901ccac9272a9beb2))
    - add AE backoff before resending messages to a node ([`6023965`](https://github.com/maidsafe/safe_network/commit/60239655dba08940bd293b3c9243ac732923acfe))
    - missing post rebase async_trait dep ([`ec3f16a`](https://github.com/maidsafe/safe_network/commit/ec3f16a8c33e8afbdfffb392466ba422216d3f68))
    - backoff during join request looping ([`4cafe78`](https://github.com/maidsafe/safe_network/commit/4cafe78e3fdb60d144e8cf810788116ce01de025))
    - make subcommand optional, to get stats quickly ([`d1ecf96`](https://github.com/maidsafe/safe_network/commit/d1ecf96a6d965928d434810ccc9c89d1bc7fac4e))
    - move AE updat resend after retry to only happen for validated saps ([`993d564`](https://github.com/maidsafe/safe_network/commit/993d564f0ad5d8cac8d9a32b8c6c8d1dd00c3fd9))
    - only resend if we've had an update ([`73c5baf`](https://github.com/maidsafe/safe_network/commit/73c5baf94dd37346c6c9987aa51ca26f3a2fea1f))
    - add LogMarker for all different send msg types ([`fd8e8f3`](https://github.com/maidsafe/safe_network/commit/fd8e8f3fb22f8592a45db33e75f237ef22cd1f5b))
    - make elder agreemeent its own blocking command ([`c1c2bca`](https://github.com/maidsafe/safe_network/commit/c1c2bcac021dda4bf18a4d80ad2f86370d56efa7))
    - flesh out far too many 'let _ = ' ([`e2727b9`](https://github.com/maidsafe/safe_network/commit/e2727b91c836619dadf2e464ee7c57b338427f22))
    - enable pleasant span viewing for node logs ([`958e38e`](https://github.com/maidsafe/safe_network/commit/958e38ecd3b4e4dc908913192a1d43b83e082d08))
    - client config test ([`6aa1130`](https://github.com/maidsafe/safe_network/commit/6aa1130a7b380dc3d1ad12f3054cda3a390ff20d))
    - make section peers concurrent by using Arc<DashMap> ([`148140f`](https://github.com/maidsafe/safe_network/commit/148140f1d932e6c4b30122ebcca3450ab6c84544))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_api_into_workspace ([`8ed5aff`](https://github.com/maidsafe/safe_network/commit/8ed5aff8b30ce798f71eac22d66eb3aa9b0bdcdd))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_api_into_workspace ([`3a2817a`](https://github.com/maidsafe/safe_network/commit/3a2817a4c802d74b57d475d88d7bc23223994147))
    - Merge remote-tracking branch 'upstream/main' into merge_sn_api_into_workspace ([`50f48ae`](https://github.com/maidsafe/safe_network/commit/50f48aefcba272345df7d4cd45a59071a5844932))
    - Merge branch 'merge_sn_api_into_workspace' into nrs_resolver_refactor ([`a273d97`](https://github.com/maidsafe/safe_network/commit/a273d9733b8d50b94b0ea3faec1d9e721d86aa27))
    - merge github.com:maidsafe/sn_cli into safe_network ([`414aca2`](https://github.com/maidsafe/safe_network/commit/414aca284b35f1bcb27e5d0cca2bfe451b69e27b))
    - resolution test except for the blob range tests ([`c88a1bc`](https://github.com/maidsafe/safe_network/commit/c88a1bc5fda40093bb129b4351eef73d2eb7c041))
    - update actions workflows for workspace refactor ([`3703819`](https://github.com/maidsafe/safe_network/commit/3703819c7f0da220c8ff21169ca1e8161a20157b))
    - update actions workflows for workspace refactor ([`d0134e8`](https://github.com/maidsafe/safe_network/commit/d0134e870bb097e095e1c8a33e607cf7994e6491))
    - move safe_network code into sn directory ([`2254329`](https://github.com/maidsafe/safe_network/commit/225432908839359800d301d9e5aa8274e4652ee1))
</details>

