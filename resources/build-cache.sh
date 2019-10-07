#!/usr/bin/env bash

if [[ -z "$build_type" ]]; then
    echo "build_type must be set to dev or non-dev"
    exit 1
fi

if [[ -z "$build_component" ]]; then
    echo "build_component must be set safe-api, safe-cli or safe-ffi"
    exit 1
fi

cd "$build_component"
if [[ "$build_type" == "dev" ]]; then
    cargo build --release --tests --features=mock-network,fake-auth
else
    cargo build --release
fi
