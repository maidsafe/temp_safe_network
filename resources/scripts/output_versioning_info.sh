#!/usr/bin/env bash

sn_dysfunction_version=""
sn_interface_version=""
sn_client_version=""
sn_node_version=""
sn_api_version=""
sn_cli_version=""

function get_crate_versions() {
  sn_dysfunction_version=$( \
    grep "^version" < sn_dysfunction/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_interface_version=$( \
    grep "^version" < sn_interface/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_client_version=$( \
    grep "^version" < sn_client/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_node_version=$(grep "^version" < sn_node/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_api_version=$(grep "^version" < sn_api/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_cli_version=$(grep "^version" < sn_cli/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
}

function build_release_name() {
  gh_release_name="Safe Network v$sn_dysfunction_version/"
  gh_release_name="${gh_release_name}v$sn_interface_version/"
  gh_release_name="${gh_release_name}v$sn_client_version/"
  gh_release_name="${gh_release_name}v$sn_node_version/"
  gh_release_name="${gh_release_name}v$sn_api_version/"
  gh_release_name="${gh_release_name}v$sn_cli_version"
}

function build_release_tag_name() {
  gh_release_tag_name="$sn_interface_version-"
  gh_release_tag_name="${gh_release_tag_name}$sn_dysfunction_version-"
  gh_release_tag_name="${gh_release_tag_name}$sn_client_version-"
  gh_release_tag_name="${gh_release_tag_name}$sn_node_version-"
  gh_release_tag_name="${gh_release_tag_name}$sn_api_version-"
  gh_release_tag_name="${gh_release_tag_name}$sn_cli_version"
}

function output_version_info() {
  echo "::set-output name=sn_dysfunction_version::$sn_dysfunction_version"
  echo "::set-output name=sn_interface_version::$sn_interface_version"
  echo "::set-output name=sn_client_version::$sn_client_version"
  echo "::set-output name=sn_node_version::$sn_node_version"
  echo "::set-output name=sn_api_version::$sn_api_version"
  echo "::set-output name=sn_cli_version::$sn_cli_version"
  echo "::set-output name=gh_release_name::$gh_release_name"
  echo "::set-output name=gh_release_tag_name::$gh_release_tag_name"
}

gh_release_name=""
gh_release_tag_name=""
commit_message=$(git log --oneline --pretty=format:%s | head -n 1)
get_crate_versions
build_release_name
build_release_tag_name
output_version_info
