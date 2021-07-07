SHELL := /bin/bash
SN_NODE_VERSION := $(shell grep "^version" < Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
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

.ONESHELL:
package-version-artifacts-for-deploy:
	rm -f *.zip *.tar.gz
	rm -rf ${DEPLOY_PATH}
	mkdir -p ${DEPLOY_PROD_PATH}

	zip -j sn_node-${SN_NODE_VERSION}-x86_64-unknown-linux-musl.zip \
		artifacts/prod/x86_64-unknown-linux-musl/release/sn_node
	zip -j sn_node-latest-x86_64-unknown-linux-musl.zip \
		artifacts/prod/x86_64-unknown-linux-musl/release/sn_node
	zip -j sn_node-${SN_NODE_VERSION}-x86_64-pc-windows-msvc.zip \
		artifacts/prod/x86_64-pc-windows-msvc/release/sn_node.exe
	zip -j sn_node-latest-x86_64-pc-windows-msvc.zip \
		artifacts/prod/x86_64-pc-windows-msvc/release/sn_node.exe
	zip -j sn_node-${SN_NODE_VERSION}-x86_64-apple-darwin.zip \
		artifacts/prod/x86_64-apple-darwin/release/sn_node
	zip -j sn_node-latest-x86_64-apple-darwin.zip \
		artifacts/prod/x86_64-apple-darwin/release/sn_node
	zip -j sn_node-${SN_NODE_VERSION}-arm-unknown-linux-musleabi.zip \
		artifacts/prod/arm-unknown-linux-musleabi/release/sn_node
	zip -j sn_node-latest-arm-unknown-linux-musleabi.zip \
		artifacts/prod/arm-unknown-linux-musleabi/release/sn_node
	zip -j sn_node-${SN_NODE_VERSION}-armv7-unknown-linux-musleabihf.zip \
		artifacts/prod/armv7-unknown-linux-musleabihf/release/sn_node
	zip -j sn_node-latest-armv7-unknown-linux-musleabihf.zip \
		artifacts/prod/armv7-unknown-linux-musleabihf/release/sn_node

	tar -C artifacts/prod/x86_64-unknown-linux-musl/release \
		-zcvf sn_node-${SN_NODE_VERSION}-x86_64-unknown-linux-musl.tar.gz sn_node
	tar -C artifacts/prod/x86_64-unknown-linux-musl/release \
		-zcvf sn_node-latest-x86_64-unknown-linux-musl.tar.gz sn_node
	tar -C artifacts/prod/x86_64-pc-windows-msvc/release \
		-zcvf sn_node-${SN_NODE_VERSION}-x86_64-pc-windows-msvc.tar.gz sn_node.exe
	tar -C artifacts/prod/x86_64-pc-windows-msvc/release \
		-zcvf sn_node-latest-x86_64-pc-windows-msvc.tar.gz sn_node.exe
	tar -C artifacts/prod/x86_64-apple-darwin/release \
		-zcvf sn_node-${SN_NODE_VERSION}-x86_64-apple-darwin.tar.gz sn_node
	tar -C artifacts/prod/x86_64-apple-darwin/release \
		-zcvf sn_node-latest-x86_64-apple-darwin.tar.gz sn_node
	tar -C artifacts/prod/arm-unknown-linux-musleabi/release \
		-zcvf sn_node-${SN_NODE_VERSION}-arm-unknown-linux-musleabi.tar.gz sn_node
	tar -C artifacts/prod/arm-unknown-linux-musleabi/release \
		-zcvf sn_node-latest-arm-unknown-linux-musleabi.tar.gz sn_node
	tar -C artifacts/prod/armv7-unknown-linux-musleabihf/release \
		-zcvf sn_node-${SN_NODE_VERSION}-armv7-unknown-linux-musleabihf.tar.gz sn_node
	tar -C artifacts/prod/armv7-unknown-linux-musleabihf/release \
		-zcvf sn_node-latest-armv7-unknown-linux-musleabihf.tar.gz sn_node

	mv *.zip ${DEPLOY_PROD_PATH}
	mv *.tar.gz ${DEPLOY_PROD_PATH}
