#!/usr/bin/env bash

echo "Waiting for all nodes to join..."
until ./resources/scripts/all_nodes_joined.sh || $( sleep 15 && false ); do :; done
