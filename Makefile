SHELL := /bin/bash
SAFE_CLI_VERSION := $(shell grep "^version" < safe-cli/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
SAFE_AUTHD_VERSION := $(shell grep "^version" < safe-authd/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
SAFE_FFI_VERSION := $(shell grep "^version" < safe-ffi/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
COMMIT_HASH := $(shell git rev-parse --short HEAD)
USER_ID := $(shell id -u)
GROUP_ID := $(shell id -g)
UNAME_S := $(shell uname -s)
PWD := $(shell echo $$PWD)
UUID := $(shell uuidgen | sed 's/-//g')
SAFE_AUTH_DEFAULT_PORT := 41805

build-component:
ifndef SAFE_API_BUILD_COMPONENT
	@echo "A build component must be specified."
	@echo "Please set SAFE_API_BUILD_COMPONENT to 'safe-api', 'safe-ffi', 'safe-authd', 'safe-authd' or 'safe-cli'."
	@exit 1
endif
ifndef SAFE_API_BUILD_TYPE
	@echo "A build type must be specified."
	@echo "Please set SAFE_API_BUILD_TYPE to 'dev' or 'prod'."
	@exit 1
endif
ifndef SAFE_API_BUILD_TARGET
	@echo "A build target must be specified."
	@echo "Please set SAFE_API_BUILD_TARGET to a valid Rust 'target triple', e.g. 'x86_64-unknown-linux-gnu'."
	@exit 1
endif
ifndef SAFE_API_BUILD_CLEAN
	$(eval SAFE_API_BUILD_CLEAN := false)
endif
	./resources/build-component.sh \
		"${SAFE_API_BUILD_COMPONENT}" \
		"${SAFE_API_BUILD_TARGET}" \
		"${SAFE_API_BUILD_TYPE}" \
		"${SAFE_API_BUILD_CLEAN}"

build-all-containers:
	SAFE_CLI_CONTAINER_TARGET=x86_64-unknown-linux-gnu \
	SAFE_CLI_CONTAINER_TYPE=prod \
	SAFE_CLI_CONTAINER_COMPONENT=safe-cli \
		make build-container
	# SAFE_CLI_CONTAINER_TARGET=x86_64-unknown-linux-gnu \
	# SAFE_CLI_CONTAINER_TYPE=dev \
	# SAFE_CLI_CONTAINER_COMPONENT=safe-cli \
	# 	make build-container
	# SAFE_CLI_CONTAINER_TARGET=x86_64-unknown-linux-gnu \
	# SAFE_CLI_CONTAINER_TYPE=dev \
	# SAFE_CLI_CONTAINER_COMPONENT=safe-api \
	# 	make build-container
	# SAFE_CLI_CONTAINER_TARGET=x86_64-unknown-linux-gnu \
	# SAFE_CLI_CONTAINER_TYPE=dev \
	# SAFE_CLI_CONTAINER_COMPONENT=safe-ffi \
	# 	make build-container
	SAFE_CLI_CONTAINER_TARGET=x86_64-unknown-linux-gnu \
	SAFE_CLI_CONTAINER_TYPE=prod \
	SAFE_CLI_CONTAINER_COMPONENT=safe-ffi \
		make build-container
	# SAFE_CLI_CONTAINER_TARGET=x86_64-linux-android \
	# SAFE_CLI_CONTAINER_TYPE=dev \
	# SAFE_CLI_CONTAINER_COMPONENT=safe-ffi \
	# 	make build-container
	SAFE_CLI_CONTAINER_TARGET=x86_64-linux-android \
	SAFE_CLI_CONTAINER_TYPE=prod \
	SAFE_CLI_CONTAINER_COMPONENT=safe-ffi \
		make build-container
	# SAFE_CLI_CONTAINER_TARGET=armv7-linux-androideabi \
	# SAFE_CLI_CONTAINER_TYPE=dev \
	# SAFE_CLI_CONTAINER_COMPONENT=safe-ffi \
	# 	make build-container
	SAFE_CLI_CONTAINER_TARGET=armv7-linux-androideabi \
	SAFE_CLI_CONTAINER_TYPE=prod \
	SAFE_CLI_CONTAINER_COMPONENT=safe-ffi \
		make build-container

build-container:
ifndef SAFE_CLI_CONTAINER_COMPONENT
	@echo "A component to build must be specified."
	@echo "Please set SAFE_CLI_CONTAINER_COMPONENT to 'safe-api', 'safe-ffi', 'safe-authd' or 'safe-cli'."
	@exit 1
endif
ifndef SAFE_CLI_CONTAINER_TYPE
	@echo "A container type must be specified."
	@echo "Please set SAFE_CLI_CONTAINER_TYPE to 'dev' or 'prod'."
	@exit 1
endif
ifndef SAFE_CLI_CONTAINER_TARGET
	@echo "A build target must be specified."
	@echo "Please set SAFE_CLI_CONTAINER_TARGET to a valid Rust 'target triple', e.g. 'x86_64-unknown-linux-gnu'."
	@exit 1
endif
	./resources/build-container.sh \
		"${SAFE_CLI_CONTAINER_COMPONENT}" \
		"${SAFE_CLI_CONTAINER_TARGET}" \
		"${SAFE_CLI_CONTAINER_TYPE}"

push-container:
ifndef SAFE_CLI_CONTAINER_COMPONENT
	@echo "A component to build must be specified."
	@echo "Please set SAFE_CLI_CONTAINER_COMPONENT to 'safe-api', 'safe-ffi', 'safe-authd' or 'safe-cli'."
	@exit 1
endif
ifndef SAFE_CLI_CONTAINER_TYPE
	@echo "A container type must be specified."
	@echo "Please set SAFE_CLI_CONTAINER_TYPE to 'dev' or 'prod'."
	@exit 1
endif
ifndef SAFE_CLI_CONTAINER_TARGET
	@echo "A build target must be specified."
	@echo "Please set SAFE_CLI_CONTAINER_TARGET to a valid Rust 'target triple', e.g. 'x86_64-unknown-linux-gnu'."
	@exit 1
endif
	docker push \
		maidsafe/safe-cli-build:${SAFE_CLI_CONTAINER_COMPONENT}-${SAFE_CLI_CONTAINER_TARGET}-${SAFE_CLI_CONTAINER_TYPE}

universal-ios-lib-dev:
ifneq ($(UNAME_S),Darwin)
	@echo "This target can only be run on macOS"
	@exit 1
endif
	mkdir -p artifacts/safe-ffi/dev/universal

	lipo -create -output artifacts/safe-ffi/dev/universal/libsafe_ffi.a \
		artifacts/safe-ffi/dev/x86_64-apple-ios/release/libsafe_ffi.a \
		artifacts/safe-ffi/dev/aarch64-apple-ios/release/libsafe_ffi.a

universal-ios-lib-prod:
ifneq ($(UNAME_S),Darwin)
	@echo "This target can only be run on macOS"
	@exit 1
endif
	mkdir -p artifacts/safe-ffi/prod/universal

	lipo -create -output artifacts/safe-ffi/prod/universal/libsafe_ffi.a \
		artifacts/safe-ffi/prod/x86_64-apple-ios/release/libsafe_ffi.a \
		artifacts/safe-ffi/prod/aarch64-apple-ios/release/libsafe_ffi.a

clippy:
ifeq ($(UNAME_S),Linux)
	docker run --name "safe-cli-build-${UUID}" -v "${PWD}":/usr/src/safe-cli:Z \
		-u ${USER_ID}:${GROUP_ID} \
		maidsafe/safe-cli-build:cli-x86_64-unknown-linux-gnu \
		/bin/bash -c "cargo clippy --all-targets --all-features -- -D warnings"
else
	cargo clippy --all-targets --all-features -- -D warnings
endif

.ONESHELL:
test-cli:
ifndef SAFE_AUTH_PORT
	$(eval SAFE_AUTH_PORT := ${SAFE_AUTH_DEFAULT_PORT})
endif
	rm -rf artifacts
	mkdir artifacts
ifeq ($(UNAME_S),Linux)
	docker run --name "safe-cli-build-${UUID}" -v "${PWD}":/usr/src/safe-cli:Z \
		-u ${USER_ID}:${GROUP_ID} \
		-e RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} \
		-e SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
		maidsafe/safe-cli-build:cli-x86_64-unknown-linux-gnu-dev \
		bash -c "./resources/test-scripts/run-auth-daemon && ./resources/test-scripts/cli-mock-tests"
	docker cp "safe-cli-build-${UUID}":/target .
	docker rm "safe-cli-build-${UUID}"
else
	$(eval MOCK_VAULT_PATH := ~/safe_auth-${SAFE_AUTH_PORT})
	RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
	   ./resources/test-scripts/run-auth-daemon
	RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
	   ./resources/test-scripts/cli-mock-tests
endif
	find target/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

.ONESHELL:
test-authd:
ifndef SAFE_AUTH_PORT
	$(eval SAFE_AUTH_PORT := ${SAFE_AUTH_DEFAULT_PORT})
endif
	rm -rf artifacts
	mkdir artifacts
ifeq ($(UNAME_S),Linux)
	docker run --name "safe-authd-build-${UUID}" -v "${PWD}":/usr/src/safe-authd:Z \
		-u ${USER_ID}:${GROUP_ID} \
		-e RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} \
		-e SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
		maidsafe/safe-authd-build:cli-x86_64-unknown-linux-gnu-dev \
		bash -c "./resources/test-scripts/run-auth-daemon && ./resources/test-scripts/cli-mock-tests"
	docker cp "safe-authd-build-${UUID}":/target .
	docker rm "safe-authd-build-${UUID}"
else
	$(eval MOCK_VAULT_PATH := ~/safe_auth-${SAFE_AUTH_PORT})
	RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
	   ./resources/test-scripts/run-auth-daemon
	RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
	   ./resources/test-scripts/cli-mock-tests
endif
	find target/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

.ONESHELL:
test-api:
ifndef SAFE_AUTH_PORT
	$(eval SAFE_AUTH_PORT := ${SAFE_AUTH_DEFAULT_PORT})
endif
	rm -rf artifacts
	mkdir artifacts
ifeq ($(UNAME_S),Linux)
	docker run --name "safe-cli-build-${UUID}" -v "${PWD}":/usr/src/safe-cli:Z \
		-u ${USER_ID}:${GROUP_ID} \
		-e RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} \
		-e SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
		maidsafe/safe-cli-build:api-x86_64-unknown-linux-gnu-dev \
		bash -c "./resources/test-scripts/run-auth-daemon && ./resources/test-scripts/api-tests"
	docker cp "safe-cli-build-${UUID}":/target .
	docker rm "safe-cli-build-${UUID}"
else
	$(eval MOCK_VAULT_PATH := ~/safe_auth-${SAFE_AUTH_PORT})
	RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
		./resources/test-scripts/run-auth-daemon
	RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
		./resources/test-scripts/api-tests
endif
	find target/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

clean:
ifndef SAFE_AUTH_PORT
	$(eval SAFE_AUTH_PORT := ${SAFE_AUTH_DEFAULT_PORT})
endif
ifeq ($(OS),Windows_NT)
	powershell.exe -File resources/test-scripts/cleanup.ps1 -port ${SAFE_AUTH_PORT}
else ifeq ($(UNAME_S),Darwin)
	lsof -t -i tcp:${SAFE_AUTH_PORT} | xargs -n 1 -x kill
endif
	$(eval MOCK_VAULT_PATH := ~/safe_auth-${SAFE_AUTH_PORT})
	rm -rf ${MOCK_VAULT_PATH}

package-commit_hash-artifacts-for-deploy:
	rm -rf deploy
	mkdir -p deploy/prod
	mkdir -p deploy/dev
	./resources/package-deploy-artifacts.sh "safe-authd" ${COMMIT_HASH}
	./resources/package-deploy-artifacts.sh "safe-cli" ${COMMIT_HASH}
	./resources/package-deploy-artifacts.sh "safe-ffi" ${COMMIT_HASH}
	find deploy -name "*.tar.gz" -exec rm '{}' \;

package-version-artifacts-for-deploy:
	rm -rf deploy
	mkdir -p deploy/prod
	mkdir -p deploy/dev
	./resources/package-deploy-artifacts.sh "safe-authd" "${SAFE_AUTHD_VERSION}"
	./resources/package-deploy-artifacts.sh "safe-cli" "${SAFE_CLI_VERSION}"
	./resources/package-deploy-artifacts.sh "safe-ffi" "${SAFE_FFI_VERSION}"
	find deploy -name "safe-ffi-*.tar.gz" -exec rm '{}' \;
