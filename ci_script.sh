#!/bin/bash

# Show expanded commands while running
set -x

# Stop the script if any command fails
set -o errtrace
trap 'exit' ERR

if [[ $TRAVIS = true ]]; then
  echo "Into Travis ----"
  cd $TRAVIS_BUILD_DIR
else
  echo "Appveyor Folder ----"
  echo $APPVEYOR_BUILD_FOLDER
  cd $APPVEYOR_BUILD_FOLDER
  echo "ABCD Folder ----"
  echo $ABCD
  echo "ASDF Folder ${ASDF}----"
fi

if [[ ! $TRAVIS_RUST_VERSION = nightly ]]; then
  cargo test --release --no-run
  cargo test --release --features use-mock-routing
fi
