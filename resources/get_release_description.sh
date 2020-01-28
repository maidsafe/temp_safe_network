#!/usr/bin/env bash

api_version=$1
if [[ -z "$api_version" ]]; then
    echo "You must supply a version number for safe-api."
    exit 1
fi
cli_version=$2
if [[ -z "$cli_version" ]]; then
    echo "You must supply a version number for safe-cli."
    exit 1
fi
ffi_version=$3
if [[ -z "$ffi_version" ]]; then
    echo "You must supply a version number for safe-ffi."
    exit 1
fi
authd_version=$4
if [[ -z "$authd_version" ]]; then
    echo "You must supply a version number for safe-authd."
    exit 1
fi

# The single quotes around EOF is to stop attempted variable and backtick expansion.
read -r -d '' release_description << 'EOF'
Command line interface for the SAFE Network. Refer to [SAFE CLI User Guide](https://github.com/maidsafe/safe-api/blob/master/safe-cli/README.md) for detailed instructions.

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
```

### SAFE Authenticator daemon

The Authenticator daemon exposes services which allow applications and users to create SAFE Network accounts, log in using an existing account's credentials (passphrase and password), authorise applications which need to store data on the network on behalf of the user, as well as revoke permissions previously granted to applications.
The SAFE Authenticator, which runs as a daemon or as a service in Windows platforms, can be started and managed with the SAFE CLI if the `safe-authd`/`safe-authd.exe` binary is properly installed in the system with execution permissions. Please refer to [Authenticator section in CLI User Guide](https://github.com/maidsafe/safe-api/blob/master/safe-cli/README.md#the-authenticator-daemon-authd) for detailed instructions.

| OS | Download link | SHA-256 checksum |
| --- | --- | --- |
| Linux | [Download](S3_AUTHD_LINUX_DEPLOY_URL) | ZIP_LINUX_CHECKSUM_AUTHD |
| macOS | [Download](S3_AUTHD_MACOS_DEPLOY_URL) | ZIP_MACOS_CHECKSUM_AUTHD |
| Windows | [Download](S3_AUTHD_WIN_DEPLOY_URL) | ZIP_WIN_CHECKSUM_AUTHD |

### FFI

FFI is used to generate the native libraries which can be used by other high level languages to consume the Rust API. The development versions use a mocked version of the SAFE Network.

| OS | Download link |
| --- | --- |
| Linux | [Download](S3_FFI_LINUX_DEPLOY_URL) |
| macOS | [Download](S3_FFI_MACOS_DEPLOY_URL) |
| Windows | [Download](S3_FFI_WIN_DEPLOY_URL) |
| iOS | [Download](S3_FFI_IOS_DEPLOY_URL) |
| Android ARMv7 | [Download](S3_FFI_ANDROID_ARMV7_DEPLOY_URL) |
| Android x86_64 | [Download](S3_FFI_ANDROID_X86_64_DEPLOY_URL) |

## Related Links
* [SAFE CLI User Guide](https://github.com/maidsafe/safe-api/blob/master/safe-cli/README.md)
* [SAFE Browser](https://github.com/maidsafe/safe_browser/releases/)
* [SAFE Vault](https://github.com/maidsafe/safe_vault/releases/latest/)
EOF

s3_authd_linux_deploy_url="https:\/\/safe-api.s3.amazonaws.com\/safe-authd-$authd_version-x86_64-unknown-linux-gnu.zip"
s3_authd_win_deploy_url="https:\/\/safe-api.s3.amazonaws.com\/safe-authd-$authd_version-x86_64-pc-windows-msvc.zip"
s3_authd_macos_deploy_url="https:\/\/safe-api.s3.amazonaws.com\/safe-authd-$authd_version-x86_64-apple-darwin.zip"

zip_linux_checksum=$(sha256sum \
    "./deploy/prod/safe-cli-$cli_version-x86_64-unknown-linux-gnu.zip" | \
    awk '{ print $1 }')
zip_macos_checksum=$(sha256sum \
    "./deploy/prod/safe-cli-$cli_version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
zip_win_checksum=$(sha256sum \
    "./deploy/prod/safe-cli-$cli_version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')
tar_linux_checksum=$(sha256sum \
    "./deploy/prod/safe-cli-$cli_version-x86_64-unknown-linux-gnu.tar.gz" | \
    awk '{ print $1 }')
tar_macos_checksum=$(sha256sum \
    "./deploy/prod/safe-cli-$cli_version-x86_64-apple-darwin.tar.gz" | \
    awk '{ print $1 }')
tar_win_checksum=$(sha256sum \
    "./deploy/prod/safe-cli-$cli_version-x86_64-pc-windows-msvc.tar.gz" | \
    awk '{ print $1 }')

zip_linux_checksum_authd=$(sha256sum \
    "./deploy/prod/safe-authd-$authd_version-x86_64-unknown-linux-gnu.zip" | \
    awk '{ print $1 }')
zip_macos_checksum_authd=$(sha256sum \
    "./deploy/prod/safe-authd-$authd_version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
zip_win_checksum_authd=$(sha256sum \
    "./deploy/prod/safe-authd-$authd_version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')

s3_ffi_linux_deploy_url="https:\/\/safe-api.s3.amazonaws.com\/safe-ffi-$ffi_version-x86_64-unknown-linux-gnu.zip"
s3_ffi_win_deploy_url="https:\/\/safe-api.s3.amazonaws.com\/safe-ffi-$ffi_version-x86_64-pc-windows-msvc.zip"
s3_ffi_macos_deploy_url="https:\/\/safe-api.s3.amazonaws.com\/safe-ffi-$ffi_version-x86_64-apple-darwin.zip"
s3_ffi_android_x86_64_deploy_url="https:\/\/safe-api.s3.amazonaws.com\/safe-ffi-$ffi_version-x86_64-linux-android.zip"
s3_ffi_android_armv7_deploy_url="https:\/\/safe-api.s3.amazonaws.com\/safe-ffi-$ffi_version-armv7-linux-androideabi.zip"
s3_ffi_ios_deploy_url="https:\/\/safe-api.s3.amazonaws.com\/safe-ffi-$ffi_version-apple-ios.zip"

release_description=$(sed "s/S3_AUTHD_LINUX_DEPLOY_URL/$s3_authd_linux_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_AUTHD_MACOS_DEPLOY_URL/$s3_authd_macos_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_AUTHD_WIN_DEPLOY_URL/$s3_authd_win_deploy_url/g" <<< "$release_description")

release_description=$(sed "s/ZIP_LINUX_CHECKSUM/$zip_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/ZIP_MACOS_CHECKSUM/$zip_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/ZIP_WIN_CHECKSUM/$zip_win_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_LINUX_CHECKSUM/$tar_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_MACOS_CHECKSUM/$tar_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_WIN_CHECKSUM/$tar_win_checksum/g" <<< "$release_description")

release_description=$(sed "s/ZIP_LINUX_CHECKSUM_AUTHD/$zip_linux_checksum_authd/g" <<< "$release_description")
release_description=$(sed "s/ZIP_MACOS_CHECKSUM_AUTHD/$zip_macos_checksum_authd/g" <<< "$release_description")
release_description=$(sed "s/ZIP_WIN_CHECKSUM_AUTHD/$zip_win_checksum_authd/g" <<< "$release_description")

release_description=$(sed "s/S3_FFI_LINUX_DEPLOY_URL/$s3_ffi_linux_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_FFI_WIN_DEPLOY_URL/$s3_ffi_win_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_FFI_MACOS_DEPLOY_URL/$s3_ffi_macos_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_FFI_IOS_DEPLOY_URL/$s3_ffi_ios_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_FFI_ANDROID_ARMV7_DEPLOY_URL/$s3_ffi_android_armv7_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_FFI_ANDROID_X86_64_DEPLOY_URL/$s3_ffi_android_x86_64_deploy_url/g" <<< "$release_description")

echo "$release_description"
