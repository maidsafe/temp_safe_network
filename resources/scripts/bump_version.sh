#!/usr/bin/env bash

dry_run_output=""
commit_message=""
sn_testnet=""
sn_updater=""
sn_fault_detection_version=""
sn_interface_version=""
sn_comms_version=""
sn_client_version=""
sn_node_version=""
sn_api_version=""
sn_cli_version=""
sn_updater_has_changes="false"
sn_fault_detection_has_changes="false"
sn_interface_has_changes="false"
sn_comms_has_changes="false"
sn_client_has_changes="false"
sn_node_has_changes="false"
sn_api_has_changes="false"
sn_cli_has_changes="false"

function perform_smart_release_dry_run() {
  echo "Performing dry run for smart-release..."
  dry_run_output=$(cargo smart-release \
    --update-crates-index \
    --no-push \
    --no-publish \
    --no-changelog-preview \
    --allow-fully-generated-changelogs \
    --no-changelog-github-release \
    "sn_testnet" \
    "sn_updater" \
    "sn_fault_detection" "sn_interface" "sn_comms" "sn_node" "sn_client" "sn_api" "sn_cli" 2>&1)
  echo "Dry run output for smart-release:"
  echo $dry_run_output
}

function crate_has_changes() {
  local crate_name="$1"
  if [[ $dry_run_output == *"WOULD auto-bump provided package '$crate_name'"* ]] || \
     [[ $dry_run_output == *"WOULD auto-bump dependent package '$crate_name'"* ]]; then
    echo "true"
  else
    echo "false"
  fi
}

function determine_which_crates_have_changes() {
  local has_changes
  has_changes=$(crate_has_changes "sn_testnet")
  if [[ $has_changes == "true" ]]; then
    echo "smart-release has determined sn_testnet crate has changes"
    sn_testnet_has_changes="true"
  fi

  has_changes=$(crate_has_changes "sn_updater")
  if [[ $has_changes == "true" ]]; then
    echo "smart-release has determined sn_updater crate has changes"
    sn_updater_has_changes="true"
  fi

  has_changes=$(crate_has_changes "sn_fault_detection")
  if [[ $has_changes == "true" ]]; then
    echo "smart-release has determined sn_fault_detection crate has changes"
    sn_fault_detection_has_changes="true"
  fi

  has_changes=$(crate_has_changes "sn_interface")
  if [[ $has_changes == "true" ]]; then
    echo "smart-release has determined sn_interface crate has changes"
    sn_interface_has_changes="true"
  fi

  has_changes=$(crate_has_changes "sn_comms")
  if [[ $has_changes == "true" ]]; then
    echo "smart-release has determined sn_comms crate has changes"
    sn_comms_has_changes="true"
  fi

  has_changes=$(crate_has_changes "sn_node")
  if [[ $has_changes == "true" ]]; then
    echo "smart-release has determined sn_node crate has changes"
    sn_node_has_changes="true"
  fi

  has_changes=$(crate_has_changes "sn_client")
  if [[ $has_changes == "true" ]]; then
    echo "smart-release has determined sn_client crate has changes"
    sn_client_has_changes="true"
  fi

  has_changes=$(crate_has_changes "sn_api")
  if [[ $has_changes == "true" ]]; then
    echo "smart-release has determined sn_api crate has changes"
    sn_api_has_changes="true"
  fi

  has_changes=$(crate_has_changes "sn_cli")
  if [[ $has_changes == "true" ]]; then
    echo "smart-release has determined sn_cli crate has changes"
    sn_cli_has_changes="true"
  fi

  if [[ $sn_testnet_has_changes == "false" ]] && \
     [[ $sn_updater_has_changes == "false" ]] && \
     [[ $sn_fault_detection_has_changes == "false" ]] && \
     [[ $sn_interface_has_changes == "false" ]] && \
     [[ $sn_comms_has_changes == "false" ]] && \
     [[ $sn_client_has_changes == "false" ]] && \
     [[ $sn_node_has_changes == "false" ]] && \
     [[ $sn_api_has_changes == "false" ]] && \
     [[ $sn_cli_has_changes == "false" ]]; then
       echo "smart-release detected no changes in any crates. Exiting."
       exit 0
  fi
}

function generate_version_bump_commit() {
  echo "Running smart-release with --execute flag..."
  cargo smart-release \
    --update-crates-index \
    --no-push \
    --no-publish \
    --no-changelog-preview \
    --allow-fully-generated-changelogs \
    --no-changelog-github-release \
    --execute \
    "sn_testnet" \
    "sn_updater" \
    "sn_fault_detection" "sn_interface" "sn_comms" "sn_node" "sn_client" "sn_api" "sn_cli"
  exit_code=$?
  if [[ $exit_code -ne 0 ]]; then
    echo "smart-release did not run successfully. Exiting with failure code."
    exit 1
  fi
}

function generate_new_commit_message() {
  sn_testnet_version=$( \
    grep "^version" < sn_testnet/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_updater_version=$( \
    grep "^version" < sn_updater/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_interface_version=$( \
    grep "^version" < sn_interface/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_fault_detection_version=$( \
    grep "^version" < sn_fault_detection/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_comms_version=$(grep "^version" < sn_comms/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_client_version=$(grep "^version" < sn_client/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_node_version=$(grep "^version" < sn_node/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_api_version=$(grep "^version" < sn_api/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  sn_cli_version=$(grep "^version" < sn_cli/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  commit_message="chore(release): "

  if [[ $sn_testnet_has_changes == "true" ]]; then
    commit_message="${commit_message}sn_testnet-${sn_testnet_version}/"
  fi
  if [[ $sn_updater_has_changes == "true" ]]; then
    commit_message="${commit_message}sn_updater-${sn_updater_version}/"
  fi
  if [[ $sn_interface_has_changes == "true" ]]; then
    commit_message="${commit_message}sn_interface-${sn_interface_version}/"
  fi
  if [[ $sn_fault_detection_has_changes == "true" ]]; then
    commit_message="${commit_message}sn_fault_detection-${sn_fault_detection_version}/"
  fi
  if [[ $sn_comms_has_changes == "true" ]]; then
    commit_message="${commit_message}sn_comms-${sn_comms_version}/"
  fi
  if [[ $sn_client_has_changes == "true" ]]; then
    commit_message="${commit_message}sn_client-${sn_client_version}/"
  fi
  if [[ $sn_node_has_changes == "true" ]]; then
    commit_message="${commit_message}sn_node-${sn_node_version}/"
  fi
  if [[ $sn_api_has_changes == "true" ]]; then
    commit_message="${commit_message}sn_api-${sn_api_version}/"
  fi
  if [[ $sn_cli_has_changes == "true" ]]; then
    commit_message="${commit_message}sn_cli-${sn_cli_version}/"
  fi
  commit_message=${commit_message::-1} # strip off any trailing '/'
  echo "generated commit message -- $commit_message"
}

function amend_version_bump_commit() {
  git reset --soft HEAD~1
  git add --all
  git commit -m "$commit_message"
}

function amend_tags() {
  if [[ $sn_testnet_has_changes == "true" ]]; then
    git tag "sn_testnet-v${sn_testnet_version}" -f
  fi
  if [[ $sn_updater_has_changes == "true" ]]; then
    git tag "sn_updater-v${sn_updater_version}" -f
  fi
  if [[ $sn_interface_has_changes == "true" ]]; then
    git tag "sn_interface-v${sn_interface_version}" -f
  fi
  if [[ $sn_fault_detection_has_changes == "true" ]]; then
    git tag "sn_fault_detection-v${sn_fault_detection_version}" -f
  fi
  if [[ $sn_comms_has_changes == "true" ]]; then git tag "sn_comms-v${sn_comms_version}" -f; fi
  if [[ $sn_client_has_changes == "true" ]]; then git tag "sn_client-v${sn_client_version}" -f; fi
  if [[ $sn_node_has_changes == "true" ]]; then git tag "sn_node-v${sn_node_version}" -f; fi
  if [[ $sn_api_has_changes == "true" ]]; then git tag "sn_api-v${sn_api_version}" -f; fi
  if [[ $sn_cli_has_changes == "true" ]]; then git tag "sn_cli-v${sn_cli_version}" -f; fi
}

perform_smart_release_dry_run
determine_which_crates_have_changes
generate_version_bump_commit
generate_new_commit_message
amend_version_bump_commit
amend_tags
