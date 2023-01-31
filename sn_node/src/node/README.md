# sn_node

sn_node - a Safe Network Node.

This produces a `bin` which can be used to run a node on The Safe Network. It also provides APIs to do the same.

## Prerequisites
For a node to successfully join a network, it requires the below-mentioned things beforehand to start the joining process:
a) The Node Config: Config - which contains all the tweakables
b) Directories: These are derived from the config to generate various dirs where node_data and logs will be stored
d) Network contacts file - An address book for the SAFE Network
e) Join duration timeout - Timeout to not wait forever looping on a response during bootstrapping

## Overview
For a node to successfully join the network it first needs to contact a node that is actively a member of the SAFE Network. Contacts can be picked up from the provided Network contacts file where details about various sections of the network are available.

Once contacted the Joining node and the Network go through a set of verifications after which the Network gives a thumbs up to the joining node along with details for it to function as a part of the SAFE Network.

## The Join Process
1. The joining node fetches its closest section from the network contacts file.
2. A `NodeMsg::TryJoin` is then sent to that closest section’s elders.
3. The joining node then waits for the signed section information in its network knowledge, to contain itself as a member. In the meantime it receives msgs as normal.
4. Ae checks will ensure that the `NodeMsg::TryJoin` msg dst is correct, by updating the node if not. The node simply resends the msg after that.
5. The Elders then run the below set of verifications in sequence on the joining node:
   1. If the processing node is not an Elder, the request is dropped as only Elders handle joining nodes.
   2. If our prefix does not match the name the parameters were wrong, otherwise ae would have caught it. So msg is dropped.
   3. If the age of the node(which is calculated from its name) is not the expected age in the network, the msg is dropped, since it is a malformed msg since that state is unreachable with this code.
   4. (Temporarily disabled) A communications test is done by performing a reachability test on the joining to verify if communications are fine. The request is dropped if the test fails.
6. Upon successful verification of all the above-mentioned checks, the Elders propose a membership change where they vote for this node as joined.
7. Upon reaching consensus on membership voting, the joining node will be sent `JoinResponse::Approved` msg with a slew of details about the network.
8. The msgs sent to the node will entail ae-updates, which will make the network knowledge of the node contain it as member.
9. By that, the joining nodes finishes the waiting as it finds itself among the members, and continues working as a joined node.

Note: If there is no response from the network nodes during this to-and-fro exchange the joining node will timeout after the JOIN_TIMEOUT duration is elapsed (this is currently set to 60 seconds).

## The Relocation Process
For node ageing, there is also Relocation process to make node having its age increased and name changed correspondently.

The detailed process is as following:
1. When elders detect relocation candidates, they attempt to propose a membership change of these candidates as `Relocated`.
2. Once the membership votes aggregate, elders update their network_knowledge (i.e. remove those nodes from the active member list) AND notify relocating nodes with the decision, containing dst section.
3. When a relocating node receives the notification, it generates a new name and signs it with previous keys.
4. The relocating node then switches to the dst section and sends a join request.
5. When the dst section elders receive the join request, they validate the `RelocationProof` and then vote for this node as joined.
6. Once the vote is aggregated, the relocated node will receive the notifications and consider the relocation process completed.

## License

Licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).

### Linking exception

sn_node is licensed under GPLv3 with linking exception. This means you can link to and use the library from any program, proprietary or open source; paid or gratis. However, if you modify sn_node, you must distribute the source to your modified version under the terms of the GPLv3.

See the LICENSE file for more details.

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [Guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
