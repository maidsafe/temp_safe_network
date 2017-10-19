FROM alpine:latest
MAINTAINER MaidSafe Developers <dev@maidsafe.net>

#
# Set argument default values
# @note: Do not add spaces on either side of the equal sign
#
ARG RUST_LOG=safe_vault=debug
ARG SAFE_VAULT_HARD_CODED_CONTACTS=
ARG SAFE_VAULT_WHITELISTED_NODE_IPS=
ARG SAFE_VAULT_WHITELISTED_CLIENT_IPS=
ARG SAFE_VAULT_TCP_ACCEPTOR_PORT=null
ARG SAFE_VAULT_FORCE_ACCEPTOR_PORT_IN_EXT_EP=false
ARG SAFE_VAULT_SERVICE_DISCOVERY_PORT=null
ARG SAFE_VAULT_BOOTSTRAP_CACHE_NAME=null
ARG SAFE_VAULT_NETWORK_NAME=null
ARG SAFE_VAULT_DISABLE_REACHABILITY_REQUIREMENT=true
ARG SAFE_VAULT_ALLOW_MULTIPLE_LAN_NODES=true
ARG SAFE_VAULT_DISABLE_CLIENT_RATE_LIMITER=true
ARG SAFE_VAULT_DISABLE_RESOURCE_PROOF=true
ARG SAFE_VAULT_MIN_SECTION_SIZE=1
ARG SAFE_VAULT_WALLET_ADDRESS=null
ARG SAFE_VAULT_MAX_CAPACITY=null
ARG SAFE_VAULT_DISABLE_MUTATION_LIMIT=true
ARG SAFE_VAULT_LOG_LEVEL_ASYNC_CONSOLE_FILTERS=info
ARG SAFE_VAULT_LOG_LEVEL_ROOT=error
ARG SAFE_VAULT_LOG_LEVEL_CRUST=debug
ARG SAFE_VAULT_LOG_LEVEL_ROUTING=trace
ARG SAFE_VAULT_LOG_LEVEL_ROUTING_STATS=trace
ARG SAFE_VAULT_LOG_LEVEL_SAFE_VAULT=trace

#
# Convert arguments to container environment variables
# @note: Do not add spaces on either side of the equal sign
#
ENV RUST_LOG=${RUST_LOG}
ENV SAFE_VAULT_HARD_CODED_CONTACTS=$SAFE_VAULT_HARD_CODED_CONTACTS
ENV SAFE_VAULT_WHITELISTED_NODE_IPS=$SAFE_VAULT_WHITELISTED_NODE_IPS
ENV SAFE_VAULT_WHITELISTED_CLIENT_IPS=$SAFE_VAULT_WHITELISTED_CLIENT_IPS
ENV SAFE_VAULT_TCP_ACCEPTOR_PORT=$SAFE_VAULT_TCP_ACCEPTOR_PORT
ENV SAFE_VAULT_FORCE_ACCEPTOR_PORT_IN_EXT_EP=$SAFE_VAULT_FORCE_ACCEPTOR_PORT_IN_EXT_EP
ENV SAFE_VAULT_SERVICE_DISCOVERY_PORT=$SAFE_VAULT_SERVICE_DISCOVERY_PORT
ENV SAFE_VAULT_BOOTSTRAP_CACHE_NAME=$SAFE_VAULT_BOOTSTRAP_CACHE_NAME
ENV SAFE_VAULT_NETWORK_NAME=$SAFE_VAULT_NETWORK_NAME
ENV SAFE_VAULT_DISABLE_REACHABILITY_REQUIREMENT=$SAFE_VAULT_DISABLE_REACHABILITY_REQUIREMENT
ENV SAFE_VAULT_ALLOW_MULTIPLE_LAN_NODES=$SAFE_VAULT_ALLOW_MULTIPLE_LAN_NODES
ENV SAFE_VAULT_DISABLE_CLIENT_RATE_LIMITER=$SAFE_VAULT_DISABLE_CLIENT_RATE_LIMITER
ENV SAFE_VAULT_DISABLE_RESOURCE_PROOF=$SAFE_VAULT_DISABLE_RESOURCE_PROOF
ENV SAFE_VAULT_MIN_SECTION_SIZE=$SAFE_VAULT_MIN_SECTION_SIZE
ENV SAFE_VAULT_WALLET_ADDRESS=$SAFE_VAULT_WALLET_ADDRESS
ENV SAFE_VAULT_MAX_CAPACITY=$SAFE_VAULT_MAX_CAPACITY
ENV SAFE_VAULT_DISABLE_MUTATION_LIMIT=$SAFE_VAULT_DISABLE_MUTATION_LIMIT
ENV SAFE_VAULT_LOG_LEVEL_ASYNC_CONSOLE_FILTERS=$SAFE_VAULT_LOG_LEVEL_ASYNC_CONSOLE_FILTERS
ENV SAFE_VAULT_LOG_LEVEL_ROOT=$SAFE_VAULT_LOG_LEVEL_ROOT
ENV SAFE_VAULT_LOG_LEVEL_CRUST=$SAFE_VAULT_LOG_LEVEL_CRUST
ENV SAFE_VAULT_LOG_LEVEL_ROUTING=$SAFE_VAULT_LOG_LEVEL_ROUTING
ENV SAFE_VAULT_LOG_LEVEL_ROUTING_STATS=$SAFE_VAULT_LOG_LEVEL_ROUTING_STATS
ENV SAFE_VAULT_LOG_LEVEL_SAFE_VAULT=$SAFE_VAULT_LOG_LEVEL_SAFE_VAULT

#
# Install dependencies and configure
#
RUN addgroup -S -g 1000 maidsafe                           && \
    adduser -D -S -u 1000 -G maidsafe maidsafe             && \
    echo "maidsafe ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers && \
    sed -i -e "s/\/bin\/ash/\/bin\/bash/" "/etc/passwd"    && \
    apk update                                             && \
    apk add alpine-sdk                                     && \
    apk add bash                                           && \
    apk add bash-completion                                && \
    apk add wget                                           && \
    apk add curl                                           && \
    apk add vim                                            && \
    apk add rust                                           && \
    apk add cargo                                          && \
    rm -rf /var/cache/apk/*                                && \
    mkdir -p /home/maidsafe/src/safe_vault

#
# Copy the .bashrc file to root and maidsafe user's home directories
#
COPY installer/docker/config/.bashrc /home/maidsafe/.bashrc
COPY installer/docker/config/.bashrc /root/.bashrc

#
# Set container settings 
#
USER    maidsafe
ENV     SHELL  /bin/bash
ENV     EDITOR /usr/bin/vim
WORKDIR /home/maidsafe/src/safe_vault

#
# Copy files and folders to the current working directory
#
COPY src/            ./src/
COPY tests/          ./tests/
COPY installer/      ./installer/
COPY target/release/ ./target/release/
COPY Cargo.lock      .
COPY Cargo.toml      .
COPY build.rs        .
COPY rustfmt.toml    .

#
# Update permissions of safe_vault source and entrypoint script
#
RUN sudo chown -R maidsafe:maidsafe /home/maidsafe/src && \
         chmod +x installer/docker/docker-entrypoint.sh

#
# Run the safe vault application
#
EXPOSE 5000 5100
ENTRYPOINT ["./installer/docker/docker-entrypoint.sh"]
