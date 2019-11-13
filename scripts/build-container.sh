#!/usr/bin/env bash

set -e -x

target=$1
if [[ -z "$target" ]]; then
    echo "You must supply the target for the build."
    echo "Valid values are rust target triples, e.g. 'x86_64-unknown-linux-gnu', 'safe-api' or 'safe-ffi'."
    exit 1
fi

build_type=$2
if [[ -z "$build_type" ]]; then
    echo "You must supply the type for the build."
    echo "Valid values are 'dev' or 'prod'."
    exit 1
fi

function get_dockerfile_name() {
    case $target in
        x86_64-unknown-linux-gnu)
            echo "scripts/Dockerfile.build"
            ;;
        x86_64-linux-android)
            echo "scripts/Dockerfile.android.x86_64.build"
            ;;
        armv7-linux-androideabi)
            echo "scripts/Dockerfile.android.armv7.build"
            ;;
        *)
            echo "$target is not supported. Please extend to support this target."
            exit 1
            ;;
    esac
}

tag="$target-$build_type"
rm -rf target/
docker rmi -f maidsafe/safe-client-libs-build:"$tag" || true
docker build -f $(get_dockerfile_name) -t maidsafe/safe-client-libs-build:"$tag" \
    --build-arg build_target="$target" \
    --build-arg build_type="$build_type" .
