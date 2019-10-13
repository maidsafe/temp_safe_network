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
    case "$build_component" in
        safe-ffi)
            cargo build --release --features=mock-network --target="$build_target"
            ;;
        safe-cli)
            cargo build --release --tests --features=mock-network,fake-auth --target="$build_target"
            ;;
        safe-api)
            cargo build --release --lib --tests --features=mock-network,fake-auth --target="$build_target"
            ;;
    esac
else
    cargo build --release --target="$build_target"
fi
