#!/usr/bin/env bash

version=$1
if [[ -z "$version" ]]; then
    echo "You must supply a version number for sn_cli."
    exit 1
fi

# The single quotes around EOF is to stop attempted variable and backtick expansion.
read -r -d '' release_description << 'EOF'
Command line interface for the Safe Network. Refer to [Safe CLI User Guide](https://github.com/maidsafe/sn_cli/blob/master/README.md) for detailed instructions.

## SHA-256 checksums for CLI binaries:
```
Linux
zip: ZIP_LINUX_CHECKSUM
tar.gz: TAR_LINUX_CHECKSUM

macOS
zip: ZIP_MACOS_CHECKSUM
tar.gz: TAR_MACOS_CHECKSUM

Windows
zip: ZIP_WIN_CHECKSUM
tar.gz: TAR_WIN_CHECKSUM

ARM
zip: ZIP_ARM_CHECKSUM
tar.gz: TAR_ARM_CHECKSUM

ARMv7
zip: ZIP_ARMv7_CHECKSUM
tar.gz: TAR_ARMv7_CHECKSUM

Aarch64
zip: ZIP_AARCH64_CHECKSUM
tar.gz: TAR_AARCH64_CHECKSUM
```

## Related Links
* [Safe CLI User Guide](https://github.com/maidsafe/sn_cli/blob/master/README.md)
* [Safe Network Browser](https://github.com/maidsafe/sn_browser/releases/)
* [Safe Network Node](https://github.com/maidsafe/sn_node/releases/latest/)
EOF

zip_linux_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-x86_64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
zip_macos_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
zip_win_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')
zip_arm_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-arm-unknown-linux-musleabi.zip" | \
    awk '{ print $1 }')
zip_armv7_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-armv7-unknown-linux-musleabihf.zip" | \
    awk '{ print $1 }')
zip_aarch64_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-aarch64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
tar_linux_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-x86_64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')
tar_macos_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-x86_64-apple-darwin.tar.gz" | \
    awk '{ print $1 }')
tar_win_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-x86_64-pc-windows-msvc.tar.gz" | \
    awk '{ print $1 }')
tar_arm_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-arm-unknown-linux-musleabi.tar.gz" | \
    awk '{ print $1 }')
tar_armv7_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-armv7-unknown-linux-musleabihf.tar.gz" | \
    awk '{ print $1 }')
tar_aarch64_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$version-aarch64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')

release_description=$(sed "s/ZIP_LINUX_CHECKSUM/$zip_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/ZIP_MACOS_CHECKSUM/$zip_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/ZIP_WIN_CHECKSUM/$zip_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=ZIP_ARM_CHECKSUM=$zip_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=ZIP_ARMv7_CHECKSUM=$zip_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=ZIP_AARCH64_CHECKSUM=$zip_aarch64_checksum=g" <<< "$release_description")
release_description=$(sed "s/TAR_LINUX_CHECKSUM/$tar_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_MACOS_CHECKSUM/$tar_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_WIN_CHECKSUM/$tar_win_checksum/g" <<< "$release_description")
release_description=$(sed "s=TAR_ARM_CHECKSUM=$tar_arm_checksum=g" <<< "$release_description")
release_description=$(sed "s=TAR_ARMv7_CHECKSUM=$tar_armv7_checksum=g" <<< "$release_description")
release_description=$(sed "s=TAR_AARCH64_CHECKSUM=$tar_aarch64_checksum=g" <<< "$release_description")

echo "$release_description"
