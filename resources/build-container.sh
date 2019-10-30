#!/usr/bin/env bash

set -e -x

component=$1
if [[ -z "$component" ]]; then
    echo "You must supply the component to build."
    echo "Valid values are 'safe-cli', 'safe-api' or 'safe-ffi'."
    exit 1
fi

target=$2
if [[ -z "$target" ]]; then
    echo "You must supply the target for the build."
    echo "Valid values are rust target triples, e.g. 'x86_64-unknown-linux-gnu', 'safe-api' or 'safe-ffi'."
    exit 1
fi

build_type=$3
if [[ -z "$build_type" ]]; then
    echo "You must supply the type for the build."
    echo "Valid values are 'dev' or 'non-dev'."
    exit 1
fi

function get_dockerfile_name() {
    case $target in
        x86_64-unknown-linux-gnu)
            echo "Dockerfile.build"
            ;;
        x86_64-linux-android)
            echo "Dockerfile.android.x86_64.build"
            ;;
        armv7-linux-androideabi)
            echo "Dockerfile.android.armv7.build"
            ;;
        *)
            echo "$target is not supported. Please extend to support this target."
            exit 1
            ;;
    esac
}

tag="$component-$target-$build_type"
rm -rf target/
docker rmi -f maidsafe/safe-cli-build:"$tag"
docker build -f $(get_dockerfile_name) -t maidsafe/safe-cli-build:"$tag" \
    --build-arg build_target="$target" \
    --build-arg build_type="$build_type" \
    --build-arg build_component="$component" .
