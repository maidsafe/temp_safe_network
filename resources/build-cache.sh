#!/usr/bin/env bash

if [[ -z "$build_type" ]]; then
    echo "build_type must be set to dev or non-dev"
    exit 1
fi

if [[ -z "$build_component" ]]; then
    echo "build_component must be set safe-api, safe-cli or safe-ffi"
    exit 1
fi

if [[ -z "$build_target" ]]; then
    echo "build_target must be set to a valid Rust 'target triple'"
    exit 1
fi

cd "$build_component"
if [[ "$build_type" == "dev" ]]; then
    cargo build --release --tests --features=mock-network,fake-auth --target="$build_target"
else
    cargo build --release --target="$build_target"
fi
