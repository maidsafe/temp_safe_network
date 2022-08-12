# Traceroute

Traceroute was introduced as a feature to trace an operation/a message’s flow through the network. Right from its origin, the entities that create/process/forward this message insert their identity(PublicKey) into a trace that is carried along in the message’s header. Once the flow is complete, the trace is logged via regular logging method, and the trace is dropped.

Example:

- A Client sending a RegisterRead query creates the message along with its PK in the trace. This message is then sent to 3 closest Elders individually.
- The Elders who then handle this message append their identity to the trace before forwarding the query to their respective Adults.
- The adults after processing the query then add their identity to the message’s traces and reply with the response back to the Elder
- Elders on receiving the response from Adults forward it to the Client after appending their identity once again to the trace of the message.
- The client on receiving the response can then see what the nodes/entities processed the query that it sent initially by looking into the traceroute of the query response:
```
Traceroute: Client(Ed25519(PublicKey(7430..5bbb))) => Elder(Ed25519(PublicKey(f5ac..0bff))) => Adult(Ed25519(PublicKey(5a42..2634))) => Elder(Ed25519(PublicKey(f5ac..0bff)))
```

This shows us the path the query took through the network to get processed and return a query response.

## Implementation
Traceroute is implemented by enabling nodes to append their identities to a Vec<PublicKey> field in the MsgEnvelope of WireMsgHeader. Nodes that create (or) handle (or) forward a message is supposed to add its PublicKey to the already present list of PKs. It is a feature-gated implementation that requires the “traceroute” flag to be enabled, although it is now temporarily enabled by default for debugging.

As of now, only Client Cmds, CmdAcks, Queries, QueryResponses, and AE-SystemMsgs support traceroute. This can be expanded in the future to support other SystemMsgs and be logged at the end of each flow to see how messages have traveled across the network. Though care must be taken to not overuse this as a trace for every message might be a bit overwhelming in the logs.

Another caveat is that it is easy to drop a trace mid-way through its flow, therefore extra care must be taken to ensure that traces are carried over if new WireMsgs are created in the same flow.

## Future features and optimizations
To avoid overwhelming the regular logs with message traces, we could write them down to a separate file in a specific format with their corresponding MsgIDs when their flow ends. This file could then be exported to ELK to visualize which Elders/Adults were called upon during execution and be used to create a Heatmap of the network.
