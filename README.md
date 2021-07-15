# Safe Network CLI

| [MaidSafe website](https://maidsafe.net) | [Safe Dev Forum](https://forum.safedev.org) | [Safe Network Forum](https://safenetforum.org) |
|:----------------------------------------:|:-------------------------------------------:|:----------------------------------------------:|

## Table of contents

- [Safe Network CLI](#safe-network-cli)
  - [Table of contents](#table-of-contents)
  - [Description](#description)
  - [Download](#download)
    - [Linux and Mac](#linux-and-mac)
    - [Windows](#windows)
  - [Build](#build)
  - [Using the CLI](#using-the-cli)
      - [`--help`](#--help)
    - [Networks](#networks)
      - [Node install](#node-install)
      - [Run a local network](#run-a-local-network)
        - [Run a local network for testing: `--test`](#run-a-local-network-for-testing---test)
      - [Connect to a shared network](#connect-to-a-shared-network)
      - [Switch networks](#switch-networks)
      - [Set network bootstrap address](#set-network-bootstrap-address)
      - [Node update](#node-update)
    - [Auth](#auth)
      - [The Authenticator daemon (authd)](#the-authenticator-daemon-authd)
      - [Auth install](#auth-install)
      - [Auth start](#auth-start)
      - [Auth status](#auth-status)
      - [Auth create](#auth-create)
      - [Auth unlock](#auth-unlock)
        - [Passing credentials from a config file](#passing-credentials-from-a-config-file)
        - [Using environment variables](#using-environment-variables)
      - [Auth reqs](#auth-reqs)
      - [Auth allow/deny](#auth-allowdeny)
        - [Self authorising the CLI application](#self-authorising-the-cli-application)
      - [Auth update](#auth-update)
    - [The interactive shell](#the-interactive-shell)
    - [SafeKeys](#safekeys)
      - [SafeKeys Creation](#safekeys-creation)
      - [SafeKey's Balance](#safekeys-balance)
      - [SafeKeys Transfer](#safekeys-transfer)
    - [Wallet](#wallet)
      - [Wallet Creation](#wallet-creation)
      - [Wallet Balance](#wallet-balance)
      - [Wallet Insert](#wallet-insert)
      - [Wallet Transfer](#wallet-transfer)
    - [Files](#files)
      - [[ Warning: Underlying API to be deprecated ]](#-warning-underlying-api-to-be-deprecated-)
      - [Files...](#files-1)
      - [Files Put](#files-put)
        - [Base path of files in a FilesContainer](#base-path-of-files-in-a-filescontainer)
      - [Files Sync](#files-sync)
      - [Files Add](#files-add)
      - [Files Ls](#files-ls)
      - [Files Get](#files-get)
        - [Example: retrieving contents of a file container to local working directory](#example-retrieving-contents-of-a-file-container-to-local-working-directory)
        - [Example: retrieving subfolder in a file container to an existing local directory.](#example-retrieving-subfolder-in-a-file-container-to-an-existing-local-directory)
        - [Example: retrieving subfolder in a file container to a non-existent local directory (rename)](#example-retrieving-subfolder-in-a-file-container-to-a-non-existent-local-directory-rename)
        - [Example: Retrieving individual file to an existing directory](#example-retrieving-individual-file-to-an-existing-directory)
        - [Example: Retrieving individual file to a new filename](#example-retrieving-individual-file-to-a-new-filename)
        - [A performance note about very large FileContainers](#a-performance-note-about-very-large-filecontainers)
      - [Files Tree](#files-tree)
      - [Files Rm](#files-rm)
    - [Xorurl](#xorurl)
      - [Xorurl decode](#xorurl-decode)
    - [Cat](#cat)
      - [Retrieving binary files with --hexdump](#retrieving-binary-files-with---hexdump)
      - [Retrieving older versions of content](#retrieving-older-versions-of-content)
    - [NRS (Name Resolution System)](#nrs-name-resolution-system)
      - [NRS Create](#nrs-create)
        - [Sub Names](#sub-names)
      - [NRS Add](#nrs-add)
      - [NRS Remove](#nrs-remove)
    - [Safe-URLs](#safe-urls)
      - [Symlinks](#symlinks)
    - [Dog](#dog)
    - [Seq (Sequence)](#seq-sequence)
      - [Seq Store](#seq-store)
        - [Private Sequence](#private-sequence)
      - [Seq Append](#seq-append)
    - [Shell Completions](#shell-completions)
      - [Bash Completions](#bash-completions)
      - [Windows PowerShell Completions](#windows-powershell-completions)
    - [Update](#update)
  - [Further Help](#further-help)
  - [License](#license)
  - [Contributing](#contributing)

## Description

This crate implements a CLI (Command Line Interface) for the Safe Network.

The Safe CLI provides all the tools necessary to interact with the Safe Network, including storing and browsing data of any kind, following links that are contained in the data and using their addresses on the network, using safecoin wallets, and much more. Using the CLI, users have access to any type of operation that can be made on the Safe Network and the data stored on it, allowing them to also use it for automated scripts and piped chain of commands.

## Download

The latest version of the Safe CLI can be downloaded and installed using the [install script](https://sn-api.s3.amazonaws.com/install.sh).

The [install script](https://sn-api.s3.amazonaws.com/install.sh) will not only download the latest Safe CLI, but it will also unpack the CLI binary into the `~/.safe/cli/` folder (`C:\Users\<user>\.safe\cli` in Windows), as well as set it in the PATH, so you can run the `safe` binary from any location when opening a console.

### Linux and Mac

Open a new console and run either of the following `curl` or `wget` commands:
```
$ curl -so- https://sn-api.s3.amazonaws.com/install.sh | bash
```
or
```
$ wget -qO- https://sn-api.s3.amazonaws.com/install.sh | bash
```

### Windows

If you are a Windows user, you will need to download and install the [Visual C++ Redistributable for Visual Studio](https://www.microsoft.com/en-in/download/details.aspx?id=48145) if not already installed on your machine, otherwise attempting to run the CLI will result in errors such as:
`error while loading shared libraries: api-ms-win-crt-locale-l1-1-0.dll: cannot open shared object file: No such file or directory`

You may already have these Visual C++ libraries on your machine if you already use Visual Studio.

Next, you'll need to open a [Git BASH](https://gitforwindows.org) console with admin permissions.

Click the "Start" button and type "git-bash" in the search bar, then press the **Shift+Ctrl+Enter** keys to reach Git-Bash console. The Git-Bash icon may also be in the Start Menu. You can [download Git Bash from here](https://gitforwindows.org/) if you don't already have installed on your PC.

Once you have an admin Git Bash console running, just run the above `curl` command to download and execute our install script:
```
$ curl -so- https://sn-api.s3.amazonaws.com/install.sh | bash
```

Once the Safe CLI is downloaded and installed on your system, you can follow the steps in this User Guide by starting from the [Using the CLI](#using-the-cli) section below in this document.

Alternatively, you can download the latest version of the Safe CLI from the [releases page](https://github.com/maidsafe/sn_cli/releases/latest) and install it manually on your system.

If you prefer to build the Safe CLI from source code, please follow the instructions in the [Build](#build) section below.

## Build

In order to build this CLI from source code you need to make sure you have `rustc v1.48.0` (or higher) installed. Please take a look at this [notes about Rust installation](https://www.rust-lang.org/tools/install) if you need help with installing it. We recommend you install it with `rustup` which will install the `cargo` tool which this guide makes use of.

Once Rust and its toolchain are installed, run the following commands to clone this repository and build the `sn_cli` (the build process may take several minutes the first time you run it on this crate):
```shell
$ git clone https://github.com/maidsafe/sn_cli.git
$ cd sn_cli
$ cargo build
```

Since this project has a cargo workspace with the `sn_cli` as the default crate, when building from the root location it will build the Safe CLI. Once it's built, you can find the `safe` executable at `target/debug/`, or `target/release` if you built it with the `--release` flag.

## Using the CLI

Right now the CLI is under active development. Here we're listing commands ready to be tested.

In this guide we will assume from now on that you have used the install script and so have the CLI binary added to your system PATH, or you have navigated your terminal to the directory where the `safe` executable file you downloaded or built for your platform is located.

The base command, once built, is `$ safe`, or all commands can be run via `$ cargo run -- <command>`.

Various global flags are available:

```
-n, --dry-run              Dry run of command. No data will be written. No coins spent.
-h, --help                 Prints help information
    --json                 Sets JSON as output serialisation format (alias of '--output json')
-V, --version              Prints version information
-o, --output <output_fmt>  Output data serialisation: [json, jsoncompact, yaml]
    --xorurl <xorurl_base> Base encoding to be used for XOR-URLs generated. Currently supported: base32z
                           (default), base32 and base64
```

#### `--help`

All commands have a `--help` function which lists args, options and subcommands.

### Networks

The CLI, like any other Safe application, can connect to different Safe networks that may be available. As the project advances several networks may coexist with the main Safe Network, there could be Safe networks available for testing upcoming features, or networks that are local to the user in their own computer or WAN/LAN.

The way Safe applications currently connect to a Safe network is by reading the connection information from a specific location in the system: `~/.safe/node/node_connection_info.config`

Currently, there is a public test network accessible to anyone. Users may also have local nodes running in their own environment creating a local network. The CLI allows users to easily create a list of different Safe networks in its config settings, to then be able to switch between them with just a simple command.

#### Node install

Let's first look at how to run a local Safe network using the CLI. A local network is bootstrapped by running several Safe Network nodes which automatically interconnect forming a network. We therefore first need to install the Safe Network node in our system:
```shell
$ safe node install
Latest release found: sn_node v0.25.14
Downloading https://sn-node.s3.eu-west-2.amazonaws.com/sn_node-0.25.14-x86_64-unknown-linux-musl.zip...
[00:00:08] [========================================] 9.34MB/9.34MB (0s) Done
Installing sn_node binary at ~/.safe/node ...
Setting execution permissions to installed binary '~/.safe/node/sn_node'...
Done!
```

#### Run a local network

At the current state of the Safe project, a single-section Safe network can be launched locally in our system. If the Safe Network node was installed in the system using the CLI as described in the previous section we can then launch it with a simple command:
```shell
$ safe node run-baby-fleming
Creating '~/.safe/node/baby-fleming-nodes' folder
Storing nodes' generated data at ~/.safe/node/baby-fleming-nodes
Launching local Safe network...
Launching with node executable from: ~/.safe/node/sn_node
Version: sn_node 0.26.8
Network size: 11 nodes
Launching genesis node (#1)...
Connection info directory: ~/.safe/node/node_connection_info.config
Genesis node contact info: ["127.0.0.1:12000"]
Common node args for launching the network: ["-vv", "--idle-timeout-msec", "5500", "--keep-alive-interval-msec", "4000", "--local"]
No RUST_LOG override provided
Launching node #2...
Launching node #3...
Launching node #4...
Launching node #5...
Launching node #6...
Launching node #7...
Launching node #8...
Launching node #9...
Launching node #10...
Launching node #11...
Done!
```

Once the local network is running, the connection configuration file will be already in the correct place for the CLI to connect to it. Thus from this point on, you can simply use the CLI to connect to your local network.

In order to shutdown a running local network, the following CLI command can be invoked to kill all running sn_node processes:
```shell
$ safe node killall
Success, all processes instances of sn_node were stopped!
```

##### Run a local network for testing: `--test`

The `run-baby-fleming` command accepts a `--test` or `-t` flag to automatically create a new Safe and authorise the CLI for test purposes. This requires that the `sn_node`, `sn_authd` and CLI themselves be installed in the correct locations on the system

#### Connect to a shared network

Ready to play your part in a shared network by adding your node from home to a single section with other people's nodes? Keep reading...

MaidSafe are currently hosting some bootstrap nodes on Digital Ocean to kickstart a single section, you can bootstrap using these nodes as hardcoded contacts, then watch the logs as your node joins the network, progresses to Adult status, and plays its part in hosting Immutable Data Chunks. Of course you will also be able to create a Safe on this network, unlock it, upload data, create keys and wallets, and all the other commands described in this user guide. This guide will take you through connecting to this MaidSafe started network, but of course it can be applied to connecting to any shared section, hosted by anyone.

You will need the network configuration containing the details of the hardcoded contacts that will bootstrap you to the shared section. If you have connected to this or previous iterations of the MaidSafe shared section then you may already have a `maidsafe-testnet` network profile saved on your machine. You can confirm this and update it to the latest configuration using `safe networks check`:
```shell
$ safe networks check
Checking current setup network connection information...
Fetching 'my-network' network connection information from '~/.config/sn_cli/networks/my-network_node_connection_info.config' ...
Fetching 'maidsafe-testnet' network connection information from 'https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config' ...

'maidsafe-testnet' network matched!
Current set network connection information at '~/.config/sn_node/node_connection_info.config' matches 'maidsafe-testnet' network as per current config
```

If you don't have a configuration in your results which points to the exact [S3 location](https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config) listed in the results above, you can add using `safe networks add`:
```shell
$ safe networks add maidsafe-testnet https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config
Network 'maidsafe-testnet' was added to the list. Connection information is located at 'https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config'
```

Now you need to ensure you are set to use this `maidsafe-testnet` configuration that we have updated/added, we can use `safe networks switch maidsafe-testnet` for this:
```shell
$ safe networks switch maidsafe-testnet
Switching to 'maidsafe-testnet' network...
Fetching 'maidsafe-testnet' network connection information from 'https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config' ...
Successfully switched to 'maidsafe-testnet' network in your system!
If you need write access to the 'maidsafe-testnet' network, you'll need to restart authd (safe auth restart), unlock a Safe and re-authorise the CLI again
```

We're now ready to launch our node and add it as a node. This is achieved using `safe node join` as follows:
```shell
$ safe node join
Joining network with contacts {161.35.36.185:12000}...
Creating '~/.safe/node/local-node' folder
Storing nodes' generated data at ~/.safe/node/local-node
Starting a node to join a Safe network...
Launching with node executable from: ~/.safe/node/sn_node
Version: sn_node 0.26.8
Node to be started with contact(s): ["161.35.36.185:12000"]
Launching node...
Node logs are being stored at: ~/.safe/node/local-node/sn_node.log
```

Your node will now launch and attempt to connect to the shared network. You can keep an eye on its progress via its logs, which can be found at `~/.safe/node/local-node/sn_node.log`.

Note that at the time of writing nodes from home is being restricted to those with home routers which correctly implement [IGD](https://en.wikipedia.org/wiki/Internet_Gateway_Device_Protocol). This will be expanded imminently to include those with routers which don't support IGD, with instructions added here for manual port forwarding at that point. If your log file states `Automatic Port forwarding Failed` then be on stand by for the next iteration.

#### Switch networks

MaidSafe currently hosts a test network for those who don't want to run a local network but still have a go at using the CLI and client applications. It's very common for users testing and experimenting with CLI and Safe applications to have a local network running, but switching to use the MaidSafe hosted network, back and forth, is also quite common.

The CLI allows you to set up a list of networks in its config settings for easily switching to connect to them. If you just launched a local network, you can keep current connection information as a configured network on CLI with the following command:
```shell
$ safe networks add my-network
Caching current network connection information into: ~/.config/sn_cli/networks/my-network_node_connection_info.config
Network 'my-network' was added to the list. Connection information is located at '~/.config/sn_cli/networks/my-network_node_connection_info.config'
```

If you also would like to connect to the MaidSafe hosted test network, you would need to set it up in CLI settings as another network, specifying the URL where to fetch latest connection information from:
```shell
$ safe networks add maidsafe-testnet https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config
Network 'maidsafe-testnet' was added to the list
```

We can also retrieve the list of the different networks that were set up in the CLI config:
```shell
$ safe networks
+----------+------------------+------------------------------------------------------------------------------------------------+
| Networks |                  |                                                                                                |
+----------+------------------+------------------------------------------------------------------------------------------------+
| Current  | Network name     | Connection info                                                                                |
+----------+------------------+------------------------------------------------------------------------------------------------+
|          | maidsafe-testnet | https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config                  |
+----------+------------------+------------------------------------------------------------------------------------------------+
| *        | my-network       | ~/.safe/cli/networks/my-network_node_connection_info.config                                    |
+----------+------------------+------------------------------------------------------------------------------------------------+
```

Once we have them in the CLI settings, we can use the CLI to automatically fetch the connection information data using the configured location, and place it at the right location in the system for Safe applications to connect to the selected network. E.g. let's switch to the 'maidsafe-testnet' network we previously configured:
```shell
$ safe networks switch maidsafe-testnet
Switching to 'maidsafe-testnet' network...
Fetching 'maidsafe-testnet' network connection information from 'https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config' ...
Successfully switched to 'maidsafe-testnet' network in your system!
If you need write access to the 'maidsafe-testnet' network, you'll need to restart authd (safe auth restart), unlock a Safe and re-authorise the CLI again
```

Remember that every time you launch a local network the connection configuration in your system is automatically overwritten with new connection information. Also, if the test network was restarted by MaidSafe, the new connection information is published in the same URL and needs to be updated in your system to be able to successfully connect to it. Thus if you want to make sure your current setup network matches any of those set up in the CLI config, you can use the `check` subcommand:
```shell
$ safe networks check
Checking current setup network connection information...
Fetching 'my-network' network connection information from '~/.config/sn_cli/networks/my-network_node_connection_info.config' ...
Fetching 'maidsafe-testnet' network connection information from 'https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config' ...

'maidsafe-testnet' network matched!
Current set network connection information at '~/.config/sn_node/node_connection_info.config' matches 'maidsafe-testnet' network as per current config
```

Note that in the scenario that your current network is set to be the MaidSafe test network, and that is restarted by MaidSafe (which causes new connection information to be published at the same URL), you then only need to re-run the `networks switch` command with the corresponding network name to update your system with the new connection information.

#### Set network bootstrap address

Another way to add a network to the CLI config settings is by directly mapping a network name to its bootstrapping address/es (IPs and ports). This can be achieved by using the `networks set` subcommand:
```shell
$ safe networks set community-network 161.35.36.112:15000
Network 'community-network' was added to the list. Contacts: '{161.35.36.112:15000}'
```

And as we did before, we could then switch to use this network using its name:
```shell
$ safe networks switch community-network
Switching to 'community-network' network...
Successfully switched to 'community-network' network in your system!
If you need write access to the 'community-network' network, you'll need to restart authd (safe auth restart), unlock a Safe and re-authorise the CLI again
```

If now check the list of networks we have in the CLI config settings we can see the 'community-network' is listed as the one currently set:
```shell
$ safe networks
+----------+-------------------+------------------------------------------------------------------------------------------------+
| Networks |                   |                                                                                                |
+----------+-------------------+------------------------------------------------------------------------------------------------+
| Current  | Network name      | Connection info                                                                                |
+----------+-------------------+------------------------------------------------------------------------------------------------+
| *        | community-network | {161.35.36.112:15000}                                                                          |
+----------+-------------------+------------------------------------------------------------------------------------------------+
|          | maidsafe-testnet  | https://sn-node.s3.eu-west-2.amazonaws.com/config/node_connection_info.config                  |
+----------+-------------------+------------------------------------------------------------------------------------------------+
|          | my-network        | ~/.safe/cli/networks/my-network_node_connection_info.config                                    |
+----------+-------------------+------------------------------------------------------------------------------------------------+
```

#### Node update

The node binary can be updated to the latest available version:
```shell
$ safe node update
```

This command will check if a newer sn_node release is available on [GitHub](https://github.com/maidsafe/sn_node/releases). After prompting to confirm if you want to take the latest version, it will be downloaded and the binary will be updated. By default it will assume the sn_node binary is at `~/.safe/node/`, but you can override that path by providing `--node-path <path>` argument to the above command.

### Auth

The CLI is just another client Safe application, therefore it needs to be authorised by the user to gain access to the Safe Network on behalf of the user. The `auth` command allows us to obtain such authorisation from the user via the Safe Authenticator.

This command simply sends an authorisation request to the Authenticator available, e.g. the `sn_authd` daemon (see further below for an explanation of how to run it), and it then stores the authorisation response (credentials) in the user's `~/.safe/cli/credentials` file. Any subsequent CLI command will read this file to obtain the credentials and connect to the Safe Network for the corresponding operation.

#### The Authenticator daemon (authd)

In order to be able to allow any Safe application to connect to the Network and have access to your data, we need to start the Safe Authenticator daemon (authd). This application exposes an interface as a [QUIC (Quick UDP Internet Connections)](https://en.wikipedia.org/wiki/QUIC) endpoint, which Safe applications will communicate with to request for access permissions. These permissions need to be reviewed by the user and approved, which can be all done with the Safe CLI as we'll see in this guide.

The Safe Authenticator, which runs as a daemon, can be started and managed with the Safe CLI if the `sn_authd`/`sn_authd.exe` binary is properly installed in the system.

#### Auth install

Downloading and installing the Authenticator daemon is very simple:
```shell
$ safe auth install
Latest release found: sn_authd v0.0.3
Downloading https://sn-api.s3.eu-west-2.amazonaws.com/sn_authd-0.0.3-x86_64-unknown-linux-musl.tar.gz...
[00:00:25] [========================================] 6.16MB/6.16MB (0s) Done
Installing sn_authd binary at ~/.safe/authd ...
Setting execution permissions to installed binary '~/.safe/authd/sn_authd'...
Done!
```

#### Auth start

In order to start the `Safe Authenticator daemon (sn_authd)` so it can start receiving requests we simply need to run the following command:
```shell
$ safe auth start
Starting Safe Authenticator daemon (sn_authd)...
sn_authd started (PID: <pid>)
```

#### Auth status

Once we started the `authd`, it should be running in the background and ready to receive requests, we can send a status request to check it's up and running:
```shell
$ safe auth status
Sending request to authd to obtain a status report...
+------------------------------------------+-------+
| Safe Authenticator status                |       |
+------------------------------------------+-------+
| Authenticator daemon version             | 0.0.3 |
+------------------------------------------+-------+
| Is there a Safe currently unlocked?      | No    |
+------------------------------------------+-------+
| Number of pending authorisation requests | 0     |
+------------------------------------------+-------+
| Number of notifications subscribers      | 0     |
+------------------------------------------+-------+
```

#### Auth create

Since we now have our Safe Authenticator running and ready to accept requests, we can start interacting with it by using other Safe CLI `auth` subcommands.

In order to create a Safe on the network, we need some `safecoins` to pay with. Since this is still under development, we can have the CLI to generate some test-coins and use them for paying the cost of creating a Safe. We can do so by passing `--test-coins` flag to the `create` subcommand. The CLI will request us to enter a passphrase and password for the new account to be created:
```shell
$ safe auth create --test-coins
Passphrase:
Password:
Sending request to authd to create a Safe...
Safe was created successfully!
```

Alternatively, if we own some safecoins on a `SafeKey` already (see [`SafeKeys` section](#safekeys) for details about `SafeKey`s), we can provide the corresponding secret key to the safe CLI to use it for paying the cost of creating the account, as well as setting it as the default `SafeKey` for the account being created:
```shell
$ safe auth create
Passphrase:
Password:
Enter SafeKey's secret key to pay with:
Sending request to authd to create a Safe...
Safe was created successfully!
```

#### Auth unlock

When a new Safe is created with CLI, as we've seen above, we can unlock it using the following command:
```shell
$ safe auth unlock
Passphrase:
Password:
Sending action request to authd to unlock the Safe...
Safe unlocked successfully
```

If we now send a status report request to `authd`, it should now show that a Safe is currently unlocked:
```shell
$ safe auth status
Sending request to authd to obtain a status report...
+------------------------------------------+-------+
| Safe Authenticator status                |       |
+------------------------------------------+-------+
| Authenticator daemon version             | 0.0.3 |
+------------------------------------------+-------+
| Is there a Safe currently unlocked?      | Yes   |
+------------------------------------------+-------+
| Number of pending authorisation requests | 0     |
+------------------------------------------+-------+
| Number of notifications subscribers      | 0     |
+------------------------------------------+-------+
```

The Safe Authenticator is now ready to receive authorisation requests from any Safe application, including the Safe CLI which needs to also get permissions to perform any data operations on behalf of our account.

##### Passing credentials from a config file

It's possible (though not secure) to use a simple json file to pass the passphrase and password to the auth commands, and so avoid having to manually input both, either when creating a Safe or when unlocking it. E.g., having a file named `my-config.json` with:
```
{
  "passphrase": "mypassphrase",
  "password": "mypassword"
}
```
And so you can unlock the Safe with:
```shell
$ safe auth unlock --config ./my-config.json
Sending action request to authd to unlock the Safe...
Safe unlocked successfully
```

##### Using environment variables

Another method for passing passphrase/password involves using the environment variables `SAFE_AUTH_PASSPHRASE` and `SAFE_AUTH_PASSWORD`.

With those set (eg, on Linux/macOS: `export SAFE_AUTH_PASSPHRASE="<your passphrase>;"`, and `export SAFE_AUTH_PASSWORD="<your password>"`), you can then unlock a Safe without needing to enter this information, or pass a config file:
```shell
$ safe auth unlock
Sending action request to authd to unlock the Safe...
Safe unlocked successfully
```
Or, you can choose to pass the environment variables to the command directly (though this can be insecure):
```shell
$ SAFE_AUTH_PASSPHRASE="<passphrase>" SAFE_AUTH_PASSWORD="<password>" safe auth unlock
Sending action request to authd to unlock the Safe...
Safe unlocked successfully
```
Please note, that both the passphrase and password environment variables must be set to use this method. If only one is set, an error will be thrown.

#### Auth reqs

Now that the Authenticator is running and ready to authorise applications, we can try to authorise the CLI application.

In a normal scenario, an Authenticator GUI would be using `authd` as its backend process, e.g. the [Safe Network Application](https://github.com/maidsafe/sn_app) provides such a GUI to review authorisation requests and allow the permissions requested to be granted.

For the purpose of making this guide self-contained with the Safe CLI application, we will now use also the CLI on a second console to review and allow/deny authorisation requests.

Let's first send an authorisation request from the current console by simply invoking the `auth` command with no subcommands:
```shell
$ safe auth
Authorising CLI application...
```

The CLI application is now waiting for an authorisation response from the `authd`.

We can now open a second console which we'll use to query `authd` for pending authorisation requests, and also to allow/deny them (remember the following steps wouldn't be needed if we had any other Authenticator UI running, like the `Safe Network App`).

Once we have a second console, we can start by fetching from `authd` the list of authorisation requests pending for approval/denial:
```shell
$ safe auth reqs
Requesting list of pending authorisation requests from authd...
+--------------------------------+------------------+----------+------------------+-------------------------+
| Pending Authorisation requests |                  |          |                  |                         |
+--------------------------------+------------------+----------+------------------+-------------------------+
| Request Id                     | App Id           | Name     | Vendor           | Permissions requested   |
+--------------------------------+------------------+----------+------------------+-------------------------+
| 584798987                      | net.maidsafe.cli | Safe CLI | MaidSafe.net Ltd | Own container: No       |
|                                |                  |          |                  | Transfer coins: Yes     |
|                                |                  |          |                  | Mutations: Yes          |
|                                |                  |          |                  | Read coin balance: Yes  |
|                                |                  |          |                  | Containers: None        |
+--------------------------------+------------------+----------+------------------+-------------------------+
```

We see there is one authorisation request pending for approval/denial, which is the one requested by the CLI application from the other console.

#### Auth allow/deny

In order to allow any pending authorisation request we use its request ID (e.g. '584798987' from above), the `authd` will then proceed to send a response back to the CLI with the corresponding credentials it can use to connect directly with the Network:
```shell
$ safe auth allow 584798987
Sending request to authd to allow an authorisation request...
Authorisation request was allowed.
```

Note we could have otherwise decided to deny this authorisation request and invoke `$ safe auth deny 584798987` instead, but let's allow it so we can continue with the next steps of this guide.

If we now switch back to our previous console, the one where we sent the authorisation request with `$ safe auth` command from, we will see the Safe CLI receiving the response from `authd`. You should see in that console a message like the following:
```shell
Safe CLI app was successfully authorised
Credentials were stored in ~/.safe/cli/credentials
```

We are now ready to start using the CLI to operate with the network, via its commands and supported operations!.

##### Self authorising the CLI application

It could be the case the Safe CLI is the only Safe application that the user is intended to use to interact with the Safe Network. In such a case authorising the CLI application as explained above (when there is no other UI for the `authd`) using another instance of the CLI in a second console is not that comfortable.

Therefore there is an option which allows the Safe CLI to automatically self authorise when the user unlocks a Safe using the CLI, which is as simple as:
```shell
$ safe auth unlock --self-auth
Passphrase:
Password:
Sending action request to authd to unlock the Safe...
Safe unlocked successfully
Authorising CLI application...
Safe CLI app was successfully authorised
Credentials were stored in ~/.safe/cli/credentials
```

#### Auth update

The Authenticator binary (`sn_authd`/`sn_authd.exe`) can be updated to the latest available version using the CLI:
```shell
$ safe auth update
```
It will check if a newer release is available on [Amazon S3](https://sn-api.s3.eu-west-2.amazonaws.com). After prompting to confirm if you want to take the latest version, it will be downloaded and the sn_authd binary will be updated.

After the sn_authd was updated, you'll need to restart it to start using new version:
```shell
$ safe auth restart
Stopping Safe Authenticator daemon (sn_authd)...
Success, sn_authd (PID: <pid>) stopped!
Starting Safe Authenticator daemon (sn_authd)...
sn_authd started (PID: <new pid>)
Success, sn_authd restarted!
```

### The interactive shell

When the CLI is invoked without any command, it enters into an interactive shell, which allows the user to run commands within a shell:
```shell
$ safe

Welcome to Safe CLI interactive shell!
Type 'help' for a list of supported commands
Pass '--help' flag to any top level command for a complete list of supported subcommands and arguments
Type 'quit' to exit this shell. Enjoy it!

>
```

The interactive shell supports all the same commands and operations that can be performed in the command line. E.g., we can use the `auth status` command to retrieve a status report from the `authd`:
```shell
> auth status
Sending request to authd to obtain a status report...
+------------------------------------------+-------+
| Safe Authenticator status                |       |
+------------------------------------------+-------+
| Authenticator daemon version             | 0.0.3 |
+------------------------------------------+-------+
| Is there a Safe currently unlocked?      | No    |
+------------------------------------------+-------+
| Number of pending authorisation requests | 0     |
+------------------------------------------+-------+
| Number of notifications subscribers      | 0     |
+------------------------------------------+-------+
```

As you can see, the commands operate in an analogous way as when they are invoked outside of the interactive shell. Although there are some operations which are only possible when they are executed from the interactive shell, one nice example is the possibility to subscribe to receive authorisation request notifications, let's see how that works.

In the previous section we've used the `safe auth reqs` command to obtain a list of the authorisation requests which are waiting for approval/denial. We could instead use the interactive shell to subscribe it as an endpoint to receive notifications when this authorisation requests are sent to the `authd`:
```shell
> auth subscribe
Sending request to subscribe...
Subscribed successfully
Keep this shell session open to receive the notifications
```

This is telling us that as long as we keep this session of the interactive shell open, we will be notified of any new authorisation request, such notifications are being sent by the `authd` to our interactive shell session. Thus if we have any other Safe app which is sending an authorisation request to `authd`, e.g. the Safe Browser, a `safe auth` command invoked from another instance of the CLI, etc., we will be notified by the interactive shell:
```shell
>
A new application authorisation request was received:
+------------+------------------+---------+---------+-------------------------+
| Request Id | App Id           | Name    | Vendor  | Permissions requested   |
+------------+------------------+---------+---------+-------------------------+
| 754801191  | net.maidsafe.cli | Unknown | Unknown | Own container: No       |
|            |                  |         |         | Transfer coins: Yes     |
|            |                  |         |         | Mutations: Yes          |
|            |                  |         |         | Read coin balance: Yes  |
|            |                  |         |         | Containers: None        |
+------------+------------------+---------+---------+-------------------------+
You can use "auth allow"/"auth deny" commands to allow/deny the request respectively, e.g.: auth allow 754801191
Press Enter to continue
```

The notification message contains the same information we can obtain with `safe auth reqs` command. We can now do the same as before and allow/deny such a request using its ID, in this case '754801191':
```shell
> auth allow 754801191
Sending request to authd to allow an authorisation request...
Authorisation request was allowed
```

The interactive shell will be expanded to support many more operations, and especially to cover the use cases which are not possible to cover with the non-interactive shell, like the use case we've seen of receiving notifications from `authd`.

It enables the possibility to also have a state in the session, e.g. allowing the user to set a wallet to be used for all operations within that session instead of using the default wallet from the account, ...or several other use cases and features we'll be adding as we move forward in its development.

### SafeKeys

`SafeKey` management allows users to generate sign/encryption key pairs that can be used for different types of operations, like choosing which sign key to use for uploading files (and therefore paying for the storage used), or signing a message posted on some social application when a `SafeKey` is linked from a public profile (e.g. a WebID/SAFE-ID), or even for encrypting messages that are privately sent to another party so it can verify the authenticity of the sender.

Users can record `SafeKey`s in a `Wallet` (see further below for more details about `Wallet`s), having friendly names to refer to them, but they can also be created as throw away `SafeKey`s which are not linked from any `Wallet`, container, or any other type of data on the network.

Note that the key pair is automatically generated by the CLI, although `SafeKey`s don’t hold the secret key on the network but just represent the public key. `SafeKey`s can also be used for safecoin transfers. In this sense, a `SafeKey` can be compared to a Bitcoin address, it has a coin balance associated to it, such balance can be only queried using the secret key, and in order to spend its balance the corresponding secret key needs to be provided in the `transfer` request to the network. The secret key can be provided by the user, or retrieved from a `Wallet`, at the moment of creating the transfer (again, see the [`Wallet` section](#wallet) below for more details).

#### SafeKeys Creation

To generate a `SafeKey`:
```shell
$ safe keys create 
New SafeKey created: "safe://bbkulcbnrmdzhdkrfb6zbbf7fisbdn7ggztdvgcxueyq2iys272koaplks"
Key pair generated:
Public Key = b62c1e4e3544a1f64212fca89046df98d998ea615e84c4348c4b5fd29c07ad52a970539df819e31990c1edf09b882e61
Secret Key = c4cc596d7321a3054d397beff82fe64f49c3896a07a349d31f29574ac9f56965
```


### Files

#### [ Warning: Underlying API to be deprecated ]

The underlying files APIs used in the CLI, and perhaps much of the CLI will be deprecated in order to use [`sn_fs`](https://github.com/maidsafe/sn_fs) at some point. This is a POSIX based filesystem which is much more comprehensive and performant. It's not yet known the impact (if any) this will have on the CLI commands. But if you're interested in a filesystem on Safe, `sn_fs` is most definitely where you should be looking at the moment.

#### Files...

Uploading files and folders onto the network is also possible with the CLI application, and as we'll see here it's extremely simple to not just upload them but also keep them in sync with any modifications made locally to the folders and files.

Files are uploaded on the Network and stored as `Public Blob` files, and the folders and sub-folders hierarchy is flattened out and stored in a container mapping each file's path with the corresponding `Blob` XOR-URL. This map is maintained on the Network in a special container called `FilesContainer`, which is stored as `Public Sequence` data. The data representation in the `FilesContainer` is planned to be implemented with [RDF](https://en.wikipedia.org/wiki/Resource_Description_Framework) and the corresponding `FilesContainer` RFC will be submitted, but at this stage this is being done only using a simple serialised structure.

#### Files Put

The most simple scenario is to upload all files and subfolders found within a local `./to-upload/` directory, recursively, onto a `FilesContainer` on the Network, obtaining the XOR-URL of the newly created container, as well as the XOR-URL of each of the files uploaded:
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

We can additionally pass a destination path argument to set a base path for each of the paths in the `FilesContainer`, e.g. if we provide `/mychosenroot/` destination path argument to the `files put` command when uploading the above files, they will be published on the `FilesContainer` with paths `/mychosenroot/file1.txt`, `/mychosenroot/myfolder/file2.txt`, and `/mychosenroot/myotherfolder/subfolder/file3.txt` respectively. This can be verified by querying the `FilesContainer` content with the `safe cat` command, please see further below for details of how this command works.

#### Files Sync

Once a set of files, folders and subfolders, have been uploaded to the Network onto a `FilesContainer` using the `files put` command, local changes made to those files and folders can be easily synced up using the `files sync` command. This command takes care of finding the differences/changes on the local files and folders, creating new `Public Blob` files as necessary, and updating the `FilesContainer` by publishing a new version of it at the same location on the Network.

The `files sync` command follows a very similar logic to the well known `rsync` command, supporting a subset of its functionality. This subset will gradually be expanded with more supported features. Users knowing how to use `rsync` can easily start using the Safe CLI and the Safe Network for uploading files and folders, making it also easy to integrate existing automated systems which are currently making use of `rsync`.

As an example, let's suppose we upload all files and subfolders found within the `./to-upload/` local directory, recursively, using `files put` command:
```shell
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc"
+  ./to-upload/another.md              safe://hbhyrydt5b95dmumcm8yig4u1keuuh8hgsr5yx39xn4mqikp91sbdhbpwp
+  ./to-upload/subfolder/subexists.md  safe://hbhyryn9uodh1ju5uzyti3gmmtwburrssd89rcwcy3rzofdpypwomrzzte
+  ./to-upload/subfolder/note.md       safe://hbhyryncjzga5uqp3ogeadqctigyaurpju8yauqptzgh5uyctogh3dkcbt
+  ./to-upload/test.md                 safe://hbhyrydpan7d94mwp1bun3mxfnrfrui131an7ihu11wsn8dkr8odab9qwn
```

All the content of the `./to-upload/` local directory is now stored and published on the Safe Network. Now, let's say we make the following changes to our local files within the `./to-upload/` folder:
- We edit `./to-upload/another.md` and change its content
- We create a new file at `./to-upload/new.md`
- And we remove the file `./to-upload/test.md`

We can now sync up all the changes we just made, recursively, with the `FilesContainer` we previously created:
```shell
$ safe files sync ./to-upload/ safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc --recursive --delete
FilesContainer synced up (version 1): "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc?v=1"
*  ./to-upload/another.md  safe://hbhyrynyr3osimhxa3mfqok7tto6cf3hhjy4sp3wdri6ee46x8xg68r9mj
+  ./to-upload/new.md      safe://hbhyrydky3ga3xgkneiy1y5o6513rq6wdipqthkhd3ujqci9qmy8weihom
-  /test.md                safe://hbhyrydpan7d94mwp1bun3mxfnrfrui131an7ihu11wsn8dkr8odab9qwn
```

The `*`, `+` and `-` signs mean that the files were updated, added, and removed respectively.

Also, please note we provided the optional `--delete` flag to the command above which forces the deletion of the files found at the targeted `FilesContainer` that are not found in the source location, like the case of `./to-upload/test.md` file in our example above. If we didn't provide such flag, only the modification and creation of files would have been updated on the `FilesContainer`, like the case of `./to-upload/another.md` and `./to-upload/new` files in our example above. Note that `--delete` is only allowed if the `--recursive` flag is also provided.

The `files sync` command also supports to be passed a destination path as the `files put` command, but in this case the destination path needs to be provided as part of the target XOR-URL. E.g., we can sync a `FilesContainer` using the local path and provide a specific destination path `new-files` in the target XOR-URL:
```shell
$ safe files sync ./other-folder/ safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc/new-files
FilesContainer synced up (version 2): "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc?v=2"
+  ./other-folder/file1.txt  safe://hbhydyn6b5x9nqxt5escpuzy3axrcqb9dgs7p74izmpfkmmquwrdgjig4k
```

The `./other-folder/file1.txt` file will be uploaded and published in the `FilesContainer` with path `/new-files/file1.txt`.

One more thing to note about `files sync` command is the use of the `--update-nrs` flag. When syncing content using an NRS-URL (see [NRS section](#nrs-name-resolution-system) below for more information about NRS names and commands), if you want to update the NRS name to the new version generated after syncing the target `FilesContainer`, then it can be specified using the `--update-nrs` flag:
```shell
$ safe files sync ./to-upload/ safe://mywebsite --update-nrs
FilesContainer synced up (version 1): "safe://mywebsite"
*  ./to-upload/another.md  safe://hbhyrynyr3osimhxa3mfqok7tto6cf3hhjy4sp3wdri6ee46x8xg68r9mj
+  ./to-upload/new.md      safe://hbhyrydky3ga3xgkneiy1y5o6513rq6wdipqthkhd3ujqci9qmy8weihom
```

#### Files Add

It could be desirable in some scenarios to simply add a file to a `FilesContainer` rather than having the CLI to sync up a complete local folder, so the `files add` command could be used in such cases.

We can add a single file from a local path, let's say `./some-other-folder/file.txt`, to our existing `FilesContainer` on the Safe Network with the following command:
```shell
$ safe files add ./some-other-folder/file.txt safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc/files-added/just-a-file.txt
FilesContainer updated (version 3): "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc?v=3"
+  ./some-other-folder/file.txt  safe://hbhydynx64dxu5594yunsu41dykxt3nu1be81cy9igqzz3qtqrq1w3y6d9
```

If we have previously uploaded a file to the network, we can also add it to any existing `FilesContainer` by providing its XOR-URL as the `<location>` argument to the `files add` command. Let's add a file (the same file we uploaded in the previous command) to our `FilesContainer` again, but choosing a new destination filename, e.g. `/files-added/same-file.txt`:
```shell
$ safe files add safe://hbhydynx64dxu5594yunsu41dykxt3nu1be81cy9igqzz3qtqrq1w3y6d9 safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc/files-added/same-file.txt
FilesContainer updated (version 4): "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc?v=4"
+  /files-added/same-file.txt  safe://hbhydynx64dxu5594yunsu41dykxt3nu1be81cy9igqzz3qtqrq1w3y6d9
```

#### Files Ls

We can list the files from a `FilesContainer` using the `files ls` command:
```shell
$ safe files ls safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc
Files of FilesContainer (version 4) at "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc":
Total: 6
SIZE  CREATED               MODIFIED              NAME
11    2020-01-28T20:26:05Z  2020-01-28T20:29:04Z  another.md
38    2020-01-28T20:35:43Z  2020-01-28T20:35:43Z  files-added/
30    2020-01-28T20:31:01Z  2020-01-28T20:31:01Z  new-files/
10    2020-01-28T20:29:04Z  2020-01-28T20:29:04Z  new.md
23    2020-01-28T20:26:05Z  2020-01-28T20:26:05Z  subfolder/
```

Note the size displayed for a subfolder is its total size taking into account all files contained within it.

If we provide a path to a subfolder of the `FilesContainer`, this command will resolve the path and list only those files which paths are a child of the provided path:
```shell
$ safe files ls safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc/subfolder
Files of FilesContainer (version 4) at "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc/subfolder":
Total: 1
SIZE  CREATED               MODIFIED              NAME
23    2020-01-28T20:26:05Z  2020-01-28T20:26:05Z  subexists.md
8     2020-01-28T20:26:05Z  2020-01-28T20:26:05Z  note.md
```

#### Files Get

The `files get` command copies file(s) from the network to the local filesystem.

This command works similarly to Unix `cp` or `scp` or the windows `copy` command.  It accepts two arguments:

```shell
<source>  The target FilesContainer to retrieve from, optionally including the path to the directory or file within
<dest>    The local destination path for the retrieved files and folders (default is '.')
```

note: Wildcards (eg, *.txt) and set/range expansion (eg photo{1-3}.jpg, photo{1,3,5}.jpg ) in
the source URL path are not supported at this time, but are planned for a future release.


It also accepts some unique flags/options:

```shell
-e, --exists <exists>         How to handle pre-existing files [default: ask]  [possible values: ask, preserve, overwrite]
-i, --progress <progress>     How to display progress [default: bars]  [possible values: bars, text, none]
```

##### Example: retrieving contents of a file container to local working directory
```shell
$ safe files get safe://hnyynywwu865s4zgxj5z9gdjynpz9z93n8ru68931odfio7ogkjco7er7abnc
[00:00:00] [########################################] 45B/45B (329B/s, 0s) Transfer
Done. Retrieved 5 files to .
```

##### Example: retrieving subfolder in a file container to an existing local directory.

```shell
$ safe files get safe://hnyynywwu865s4zgxj5z9gdjynpz9z93n8ru68931odfio7ogkjco7er7abnc/testdata/subfolder existing_dir
[00:00:00] [########################################] 27B/27B (425B/s, 0s) Transfer
Done. Retrieved 2 files to existing_dir
```

We see that `subfolder` has been placed inside `existing_dir`.

```shell
$ tree existing_dir
existing_dir
└── subfolder
    ├── sub2.md
    └── subexists.md
```

##### Example: retrieving subfolder in a file container to a non-existent local directory (rename)

```shell
$ safe files get safe://hnyynywwu865s4zgxj5z9gdjynpz9z93n8ru68931odfio7ogkjco7er7abnc/testdata/subfolder nonexistent_dir
[00:00:00] [########################################] 27B/27B (425B/s, 0s) Transfer
Done. Retrieved 2 files to nonexistent_dir
```

We see that `subfolder` has been renamed to `nonexistent_dir`.

```shell
$ tree nonexistent_dir
nonexistent_dir
├── sub2.md
└── subexists.md
```

##### Example: Retrieving individual file to an existing directory

```shell
$ safe files get safe://hnyynywwu865s4zgxj5z9gdjynpz9z93n8ru68931odfio7ogkjco7er7abnc/testdata/subfolder/sub2.md existing_dir/
[00:00:00] [########################################] 4B/4B (378B/s, 0s) Transfer
Done. Retrieved 1 files to existing_dir/
```

We see that the file is now inside `existing_dir`.

```shell
$ tree existing_dir
existing_dir
└── sub2.md
```

##### Example: Retrieving individual file to a new filename

```shell
$ safe files get safe://hnyynywwu865s4zgxj5z9gdjynpz9z93n8ru68931odfio7ogkjco7er7abnc/testdata/subfolder/sub2.md existing_dir/new_filename
[00:00:00] [########################################] 4B/4B (374B/s, 0s) Transfer
Done. Retrieved 1 files to existing_dir/new_filename
```

We see that `new_filename` is now inside `existing_dir`.

```shell
$ tree existing_dir
existing_dir
└── new_filename
```

##### A performance note about very large FileContainers

Subfolder or single-file downloads from a FileContainer with thousands of files
may be slower than expected.

This is because the entire FileContainer is fetched and locally filtered to
obtain the XorUrl for each file that matches the source URL path.

Future releases may operate differently.


#### Files Tree

The `files tree` command displays a visual representation of an entire directory tree.

```shell
$ safe files tree safe://hnyynyiodw4extpc7xh3dncfgsg4sjzsygru9k8omo988brz688oxkxhxgbnc
safe://hnyynyiodw4extpc7xh3dncfgsg4sjzsygru9k8omo988brz688oxkxhxgbnc
└── testdata
    ├── another.md
    ├── noextension
    ├── subfolder
    │   ├── sub2.md
    │   └── subexists.md
    └── test.md

2 directories, 5 files
```

If we provide a path to a subfolder of the `FilesContainer`, this command will resolve the path and list only those files which paths are a child of the provided path:

```shell
$ safe files tree safe://hnyynyiodw4extpc7xh3dncfgsg4sjzsygru9k8omo988brz688oxkxhxgbnc/testdata/subfolder
safe://hnyynyiodw4extpc7xh3dncfgsg4sjzsygru9k8omo988brz688oxkxhxgbnc/testdata/subfolder
├── sub2.md
└── subexists.md

0 directories, 2 files
```

File details can be displayed with the `--details` flag:

```shell
$ safe files tree --details safe://hnyynyiodw4extpc7xh3dncfgsg4sjzsygru9k8omo988brz688oxkxhxgbnc/testdata
SIZE  CREATED               MODIFIED              NAME
                                                  safe://hnyynyiodw4extpc7xh3dncfgsg4sjzsygru9k8omo988brz688oxkxhxgbnc/testdata
6     2020-03-06T18:31:55Z  2020-03-06T18:31:55Z  ├── another.md
0     2020-03-06T18:31:55Z  2020-03-06T18:31:55Z  ├── noextension
                                                  ├── subfolder
4     2020-03-06T18:31:55Z  2020-03-06T18:31:55Z  │   ├── sub2.md
23    2020-03-06T18:31:55Z  2020-03-06T18:31:55Z  │   └── subexists.md
12    2020-03-06T18:31:55Z  2020-03-06T18:31:55Z  └── test.md

1 directory, 5 files
```

#### Files Rm

Removing files from a `FilesContainer` which is in sync with a folder in the local file system can be done by simply removing them locally followed by a call to `files sync` command. If we otherwise are not in such a scenario and would like to remove files directly from a `FilesContainer` we can achieve it with the `file rm` command.

As an example, we can remove single file to our existing `FilesContainer` on the Safe Network with the following command:
```shell
$ safe files rm safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc/another.md
FilesContainer updated (version 5): "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc?v=5"
-  /another.md  safe://hbhyrynyr3osimhxa3mfqok7tto6cf3hhjy4sp3wdri6ee46x8xg68r9mj
```

Removing an entire subfolder from a `FilesContainer` rather than a single file is also possible, we just need to pass the `--recursive` flag and the path to the subfolder we would like to remove:
```shell
$ safe files rm safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc/subfolder --recursive
FilesContainer updated (version 6): "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc?v=6"
-  /subfolder/subexists.md  safe://hbhyryn9uodh1ju5uzyti3gmmtwburrssd89rcwcy3rzofdpypwomrzzte
-  /subfolder/note.md       safe://hbhyryncjzga5uqp3ogeadqctigyaurpju8yauqptzgh5uyctogh3dkcbt
```

### Xorurl

As we've seen, when uploading files to the network, each file is uploaded as an `Blob` using the [self-encryption algorithm](https://github.com/maidsafe/self_encryption) in the client, splitting the files into encrypted chunks, and the resulting file's XOR-URL is linked from a `FilesContainer`.

The file's XOR-URL is deterministic based on its content, i.e. the location where each of its chunks are stored is determined based on the file's content, and performed at the client before uploading the chunks to the network. Therefore the XOR-URL is always the same if the content of a file doesn't change. All this means is we can know what the file's XOR-URL will be without uploading it to the network.

Obtaining local files' XOR-URLs without uploading them to the network can be done in two different ways. We can use the `--dry-run` flag in any of the files commands, e.g.:
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
Native data type: PublicSequence
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
FilesContainer created at: "safe://hnyynyixxj9uewuhh64rgg9zsdhaynwhc88mpyfpor5carg8xx6qs6jknnbnc"
+  ./to-upload/another.md              safe://hbhyrynyr3osimhxa3mfqok7tto6cf3hhjy4sp3wdri6ee46x8xg68r9mj
+  ./to-upload/subfolder/subexists.md  safe://hbhyryn9uodh1ju5uzyti3gmmtwburrssd89rcwcy3rzofdpypwomrzzte
+  ./to-upload/test.md                 safe://hbhyrydpan7d94mwp1bun3mxfnrfrui131an7ihu11wsn8dkr8odab9qwn
```

We can then use `safe cat` command with the XOR-URL of the `FilesContainer` just created to render the list of files linked from it:
```shell
$ safe cat safe://hnyynyixxj9uewuhh64rgg9zsdhaynwhc88mpyfpor5carg8xx6qs6jknnbnc
Files of FilesContainer (version 0) at "safe://hnyynyixxj9uewuhh64rgg9zsdhaynwhc88mpyfpor5carg8xx6qs6jknnbnc":
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| Name                    | Size | Created              | Modified             | Link                                                              |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /another.md             | 11   | 2020-01-28T20:51:05Z | 2020-01-28T20:51:05Z | safe://hbhyrynyr3osimhxa3mfqok7tto6cf3hhjy4sp3wdri6ee46x8xg68r9mj |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /test.md                | 12   | 2020-01-28T20:51:05Z | 2020-01-28T20:51:05Z | safe://hbhyrydpan7d94mwp1bun3mxfnrfrui131an7ihu11wsn8dkr8odab9qwn |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
| /subfolder/subexists.md | 23   | 2020-01-28T20:51:05Z | 2020-01-28T20:51:05Z | safe://hbhyryn9uodh1ju5uzyti3gmmtwburrssd89rcwcy3rzofdpypwomrzzte |
+-------------------------+------+----------------------+----------------------+-------------------------------------------------------------------+
```

We could also take any of the XOR-URLs of the individual files and have the `cat` command fetch the content of the file and show it in the output, e.g. let's use the XOR-URL of the `/test.md` file to fetch its content:
```shell
$ safe cat safe://hbhyrydpan7d94mwp1bun3mxfnrfrui131an7ihu11wsn8dkr8odab9qwn
hello tests!
```

Alternatively, we could use the XOR-URL of the `FilesContainer` and provide the path of the file we are trying to fetch, in this case the `cat` command will resolve the path and follow the corresponding link to read the file's content directly for us. E.g. we can also read the content of the `/test.md` file with the following command:
```shell
$ safe cat safe://hnyynyixxj9uewuhh64rgg9zsdhaynwhc88mpyfpor5carg8xx6qs6jknnbnc/test.md
hello tests!
```

A `Wallet` can be also fetched with `cat` to inspect its content, the list of spendable balances it holds will be listed, and we can see which of them is currently the default to be used in a transfer operation:
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

As seen above, the `safe cat` command can be used to fetch any type of content from the Safe Network. At this point it only supports files (`Blob`), `FilesContainer`s, `Wallet`s, and `NRS-Container`s (see further below about NRS Containers and commands), but it will be expanded as more types are supported by the CLI and its API.

#### Retrieving binary files with --hexdump

By default, binary files are treated just like a plaintext file and will typically display unreadable garbage on the screen unless the output is redirected to a file, eg:

```shell
$ safe cat safe://hbwybynbbwotm5qykdfxuu4r4doogaywf8jupxats5zg39xjjtd8xmtpky > /tmp/favicon.ico
```

However, the flag --hexdump is available which provides a more human-friendly hexadecimal dump, similar to that of the standard *xxd* Unix tool.  Here's an example.

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

As we've seen above, we can use `cat` command to retrieve the latest/current version of any type of content from the Network using their URL. But every change made to content that is uploaded to the Network as `Public` data is perpetual, and therefore a new version is generated when performing any amendments to it, keeping older versions also available forever.

We can use the `cat` command to also retrieve any version of content that was uploaded by appending a query param to the URL. E.g. given the XOR-URL of the `FilesContainer` we created in previous sections (`safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc`), which reached version 2 after a couple of amendments we made with `files sync` command, we can retrieve the very first version (version 0) by using `v=<version>` query param:
```shell
$ safe cat "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc?v=0"
Files of FilesContainer (version 0) at "safe://hnyynyi6tgumo67yoauewe3ee3ojh37sbyr7rnh3nd6kkqhbo9decpjk64bnc?v=0":
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

As we've seen in all the above sections, every piece of data on the Safe Network has a unique location. Such location is determined by the XoR name given to it in the Network's XoR address space, as well as some other information which depends on the native date type, like in the case of `MutableData` data type which also has a type tag associated to it apart from its XoR address.

So far all commands were using the XOR-URLs to either inform of the new data created/stored on the Network, as well as for retrieving data from the Network.

While XOR-URLs are simply a way to encode Safe Network data unique location into a URL, there are some incentives for having more human-friendly URLs that can be easily remembered and recognisable when trying to share them with other people, or use them with tools and applications like the Safe CLI or the Safe Browser.

This is why the Safe Network also supports having such human-friendly URLs through what it's called the `Name Resolution System (NRS)`. The NRS allows users to create friendly names that are resolvable to a unique location on the Network. These friendly names can be used in the form of a URL (NRS-URL) to share with other people the location of websites, web applications, files and folders, safecoin wallets for receiving transfers, Safe IDs, etc.

In this section we will explore the CLI commands which allow users to generate, administer, and use the NRS and its NRS-URLs for publishing and retrieving data to/from the Safe Network.

#### NRS Create

Creating a friendly name on the Network can be achieved with the `nrs create` subcommand. This subcommand generates an NRS Container automatically linking it to any data we decide to link the friendly name to. An NRS Container is stored on the Network as a `Public Sequence` data, and it contains an NRS Map using RDF for its data representation (since this is still under development, pseudo-RDF data is now being used temporarily). This Map has a list of subnames and where each of them are being linked to, e.g. `mysubname` can be created as a subname of `mywebsite` NRS name by having `mysubname` linked to a particular `FilesContainer` XOR-URL so that it can be fetched with `safe://mysubname.mywebsite`.

Let's imagine we have uploaded the files and folders of a website we want to publish on the Safe Network with `files put` command:
```shell
$ safe files put ./website-to-publish/ --recursive
FilesContainer created at: "safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc"
+  ./website-to-publish/index.html              safe://hbyyyydhp7y3mb6zcj4herpqm53ywnbycstamb54yhniud1cij7frjfe8c
+  ./website-to-publish/image.jpg               safe://hbyyyynkt8ak5mxmbqkdt81hqceota8fu83e49gi3weszddujfc8fxcugp
+  ./website-to-publish/contact/form.html       safe://hbyyyyd1sw4dd57k1xeeijukansatia6mthaz1h6htnb8pjoh9naskoaks
```

As we know that website is now publicly available on the Safe Network for anyone who wants to visit using its XOR-URL "safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc" with either `$ safe cat` command, or a Safe Browser. But let's now create a NRS name for it and obtain its human-friendly NRS-URL:
```shell
$ safe nrs create mywebsite --link "safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0"
New NRS Map for "safe://mywebsite" created at: "safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh"
+  mywebsite  safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0
```

Note that we provided a versioned URL to the `--link` argument in the command above, i.e. a URL which targets a specific version of the content with `?v=<version number>`. Any type of content which can have different versions (like the case of a `FilesContainer` in our example) can be mapped/linked from an NRS name/subname only if a specific version is provided in the link URL. If you are using a bash based system and want to provide a version (or any other command containing a question mark), the URL must be wrapped in double quotes or bash will interpret the link as a file path and throw an error, e.g. use `"safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0"`, not `safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobncv=0`.

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

In this example the `cat` simply prints out the content of the top level folder (`FilesContainer`) as we've learned from previous sections of this guide, but any other tool or application would be treating this in different ways, e.g. the Safe Browser would be automatically fetching the `index.html` file from it and rendering the website to the user.

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

Removing sub names from an NRS Map Container is very simple and straightforward, since the only information required to do so is just the NRS-URL. The `nrs remove` command will remove only the sub name specified in the provided NRS-URL without touching any of the other existing sub names, e.g. if the `safe://sub-b.sub-a.mypubname` NRS-URL is provided then only `sub-b` sub name will be removed from `mypubname` NRS Map Container (by creating a new version of it, remember this is all part of the perpetual web).

Let's remove the `profile` sub name from the `mywebsite` NRS name we added before:
```shell
$ safe nrs remove profile.mywebsite
NRS Map updated (version 3): "safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh"
-  profile.mywebsite  safe://hnyynyw9ru4afkbfee5m4ca4jbho4f5bj6ynep5k1pioyge6dihfqyjfrnbnc?v=0
```

### Safe-URLs

In previous sections of this guide we explained how we can create two types of safe:// URLs, XOR-URLs and NRS-URLs. It has been explained that safe:// URLs can contain a path as well, if they target a `FilesContainer`, and they can also be post-fixed with `v=<version>` query param in order to target a specific version of the content rather than the latest/current version when this query param is omitted.

All these types of safe:// URLs can be used in any of the supported CLI commands interchangeably as the argument of any command which expects safe:// URL.

E.g. we can retrieve the content of a website with the `cat` command using either its XOR-URL or its NRS-URL, and either fetching the latest version of it or supplying the query param to get a specific version of it. Thus, if we wanted to fetch `version #1` of the site we published at `safe://mywebsite` (which NRS Map Container XOR-URL is `safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh`), the following two commands would be equivalent:
- `$ safe cat "safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh?v=1"`
- `$ safe cat "safe://mywebsite?v=1"`

In both cases the NRS Map Container will be found (from above URLs) by decoding the XOR-URL or by resolving NRS public name. Once that's done, and since the content is an NRS Map, following the rules defined by NRS and the map found in it the target link will be resolved from it. In some circumstances, it may be useful to get information about the resolution of a URL, which can be obtained using the `dog` command.

#### Symlinks

The sn_cli supports upload and retrieval of symlinks using the above commands. It can also resolve relative symlinks in a FileContainer provided that the target exists in the FileContainer.

[More Details](README-symlinks.md)

### Dog

The Safe Network relates information and content using links, as an example, just considering some of the type of content we've seen in this guide, `FilesContainer`s, `Wallet`s and `NRS Map Container`s, they are all containers with named links (Safe-URLs) to other content on the network, and depending on the abstraction they provide, each of these links are resolved following a specific set of rules for each type of container, e.g. NRS subnames are resolved with a predefined set of rules, while a file's location is resolved from a FilesContainer with another set of predefined rules.

Using the `cat` command is a very straightforward way of retrieving any type of data and see its content, but sometimes we may want to understand how the location of the content being retrieved is resolved using these set of predefined rules, and how links are resolved to eventually find the location of the content we are retrieving. This is when we need the `dog` command to sniff around and show the trace when resolving all these links from a URL.

The most basic case for the `dog` command is to get information about the native data type holding a content found with a XOR-URL:
```shell
$ safe dog safe://hnyynywttiyr6tf3qk811b3rto9azx8579h95ewbs3ikwpctxdhtqesmwnbnc
Native data type: PublicSequence
Version: 0
Type tag: 1100
XOR name: 0x231a809e8972e51e520e49187f1779f7dff3fb45036cd5546b22f1f22e459741
XOR-URL: safe://hnyynywttiyr6tf3qk811b3rto9azx8579h95ewbs3ikwpctxdhtqesmwnbnc
```

In this case we see the location where this data is stored on the Network (this is called the XOR name), a type tag number associated with the content (1100 was set for this particular type of container), and the native Safe Network data type where this data is being held on (`PublicSequence`), and since this type of data is versionable we also see which is the version of the content the URL resolves to.

Of course the `safe dog` command can be used also with other type of content like files (`Blob`), e.g. if we use it with a `FilesContainer`'s XOR-URL and the path of one of the files it contains:
```shell
$ safe dog safe://hnyynywttiyr6tf3qk811b3rto9azx8579h95ewbs3ikwpctxdhtqesmwnbnc/subfolder/index.html
Native data type: PublicBlob
XOR name: 0xda4ce4aa59889874921817e79c2b98dc3dbede7fd9a9808a60aa2d35efaa05f4
XOR-URL: safe://hbhybyds1ch1ifunraq1jbof98uoi3tzb7z5x89spjonfgbktpgzz4wbxw
Media type: text/html
```

But how about using the `dog` command with an NRS URL, as we now know it's resolved using the NRS rules and following the links found in the NRS Map Container:
```shell
$ safe dog safe://mywebsite/contact/form.html
Native data type: PublicBlob
XOR name: 0xda4ce4aa59889874921817e79c2b98dc3dbede7fd9a9808a60aa2d35efaa05f4
XOR-URL: safe://hbhybyds1ch1ifunraq1jbof98uoi3tzb7z5x89spjonfgbktpgzz4wbxw
Media type: text/html

Resolved using NRS Map:
PublicName: "mywebsite"
Container XOR-URL: safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh
Native data type: PublicSequence
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

### Seq (Sequence)

As mentioned before, `FilesContainers` and `NRS Map Containers` are abstractions created on top of the network's native `Public Sequence` data type. A `Public Sequence` is a very simple data type that allows the user to only append elements to it once it has been created on the network.

Mutations made to `FilesContainers` and `NRS Map Containers` are made by storing the new version as a snapshot of the content and appended as a new item into its underlying `Public Sequence`. This is how these types of content are able to keep the complete history on the network.

#### Seq Store

The CLI also allows us to store our own `Public Sequence` instances, with any other type of content we would like to store, instead of the data representing a `FilesContainer` or `NRS Map Container`. We can store a `Public Sequence` on the network with "my initial note" string as its first item:
```shell
$ safe seq store "my initial note"
Public Sequence stored at: "safe://hnyyyyp3yb3dczuaaiwx1mb5491xir4kz1hex3d1pc34oxwicy7scm3x4ybfo"
```

We can then retrieve the content of this Sequence data, using either the `cat`/`dog` command as we do it with any other type of content:
```shell
$ safe cat safe://hnyyyyp3yb3dczuaaiwx1mb5491xir4kz1hex3d1pc34oxwicy7scm3x4ybfo
Public Sequence (version 0) at "safe://hnyyyyp3yb3dczuaaiwx1mb5491xir4kz1hex3d1pc34oxwicy7scm3x4ybfo":
my initial note
```

It's also possible to pipe the output of another command into the `seq store` command to store a new `Sequence` with the content obtained from STDIN by providing `-` as the data argument:
```shell
$ echo "hello from stdin" | safe seq store -
Public Sequence stored at: "safe://hnyyyypfneksex7qxr5zuqqizdkbqbmn1tir1pmfwz1wsghb69rna76syabfo"

$ safe cat safe://hnyyyypfneksex7qxr5zuqqizdkbqbmn1tir1pmfwz1wsghb69rna76syabfo
Public Sequence (version 0) at "safe://hnyyyypfneksex7qxr5zuqqizdkbqbmn1tir1pmfwz1wsghb69rna76syabfo
hello from stdin
```

##### Private Sequence

The above CLI command will store the `Sequence` as Public by default, i.e. it's perpetually stored on the network and publicly available for other users to read it. We can otherwise store the new `Sequence` as private content, in which case, only the creator of it will have access to read and mutate. This can be simply achieved by providing the `--private` flag:
```shell
$ safe seq store "my initial private note" --private
Private Sequence stored at: "safe://hnyyyytcgbrcfq5aw8myg6ihw6d8ss6bsgr9szm8y6qwjxsbiqufr8n3tebfo"
```

We can retrieve it, just as with `Public Sequence` using its XOR-URL, as long as the CLI has been authorised with the Authenticator by the same user who stored the Sequence, any other user will get an error when trying to retrieve it with the following command:
```shell
$ safe cat safe://hnyyyytcgbrcfq5aw8myg6ihw6d8ss6bsgr9szm8y6qwjxsbiqufr8n3tebfo
Private Sequence (version 0) at "safe://hnyyyytcgbrcfq5aw8myg6ihw6d8ss6bsgr9szm8y6qwjxsbiqufr8n3tebfo
my initial private note
```

#### Seq Append

Once we have a `Sequence` stored on the network, either `Public` or `Private`, new items can be appended to it:
```shell
$ safe seq append "first update to my note" safe://hnyyyyp3yb3dczuaaiwx1mb5491xir4kz1hex3d1pc34oxwicy7scm3x4ybfo
Data appended to the Sequence: "safe://hnyyyyp3yb3dczuaaiwx1mb5491xir4kz1hex3d1pc34oxwicy7scm3x4ybfo"
```

We can confirm the new item has been appended to the `Sequence`:
```shell
$ safe cat safe://hnyyyyp3yb3dczuaaiwx1mb5491xir4kz1hex3d1pc34oxwicy7scm3x4ybfo
Public Sequence (version 1) at "safe://hnyyyyp3yb3dczuaaiwx1mb5491xir4kz1hex3d1pc34oxwicy7scm3x4ybfo":
first update to my note
```

And we can also confirm the previous item has been kept in the `Sequence` if we provide the same XOR-URL but specifying a version (with `?v=<version>`):
```shell
$ safe cat safe://hnyyyyp3yb3dczuaaiwx1mb5491xir4kz1hex3d1pc34oxwicy7scm3x4ybfo?v=0
Public Sequence (version 0) at "safe://hnyyyyp3yb3dczuaaiwx1mb5491xir4kz1hex3d1pc34oxwicy7scm3x4ybfo?v=0":
my initial note
```

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
SC=~/.bash_sn_cli CL="source $SC" RC=~/.bashrc; safe setup completions bash > $SC && grep -qxF "$CL" $RC || echo $CL >> $RC
```

#### Windows PowerShell Completions

To enable completions in the current PowerShell session, use the following commands:
```shell
safe setup completions bash > sn_cli.ps1
sn_cli.ps1
```

To enable PowerShell completions permanently, generate the sn_cli.ps1 file as per above and then see this [stackoverflow answer](<https://stackoverflow.com/questions/20575257/how-do-i-run-a-powershell-script-when-the-computer-starts#32189430>).

### Update

The CLI can update itself to the latest available version. If you run `safe update`, the application will check if a newer release is available on [GitHub](https://github.com/maidsafe/sn_cli/releases). After prompting to confirm if you want to take the latest version, it will be downloaded and the binary will be updated.

## Further Help

You can discuss development-related topics on the [Safe Dev Forum](https://forum.safedev.org/).

If you are just starting to develop an application for the Safe Network, it's very advisable to visit the [Safe Network Dev Hub](https://hub.safedev.org) where you will find a lot of relevant information.

If you find any issues, or have ideas for improvements and/or new features for this application and the project, please raise them by [creating a new issue in this repository](https://github.com/maidsafe/sn_cli/issues).

## License
This Safe Network library is dual-licensed under the Modified BSD ([LICENSE-BSD](LICENSE-BSD) https://opensource.org/licenses/BSD-3-Clause) or the MIT license ([LICENSE-MIT](LICENSE-MIT) https://opensource.org/licenses/MIT) at your option.

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [Guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
