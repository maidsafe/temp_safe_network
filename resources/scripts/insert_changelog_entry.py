#!/usr/bin/env python
# The changelog file contains the changes for every version that's been released.
# For a release, we want to extract the changelog entry for a particular version and
# put that into the release description. This gets very painful in Bash because the entry
# contains newline characters.

import argparse
import sys
import toml


def get_crate_version(crate_name):
    manifest = toml.load(f"{crate_name}/Cargo.toml")
    return manifest["package"]["version"]


def get_changelog_entry(changelog_path, version):
    sn_changelog_content = ""
    with open(changelog_path, "r") as sn_changelog_file:
        sn_changelog_content = sn_changelog_file.read()
    start = sn_changelog_content.find("## v{version}".format(version=version))
    end = sn_changelog_content.find("## v", start + 10)
    return sn_changelog_content[start:end].strip()


def insert_changelog_entry(entry, pattern):
    if not entry.strip():
        entry = "No changes for this release"
    release_description = ""
    with open("release_description.md", "r") as file:
        release_description = file.read()
        release_description = release_description.replace(pattern, entry)
    with open("release_description.md", "w") as file:
        file.write(release_description)


def get_args():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--sn-updater", action="store_true", help="Generate changelog for sn_updater"
    )
    parser.add_argument(
        "--sn-interface",
        action="store_true",
        help="Generate changelog for sn_interface",
    )
    parser.add_argument(
        "--sn-fault-detection",
        action="store_true",
        help="Generate changelog for sn_fault_detection",
    )
    parser.add_argument(
        "--sn-comms", action="store_true", help="Generate changelog for sn_comms"
    )
    parser.add_argument(
        "--sn-client", action="store_true", help="Generate changelog for sn_client"
    )
    parser.add_argument(
        "--sn-api", action="store_true", help="Generate changelog for sn_api"
    )
    parser.add_argument(
        "--safenode", action="store_true", help="Generate changelog for safenode"
    )
    parser.add_argument(
        "--safe", action="store_true", help="Generate changelog for safe"
    )
    parser.add_argument(
        "--testnet", action="store_true", help="Generate changelog for testnet"
    )
    return parser.parse_args()


def main():
    args = get_args()
    if args.sn_updater:
        version = get_crate_version("sn_updater")
        changelog_entry = get_changelog_entry("sn_updater/CHANGELOG.md", version)
        insert_changelog_entry(changelog_entry, "__SN_UPDATER_CHANGELOG_TEXT__")
    if args.sn_interface:
        version = get_crate_version("sn_interface")
        changelog_entry = get_changelog_entry("sn_interface/CHANGELOG.md", version)
        insert_changelog_entry(changelog_entry, "__SN_INTERFACE_CHANGELOG_TEXT__")
    if args.sn_comms:
        version = get_crate_version("sn_comms")
        changelog_entry = get_changelog_entry("sn_comms/CHANGELOG.md", version)
        insert_changelog_entry(changelog_entry, "__SN_COMMS_CHANGELOG_TEXT__")
    if args.sn_fault_detection:
        version = get_crate_version("sn_fault_detection")
        changelog_entry = get_changelog_entry(
            "sn_fault_detection/CHANGELOG.md", version
        )
        insert_changelog_entry(changelog_entry, "__SN_FAULT_DETECTION_CHANGELOG_TEXT__")
    if args.sn_client:
        version = get_crate_version("sn_client")
        changelog_entry = get_changelog_entry("sn_client/CHANGELOG.md", version)
        insert_changelog_entry(changelog_entry, "__SN_CLIENT_CHANGELOG_TEXT__")
    if args.safenode:
        version = get_crate_version("sn_node")
        changelog_entry = get_changelog_entry("sn_node/CHANGELOG.md", version)
        insert_changelog_entry(changelog_entry, "__SAFENODE_CHANGELOG_TEXT__")
    if args.sn_api:
        version = get_crate_version("sn_api")
        changelog_entry = get_changelog_entry("sn_api/CHANGELOG.md", version)
        insert_changelog_entry(changelog_entry, "__SN_API_CHANGELOG_TEXT__")
    if args.safe:
        version = get_crate_version("sn_cli")
        changelog_entry = get_changelog_entry("sn_cli/CHANGELOG.md", version)
        insert_changelog_entry(changelog_entry, "__SAFE_CHANGELOG_TEXT__")
    if args.testnet:
        version = get_crate_version("sn_testnet")
        changelog_entry = get_changelog_entry("sn_testnet/CHANGELOG.md", version)
        insert_changelog_entry(changelog_entry, "__TESTNET_CHANGELOG_TEXT__")


if __name__ == "__main__":
    sys.exit(main())
