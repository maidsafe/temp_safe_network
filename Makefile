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
	cargo build --release --target x86_64-unknown-linux-musl --verbose
	find target/x86_64-unknown-linux-musl/release \
		-maxdepth 1 -type f -exec cp '{}' artifacts \;

.ONESHELL:
package-version-artifacts-for-deploy:
	rm -f *.zip *.tar.gz
	rm -rf ${DEPLOY_PATH}
	mkdir -p ${DEPLOY_PROD_PATH}

	zip -j sn_cli-${SN_CLI_VERSION}-x86_64-unknown-linux-musl.zip \
		artifacts/prod/x86_64-unknown-linux-musl/release/safe
	zip -j sn_cli-latest-x86_64-unknown-linux-musl.zip \
		artifacts/prod/x86_64-unknown-linux-musl/release/safe
	zip -j sn_cli-${SN_CLI_VERSION}-x86_64-pc-windows-msvc.zip \
		artifacts/prod/x86_64-pc-windows-msvc/release/safe.exe
	zip -j sn_cli-latest-x86_64-pc-windows-msvc.zip \
		artifacts/prod/x86_64-pc-windows-msvc/release/safe.exe
	zip -j sn_cli-${SN_CLI_VERSION}-x86_64-apple-darwin.zip \
		artifacts/prod/x86_64-apple-darwin/release/safe
	zip -j sn_cli-latest-x86_64-apple-darwin.zip \
		artifacts/prod/x86_64-apple-darwin/release/safe

	tar -C artifacts/prod/x86_64-unknown-linux-musl/release \
		-zcvf sn_cli-${SN_CLI_VERSION}-x86_64-unknown-linux-musl.tar.gz safe
	tar -C artifacts/prod/x86_64-unknown-linux-musl/release \
		-zcvf sn_cli-latest-x86_64-unknown-linux-musl.tar.gz safe
	tar -C artifacts/prod/x86_64-pc-windows-msvc/release \
		-zcvf sn_cli-${SN_CLI_VERSION}-x86_64-pc-windows-msvc.tar.gz safe.exe
	tar -C artifacts/prod/x86_64-pc-windows-msvc/release \
		-zcvf sn_cli-latest-x86_64-pc-windows-msvc.tar.gz safe.exe
	tar -C artifacts/prod/x86_64-apple-darwin/release \
		-zcvf sn_cli-${SN_CLI_VERSION}-x86_64-apple-darwin.tar.gz safe
	tar -C artifacts/prod/x86_64-apple-darwin/release \
		-zcvf sn_cli-latest-x86_64-apple-darwin.tar.gz safe

	mv *.zip ${DEPLOY_PROD_PATH}
	mv *.tar.gz ${DEPLOY_PROD_PATH}
