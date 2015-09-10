#!/bin/bash
#
# Create a package for Client Release binaries

# Stop the script if any command fails
set -o errtrace
trap 'exit' ERR

# Get current version and executable's name from Cargo.toml
RootDir=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)
Version=$(sed -n 's/[ \t]*version[ \t]*=[ \t]*"\([^"]*\)".*/\1/p' "$RootDir/Cargo.toml")
ClientName=$(sed -n 's/[ \t]*name[ \t]*=[ \t]*"\([^"]*\)".*/\1/p' "$RootDir/Cargo.toml")
ClientPath=/usr/bin/
ConfigFilePath=$HOME/.config/$ClientName/
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
    --url "http://maidsafe.net" \
    "$RootDir/target/release/examples/$ClientName"=$ClientPath \
    "$RootDir/installer/common/$ClientName.crust.config"=$ConfigFilePath
}

# Build the targets
cd "$RootDir"
cargo update
cargo build --release --example $ClientName
strip "$RootDir/target/release/examples/$ClientName"

# Prepare to create packages
rm -rf "$RootDir/packages/$Platform" || true
mkdir -p "$RootDir/packages/$Platform"
cd "$RootDir/packages/$Platform"

# Create tarball
Bits=$(getconf LONG_BIT)
PackageName="$ClientName"_"$Version"_"$Bits"-bit
create_package tar
gzip $PackageName.tar

# Create platform-specific packages
PackageName=$ClientName
if [[ "$1" == "linux" ]]
then
  create_package deb
  create_package rpm
elif [[ "$1" == "osx" ]]
then
  create_package osxpkg
fi
