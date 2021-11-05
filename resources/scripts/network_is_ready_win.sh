#!/usr/bin/env bash

echo "Waiting for a healthy network to be detected, as per the 'split_network_assert_health_check' test in sn/src/lib.rs "
cd sn
until ./has_split || $( sleep 5 && false ); do :; done
cd ..
