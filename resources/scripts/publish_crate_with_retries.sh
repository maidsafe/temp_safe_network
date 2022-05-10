#!/usr/bin/env bash

crate_name=$1
if [[ -z "$crate_name" ]]; then
  echo "You must supply the name of the crate to publish"
  exit 1
fi

count=1
max_retries=15
while [[ $count -le $max_retries ]]
do
  (
    echo "Attempt $count to publish $crate_name..."
    cd $crate_name
    cargo publish --allow-dirty
    exit_code=$?
    if [[ $exit_code -eq 0 ]]; then
      echo "Successfully published $crate_name"
      exit 0
    fi
  )
  ((count++))
  echo "Publishing failed. Will sleep for 5 seconds then retry."
  sleep 5
done

echo "Max retry attempts exceeded. Failed to publish crate $crate_name."
exit 1
