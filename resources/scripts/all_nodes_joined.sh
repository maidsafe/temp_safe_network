#!/usr/bin/env bash

if ! command -v rg &> /dev/null
then
    echo "ripgrep could not be found and is required"
    exit 1
fi

DEFAULT_ELDER_COUNT=7
ELDER_COUNT="${SN_ELDER_COUNT:-$DEFAULT_ELDER_COUNT}"
DEFAULT_NODE_COUNT=30
NODE_COUNT="${NODE_COUNT:-$DEFAULT_NODE_COUNT}"

# It's better to use the network health test in rust as it's type safe.
# This is needed for windows though at the moment due to the logfile locking...
echo "Checking logfiles to check all nodes have joined"
log_dir="$HOME/.safe/node/local-test-network"

# -u needed here to search log dirs
nodes_joined=$(rg "ReceivedJoinApproval" "$log_dir" -g "*.log*"  -u -c | wc -l)

echo "Joined nodes found: $nodes_joined ."

# Node count will always be 1 less than total nodes as genesis does not join anything.
if ! [[ $nodes_joined -gt $(($NODE_COUNT - 2)) ]]
    then
        echo "Some nodes have not joined successfully! expected $(($NODE_COUNT -1)), we have $nodes_joined"
        exit 100
    else
        echo "All nodes have joined!"
        exit 0
fi
