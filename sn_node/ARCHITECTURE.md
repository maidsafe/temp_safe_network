# Architecture of sn_node

This document attempts to give a high level overview of the `sn_node` crate.

A node mainly does these important things:
- Connect to a network
- Communicating with other nodes and clients on the network
- Storing data (chunks or registers) belonging to the section
- Serving data on request
- Verify other nodes behave well


## [`comm`](src/comm/)

To join an existing network, the node reaches out to a node already part of the network. The `comm` module handles this initial contact using `qp2p`. An endpoint (socket) is created, listening for _incoming_ new connections. Also, _outgoing_ connections are made on demand when sending messages out. For all connections, a task is spawned listening for incoming messages.

These incoming messages are passed up through a channel, to be handled outside of `comm`. Essentially, `comm` is responsible for connections and messages, but not about the content of these messages.

`comm` uses `qp2p` heavily, which helps streamline how connections are made and messages are processed; it depends on `quinn`, an implementation of QUIC.


## [`node/flow_ctrl`](src/node/flow_ctrl/)

Flow control encompasses the way incoming messages are prioritised and flow through a kind of pipeline. `FlowCtrl` takes incoming messages (sent from `comm`), parses these as commands and pushes these into a prioritised queue (`CmdCtrl`).

A message is initially always encoded as a command that is to be validated (`Cmd::ValidateMsg`). Processing commands from the queue can spawn new commands, meaning the `Cmd::ValidateMsg` should result in a new `Cmd`. The processing logic is located in [`node/flow_ctrl/dispatcher`](src/node/flow_ctrl/dispatcher.rs).

This queue is also the way for the node to queue tasks for itself, like cleaning up connections to peers and ask for messages to be sent.


## [`node/node_starter`](src/node/node_starter.rs)

Instantiating the node happens in the node starter. This involves using the `comm` module to bootstrap the node and setting message handling. While `comm` merely establishes 'raw' connections, the node starter actually attempts to join the network, waiting for approval.

Once bootstrapped, a task is spawned that loops the flow control, which is the main process that can be seen as the event loop.


### The actual binary: [`bin/sn_node`](bin/sn_node/)

`sn_node` is the name of the binary that runs the Safe node. This mainly involves high-level error handling and printing, configuration setup and calling on the node starter.
