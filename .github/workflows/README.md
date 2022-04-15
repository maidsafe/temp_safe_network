# Github Actions Workflows

A place to document any of our workflows or processes that run on Github Actions.

## Release Process

After we changed our codebase to use a workspace with multiple crates, as opposed to a different repository for each crate, we had to change our automated release process.

There are a couple of workflows involved in this and there's also a personal access token with repo access, to enable the process to push a release commit and tags. We use the [smart-release](https://github.com/Byron/gitoxide/tree/main/cargo-smart-release) tool to bump version numbers and generate changelogs automatically, based on [conventional commits](https://www.conventionalcommits.org/en/v1.0.0-beta.2/). `smart-release` can also be used to generate Github Releases for each crate and push directly to main, but currently we've opted out of that. As of version `0.6.0`, `smart-release` seems to require you to supply the list of crates that have changed. Hopefully in the future you may just be able to run it at the root of the workspace and it can do its own work to figure out what's changed.

Here is the high-level overview of the process:

* A PR is merged into `main`.
* The `release.yml` workflow will run, but because there's no release commit, the job conditions won't be met, so nothing will happen.
* The `bump_version.yml` workflow will run:
    - The code is cloned using the PAT (this is important because for some reason you can't push to the repository unless it was also cloned with this token)
    - Git is configured to use the `Github Action` user for the release commit
    - A version bumping script runs and performs the following actions:
        + Use `smart-release` to determine which crates have changes.
        + Run `smart-release` using the list of crates that have changes. This creates a commit that has the automatically generated changelogs and the version bumps. Any dependent crates in the workspace will also have their references updated to any new versions. The tool also generates a tag for each crate that changed.
        + The generated commit is amended to have its message changed, based on the crates that have changed. For example, if both the `safe_network` and `sn_api` crates have changed, the commit message will be amended like so: `chore(release): safe_network-v0.50.0/sn_api-v0.51.0`.
        + The tags that were created are updated to point to the amended commit.
    - Both the release commit and the tags that were generated are pushed to the `main` branch
* The `release.yml` workflow runs and this time the latest commit is a `chore(release):` commit, so the first job condition is met:
    - Build all the code in the repository for all supported platforms and architectures
    - Generate a Github Release:
        + Get the versioning info for each crate being released.
        + Generate the description for the release, which includes the changelogs for each crate that was updated.
        + Create the Release.
        + Upload all the sn_node binaries as assets.
    - Publish the `safe_network` crate (if the commit message contains `safe_network`)
    - Publish the `sn_api` crate (if the commit message contains `sn_api`)

It's worth mentioning that on the final step that publishes the API crate, this is done using a script that has a retry loop. This is because it sometimes takes some time for the previously published `safe_network` crate to propagate.

## Running GitHub Actions on self-hosted runners

By default, GitHub actions are run on machines hosted by GitHub. These can be rather slow. This repository is setup to run workflows on self-hosted runners on AWS EC2 Instances. These instances scale up and down as required. Self-hosted runners are currently set up for the following operating systems:

#### Ubuntu

For Ubuntu, the process is straightforward. In the runs-on field, use `self-hosted-ubuntu`.

#### Windows

For Windows, the process is similar, but requires some additional setup.

- Use `self-hosted-windows` in the `runs-on` field.
- Use lionel1704/toolchain instead of actions-rs/toolchain. (until https://github.com/actions-rs/core/pull/216 is resolved)
- Add `shell: bash` to job steps with `run: ...` (Powershell is used by default so notations like `&&` and `./some_binary` are not supported.

## Adding New Crates to the Release Process

When a new crate is added to the workspace, it will either require publishing, or it will be an
internal crate. If it's the former, the crate will need to be incorporated into the release process.
To do so, follow these steps:

* Perform the first publish of the new crate manually:
    - Create the first CHANGELOG.md for the crate by copying one of the other ones: just put a
      manual entry at the top and delete all the other content from the file you copied.
    - Make sure the new crate has a README.md, even if it just has one sentence in it. You can't
      publish without it.
    - Run `cd <crate_name> && cargo publish --dry-run` to make sure the crate would be eligible for
      publishing. If it has any errors, fix them until you can get the dry run to pass.
    - Create a PR to merge in the new crate along with the new CHANGELOG and README.
    - Get that PR merged into `main`.
    - Tag the `main` branch using `crate_name-version_number`, e.g., `sn_dysfunction-0.1.0` and push
      that tag.
    - Run `cd <crate_name> && cargo publish` from `main`.
* If the new crate is a binary we want to release:
    - Update the `Makefile` as follows:
        + Update the `gha-build-x86_64-unknown-linux-musl` target to include the new binary
        + Update the `release-build` target to include the new binary
        + Create new `x-package-version-artifacts-for-release` target for the new binary
    - Update the `gh_release` job in the `release.yml` workflow:
        + Update the step to call the new packaging target along with the other ones
        + Add new steps for uploading the new binary to S3 by copying the other ones
* Output the version info for the new crate by updating `output_versioning_info.sh`. Update all the
  functions to include the new crate in the same fashion as the other ones.
* Include the crate in version bumping by updating `bump_version.sh`. Update all the functions to
  include the new crate in the same fashion as the other ones.
* Include the crate in the CHANGELOG for the Github Release:
    - Update `get_release_description.sh`:
        + Add a changelog section for the new crate.
        + If the crate is a binary:
            + Include the new checksums
            + Extend the script to pass the version number of the new binary and update the call to
              the script in the `gh_release` job
    - Update `insert_changelog_entry.py` to include the new crate in the same fashion as the other
      ones.
    - Update the `gh_release` job to add the new version number when `insert_changelog_entry.py`
      is called.
* Include the crate in the publishing process by adding a new `publish_x` job to the `release.yml`
  workflow. Copy one of the other jobs. Note that order is significant in the publishing process. If
  the crate being added has dependencies, the new job must run after the dependent crates have been
  published. Any crate with dependencies uses the `publish_crate.sh` script to check if the
  dependent crate has been published on crates.io. Make sure you update your newly added job to
  reference the correct dependent crate. No changes should be required for the publishing script
  itself.
* Update the `node install` command to select the correct version number for `sn_node`. Since the GH
  release tag name has the versions of all the crates in it, the correct version number has to be
  selected during the command. Update these:
    - Update `get_version_from_release_version` `sn_cli/src/operations/helpers.rs` to select the
      correct version number.
    - Update `get_version_from_release_version` `sn_cmd_test_utilities/src/lib.rs` to select the
      correct version number.

Unfortunately it's quite hard to test the changes without actually running the release process, so
it will probably take a few commits and release cycles before it works properly.

The initial manual publish is required for getting `smart-release` to work properly with the newly
added crate.
