#!/usr/bin/env bash

#
# Save the path to this script's directory in a global env variable
#
if [[ -z "${SAFE_VAULT_DIR_ENTRYPOINT}" ]]; then
    export SAFE_VAULT_DIR_ENTRYPOINT="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
fi

#
# Save the app's directory path to an environment variable
#
if [[ -z "${SAFE_VAULT_DIR_ROOT}" ]]; then
    export SAFE_VAULT_DIR_ROOT="$(dirname $(dirname "${SAFE_VAULT_DIR_ENTRYPOINT}"))"
fi

#
# Save the target directory's path to an environment variable
#
if [[ -z "${SAFE_VAULT_DIR_TARGET}" ]]; then
    export SAFE_VAULT_DIR_TARGET="${SAFE_VAULT_DIR_ROOT}/target/release"
fi

#
# Set temp. line separator to a comma and make copy of the original
#
declare oIFS="${IFS}"
declare  IFS=","

#
# Set Rust's log level for the application
#
export RUST_LOG=${RUST_LOG:-"safe_vault=debug"}

#
# Loop through the comma delimited string and add put quotes around each value
# @todo: Add more information about this setting
# @see : config/safe_vault.crust.config
#
export  SAFE_VAULT_HARD_CODED_CONTACTS=${SAFE_VAULT_HARD_CODED_CONTACTS:-""}
declare hard_coded_contacts=""
for contact in ${SAFE_VAULT_HARD_CODED_CONTACTS[@]}; do
    contact="$(echo -e "${contact}" | tr -d '[:space:]')"
    if [[ ${#contact} -le 0 ]]; then
        continue
    fi
    if [[ ${#hard_coded_contacts} -gt 0 ]]; then
        hard_coded_contacts="${hard_coded_contacts}, \"${contact}\""
    else
        hard_coded_contacts="\"${contact}\""
    fi
done
export SAFE_VAULT_HARD_CODED_CONTACTS="${hard_coded_contacts}"

#
# Loop through the comma delimited string and add put quotes around each value
# @todo: Add more information about this setting
# @see : config/safe_vault.crust.config
#
export  SAFE_VAULT_WHITELISTED_NODE_IPS=${SAFE_VAULT_WHITELISTED_NODE_IPS:-""}
declare ips=""
for ip in ${SAFE_VAULT_WHITELISTED_NODE_IPS[@]}; do
    ip="$(echo -e "${ip}" | tr -d '[:space:]')"
    if [[ ${#ip} -le 0 ]]; then
        continue
    fi
    if [[ ${#ips} -gt 0 ]]; then
        ips="${ips}, \"${ip}\""
    else
        ips="\"${ip}\""
    fi
done
export SAFE_VAULT_WHITELISTED_NODE_IPS="${ips}"

#
# Loop through the comma delimited string and add put quotes around each value
# @todo: Add more information about this setting
# @see : config/safe_vault.crust.config
#
export  SAFE_VAULT_WHITELISTED_CLIENT_IPS=${SAFE_VAULT_WHITELISTED_CLIENT_IPS:-""}
declare ips=""
for ip in ${SAFE_VAULT_WHITELISTED_CLIENT_IPS[@]}; do
    ip="$(echo -e "${ip}" | tr -d '[:space:]')"
    if [[ ${#ip} -le 0 ]]; then
        continue
    fi
    if [[ ${#ips} -gt 0 ]]; then
        ips="${ips}, \"${ip}\""
    else
        ips="\"${ip}\""
    fi
done
export SAFE_VAULT_WHITELISTED_CLIENT_IPS="${ips}"

#
# @todo: Add more information about this setting
# @see : config/safe_vault.crust.config
#
export SAFE_VAULT_TCP_ACCEPTOR_PORT=${SAFE_VAULT_TCP_ACCEPTOR_PORT:-null}

#
# @todo: Add more information about this setting
# @see : config/safe_vault.crust.config
#
export SAFE_VAULT_FORCE_ACCEPTOR_PORT_IN_EXT_EP=${SAFE_VAULT_FORCE_ACCEPTOR_PORT_IN_EXT_EP:-false}

#
# @todo: Add more information about this setting
# @see : config/safe_vault.crust.config
#
export SAFE_VAULT_SERVICE_DISCOVERY_PORT=${SAFE_VAULT_SERVICE_DISCOVERY_PORT:-null}

#
# @todo: Add more information about this setting
# @see : config/safe_vault.crust.config
#
export SAFE_VAULT_BOOTSTRAP_CACHE_NAME=${SAFE_VAULT_BOOTSTRAP_CACHE_NAME:-null}
if [[ "${SAFE_VAULT_BOOTSTRAP_CACHE_NAME}" != "null" ]]; then
    export SAFE_VAULT_BOOTSTRAP_CACHE_NAME="\"${SAFE_VAULT_BOOTSTRAP_CACHE_NAME}\""
fi

#
# @todo: Add more information about this setting
# @see : config/safe_vault.crust.config
#
export SAFE_VAULT_NETWORK_NAME=${SAFE_VAULT_NETWORK_NAME:-null}
if [[ "${SAFE_VAULT_NETWORK_NAME}" != "null" ]]; then
    export SAFE_VAULT_NETWORK_NAME="\"${SAFE_VAULT_NETWORK_NAME}\""
fi

#
# Normally a vault is required to be able to allow a remote vault to connect 
# to it; i.e. it must pass the “external reachability check”. For running a 
# local testnet where all vaults are on the same machine or LAN, not only is 
# this check useless, it could well fail since many routers don’t support hairpinning. 
# Setting disable_external_reachability_requirement to true disables that check.
# @see : config/safe_vault.crust.config
# @link: https://forum.safedev.org/t/how-to-run-a-local-test-network/842
#
export SAFE_VAULT_DISABLE_REACHABILITY_REQUIREMENT=${SAFE_VAULT_DISABLE_REACHABILITY_REQUIREMENT:-true}

#
# Currently, we disallow more than one vault on a single LAN or machine. 
# To disable this restriction, set allow_multiple_lan_nodes to true.
# @see : config/safe_vault.routing.config
# @link: https://forum.safedev.org/t/how-to-run-a-local-test-network/842
#
export SAFE_VAULT_ALLOW_MULTIPLE_LAN_NODES=${SAFE_VAULT_ALLOW_MULTIPLE_LAN_NODES:-true}

#
# When a vault acts as a proxy for one or more clients, it limits the number 
# and size of requests that it’ll forward on their behalf. To remove this limitation, 
# set disable_client_rate_limiter to true. Setting this to true also allows a proxy to 
# act on behalf of multiple clients with the same IP address, whereas this would not 
# normally be allowed.
# @see : config/safe_vault.routing.config
# @link: https://forum.safedev.org/t/how-to-run-a-local-test-network/842
#
export SAFE_VAULT_DISABLE_CLIENT_RATE_LIMITER=${SAFE_VAULT_DISABLE_CLIENT_RATE_LIMITER:-true}

#
# When a new vault connects to the network, each peer from the section it’s joining sends
# it a resource proof challenge. The challenge involves the new vault doing process-heavy
# work and sending a large message in response to confirm its upload capability. After a lengthy 
# delay (several minutes) the peers all vote on whether to accept the new vault or not, and only 
# if a quorum of positive votes are accumulated is the new vault allowed to join. 
# If disable_resource_proof is set to true, the new vault is sent a challenge which involves minimal 
# effort, time and traffic and the existing peers don’t delay their votes, enabling new vaults to 
# join much faster.
# @see : config/safe_vault.routing.config
# @link: https://forum.safedev.org/t/how-to-run-a-local-test-network/842
#
export SAFE_VAULT_DISABLE_RESOURCE_PROOF=${SAFE_VAULT_DISABLE_RESOURCE_PROOF:-true}

#
# The network is comprised of sections of vaults, grouped together by IDs which are close to each other. 
# These sections need to be comprised of a minimum number of vaults, currently defined as 8. Data is 
# replicated across this number of vaults. This figure can be adjusted via the optional min_section_size 
# field. As this figure reduces, the risk of data-loss increases 
# (as there are fewer copies of each chunk of data). However, increasing this figure will increase the burden 
# on each vault in terms of storage used, traffic sent and received, and network connections being maintained.
# @see : config/safe_vault.routing.config
# @link: https://forum.safedev.org/t/how-to-run-a-local-test-network/842
#
export SAFE_VAULT_MIN_SECTION_SIZE=${SAFE_VAULT_MIN_SECTION_SIZE:-1}

#
# Set the vault's wallet address
# @see : config/safe_vault.vault.config
#
export SAFE_VAULT_WALLET_ADDRESS=${SAFE_VAULT_WALLET_ADDRESS:-null}
if [[ "${SAFE_VAULT_WALLET_ADDRESS}" != "null" ]]; then
    export SAFE_VAULT_WALLET_ADDRESS="\"${SAFE_VAULT_WALLET_ADDRESS}\""
fi

#
# Set the vault's max storage capacity (in bytes)
# @see : config/safe_vault.vault.config
#
export SAFE_VAULT_MAX_CAPACITY=${SAFE_VAULT_MAX_CAPACITY:-null}
if [[ "${SAFE_VAULT_MAX_CAPACITY}" != "null" ]]; then
    export SAFE_VAULT_MAX_CAPACITY="\"${SAFE_VAULT_MAX_CAPACITY}\""
fi

#
# Currently, clients are only allowed a limited number of requests 
# to mutate (store, modify or delete) data on the network.  To remove this 
# limitation, set disable_mutation_limit to true.
# @see : config/safe_vault.vault.config
# @link: https://forum.safedev.org/t/how-to-run-a-local-test-network/842
#
export SAFE_VAULT_DISABLE_MUTATION_LIMIT=${SAFE_VAULT_DISABLE_MUTATION_LIMIT:-true}

#
# Set default value for [[async_consol.filters]] in config/log.toml
#
export SAFE_VAULT_LOG_LEVEL_ASYNC_CONSOLE_FILTERS=${SAFE_VAULT_LOG_LEVEL_ASYNC_CONSOLE_FILTERS:-info}

#
# Set default value for [root] in config/log.toml
#
export SAFE_VAULT_LOG_LEVEL_ROOT=${SAFE_VAULT_LOG_LEVEL_ROOT:-error}

#
# Set default value for [crust] in config/log.toml
#
export SAFE_VAULT_LOG_LEVEL_CRUST=${SAFE_VAULT_LOG_LEVEL_CRUST:-debug}

#
# Set default value for [routing] in config/log.toml
#
export SAFE_VAULT_LOG_LEVEL_ROUTING=${SAFE_VAULT_LOG_LEVEL_ROUTING:-trace}

#
# Set default value for [routing_stats] in config/log.toml
#
export SAFE_VAULT_LOG_LEVEL_ROUTING_STATS=${SAFE_VAULT_LOG_LEVEL_ROUTING_STATS:-trace}

#
# Set default value for [safe_vault] in config/log.toml
#
export SAFE_VAULT_LOG_LEVEL_SAFE_VAULT=${SAFE_VAULT_LOG_LEVEL_SAFE_VAULT:-trace}

#
# String parsing is complete. Set line separator back to the original.
#
declare IFS="${oIFS}"

#
# Ensure target directory exists
#
mkdir -p "${SAFE_VAULT_DIR_TARGET}/"

#
# Copy config files to the target directory
#
cp -R "${SAFE_VAULT_DIR_ROOT}/installer/docker/config/"* "${SAFE_VAULT_DIR_TARGET}/"

#
# Replace template variables in safe_vault.crust.config with actual env variables
#
sed -i                                                                                                        \
    -e "s/%{env:SAFE_VAULT_HARD_CODED_CONTACTS}/${SAFE_VAULT_HARD_CODED_CONTACTS}/"                           \
    -e "s/%{env:SAFE_VAULT_WHITELISTED_NODE_IPS}/${SAFE_VAULT_WHITELISTED_NODE_IPS}/"                         \
    -e "s/%{env:SAFE_VAULT_WHITELISTED_CLIENT_IPS}/${SAFE_VAULT_WHITELISTED_CLIENT_IPS}/"                     \
    -e "s/%{env:SAFE_VAULT_TCP_ACCEPTOR_PORT}/${SAFE_VAULT_TCP_ACCEPTOR_PORT}/"                               \
    -e "s/%{env:SAFE_VAULT_FORCE_ACCEPTOR_PORT_IN_EXT_EP}/${SAFE_VAULT_FORCE_ACCEPTOR_PORT_IN_EXT_EP}/"       \
    -e "s/%{env:SAFE_VAULT_SERVICE_DISCOVERY_PORT}/${SAFE_VAULT_SERVICE_DISCOVERY_PORT}/"                     \
    -e "s/%{env:SAFE_VAULT_BOOTSTRAP_CACHE_NAME}/${SAFE_VAULT_BOOTSTRAP_CACHE_NAME}/"                         \
    -e "s/%{env:SAFE_VAULT_NETWORK_NAME}/${SAFE_VAULT_NETWORK_NAME}/"                                         \
    -e "s/%{env:SAFE_VAULT_DISABLE_REACHABILITY_REQUIREMENT}/${SAFE_VAULT_DISABLE_REACHABILITY_REQUIREMENT}/" \
    "${SAFE_VAULT_DIR_TARGET}/safe_vault.crust.config"

#
# Replace template variables in safe_vault.routing.config with actual env variables
#
sed -i                                                                                              \
    -e "s/%{env:SAFE_VAULT_ALLOW_MULTIPLE_LAN_NODES}/${SAFE_VAULT_ALLOW_MULTIPLE_LAN_NODES}/"       \
    -e "s/%{env:SAFE_VAULT_DISABLE_CLIENT_RATE_LIMITER}/${SAFE_VAULT_DISABLE_CLIENT_RATE_LIMITER}/" \
    -e "s/%{env:SAFE_VAULT_DISABLE_RESOURCE_PROOF}/${SAFE_VAULT_DISABLE_RESOURCE_PROOF}/"           \
    -e "s/%{env:SAFE_VAULT_MIN_SECTION_SIZE}/${SAFE_VAULT_MIN_SECTION_SIZE}/"                       \
    "${SAFE_VAULT_DIR_TARGET}/safe_vault.routing.config"

#
# Replace template variables in safe_vault.vault.config with actual env variables
#
sed -i                                                                                              \
    -e "s/%{env:SAFE_VAULT_DISABLE_MUTATION_LIMIT}/${SAFE_VAULT_DISABLE_MUTATION_LIMIT}/"           \
    -e "s/%{env:SAFE_VAULT_WALLET_ADDRESS}/${SAFE_VAULT_WALLET_ADDRESS}/"                           \
    -e "s/%{env:SAFE_VAULT_MAX_CAPACITY}/${SAFE_VAULT_MAX_CAPACITY}/"                               \
    "${SAFE_VAULT_DIR_TARGET}/safe_vault.vault.config"

#
# Replace template variables in log.toml with actual env variables
#
sed -i                                                                                                      \
    -e "s/%{env:SAFE_VAULT_LOG_LEVEL_ASYNC_CONSOLE_FILTERS}/${SAFE_VAULT_LOG_LEVEL_ASYNC_CONSOLE_FILTERS}/" \
    -e "s/%{env:SAFE_VAULT_LOG_LEVEL_ROOT}/${SAFE_VAULT_LOG_LEVEL_ROOT}/"                                   \
    -e "s/%{env:SAFE_VAULT_LOG_LEVEL_CRUST}/${SAFE_VAULT_LOG_LEVEL_CRUST}/"                                 \
    -e "s/%{env:SAFE_VAULT_LOG_LEVEL_ROUTING}/${SAFE_VAULT_LOG_LEVEL_ROUTING}/"                             \
    -e "s/%{env:SAFE_VAULT_LOG_LEVEL_ROUTING_STATS}/${SAFE_VAULT_LOG_LEVEL_ROUTING_STATS}/"                 \
    -e "s/%{env:SAFE_VAULT_LOG_LEVEL_SAFE_VAULT}/${SAFE_VAULT_LOG_LEVEL_SAFE_VAULT}/"                       \
    "${SAFE_VAULT_DIR_TARGET}/log.toml"

#
# Build the application if it doesn't exist or update the permissions if it does
#
if [[ ! -f "${SAFE_VAULT_DIR_TARGET}/safe_vault" ]]; then
    cargo build --release && chmod +x "${SAFE_VAULT_DIR_TARGET}/safe_vault"
else
    chmod +x "${SAFE_VAULT_DIR_TARGET}/safe_vault"
fi

#
# Run the process(es)
#
cd "${SAFE_VAULT_DIR_TARGET}"
if [[ ${SAFE_VAULT_MIN_SECTION_SIZE} -gt 1 && "${SAFE_VAULT_ALLOW_MULTIPLE_LAN_NODES}" == true ]]; then
    ./safe_vault -f &
    echo "Pausing while the bootstrap node completely starts...."
    sleep 10s
    for ((i=2; i <= ${SAFE_VAULT_MIN_SECTION_SIZE}; i++)); do
        echo "Starting node #${i}/${SAFE_VAULT_MIN_SECTION_SIZE}..."
        if [[ $i -lt ${SAFE_VAULT_MIN_SECTION_SIZE} ]]; then
            # Run this node in the background, so other process can be started
            ./safe_vault &
        else
            # Run the last process in the foreground, so the docker container doesn't exit
            ./safe_vault
        fi
    done
else
    ./safe_vault -f
fi

cd "${SAFE_VAULT_DIR_ROOT}"
