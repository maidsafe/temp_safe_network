#!/usr/bin/env bash

set -x

if ! command -v rg &> /dev/null
then
    echo "ripgrep could not be found and is required"
    exit 1
fi

if ! command -v jq &> /dev/null
then
    echo "jq could not be found and is required"
    exit 1
fi

log_dir="$HOME/.safe/node/local-test-network"
genesis_log_dir="$HOME/.safe/node/local-test-network/sn-node-genesis"
out_file="safe_states.out"

# Extract statemap metadata
rg -IN ".*STATEMAP_METADATA: " "$genesis_log_dir" --replace "" > "$out_file"
rg -IN ".*STATEMAP_ENTRY: " "$log_dir" --replace "" | jq -s 'sort_by(.time|tonumber)' | jq -c '.[]' >> "$out_file"

echo "Generated statemap data at $out_file"
echo "Render the statemap SVG with"
echo "    statemap -c 10000 $out_file > safe.svg"
