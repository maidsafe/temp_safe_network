# Safe Network

The Safe Network is a fully autonomous data and communications network. For a general introduction and more information about its features and the problems it intends to solve, please see [The Safe Network Primer](https://primer.safenetwork.org/).

The network is implemented in Rust. This repository is a workspace consisting of 4 crates:

* [safe_network](https://crates.io/crates/safe_network): provides the core implementation of network features and the node binary.
* [sn_api](https://crates.io/crates/sn_api): an interface to expose network features for client applications.
* [sn_cli](https://crates.io/crates/sn_cli): a command line interface for using the network.
* sn_cmd_test_utilities: internal, unpublished crate that provides tools for integration testing.

Currently, the network can be used via these 3 published crates. To see how, a good place to start is the [README](sn_cli/README.md) for the CLI. You can run your own local network or perhaps participate in a remote network.

## Testing

Some of the `safe_network` tests require a live network to test against.

### Running a local testnet

You should first ensure that your local machine does not have any artefacts from prior runs. Eg on unix: `killall sn_node ||true && rm  -r ~/.safe/node/local-test-network || true` will remove any running `sn_node` instances and remove any prior run's data stored.

You can then run a local testnet using the `testnet` bin:

`NODE_COUNT=15 RUST_LOG=safe_network=trace cargo run --release --bin testnet`

`NODE_COUNT` defaults to 33 nodes and will give you a split section. 15 nodes as above will give only one section. How many nodes you want to run will depend on your hardware. 15 nodes can be considered the minimum for a viable section.

### Running tests

Once you have your network running you can simply run `cargo test --release`. This will run _all_ tests in `sn`. 

> Note: if you're running in the root directory, either `cd sn` before running tests or inclide `-p safe_network` in the cargo command to target _only_ that package. Otherwise you'll be running tests from _all_ crates, including sn_api and sn_cli. Eg: `cargo test --release -p safe_network`

In general it can be useful to scope your test running, eg `cargo test --release client_api` will run _only_ the client tests. Or perhaps you want to ignore `proptests` as they can be quite long: `cargo test --release client_api --skip proptest`



## Releases

Safe is being developed iteratively and has frequent [releases](https://github.com/maidsafe/safe_network/releases). You can use these to experiment with new features when they become available. It's also possible to participate in community 'testnets' hosted by members of [The Safe Network Forum](https://safenetforum.org/).

## License

This Safe Network repository is licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).

### Linking exception

safe_network is licensed under GPLv3 with linking exception. This means you can link to and use the library from any program, proprietary or open source; paid or gratis. However, if you modify safe_network, you must distribute the source to your modified version under the terms of the GPLv3.

See the [LICENSE](LICENSE) file for more details.

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
