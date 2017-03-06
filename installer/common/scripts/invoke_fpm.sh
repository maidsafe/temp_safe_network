#!/bin/bash
#
# Create a package for Core Release binaries

# Stop the script if any command fails
set -o errtrace
trap 'exit' ERR

# Get current version and executable's name from Cargo.toml
RootDir=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
Version=$(sed -n 's/[ \t]*version[ \t]*=[ \t]*"\([^"]*\)".*/\1/p' "$RootDir/Cargo.toml")
CoreName=$(sed -n 's/[ \t]*name[ \t]*=[ \t]*"\([^"]*\)".*/\1/p' "$RootDir/Cargo.toml")
CorePath=/usr/bin/
ConfigFilePath=$HOME/.config/$CoreName/
Platform=$1
Description="SAFE Network client"

function create_package {
  fpm \
    -t $1 \
    -s dir \
    --force \
    --name $PackageName \
    --version $Version \
    --license GPLv3 \
    --vendor MaidSafe \
    --directories $ConfigFilePath \
    --maintainer "MaidSafeQA <qa@maidsafe.net>" \
    --description "$Description" \
    --url "https://maidsafe.net" \
    "$RootDir/target/release/examples/$CoreName"=$CorePath \
    "$RootDir/installer/common/$ClientName.crust.config"=$ConfigFilePath
}

# Build the targets
cd "$RootDir"
cargo update
cargo build --release --example $CoreName
strip "$RootDir/target/release/examples/$CoreName"

# Prepare to create packages
rm -rf "$RootDir/packages/$Platform" || true
mkdir -p "$RootDir/packages/$Platform"
cd "$RootDir/packages/$Platform"

# Create tarball
Bits=$(getconf LONG_BIT)
PackageName="$CoreName"_"$Version"_"$Bits"-bit
create_package tar
gzip $PackageName.tar

# Create platform-specific packages
PackageName=$CoreName
if [[ "$1" == "linux" ]]
then
  create_package deb
  create_package rpm
elif [[ "$1" == "osx" ]]
then
  create_package osxpkg
fi
