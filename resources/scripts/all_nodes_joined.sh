#!/usr/bin/env bash

if ! command -v rg &> /dev/null
then
    echo "ripgrep could not be found and is required"
    exit 1
fi

DEFAULT_ELDER_COUNT=7
DEFAULT_NODE_COUNT=30
DEFAULT_LOG_DIR="$HOME/.safe/node/local-test-network"

NODE_COUNT="${NODE_COUNT:-$DEFAULT_NODE_COUNT}"
ELDER_COUNT="${SN_ELDER_COUNT:-$DEFAULT_ELDER_COUNT}"

# It's better to use the network health test in rust as it's type safe.
# This is needed for windows though at the moment due to the logfile locking...
echo "Checking logfiles to check all nodes have joined"
log_dir="${LOG_DIR:-$DEFAULT_LOG_DIR}"

# -u needed here to search log dirs
nodes_joined=$(rg "ReceivedJoinApproval" "$log_dir" -g "*.log*" -u -c | wc -l)

echo "Joined nodes found: $nodes_joined"

# Node count will always be 1 less than total nodes as genesis does not join anything.
if [[ $nodes_joined -ne $(($NODE_COUNT - 1)) ]]
    then
        echo "Unexpected number of joined nodes. Expected $(($NODE_COUNT -1)), we have $nodes_joined"
        list_nodes_joined=$(rg "ReceivedJoinApproval" "$log_dir" -g "*.log*" -u -c)
        echo "Nodes joined logs found:"
        echo "$list_nodes_joined"
        exit 100
    else
        echo "All nodes have joined!"
        exit 0
fi
