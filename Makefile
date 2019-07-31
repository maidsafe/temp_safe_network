SHELL := /bin/bash
USER_ID := $(shell id -u)
GROUP_ID := $(shell id -g)
UNAME_S := $(shell uname -s)
PWD := $(shell echo $$PWD)
UUID := $(shell uuidgen | sed 's/-//g')
S3_BUCKET := safe-jenkins-build-artifacts
GITHUB_REPO_OWNER := maidsafe
GITHUB_REPO_NAME := safe-cli

build-container:
	rm -rf target/
	docker rmi -f maidsafe/safe-vault-build:build
	docker build -f Dockerfile.build -t maidsafe/safe-vault-build:build .

push-container:
	docker push maidsafe/safe-vault-build:build

clippy:
ifeq ($(UNAME_S),Linux)
	docker run --name "safe-vault-build-${UUID}" \
		-v "${PWD}":/usr/src/safe_vault:Z \
		-u ${USER_ID}:${GROUP_ID} \
		maidsafe/safe-vault-build:build \
		./scripts/clippy --verbose
else
	./scripts/clippy --verbose
endif

test:
	rm -rf artifacts
	mkdir artifacts
ifeq ($(UNAME_S),Linux)
	docker run --name "safe-vault-build-${UUID}" \
		-v "${PWD}":/usr/src/safe_vault:Z \
		-u ${USER_ID}:${GROUP_ID} \
		maidsafe/safe-vault-build:build \
		./scripts/tests --verbose
	docker cp "safe-vault-build-${UUID}":/target .
	docker rm "safe-vault-build-${UUID}"
else
	./scripts/tests --verbose
endif
	find target/release -maxdepth 1 -type f -exec cp '{}' artifacts \;
