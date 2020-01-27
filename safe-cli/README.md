# SAFE CLI

| [MaidSafe website](https://maidsafe.net) | [SAFE Dev Forum](https://forum.safedev.org) | [SAFE Network Forum](https://safenetforum.org) |
|:----------------------------------------:|:-------------------------------------------:|:----------------------------------------------:|

## Table of contents

1. [Description](#description)
2. [Download](#download)
3. [Build](#build)
4. [Using the CLI](#using-the-cli)
  - [Auth](#auth)
    - [The Authenticator daemon (authd)](#the-authenticator-daemon-authd)
    - [Auth install](#auth-install)
    - [Auth start](#auth-start)
    - [Auth status](#auth-status)
    - [Auth create-acc](#auth-create-acc)
    - [Auth login](#auth-login)
    - [Auth reqs](#auth-reqs)
    - [Auth allow-deny](#auth-allowdeny)
    - [Auth update](#auth-update)
  - [The interactive shell](#the-interactive-shell)
  - [SafeKeys](#safekeys)
    - [Create](#safekeys-creation)
    - [Balance](#safekeys-balance)
    - [Transfer](#safekeys-transfer)
  - [Key pair](#key-pair)
  - [Wallet](#wallet)
    - [Create](#wallet-creation)
    - [Insert](#wallet-insert)
    - [Balance](#wallet-balance)
    - [Transfer](#wallet-transfer)
  - [Files](#files)
    - [Put](#files-put)
    - [Sync](#files-sync)
    - [Add](#files-add)
  - [Xorurl](#xorurl)
    - [Decode](#xorurl-decode)
  - [Cat](#cat)
  - [NRS](#nrs-name-resolution-system)
    - [Create](#nrs-create)
    - [Add](#nrs-add)
    - [Remove](#nrs-remove)
  - [SAFE-URLs](#safe-urls)
  - [Dog](#dog)
  - [Networks](#networks)
    - [Run a local vault](#run-a-local-vault)
    - [Switch networks](#switch-networks)
  - [Shell Completions](#shell-completions)
    - [Bash Completions](#bash-completions)
    - [Windows Powershell Completions](#windows-powershell-completions)
  - [Update](#update)
5. [Further Help](#further-help)
6. [License](#license)

## Description

This crate implements a CLI (Command Line Interface) for the SAFE Network.

The SAFE CLI provides all the tools necessary to interact with the SAFE Network, including storing and browsing data of any kind, following links that are contained in the data and using their addresses on the network, using safecoin wallets, and much more. Using the CLI users have access to any type of operation that can be made on the SAFE Network and the data stored on it, allowing them to also use it for automated scripts and piped chain of commands.

## Download

The latest version of the SAFE CLI can be downloaded from the [releases page](https://github.com/maidsafe/safe-api/releases/latest). Once it's downloaded and unpacked, you can follow the steps in this User Guide by starting from the [Using the CLI](#using-the-cli) section below in this document.

If otherwise you prefer to build the SAFE CLI from source code, please follow the instructions in the next two section below.

## Build

In order to build this CLI from source code you need to make sure you have `rustc v1.38.0` (or higher) installed. Please take a look at this [notes about Rust installation](https://www.rust-lang.org/tools/install) if you need help with installing it. We recommend you install it with `rustup` which will install the `cargo` tool which this guide makes use of.

Once Rust and its toolchain are installed, run the following commands to clone this repository and build the `safe-cli` (the build process may take several minutes the first time you run it on this crate):
```shell
$ git clone https://github.com/maidsafe/safe-api.git
$ cd safe-api
$ cargo build
```

Since this project has a cargo workspace with the `safe-cli` as the default crate, when building from the root location will build the SAFE CLI. Once it's built you can find the `safe` executable at `target/debug/`.

### Using the Mock or Non-Mock SAFE Network

By default, the `safe-cli` is built with [Non-Mock libraries](https://github.com/maidsafe/safe_client_libs/wiki/Mock-vs.-non-mock). If you are intending to use it with the `Mock` network you'll need to specify the `mock-network` feature in every command you run with `cargo`, e.g. to build it for the `Mock` network you can run:
```
$ cargo build --features mock-network
```

Keep in mind that if you run the `safe-cli` with `cargo run`, you also need to make sure to set the `mock-network` feature if you want to use the `Mock` network, e.g. with the following command the `safe-cli` will try to create a `SafeKey` with test-coins on the `Mock` network:
```
$ cargo run --features mock-network -- keys create --test-coins
```

## Using the CLI

Right now the CLI is under active development. Here we're listing commands ready to be tested.

The base command, if built is `$ safe`, or all commands can be run via `$ cargo run -- <command>`.

Various global flags are available:

```
--dry-run                  Dry run of command. No data will be written. No coins spent.
-h, --help                 Prints help information
--json                     Sets JSON as output serialisation format (alias of '--output json')
-V, --version              Prints version information
-o, --output <output_fmt>  Output data serialisation: [json, jsoncompact, yaml]
--xorurl <xorurl_base>     Base encoding to be used for XOR-URLs generated. Currently supported: base32z
                           (default), base32 and base64
```

#### `--help`

All commands have a `--help` function which lists args, options and subcommands.

### Auth

The CLI is just another client SAFE application, therefore it needs to be authorised by the user to gain access to the SAFE Network on behalf of the user. The `auth` command allows us to obtain such authorisation from the account owner (the user) via the SAFE Authenticator.

This command simply sends an authorisation request to the Authenticator available, e.g. the `safe-authd` daemon (see further bellow for explanation of how to run it), and it then stores the authorisation response (credentials) in the user's `$XDG_DATA_DIRS/safe-cli/credentials` file. Any subsequent CLI command will read this file to obtain the credentials and connect to the SAFE Network for the corresponding operation.

#### The Authenticator daemon (authd)

In order to be able to allow any SAFE application to connect to the Network and have access to your data, we need to start the SAFE Authenticator daemon (authd). This application exposes an interface as a [QUIC (Quick UDP Internet Connections)](https://en.wikipedia.org/wiki/QUIC) endpoint, which SAFE applications will communicate with to request for access permissions. These permissions need to be reviewed by the user and approved, which can be all done with the SAFE CLI as we'll see in this guide.

The SAFE Authenticator, which runs as a daemon or as a service in Windows platforms, can be started and managed with the SAFE CLI if the `safe-authd`/`safe-authd.exe` binary is properly installed in the system.

#### Auth install

Downloading and installing the Authenticator daemon is very simple:
```shell
$ safe auth install
Latest release found: safe-authd v0.0.3
Downloading https://safe-api.s3.eu-west-2.amazonaws.com/safe-authd-0.0.3-x86_64-unknown-linux-gnu.tar.gz...
[00:00:25] [========================================] 6.16MB/6.16MB (0s) Done
Installing safe-authd binary at ~/.safe/authd ...
Done!
```

**If you on a Windows platform**, the CLI requires administrator permissions to install it, so please open a console with administrator permissions (you can look at [this guide which explains how to do it on Windows 10](https://www.intowindows.com/command-prompt-as-administrator-in-windows-10/)), and then run the install command:
```shell
> safe auth install
Latest release found: safe-authd v0.0.3
Downloading https://safe-api.s3.eu-west-2.amazonaws.com/safe-authd-0.0.3-x86_64-pc-windows-msvc.zip...
[00:00:19] [========================================] 4.3MB/4.3MB (0s) Done
Installing safe-authd binary at ~/.safe/authd ...
Done!
Installing SAFE Authenticator (safe-authd) as a Windows service...
The safe-authd service (<'safe-authd.exe' path>) was just installed successfully!
```

Note that in the case of a Windows platform, the command not only downloads the binary to the system, but it also takes care of setting it up as a Windows service so it's ready to then be started.

#### Auth start

In order to start the `SAFE Authenticator daemon (safe-authd)` so it can start receiving requests we simply need to run the following command:
```shell
$ safe auth start
Starting SAFE Authenticator daemon (safe-authd)...
```

Again, **if you are on a Windows platform**, the CLI requires administrator permissions to be able to start the safe-authd service, so please open a console with administrator permissions (you can look at [this guide which explains how to do it on Windows 10](https://www.intowindows.com/command-prompt-as-administrator-in-windows-10/)), and then run the following commands:
```shell
> safe auth start
Starting SAFE Authenticator service (safe-authd) from command line...
safe-authd service started successfully!
```

#### Auth status

Once we started the `authd`, it should be running in the background and ready to receive requests, we can send an status request to check it's up and running:
```shell
$ safe auth status
Sending request to authd to obtain an status report...
+------------------------------------------+-------+
| SAFE Authenticator status                |       |
+------------------------------------------+-------+
| Authenticator daemon version             | 0.0.3 |
+------------------------------------------+-------+
| Logged in to a SAFE account?             | No    |
+------------------------------------------+-------+
| Number of pending authorisation requests | 0     |
+------------------------------------------+-------+
| Number of notifications subscribers      | 0     |
+------------------------------------------+-------+
```

##### Vault must be running

Before we can create an account (or login) we need either a `SAFE Vault` running locally, or the configuration file to connect to a remote vault/network in the right place. Please refer to the [Networks section](#networks) in this guide if you need help to set this up.

Once you have made sure you have a vault running locally, or have the connection file for a remote vault/network in place, you can proceed with the next steps.

#### Auth create-acc

Since we now have our SAFE Authenticator running and ready to accept requests, we can start interacting with it by using others SAFE CLI `auth` subcommands.

In order to create a SAFE Network account we need some `safecoins` to pay with. Since this is still under development, we can have the CLI to generate some test-coins and use them for paying the cost of creating an account. We can do so by passing `--test-coins` flag to the `create-acc` subcommand. The CLI will request us to enter a passphrase and password for the new account to be created:
```shell
$ safe auth create-acc --test-coins
Passphrase:
Password:
Creating a SafeKey with test-coins...
Sending account creation request to authd...
Account was created successfully!
SafeKey created and preloaded with test-coins. Owner key pair generated:
Public Key = a42c991eb33c1e2530205bc726eba0279e151a334ba8dcd7212b131abb210145bc859ae5f6f5d4ce63ece54c64fe8792
Secret Key = 5cc0951bb95be85dec3f0358ddb40570d0e045b3ff0007562af9b5c9162f2518
```

Alternatively, if we own some safecoins on a `SafeKey` already (see [`SafeKeys` section](#safekeys) for details about `SafeKey`s), we can provide the corresponding secret key to the safe CLI to use it for paying the cost of creating the account, as well as setting it as the default `SafeKey` for the account being created:
```shell
$ safe auth create-acc
Passphrase:
Password:
Enter SafeKey's secret key to pay with:
Sending account creation request to authd...
Account was created successfully!
```

#### Auth login

Once you have a SAFE account created, we can login:
```shell
$ safe auth login
Passphrase:
Password:
Sending login action request to authd...
Logged in successfully
```

If we again send an status report request to `authd`, it should now show that it's logged in to a SAFE account:
```shell
$ safe auth status
Sending request to authd to obtain an status report...
+------------------------------------------+-------+
| SAFE Authenticator status                |       |
+------------------------------------------+-------+
| Authenticator daemon version             | 0.0.3 |
+------------------------------------------+-------+
| Logged in to a SAFE account?             | Yes   |
+------------------------------------------+-------+
| Number of pending authorisation requests | 0     |
+------------------------------------------+-------+
| Number of notifications subscribers      | 0     |
+------------------------------------------+-------+
```

The SAFE Authenticator is now ready to receive authorisation requests from any SAFE application, including the SAFE CLI which needs to also get permissions to perform any data operations on behalf of our account.

#### Auth reqs

Now that the Authenticator is running and ready to authorise applications, we can try to authorise the CLI application.

In a normal scenario, an Authenticator GUI would be using `authd` as its backend process, e.g. the [SAFE Network Application](https://github.com/maidsafe/safe-network-app) provides such a GUI to review authorisation requests and allow the permissions requested to be granted.

For the purpose of making this guide self contained with the SAFE CLI application, we will now use also the CLI on a second console to review and allow/deny authorisation requests.

Let's first send an authorisation request from current console by simply invoking the `auth` command with no subcomands:
```shell
$ safe auth
Authorising CLI application...
```

The CLI application is now waiting for an authorisation response from the `authd`.

We can now open a second console which we'll use to query `authd` for pending authorisation requests, and also to allow/deny them (remember the following steps wouldn't be needed if we had any other Authenticator UI running, like the `SAFE Network App`).

Once we have a second console, we can start by fetching from `authd` the list of authorisation requests pending for approval/denial:
```shell
$ safe auth reqs
Requesting list of pending authorisation requests from authd...
+--------------------------------+------------------+----------+------------------+-------------------------+
| Pending Authorisation requests |                  |          |                  |                         |
+--------------------------------+------------------+----------+------------------+-------------------------+
| Request Id                     | App Id           | Name     | Vendor           | Permissions requested   |
+--------------------------------+------------------+----------+------------------+-------------------------+
| 584798987                      | net.maidsafe.cli | SAFE CLI | MaidSafe.net Ltd | Own container: false    |
|                                |                  |          |                  | Transfer coins: true    |
|                                |                  |          |                  | Mutations: true         |
|                                |                  |          |                  | Read coin balance: true |
|                                |                  |          |                  | Containers: None        |
+--------------------------------+------------------+----------+------------------+-------------------------+
```

We see there is one authorisation request pending for approval/denial, which is the one requested by the CLI application from the other console.

#### Auth allow/deny

In order to allow any pending authorisation request we use its request ID (e.g. '584798987' from above), the `authd` will then proceed to send a response back to the CLI with the corresponding credentials it can use to connect directly with the Network:
```shell
$ safe auth allow 584798987
Sending request to authd to allow an authorisation request...
Authorisation request was allowed successfully
```

Note we could have otherwise decided to deny this authorisation request and invoke `$ safe auth deny 584798987` instead, but let's allow it so we can continue with the next steps of this guide.

If we now switch back to our previous console, the one where we sent the authorisation request with `$ safe auth` command from, we will see the SAFE CLI receiving the response from `authd`. You should see in that console a message like the following:
```shell
SAFE CLI app was successfully authorised
Credentials were stored in <home directory>/.local/share/safe-cli/credentials
```

We are now ready to start using the CLI to operate with the network, via its commands and supported operations!.

##### Self authorising the CLI application

It could be the case the SAFE CLI is the only SAFE application that the user is intended to use to interact with the SAFE Network. In such a case authorising the CLI application as explained above (when there is no other UI for the `authd`) using another instance of the CLI in a second console is not that comfortable.

Therefore there is an option which allows the SAFE CLI to automatically self authorise when the user logs in using the CLI, which is as simply as:
```shell
$ safe auth login --self-auth
Passphrase:
Password:
Sending login action request to authd...
Logged in successfully
Authorising CLI application...
SAFE CLI app was successfully authorised
Credentials were stored in <home directory>/.local/share/safe-cli/credentials
```

#### Auth update

The Authenticator binary (`safe-authd`/`safe-authd.exe`) can be updated to the latest available version using the CLI. Running `safe auth update`, the application will check if a newer release is available on [Amazon S3](https://safe-api.s3.eu-west-2.amazonaws.com). After prompting to confirm if you want to take the latest version, it will be downloaded and the safe-authd binary will be updated.

After the safe-authd was updated, you'll need to restart it to start using new version:
```shell
$ safe auth restart
Stopping SAFE Authenticator daemon (safe-authd)...
Success, safe-authd (PID: <pid>) stopped!
Starting SAFE Authenticator daemon (safe-authd)...
```

### The interactive shell

When the CLI is invoked without any command, it enters into an interactive shell, which allows the user to run commands within a shell:
```shell
$ safe

Welcome to SAFE CLI interactive shell!
Type 'help' for a list of supported commands
Pass '--help' flag to any top level command for a complete list of supported subcommands and arguments
Type 'quit' to exit this shell. Enjoy it!

>
```

The interactive shell supports all the same commands and operations that can be performed in the command line. E.g., we can use the `auth status` command to retrieve an status report from the `authd`:
```shell
> auth status
Sending request to authd to obtain an status report...
+------------------------------------------+-------+
| SAFE Authenticator status                |       |
+------------------------------------------+-------+
| Authenticator daemon version             | 0.0.3 |
+------------------------------------------+-------+
| Logged in to a SAFE account?             | No    |
+------------------------------------------+-------+
| Number of pending authorisation requests | 0     |
+------------------------------------------+-------+
| Number of notifications subscribers      | 0     |
+------------------------------------------+-------+
```

As you can see, the commands operate in an analogous way as when they are invoked outside of the interactive shell. Although there are some operations which are only possible when they are executed from the interactive shell, one nice example is the possibility to subscribe to receive authorisation request notifications, let's see how that works.

In the previous section we've used the `safe auth reqs` command to obtain a list of the authorisation requests which are awaiting for approval/denial. We could instead use the interactive shell to subscribe it as an endpoint to receive notifications when this authorisation requests are sent to the `authd`:
```shell
> auth subscribe
Sending request to subscribe...
Subscribed successfully
Keep this shell session open to receive the notifications
```

This is telling us that as long as we keep this session of the interactive shell open, we will be notified of any new authorisation request, such notifications are being sent by the `authd` to our interactive shell session. Thus if we have any other SAFE app which is sending an authorisation request to `authd`, e.g. the SAFE Browser, a `safe auth` command invoked from another instance of the CLI, etc., we will be notified by the interactive shell:
```shell
>
A new application authorisation request was received:
+------------+------------------+---------+---------+-------------------------+
| Request Id | App Id           | Name    | Vendor  | Permissions requested   |
+------------+------------------+---------+---------+-------------------------+
| 754801191  | net.maidsafe.cli | Unknown | Unknown | Own container: false    |
|            |                  |         |         | Transfer coins: true    |
|            |                  |         |         | Mutations: true         |
|            |                  |         |         | Read coin balance: true |
|            |                  |         |         | Containers: None        |
+------------+------------------+---------+---------+-------------------------+
You can use "auth allow"/"auth deny" commands to allow/deny the request respectively, e.g.: auth allow 754801191
Press Enter to continue
```

The notification message contains the same information we can obtain with `safe auth reqs` command. We can now do the same as before and allow/deny such a request using its ID, in this case '754801191':
```shell
> auth allow 754801191
Sending request to authd to allow an authorisation request...
Authorisation request was allowed successfully
```

The interactive shell will be expanded to support many more operations, and especially to cover the use cases which are not possible to cover with the non-interactive shell, like the use case we've seen of receiving notifications from `authd`.

It enables the possibility to also have a state in the session, e.g. allowing the user to set a wallet to be used for all operation within that session instead of using the default wallet from the account, ...or several other use cases and features we'll be adding as we move forward in its development.

### SafeKeys

`SafeKey` management allows users to generate sign/encryption key pairs that can be used for different type of operations, like choosing which sign key to use for uploading files (and therefore paying for the storage used), or signing a message posted on some social application when a `SafeKey` is linked from a public profile (e.g. a WebID/SAFE-ID), or even for encrypting messages that are privately sent to another party so it can verify the authenticity of the sender.

Users can record `SafeKey`s in a `Wallet` (see further below for more details about `Wallet`s), having friendly names to refer to them, but they can also be created as throw away `SafeKey`s which are not linked from any `Wallet`, container, or any other type of data on the network.

Note that even though the key pair is automatically generated by the CLI, `SafeKey`s don’t hold the secret key on the network but just the public key, and `SafeKey`s can optionally have a safecoin balance attached to it. Thus `SafeKey`s can also be used for safecoin transactions. In this sense, a `SafeKey` can be compared to a Bitcoin address, it has a coin balance associated to it, such balance can be only queried using the secret key, and in order to spend its balance the corresponding secret key needs to be provided in the `transfer` request as well. The secret key can be provided by the user, or retrieved from a `Wallet`, at the moment of creating the transaction (again, see the [`Wallet` section](#wallet) below for more details).

#### SafeKeys Creation

To generate a key pair and create a new `SafeKey` on the network, the secret key of another `SafeKey` is needed to pay for storage costs:
```shell
$ safe keys create --pay-with <secret key>
```

But we can also create a `SafeKey` with test-coins to begin with (this is temporary and only for testing until farming is available):
```shell
$ safe keys create --test-coins --preload 15.342
New SafeKey created at: "safe://bbkulcbnrmdzhdkrfb6zbbf7fisbdn7ggztdvgcxueyq2iys272koaplks"
Preloaded with 15.342 coins
Key pair generated:
Public Key = b62c1e4e3544a1f64212fca89046df98d998ea615e84c4348c4b5fd29c07ad52a970539df819e31990c1edf09b882e61
Secret Key = c4cc596d7321a3054d397beff82fe64f49c3896a07a349d31f29574ac9f56965
```

Once we have some `SafeKey`s with some test-coins we can use them to pay for the creation of a SAFE Account (using the [SAFE Authenticator](https://github.com/maidsafe/safe-authenticator-cli)), or to pay for the creation of new `SafeKey`s. Thus if we use the `SafeKey` we just created with test-coins we can create a second `SafeKey`:
```shell
$ safe keys create --preload 8.15 --pay-with c4cc596d7321a3054d397beff82fe64f49c3896a07a349d31f29574ac9f56965
New SafeKey created at: "safe://bbkulcbf2uuqwawvuonevraqa4ieu375qqrdpwvzi356edwkdjhwgd4dum"
Preloaded with 8.15 coins
Key pair generated:
Public Key = 9754a42c0b568e692b10401c4129bff61088df6ae51bef883b28693d8c3e0e8ce23054e236bd64edc45791549ef60ce1
Secret Key = 2f211ad4606c716c2c2965e8ea2bd76a63bfc5a5936b792cda448ddea70a031c
```

In this case, the new `SafeKey` is preloaded with coins which are transferred from the `SafeKey` we pay the operation with. In next section we'll see how to check the coins balance of them.

If we omit the `--pay-with` argument from the command above, or from any other command which supports it, the CLI will make use of the default `SafeKey` which is linked from the account for paying the costs of the operation. Upon the creation of a SAFE Account, a default `SafeKey` is linked to it and used for paying the costs incurred in any operations made by the applications that have been authorised by the owner of that account, like it's the case of the `safe-cli` application. Currently it's not possible to change the default `SafeKey` linked from an account, but it will be possible with the `safe-cli` in the near future.

Other optional args that can be used with `keys create` sub-command are:
```
--pk <pk>            Don't generate a key pair and just use the provided public key
--preload <preload>  Preload the SafeKey with a coin balance
```

#### SafeKey's Balance

We can retrieve a given `SafeKey`'s balance simply using its secret key, which we can pass to `keys balance` subcommand with `--sk <secret key>` argument, or we can enter it when the CLI prompts us.

We can optionally also pass the `SafeKey`'s XorUrl to have the CLI to verify they correspond to each other, i.e. if the `SafeKey`'s XorUrl is provided, the CLI will check if it corresponds to the public key derived from the passed secret key, and throw an error in it doesn't.

The target `SafeKey`'s secret key can be passed as an argument (or it will be retrieved from `stdin`), let's check the balance of the `SafeKey` we created in previous section:
```bash
$ safe keys balance
Enter secret key corresponding to the SafeKey to query the balance from: c4cc596d7321a3054d397beff82fe64f49c3896a07a349d31f29574ac9f56965
SafeKey's current balance: 15.342000000
```

#### SafeKeys Transfer

We now have a `SafeKey` with a positive balance, we can transfer `--from` a `SafeKey` (using its secret key), an `<amount>` of safecoins, `--to` another `Wallet` or `SafeKey`. The destination `Wallet`/`SafeKey` can be passed as an argument with `--to`, or it will be read from `stdin`. If we omit the `--from` argument, the Account's default `SafeKey` will be used as the source of funds for the transaction.

```shell
$ safe keys transfer <amount> --from <source SafeKey secret key> --to <destination Wallet/SafeKey URL>
```
E.g.:
```shell
$ safe keys transfer 1.519 --from c4cc596d7321a3054d397beff82fe64f49c3896a07a349d31f29574ac9f56965 --to safe://bbkulcbf2uuqwawvuonevraqa4ieu375qqrdpwvzi356edwkdjhwgd4dum
Success. TX_ID: 12584479662656231449
```

### Key-pair

There are some scenarios that being able to generate a sign/encryption key-pair, without creating and/or storing a `SafeKey` on the network, is required.

As an example, if we want to have a friend to create a `SafeKey` for us, and preload it with some coins, we can generate a key-pair, and share with our friend only the public key so he/she can generate the `SafeKey` to be owned by it (this is where we can use the `--pk` argument on the `keys create` sub-command).

Thus, let's see how this use case would work. First we create a key-pair:
```shell
$ safe keypair
Key pair generated:
Public Key = b2371df48684dc9456988f45b56d7640df63895fea3d7cee45c79b26ba268d259b864330b83fa28669ab910a1725b833
Secret Key = 62e323615235122f7e20c7f05ddf56c5e5684853d21f65fca686b0bfb2ed851a
```

We now take note of both the public key, and the secret key. Now, we only share the public key with our friend, who can use it to generate a `SafeKey` to be owned by it and preload it with some test-coins:
```shell
$ safe keys create --test-coins --preload 64.24 --pk b2371df48684dc9456988f45b56d7640df63895fea3d7cee45c79b26ba268d259b864330b83fa28669ab910a1725b833
New SafeKey created at: "safe://hodby8y3qgina9nqzxmsoi8ytjfh6gwnia7hdupo49ibt8yy3ytgdq"
Preloaded with 64.24 coins
```

Finally, our friend gives us the XOR-URL of the `SafeKey` he/she has created for us. We own the balance this `SafeKey` holds since we have the secret key associated to it. Therefore we can now use the `SafeKey` for any operation, like creating an account with safe_auth CLI to then be able to store data on the Network.

### Wallet

A `Wallet` is a specific type of Container on the network, holding a set of spendable safecoin balances.

A `Wallet` effectively contains links to `SafeKey`s which have safecoin balances attached to them, but the `Wallet` also stores the secret keys needed to spend them, and this is why each of these links/items in a `Wallet` is called a `spendable balances`. `Wallet`s are stored encrypted and only accessible to the owner by default.

Each of these links to `SafeKey`s (the spendable balances) can have a friendly name provided by the user, and these friendly names can then be used in different types of operations. E.g. one spendable balance in a `Wallet` can be named 'for-night-outs', while another one is named 'to-pay-the-rent', so when using the `Wallet` you could provide those names to the command in order to choose which spendable balance to use for the operation.

There are several sub-commands that can be used to manage the `Wallet`s with the `safe wallet` command (those commented out are not yet implemented):

```
SUBCOMMANDS:
    balance       Query a Wallet's total balance
    # check-tx    Check the status of a given transaction
    create        Create a new Wallet
    help          Prints this message or the help of the given subcommand(s)
    insert        Insert a spendable balance into a Wallet
    # sweep       Move all coins within a Wallet to a second given Wallet or Key
    transfer      Transfer safecoins from one Wallet, SafeKey or pk, to another.
```

#### Wallet Creation

```shell
USAGE:
    safe wallet create [FLAGS] [OPTIONS]

FLAGS:
    -n, --dry-run       Dry run of command. No data will be written. No coins spent
    -h, --help          Prints help information
        --no-balance    If true, do not create a spendable balance
        --json          Sets JSON as output serialisation format (alias of '--output json')
        --test-coins    Create a SafeKey, allocate test-coins onto it, and add the SafeKey to the Wallet
    -V, --version       Prints version information

OPTIONS:
        --keyurl <keyurl>         An existing SafeKey's safe://xor-url. If this is not supplied, a new SafeKey will be
                                  automatically generated and inserted. The corresponding secret key will be prompted if
                                  not provided with '--sk'
        --name <name>             The name to give the spendable balance
    -o, --output <output_fmt>     Output data serialisation. Currently only supported 'json'
    -w, --pay-with <pay_with>     The secret key of a SafeKey for paying the operation costs
        --preload <preload>       Preload with a balance
        --sk <secret>             Pass the secret key needed to make the balance spendable, it will be prompted if not
                                  provided
        --xorurl <xorurl_base>    Base encoding to be used for XOR-URLs generated. Currently supported: base32z
                                  (default), base32 and base64
```

Right now, only a secret key (of a `SafeKey` with coins) can be used to pay for the costs, but in the future a `Wallet` will be also allowed for this purpose.

For example, if we use the secret key we obtained when creating a `SafeKey` in our example in previous section to pay for the costs, we can create a `Wallet` with a new spendable balance by simply running:

```shell
$ safe wallet create --pay-with 62e323615235122f7e20c7f05ddf56c5e5684853d21f65fca686b0bfb2ed851a --name first-spendable-balance
Wallet created at: "safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e"
New SafeKey created at: "safe://hbyyyybqk69tpm67ecnzjg66tcrja3ugq81oh6gfaffwaty614rmttmyeu"
Key pair generated:
Public Key = b95efc5abf750c15d26f7a2c22719999c79439e317052d31107a5a22e3158113d6003af4980b72ff076813eda30f1d8b
Secret Key = b9b2edffa8ef103dc98ba2160e295f98fdf981eb572bc2f8b018a12574ce435e
```

#### Wallet Balance

The balance of a given `Wallet` can be queried using its XorUrl. This returns the balance of the whole `Wallet`, i.e. the sum of the contained spendable balances. The target `Wallet` can be passed as an argument (or it will be retrieved from `stdin`):
```shell
$ safe wallet balance safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e
Wallet at "safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e" has a total balance of 0 safecoins
```

The coin balance of an individual spendable balance can also be queried by providing its friendly name as part of the `Wallet` URL, e.g. the `Wallet` we created above contains an spendable balance named 'first-spendable-balance', so we can check the balance of it (instead of the total balance of the `Wallet`) with the following command:
```shell
$ safe wallet balance safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e/first-spendable-balance
Wallet's spendable balance at "safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e/first-spendable-balance" has a balance of 0 safecoins
```

#### Wallet Insert

As mentioned before, a `SafeKey` doesn't hold the secret key on the network, therefore even if it has some non-zero coin balance, it cannot be spent. This is where the `Wallet` comes into play, holding the links to `SafeKey`s, and making their balances spendable by storing the corresponding secret keys.

Aside from at wallet creation, we can add _more_ keys to use as spendable balances by `insert`-ing into a `Wallet` a link to a `SafeKey`, making it a spendable balance.

```shell
USAGE:
    safe wallet insert [FLAGS] [OPTIONS] <target>

FLAGS:
        --default    Set the inserted SafeKey as the default one in the target Wallet
    -n, --dry-run    Dry run of command. No data will be written. No coins spent
    -h, --help       Prints help information
        --json       Sets JSON as output serialisation format (alias of '--output json')
    -V, --version    Prints version information

OPTIONS:
        --keyurl <keyurl>         The SafeKey's safe://xor-url to verify it matches/corresponds to the secret key provided.
                                  The corresponding secret key will be prompted if not provided with '--sk'
        --name <name>             The name to give this spendable balance
    -o, --output <output_fmt>     Output data serialisation. Currently only supported 'json'
    -w, --pay-with <pay_with>     The secret key of a SafeKey for paying the operation costs. If not provided, the default
                                  wallet from the account will be used
        --sk <secret>             Pass the secret key needed to make the balance spendable, it will be prompted if not
                                  provided
        --xorurl <xorurl_base>    Base encoding to be used for XOR-URLs generated. Currently supported: base32z
                                  (default), base32 and base64

ARGS:
    <target>    The target Wallet to insert the spendable balance
```

- The `<target>` is the `Wallet` to insert the spendable balance to
- The `--name` is an optional nickname to give a spendable balance for easy reference
- The `--default` flag sets _this_ new spendable balance as the default for the containing `Wallet`. This can be used by wallet applications to apply some logic on how to spend and/or choose the balances for a transaction

With the above options, the user will be prompted to input the secret key corresponding to the `SafeKey` to be inserted, unless it was already provided with `--sk`. This is stored in the `Wallet`.

The `--sk` argument can also be combined with `--keyurl` to pass the `SafeKey`'s XorUrl as part of the command line instruction itself, e.g.:

```shell
$ safe wallet insert safe://<wallet-xorurl> --keyurl safe://<key-xor-url> --name my-default-balance --default
Enter secret key corresponding to public key at safe://<key-xor-url>:
b493a84e3b35239cbffdf10b8ebfa49c0013a5d1b59e5ef3c000320e2d303311
Spendable balance inserted with name 'my-default-balance' in Wallet located at "safe://<wallet-xorurl>"
```

#### Wallet Transfer

Once a `Wallet` contains some spendable balance/s, we can transfer `--from` a `Wallet` an `<amount>` of safecoins `--to` another `Wallet` or `SafeKey`. The destination `Wallet`/`SafeKey` can be passed as an argument with `--to`, or it will be read from `stdin`.

```shell
$ safe wallet transfer <amount> --from <source Wallet URL> --to <destination Wallet/SafeKey URL>
```

If the `Wallet` being provided either as the source or destination of a transfer has a _default_ spendable balance, we then only need to provide its URL, e.g.:
```shell
$ safe wallet transfer 323.23 --from safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e --to safe://hbyek1io7m6we5ges83fcn16xd51bqrrjjea4yyhu4hbu9yunyc5mucjao
Success. TX_ID: 6183829450183485238
```

If on the contrary a `Wallet` being used (either as the source or destination of a transfer) doesn't have a _default_ spendable balance set, we can specify which spendable balance the operation should be applied to by passing its friendly name as the path of the `Wallet` URL. Or even if it has a _default_ spendable balance set, we can still choose which spendable balance to use in the operation. E.g. we can transfer from 'for-night-outs' spendable balance of the source `Wallet` with the following command:
```shell
$ safe wallet transfer 0.053 --from safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e/for-night-outs --to safe://hbyek1io7m6we5ges83fcn16xd51bqrrjjea4yyhu4hbu9yunyc5mucjao
Success. TX_ID: 277748716389078887
```

### Files

Uploading files and folders onto the network is also possible with the CLI application, and as we'll see here it's extremely simple to not just upload them but also keep them in sync with any modifications made locally to the folders and files.

Files are uploaded on the Network and stored as `Published ImmutableData` files, and the folders and sub-folders hierarchy is flattened out and stored in a container mapping each files's path with the corresponding `ImmutableData` XOR-URL. This map is maintained on the Network in a special container called `FilesContainer`, which is stored as `Published AppendOnlyData` data. The data representation in the `FilesContainer` is planned to be implemented with [RDF](https://en.wikipedia.org/wiki/Resource_Description_Framework) and the corresponding `FilesContainer` RFC will be submitted, but at this stage this is being done only using a simple serialised structure.

#### Files Put

The most simple scenario is to upload all files and sub-folders found within a local `./to-upload/` directory, recursively, onto a `FilesContainer` on the Network, obtaining the XOR-URL of the newly created container, as well as the XOR-URL of each of the files uploaded:
```shell
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://bbkulcb5hsl2zbsia4af5i7myv2ujbet7di4gx5bstduikwgobru67esqu"
+  ./to-upload/index.html          safe://bbkulcax6ulw7ovqhpsindkybsum4tusmvuc7ovtr2bu5gj6m4ugtu7euh
+  ./to-upload/myfolder/notes.txt  safe://bbkulcan3may5gmqxqonwaoz2cjlkuc4cflrhwitmzy7ur4paof4u57yxz
+  ./to-upload/img.jpeg            safe://bbkulcbtiq3vg4xehqbrjd2gz4kljguqtds5ko5khexya3v3k7scymcphj
```

Note that the `+` sign means the files were all added to the `FilesContainer`, which will make more sense later on when we see how to use the `files sync` command to update and/or delete files.

A single file can also be uploaded using the `files put` command, which will create a `FilesContainer` as well but this time only a single file will be added to the map:
```shell
$ safe files put ./to-upload/myfile.txt
FilesContainer created at: "safe://bbkulca25xhzwo6mcxlji7ocf5tm5hgn2x3mxtg62qzycculur4aixeywm"
+  ./to-upload/myfile.txt  safe://bbkulcbxk23cfnj7gz3r4y7624kpb5spwf4b7jogu2rofhuj5xiqa5huh7
```

##### Base path of files in a FilesContainer

When uploading files onto a `FilesContainer` with the CLI, the base path for the files in the container is set by default to be `/`. All the files at the source are published on the `FilesContainer` with an absolute path with base `/` path.

As an example, if we upload three files, which at source are located at `/to-upload/file1.txt`, `/to-upload/myfolder/file2.txt`, and `/to-upload/myotherfolder/subfolder/file3.txt`, the files will be published on the `FilesContainer` with paths `/file1.txt`, `/myfolder/file2.txt`, and `/myotherfolder/subfolder/file3.txt` respectively.

We can additionally pass a destination path argument to set a base path for each of the paths in the `FilesContainer`, e.g. if we provide `/mychosenroot/` destination path argument to the `files put` command when uploading the above files, they will be published on the `FilesContainer` with paths `/mychosenroot/file1.txt`, `/mychosenroot/myfolder/file2.txt`, and `/mychosenroot/myotherfolder/subfolder/file3.txt` respectively. This can be verified by querying the `FilesContainer` content with the `safe cat` command, please see further below for details of how this command.

#### Files Sync

Once a set of files, folders and subfolders, have been uploaded to the Network onto a `FilesContainer` using the `files put` command, local changes made to those files and folders can be easily synced up using the `files sync` command. This command takes care of finding the differences/changes on the local files and folders, creating new `Published ImmutableData` files as necessary, and updating the `FilesContainer` by publishing a new version of it at the same location on the Network.

The `files sync` command follows a very similar logic to the well known `rsync` command, supporting a subset its functionality. This subset will gradually be expanded with more supported features. Users knowing how to use `rsync` can easily start using the SAFE CLI and the SAFE Network for uploading files and folders, making it also easy to integrate existing automated systems which are currently making use of `rsync`.

As an example, let's suppose we upload all files and subfolders found within the `./to-upload/` local directory, recursively, using `files put` command:
```shell
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w"
+  ./to-upload/another.md              safe://hoxm5aps8my8he8cpgdqh8k5wuox5p7kzed6bsbajayc3gc8pgp36s
+  ./to-upload/subfolder/subexists.md  safe://hoqc6etdwbx6s86u3bkxenos3rf7dtr51eqdt17smxsw7aejot81dc
+  ./to-upload/test.md                 safe://hoxibhqth9awkjgi35sz73u35wyyscuht65m3ztrznb6thd5z8hepx
```

All the content of the `./to-upload/` local directory is now stored and published on the SAFE Network. Now, let's say we make the following changes to our local files within the `./to-upload/` folder:
- We edit `./to-upload/another.md` and change its content
- We create a new file at `./to-upload/new.md`
- And we remove the file `./to-upload/test.md`

We can now sync up all the changes we just made, recursively, with the `FilesContainer` we previously created:
```shell
$ safe files sync ./to-upload/ safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w --recursive --delete
FilesContainer synced up (version 1): "safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w?v=1"
*  ./to-upload/another.md     safe://hox6jstso13b7wzfkw1wbs3kwn9gpssudqunk6sw5yt3d6pnmaec53
+  ./to-upload/new.md         safe://hoxpdc8ywz18twkg7wboarj45hem3pq6ou6sati9i3dud68tzutw34
-  /test.md                   safe://hoxibhqth9awkjgi35sz73u35wyyscuht65m3ztrznb6thd5z8hepx
```

The `*`, `+` and `-` signs mean that the files were updated, added, and removed respectively.

Also, please note we provided the optional `--delete` flag to the command above which forces the deletion of the files found at the targeted `FilesContainer` that are not found in the source location, like the case of `./to-upload/test.md` file in our example above. If we didn't provide such flag, only the modification and creation of files would have been updated on the `FilesContainer`, like the case of `./to-upload/another.md` and `./to-upload/new` files in our example above. Note that `--delete` is only allowed if the `--recursive` flag is also provided.

The `files sync` command also supports to be passed a destination path as the `files put` command, but in this case the destination path needs to be provided as part of the target XOR-URL. E.g., we can sync a `FilesContainer` using the local path and provide a specific destination path `new-files` in the target XOR-URL:
```shell
$ safe files sync ./other-folder/ safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w/new-files
FilesContainer synced up (version 2): "safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w?v=2"
+  ./other-folder/file1.txt  safe://hoqi7papyp7c6riyxiez6y5fh5ugj4xc7syqhmex774ws4g4b1z1xq
```

The `./other-folder/file1.txt` file will be uploaded and published in the `FilesContainer` with path `/new-files/file1.txt`.

One more thing to note about `files sync` command is the use of the `--update-nrs` flag. When syncing content using an NRS-URL (see [NRS section](#nrs-name-resolution-system) below for more information about NRS names and commands), if you want to update the NRS name to the new version generated after syncing the target `FilesContainer`, then it can be specified using the `--update-nrs` flag:
```shell
$ safe files sync ./to-upload/ safe://mywebsite --update-nrs
FilesContainer synced up (version 1): "safe://mywebsite"
*  ./to-upload/another.md     safe://hox6jstso13b7wzfkw1wbs3kwn9gpssudqunk6sw5yt3d6pnmaec53
+  ./to-upload/new.md         safe://hoxpdc8ywz18twkg7wboarj45hem3pq6ou6sati9i3dud68tzutw34
```

#### Files Add

It could be desirable in some scenarios to simply add a file to a `FilesContainer` rather than having the CLI to sync up a complete local folder, so the `files add` command could be used in such cases.

We can add a single file from a local path, let's say `./some-other-folder/file.txt`, to our existing `FilesContainer` on the SAFE Network with the following command:
```shell
$ safe files add ./some-other-folder/file.txt safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w/files-added/just-a-file.txt
FilesContainer updated (version 3): "safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w?v=3"
+  ./some-other-folder/file1.txt  safe://hbhydydbhgmo7rxdgqyr98b5ojqwwjx4abnp4go6iw69gg4e7naigibr1n
```

If we have previously uploaded a file to the network, we can also add it to any existing `FilesContainer` by providing its XOR-URL as the `<location>` argument to the `files add` command. Let's add a file (same file we uploaded in previous command) to our `FilesContainer` again, but choosing a new destination filename, e.g. `/files-added/same-file.txt`:
```shell
$ safe files add safe://hbhydydbhgmo7rxdgqyr98b5ojqwwjx4abnp4go6iw69gg4e7naigibr1n safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w/files-added/same-file.txt
FilesContainer updated (version 4): "safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w?v=4"
+  /files-added/same-file1.txt  safe://hbhydydbhgmo7rxdgqyr98b5ojqwwjx4abnp4go6iw69gg4e7naigibr1n
```

### Xorurl

As we've seen, when uploading files to the network, each file is uploaded as an `ImmutableData` using the [self-encryption algorithm](https://github.com/maidsafe/self_encryption) in the client, splitting the files into encrypted chunks, and the resulting file's XOR-URL is linked from a `FilesContainer`.

The file's XOR-URL is deterministic based on its content, i.e. the location where each of its chunks are stored is determined based on the files's content, and performed at the client before uploading the chunks to the network. Therefore the XOR-URL is always the same if the content of a file doesn't change. All this means is we can know what the file's XOR-URL will be without uploading it to the network.

Obtaining local files' XOR-URLs without uploading them to the network can be done in two diffent ways. We can use the `--dry-run` flag in any of the files commands, e.g.:
```shell
$ safe files put ./to-upload/ --recursive --dry-run
NOTE the operation is being performed in dry-run mode, therefore no changes are committed to the network.
FilesContainer not created since running in dry-run mode
+  ./to-upload/another.md              safe://hoxm5aps8my8he8cpgdqh8k5wuox5p7kzed6bsbajayc3gc8pgp36s
+  ./to-upload/subfolder/subexists.md  safe://hoqc6etdwbx6s86u3bkxenos3rf7dtr51eqdt17smxsw7aejot81dc
+  ./to-upload/test.md                 safe://hoxibhqth9awkjgi35sz73u35wyyscuht65m3ztrznb6thd5z8hepx
```

There is also a handy `safe xorurl` command which allows us to provide a local path and obtain the XOR-URLs of the files found in such path, without uploading them to the network:
```shell
$ safe xorurl ./to-upload/ --recursive
3 file/s processed:
+  ./to-upload/another.md              safe://hoxm5aps8my8he8cpgdqh8k5wuox5p7kzed6bsbajayc3gc8pgp36s
+  ./to-upload/subfolder/subexists.md  safe://hoqc6etdwbx6s86u3bkxenos3rf7dtr51eqdt17smxsw7aejot81dc
+  ./to-upload/test.md                 safe://hoxibhqth9awkjgi35sz73u35wyyscuht65m3ztrznb6thd5z8hepx
```

#### Xorurl decode

XOR-URLs encode not only information about the location of the content, but also about the content type, native data type the data is being held on, etc.

In some particular cases it may be useful for the user to be able to decode this type of information from a given XOR-URL:
```shell
$ safe xorurl decode safe://hnyynyzonskbrgd57kt8c1pnb14qg8oh8wjo7xiku4mh4tc67wjax3c54sbnc
Information decoded from XOR-URL: safe://hnyynyzonskbrgd57kt8c1pnb14qg8oh8wjo7xiku4mh4tc67wjax3c54sbnc
Xorname: e02b282430f7d544ec93441969c63c387a261d7d553d2f9a8b3dda270fcb37ab
Type tag: 1100
Native data type: PublishedSeqAppendOnlyData
Path: none
Sub names: []
Content version: latest
```

### Cat

The `cat` command is probably the most straight forward command, it allows users to fetch data from the Network using a URL, and render it according to the type of data being fetched:
```shell
$ safe cat safe://<NRS-URL or XOR-URL>
```

If the URL targets a published `FilesContainer`, the `cat` command will fetch its content, and render it showing the list of files contained (linked) in it, along with the corresponding XOR-URLs for each of the linked files.

Let's see this in action, if we upload some folder using the `files put` command, e.g.:
```shell
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc"
+  ./to-upload/another.md              safe://hbyyyynhci18zwrjmiwqgpf5ofukf3dtryrkeizk1yxda3a5zoew6mgeox
+  ./to-upload/subfolder/subexists.md  safe://hbyyyydo4dhazjnj4i1sb4gpz94m19u31asrjaq3d8rzzc8s648w6xkzpb
+  ./to-upload/test.md                 safe://hbyyyydx1c168rwuqi6hcctwfbf1ihf9dfhr4bkmb6kzacs96uyj7bp4n6
```

We can then use `safe cat` command with the XOR-URL of the `FilesContainer` just created to render the list of files linked from it:
```shell
$ safe cat safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc
Files of FilesContainer (version 0) at "safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc":
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| Name                    | Size | Created              | Modified             | Link                                                              |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /another.md             | 6    | 2019-07-24T13:22:49Z | 2019-07-24T13:22:49Z | safe://hbyyyynhci18zwrjmiwqgpf5ofukf3dtryrkeizk1yxda3a5zoew6mgeox |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /subfolder/subexists.md | 7    | 2019-07-24T13:22:49Z | 2019-07-24T13:22:49Z | safe://hbyyyydo4dhazjnj4i1sb4gpz94m19u31asrjaq3d8rzzc8s648w6xkzpb |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /test.md                | 12   | 2019-07-24T13:22:49Z | 2019-07-24T13:22:49Z | safe://hbyyyydx1c168rwuqi6hcctwfbf1ihf9dfhr4bkmb6kzacs96uyj7bp4n6 |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
```

We could also take any of the XOR-URLs of the individual files and have the `cat` command fetch the content of the file and show it in the output, e.g. let's use the XOR-URL of the `/test.md` file to fetch its content:
```shell
$ safe cat safe://hbyyyydx1c168rwuqi6hcctwfbf1ihf9dfhr4bkmb6kzacs96uyj7bp4n6
hello tests!
```

Alternatively, we could use the XOR-URL of the `FilesContainer` and provide the path of the file we are trying to fetch, in this case the `cat` command will resolve the path and follow the corresponding link to read the file's content directly for us. E.g. we can also read the content of the `/test.md` file with the following command:
```shell
$ safe cat safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc/test.md
hello tests!
```

And if we provide a path to a subfolder of the `FilesContainer`, the `cat` command will resolve the path and list only those files which path is a child of the provided path:
```shell
$ safe cat safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc/subfolder
Files of FilesContainer (version 0) at "safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc/subfolder":
+--------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| Name         | Size | Created              | Modified             | Link                                                              |
+--------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| subexists.md | 7    | 2019-07-24T13:22:49Z | 2019-07-24T13:22:49Z | safe://hbyyyydo4dhazjnj4i1sb4gpz94m19u31asrjaq3d8rzzc8s648w6xkzpb |
+--------------+------+----------------------+----------------------+-------------------------------------------------------------------+
```

A `Wallet` can be also fetched with `cat` to inspect its content, the list of spendable balances it holds will be listed, and we can see which of them is currently the default to be used in a transaction operation:
```shell
$ safe cat safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e
Spendable balances of Wallet at "safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e":
+---------+-------------------------+-------------------------------------------------------------------+
| Default | Friendly Name           | SafeKey URL                                                       |
+---------+-------------------------+-------------------------------------------------------------------+
| *       | my-default-balance      | safe://hbyyyybffgc3smq1ynjewsbtqkm5h9rq367n6krzd9rz65p8684x9wy81m |
+---------+-------------------------+-------------------------------------------------------------------+
|         | first-spendable-balance | safe://hbyyyybqk69tpm67ecnzjg66tcrja3ugq81oh6gfaffwaty614rmttmyeu |
+---------+-------------------------+-------------------------------------------------------------------+
```

As seen above, the `safe cat` command can be used to fetch any type of content from the SAFE Network. At this point it only supports files (`ImmutableData`), `FilesContainer`s, `Wallet`s, and `NRS-Container`s (see further below about NRS Containers and commands), but it will be expanded as more types are supported by the CLI and its API.

#### Retrieving binary files with --hexdump

By default, binary files are treated just like a plaintext file and will typically display unreadable garbage on the screen unless output is redirected to a file, eg:

```shell
$ safe cat safe://hbwybynbbwotm5qykdfxuu4r4doogaywf8jupxats5zg39xjjtd8xmtpky > /tmp/favicon.ico
```

However, the flag --hexdump is available which provides a more human friendly hexadecimal dump, similar to that of the standard *xxd* unix tool.  Here's an example.

```shell
$ safe cat --hexdump safe://hbwybynbbwotm5qykdfxuu4r4doogaywf8jupxats5zg39xjjtd8xmtpky | head
Length: 1406 (0x57e) bytes
0000:   00 00 01 00  01 00 10 10  00 00 01 00  08 00 68 05   ..............h.
0010:   00 00 16 00  00 00 28 00  00 00 10 00  00 00 20 00   ......(....... .
0020:   00 00 01 00  08 00 00 00  00 00 00 01  00 00 00 00   ................
0030:   00 00 00 00  00 00 00 01  00 00 00 00  00 00 f4 cc   ................
0040:   a8 00 cb 7b  45 00 fb f2  e5 00 ab 62  46 00 ab 60   ...{E......bF..`
0050:   46 00 c0 a6  8e 00 f2 d9  c1 00 f5 e8  df 00 e0 9a   F...............
0060:   5e 00 ea c0  9e 00 e8 ae  77 00 be 85  5d 00 bb 61   ^.......w...]..a
0070:   35 00 fa ed  d7 00 ff fc  f7 00 ce 88  4c 00 b0 56   5...........L..V
0080:   34 00 fe fa  f6 00 bf 87  5b 00 b1 6b  50 00 dd 82   4.......[..kP...
```

#### Retrieving older versions of content

As we've seen above, we can use `cat` command to retrieve the latest/current version of any type of content from the Network using their URL. But every change made to content that is uploaded to the Network as Published data is perpetual, and therefore a new version is generated when performing any amendments to it, keeping older versions also available forever.

We can use the `cat` command to also retrieve any version of content that was uploaded as Published data by appending a query param to the URL. E.g. given the XOR-URL of the `FilesContainer` we created in previous sections (`safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w`), which reached version 2 after a couple of amendments we made with `files sync` command, we can retrieve the very first version (version 0) by using `v=<version>` query param:
```shell
$ safe cat "safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w?v=0"
Files of FilesContainer (version 0) at "safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w?v=0":
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| Name                    | Size | Created              | Modified             | Link                                                              |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /another.md             | 6    | 2019-07-24T13:22:49Z | 2019-07-24T13:22:49Z | safe://hoxm5aps8my8he8cpgdqh8k5wuox5p7kzed6bsbajayc3gc8pgp36s |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /subfolder/subexists.md | 7    | 2019-07-24T13:22:49Z | 2019-07-24T13:22:49Z | safe://hoqc6etdwbx6s86u3bkxenos3rf7dtr51eqdt17smxsw7aejot81dc |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /test.md                | 12   | 2019-07-24T13:22:49Z | 2019-07-24T13:22:49Z | safe://hoxibhqth9awkjgi35sz73u35wyyscuht65m3ztrznb6thd5z8hepx |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
```

### NRS (Name Resolution System)

As we've seen in all the above sections, every piece of data on the SAFE Network has a unique location. Such location is determined by the XoR name given to it in the Network's XoR address space, as well as some other information which depends on the native date type, like in the case of `MutableData` data type which also has a type tag associated to it apart from its XoR address.

So far all commands were using the XOR-URLs to either inform of the new data created/stored on the Network, as well as for retrieving data form the Network.

While XOR-URLs are simply a way to encode SAFE Network data unique location into a URL, there are some incentives for having more human friendly URLs that can be easily remembered and recognisable when trying to share them with other people, or use them with tools and applications like the SAFE CLI or the SAFE Browser.

This is why the SAFE Network also supports having such human friendly URLs through what it's called the `Name Resolution System (NRS)`. The NRS allows users to create friendly names that are resolvable to a unique location on the Network. These friendly names can be used in the form of a URL (NRS-URL) to share with other people the location of websites, web applications, files and folders, safecoin wallets for receiving transfers, SAFE IDs, etc.

In this section we will explore the CLI commands which allow users to generate, administer, and use the NRS and its NRS-URLs for publishing and retrieving data to/from the SAFE Network.

#### NRS Create

Creating a friendly name on the Network can be achieved with the `nrs create` subcommand. This subcommand generates an NRS Container automatically linking it to any data we decide to link the friendly name to. An NRS Container is stored on the Network as a `Published AppendOnlyData` data, and it contains an NRS Map using RDF for its data representation (since this is still under development, pseudo-RDF data is now being used temporarily). This Map has a list of subnames and where each of them are being linked to, e.g. `mysubname` can be created as a subname of `mywebsite` NRS name by having `mysubname` linked to a particular `FilesContainer` XOR-URL so that it can be fetched with `safe://mysubname.mywebsite`.

Let's imagine we have uploaded the files and folders of a website we want to publish on the SAFE Network with `files put` command:
```shell
$ safe files put ./website-to-publish/ --recursive`
FilesContainer created at: "safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc"
+  ./website-to-publish/index.html              safe://hbyyyydhp7y3mb6zcj4herpqm53ywnbycstamb54yhniud1cij7frjfe8c
+  ./website-to-publish/image.jpg               safe://hbyyyynkt8ak5mxmbqkdt81hqceota8fu83e49gi3weszddujfc8fxcugp
+  ./website-to-publish/contact/form.html       safe://hbyyyyd1sw4dd57k1xeeijukansatia6mthaz1h6htnb8pjoh9naskoaks
```

As we know that website is now publicly available on the SAFE Network for anyone who wants to visit using its XOR-URL "safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc" with either `$ safe cat` command, or a SAFE Browser. But let's now create a NRS name for it and obtain its human friendly NRS-URL:
```shell
$ safe nrs create mywebsite --link "safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0"
New NRS Map for "safe://mywebsite" created at: "safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh"
+  mywebsite  safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0
```

Note that we provided a versioned URL to the `--link` argument in the command above, i.e. a URL which targets a specific version of the content with `?v=<version number>`. Any type of content which can have different versions (like the case of a `FilesContainer` in our example) can be mapped/linked from an NRS name/subname only if a specific version is provided in the link URL.

We can now share the NRS-URL `safe://mywebsite` to anyone who wants to visit our website. Using this NRS-URL we can now fetch the same content we would do when using the `FilesContainer` XOR-URL we linked to it, thus we can fetch it using the following command:
```shell
$ safe cat safe://mywebsite
Files of FilesContainer (version 0) at "safe://mywebsite":
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| Name                    | Size | Created              | Modified             | Link                                                              |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /index.html             | 146  | 2019-07-24T14:31:42Z | 2019-07-24T14:31:42Z | safe://hbyyyydhp7y3mb6zcj4herpqm53ywnbycstamb54yhniud1cij7frjfe8c |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /image.jpg              | 391  | 2019-07-24T14:31:42Z | 2019-07-24T14:31:42Z | safe://hbyyyynkt8ak5mxmbqkdt81hqceota8fu83e49gi3weszddujfc8fxcugp |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /contact/form.html      | 23   | 2019-07-24T14:31:42Z | 2019-07-24T14:31:42Z | safe://hbyyyyd1sw4dd57k1xeeijukansatia6mthaz1h6htnb8pjoh9naskoaks |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
```

In this example the `cat` simply prints out the content of the top level folder (`FilesContainer`) as we've learned from previous sections of this guide, but any other tool or application would be treating this in different ways, e.g. the SAFE Browser would be automatically fetching the `index.html` file from it and rendering the website to the user.

We can obviously fetch the content of any of the files published at this NRS-URL using the corresponding path:
```shell
$ safe cat safe://mywebsite/contact/form.html
<!DOCTYPE html>
<html>
<body>
<h2>Contact Form</h2>
<form>
  ...
</form>
</body>
</html>
```

##### Sub Names

Much like the old internet, the NRS system provides increased flexibility for those wanting to have a variety of resources available under one public name, via using `Sub Names`. This is done by creating a Public name and using a `.` (dot) character to separate it into various, individually controllable parts.

For example, you may wish to have `safe://mywebsite` to be about you in general, whereas `safe://blog.mywebsite` point to a different site which is your blog.

To create a public name with subnames, you need only to pass the full string with `.` separators, just like any traditional URL, to the CLI:
```shell
$ safe nrs create blog.mywebsite --link "safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0"
New NRS Map for "safe://blog.mywebsite" created at: "safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh"
+  blog.mywebsite  safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0
```

As the NRS CLI advances, you'll be able to individually add to both `blog.mywebsite`, or indeed just `mywebsite`, as well as change what the `default` resource to retrieve is for both.

#### NRS Add

Once a public name has been created with `nrs create` command, more sub names can be added to it using the `nrs add` command. This command expects the same arguments as `nrs create` command but it only requires and assumes that the public name already exists.

Let's add `profile` sub name to the `mywebsite` NRS name we created before:
```shell
$ safe nrs add profile.mywebsite --link "safe://hnyynyipybem7ihnzqya3w31seezj4i6u8ckg9d7s39exz37z3nxue3cnkbnc?v=0"
NRS Map updated (version 1): "safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh"
+  profile.mywebsite  safe://hnyynyz8m4pkok41qrn9gkrwz35fu8zxfkwrc9xrt595wjtodacx9n8u3wbnc
```

The safe nrs add command can also be used to update subnames after they have been added to a public name.
For example, if we have made changes to files mapped to the `profile.mywebsite` NRS subname we created before, we can use `nrs add` to update its link:
```shell
$ safe nrs add profile.mywebsite --link safe://hnyynyw9ru4afkbfee5m4ca4jbho4f5bj6ynep5k1pioyge6dihfqyjfrnbnc?v=0
NRS Map updated (version 2): "safe://profile.hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh"
+  profile.mywebsite  safe://hnyynyw9ru4afkbfee5m4ca4jbho4f5bj6ynep5k1pioyge6dihfqyjfrnbnc?v=0
```

#### NRS Remove

Removing sub names from an NRS Map Container is very simple and straight forward, since the only information required to do so is just the NRS-URL. The `nrs remove` command will remove only the sub name specified in the provided NRS-URL without touching any of the other existing sub names, e.g. if the `safe://sub-b.sub-a.mypubname` NRS-URL is provided then only `sub-b` sub name will be removed from `mypubname` NRS Map Container (by creating a new version of it, remember this is all part of the perpetual web).

Let's remove the `profile` sub name from the `mywebsite` NRS name we added before:
```shell
$ safe nrs remove profile.mywebsite
NRS Map updated (version 3): "safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh"
-  profile.mywebsite  safe://hnyynyw9ru4afkbfee5m4ca4jbho4f5bj6ynep5k1pioyge6dihfqyjfrnbnc?v=0
```

### SAFE-URLs

In previous sections of this guide we explained how we can create two types of safe:// URLs, XOR-URLs and NRS-URLs. It has been explained that safe:// URLs can contain a path as well, if they target a `FilesContainer`, and they can also be post-fixed with `v=<version>` query param in order to target a specific version of the content rather than the latest/current version when this query param is omitted.

All these types of safe:// URLs can be used in any of the supported CLI commands interchangeably as the argument of any command which expects safe:// URL.

E.g. we can retrieve the content of a website with the `cat` command using either its XOR-URL or its NRS-URL, and either fetching the latest version of it or supplying the query param to get a specific version of it. Thus, if we wanted to fetch `version #1` of the site we published at `safe://mywebsite` (which NRS Map Container XOR-URL is `safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh`), the following two commands would be equivalent:
- `$ safe cat "safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh?v=1"`
- `$ safe cat "safe://mywebsite?v=1"`

In both cases the NRS Map Container will be found (from above URLs) by decoding the XOR-URL or by resolving NRS public name. Once that's done, and since the content is an NRS Map, following the rules defined by NRS and the map found in it the target link will be resolved from it. In some circumstances, it may be useful to get information about the resolution of a URL, which can be obtained using the `dog` command.

### Dog

The SAFE Network relates information and content using links, as an example, just considering some of the type of content we've seen in this guide, `FilesContainer`s, `Wallet`s and `NRS Map Container`s, they are all containers with named links (SAFE-URLs) to other content on the network, and depending on the abstraction they provide, each of these links are resolved following a specific set of rules for each type of container, e.g. NRS subnames are resolved with a pre-defined set of rules, while a file's location is resolved from a FilesContainer with another set of pre-defined rules.

Using the `cat` command is a very straight forward way of retrieving any type of data and see its content, but sometimes we may want to understand how the location of the content being retrieved is resolved using these set of pre-defined rules, and how links are resolved to eventually find the location of the content we are retrieving. This is when we need the `dog` command to sniff around and show the trace when resolving all these links from a URL.

The most basic case for the `dog` command is to get information about the native data type holding a content found with a XOR-URL:
```shell
$ safe dog safe://hnyynywttiyr6tf3qk811b3rto9azx8579h95ewbs3ikwpctxdhtqesmwnbnc
Native data type: PublishedSeqAppendOnlyData
Version: 0
Type tag: 1100
XOR name: 0x231a809e8972e51e520e49187f1779f7dff3fb45036cd5546b22f1f22e459741
XOR-URL: safe://hnyynywttiyr6tf3qk811b3rto9azx8579h95ewbs3ikwpctxdhtqesmwnbnc
```

In this case we see the location where this data is stored on the Network (this is called the XOR name), a type tag number associated with the content (1100 was set for this particular type of container), and the native SAFE Network data type where this data is being held on (`PublishedSeqAppendOnlyData`), and since this type of data is versionable we also see which is the version of the content the URL resolves to.

Of course the `safe dog` command can be used also with other type of content like files (`ImmutableData`), e.g. if we use it with a `FilesContainer`'s XOR-URL and the path of one of the files it contains:
```shell
$ safe dog safe://hnyynywttiyr6tf3qk811b3rto9azx8579h95ewbs3ikwpctxdhtqesmwnbnc/subfolder/index.html
Native data type: ImmutableData (published)
XOR name: 0xda4ce4aa59889874921817e79c2b98dc3dbede7fd9a9808a60aa2d35efaa05f4
XOR-URL: safe://hbhybyds1ch1ifunraq1jbof98uoi3tzb7z5x89spjonfgbktpgzz4wbxw
Media type: text/html
```

But how about using the `dog` command with an NRS URL, as we now know it's resolved using the NRS rules and following the links found in the NRS Map Container:
```shell
$ safe dog safe://mywebsite/contact/form.html
Native data type: ImmutableData (published)
XOR name: 0xda4ce4aa59889874921817e79c2b98dc3dbede7fd9a9808a60aa2d35efaa05f4
XOR-URL: safe://hbhybyds1ch1ifunraq1jbof98uoi3tzb7z5x89spjonfgbktpgzz4wbxw
Media type: text/html

Resolved using NRS Map:
PublicName: "mywebsite"
Container XOR-URL: safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh
Native data type: PublishedSeqAppendOnlyData
Type tag: 1500
XOR name: 0xfb3887c26c7ea3670ab1a042d16a6f1113ccf7cc09a15a6716429382a86eb1f9
Version: 3
+------------------+----------------------+----------------------+--------------------------------------------------------------------------+
| NRS name/subname | Created              | Modified             | Link                                                                     |
+------------------+----------------------+----------------------+--------------------------------------------------------------------------+
| mywebsite        | 2019-07-24T14:32:13Z | 2019-07-24T14:32:13Z | safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0 |
+------------------+----------------------+----------------------+--------------------------------------------------------------------------+
| blog.mywebsite   | 2019-07-24T16:52:30Z | 2019-07-24T16:52:30Z | safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0 |
+------------------+----------------------+----------------------+--------------------------------------------------------------------------+
```

In this case we don't only get information about the content that the URL resolves to, but also about the NRS Map Container this NRS-URL was resolved with. E.g. we see the XOR-URL of the NRS Map Container, its version, and among other data we also see the list of all NRS names defined by it with their corresponding XOR-URL links.

### Networks

The CLI, like any other SAFE application, can connect to different SAFE networks that may be available. As the project advances several networks may coexist apart from the main SAFE Network, there could be SAFE Networks available for testing upcoming features, or networks that are local to the user in their own computer or WAN/LAN.

Currently, there is a shared Vault which acts as public test network accessible to anyone. And also users may have local vaults running in their own environment. The CLI allows users to easily create a list of different SAFE networks in its config settings, to then be able to switch between them with just a simple command. Let's see some examples.

The way SAFE applications currently connect to a SAFE Vault is by reading the vault's connection information from a specific location in the system:
- Linux: from `~/.config/safe_vault/`
- macOS: from `/Users/<USERNAME>/Library/Preferences/net.MaidSafe.safe_vault/`
- Windows: from `C:\Users\<USERNAME>\AppData\Roaming\MaidSafe\safe_vault\config\`

#### Run a local vault

If you wish to run you own local vault you need to follow these steps:
1. download latest release from [safe_vault releases](https://github.com/maidsafe/safe_vault/releases/latest)
2. untar/unzip the downloaded file into a directory of your choice
3. execute the safe_vault

Example command to perform this on Linux or Mac:
```shell
$ mkdir ~/safe_vault
$ cd ~/safe_vault
$ wget https://github.com/maidsafe/safe_vault/releases/download/0.19.2/safe_vault-0.19.2-x86_64-unknown-linux-musl.tar.gz
$ tar -xzvf safe_vault-0.19.2-x86_64-unknown-linux-musl.tar.gz
$ ./safe_vault
```

Once the local vault is running, the connection configuration file will already be in the correct place for your applications (including the CLI) to connect to this vault, so you can simply use the CLI or any application from now on to connect to your local vault. Note that depending on the application, you may need to restart it so it uses the new connection information for your local vault rather than a previously existing one.

#### Switch networks

MaidSafe hosts a vault for those who don't want to run a local vault but still have a go at using the CLI and client applications. It's very common for users testing and experimenting with CLI and SAFE applications to have a local vault running, but switching to use the MaidSafe shared vault, back and forth, is also quite common.

The CLI allows you to set up a list of networks/vaults in its config settings for easily switching to connect to them. If you just started a local vault, you can keep current connection information as a configured network on CLI:
```shell
$ safe networks add my-vault
Caching current network connection information into: ~/.config/safe-cli/networks/my-vault_vault_connection_info.config
Network 'my-vault' was added to the list. Connection information is located at '~/.config/safe-cli/networks/my-vault_vault_connection_info.config'
```

If you also would like to connect to the MaidSafe shared vault, you'd need to set it up in CLI settings as another network too, specifying the URL where to fetch latest connection information from, e.g.:
```shell
$ safe networks add shared-vault https://safe-vault-config.s3.eu-west-2.amazonaws.com/shared-vault/vault_connection_info.config
Network 'shared-vault' was added to the list
```

We can retrieve the list of the different networks set up in the CLI config:
```shell
$ safe networks
+--------------+------------------------------------------------------------------------------------------------+
| Networks     |                                                                                                |
+--------------+------------------------------------------------------------------------------------------------+
| Network name | Connection info location                                                                       |
+--------------+------------------------------------------------------------------------------------------------+
| my-vault     | ~/.config/safe-cli/networks/my-vault_vault_connection_info.config                              |
+--------------+------------------------------------------------------------------------------------------------+
| shared-vault | https://safe-vault-config.s3.eu-west-2.amazonaws.com/shared-vault/vault_connection_info.config |
+--------------+------------------------------------------------------------------------------------------------+
```

Once we have them in the CLI settings, we can use the CLI to automatically fetch the connection information data using the configured location, and place it in the right place in the system for SAFE applications to connect to the selected network. E.g. let's switch to the 'shared-vault' network we previously configured:
```shell
$ safe networks switch shared-vault
Switching to 'shared-vault' network...
Fetching 'shared-vault' network connection information from 'https://safe-vault-config.s3.eu-west-2.amazonaws.com/shared-vault/vault_connection_info.config' ...
Successfully switched to 'shared-vault' network in your system!
If you need write access to the '{}' network, you'll need to restart authd, login and re-authorise the CLI again
```

Remember that every time you run a local vault the connection configuration in your system is automatically overwritten by the local vault with new connection information. Also if the shared vault was restarted by MaidSafe, the new connection information is published in the same URL and needs to be updated in your system to be able to successfully connect to it. Thus if you want to make sure your currently setup network matches any of those set up in the CLI config, you can use the `check` subcommand:
```shell
$ safe networks check
Checking current setup network connection information...
Fetching 'my-vault' network connection information from '~/.config/safe-cli/networks/my-vault_vault_connection_info.config' ...
Fetching 'shared-vault' network connection information from 'https://safe-vault-config.s3.eu-west-2.amazonaws.com/shared-vault/vault_connection_info.config' ...

'shared-vault' network matched. Current set network connection information at '~/.config/safe_vault/vault_connection_info.config' matches 'shared-vault' network as per current config
```

Note that in the scenario that your current network is set to be the MaidSafe shared vault, and the shared vault is restarted by MaidSafe (which causes new connection information to be published at the same URL), you then only need to re-run the `networks switch` command with the corresponding network name to update your system with the new connection information.

### Shell Completions

Automatic command completions via <tab> are available for popular shells such as bash and PowerShell (Windows). Completions are also provided for the shells fish, zsh, and elvish.

Until an installer becomes available, these completions must be manually enabled as per below.

#### Bash Completions

To enable bash completions in the current bash session, use the following command:
```shell
SC=/tmp/safe.rc && safe setup completions bash > $SC && source $SC
```

To enable bash completions always for the current user:
```shell
SC=~/.bash_safe_cli && safe setup completions bash > $SC && echo "source $SC" >> ~/.bashrc
```

#### Windows PowerShell Completions

To enable completions in the current PowerShell session, use the following commands:
```shell
safe setup completions bash > safe_cli.ps1
./safe_cli.ps1
```

To enable PowerShell completions permanently, generate the safe_cli.ps1 file as per above and then see this [stackoverflow answer](<https://stackoverflow.com/questions/20575257/how-do-i-run-a-powershell-script-when-the-computer-starts#32189430>).

### Update

The CLI can update itself to the latest available version. If you run `safe update`, the application will check if a newer release is available on [GitHub](https://github.com/maidsafe/safe-api/releases). After prompting to confirm if you want to take the latest version, it will be downloaded and the binary will be updated.

## Further Help

You can discuss development-related topics on the [SAFE Dev Forum](https://forum.safedev.org/).

If you are just starting to develop an application for the SAFE Network, it's very advisable to visit the [SAFE Network Dev Hub](https://hub.safedev.org) where you will find a lot of relevant information.

If you find any issues, or have ideas for improvements and/or new features for this application and the project, please raise them by [creating a new issue in this repository](https://github.com/maidsafe/safe-api/issues).

## License
This SAFE Network library is dual-licensed under the Modified BSD ([LICENSE-BSD](LICENSE-BSD) https://opensource.org/licenses/BSD-3-Clause) or the MIT license ([LICENSE-MIT](LICENSE-MIT) https://opensource.org/licenses/MIT) at your option.

## Contribute
Copyrights in the SAFE Network are retained by their contributors. No copyright assignment is required to contribute to this project.
