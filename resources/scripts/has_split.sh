#!/usr/bin/env bash

if ! command -v rg &> /dev/null
then
    echo "ripgrep could not be found and is required"
    exit
fi

# It's better to use the network health test in rust as it's type safe.
# This is needed for windows though at the moment due to the logfile locking...
echo "Checking logfiles to check for split"
log_dir="$HOME/.safe/node/local-test-network"

echo "Log dir contains:"
ls -la $log_dir



# -u needed here to search log dirs
prefix1_prior_elder_nodes=$(rg "SplitSuccess: Prefix\(1\)" "$log_dir" -u -c | wc -l)
prefix1_new_elder_nodes=$(rg "PromotedToElder: Prefix\(1\)" "$log_dir" -u -c | wc -l)
prefix0_prior_elder_nodes=$(rg "SplitSuccess: Prefix\(0\)" "$log_dir" -u -c | wc -l)
prefix0_new_elder_nodes=$(rg "PromotedToElder: Prefix\(0\)" "$log_dir" -u -c | wc -l)


echo "Prefix(1) prior nodes found $prefix1_prior_elder_nodes times."
echo "Prefix(1) new nodes found $prefix1_new_elder_nodes times."
echo "Prefix(0) prior nodes found $prefix0_prior_elder_nodes times."
echo "Prefix(0) new nodes found $prefix0_new_elder_nodes times."

total_prefix0_elders=$(($prefix0_new_elder_nodes + $prefix0_prior_elder_nodes))
total_prefix1_elders=$(($prefix1_new_elder_nodes + $prefix1_prior_elder_nodes))
total_elders=$(($total_prefix1_elders + $total_prefix0_elders))



if ! [[ $total_elders -eq 14 ]]
    then
        echo "No split, retry or perhaps change NODE_COUNT!"
        exit 100
    else
        echo "Successful split!"
fi
