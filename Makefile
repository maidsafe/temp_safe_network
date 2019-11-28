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
S3_BUCKET := safe-jenkins-build-artifacts
SAFE_AUTH_DEFAULT_PORT := 41805
GITHUB_REPO_OWNER := maidsafe
GITHUB_REPO_NAME := safe-api

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

retrieve-ios-build-artifacts:
ifndef SAFE_CLI_BRANCH
	@echo "A branch or PR reference must be provided."
	@echo "Please set SAFE_CLI_BRANCH to a valid branch or PR reference."
	@exit 1
endif
ifndef SAFE_API_BUILD_NUMBER
	@echo "A valid build number must be supplied for the artifacts to be retrieved."
	@echo "Please set SAFE_API_BUILD_NUMBER to a valid build number."
	@exit 1
endif
	rm -rf artifacts
	./resources/retrieve-build-artifacts.sh "x86_64-apple-ios" "aarch64-apple-ios"

universal-ios-lib:
ifneq ($(UNAME_S),Darwin)
	@echo "This target can only be run on macOS"
	@exit 1
endif
	mkdir -p artifacts/safe-ffi/prod/universal
	# mkdir -p artifacts/safe-ffi/dev/universal
	lipo -create -output artifacts/safe-ffi/prod/universal/libsafe_ffi.a \
		artifacts/safe-ffi/prod/x86_64-apple-ios/release/libsafe_ffi.a \
		artifacts/safe-ffi/prod/aarch64-apple-ios/release/libsafe_ffi.a

strip-artifacts:
ifeq ($(OS),Windows_NT)
	find artifacts -name "safe.exe" -exec strip -x '{}' \;
else ifeq ($(UNAME_S),Darwin)
	find artifacts -name "safe" -exec strip -x '{}' \;
else
	find artifacts -name "safe" -exec strip '{}' \;
endif

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
		bash -c "./resources/test-scripts/run-auth-daemon && ./resources/test-scripts/cli-tests"
	docker cp "safe-cli-build-${UUID}":/target .
	docker rm "safe-cli-build-${UUID}"
else
	$(eval MOCK_VAULT_PATH := ~/safe_auth-${SAFE_AUTH_PORT})
	RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
	   ./resources/test-scripts/run-auth-daemon
	RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
	   ./resources/test-scripts/cli-tests
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
		bash -c "./resources/test-scripts/run-auth-daemon && ./resources/test-scripts/cli-tests"
	docker cp "safe-authd-build-${UUID}":/target .
	docker rm "safe-authd-build-${UUID}"
else
	$(eval MOCK_VAULT_PATH := ~/safe_auth-${SAFE_AUTH_PORT})
	RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
	   ./resources/test-scripts/run-auth-daemon
	RANDOM_PORT_NUMBER=${SAFE_AUTH_PORT} SAFE_MOCK_VAULT_PATH=${MOCK_VAULT_PATH} \
	   ./resources/test-scripts/cli-tests
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

package-build-artifacts:
ifndef SAFE_CLI_BRANCH
	@echo "A branch or PR reference must be provided."
	@echo "Please set SAFE_CLI_BRANCH to a valid branch or PR reference."
	@exit 1
endif
ifndef SAFE_API_BUILD_NUMBER
	@echo "A build number must be supplied for build artifact packaging."
	@echo "Please set SAFE_API_BUILD_NUMBER to a valid build number."
	@exit 1
endif
ifndef SAFE_API_BUILD_TYPE
	@echo "A value must be supplied for SAFE_API_BUILD_TYPE."
	@echo "Valid values are 'dev' or 'prod'."
	@exit 1
endif
ifndef SAFE_API_BUILD_COMPONENT
	@echo "A value must be supplied for SAFE_API_BUILD_COMPONENT."
	@echo "Valid values are 'safe-li', 'safe-api' or 'safe-ffi'."
	@exit 1
endif
ifndef SAFE_API_BUILD_TARGET
	@echo "A value must be supplied for SAFE_API_BUILD_TARGET."
	@exit 1
endif
	$(eval ARCHIVE_NAME := ${SAFE_CLI_BRANCH}-${SAFE_API_BUILD_NUMBER}-${SAFE_API_BUILD_COMPONENT}-${SAFE_API_BUILD_TYPE}-${SAFE_API_BUILD_TARGET}.tar.gz)
	tar -C artifacts -zcvf ${ARCHIVE_NAME} .
	rm artifacts/**
	mv ${ARCHIVE_NAME} artifacts

retrieve-all-build-artifacts:
ifndef SAFE_CLI_BRANCH
	@echo "A branch or PR reference must be provided."
	@echo "Please set SAFE_CLI_BRANCH to a valid branch or PR reference."
	@exit 1
endif
ifndef SAFE_API_BUILD_NUMBER
	@echo "A build number must be supplied for build artifact packaging."
	@echo "Please set SAFE_API_BUILD_NUMBER to a valid build number."
	@exit 1
endif
	rm -rf artifacts
	./resources/retrieve-build-artifacts.sh \
		"x86_64-apple-darwin" "x86_64-apple-ios" \
		"aarch64-apple-ios" "apple-ios"
	find artifacts -type d -empty -delete
	rm -rf artifacts/safe-ffi/prod/aarch64-apple-ios
	rm -rf artifacts/safe-ffi/prod/x86_64-apple-ios

package-universal-ios-lib:
ifndef SAFE_CLI_BRANCH
	@echo "A branch or PR reference must be provided."
	@echo "Please set SAFE_CLI_BRANCH to a valid branch or PR reference."
	@exit 1
endif
ifndef SAFE_API_BUILD_NUMBER
	@echo "A valid build number must be supplied for the artifacts to be retrieved."
	@echo "Please set SAFE_API_BUILD_NUMBER to a valid build number."
	@exit 1
endif
	( \
		cd artifacts; \
		tar -C safe-ffi/prod/universal -zcvf \
			${SAFE_CLI_BRANCH}-${SAFE_API_BUILD_NUMBER}-safe-ffi-prod-apple-ios.tar.gz .; \
	)
	rm -rf artifacts/safe-ffi

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
	./resources/package-deploy-artifacts.sh "safe-authd" ${COMMIT_HASH}
	./resources/package-deploy-artifacts.sh "safe-cli" ${COMMIT_HASH}
	./resources/package-deploy-artifacts.sh "safe-ffi" ${COMMIT_HASH}
	find deploy -name "*.tar.gz" -exec rm '{}' \;

package-version-artifacts-for-deploy:
	rm -rf deploy
	mkdir -p deploy/prod
	./resources/package-deploy-artifacts.sh "safe-authd" "${SAFE_AUTHD_VERSION}"
	./resources/package-deploy-artifacts.sh "safe-cli" "${SAFE_CLI_VERSION}"
	./resources/package-deploy-artifacts.sh "safe-ffi" "${SAFE_FFI_VERSION}"
	find deploy -name "safe-ffi-*.tar.gz" -exec rm '{}' \;

deploy-github-release:
ifndef GITHUB_TOKEN
	@echo "Please set GITHUB_TOKEN to the API token for a user who can create releases."
	@exit 1
endif
	github-release release \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe-cli" \
		--description "$$(./resources/get_release_description.sh ${SAFE_CLI_VERSION})";
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe-cli-${SAFE_CLI_VERSION}-x86_64-unknown-linux-gnu.zip" \
		--file deploy/real/safe-cli-${SAFE_CLI_VERSION}-x86_64-unknown-linux-gnu.zip;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe-cli-${SAFE_CLI_VERSION}-x86_64-pc-windows-gnu.zip" \
		--file deploy/real/safe-cli-${SAFE_CLI_VERSION}-x86_64-pc-windows-gnu.zip;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe-cli-${SAFE_CLI_VERSION}-x86_64-apple-darwin.zip" \
		--file deploy/real/safe-cli-${SAFE_CLI_VERSION}-x86_64-apple-darwin.zip;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe-cli-${SAFE_CLI_VERSION}-x86_64-unknown-linux-gnu.tar.gz" \
		--file deploy/real/safe-cli-${SAFE_CLI_VERSION}-x86_64-unknown-linux-gnu.tar.gz;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe-cli-${SAFE_CLI_VERSION}-x86_64-pc-windows-gnu.tar.gz" \
		--file deploy/real/safe-cli-${SAFE_CLI_VERSION}-x86_64-pc-windows-gnu.tar.gz;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe-cli-${SAFE_CLI_VERSION}-x86_64-apple-darwin.tar.gz" \
		--file deploy/real/safe-cli-${SAFE_CLI_VERSION}-x86_64-apple-darwin.tar.gz;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe_completion.sh" \
		--file resources/safe_completion.sh
	# safe-authd
	github-release release \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_AUTHD_VERSION} \
		--name "safe-authd" \
		--description "$$(./resources/get_release_description.sh ${SAFE_AUTHD_VERSION})";
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_AUTHD_VERSION} \
		--name "safe-authd-${SAFE_AUTHD_VERSION}-x86_64-unknown-linux-gnu.zip" \
		--file deploy/real/safe-authd-${SAFE_AUTHD_VERSION}-x86_64-unknown-linux-gnu.zip;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_AUTHD_VERSION} \
		--name "safe-authd-${SAFE_AUTHD_VERSION}-x86_64-pc-windows-gnu.zip" \
		--file deploy/real/safe-authd-${SAFE_AUTHD_VERSION}-x86_64-pc-windows-gnu.zip;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_AUTHD_VERSION} \
		--name "safe-authd-${SAFE_AUTHD_VERSION}-x86_64-apple-darwin.zip" \
		--file deploy/real/safe-authd-${SAFE_AUTHD_VERSION}-x86_64-apple-darwin.zip;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_AUTHD_VERSION} \
		--name "safe-authd-${SAFE_AUTHD_VERSION}-x86_64-unknown-linux-gnu.tar.gz" \
		--file deploy/real/safe-authd-${SAFE_AUTHD_VERSION}-x86_64-unknown-linux-gnu.tar.gz;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_AUTHD_VERSION} \
		--name "safe-authd-${SAFE_AUTHD_VERSION}-x86_64-pc-windows-gnu.tar.gz" \
		--file deploy/real/safe-authd-${SAFE_AUTHD_VERSION}-x86_64-pc-windows-gnu.tar.gz;
	github-release upload \
		--user ${GITHUB_REPO_OWNER} \
		--repo ${GITHUB_REPO_NAME} \
		--tag ${SAFE_AUTHD_VERSION} \
		--name "safe-authd-${SAFE_AUTHD_VERSION}-x86_64-apple-darwin.tar.gz" \
		--file deploy/real/safe-authd-${SAFE_AUTHD_VERSION}-x86_64-apple-darwin.tar.gz;


retrieve-cache:
ifndef SAFE_CLI_BRANCH
	@echo "A branch reference must be provided."
	@echo "Please set SAFE_CLI_BRANCH to a valid branch reference."
	@exit 1
endif
ifndef SAFE_CLI_OS
	@echo "The OS for the cache must be specified."
	@echo "Please set SAFE_CLI_OS to either 'macos' or 'windows'."
	@exit 1
endif
	aws s3 cp \
		--no-sign-request \
		--region eu-west-2 \
		s3://${S3_BUCKET}/safe_cli-${SAFE_CLI_BRANCH}-${SAFE_CLI_OS}-cache.tar.gz .
	mkdir target
	tar -C target -xvf safe_cli-${SAFE_CLI_BRANCH}-${SAFE_CLI_OS}-cache.tar.gz
	rm safe_cli-${SAFE_CLI_BRANCH}-${SAFE_CLI_OS}-cache.tar.gz

publish-api:
ifndef CRATES_IO_TOKEN
	@echo "A login token for crates.io must be provided."
	@exit 1
endif
	rm -rf artifacts deploy
	docker run --rm -v "${PWD}":/usr/src/safe-cli:Z \
		-u ${USER_ID}:${GROUP_ID} \
		maidsafe/safe-cli-build:cli-x86_64-unknown-linux-gnu \
		/bin/bash -c "cd safe-api && cargo login ${CRATES_IO_TOKEN} && cargo package && cargo publish"

publish-authd:
ifndef CRATES_IO_TOKEN
	@echo "A login token for crates.io must be provided."
	@exit 1
endif
	rm -rf artifacts deploy
	docker run --rm -v "${PWD}":/usr/src/safe-authd:Z \
		-u ${USER_ID}:${GROUP_ID} \
		maidsafe/safe-authd-build:cli-x86_64-unknown-linux-gnu \
		/bin/bash -c "cd safe-authd && cargo login ${CRATES_IO_TOKEN} && cargo package && cargo publish"
