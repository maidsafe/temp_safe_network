#!/usr/bin/env bash

if ! command -v rg &> /dev/null
then
    echo "ripgrep could not be found and is required"
    exit
fi


echo "Checking logfiles to check for split"
log_dir="$HOME/.safe/node/local-test-network"
# -u needed here to search log dirs
prefix_count=$(rg "Prefix\(0\)" "$log_dir" -u | wc -l)

if ! [ $prefix_count -gt 0 ]
    then
        echo "No split, try changing INCREASED_COUNT in the script!"
        exit 100
fi

echo "Prefix(0) found $prefix_count times. Successful split!"
