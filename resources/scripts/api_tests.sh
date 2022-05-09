#!/usr/bin/env bash

set -e -x

export RUST_BACKTRACE=1
export TEST_BOOTSTRAPPING_PEERS=$(cat ~/.safe/node/node_connection_info.config)

# TODO: enable doc tests, for now they require work
cargo test -p sn_api --lib --release
