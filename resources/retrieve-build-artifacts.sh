#!/usr/bin/env bash

if [[ -z "$SAFE_API_BUILD_NUMBER" ]]; then
	echo "Please set SAFE_API_BUILD_NUMBER to a valid build number."
    exit 1
fi

if [[ -z "$SAFE_CLI_BRANCH" ]]; then
	echo "Please set SAFE_CLI_BRANCH to a valid branch or PR reference."
    exit 1
fi

S3_BUCKET=safe-jenkins-build-artifacts
declare -a types=("dev" "prod")
declare -a components=("safe-cli" "safe-ffi")

for component in "${components[@]}"; do
    for target in "$@"; do
        echo "Getting $component artifacts for $target"
        for type in "${types[@]}"; do
            mkdir -p "artifacts/$component/$type/$target/release"
            (
                cd "artifacts/$component/$type/$target/release"
                key="$SAFE_CLI_BRANCH-$SAFE_API_BUILD_NUMBER-$component-$type-$target.tar.gz"
                # If the key being queried doesn't exist this check prints out an ugly error message
                # that could potentially be confusing to people who are reading the logs.
                # It's not a problem, so the output is suppressed.
                aws s3api head-object \
                    --no-sign-request --region eu-west-2 \
                    --bucket "$S3_BUCKET" --key "$key" > /dev/null 2>&1
                rc=$?
                if [[ $rc == 0 ]]; then
                    echo "Retrieving $key"
                    aws s3 cp --no-sign-request --region eu-west-2 "s3://$S3_BUCKET/$key" .
                    tar -xvf "$key"
                    rm "$key"
                else
                    echo "$component $type has no artifacts for $target"
                fi
            )
        done
    done
done
