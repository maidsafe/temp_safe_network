#!/usr/bin/env bash

if [[ -z "$SAFE_CLI_BUILD_NUMBER" ]]; then
	echo "Please set SAFE_CLI_BUILD_NUMBER to a valid build number."
    exit 1
fi

if [[ -z "$SAFE_CLI_BUILD_BRANCH" ]]; then
	echo "Please set SAFE_CLI_BUILD_BRANCH to a valid branch or PR reference."
    exit 1
fi

S3_BUCKET=safe-jenkins-build-artifacts
declare -a types=("mock" "real")
declare -a components=("safe-cli" "safe-ffi")

rm -rf artifacts
for component in "${components[@]}"; do
    for target in "$@"; do
        echo "Getting $component artifacts for $target"
        for type in "${types[@]}"; do
            mkdir -p "artifacts/$type/$target/release"
            (
                cd "artifacts/$type/$target/release"
                key="$SAFE_CLI_BUILD_BRANCH-$SAFE_CLI_BUILD_NUMBER-$component-$target.tar.gz"
                if [[ "$type" == "mock" ]]; then
                    key="$SAFE_CLI_BUILD_BRANCH-$SAFE_CLI_BUILD_NUMBER-$component-$target-dev.tar.gz"
                fi
                aws s3api head-object --bucket "$S3_BUCKET" --key "$key"
                rc=$?
                if [[ $rc == 0 ]]; then
                    echo "Retrieving $key"
                    aws s3 cp --no-sign-request --region eu-west-2 "s3://$S3_BUCKET/$key" .
                    tar -xvf "$key"
                    rm "$key"
                fi
            )
        done
    done
done
