SHELL := /bin/bash
SN_NODE_VERSION := $(shell grep "^version" < sn/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
SN_CLI_VERSION := $(shell grep "^version" < sn_cli/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
UNAME_S := $(shell uname -s)
DEPLOY_PATH := deploy
DEPLOY_PROD_PATH := ${DEPLOY_PATH}/prod

gha-build-x86_64-pc-windows-msvc: release-build

gha-build-x86_64-apple-darwin: release-build

gha-build-x86_64-unknown-linux-musl:
	rm -rf target
	rm -rf artifacts
	mkdir artifacts
	sudo apt update -y && sudo apt install -y musl-tools
	rustup target add x86_64-unknown-linux-musl
	cargo build --release --target x86_64-unknown-linux-musl --bin sn_node
	cargo build --release --target x86_64-unknown-linux-musl \
		--no-default-features --features testing --bin safe
	find target/x86_64-unknown-linux-musl/release -maxdepth 1 -type f -exec cp '{}' artifacts \;
	rm -f artifacts/.cargo-lock

gha-build-arm-unknown-linux-musleabi: arm-unknown-linux-musleabi
gha-build-armv7-unknown-linux-musleabihf: armv7-unknown-linux-musleabihf
gha-build-aarch64-unknown-linux-musl: aarch64-unknown-linux-musl

arm-unknown-linux-musleabi:
	rm -rf target
	rm -rf artifacts
	mkdir artifacts
	cargo install cross
	cross build --release --target arm-unknown-linux-musleabi --bin sn_node
	cross build --release --target arm-unknown-linux-musleabi --bin safe \
		--no-default-features --features testing
	find target/arm-unknown-linux-musleabi/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

armv7-unknown-linux-musleabihf:
	rm -rf target
	rm -rf artifacts
	mkdir artifacts
	cargo install cross
	cross build --release --target armv7-unknown-linux-musleabihf --bin sn_node
	cross build --release --target armv7-unknown-linux-musleabihf --bin safe \
		--no-default-features --features testing
	find target/armv7-unknown-linux-musleabihf/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

aarch64-unknown-linux-musl:
	rm -rf target
	rm -rf artifacts
	mkdir artifacts
	cargo install cross
	cross build --release --target aarch64-unknown-linux-musl --bin sn_node
	cross build --release --target aarch64-unknown-linux-musl --bin safe \
		--no-default-features --features testing
	find target/aarch64-unknown-linux-musl/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

release-build:
	rm -rf artifacts
	mkdir artifacts
	cargo build --release --bin sn_node
	cargo build --release --no-default-features --features testing --bin safe
	find target/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

.ONESHELL:
safe_network-build-artifacts-for-deploy:
	# This target is just for debugging the packaging process.
	# Given the zipped artifacts retrieved from Github, it creates the
	# directory structure that's expected by the packaging target.
	declare -a architectures=( \
		"x86_64-unknown-linux-musl" \
		"x86_64-pc-windows-msvc" \
		"x86_64-apple-darwin" \
		"arm-unknown-linux-musleabi" \
		"armv7-unknown-linux-musleabihf" \
		"aarch64-unknown-linux-musl")
	cd artifacts
	for arch in "$${architectures[@]}" ; do \
		mkdir -p prod/$$arch/release; \
		unzip safe_network-$$arch.zip -d prod/$$arch/release; \
		rm safe_network-$$arch.zip; \
	done

prepare-deploy:
	find artifacts/ -name safe -exec chmod +x '{}' \;
	find artifacts/ -name sn_node -exec chmod +x '{}' \;
	rm -f *.zip *.tar.gz
	rm -rf ${DEPLOY_PATH}
	mkdir -p ${DEPLOY_PROD_PATH}/safe
	mkdir -p ${DEPLOY_PROD_PATH}/sn_node

.ONESHELL:
safe_network-package-version-artifacts-for-release:
	declare -a architectures=( \
		"x86_64-unknown-linux-musl" \
		"x86_64-pc-windows-msvc" \
		"x86_64-apple-darwin" \
		"arm-unknown-linux-musleabi" \
		"armv7-unknown-linux-musleabihf" \
		"aarch64-unknown-linux-musl")

	for arch in "$${architectures[@]}" ; do \
		if [[ $$arch == *"windows"* ]]; then bin_name="sn_node.exe"; else bin_name="sn_node"; fi; \
		zip -j sn_node-${SN_NODE_VERSION}-$$arch.zip artifacts/prod/$$arch/release/$$bin_name; \
		tar -C artifacts/prod/$$arch/release -zcvf sn_node-${SN_NODE_VERSION}-$$arch.tar.gz $$bin_name; \
	done

	mv *.tar.gz ${DEPLOY_PROD_PATH}/sn_node
	mv *.zip ${DEPLOY_PROD_PATH}/sn_node

.ONESHELL:
sn_cli-package-version-artifacts-for-release:
	declare -a architectures=( \
		"x86_64-unknown-linux-musl" \
		"x86_64-pc-windows-msvc" \
		"x86_64-apple-darwin" \
		"arm-unknown-linux-musleabi" \
		"armv7-unknown-linux-musleabihf" \
		"aarch64-unknown-linux-musl")

	for arch in "$${architectures[@]}" ; do \
		if [[ $$arch == *"windows"* ]]; then bin_name="safe.exe"; else bin_name="safe"; fi; \
		zip -j sn_cli-${SN_CLI_VERSION}-$$arch.zip artifacts/prod/$$arch/release/$$bin_name; \
		tar -C artifacts/prod/$$arch/release -zcvf sn_cli-${SN_CLI_VERSION}-$$arch.tar.gz $$bin_name; \
	done

	mv *.tar.gz ${DEPLOY_PROD_PATH}/safe
	mv *.zip ${DEPLOY_PROD_PATH}/safe

.ONESHELL:
upload-sn_node-musl-to-s3:
	# This target can be used for quickly uploading a build to S3 to be used with the testnet tool.
	# The testnet tool will download this 'latest' version as the default.
	rm -rf target
	rm -rf deploy
	mkdir -p deploy
	rustup target add x86_64-unknown-linux-musl
	cargo build --release --target x86_64-unknown-linux-musl --bin sn_node
	(
		cd deploy;
		tar -C ../target/x86_64-unknown-linux-musl/release \
			-zcvf sn_node-latest-x86_64-unknown-linux-musl.tar.gz sn_node;
		aws s3 cp sn_node-latest-x86_64-unknown-linux-musl.tar.gz s3://sn-node --acl public-read;
	)

.ONESHELL:
run-local-baby-fleming:
	pgrep sn_node | xargs kill -9
	rm -rf ~/.safe
	mkdir -p ~/.safe/node
	(cd sn && cargo build --release)
	cp target/release/sn_node ~/.safe/node
	cargo run --release node run-baby-fleming
