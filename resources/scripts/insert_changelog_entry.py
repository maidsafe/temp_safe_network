#!/usr/bin/env python
# The changelog file contains the changes for every version that's been released.
# For a release, we want to extract the changelog entry for a particular version and
# put that into the release description. This gets very painful in Bash because the entry
# contains newline characters.

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


def main(
    sn_updater_version,
    sn_interface_version,
    sn_fault_detection_version,
    sn_comms_version,
    sn_client_version,
    safenode_version,
    sn_api_version,
    safe_version,
):
    if sn_updater_version:
        changelog_entry = get_changelog_entry(
            "sn_updater/CHANGELOG.md", sn_updater_version
        )
        insert_changelog_entry(changelog_entry, "__SN_UPDATER_CHANGELOG_TEXT__")
    if sn_interface_version:
        changelog_entry = get_changelog_entry(
            "sn_interface/CHANGELOG.md", sn_interface_version
        )
        insert_changelog_entry(changelog_entry, "__SN_INTERFACE_CHANGELOG_TEXT__")
    if sn_comms_version:
        changelog_entry = get_changelog_entry("sn_comms/CHANGELOG.md", sn_comms_version)
        insert_changelog_entry(changelog_entry, "__SN_COMMS_CHANGELOG_TEXT__")
    if sn_fault_detection_version:
        changelog_entry = get_changelog_entry(
            "sn_fault_detection/CHANGELOG.md", sn_fault_detection_version
        )
        insert_changelog_entry(changelog_entry, "__SN_FAULT_DETECTION_CHANGELOG_TEXT__")
    if sn_client_version:
        changelog_entry = get_changelog_entry(
            "sn_client/CHANGELOG.md", sn_client_version
        )
        insert_changelog_entry(changelog_entry, "__SN_CLIENT_CHANGELOG_TEXT__")
    if safenode_version:
        changelog_entry = get_changelog_entry("sn_node/CHANGELOG.md", safenode_version)
        insert_changelog_entry(changelog_entry, "__SAFENODE_CHANGELOG_TEXT__")
    if sn_api_version:
        changelog_entry = get_changelog_entry("sn_api/CHANGELOG.md", sn_api_version)
        insert_changelog_entry(changelog_entry, "__SN_API_CHANGELOG_TEXT__")
    if safe_version:
        changelog_entry = get_changelog_entry(
            "sn_cli/CHANGELOG.md", safe_version
        )
        insert_changelog_entry(changelog_entry, "__SAFE_CHANGELOG_TEXT__")


if __name__ == "__main__":
    sn_updater_version = get_crate_version("sn_updater")
    sn_interface_version = get_crate_version("sn_interface")
    sn_fault_detection_version = get_crate_version("sn_fault_detection")
    sn_comms_version = get_crate_version("sn_comms")
    sn_client_version = get_crate_version("sn_client")
    safenode_version = get_crate_version("sn_node")
    sn_api_version = get_crate_version("sn_api")
    safe_version = get_crate_version("sn_cli")
    main(
        sn_updater_version,
        sn_interface_version,
        sn_fault_detection_version,
        sn_comms_version,
        sn_client_version,
        safenode_version,
        sn_api_version,
        safe_version,
    )
