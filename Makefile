SHELL := /bin/bash
SAFE_CLI_VERSION := $(shell grep "^version" < Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
USER_ID := $(shell id -u)
GROUP_ID := $(shell id -g)
UNAME_S := $(shell uname -s)
PWD := $(shell echo $$PWD)
UUID := $(shell uuidgen | sed 's/-//g')
S3_BUCKET := safe-jenkins-build-artifacts

build-container:
	rm -rf target/
	docker rmi -f maidsafe/safe-cli-build:${SAFE_CLI_VERSION}
	docker build -f Dockerfile.build -t maidsafe/safe-cli-build:${SAFE_CLI_VERSION} .

push-container:
	docker push maidsafe/safe-cli-build:${SAFE_CLI_VERSION}

test:
	rm -rf artifacts
	mkdir artifacts
ifeq ($(UNAME_S),Linux)
	docker run --name "safe-cli-build-${UUID}" -v "${PWD}":/usr/src/safe-cli:Z \
		-u ${USER_ID}:${GROUP_ID} \
		maidsafe/safe-cli-build:${SAFE_CLI_VERSION} \
		./resources/test-scripts/all-tests
	docker cp "safe-cli-build-${UUID}":/target .
	docker rm "safe-cli-build-${UUID}"
else
	./resources/test-scripts/all-tests
endif
	find target/release -maxdepth 1 -type f -exec cp '{}' artifacts \;

package-build-artifacts:
ifndef SAFE_CLI_BRANCH
	@echo "A branch or PR reference must be provided."
	@echo "Please set SAFE_CLI_BRANCH to a valid branch or PR reference."
	@exit 1
endif
ifndef SAFE_CLI_BUILD_NUMBER
	@echo "A build number must be supplied for build artifact packaging."
	@echo "Please set SAFE_CLI_BUILD_NUMBER to a valid build number."
	@exit 1
endif
ifndef SAFE_CLI_BUILD_OS
	@echo "A value must be supplied for SAFE_CLI_BUILD_OS."
	@echo "Valid values are 'linux' or 'windows' or 'macos'."
	@exit 1
endif
	$(eval ARCHIVE_NAME := ${SAFE_CLI_BRANCH}-${SAFE_CLI_BUILD_NUMBER}-safe_cli-${SAFE_CLI_BUILD_OS}-x86_64.tar.gz)
	tar -C artifacts -zcvf ${ARCHIVE_NAME} .
	rm artifacts/**
	mv ${ARCHIVE_NAME} artifacts

retrieve-all-build-artifacts:
ifndef SAFE_CLI_BRANCH
	@echo "A branch or PR reference must be provided."
	@echo "Please set SAFE_CLI_BRANCH to a valid branch or PR reference."
	@exit 1
endif
ifndef SAFE_CLI_BUILD_NUMBER
	@echo "A build number must be supplied for build artifact packaging."
	@echo "Please set SAFE_CLI_BUILD_NUMBER to a valid build number."
	@exit 1
endif
	rm -rf artifacts
	mkdir -p artifacts/linux/release
	mkdir -p artifacts/win/release
	mkdir -p artifacts/macos/release
	aws s3 cp --no-sign-request --region eu-west-2 s3://${S3_BUCKET}/${SAFE_CLI_BRANCH}-${SAFE_CLI_BUILD_NUMBER}-safe_cli-linux-x86_64.tar.gz .
	aws s3 cp --no-sign-request --region eu-west-2 s3://${S3_BUCKET}/${SAFE_CLI_BRANCH}-${SAFE_CLI_BUILD_NUMBER}-safe_cli-windows-x86_64.tar.gz .
	aws s3 cp --no-sign-request --region eu-west-2 s3://${S3_BUCKET}/${SAFE_CLI_BRANCH}-${SAFE_CLI_BUILD_NUMBER}-safe_cli-macos-x86_64.tar.gz .
	tar -C artifacts/linux/release -xvf ${SAFE_CLI_BRANCH}-${SAFE_CLI_BUILD_NUMBER}-safe_cli-linux-x86_64.tar.gz
	tar -C artifacts/win/release -xvf ${SAFE_CLI_BRANCH}-${SAFE_CLI_BUILD_NUMBER}-safe_cli-windows-x86_64.tar.gz
	tar -C artifacts/macos/release -xvf ${SAFE_CLI_BRANCH}-${SAFE_CLI_BUILD_NUMBER}-safe_cli-macos-x86_64.tar.gz
	rm ${SAFE_CLI_BRANCH}-${SAFE_CLI_BUILD_NUMBER}-safe_cli-linux-x86_64.tar.gz
	rm ${SAFE_CLI_BRANCH}-${SAFE_CLI_BUILD_NUMBER}-safe_cli-windows-x86_64.tar.gz
	rm ${SAFE_CLI_BRANCH}-${SAFE_CLI_BUILD_NUMBER}-safe_cli-macos-x86_64.tar.gz

clean:
# Both taskkill and killall could be called when safe_auth isn't running.
# The behaviour of both is to error if there are no processes that match the name,
# so we can just pipe it through || true to ignore that error and continue.
ifeq ($(OS),Windows_NT)
	cmd.exe /c "taskkill /F /IM safe_auth.exe" || true
else ifeq ($(UNAME_S),Darwin)
	lsof -t -i tcp:${RANDOM_PORT_NUMBER} | xargs -n 1 -x kill
	rm -rf ~/safe_auth-${RANDOM_PORT_NUMBER}
endif

package-commit_hash-artifacts-for-deploy:
	rm -f *.tar
	rm -rf deploy
	mkdir deploy
	tar -C artifacts/linux/release -cvf safe_cli-linux-$$(git rev-parse --short HEAD)-x86_64.tar safe
	tar -C artifacts/win/release -cvf safe_cli-win-$$(git rev-parse --short HEAD)-x86_64.tar safe.exe
	tar -C artifacts/macos/release -cvf safe_cli-macos-$$(git rev-parse --short HEAD)-x86_64.tar safe
	mv safe_cli-linux-$$(git rev-parse --short HEAD)-x86_64.tar deploy
	mv safe_cli-win-$$(git rev-parse --short HEAD)-x86_64.tar deploy
	mv safe_cli-macos-$$(git rev-parse --short HEAD)-x86_64.tar deploy

package-version-artifacts-for-deploy:
	rm -f *.tar
	rm -rf deploy
	mkdir deploy
	tar -C artifacts/linux/release -cvf safe_cli-linux-${SAFE_CLI_VERSION}-x86_64.tar safe
	tar -C artifacts/win/release -cvf safe_cli-win-${SAFE_CLI_VERSION}-x86_64.tar safe.exe
	tar -C artifacts/macos/release -cvf safe_cli-macos-${SAFE_CLI_VERSION}-x86_64.tar safe
	mv safe_cli-linux-${SAFE_CLI_VERSION}-x86_64.tar deploy
	mv safe_cli-win-${SAFE_CLI_VERSION}-x86_64.tar deploy
	mv safe_cli-macos-${SAFE_CLI_VERSION}-x86_64.tar deploy

deploy-github-release:
ifndef GITHUB_TOKEN
	@echo "Please set GITHUB_TOKEN to the API token for a user who can create releases."
	@exit 1
endif
	github-release release \
		--user maidsafe \
		--repo safe-cli \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe-cli" \
		--description "Command line interface for the SAFE Network";
	github-release upload \
		--user maidsafe \
		--repo safe-cli \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe_cli-linux-${SAFE_CLI_VERSION}-x86_64.tar" \
		--file deploy/safe_cli-linux-${SAFE_CLI_VERSION}-x86_64.tar;
	github-release upload \
		--user maidsafe \
		--repo safe-cli \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe_cli-win-${SAFE_CLI_VERSION}-x86_64.tar" \
		--file deploy/safe_cli-win-${SAFE_CLI_VERSION}-x86_64.tar;
	github-release upload \
		--user maidsafe \
		--repo safe-cli \
		--tag ${SAFE_CLI_VERSION} \
		--name "safe_cli-macos-${SAFE_CLI_VERSION}-x86_64.tar" \
		--file deploy/safe_cli-macos-${SAFE_CLI_VERSION}-x86_64.tar;
