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
echo "Checking logfiles to check for split"
log_dir="$HOME/.safe/node/local-test-network"

# -u needed here to search log dirs
prefix1_prior_elder_nodes=$(rg "StillElderAfterSplit: Prefix\(1\)" "$log_dir" -g "*.log*"  -u -c | wc -l)
prefix1_new_elder_nodes=$(rg "PromotedToElder: Prefix\(1\)" "$log_dir" -g "*.log*"  -u -c | wc -l)
prefix0_prior_elder_nodes=$(rg "StillElderAfterSplit: Prefix\(0\)" "$log_dir" -g "*.log*"  -u -c | wc -l)
prefix0_new_elder_nodes=$(rg "PromotedToElder: Prefix\(0\)" "$log_dir" -g "*.log*"  -u -c | wc -l)
split_count=$(rg "SplitSuccess" "$log_dir" -g "*.log*"  -u -c | wc -l)
relocation_start_nodes=$(rg "RelocateStart" "$log_dir" -g "*.log*"  -u -c | wc -l)
relocation_end_nodes=$(rg "RelocateEnd" "$log_dir" -g "*.log*"  -u -c | wc -l)

echo "Prefix(1) prior nodes found: $prefix1_prior_elder_nodes ."
echo "Prefix(1) new nodes found: $prefix1_new_elder_nodes ."
echo "Prefix(0) prior nodes found: $prefix0_prior_elder_nodes ."
echo "Prefix(0) new nodes found: $prefix0_new_elder_nodes ."
echo "Relocation start nodes found: $relocation_start_nodes ."
echo "Relocation end nodes found: $relocation_end_nodes ."
echo "split_count: $split_count ."

# With ElderSize=7 and NetworkSize=33, high chance there will be no relocation happens
# With ElderSize=7 and NetworkSize=45, high chance there will be relocation happens
# With ElderSize=5 and NetworkSize=33, high chance there will be relocation happens
# With ElderSize=5 and NetworkSize=45, high chance there will be relocation and split into prefix(XX)
if ! [[ $relocation_start_nodes -eq $relocation_end_nodes ]]
    then
        echo "Some relocations were not completed successfully!"
        exit 100
    else
        echo "All relocations completed successfully!"
fi

total_prefix0_elders=$(($prefix0_new_elder_nodes + $prefix0_prior_elder_nodes))
total_prefix1_elders=$(($prefix1_new_elder_nodes + $prefix1_prior_elder_nodes))
total_elders=$(($total_prefix1_elders + $total_prefix0_elders))

# 14 elders after or more (we're not discounting demotions here...)
echo "$split_count is bigger than $((2*$ELDER_COUNT - 1))?"
if ! [[ $split_count -gt $((2*$ELDER_COUNT - 1)) ]]
    then
        echo "No split, retry or perhaps change NODE_COUNT!"
        exit 100
    else
        echo "Successful split!"
        exit 0;
fi
