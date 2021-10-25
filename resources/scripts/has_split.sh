#!/usr/bin/env bash

if ! command -v rg &> /dev/null
then
    echo "ripgrep could not be found and is required"
    exit
fi

echo "Checking logfiles to check for split"
log_dir="$HOME/.safe/node/local-test-network"
# -u needed here to search log dirs
prefix_empty_count=$(rg "prefix=\(\)" "$log_dir" -u | wc -l)
prefix_0_count=$(rg "prefix=\(0\)" "$log_dir" -u | wc -l)
prefix_1_count=$(rg "prefix=\(1\)" "$log_dir" -u | wc -l)
prefix_0_attempt_count=$(rg "Prefix\(0\)" "$log_dir" -u | wc -l)
prefix_1_attempt_count=$(rg "Prefix\(1\)" "$log_dir" -u | wc -l)

echo "Prefix() found $prefix_empty_count times."
echo "Prefix(0) found $prefix_0_count times."
echo "Prefix(1) found $prefix_1_count times."
echo "Attempts to split to Prefix(0) found $prefix_0_attempt_count times."
echo "Attempts to split to Prefix(1) found $prefix_1_attempt_count times."

if ! [[ $prefix_0_count -gt 0 && $prefix_1_count -gt 0 ]]
    then
        echo "No split, try changing NODE_COUNT!"
        exit 100
    else
        echo "Successful split!"
fi
