#!/usr/bin/env bash

set -e

component=$1
if [[ -z "$component" ]]; then
    echo "You must supply the component to build."
    echo "Valid values are 'safe-cli', 'safe-api' or 'safe-ffi'."
    exit 1
fi

stamp=$2
if [[ -z "$stamp" ]]; then
    echo "Please pass the stamp for the release archives."
    echo "Use a version number, commit hash or a string such as 'nightly'."
    exit 1
fi

function get_distributable_for_component() {
    local component
    local distributable
    local target
    component=$1
    target=$2
    case "$component" in
        safe-cli)
            if [[ "$target" == *"windows"* ]]; then
                distributable="safe.exe"
            else
                distributable="safe"
            fi
            ;;
        safe-ffi)
            if [[ "$target" == *"darwin"* ]]; then
                distributable="libsafe_ffi.dylib"
            elif [[ "$target" == *"windows"* ]]; then
                distributable="safe_ffi.dll"
            elif [[ "$target" == *"linux"* ]]; then
                distributable="libsafe_ffi.so"
            else
                distributable="libsafe_ffi.a"
            fi
            ;;
        *)
            echo "$component not yet supported. Please extend this script for support."
            exit 1
            ;;
    esac
    echo "$distributable"
}

function get_archive_name() {
    local archive_name
    local component
    local target
    local type
    local extension
    component=$1
    target=$2
    type=$3
    extension=$4

    archive_name="$component-$stamp-$target"
    [[ "$type" == "mock" ]] && archive_name="$archive_name-dev"
    archive_name="$archive_name.$extension"
    echo "$archive_name"
}

function create_tar_archive() {
    local archive_name
    local distributable
    local component
    local target
    local type
    component=$1
    target=$2
    type=$3

    distributable=$(get_distributable_for_component "$component" "$target")
    archive_name=$(get_archive_name "$component" "$target" "$type" "tar.gz")
    tar -C "../../artifacts/$component/$type/$target/release" \
        -zcvf "$archive_name" "$distributable"
}

function create_zip_archive() {
    local archive_name
    local distributable
    local component
    local target
    local type
    component=$1
    target=$2
    type=$3

    distributable=$(get_distributable_for_component "$component" "$target")
    archive_name=$(get_archive_name "$component" "$target" "$type" "zip")
    zip -j "$archive_name" \
        "../../artifacts/$component/$type/$target/release/$distributable"
}

declare -a types=("mock" "real")
for type in "${types[@]}"; do
    targets=($(ls -1 "artifacts/$component/$type"))
    for target in "${targets[@]}"
    do
        (
            cd "deploy/$type"
            create_tar_archive "$component" "$target" "$type"
            create_zip_archive "$component" "$target" "$type"
        )
    done
done
