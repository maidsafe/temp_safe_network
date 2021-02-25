# The Safe API

| [MaidSafe website](https://maidsafe.net) | [Safe Dev Forum](https://forum.safedev.org) | [Safe Network Forum](https://safenetforum.org) |
|:----------------------------------------:|:-------------------------------------------:|:----------------------------------------------:|

## Table of contents

- [The Safe API](#the-safe-api)
  - [Table of contents](#table-of-contents)
  - [Description](#description)
  - [The API](#the-api)
  - [Further Help](#further-help)
  - [License](#license)
  - [Contributing](#contributing)

## Description

This crate provides all that's needed by any Rust application which intends to connect and read/write data on [The Safe Network](https://safenetwork.tech).

A Rust Safe application can make use of the `sn_api` crate to be able to not only read/write data on the Safe Network but also to send/receive authorisation requests to the Safe Authenticator (see https://hub.safedev.org/discover for additional info of the Authenticator). You can find more information about the Authenticator in MaidSafe reference implementation [sn_authd](sn_authd/README.md).

![Safe app authorisation flow](misc/auth-flow-diagram.png)

The following diagram depicts how each of the client side components fit in the Safe applications ecosystem.

![Safe API ecosystem](misc/safe-api-ecosystem.png)

## The API

The Safe API provides all the functions needed to communicate with the [The Safe Network](https://safenetwork.tech) and the [Safe Authenticator](sn_authd/README.md). If you are developing a Rust application for Safe, this is all you need as a dependency from your app.

There are currently three different APIs provided by this crate:
1. The [API for regular Safe applications](./src/api/app) which read and write data to Safe. This API exposes all the functions needed to manipulate data, with the additional `auth_app` to obtain a key-pair from `sn_authd`, and `connect` for connecting to Safe (providing a key-pair if write access is required by the app).
2. An [API for Authenticator apps](./src/api/authenticator), like `sn_authd` which makes use of this API. This is a small API which exposes functions to create, read and update a private container on Safe where to store the set of key-pairs the user administers for his/her apps, as well as some utilities to parse/generate messages that can be received/sent on an RPC mechanism like what `sn_authd` does with [JSON-RPC over QUIC](https://crates.io/crates/qjsonrpc). The container with key-pairs is stored on Safe at a location derived from a passphrase and password provided by the user.
3. [API to communicate with an Authenticator through JSON-RPC over QUIC](./src/api/authd_client). This can be used by apps which can manage an Authenticator app, as an example CLI uses this API to start/stop authd, to send a request to create a Safe, to allow/deny an app authorisation request, to lock/unlock a Safe, etc. The [`$ safe auth` commands](sn_cli/README.md#auth) act simply as the user interface for the `sn_authd` using this API to communicate with it.

## Further Help

You can discuss development-related questions on the [Safe Dev Forum](https://forum.safedev.org/).
If you are just starting to develop an application for the Safe Network, it's very advisable to visit the [Safe Network Dev Hub](https://hub.safedev.org) where you will find a lot of relevant information.

## License

This Safe Network library is dual-licensed under the Modified BSD ([LICENSE-BSD](LICENSE-BSD) https://opensource.org/licenses/BSD-3-Clause) or the MIT license ([LICENSE-MIT](LICENSE-MIT) https://opensource.org/licenses/MIT) at your option.

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [Guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
