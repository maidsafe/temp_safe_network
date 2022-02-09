#!/usr/bin/env bash

set -e -x

# The default timeout value is 120 seconds, which causes NRS to run extremely slow.
export SN_QUERY_TIMEOUT=10
export RUST_BACKTRACE=1
export TEST_BOOTSTRAPPING_PEERS=$(cat ~/.safe/node/node_connection_info.config)

# TODO: enable doc tests, for now they require work
cd sn_api && cargo test --lib --release -- --test-threads=2
