#!/usr/bin/env bash

api_version=$1
if [[ -z "$api_version" ]]; then
    echo "You must supply a version number for sn_api."
    exit 1
fi
cli_version=$2
if [[ -z "$cli_version" ]]; then
    echo "You must supply a version number for sn_cli."
    exit 1
fi

authd_version=$3
if [[ -z "$authd_version" ]]; then
    echo "You must supply a version number for sn_authd."
    exit 1
fi

# The single quotes around EOF is to stop attempted variable and backtick expansion.
read -r -d '' release_description << 'EOF'
Command line interface for the Safe Network. Refer to [Safe CLI User Guide](https://github.com/maidsafe/sn_api/blob/master/sn_cli/README.md) for detailed instructions.

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

### Safe Authenticator daemon

The Authenticator daemon exposes services which allow applications and users to create a Safe, unlock it using its credentials (passphrase and password), authorise applications which need to store data on the network on behalf of the user, as well as revoke permissions previously granted to applications.
The Safe Authenticator, which runs as a daemon or as a service in Windows platforms, can be started and managed with the Safe CLI if the `sn_authd`/`sn_authd.exe` binary is properly installed in the system with execution permissions. Please refer to [Authenticator section in CLI User Guide](https://github.com/maidsafe/sn_api/blob/master/sn_cli/README.md#the-authenticator-daemon-authd) for detailed instructions.

| OS | Download link | SHA-256 checksum |
| --- | --- | --- |
| Linux | [Download](S3_AUTHD_LINUX_DEPLOY_URL) | ZIP_AUTHD_LINUX_CHECKSUM |
| macOS | [Download](S3_AUTHD_MACOS_DEPLOY_URL) | ZIP_AUTHD_MACOS_CHECKSUM |
| Windows | [Download](S3_AUTHD_WIN_DEPLOY_URL) | ZIP_AUTHD_WIN_CHECKSUM |



## Related Links
* [Safe CLI User Guide](https://github.com/maidsafe/sn_api/blob/master/sn_cli/README.md)
* [Safe Network Browser](https://github.com/maidsafe/sn_browser/releases/)
* [Safe Network Node](https://github.com/maidsafe/sn_node/releases/latest/)
EOF

s3_authd_linux_deploy_url="https:\/\/sn-api.s3.amazonaws.com\/sn_authd-$authd_version-x86_64-unknown-linux-gnu.zip"
s3_authd_win_deploy_url="https:\/\/sn-api.s3.amazonaws.com\/sn_authd-$authd_version-x86_64-pc-windows-msvc.zip"
s3_authd_macos_deploy_url="https:\/\/sn-api.s3.amazonaws.com\/sn_authd-$authd_version-x86_64-apple-darwin.zip"

zip_linux_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$cli_version-x86_64-unknown-linux-gnu.zip" | \
    awk '{ print $1 }')
zip_macos_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$cli_version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
zip_win_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$cli_version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')
tar_linux_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$cli_version-x86_64-unknown-linux-gnu.tar.gz" | \
    awk '{ print $1 }')
tar_macos_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$cli_version-x86_64-apple-darwin.tar.gz" | \
    awk '{ print $1 }')
tar_win_checksum=$(sha256sum \
    "./deploy/prod/sn_cli-$cli_version-x86_64-pc-windows-msvc.tar.gz" | \
    awk '{ print $1 }')

zip_authd_linux_checksum=$(sha256sum \
    "./deploy/prod/sn_authd-$authd_version-x86_64-unknown-linux-gnu.zip" | \
    awk '{ print $1 }')
zip_authd_macos_checksum=$(sha256sum \
    "./deploy/prod/sn_authd-$authd_version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
zip_authd_win_checksum=$(sha256sum \
    "./deploy/prod/sn_authd-$authd_version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')

release_description=$(sed "s/S3_AUTHD_LINUX_DEPLOY_URL/$s3_authd_linux_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_AUTHD_MACOS_DEPLOY_URL/$s3_authd_macos_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_AUTHD_WIN_DEPLOY_URL/$s3_authd_win_deploy_url/g" <<< "$release_description")

release_description=$(sed "s/ZIP_LINUX_CHECKSUM/$zip_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/ZIP_MACOS_CHECKSUM/$zip_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/ZIP_WIN_CHECKSUM/$zip_win_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_LINUX_CHECKSUM/$tar_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_MACOS_CHECKSUM/$tar_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_WIN_CHECKSUM/$tar_win_checksum/g" <<< "$release_description")

release_description=$(sed "s/ZIP_AUTHD_LINUX_CHECKSUM/$zip_authd_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/ZIP_AUTHD_MACOS_CHECKSUM/$zip_authd_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/ZIP_AUTHD_WIN_CHECKSUM/$zip_authd_win_checksum/g" <<< "$release_description")

echo "$release_description"
