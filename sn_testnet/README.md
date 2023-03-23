# sn_testnet

This crate has utilities for working with local test networks.

It exposes two structs, `Testnet` and `TestnetBuilder`, that can be used to configure and launch local test networks:
```
let (mut testnet, network_contacts_path) = Testnet::configure()
    .node_bin_path("~/.safe/node/safenode")
    .node_launch_interval(5000)
    .clear_nodes_dir()
    .flamegraph_mode(true)
    .build()?;
testnet.launch_genesis(None, vec["--json-output"])?;
testnet.launch_nodes(30, &network_contacts_path, vec!["--json-output"])?;
testnet.configure_network_contacts(&network_contacts_path)?;
```

It also has a binary, `testnet`, which can be used to create local test networks and have new nodes join an existing network. Run `testnet --help` to see the tool can be used.

## License

This Safe Network repository is licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).
