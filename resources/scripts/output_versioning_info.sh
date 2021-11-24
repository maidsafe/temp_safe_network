#!/usr/bin/env bash

function build_release_name() {
    if [[ $commit_message == *"safe_network"* ]]; then
        gh_release_name="Safe Network v$sn_version/"
    fi
    if [[ $commit_message == *"sn_api"* ]]; then
        gh_release_name="${gh_release_name}Safe API v$sn_api_version/"
    fi
    if [[ $commit_message == *"sn_cli"* ]]; then
        gh_release_name="${gh_release_name}Safe CLI v$sn_cli_version/"
    fi
    gh_release_name=${gh_release_name::-1} # strip off any trailing '/' 
}

function build_release_tag_name() {
    # This is to avoid having a '/' in the tag name, which Github don't seem to like.
    # If the sn crate has had changes, we'll just use that as the tag name, otherwise
    # use sn_api, otherwise sn_cli. It doesn't really matter too much, as this is just
    # tag for the Github Release. The actual release tags will have already been created
    # at version bumping, and there will be a tag for each of the crates.
    if [[ $commit_message == *"safe_network"* ]]; then
        gh_release_tag_name="safe_network-v$sn_version"
    elif [[ $commit_message == *"sn_api"* ]]; then
        gh_release_tag_name="sn_api-v$sn_api_version"
    elif [[ $commit_message == *"sn_cli"* ]]; then
        gh_release_tag_name="sn_cli-v$sn_cli_version"
    else
        echo "Unable to set the tag name for the Github Release"
        echo "The commit message doesn't contain any expected text"
        exit 1
    fi
}

function output_version_info() {
    echo "::set-output name=sn_version::$sn_version"
    echo "::set-output name=sn_api_version::$sn_api_version"
    echo "::set-output name=sn_cli_version::$sn_cli_version"
    echo "::set-output name=gh_release_name::$gh_release_name"
    echo "::set-output name=gh_release_tag_name::$gh_release_tag_name"
}

gh_release_name=""
gh_release_tag_name=""
commit_message=$(git log --oneline --pretty=format:%s | head -n 1)
sn_version=$(grep "^version" < sn/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_api_version=$(grep "^version" < sn_api/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
sn_cli_version=$(grep "^version" < sn_cli/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
build_release_name
build_release_tag_name
output_version_info
