#!/usr/bin/env bash

NORMAL=$(tput sgr0)
RED=$(tput setaf 1)
GREEN=$(tput setaf 2)
YELLOW=$(tput setaf 3)
BOLD=$(tput bold)
TESTNET_NAME="beta"
SN_NODE_VERSION="0.10.0"

usage_info=""
report=""
result="pass"

#
# Logging/Reporting
#
function append_result_to_report() {
    local duration=$SECONDS
    local message="$1"
    local result="$2"
    local time_taken="$(($duration / 60))m $(($duration % 60))s"
    if [[ "$result" == "pass" ]]; then
        report="${report}${message} ✅ (${time_taken})\n"
    else
        report="${report}${message} ❌ (${time_taken})\n"
    fi
}

function log_start() {
    SECONDS=0
    local message="$1"
    printf "${BOLD}info:${NORMAL} ${YELLOW}%s${NORMAL}\n" "$message"
}

function log_pass() {
    printf "${BOLD}info:${NORMAL} ${GREEN}Test passed${NORMAL}\n"
}

function log_error() {
    printf "${BOLD}error:${NORMAL} ${RED}please inspect the output of the test${NORMAL}\n"
}

function get_usage_info() {
    safe_version=$(get_safe_version)
    sn_node_version=$(get_local_node_version)
    sn_node_remote_version=$(get_remote_node_version)
    sn_node_remote_bin_info=$(get_remote_node_bin_info)

    usage_info="Safe CLI Version: ${safe_version}\n"
    usage_info="${usage_info}$(file $HOME/.safe/cli/safe | awk -F ':' '{ print $2 }' | xargs)\n\n"
    usage_info="${usage_info}Local Safe Node Version: ${sn_node_version}\n"
    usage_info="${usage_info}$(file $HOME/.safe/node/sn_node | awk -F ':' '{ print $2 }' | xargs)\n\n"
    usage_info="${usage_info}Remote Safe Node Version: ${sn_node_remote_version}\n"
    usage_info="${usage_info}${sn_node_remote_bin_info}\n"
}

#
# Test Utils
#
function clean() {
    rm -f test.txt
    rm -rf $HOME/.safe
    rm -rf /tmp/sn_testnet_tool
}

function run_testnet() {
    (
        cd /tmp
        git clone https://github.com/maidsafe/sn_testnet_tool.git
        cd sn_testnet_tool
        export SN_TESTNET_NODE_VERSION="${SN_NODE_VERSION}"
        make $TESTNET_NAME # Runs testnet and copies prefix-map to ~/.safe/prefix_maps
    )
}

function clean_testnet() {
    (
        cd /tmp/sn_testnet_tool
        make clean-$TESTNET_NAME
    )
    rm -rf /tmp/sn_testnet_tool
}

function get_safe_version() {
    $HOME/.safe/cli/safe --version | awk '{print $2}' | xargs
    # This failure could occur if safe was upgraded to the wrong type, e.g. from Aarch64 -> Linux.
    if [[ $? -ne 0 ]]; then result="fail"; log_error; fi
}

function get_latest_safe_version() {
    # The xargs strips whitespace and the cut strips off the leading 'v'.
    gh release view --repo maidsafe/sn_cli \
        | head -n2 | grep tag | awk -F ':' '{print $2}' | xargs | cut -c2-
}

function get_local_node_version() {
    $HOME/.safe/node/sn_node --version | awk '{print $2}' | xargs
    # This failure could occur if sn_node was upgraded to the wrong type, e.g. from Aarch64 -> Linux.
    if [[ $? -ne 0 ]]; then result="fail"; log_error; fi
}

function get_remote_node_version() {
    (
        cd /tmp/sn_testnet_tool
        remote_ip_address=$(cat $TESTNET_NAME-ip-list | head -n1)
        output=$(ssh root@$remote_ip_address "./sn_node --version")
        echo $output | awk '{ print $2 }'
    )
}

function get_remote_node_bin_info() {
    (
        cd /tmp/sn_testnet_tool
        remote_ip_address=$(cat $TESTNET_NAME-ip-list | head -n1)
        output=$(ssh root@$remote_ip_address "file ./sn_node")
        echo $output | awk -F ':' '{ print $2 }' | xargs
    )
}

function get_latest_node_version() {
    gh release view --repo maidsafe/safe_network \
        | head -n2 | grep tag | awk -F ':' '{print $2}' | xargs | cut -c2-
}

function install_old_version_of_safe() {
    # This version is one of the first with Aarch64 builds and the --no-confirm flag.
    local old_version="0.33.3"
    local archive_file_name="sn_cli-${old_version}-aarch64-unknown-linux-musl.tar.gz"
    local url="https://github.com/maidsafe/sn_cli/releases/download/v${old_version}/${archive_file_name}"

    rm -rf "$HOME/.safe/cli"
    (
        cd /tmp
        mkdir -p "$HOME/.safe/cli"
        curl -L -O $url
        tar xvf $archive_file_name -C "$HOME/.safe/cli"
        rm $archive_file_name
    )
}

function install_old_version_of_sn_node() {
    local old_version="0.14.0"
    local archive_file_name="sn_node-${old_version}-aarch64-unknown-linux-musl.tar.gz"
    local url="https://github.com/maidsafe/safe_network/releases/download/v${old_version}/${archive_file_name}"

    rm -rf "$HOME/.safe/node"
    (
        cd /tmp
        mkdir -p "$HOME/.safe/node"
        curl -L -O $url
        tar xvf $archive_file_name -C "$HOME/.safe/node"
        rm $archive_file_name
    )
}

function test_safe_matches_latest_version() {
    result="pass"
    local actual_version=$(get_safe_version)
    local expected_version=$(get_latest_safe_version)
    if [[ "$actual_version" != "$expected_version" ]]; then
        result="fail"
        log_error
        printf "Expected version: $expected_version\n"
        printf "Actual version: $actual_version\n"
    fi
}

function test_node_matches_latest_version() {
    result="pass"
    local actual_version=$(get_local_node_version)
    local expected_version=$(get_latest_node_version)
    if [[ "$actual_version" != "$expected_version" ]]; then
        result="fail"
        log_error
        printf "Expected version: $expected_version\n"
        printf "Actual version: $actual_version\n"
    fi
}

function test_local_node_has_joined_testnet() {
    log_file_path="$HOME/.safe/node/local-node/sn_node.log.$(date --utc +%Y-%m-%d-%k)"
    retries=0
    joined="false"
    while [[ $joined == "false" && retries -le 10 ]]
    do
        if grep "Bootstrapped!" "$log_file_path"; then
            joined="true"
            printf "Node successfully joined the testnet.\n"
        else
            printf "Node has still not joined the testnet. Querying again in 10 seconds...\n"
            sleep 10
            ((retries++))
        fi
    done
    if [[ "$joined" == "true" ]]; then result="pass"; else result="failed"; log_error; fi
}

#
# Tests
#
function test_build() {
    result="pass"
    log_start "Running test - The Safe Network CLI should build successfully"
    cargo build
    if [[ $? -ne 0 ]]; then result="fail"; log_error; fi
    append_result_to_report "The Safe Network CLI should build successfully" "$result"
    [[ "$result" == "pass" ]] && log_pass
    printf "\n"
}

function test_install_script() {
    clean
    result="pass"
    log_start "Running test - The Safe Network CLI install script should run successfully"
    rm -rf $HOME/.safe/
    ./install.sh
    if [[ $? -ne 0 ]]; then result="fail"; log_error; fi
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report "The Safe Network CLI install script should run successfully" "$result"

    log_start "Running test - The Safe Network CLI should be installed at the correct location"
    result="pass"
    if [[ ! -f "$HOME/.safe/cli/safe" ]]; then result="fail"; log_error; fi
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report "The Safe Network CLI should be installed at the correct location" "$result"

    log_start "Running test - The Safe Network CLI install script should install an ARM build"
    result="pass"
    local output=$(file "$HOME/.safe/cli/safe")
    if [[ ! "$output" == *"aarch64"* ]]; then result="fail"; log_error; fi
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report "The Safe Network CLI install script should install an ARM build" "$result"

    log_start "Running test - The Safe Network CLI install script should install the correct version"
    test_safe_matches_latest_version
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report \
        "The Safe Network CLI install script should install the correct version" "$result"
    printf "\n"
}

function test_safe_cli_update() {
    clean
    log_start "Running test - The Safe Network CLI update should run successfully"
    install_old_version_of_safe

    result="pass"
    $HOME/.safe/cli/safe update --no-confirm
    if [[ $? -ne 0 ]]; then result="fail"; log_error; fi
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report "The Safe Network CLI update should run successfully" "$result"

    log_start "Running test - The Safe Network CLI update should update to the latest version"
    test_safe_matches_latest_version
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report \
        "The Safe Network CLI update should update to the latest version" "$result"
    printf "\n"
}

function test_safe_node_install() {
    clean
    log_start "Running test - The Safe Network CLI should successfully install a node"
    ./install.sh

    result="pass"
    $HOME/.safe/cli/safe node install
    if [[ ! -f "$HOME/.safe/node/sn_node" ]]; then result="fail"; log_error; fi
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report \
        "The Safe Network CLI should successfully install a node" "$result"

    log_start "Running test - The Safe Network CLI should install the node with the correct version"
    result="pass"
    test_node_matches_latest_version
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report \
        "The Safe Network CLI should install the node with the correct version" "$result"
    printf "\n"
}

function test_sn_node_update() {
    clean
    install_old_version_of_sn_node
    ./install.sh

    result="pass"
    log_start "Running test - The Safe Network CLI should successfully update a node"
    $HOME/.safe/cli/safe node update
    if [[ $? -ne 0 ]]; then result="fail"; log_error; fi
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report "The Safe Network CLI should successfully update a node" "$result"

    log_start "Running test - The Safe Network CLI node update should update the node to the latest version"
    test_node_matches_latest_version
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report \
        "The Safe Network CLI node update should update the node to the latest version" "$result"
    printf "\n"
}

function test_safe_node_join() {
    clean
    ./install.sh
    $HOME/.safe/cli/safe node install --version "$SN_NODE_VERSION"
    run_testnet
    $HOME/.safe/cli/safe networks add $TESTNET_NAME

    result="pass"
    log_start "Running test - The Safe Network CLI should successfully add the local node to a testnet"
    $HOME/.safe/cli/safe node join
    test_local_node_has_joined_testnet
    [[ "$result" == "pass" ]] && log_pass
    append_result_to_report \
        "The Safe Network CLI should successfully add the local node to a testnet" "$result"
}

function test_key_creation() {
    # This test assumes the node join test has ran first and thus a testnet exists.
    # It's also making the assumption that there's a safe installation with a network configured.
    result="pass"
    log_start "Running test - The Safe Network CLI should successfully create keys on the testnet"
    $HOME/.safe/cli/safe keys create
    if [[ $? -eq 0 ]]; then result="pass"; log_pass; else result="fail"; log_error; fi
    append_result_to_report \
        "The Safe Network CLI should successfully create keys on the testnet" "$result"
}

function test_upload_download_file() {
    # These tests assume the node join test has ran first and thus a testnet exists.
    # It's also making the assumption that there's a safe installation with a network configured.
    result="pass"
    log_start "Running test - The Safe Network CLI should successfully upload a file to the testnet"
    echo "hello world" > test.txt
    safe_url=$($HOME/.safe/cli/safe files put test.txt | head -n1 | grep -oP '"\K[^"\047]+(?=["\047])')
    if [[ $? -eq 0 ]]; then result="pass"; log_pass; else result="fail"; log_error; fi
    append_result_to_report \
        "The Safe Network CLI should successfully upload a file to the testnet" "$result"

    result="pass"
    rm test.txt
    log_start "Running test - The Safe Network CLI should successfully download a file from the testnet"
    $HOME/.safe/cli/safe files get $safe_url/test.txt .
    contents=$(cat test.txt)
    if [[ $contents == "hello world" ]]; then result="pass"; log_pass; else result="fail"; log_error; fi
    append_result_to_report \
        "The Safe Network CLI should successfully download a file from the testnet" "$result"
}

function output_report() {
    get_usage_info
    cat banner.txt
    printf "$usage_info\n"
    printf "$TESTNET_NAME with 20 nodes\n"
    printf "$report"
}

function dump_remote_logs() {
    (
        cd /tmp/sn_testnet_tool
        rm -rf logs
        ./scripts/logs
    )
    cp -r /tmp/sn_testnet_tool/logs .
}

test_build
test_install_script
test_safe_cli_update
test_safe_node_install
test_sn_node_update
test_safe_node_join
test_key_creation
test_upload_download_file
dump_remote_logs
output_report
