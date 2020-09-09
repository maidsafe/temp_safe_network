SAFE_CLI_VERSION := $(shell grep "^version" < sn_cli/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
SAFE_AUTHD_VERSION := $(shell grep "^version" < sn_authd/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
SAFE_FFI_VERSION := $(shell grep "^version" < safe-ffi/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

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
	./resources/package-deploy-artifacts.sh "safe-authd" "latest"
	./resources/package-deploy-artifacts.sh "safe-cli" "${SAFE_CLI_VERSION}"
	./resources/package-deploy-artifacts.sh "safe-cli" "latest"
	./resources/package-deploy-artifacts.sh "safe-ffi" "${SAFE_FFI_VERSION}"
	./resources/package-deploy-artifacts.sh "safe-ffi" "latest"
	find deploy -name "safe-ffi-*.tar.gz" -exec rm '{}' \;
