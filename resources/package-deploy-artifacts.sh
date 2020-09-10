#!/usr/bin/env bash

set -e

component=$1
if [[ -z "$component" ]]; then
    echo "You must supply the component to build."
    echo "Valid values are 'sn_cli', 'sn_api', 'sn_authd' or 'sn_ffi'."
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
        sn_authd)
            if [[ "$target" == *"windows"* ]]; then
                distributable="sn_authd.exe"
            else
                distributable="sn_authd"
            fi
            ;;
        sn_cli)
            if [[ "$target" == *"windows"* ]]; then
                distributable="safe.exe"
            else
                distributable="safe"
            fi
            ;;
        sn_ffi)
            if [[ "$target" == *"darwin"* ]]; then
                distributable="libsn_ffi.dylib"
            elif [[ "$target" == *"windows"* ]]; then
                distributable="sn_ffi.dll"
            elif [[ "$target" == *"linux"* ]]; then
                distributable="libsn_ffi.so"
            else
                distributable="libsn_ffi.a"
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
    [[ "$type" == "dev" ]] && archive_name="$archive_name-dev"
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

    echo $archive_name;
    ls "../../artifacts/$component"
    ls "../../artifacts/$component/$type/$target"
    ls "../../artifacts/$component/$type/$target/release"

    zip -j "$archive_name" \
        "../../artifacts/$component/$type/$target/release/$distributable"
}

declare -a types=("prod" "dev")
for type in "${types[@]}"; do
    if [[ "$component" == 'sn_authd' ]] && [[ "$type" == 'dev' ]]; then
      continue
    fi
    if [[ ( "$component" = 'sn_cli' ) && ( "$type" = 'dev' ) ]]; then
      continue
    fi
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
