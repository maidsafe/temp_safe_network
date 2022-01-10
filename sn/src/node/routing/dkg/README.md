# Usage of DKG

Whenever there is a change in Elders in the network [Distributed Key Generation](https://github.com/maidsafe/bls_dkg#bls-dkg) is used to generate a new set of BLS key shares for individual Elders along with the SectionKey which will represent the section. 

DKG is triggered by the following events:
- A change in the Elders
- Section Split

## SessionId

- Each DKG session is identified by a `DkgSessionId` which contains a hash and a generation. The hash contains the list of elder candidates and the generation - the length of the chain when a new DKG round is starting. This allows us to identify older DKG sessions and clean them up when not required.

## When does DKG happen?

- For the genesis node. There is no DKG. A random section key is generated. This will be the genesis key of the network.
- During network startup a new DKG round is always started for the first `ELDER_COUNT` nodes. For 2..7 nodes, when the node is approved into the section, it calls the `routing::Core::promote_and_demote_elders()` function. This sends a `DkgStart { ... }` message to all elder candidates which starts a new DKG round.
- Whenever there is a change in members i.e. node joining / leaving / relocation we call the `routing::Core::promote_and_demote_elders()` function which again starts a new DKG if there is a desired change in Elders.
- When a DKG round has failed the active elders will aggregate their signatures and reach a `DkgFailureAgreement` on which nodes have failed to participate in the DKG. A new DKG is then triggered excluding the inactive nodes using `routing::Core::promote_and_demote_elders_except(Vec<XorName>)`

## Pre-DKG preparations

- Before a DKG session kicks off, the `DkgStart { ... }` message is individually signed by the current _set of elders_ and sent to the _new elder candidates_ to be accumulated. This is to prevent nodes from spamming `DkgStart` messages which might lead to unnecessary DKG sessions.

## Handling DKG outcome

There are two possible outcomes of DKG. A failure and a success.
Here again, we don't rely on a single node to report the outcome of the DKG as successful / a failure.

### Handling a failure

When DKG has failed at a particular node, it can raise a dkg failure event. If several nodes report such a failure, i.e.
```
nodes_that_voted_for_failure > participants_count - supermajority(participant_count)
```

`participants_count - supermajority(participant_count)` is the super-minority.

it means that a supermajority on a successful outcome is not possible. In this case, DKG failure observations are raised from the nodes along with the list of nodes that failed to participate in the DKG. The DKG is then restarted without these nodes.

### Post successful DKG

There are a number of steps that occur after successful DKG. 
- Each of the participants will generate the section key and their KeyShare and raise a `Proposal::SectionInfo(new_sap)`. This is signed using the _newly generated_ key share and sent to the current Elders.
- Once the Elders aggregate these signatures, they sign the new SAPs and send them to the new elder candidates through a `Proposal::NewElders(SAP)`.
- Each of the new elder candidates will then get new SAP(s) that now contains network authority (section signed)
- The SAP(s) are applied locally and the next steps follow. i.e. sharing data, split etc.