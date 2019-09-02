#!/usr/bin/env bash

version=$1
if [[ -z "$version" ]]; then
    echo "You must supply a version number."
    exit 1
fi

# The single quotes around EOF is to stop attempted variable and backtick expansion.
read -r -d '' release_description << 'EOF'
Command line interface for the SAFE Network.

There are also development versions of this release:
[Linux](S3_LINUX_DEPLOY_URL)
[macOS](S3_MACOS_DEPLOY_URL)
[Windows](S3_WIN_DEPLOY_URL)

The development version uses a mocked SAFE network, which allows you to work against a file that mimics the network, where SafeCoins are created for local use.

## Bash Autocompletion

If you are using Bash on Linux, you can get auto completion for `safe` by doing the following:
```
curl -L SAFE_COMPLETION_URL > ~/.safe_completion
echo "source ~/.safe_completion" >> ~/.bashrc
```

## SHA-256 checksums for release versions:
```
Linux
LINUX_CHECKSUM

macOS
MACOS_CHECKSUM

Windows
WIN_CHECKSUM
```

## Related Links
* [SAFE Authenticator CLI](https://github.com/maidsafe/safe-authenticator-cli/releases/latest/)
* [SAFE Browser PoC](https://github.com/maidsafe/safe_browser/releases/)
* [SAFE vault](https://github.com/maidsafe/safe_vault/releases/latest/)
EOF

safe_completion_url="https:\/\/github.com\/maidsafe\/safe-cli\/releases\/download\/$version\/safe_completion.sh"
s3_linux_deploy_url="https:\/\/safe-cli.s3.amazonaws.com\/safe_cli-$version-x86_64-unknown-linux-gnu-dev.zip"
s3_win_deploy_url="https:\/\/safe-cli.s3.amazonaws.com\/safe_cli-$version-x86_64-pc-windows-gnu-dev.zip"
s3_macos_deploy_url="https:\/\/safe-cli.s3.amazonaws.com\/safe_cli-$version-x86_64-apple-darwin-dev.zip"
linux_checksum=$(sha256sum \
    "./deploy/release/safe_cli-$version-x86_64-unknown-linux-gnu.zip" | \
    awk '{ print $1 }')
macos_checksum=$(sha256sum \
    "./deploy/release/safe_cli-$version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
win_checksum=$(sha256sum \
    "./deploy/release/safe_cli-$version-x86_64-pc-windows-gnu.zip" | \
    awk '{ print $1 }')

release_description=$(sed "s/S3_LINUX_DEPLOY_URL/$s3_linux_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_MACOS_DEPLOY_URL/$s3_macos_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/S3_WIN_DEPLOY_URL/$s3_win_deploy_url/g" <<< "$release_description")
release_description=$(sed "s/SAFE_COMPLETION_URL/$safe_completion_url/g" <<< "$release_description")
release_description=$(sed "s/LINUX_CHECKSUM/$linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/MACOS_CHECKSUM/$macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/WIN_CHECKSUM/$win_checksum/g" <<< "$release_description")
echo "$release_description"
