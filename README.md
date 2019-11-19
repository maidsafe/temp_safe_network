# The SAFE API

| [MaidSafe website](https://maidsafe.net) | [SAFE Dev Forum](https://forum.safedev.org) | [SAFE Network Forum](https://safenetforum.org) |
|:----------------------------------------:|:-------------------------------------------:|:----------------------------------------------:|

## Table of contents

1. [Description](#description)
2. [The SAFE API](#the-safe-api-safe-api)
3. [The FFI layer](#the-ffi-layer-safe-ffi)
4. [The SAFE CLI](#the-safe-cli)
5. [The Authenticator daemon](#the-authenticator-daemon)
6. [Contributing](#contributing)
    * [Project board](#project-board)
    * [Issues](#issues)
    * [Commits and Pull Requests](#commits-and-pull-requests)
    * [Releases](#releases)
    * [Copyrights](#copyrights)
7. [Further Help](#further-help)
8. [License](#license)

## Description

In this repository you'll find all that's needed by any application which intends to connect and read/write data on [The SAFE Network](https://safenetwork.tech).

A Rust SAFE application can make use of the `safe-api` crate to be able to not only read/write data on the SAFE Network but also to send/receive authorisation requests to the SAFE Authenticator (see https://hub.safedev.org/discover for additional info of the Authenticator).

![SAFE app authorisation flow](misc/auth-flow-diagram.png)

In addition to the `safe-api` crate to be used by Rust applications, this repository contains the [safe-ffi](safe-ffi) library and a couple of applications ([safe-authd](safe-authd) and [safe-cli](safe-cli)) which are required depending on the type of SAFE application you are developing, use case, and/or if you are just a user of the SAFE Network willing to interact with it using a simple command line interface.

The following diagram depicts how each of the artifacts of this repository fit in the SAFE applications ecosystem. You can find more information about each of them further below in the next section of this document.

![SAFE API ecosystem](misc/safe-api-ecosystem.png)

## The SAFE API ([safe-api](safe-api))

The [safe-api](safe-api) is a Rust crate which exposes the SAFE API with all the functions needed to communicate with the SAFE Network and the SAFE Authenticator. If you are developing a Rust application for SAFE, this is all you need as a dependency from your app.

## The FFI layer ([safe-ffi](safe-ffi))

The [safe-ffi](safe-ffi) is a Rust crate exposing the same functions as the SAFE API (`safe-api`) but in the form of an interface which can be consumed from other programming languages like C, this is achieved by the use of the [Rust FFI feature](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#using-extern-functions-to-call-external-code).

Therefore, if you are developing a SAFE application using a different programming language than Rust, this is the crate you need to access the SAFE API. This crate also provides scripts to automatically generate the binding libraries for some languages like Java and C#.

## The SAFE CLI

The [safe-cli](safe-cli) is a Rust application which implements a CLI (Command Line Interface) for the SAFE Network.

![SAFE CLI](misc/safe-cli-animation.svg)

The SAFE CLI provides all the tools necessary to interact with the SAFE Network, including storing and browsing data of any kind, following links that are contained in the data and using their addresses on the network, using safecoin wallets, and much more. Using the CLI users have access to any type of operation that can be made on the SAFE Network and the data stored on it.

If you are just a SAFE user, or a system engineer creating automated scripts, this application provides you with all you need to interact with the SAFE Network. Please refer to [The SAFE CLI User Guide](safe-cli/README.md) to learn how to start using it.

## The Authenticator daemon

The [safe-authd](safe-authd) is a SAFE Authenticator implementation which runs in the background a daemon on Linux and Mac, or as a service in Windows platforms.

The SAFE Authenticator gives complete control over the type of access and permissions that are granted to the applications used by the SAFE users. Any application that is intending to write data on the Network on behalf of the user needs to get credentials which are authorised by the user, and the SAFE Authenticator is the component which facilitates such mechanism.

This application is normally shipped as part of the package of an Authenticator GUI, like the [SAFE Network Application](), and therefore SAFE users and SAFE app developers don't need it or worry about since the SAFE API already provides functions to interact with the `safe-authd`, and the SAFE CLI also has commands to do so.

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
