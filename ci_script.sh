#!/bin/bash

# Show expanded commands while running
set -x

# Stop the script if any command fails
set -o errtrace
trap 'exit' ERR

if [[ $APPVEYOR = true ]]; then
  cd $APPVEYOR_BUILD_FOLDER
else
  cd $TRAVIS_BUILD_DIR
fi

if [[ ! $TRAVIS_RUST_VERSION = nightly ]]; then
  cargo test --release --no-run
  cargo test --release --features use-mock-routing
fi
