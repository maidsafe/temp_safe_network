#!/usr/bin/env bash

set -e -x

function run_api_tests() {
    export TEST_BOOTSTRAPPING_PEERS=$(cat ~/.safe/node/node_connection_info.config)

    # TODO: enable doc tests, for now they require work
    cd sn_api && cargo test --lib --release -- --test-threads=1
    cd -
}

run_api_tests
