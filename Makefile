SN_CLI_VERSION := $(shell grep "^version" < sn_cli/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
SN_AUTHD_VERSION := $(shell grep "^version" < sn_authd/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
SN_FFI_VERSION := $(shell grep "^version" < sn_ffi/Cargo.toml | head -n 1 | awk '{ print $$3 }' | sed 's/\"//g')
COMMIT_HASH := $(shell git rev-parse --short HEAD)

package-commit_hash-artifacts-for-deploy:
	rm -rf deploy
	mkdir -p deploy/prod
	mkdir -p deploy/dev
	./resources/package-deploy-artifacts.sh "sn_authd" ${COMMIT_HASH}
	./resources/package-deploy-artifacts.sh "sn_cli" ${COMMIT_HASH}
	./resources/package-deploy-artifacts.sh "sn_ffi" ${COMMIT_HASH}
	find deploy -name "*.tar.gz" -exec rm '{}' \;

package-version-artifacts-for-deploy:
	rm -rf deploy
	mkdir -p deploy/prod
	mkdir -p deploy/dev
	./resources/package-deploy-artifacts.sh "sn_authd" "${SN_AUTHD_VERSION}"
	./resources/package-deploy-artifacts.sh "sn_authd" "latest"
	./resources/package-deploy-artifacts.sh "sn_cli" "${SN_CLI_VERSION}"
	./resources/package-deploy-artifacts.sh "sn_cli" "latest"
	./resources/package-deploy-artifacts.sh "sn_ffi" "${SN_FFI_VERSION}"
	./resources/package-deploy-artifacts.sh "sn_ffi" "latest"
	find deploy -name "sn_ffi-*.tar.gz" -exec rm '{}' \;
