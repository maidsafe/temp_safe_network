# Safe Network `jq` filters

[`jq`](https://stedolan.github.io/jq/) is a command line tool for processing JSON.
The filters in this directory can be used to process the logs emitted by Safe Network binaries.

## Prerequisites

To use these filters, you need to enable the trace log level and JSON log output.
You can set the trace log level by setting `sn_node=trace` in the `RUST_LOG` environment variable.
JSON log output is currently only supported by `sn_node`, and can be enabled by supplying the `--json-logs` flag, e.g.

```sh
RUST_LOG=sn_node=trace sn_node <args> --json-logs
```

The `--json-logs` flag can also be used with the `testnet` binary:

```sh
RUST_LOG=sn_node=trace NODE_COUNT=11 testnet --json-logs
```

## Usage

A file can be used as a `jq` filter with the `-f` flag, and multiple files can be passed for processing using shell expansion, e.g.

```sh
jq -f resources/jq-filters/connection-tracing.jq ~/.safe/node/local-test-network/sn-node-*/sn_node.log.*
```
