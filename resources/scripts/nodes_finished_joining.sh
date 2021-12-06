#!/usr/bin/env bash

if ! command -v rg &> /dev/null
then
    echo "ripgrep could not be found and is required"
    exit
fi

DEFAULT_NODE_COUNT=33
NODE_COUNT="${SN_NODE_COUNT:-$DEFAULT_NODE_COUNT}"

# It's better to use the network health test in rust as it's type safe.
# This is needed for windows though at the moment due to the logfile locking...
echo "Checking logfiles to check for split"
log_dir="$HOME/.safe/node/local-test-network"

# -u needed here to search log dirs
nodes_that_received_approval=$(rg "ReceivedJoinApproved" "$log_dir" -g "*.log.*"  -u -c | wc -l)

echo "nodes_that_received_approval: $nodes_that_received_approval  ."
echo "expected nodes: $NODE_COUNT  ."

# 14 elders after or more (we're not discounting demotions here...)
if ! [[ $nodes_that_received_approval -gt $(($NODE_COUNT - 2)) ]]
    then
        echo "Not all nodes have joined yet..."
        exit 100
    else
        echo "All nodes have joined"
fi
