#!/usr/bin/env bash

GREEN="\e[32m"
RED="\e[31m"
YELLOW="\e[33m" END="\e[0m"

version=$1
if [[ -z "$version" ]]; then
    echo "You must supply a version number."
    exit 1
fi

repo=$2
if [[ -z "$repo" ]]; then
    repo="maidsafe/safe_network"
fi

function get_checksum_from_release_body() {
    local arch="$1"
    local type="$2"

    # 'A2' selects the 2 lines after e.g. "Linux" and 'm1' stops at the first match.
    # Stopping at the first match is required because of "ARM" and "ARMv7" in the release body.
    # xargs is just used to strip any whitespace from the checksum retrieved by awk.
    gh release view "$version" --repo "$repo" --json body \
        | jq --raw-output .body \
        | grep -A2 -m1 "$arch" \
        | grep "$type" \
        | awk -F ':' '{ print $2 }' \
        | xargs
}

function download_release_asset() {
    local file_name="$1"
    (
        cd /tmp
        if [[ ! -f "$file_name" ]]; then
            asset_url=$(gh release view "$version" --repo "$repo" --json assets \
                | jq --raw-output ".assets[] | select(.name == \"${file_name}\") | .url")
            curl -s -L -O "$asset_url"
        fi
    )
}

function test_release_body_contains_checksum() {
    local arch="$1"
    local type="$2"

    printf "The release body should contain the %s %s checksum..." "$arch" "$type"
    checksum=$(get_checksum_from_release_body "$arch" "$type")
    if [[ "$checksum" =~ ^[A-Fa-f0-9]{64}$ ]]; then
        printf "${GREEN}passed${END}\n"
    else
        printf "${RED}failed${END}\n"
        exit 1
    fi
}

function test_asset_checksum_matches_release_body_checksum() {
    local file_name="$1"
    local arch="$2"
    local type="$3"

    printf "The checksum for the %s %s should match the checksum specified on the release body..." $arch $type
    download_release_asset "$file_name"
    (
        cd /tmp
        expected_checksum=$(get_checksum_from_release_body "$arch" "$type")
        actual_checksum=$(sha256sum $file_name | awk '{print $1}')
        if [[ "$actual_checksum" == "$expected_checksum" ]]; then
            printf "${GREEN}passed${END}\n"
        else
            printf "${RED}failed${END}\n"
            printf "The checksum for the %s %s does not match the checksum specified on the release body\n" \
                "$arch" "$type"
            printf "Expected checksum as specified on the release: %s" $expected_checksum
            printf "Actual checksum on the file: %s" $actual_checksum
            exit 1
        fi
    )
}

function test_contents_of_archive_are_correct() {
    local file_name="$1"
    local arch="$2"
    local type="$3"
    local bin_name=""
    if [[ "$arch" == "Windows" ]]; then bin_name="sn_node.exe"; else bin_name="sn_node"; fi

    printf "The %s %s should have the correct contents..." "$arch" "$type"
    download_release_asset "$file_name"
    (
        cd /tmp
        archive_path=$(basename $file_name ".${type}")
        [[ -d "$archive_path" ]] && rm -rf $archive_path
        mkdir $archive_path
        if [[ "$type" == "tar.gz" ]]; then
            tar xvf "$file_name" -C "$archive_path" >/dev/null 2>&1
        else
            unzip -qq "$file_name" -d "$archive_path"
        fi
        count=$(ls $archive_path | wc -l)
        if [[ -f "$archive_path/$bin_name" && $count -eq 1 ]]; then
            printf "${GREEN}passed${END}\n"
        else
            printf "${RED}failed${END}\n"
            printf "The asset either has more files than expected or doesn't contain the correct file\n"
            exit 1
        fi
        rm -rf $archive_path
        rm $file_name
    )
}

function test_release_has_asset() {
    local file_name="$1"
    local arch="$2"
    local type="$3"

    printf "The release should contain the %s %s as an asset..." $arch $type
    readarray -t assets \
        < <(gh release view "$version" --repo "$repo" --json assets | jq --raw-output .assets[].name)
    if [[ " ${assets[@]} " =~ " ${file_name} " ]]; then
        printf "${GREEN}passed${END}\n"
    else
        printf "${RED}failed${END}\n"
        exit 1
    fi
}

function test_release_exists() {
    printf "The release should exist with the specified version number..."
    if gh release view "$version" --repo "$repo" >/dev/null 2>&1; then
        printf "${GREEN}passed${END}\n"
    else
        printf "${RED}failed${END}\n"
        printf "No release with %s exists on %s" $version $repo
        exit 1
    fi
}

function test_release_body_should_contain_linux_zip_checksum() {
    test_release_body_contains_checksum "Linux" "zip"
}

function test_release_body_should_contain_linux_tar_checksum() {
    test_release_body_contains_checksum "Linux" "tar.gz"
}

function test_release_body_should_contain_windows_zip_checksum() {
    test_release_body_contains_checksum "Windows" "zip"
}

function test_release_body_should_contain_windows_tar_checksum() {
    test_release_body_contains_checksum "Windows" "tar.gz"
}

function test_release_body_should_contain_macos_zip_checksum() {
    test_release_body_contains_checksum "macOS" "zip"
}

function test_release_body_should_contain_macos_tar_checksum() {
    test_release_body_contains_checksum "macOS" "tar.gz"
}

function test_release_body_should_contain_arm_zip_checksum() {
    test_release_body_contains_checksum "ARM" "zip"
}

function test_release_body_should_contain_arm_tar_checksum() {
    test_release_body_contains_checksum "ARM" "tar.gz"
}

function test_release_body_should_contain_armv7_zip_checksum() {
    test_release_body_contains_checksum "ARMv7" "zip"
}

function test_release_body_should_contain_armv7_tar_checksum() {
    test_release_body_contains_checksum "ARMv7" "tar.gz"
}

function test_release_body_should_contain_aarch64_zip_checksum() {
    test_release_body_contains_checksum "Aarch64" "zip"
}

function test_release_body_should_contain_aarch64_tar_checksum() {
    test_release_body_contains_checksum "Aarch64" "tar.gz"
}

function test_release_should_contain_linux_zip_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-x86_64-unknown-linux-musl.zip" "Linux" "zip"
}

function test_release_should_contain_linux_tar_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-x86_64-unknown-linux-musl.tar.gz" "Linux" "tar"
}

function test_release_should_contain_windows_zip_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-x86_64-pc-windows-msvc.zip" "Windows" "zip"
}

function test_release_should_contain_windows_tar_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-x86_64-pc-windows-msvc.tar.gz" "Windows" "tar"
}

function test_release_should_contain_macos_zip_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-x86_64-apple-darwin.zip" "macOS" "zip"
}

function test_release_should_contain_macos_tar_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-x86_64-apple-darwin.tar.gz" "macOS" "tar"
}

function test_release_should_contain_arm_zip_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-arm-unknown-linux-musleabi.zip" "ARM" "zip"
}

function test_release_should_contain_arm_tar_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-arm-unknown-linux-musleabi.tar.gz" "ARM" "tar.gz"
}

function test_release_should_contain_armv7_zip_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-armv7-unknown-linux-musleabihf.zip" "ARMv7" "zip"
}

function test_release_should_contain_armv7_tar_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-armv7-unknown-linux-musleabihf.tar.gz" "ARMv7" "tar.gz"
}

function test_release_should_contain_aarch64_zip_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-aarch64-unknown-linux-musl.zip" "Aarch64" "zip"
}

function test_release_should_contain_aarch64_tar_as_an_asset() {
    test_release_has_asset "sn_node-${version:1:6}-aarch64-unknown-linux-musl.tar.gz" "Aarch64" "tar.gz"
}

function test_linux_zip_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-x86_64-unknown-linux-musl.zip" "Linux" "zip"
}

function test_linux_tar_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-x86_64-unknown-linux-musl.tar.gz" "Linux" "tar.gz"
}

function test_windows_zip_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-x86_64-pc-windows-msvc.zip" "Windows" "zip"
}

function test_windows_tar_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-x86_64-pc-windows-msvc.tar.gz" "Windows" "tar.gz"
}

function test_macos_zip_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-x86_64-apple-darwin.zip" "macOS" "zip"
}

function test_macos_tar_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-x86_64-apple-darwin.tar.gz" "macOS" "tar.gz"
}

function test_arm_zip_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-arm-unknown-linux-musleabi.zip" "ARM" "zip"
}

function test_arm_tar_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-arm-unknown-linux-musleabi.tar.gz" "ARM" "tar.gz"
}

function test_armv7_zip_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-armv7-unknown-linux-musleabihf.zip" "ARMv7" "zip"
}

function test_armv7_tar_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-armv7-unknown-linux-musleabihf.tar.gz" "ARMv7" "tar.gz"
}

function test_aarch64_zip_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-aarch64-unknown-linux-musl.zip" "Aarch64" "zip"
}

function test_aarch64_tar_should_match_the_checksum_on_the_release_body() {
    test_asset_checksum_matches_release_body_checksum \
        "sn_node-${version:1:6}-aarch64-unknown-linux-musl.tar.gz" "Aarch64" "tar"
}

function test_linux_zip_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-x86_64-unknown-linux-musl.zip" "Linux" "zip"
}

function test_linux_tar_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-x86_64-unknown-linux-musl.tar.gz" "Linux" "tar.gz"
}

function test_windows_zip_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-x86_64-pc-windows-msvc.zip" "Windows" "zip"
}

function test_windows_tar_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-x86_64-pc-windows-msvc.tar.gz" "Windows" "tar.gz"
}

function test_macos_zip_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-x86_64-apple-darwin.zip" "macOS" "zip"
}

function test_macos_tar_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-x86_64-apple-darwin.tar.gz" "macOS" "tar.gz"
}

function test_arm_zip_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-arm-unknown-linux-musleabi.zip" "ARM" "zip"
}

function test_arm_tar_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-arm-unknown-linux-musleabi.tar.gz" "ARM" "tar.gz"
}

function test_armv7_zip_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-armv7-unknown-linux-musleabihf.zip" "ARMv7" "zip"
}

function test_armv7_tar_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-armv7-unknown-linux-musleabihf.tar.gz" "ARMv7" "tar.gz"
}

function test_aarch64_zip_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-aarch64-unknown-linux-musl.zip" "Aarch64" "zip"
}

function test_aarch64_tar_archive_contents_are_correct() {
    test_contents_of_archive_are_correct \
        "sn_node-${version:1:6}-aarch64-unknown-linux-musl.tar.gz" "Aarch64" "tar.gz"
}

test_release_exists
test_release_body_should_contain_linux_zip_checksum
test_release_body_should_contain_linux_tar_checksum
test_release_body_should_contain_windows_zip_checksum
test_release_body_should_contain_windows_tar_checksum
test_release_body_should_contain_macos_zip_checksum
test_release_body_should_contain_macos_tar_checksum
test_release_body_should_contain_arm_zip_checksum
test_release_body_should_contain_arm_tar_checksum
test_release_body_should_contain_armv7_zip_checksum
test_release_body_should_contain_armv7_tar_checksum
test_release_body_should_contain_aarch64_zip_checksum
test_release_body_should_contain_aarch64_tar_checksum
test_release_should_contain_linux_zip_as_an_asset
test_release_should_contain_linux_tar_as_an_asset
test_release_should_contain_windows_zip_as_an_asset
test_release_should_contain_windows_tar_as_an_asset
test_release_should_contain_macos_zip_as_an_asset
test_release_should_contain_macos_tar_as_an_asset
test_release_should_contain_arm_zip_as_an_asset
test_release_should_contain_arm_tar_as_an_asset
test_release_should_contain_armv7_zip_as_an_asset
test_release_should_contain_armv7_tar_as_an_asset
test_release_should_contain_aarch64_tar_as_an_asset
test_release_should_contain_aarch64_zip_as_an_asset
test_linux_zip_should_match_the_checksum_on_the_release_body
test_linux_tar_should_match_the_checksum_on_the_release_body
test_windows_zip_should_match_the_checksum_on_the_release_body
test_windows_tar_should_match_the_checksum_on_the_release_body
test_macos_zip_should_match_the_checksum_on_the_release_body
test_macos_tar_should_match_the_checksum_on_the_release_body
test_arm_zip_should_match_the_checksum_on_the_release_body
test_arm_tar_should_match_the_checksum_on_the_release_body
test_armv7_zip_should_match_the_checksum_on_the_release_body
test_armv7_tar_should_match_the_checksum_on_the_release_body
test_aarch64_zip_should_match_the_checksum_on_the_release_body
test_aarch64_tar_should_match_the_checksum_on_the_release_body
test_linux_zip_archive_contents_are_correct
test_linux_tar_archive_contents_are_correct
test_windows_zip_archive_contents_are_correct
test_windows_tar_archive_contents_are_correct
test_macos_zip_archive_contents_are_correct
test_macos_tar_archive_contents_are_correct
test_arm_zip_archive_contents_are_correct
test_arm_tar_archive_contents_are_correct
test_armv7_zip_archive_contents_are_correct
test_armv7_tar_archive_contents_are_correct
test_aarch64_zip_archive_contents_are_correct
test_aarch64_tar_archive_contents_are_correct
