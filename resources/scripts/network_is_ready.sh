#!/usr/bin/env bash

echo "Waiting for a haelthy network to be detected, as per the 'split_network_assert_health_check' test in sn/src/lib.rs "
cd sn
until cargo test --release --features=always-joinable,test-utils -- --ignored split_network_assert_health_check || sleep 5; do :; done
cd ..
