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
