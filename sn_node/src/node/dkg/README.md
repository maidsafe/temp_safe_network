# Safe Network DKG with `sn_sdkg`

This post describes how DKG works in the Safe Network. For this implementation, we use the `sn_sdkg` crate which is based on poanetwork's Synchronous Key Generation algorithm in their `hbbft` crate.

## DkgStart

DKG is triggered by the elders when they notice that the oldest members are not the elders or when a section splits. They do so by sending the candidates a `NodeMsg::DkgStart(DkgSessionId, SectionSigShare)` message. The `DkgSessionId` contains all the information that makes this DKG session unique, and the `SectionSigShare` is the signature of an elder. Once we receive supermajority of those, we know we can trust this request and start DKG.

```rust
pub struct DkgSessionId {
    /// Prefix of the session we are elder candidates for
    pub prefix: Prefix,
    /// Other Elders in this dkg session
    pub elders: BTreeMap<XorName, SocketAddr>,
    /// The length of the section chain main branch.
    pub section_chain_len: u64,
    /// The bootstrap members for the next Membership instance.
    pub bootstrap_members: BTreeSet<NodeState>,
    /// The membership generation this SAP was instantiated at
    pub membership_gen: Generation,
}
```

When a node receives a `DkgStart` message from at least supermajority of elders, this DKG session starts.

## Ephemeral bls Keys

The first step in our DKG is generating temporary bls keys, that are used for encryption in our DKG votes. Nodes currently have ed25519 keys on the Safe Network, although those keys are great for signatures, we can't safely do encryption with them. Since our nodes don't have bls keys (elders have a bls keyshare but not a simple bls key), we can just generate one just for this DKG session and discard it after. We need the other nodes to trust this bls key, so we first submit a `DkgEphemeralPubKey` message that contains a signature (with our ed key) over our new bls public key.

```rust
/// Sent when DKG is triggered to other participant
NodeMsg::DkgEphemeralPubKey {
    /// The identifier of the DKG session this message is for.
    session_id: DkgSessionId,
    /// Section authority for the DKG start message
    section_auth: AuthorityProof<SectionAuthProof>,
    /// The ephemeral bls key chosen by candidate
    pub_key: BlsPublicKey,
    /// The ed25519 signature of the candidate
    sig: Signature,
}
```

The message also contains the `section_auth`, so that nodes that didn't receive the `DkgStart` know that they can trust this `DkgSessionId` and use this message as a `DkgStart`.

When nodes receive all the ephemeral keys from the other Dkg participants in the session, they can begin DKG voting.

## Votes

DKG voting uses the `sn_sdkg` crate. Voting has three stages before nodes reach termination:
- `Parts`: every node submits a `Part` that will be used for the final key generation, it contains encrypted data that will be used for generating their key-share.
- `Acks`: nodes check the `Part`s and submit their `Ack`s over the `Part`s. These `Ack`s will also be used for the key generation.
- `AllAcks`: everyone makes sure that they all have the same set of `Acks` and `Parts` by sending their version of the sets. This last part is here to make sure that the candidates end up generating the same section key!

Votes are all signed so cheaters can be denounced with those cryptographic proofs.

```rust
/// Votes exchanged for DKG process.
NodeMsg::DkgVotes {
    /// The identifier of the DKG session this message is for.
    session_id: DkgSessionId,
    /// The ephemeral bls public keys used for this Dkg round
    pub_keys: BTreeMap<XorName, (BlsPublicKey, Signature)>,
    /// The DKG message.
    votes: Vec<DkgSignedVote>,
}
```

Votes contain the ephemeral `pub_keys` from the previous step, this has two purposes:
- nodes can check that they have the same keys and that nobody cheated (one node using two different keys)
- nodes that missed ephemeral keys in the first step can catch up using these keys

Once the nodes have finished voting, meaning that they received everyone's signatures over both sets (`AllAcks`), they can finally generate their keyshare. We call this DKG termination.

Sometimes though, messages are lost, so we need mechanisms to counter this.

## AE and Gossip

When a node receives a vote that they don't understand (eg. an `Ack` vote when they don't have all the `Parts` yet) they send an AE request to the sender of this vote: `NodeMsg::DkgAE(DkgSessionId)`. The sender will respond to this with all their votes in a `NodeMsg::DkgVotes`.

If for some reason a node has not received any DKG messages for a while, and if they are expecting some (didn't reach termination), they send out all their votes to the others in a `NodeMsg::DkgVotes`. If DKG voting didn't start yet they send out their ephemeral key instead.

Sometimes a node will receive a bunch of votes from another node (either from AE or gossip). If they notice that votes are missing from this batch, they send out all their votes in return to the sender. This way nodes actively keep each other up to date. This mechanism works even after a node reached termination, until the DKG session is discarded when enough section churn happened. The reason for this is that we want every node to terminate eventually, even if they missed many votes and are still gossiping long after the others terminated.

> In practice, gossip (with active responses including missing votes) alone can suffice and AE is not necessary. We keep it because it's more efficient but it is not necessary.

## Going to Handover

When DKG finishes, nodes generate their key shares and send out the new SAP signed (with that new key share) `NodeMsg::RequestHandover` to the elders. Once the elders receive super-majority of those, they know they can propose those in a `Handover` consensus round.

## Links

- [hbbft keygen code](https://github.com/poanetwork/hbbft/blob/master/src/sync_key_gen.rs)
- [hbbft keygen docs](https://docs.rs/hbbft/latest/hbbft/sync_key_gen/index.html)
