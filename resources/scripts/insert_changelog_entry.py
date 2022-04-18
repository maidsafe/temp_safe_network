#!/usr/bin/env python
# The changelog file contains the changes for every version that's been released.
# For a release, we want to extract the changelog entry for a particular version and
# put that into the release description. This gets very painful in Bash because the entry
# contains newline characters.

import getopt
import sys

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

def main(sn_interface_version, sn_dysfunction_version, sn_version, sn_api_version, sn_cli_version):
    if sn_interface_version:
        sn_changelog_entry = get_changelog_entry("sn_interface/CHANGELOG.md", sn_interface_version)
        insert_changelog_entry(sn_changelog_entry, "__SN_INTERFACE_CHANGELOG_TEXT__")
    if sn_dysfunction_version:
        sn_changelog_entry = get_changelog_entry("sn_dysfunction/CHANGELOG.md", sn_dysfunction_version)
        insert_changelog_entry(sn_changelog_entry, "__SN_DYSFUNCTION_CHANGELOG_TEXT__")
    if sn_version:
        sn_changelog_entry = get_changelog_entry("sn/CHANGELOG.md", sn_version)
        insert_changelog_entry(sn_changelog_entry, "__SN_CHANGELOG_TEXT__")
    if sn_api_version:
        sn_api_changelog_entry = get_changelog_entry("sn_api/CHANGELOG.md", sn_api_version)
        insert_changelog_entry(sn_api_changelog_entry, "__SN_API_CHANGELOG_TEXT__")
    if sn_cli_version:
        sn_cli_changelog_entry = get_changelog_entry("sn_cli/CHANGELOG.md", sn_cli_version)
        insert_changelog_entry(sn_cli_changelog_entry, "__SN_CLI_CHANGELOG_TEXT__")

if __name__ == "__main__":
    sn_interface_version = ""
    sn_dysfunction_version = ""
    sn_version = ""
    sn_api_version = ""
    sn_cli_version = ""
    opts, args = getopt.getopt(
        sys.argv[1:],
        "",
        ["sn-interface-version=", "sn-dysfunction-version=", "sn-version=", "sn-api-version=", "sn-cli-version="]
    )
    for opt, arg in opts:
        if opt in "--sn-interface-version":
            sn_interface_version = arg
        if opt in "--sn-dysfunction-version":
            sn_dysfunction_version = arg
        elif opt in "--sn-version":
            sn_version = arg
        elif opt in "--sn-api-version":
            sn_api_version = arg
        elif opt in "--sn-cli-version":
            sn_cli_version = arg
    main(sn_interface_version, sn_dysfunction_version, sn_version, sn_api_version, sn_cli_version)
