# Safe Authenticator daemon

| [MaidSafe website](https://maidsafe.net) | [Safe Dev Forum](https://forum.safedev.org) | [Safe Network Forum](https://safenetforum.org) |
|:----------------------------------------:|:-------------------------------------------:|:----------------------------------------------:|

## Table of contents

- [Safe Authenticator daemon](#safe-authenticator-daemon)
  - [Table of contents](#table-of-contents)
  - [Description](#description)
  - [Download](#download)
  - [Build](#build)
  - [Launching the sn_authd](#launching-the-sn_authd)
    - [Start](#start)
    - [Stop](#stop)
    - [Restart](#restart)
  - [License](#license)
  - [Contributing](#contributing)

## Description

This crate implements a Safe Authenticator service which runs as a daemon (or as a service in Windows platforms).

The `sn_authd` exposes its services as a [JSON-RPC](https://www.jsonrpc.org/) interface, over [QUIC](https://en.wikipedia.org/wiki/QUIC), allowing applications and users to connect to create Safe Network accounts, log in using an existing account's credentials (passphrase and password), authorise applications which need to store data on the network on behalf of the user, as well as revoke permissions previously granted to applications.

It keeps in memory a list of authorisation requests pending of approval/denial, as well as the list of the registered subscribers that the notifications shall be sent to.

![authd software architecture](/misc/authd-software.png)

The following are few examples of JSON-RPC requests/responses exchanged with `sn_authd` over QUIC:
JSON-RPC call to log in:
```
Request: {
  "jsonrpc": "2.0",
  "method": "login",
  "params": ["<passphrase>", "<password>"],
  "id": 743533851
}

Response: {
  "jsonrpc": "2.0",
  "result": "Logged in successfully!",
  "id": 743533851
}
```

JSON-RPC call to obtain list of already authorised applications:
```
Request: {
  "jsonrpc": "2.0",
  "method": "authed-apps",
  "params": null,
  "id": 2294806509
}

Response: {
  "jsonrpc": "2.0",
  "result": [{
      "app_permissions": {
          "get_balance": true,
          "perform_mutations": true,
          "transfer_coins": true
      },
      "containers": {},
      "id": "net.maidsafe.cli",
      "name": "SAFE CLI",
      "own_container": false,
      "vendor": "MaidSafe.net Ltd"
  }],
  "id": 2294806509
}
```

When `sn_authd` sends a notification to each of the subscribers it also uses JSON-RPC over QUIC. The following is an example of a JSON-RPC message corresponding to an authorisation request notification sent from the `sn_authd` to a subscriber:
```
{
  jsonrpc: "2.0",
  method: "auth-req-notif",
  params: {
      "app_id": "net.maidsafe.cli",
      "app_name": "SAFE CLI",
      "app_permissions": {
          "get_balance": true,
          "perform_mutations": true,
          "transfer_coins": true
      },
      "app_vendor": "MaidSafe.net Ltd",
      "containers": {},
      "own_container": false,
      "req_id": 2039120779
  },
  id: 1195581342
}
```

## Download

The latest version of the Safe Authenticator daemon can be downloaded from the [releases page](https://github.com/maidsafe/sn_api/releases/latest). Once it's downloaded and unpacked, you can follow the steps in this guide by starting from the [Launching the sn_authd](#launching-the-sn_authd) section further down in this document.

If otherwise you prefer to build the Safe Authenticator daemon from source code, please follow the instructions in the next two section below.

## Build

In order to build this application from source code you need to make sure you have `rustc v1.44.0` (or higher) installed. Please take a look at this [notes about Rust installation](https://www.rust-lang.org/tools/install) if you need help with installing it. We recommend you install it with `rustup` which will install the `cargo` tool which this guide makes use of.

Once Rust and its toolchain are installed, run the following commands to clone this repository and build the `sn_authd` (the build process may take several minutes the first time you run it on this crate):
```shell
$ git clone https://github.com/maidsafe/sn_api.git
$ cd sn_api/sn_authd
$ cargo build
```

Once it's built you can find the `sn_authd` executable at `target/debug/`.

## Launching the sn_authd

The `sn_authd` can be launched with:
1. `cargo run -- <list of arguments/options>`
2. or directly with the executable generated: `./target/debug/sn_authd <list of arguments/options>`

As any other shell application, the `sn_authd` supports the `--help` argument which outputs a help message with information on the supported arguments and options, you can get this help message with:
```
$ sn_authd --help
```

This application supports only a few commands which are required to start it as a daemon, stop and restart it.

### Start

In order to start the Safe Authenticator daemon (`sn_authd`) we simply need to run the following command:
```shell
$ sn_authd start
Starting Safe Authenticator daemon (sn_authd)...
sn_authd started (PID: <pid>)
```

### Stop

To stop the Safe Authenticator daemon (`sn_authd`) we just run the following command (on Windows make sure you use the `sn_authd.exe` executable):
```shell
$ sn_authd stop
Stopping Safe Authenticator daemon (sn_authd)...
Success, sn_authd (PID: <pid>) stopped!
```

### Restart

We can also restart the Safe Authenticator daemon (`sn_authd`) if it's already running, with the following command (on Windows make sure you use the `sn_authd.exe` executable):
```shell
$ sn_authd restart
Stopping Safe Authenticator daemon (sn_authd)...
Success, sn_authd (PID: <pid>) stopped!
Starting Safe Authenticator daemon (sn_authd)...
```

## License
This Safe Network library is dual-licensed under the Modified BSD ([LICENSE-BSD](LICENSE-BSD) https://opensource.org/licenses/BSD-3-Clause) or the MIT license ([LICENSE-MIT](LICENSE-MIT) https://opensource.org/licenses/MIT) at your option.

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [Guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
