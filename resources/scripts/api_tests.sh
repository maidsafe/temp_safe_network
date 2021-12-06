#!/usr/bin/env bash

set -e -x

function run_api_tests() {
    export TEST_BOOTSTRAPPING_PEERS=$(cat ~/.safe/node/node_connection_info.config)

    # TODO: enable doc tests and disable test_fetch_ for now as tests require work
    cd sn_api && cargo test --lib --release -- --skip test_fetch_
    cd -
}

run_api_tests
