#!/usr/bin/env python
# The changelog file contains the changes for every version that's been released.
# For a release, we want to extract the changelog entry for a particular version and
# put that into the release description. This gets very painful in Bash because the entry
# contains newline characters.

import sys

def get_changelog_entry(version):
    sn_changelog_content = ""
    with open("sn/CHANGELOG.md", "r") as sn_changelog_file:
        sn_changelog_content = sn_changelog_file.read()
    start = sn_changelog_content.find("## v{version}".format(version=version))
    end = sn_changelog_content.find("## v", start + 10)
    return sn_changelog_content[start:end].strip()

def insert_changelog_entry(entry):
    release_description = ""
    with open("release_description.md", "r") as file:
        release_description = file.read()
        release_description = release_description.replace("__SN_CHANGELOG_TEXT__", entry)
    with open("release_description.md", "w") as file:
        file.write(release_description)

def main(version):
    sn_changelog_entry = get_changelog_entry(version)
    insert_changelog_entry(sn_changelog_entry)

if __name__ == "__main__":
    version = sys.argv[1]
    main(version)
