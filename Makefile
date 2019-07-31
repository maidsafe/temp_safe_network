SHELL := /bin/bash
USER_ID := $(shell id -u)
GROUP_ID := $(shell id -g)
UNAME_S := $(shell uname -s)
PWD := $(shell echo $$PWD)
UUID := $(shell uuidgen | sed 's/-//g')
S3_BUCKET := safe-jenkins-build-artifacts
GITHUB_REPO_OWNER := maidsafe
GITHUB_REPO_NAME := safe_vault

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

package-build-artifacts:
ifndef SAFE_VAULT_BRANCH
	@echo "A branch or PR reference must be provided."
	@echo "Please set SAFE_VAULT_BRANCH to a valid branch or PR reference."
	@exit 1
endif
ifndef SAFE_VAULT_BUILD_NUMBER
	@echo "A build number must be supplied for build artifact packaging."
	@echo "Please set SAFE_VAULT_BUILD_NUMBER to a valid build number."
	@exit 1
endif
ifndef SAFE_VAULT_BUILD_OS
	@echo "A value must be supplied for SAFE_VAULT_BUILD_OS."
	@echo "Valid values are 'linux' or 'windows' or 'macos'."
	@exit 1
endif
	$(eval ARCHIVE_NAME := ${SAFE_VAULT_BRANCH}-${SAFE_VAULT_BUILD_NUMBER}-safe_vault-${SAFE_VAULT_BUILD_OS}-x86_64.tar.gz)
	tar -C artifacts -zcvf ${ARCHIVE_NAME} .
	rm artifacts/**
	mv ${ARCHIVE_NAME} artifacts
