#!/usr/bin/env bash

version=$1
if [[ -z "$version" ]]; then
    echo "You must supply a version number for the tag."
    exit 1
fi

component=$2
if [[ -z "$component" ]]; then
    echo "You must supply the component to build."
    echo "Valid values are 'sn_cli', 'sn_api', 'sn_authd' or 'safe-ffi'."
    exit 1
fi

git config --global user.name "$GIT_USER"
git config --global user.email qa@maidsafe.net
git config credential.username "$GIT_USER"
git config credential.helper "!f() { echo password=$GIT_PASSWORD; }; f"
git tag -a "$version" -m "Creating tag for $version"
GIT_ASKPASS=true git push origin --tags --no-verify
