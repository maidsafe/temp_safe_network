#!/usr/bin/env bash

echo "Waiting for a healthy network to be detected, as per the 'split_network_assert_health_check' test in sn/src/lib.rs "
until ./resources/scripts/has_split.sh || $( sleep 5 && false ); do :; done
