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
2. A JoinRequest::Initiate is then sent to that closest section’s elders with their respective section PK.
3. The Elders then run the below set of verifications in sequence on the joining node:
   1. If the prefix of the joining node does not match the section prefix that the request processing node is in, it sends a Joinresponse::Redirect response which contains the latest SAP of the closest section back to the joining node. This allows the joining node to start the join process again with the correct contacts this time.
   2. If the processing node is not an Elder, the request is dropped as only Elders are allowed to recruit nodes for the network.
   3. If the section key that was sent with the JoinRequest is outdated, a JoinResponse::Retry is sent with the latest SAP details.
   4. If the age of the node(which is calculated from its name) is not the expected age in the network, a JoinResponse::Retry is sent with the latest SAP details and the expected age so that the joining node can retry with the correct age.
   5. A communications test is done by performing a reachability test on the joining to verify if communications are fine. The request is dropped if the test fails.
4. Upon successful verification of all the above-mentioned checks, a ResourceProof challenge is sent via JoinResponse::ResourceChallenge back to the joining node that it must solve successfully.
5. The joining node solves the ResourceProof challenge and sends back a JoinRequest::SubmitResourceProof with the answer to be verified.
6. On receiving the JoinRequest::SubmitResourceProof the Elders validate it to make sure the joining node has indeed solved the problem and has the ability to perform given tasks. In case the ResourceProof is not validated, the JoinRequest is dropped silently.
7. If the ResourceProof check is cleared, the Elders then proceed to propose a membership change among their group of other Elders.
8. The group then votes on the set of joining nodes that will be recruited into their section as a batch.
9. Upon reaching consensus on membership voting, the joining node who ideally would be in the batch of accepted nodes will be sent a JoinResponse::Approved response with a slew of details about the network.
10. With the JoinResponse::Approved and all of the information that was sent along is used by the joining node to initiate its necessary modules to start performing tasks as a part of the network

Note: If there is no response from the network nodes during this to-and-fro exchange the joining node will timeout after the JOIN_TIMEOUT duration is elapsed(this is currently set to 100 seconds).

## The Relocation Process
For node ageing, there is also Relocation process to make node having its age increased and name changed correspondently.

The detailed process is as following:
1. When elder detected a relocation candidate, the elders sign and send the vote SectionStateVote::NodeIsOffline(NodeState.state::Relocated) to each other.
2. Once the vote aggregated, elders attempt to propose a membership change of the section signed NodeState.
3. Once the membership votes aggregated, elders updates its network_knowledge(i.e. remove that node from the active member list) AND send a notification to the relocation candidate.
4. When the candidate received the notification, it send FIRST JoiningAsRelocatedRequest to the elders of the target section, note this is using the old name
5. When the target section elders received the request, they respond with their latest sap within JoiningAsRelocatedResponse
6. When the candidate received the JoiningAsRelocatedResponse, it generate new keypair and switch to it, then send SECOND JoiningAsRelocatedRequest to the elders of the target section, note this is using the new name
7. When the target section elders received the second request, they start vote for membership change of NodeIsOnline,
8. Once the vote of NodeIsOnline aggregated, the candidate will receive the notifications and consider the Relocation process completed.


