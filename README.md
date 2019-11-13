# The SAFE API

|Documentation|Linux/macOS/Windows|
|:-----------:|:-----------------:|
| [![Documentation](https://docs.rs/safe-api/badge.svg)](https://docs.rs/safe-api) | [![Build Status](https://travis-ci.com/maidsafe/safe-api.svg?branch=master)](https://travis-ci.com/maidsafe/safe-api) |

| [MaidSafe website](https://maidsafe.net) | [SAFE Dev Forum](https://forum.safedev.org) | [SAFE Network Forum](https://safenetforum.org) |
|:----------------------------------------:|:-------------------------------------------:|:----------------------------------------------:|

## Table of contents
1. [High level overview](#high-level-overview)
2. [The API](#the-api)
3. [The FFI layer](#the-ffi-layer)
3. [The CLI](#the-cli)
3. [The Authenticator daemon](#the-authenticator-daemon)
5. [Contributing](#contributing)
    * [Project board](#project-board)
    * [Issues](#issues)
    * [Commits and Pull Requests](#commits-and-pull-requests)
    * [Releases](#releases)
    * [Copyrights](#copyrights)
6. [Further Help](#further-help)
7. [License](#license)

## High level overview

The `safe-api` is a Rust library which is needed by any desktop application to connect and read/write data on [The SAFE Network](https://safenetwork.tech).

Any SAFE application can make use of this package to be able to not only read/write data on the SAFE Network but also to send/receive authorisation requests to the SAFE Authenticator (see https://hub.safedev.org/discover for additional info of the Authenticator).

![SAFE app authorisation flow](misc/auth-flow-diagram.png)

This repository contains a workspace with the following crates:
- safe-api: the Rust SAFE API
- safe-ffi: FFI interface layer which exposes all the `safe-api::Safe` Rust API
- safe-authd: the SAFE Authenticator daemon
- safe-cli: the SAFE CLI application

## The API

![SAFE API ecosystem](misc/safe-api-ecosystem.png)

## The FFI layer

The `safe-ffi` Rust library exposes an interface which can be consumed from other programming languages, this is achieved by the use of the [Rust FFI feature](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#using-extern-functions-to-call-external-code).

## The CLI

![The SAFE CLI User Guide](safe-cli/README.md)

TODO


## The Authenticator daemon

TODO


## Contributing

As an open source project we're excited to accept contributions to the code from outside of MaidSafe, and are striving to make that as easy and clean as possible.

This project adheres to the [Contributor Covenant](https://www.contributor-covenant.org/). By participating, you are expected to honor this code.

### Project board

GitHub project boards are used by the maintainers of this repository to keep track and organise development priorities: https://github.com/maidsafe/safe-api/projects

There could be one or more active project boards for a repository. One main project will be used to manage all tasks corresponding to the main development stream (`master` branch). A separate project may be used to manage each PoC and/or prototyping development, and each of them will track a dedicated development branch.

New features which imply big number of changes will be developed in a separate branch but tracked in the same main project board, re-basing it with `master` branch regularly, and fully testing the feature on its branch before it's merged onto the master branch after it was fully approved.

The main project contains the following Kanban columns to track the status of each development task:
- (optional) `Needs Triage`: new issues which need to be reviewed and evaluated to decide priority
- `To do`: task is has been scheduled to be done as part of current project plan
- `In Progress`: task is assigned to a person and it's in progress
- `Ready for QA`: the PR sent was approved by reviewer/s, merged, and ready for QA verification
- `Done`: PR associated to the issue was verified/tested (or task was completed by any other means)

### Issues

Issues should clearly lay out the problem, platforms experienced on, as well as steps to reproduce the issue.

This aids in fixing the issues but also quality assurance, to check that the issue has indeed been fixed.

Issues are labeled in the following way depending on its type:
- `bug`: the issue is a bug in the product
- `feature`: the issue is a new and inexistent feature to be implemented in the product
- `enhancement`: the issue is an enhancement to either an existing feature in the product, or to the infrastructure around the development process of the product
- `blocked`: the issue cannot be resolved as it depends on a fix in any of its dependencies
- `good first issue`: an issue considered more accessible for any developer trying to start contributing

### Commits and Pull Requests

We use [Conventional Commit](https://www.conventionalcommits.org/en/v1.0.0-beta.3/) style messages. (More usually [with a scope](https://www.conventionalcommits.org/en/v1.0.0-beta.3/#commit-message-with-scope)) for commits.

Commits should therefore strive to tackle one issue/feature, and code should be pre-linted before commit.

PRs should clearly link to an issue to be tracked on the project board. A PR that implements/fixes an issue is linked using one of the [GitHub keywords](https://help.github.com/articles/closing-issues-using-keywords). Although these type of PRs will not be added themselves to a project board (just to avoid redundancy with the linked issue). However, PRs which were sent spontaneously and not linked to any existing issue will be added to the project board and should go through the same process as any other tasks/issues.

Where appropriate, commits should contain tests for the code in question.

### Releases

The release process is triggered by the maintainers of the package, by bumping the package version according to the [SemVer](https://semver.org/) spec, and pushing a tag to have our CI scripts to automatically create the new version of `safe-api` crate and publish it at [https://crates.io/crates/safe-api](https://crates.io/crates/safe-api).

A new versions of the `safe-cli` application is also published for every new version of the `safe-api`. The `safe-cli` releases can be found at [https://github.com/maidsafe/safe-api/releases](https://github.com/maidsafe/safe-api/releases)

### Copyrights

Copyrights in the SAFE Network are retained by their contributors. No copyright assignment is required to contribute to this project.

## Further Help

You can discuss development-related questions on the [SAFE Dev Forum](https://forum.safedev.org/).
If you are just starting to develop an application for the SAFE Network, it's very advisable to visit the [SAFE Network Dev Hub](https://hub.safedev.org) where you will find a lot of relevant information.

## License

This SAFE Network library is dual-licensed under the Modified BSD ([LICENSE-BSD](LICENSE-BSD) https://opensource.org/licenses/BSD-3-Clause) or the MIT license ([LICENSE-MIT](LICENSE-MIT) https://opensource.org/licenses/MIT) at your option.
