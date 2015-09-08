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
UserGroup=$(id -gn)
ClientPath=/usr/bin/
ConfigFilePath=/var/cache/safe_client/
Platform=$1
Description="SAFE Network client"

function add_file_check {
  local TargetFileName=$1
  local TargetPath=$2
  printf 'if [ ! -f %s ]; then\n' "$TargetPath$TargetFileName" >>  after_install.sh
  printf '  echo "%s is missing from %s" >&2\n' "$TargetFileName" "$TargetPath" >>  after_install.sh
  printf '  exit 1\nfi\n\n' >>  after_install.sh
}

function set_owner {
  printf 'chown -R $USER:$UserGroup %s\n' "$ConfigFilePath" >> after_install.sh
  printf 'chown $USER:$UserGroup %s\n' "$ClientPath$ClientName" >> after_install.sh
  printf 'chmod 775 %s\n\n' "$ClientPath$ClientName" >> after_install.sh
}

function prepare_for_tar {
  mkdir -p "$RootDir/packages/$Platform"
  cd "$RootDir/packages/$Platform"
  Bits=$(getconf LONG_BIT)
  PackageName="$ClientName"_"$Version"_"$Bits"-bit
  AfterInstallCommand=
  BeforeRemoveCommand=
  ExtraFilesCommand=
}

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
    $AfterInstallCommand \
    $BeforeRemoveCommand \
    "$RootDir/target/release/examples/$ClientName"=$ClientPath \
    "$RootDir/installer/common/$ClientName.crust.config"=$ConfigFilePath \
    $ExtraFilesCommand
}

cd "$RootDir"
cargo update
cargo build --release
# strip "$RootDir/target/release/$ClientName"
rm -rf "$RootDir/packages/$Platform" || true
if [[ "$1" == "linux" ]]
then
  prepare_for_tar
  create_package tar
  gzip $PackageName.tar

  # prepare_systemd_scripts
  create_package deb
  create_package rpm

  # prepare_sysv_style_scripts
  create_package deb
  create_package rpm
elif [[ "$1" == "osx" ]]
then
  prepare_for_tar
  create_package tar

  create_package osxpkg
fi
