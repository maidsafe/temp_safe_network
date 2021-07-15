SHELL := /bin/bash
SN_CLI_VERSION := $(shell grep "^version" < Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
UNAME_S := $(shell uname -s)
DEPLOY_PATH := deploy
DEPLOY_PROD_PATH := ${DEPLOY_PATH}/prod

build:
ifeq ($(UNAME_S),Linux)
	@echo "This target should not be used for Linux - please use the `musl` target."
	@exit 1
endif
	rm -rf artifacts
	mkdir artifacts
	cargo build --release
	find target/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

musl:
ifneq ($(UNAME_S),Linux)
	@echo "This target only applies to Linux - please use the `build` target."
	@exit 1
endif
	rm -rf target
	rm -rf artifacts
	mkdir artifacts
	sudo apt update -y && sudo apt install -y musl-tools
	rustup target add x86_64-unknown-linux-musl
	cargo build --release --target x86_64-unknown-linux-musl
	find target/x86_64-unknown-linux-musl/release \
		-maxdepth 1 -type f -exec cp '{}' artifacts \;

arm-unknown-linux-musleabi:
	rm -rf target
	rm -rf artifacts
	mkdir artifacts
	cargo install cross
	cross build --release --target arm-unknown-linux-musleabi
	find target/arm-unknown-linux-musleabi/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

armv7-unknown-linux-musleabihf:
	rm -rf target
	rm -rf artifacts
	mkdir artifacts
	cargo install cross
	cross build --release --target armv7-unknown-linux-musleabihf
	find target/armv7-unknown-linux-musleabihf/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

aarch64-unknown-linux-musl:
	rm -rf target
	rm -rf artifacts
	mkdir artifacts
	cargo install cross
	cross build --release --target aarch64-unknown-linux-musl
	find target/aarch64-unknown-linux-musl/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

.ONESHELL:
build-artifacts-for-deploy:
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
		unzip sn_cli-$$arch-prod.zip -d prod/$$arch/release; \
		rm sn_cli-$$arch-prod.zip
	done

.ONESHELL:
package-version-artifacts-for-deploy:
	rm -f *.zip *.tar.gz
	rm -rf ${DEPLOY_PATH}
	mkdir -p ${DEPLOY_PROD_PATH}

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
		zip -j sn_cli-latest-$$arch.zip artifacts/prod/$$arch/release/$$bin_name; \
		tar -C artifacts/prod/$$arch/release -zcvf sn_cli-${SN_CLI_VERSION}-$$arch.tar.gz $$bin_name; \
		tar -C artifacts/prod/$$arch/release -zcvf sn_cli-latest-$$arch.tar.gz $$bin_name; \
	done

	mv *.tar.gz ${DEPLOY_PROD_PATH}
	mv *.zip ${DEPLOY_PROD_PATH}
