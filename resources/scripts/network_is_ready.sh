#!/usr/bin/env bash

echo "Waiting for a healthy network to be detected, as per the 'split_network_assert_health_check' test in sn/src/lib.rs "
cd sn
until cargo test --lib --release -- --ignored split_network_assert_health_check || $( sleep 15 && false ); do :; done
cd ..
