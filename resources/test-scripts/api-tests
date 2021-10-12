#!/usr/bin/env bash

set -e -x

function run_api_tests() {
    export TEST_AUTH_CREDENTIALS=$(cat ~/.safe/cli/credentials)
    export TEST_BOOTSTRAPPING_PEERS=$(cat ~/.safe/node/node_connection_info.config)

    cargo test --release test_register_ -- --test-threads=1
    cargo test --release test_multimap_ -- --test-threads=1
    cargo test --release test_fetch_ -- --test-threads=1
    cargo test --release test_keys_ -- --test-threads=1
    cargo test --release test_safeurl_ -- --test-threads=1
    cargo test --release test_wallet_ -- --test-threads=1
    cargo test --release test_nrs_ -- --test-threads=1
    cargo test --release test_files_ -- --test-threads=1

    cd -
}

run_api_tests
