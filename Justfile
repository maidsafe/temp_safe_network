#!/usr/bin/env just --justfile

release_repo := "maidsafe/safe_network"

build-release-artifacts arch:
  #!/usr/bin/env bash
  set -e

  arch="{{arch}}"
  supported_archs=(
    "x86_64-pc-windows-msvc"
    "x86_64-apple-darwin"
    "x86_64-unknown-linux-musl"
    "arm-unknown-linux-musleabi"
    "armv7-unknown-linux-musleabihf"
    "aarch64-unknown-linux-musl"
  )

  arch_supported=false
  for supported_arch in "${supported_archs[@]}"; do
    if [[ "$arch" == "$supported_arch" ]]; then
      arch_supported=true
      break
    fi
  done

  if [[ "$arch_supported" == "false" ]]; then
    echo "$arch is not supported."
    exit 1
  fi

  if [[ "$arch" == "x86_64-unknown-linux-musl" ]]; then
    if [[ "$(grep -E '^NAME="Ubuntu"' /etc/os-release)" ]]; then
      # This is intended for use on a fresh Github Actions agent
      sudo apt update -y
      sudo apt install -y musl-tools
    fi
    rustup target add x86_64-unknown-linux-musl
  fi

  rm -rf artifacts
  mkdir artifacts
  cargo clean
  if [[ $arch == arm* || $arch == armv7* || $arch == aarch64* ]]; then
    cargo install cross
    cross build --release --target $arch --bin safenode --features otlp
    cross build --release --target $arch --bin safe --features limit-client-upload-size,data-network
    cross build --release --target $arch --bin testnet
  else
    cargo build --release --target $arch --bin safenode --features otlp
    cargo build --release --target $arch --bin safe --features limit-client-upload-size,data-network
    cargo build --release --target $arch --bin testnet
  fi

  find target/$arch/release -maxdepth 1 -type f -exec cp '{}' artifacts \;
  rm -f artifacts/.cargo-lock

# Debugging target that builds an `artifacts` directory to be used with packaging targets.
#
# To use, download the artifact zip files from the workflow run and put them in an `artifacts`
# directory here. Then run the target.
make-artifacts-directory:
  #!/usr/bin/env bash
  set -e

  architectures=(
    "x86_64-pc-windows-msvc"
    "x86_64-apple-darwin"
    "x86_64-unknown-linux-musl"
    "arm-unknown-linux-musleabi"
    "armv7-unknown-linux-musleabihf"
    "aarch64-unknown-linux-musl"
  )
  cd artifacts
  for arch in "${architectures[@]}" ; do
    mkdir -p $arch/release
    unzip safe_network-$arch.zip -d $arch/release
    rm safe_network-$arch.zip
  done

package-release-assets bin_name version="":
  #!/usr/bin/env bash
  set -e

  architectures=(
    "x86_64-pc-windows-msvc"
    "x86_64-apple-darwin"
    "x86_64-unknown-linux-musl"
    "arm-unknown-linux-musleabi"
    "armv7-unknown-linux-musleabihf"
    "aarch64-unknown-linux-musl"
  )

  case "{{bin_name}}" in
    safe)
      crate="sn_cli"
      ;;
    safenode)
      crate="sn_node"
      ;;
    testnet)
      crate="sn_testnet"
      ;;
    *)
      echo "The only supported binaries are safe, safenode or testnet"
      exit 1
      ;;
  esac

  if [[ -z "{{version}}" ]]; then
    version=$(grep "^version" < $crate/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
  else
    version="{{version}}"
  fi

  rm -rf deploy/{{bin_name}}
  find artifacts/ -name {{bin_name}} -exec chmod +x '{}' \;
  for arch in "${architectures[@]}" ; do
    if [[ $arch == *"windows"* ]]; then bin_name="{{bin_name}}.exe"; else bin_name="{{bin_name}}"; fi
    zip -j {{bin_name}}-$version-$arch.zip artifacts/$arch/release/$bin_name
    tar -C artifacts/$arch/release -zcvf {{bin_name}}-$version-$arch.tar.gz $bin_name
  done

  mkdir -p deploy/{{bin_name}}
  mv *.tar.gz deploy/{{bin_name}}
  mv *.zip deploy/{{bin_name}}

generate-release-description:
  #!/usr/bin/env bash
  set -e
  rm -f release_description.md
    echo "Running first pass to generate release description"
    ./resources/scripts/get_release_description.sh > release_description.md
    echo "Running second pass to insert changelog"
    ./resources/scripts/insert_changelog_entry.py \
      --sn-updater \
      --sn-interface \
      --sn-fault-detection \
      --sn-comms \
      --sn-client \
      --sn-api \
      --safe \
      --safenode \
      --testnet
    echo "Release description now available in release_description.md file"

create-github-release:
  #!/usr/bin/env bash
  set -e

  source resources/scripts/output_versioning_info.sh
  gh release create $gh_release_tag_name \
    --title "$gh_release_name" --notes-file release_description.md --repo {{release_repo}}
  (
    echo "Uploading safe assets to release..."
    cd deploy/safe
    ls | xargs gh release upload $gh_release_tag_name --repo {{release_repo}}
  )
  (
    echo "Uploading safenode assets to release..."
    cd deploy/safenode
    ls | xargs gh release upload $gh_release_tag_name --repo {{release_repo}}
  )
  (
    echo "Uploading testnet assets to release..."
    cd deploy/testnet
    ls | xargs gh release upload $gh_release_tag_name --repo {{release_repo}}
  )

upload-release-assets-to-s3 bin_name:
  #!/usr/bin/env bash
  set -e

  case "{{bin_name}}" in
    safe)
      bucket="sn-cli"
      ;;
    safenode)
      bucket="sn-node"
      ;;
    testnet)
      bucket="sn-testnet"
      ;;
    *)
      echo "The only supported binaries are safe, safenode or testnet"
      exit 1
      ;;
  esac

  cd deploy/{{bin_name}}
  for file in *.zip *.tar.gz; do
    aws s3 cp "$file" "s3://$bucket/$file" --acl public-read
  done

publish-crates:
  #!/usr/bin/env bash
  set -e

  crates=(
    "sn_testnet"
    "sn_updater"
    "sn_interface"
    "sn_fault_detection"
    "sn_comms"
    "sn_node"
    "sn_client"
    "sn_api"
    "sn_cli"
  )

  for crate in "${crates[@]}" ; do
    version=$(grep "^version" < $crate/Cargo.toml | head -n 1 | awk '{ print $3 }' | sed 's/\"//g')
    response=$(curl -sS "https://crates.io/api/v1/crates/$crate")
    if echo "$response" | jq -r ".versions[].num" | grep -q "^${version}$"; then
      echo "$crate version $version has already been published"
    else
      (
        cd $crate && cargo publish
      )
    fi
  done
