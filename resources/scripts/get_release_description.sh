#!/usr/bin/env bash

sn_dysfunction_version=$( \
  grep "^version" < sn_dysfunction/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_interface_version=$( \
  grep "^version" < sn_interface/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_client_version=$( \
  grep "^version" < sn_client/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_node_version=$(grep "^version" < sn_node/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_api_version=$(grep "^version" < sn_api/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_cli_version=$(grep "^version" < sn_cli/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')

# The single quotes around EOF is to stop attempted variable and backtick expansion.
read -r -d '' release_description << 'EOF'
This release of Safe Network consists of:
* Safe Node Dysfunction v__SN_DYSFUNCTION_VERSION__
* Safe Network Interface v__SN_INTERFACE_VERSION__
* Safe Client v__SN_CLIENT_VERSION__
* Safe Node v__SN_NODE_VERSION__
* Safe API v__SN_API_VERSION__
* Safe CLI v__SN_CLI_VERSION__

## Safe Network Interface Changelog

__SN_INTERFACE_CHANGELOG_TEXT__

## Safe Node Dysfunction Changelog

__SN_DYSFUNCTION_CHANGELOG_TEXT__

## Safe Node Changelog

__SN_NODE_CHANGELOG_TEXT__

## Safe Client Changelog

__SN_CLIENT_CHANGELOG_TEXT__

## Safe API Changelog

__SN_API_CHANGELOG_TEXT__

## Safe CLI Changelog

__SN_CLI_CHANGELOG_TEXT__

## SHA-256 checksums for sn_node binaries:
```
Linux
zip: SN_ZIP_LINUX_CHECKSUM
tar.gz: SN_TAR_LINUX_CHECKSUM

macOS
zip: SN_ZIP_MACOS_CHECKSUM
tar.gz: SN_TAR_MACOS_CHECKSUM

Windows
zip: SN_ZIP_WIN_CHECKSUM
tar.gz: SN_TAR_WIN_CHECKSUM

ARM
zip: SN_ZIP_ARM_CHECKSUM
tar.gz: SN_TAR_ARM_CHECKSUM

ARMv7
zip: SN_ZIP_ARMv7_CHECKSUM
tar.gz: SN_TAR_ARMv7_CHECKSUM

Aarch64
zip: SN_ZIP_AARCH64_CHECKSUM
tar.gz: SN_TAR_AARCH64_CHECKSUM
```

## SHA-256 checksums for safe binaries:
```
Linux
zip: SN_CLI_ZIP_LINUX_CHECKSUM
tar.gz: SN_CLI_TAR_LINUX_CHECKSUM

macOS
zip: SN_CLI_ZIP_MACOS_CHECKSUM
tar.gz: SN_CLI_TAR_MACOS_CHECKSUM

Windows
zip: SN_CLI_ZIP_WIN_CHECKSUM
tar.gz: SN_CLI_TAR_WIN_CHECKSUM

ARM
zip: SN_CLI_ZIP_ARM_CHECKSUM
tar.gz: SN_CLI_TAR_ARM_CHECKSUM

ARMv7
zip: SN_CLI_ZIP_ARMv7_CHECKSUM
tar.gz: SN_CLI_TAR_ARMv7_CHECKSUM

Aarch64
zip: SN_CLI_ZIP_AARCH64_CHECKSUM
tar.gz: SN_CLI_TAR_AARCH64_CHECKSUM
```
EOF

sn_zip_linux_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-x86_64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
sn_zip_macos_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
sn_zip_win_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')
sn_zip_arm_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-arm-unknown-linux-musleabi.zip" | \
    awk '{ print $1 }')
sn_zip_armv7_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-armv7-unknown-linux-musleabihf.zip" | \
    awk '{ print $1 }')
sn_zip_aarch64_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-aarch64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
sn_tar_linux_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-x86_64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')
sn_tar_macos_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-x86_64-apple-darwin.tar.gz" | \
    awk '{ print $1 }')
sn_tar_win_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-x86_64-pc-windows-msvc.tar.gz" | \
    awk '{ print $1 }')
sn_tar_arm_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-arm-unknown-linux-musleabi.tar.gz" | \
    awk '{ print $1 }')
sn_tar_armv7_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-armv7-unknown-linux-musleabihf.tar.gz" | \
    awk '{ print $1 }')
sn_tar_aarch64_checksum=$(sha256sum \
    "./deploy/prod/sn_node/sn_node-$sn_node_version-aarch64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')

sn_cli_zip_linux_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-x86_64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
sn_cli_zip_macos_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
sn_cli_zip_win_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')
sn_cli_zip_arm_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-arm-unknown-linux-musleabi.zip" | \
    awk '{ print $1 }')
sn_cli_zip_armv7_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-armv7-unknown-linux-musleabihf.zip" | \
    awk '{ print $1 }')
sn_cli_zip_aarch64_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-aarch64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
sn_cli_tar_linux_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-x86_64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')
sn_cli_tar_macos_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-x86_64-apple-darwin.tar.gz" | \
    awk '{ print $1 }')
sn_cli_tar_win_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-x86_64-pc-windows-msvc.tar.gz" | \
    awk '{ print $1 }')
sn_cli_tar_arm_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-arm-unknown-linux-musleabi.tar.gz" | \
    awk '{ print $1 }')
sn_cli_tar_armv7_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-armv7-unknown-linux-musleabihf.tar.gz" | \
    awk '{ print $1 }')
sn_cli_tar_aarch64_checksum=$(sha256sum \
    "./deploy/prod/safe/sn_cli-$sn_cli_version-aarch64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')

release_description=$(sed "s/__SN_DYSFUNCTION_VERSION__/$sn_dysfunction_version/g" <<< "$release_description")
release_description=$(sed "s/__SN_INTERFACE_VERSION__/$sn_interface_version/g" <<< "$release_description")
release_description=$(sed "s/__SN_CLIENT_VERSION__/$sn_client_version/g" <<< "$release_description")
release_description=$(sed "s/__SN_NODE_VERSION__/$sn_node_version/g" <<< "$release_description")
release_description=$(sed "s/__SN_API_VERSION__/$sn_api_version/g" <<< "$release_description")
release_description=$(sed "s/__SN_CLI_VERSION__/$sn_cli_version/g" <<< "$release_description")

release_description=$(sed "s/SN_ZIP_LINUX_CHECKSUM/$sn_zip_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/SN_ZIP_MACOS_CHECKSUM/$sn_zip_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/SN_ZIP_WIN_CHECKSUM/$sn_zip_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=SN_ZIP_ARM_CHECKSUM=$sn_zip_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=SN_ZIP_ARMv7_CHECKSUM=$sn_zip_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=SN_ZIP_AARCH64_CHECKSUM=$sn_zip_aarch64_checksum=g" <<< "$release_description")
release_description=$(sed "s/SN_TAR_LINUX_CHECKSUM/$sn_tar_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/SN_TAR_MACOS_CHECKSUM/$sn_tar_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/SN_TAR_WIN_CHECKSUM/$sn_tar_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=SN_TAR_ARM_CHECKSUM=$sn_tar_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=SN_TAR_ARMv7_CHECKSUM=$sn_tar_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=SN_TAR_AARCH64_CHECKSUM=$sn_tar_aarch64_checksum=g" <<< "$release_description")

release_description=$(sed "s/SN_CLI_ZIP_LINUX_CHECKSUM/$sn_cli_zip_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/SN_CLI_ZIP_MACOS_CHECKSUM/$sn_cli_zip_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/SN_CLI_ZIP_WIN_CHECKSUM/$sn_cli_zip_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=SN_CLI_ZIP_ARM_CHECKSUM=$sn_cli_zip_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=SN_CLI_ZIP_ARMv7_CHECKSUM=$sn_cli_zip_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=SN_CLI_ZIP_AARCH64_CHECKSUM=$sn_cli_zip_aarch64_checksum=g" <<< "$release_description")
release_description=$(sed "s/SN_CLI_TAR_LINUX_CHECKSUM/$sn_cli_tar_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/SN_CLI_TAR_MACOS_CHECKSUM/$sn_cli_tar_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/SN_CLI_TAR_WIN_CHECKSUM/$sn_cli_tar_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=SN_CLI_TAR_ARM_CHECKSUM=$sn_cli_tar_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=SN_CLI_TAR_ARMv7_CHECKSUM=$sn_cli_tar_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=SN_CLI_TAR_AARCH64_CHECKSUM=$sn_cli_tar_aarch64_checksum=g" <<< "$release_description")

echo "$release_description"
