#!/bin/bash

set -e -x

rm -rf images
mkdir images

# Ensure Cargo.lock sn_node version is up to date - must be up to date with latest version for cargo-deps to run
cargo install cargo-deps

cargo deps --all-deps --include-orphans --filter qp2p lru_time_cache sn_fake_clock xor_name bls_signature_aggregator resource_proof sn_launch_tool | dot -T png -Nfontname=Iosevka -Gfontname=Iosevka -o images/sn_node_maidsafe_dependencies.png
cargo deps | dot -T png -o images/sn_all_dependencies.png
