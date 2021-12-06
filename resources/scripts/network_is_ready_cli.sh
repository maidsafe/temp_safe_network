#!/usr/bin/env bash

echo "Waiting for a healthy network to be detected, as per the 'split_network_assert_health_check' test in sn/src/lib.rs "
until ./resources/scripts/nodes_finished_joining.sh || $( sleep 30 && false ); do :; done
until ./resources/scripts/has_split.sh || $( sleep 5 && false ); do :; done
