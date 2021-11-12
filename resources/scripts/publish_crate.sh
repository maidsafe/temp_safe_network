#!/usr/bin/env bash
# This script uses a retry loop to wait on dependent crates becoming available at the correct version.

crate_name=$1
dependent_crate_version=$2
dependent_crate_name=$3

if [[ -z "$crate_name" ]]; then
    echo "You must supply the name of the crate to publish"
    exit 1
fi

if [[ -z "$dependent_crate_version" ]]; then
    echo "You must supply the version of the dependent crate"
    exit 1
fi

if [[ -z "$dependent_crate_name" ]]; then
    echo "You must supply the name of the dependent crate"
    exit 1
fi

count=1
max_retries=15
current_version=$(cargo search $dependent_crate_name | head -n 1 | awk '{print $3}' | sed 's/\"//g')
while [[ $current_version != $dependent_crate_version && $count -le $max_retries ]]
do
    echo "Version of $dependent_crate_name reported by crates.io is $current_version"
    echo "Waiting for $dependent_crate_name crate to reach $dependent_crate_version"
    echo "Attempted $count of $max_retries times. Will query again in 5 seconds..."
    sleep 5
    ((count++))
    current_version=$(cargo search $dependent_crate_name | head -n 1 | awk '{print $3}' | sed 's/\"//g')
done

if [[ $count -gt $max_retries ]]; then
    echo "Max retry attempts exceeded. Exiting with failure."
    exit 1
fi

cd $crate_name
cargo publish --allow-dirty
