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

clean_build=$4
[[ -z "$clean_build" ]] && clean_build="false"

features=$5
if [[ -z "$features" ]]; then
    [[ "$component" == "safe-ffi" ]] && features="mock-network"
    [[ "$component" == "safe-cli" ]] && features="fake-auth,mock-network"
fi

function get_docker_build_command() {
    local build_command
    if [[ "$clean_build" == "true" ]]; then
        if [[ "$target" == *"linux"* ]]; then
            build_command="rm -rf /target/$target/release &&"
        else
            build_command="rm -rf target/$target/release &&"
        fi
    fi
    build_command="$build_command cargo build"
    [[ "$component" != "safe-cli" ]] && build_command="$build_command --lib"
    build_command="$build_command --release --manifest-path=$component/Cargo.toml --target=$target"
    echo $build_command
}

function build_on_linux() {
    local build_command
    local container_tag
    local uuid
    uuid=$(uuidgen | sed 's/-//g')
    container_tag=$(sed 's/safe-//g' <<< "$component")
    container_tag="$container_tag-$target"
    [[ $build_type == "dev" ]] && container_tag="$container_tag-dev"
    build_command=$(get_docker_build_command)
    docker run --name "$component-build-${uuid}" -v "$(pwd)":/usr/src/safe-cli:Z \
        -u "$(id -u)":"$(id -g)" \
        maidsafe/safe-cli-build:"$container_tag" \
        bash -c "$build_command"
    docker cp "$component-build-${uuid}":/target .
    docker rm "$component-build-${uuid}"
}

function build_bin() {
    [[ "$clean_build" == "true" ]] && rm -rf target
    if [[ "$build_type" == "dev" ]]; then
        cargo build --features="$features" \
            --release --manifest-path="$component/Cargo.toml" --target="$target"
    else
        cargo build --release --manifest-path="$component/Cargo.toml" --target="$target"
    fi
}

function build_lib() {
    [[ "$clean_build" == "true" ]] && rm -rf target
    if [[ "$build_type" == "dev" ]]; then
        cargo build --features="$features" \
            --release --lib --manifest-path="$component/Cargo.toml" --target="$target"
    else
        cargo build --release --lib --manifest-path="$component/Cargo.toml" --target="$target"
    fi
}

function build_on_windows() {
    case $component in
        safe-cli)
            build_bin
            ;;
        safe-ffi)
            build_lib
            ;;
        safe-api)
            build_lib
            ;;
        *)
            echo "$component is not supported. Please extend to support this component."
            exit 1
            ;;
    esac
}

function build_on_macos() {
    # Right now it's the same process for building on Windows.
    # Potentially that could change at some point.
    build_on_windows
}

function build() {
    uname_output=$(uname -a)
    case $uname_output in
        Linux*)
            build_on_linux
            ;;
        Darwin*)
            build_on_macos
            ;;
        MSYS_NT*)
            build_on_windows
            ;;
        *)
            echo "Platform not supported. Please extend to support this platform."
            exit 1
    esac
}

function clean_artifacts() {
    rm -rf artifacts
    mkdir artifacts
}

function get_artifacts() {
    find "target/$target/release" -maxdepth 1 -type f -exec cp '{}' artifacts \;
}

clean_artifacts
build
get_artifacts
