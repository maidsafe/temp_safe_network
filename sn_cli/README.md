# Safe Network CLI

| [MaidSafe website](https://maidsafe.net) | [Safe Dev Forum](https://forum.safedev.org) | [Safe Network Forum](https://safenetforum.org) |
|:----------------------------------------:|:-------------------------------------------:|:----------------------------------------------:|

## Table of contents

- [Description](#description)
- [Quick Start](#quick-start)
- [Installation and Setup](#installation-and-setup)
  - [Prerequisites](#prerequisites)
  - [Install Script](#install-script)
    - [Linux and macOS](#linux-and-macos)
    - [Windows](#linux-and-macos)
  - [Binaries](#binaries)
  - [Build from Source](#build-from-source)
- [Using the CLI](#using-the-cli)
  - [Getting Help](#getting-help)
- [Networks](#networks)
  - [Node Management](#node-management)
  - [Run a Local Network](#run-a-local-network)
  - [Connect to a Remote Network](#connect-to-a-remote-network)
    - [Connection Info via HTTP](#connection-info-via-http)
    - [Direct Connection Info](#direct-connection-info)
    - [Provide a Node](#provide-a-node)
- [XorUrl](#xorurl)
- [Keys](#xorurl)
- [Files](#files)
  - [Put](#put)
    - [Base Path](#base-path)
  - [Sync](#put)
  - [Add](#files-add)
  - [Ls](#files-ls)
  - [Get](#files-get)
    - [Performance](#performance)
  - [Tree](#files-tree)
  - [Rm](#files-rm)
- [Cat](#cat)
  - [Retrieve Files and Containers](#retrieve-files-and-containers)
  - [Retrieve Binary Files](#retrieve-binary-files)
  - [Versioning](#versioning)
- [NRS](#nrs)
  - [Register a Top Name](#register-a-top-name)
  - [Add a Sub Name](#add-a-sub-name)
  - [List the NRS Map](#list-the-nrs-map)
- [Dog](#dog)
- [Further Help](#further-help)
- [License](#license)
- [Contributing](#contributing)

## Description

This crate implements a CLI (Command Line Interface) for the Safe Network.

The Safe CLI provides all the tools necessary to interact with the Safe Network, including storing and browsing data of any kind, following links contained in the data, using their addresses on the network, and much more. Using the CLI, users have access to any type of operation that can be made on the Safe Network and the data stored on it. Due to it being a CLI, it can also be used in automated scripts and Unix-style piping and redirection.

This is a user guide for the CLI. It can be used as a reference, but if you're completely new to the network, reading in order is recommended, since we present concepts in the order we think makes the most sense.

## Quick Start

### Linux/macOS

If you have root access, you can run the following in your terminal:
```
$ curl -so- https://raw.githubusercontent.com/maidsafe/safe_network/main/resources/scripts/install.sh | sudo bash
```

If you don't or prefer an install under your home directory:
```
$ curl -so- https://raw.githubusercontent.com/maidsafe/safe_network/main/resources/scripts/install.sh | bash
```

The non-root option may require a new shell session; using root, `safe` should be available immediately.

### Windows

Start a Powershell session and run the following:
```
Set-ExecutionPolicy Bypass -Scope Process -Force; iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/maidsafe/safe_network/main/resources/scripts/install.ps1'))
```

If you don't have the Visual C++ Redistributable installed, the script will install it for you, but that requires admin access; otherwise, admin is not required. On the first install run, the location of `safe` will be added to your user `PATH` variable. Powershell requires a new session to pick this up. If VC++ was installed, this requires a machine restart. If later you use the install script to get a newer version of `safe`, neither will be necessary again.

Now, if you can successfully run `safe --version`, you can skip to [Using the CLI](#using-the-cli).

If you want more detail or alternative setup options, read the next section.

## Installation and Setup

### Prerequisites

The Safe CLI is written in Rust and distributed as a single binary, so it has almost no prerequisites. For Linux, we build using [musl](https://musl.libc.org/), meaning we don't need different builds for particular distributions. There is no other setup required. You simply get a copy of the binary and run it.

If you are a Windows user, you need to install the [Visual C++ Redistributable for Visual Studio](https://www.microsoft.com/en-in/download/details.aspx?id=48145), otherwise attempting to run the CLI will result in errors such as:
```
error while loading shared libraries: api-ms-win-crt-locale-l1-1-0.dll: cannot open shared object file: No such file or directory`
```

If you use Visual Studio, you may already have these libraries installed.

### Install Script

#### Linux and macOS

The [install script](https://raw.githubusercontent.com/maidsafe/safe_network/master/resources/scripts/install.sh) can get you running quickly. If you have root access to your machine, in your terminal, you can run:
```
$ curl -so- https://raw.githubusercontent.com/maidsafe/safe_network/master/resources/scripts/install.sh | sudo bash
```

This downloads and unpacks the latest `safe` binary to `/usr/local/bin`; this location is almost always on the `PATH` variable, so `safe` will be immediately available.

If you don't have root access or prefer an install under your home directory:
```
$ curl -so- https://raw.githubusercontent.com/maidsafe/safe_network/master/resources/scripts/install.sh | bash
```

In this case, `safe` is unpacked to `~/.safe/cli/`, and your `PATH` variable is extended with that location. To do so, the install script modifies your shell configuration, such as your `~/.bashrc` file. If you'd prefer the install not modify your configuration, use the sudo option above.

The install script also has a `--version` argument to enable retrieval of a specific version:
```
$ curl -so- https://raw.githubusercontent.com/maidsafe/sn_cli/master/resources/install.sh | bash -s - --version=0.32.0
```

#### Windows

The quickstart section has all the details for the Windows install script. No further elaboration is required.

### Binaries

If you would prefer, you can obtain binaries for all available platforms from the [releases page](https://github.com/maidsafe/sn_cli/releases/latest). You are free to download and extract these to any location you wish. It's advisable to put them in a location that's on your system's `PATH` variable, or extend the variable to include the new location.

### Build from Source

To build from source, you need to have `rustc v1.56.1` (or higher) installed. Please refer to [this guide](https://www.rust-lang.org/tools/install) for setting up a Rust environment. We recommend installation with `rustup`, which will install the `cargo` build system.

Once Rust and its toolchain are installed, run the following commands to clone this repository and build the CLI:
```
$ git clone https://github.com/maidsafe/safe_network.git
$ cargo build --bin safe
```

Once built, you can find the `safe` executable at `target/debug/`, or `target/release/` if you used the `--release` flag.

## Using the CLI

Right now the CLI is under active development. Here we're listing commands ready to be tested.

In this guide, from this point we'll assume you have the `safe` binary installed and available on your system PATH.

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

### Getting Help

All commands have a `--help` argument, which lists and describes arguments, options and subcommands.

Sometimes it can also be useful to increase the level of output from the CLI. It may look like it isn't doing anything when it's actually retrying or waiting on something. More verbose output can help with identifying potential issues. You can control the output using the `RUST_LOG` environment variable. Here's an example:
```
export RUST_LOG=safe=debug,sn_api=debug,safe_network=debug
```

Logging is available from 3 sources: `safe`, `sn_api` and `safe_network`. Possible values for levels are `info`, `debug` and `trace`, each of those increasing in detail. You can try varying these to get the level you want.

If you experience the CLI taking a long time to respond, you can try decreasing its timeout duration. This is controlled using the `SN_CLI_QUERY_TIMEOUT` environment variable. The units of this variable is in seconds. So for example, you may try `export SN_CLI_QUERY_TIMEOUT=30`.

## Networks

We can connect to different Safe networks that may be available. As the project advances, several networks may coexist with the main Safe Network; there could be networks available for testing upcoming features, or networks local to the user in their own computer or WAN/LAN.

The CLI enables users to easily maintain different networks and switch between them.

The first thing we'll do is create a local network and connect to it, but in order to do so, we need to setup a node.

### Node Management

Before we can create a local network, we need to obtain the `sn_node` binary.

We can get the latest version using the `node install` command:
```
$ safe node install
Downloading sn_node version: 0.52.0
Downloading https://sn-node.s3.eu-west-2.amazonaws.com/sn_node-0.52.0-x86_64-unknown-linux-musl.tar.gz...
[00:00:01] [========================================] 10.49MB/10.49MB (0s) Done
Creating '/home/chris/.safe/node' folder
Installing sn_node binary at /home/chris/.safe/node ...
Setting execution permissions to installed binary '/home/chris/.safe/node/sn_node'...
Done!
```

If for some reason you want a specific version, or you want it placed at another location, the command has `--version` and `--node-path` arguments.

The node can be updated to the latest available version using the `node update` command.

After prompting to confirm if you want to take the latest version, it will be downloaded and the binary will be updated. By default it will assume `sn_node` is at `~/.safe/node/`, but you can override that path by using the `--node-path` argument.

### Run a Local Network

Let's first look at how to run a local Safe network with a single section.

A local network is bootstrapped by running several nodes which automatically interconnect to form a network. If the node was installed as described in the previous section, we can then create the network with a simple command:
```
$ safe node run-baby-fleming
Storing nodes' generated data at /home/chris/.safe/node/baby-fleming-nodes
Starting a node to join a Safe network...
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-genesis"
Node PID: 1025110, prefix: Prefix(), name: 799ac7(01111001).., age: 255, connection info:
"127.0.0.1:41904"
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-2"
Node PID: 1025149, prefix: Prefix(), name: 8e9b93(10001110).., age: 98, connection info:
"127.0.0.1:49538"
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-3"
Node PID: 1025186, prefix: Prefix(), name: 0c9ce6(00001100).., age: 96, connection info:
"127.0.0.1:40617"
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-4"
Node PID: 1025224, prefix: Prefix(), name: f7a2ff(11110111).., age: 94, connection info:
"127.0.0.1:57710"
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-5"
Node PID: 1025261, prefix: Prefix(), name: 75a2c1(01110101).., age: 92, connection info:
"127.0.0.1:49860"
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-6"
Node PID: 1025299, prefix: Prefix(), name: b6f79c(10110110).., age: 90, connection info:
"127.0.0.1:32819"
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-7"
Node PID: 1025336, prefix: Prefix(), name: 3444f0(00110100).., age: 88, connection info:
"127.0.0.1:51381"
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-8"
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-9"
Node PID: 1025379, prefix: Prefix(), name: 653f38(01100101).., age: 76, connection info:
"127.0.0.1:60200"
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-10"
Node PID: 1025399, prefix: Prefix(), name: a76137(10100111).., age: 74, connection info:
"127.0.0.1:52562"
Starting logging to file: "/home/chris/.safe/node/baby-fleming-nodes/sn-node-11"
Node PID: 1025437, prefix: Prefix(), name: 260ca3(00100110).., age: 72, connection info:
"127.0.0.1:48606"
Node PID: 1025475, prefix: Prefix(), name: c5feb4(11000101).., age: 70, connection info:
"127.0.0.1:42312"
Creating '/home/chris/.safe/cli/networks' folder for networks connection info cache
Caching current network connection information into '/home/chris/.safe/cli/networks/baby-fleming_node_connection_info.config'
```

If you now run `safe networks`, you will see the newly created network in your networks list:
```
+----------+--------------+-------------------------------------------------------------------------+
| Networks |              |                                                                         |
+----------+--------------+-------------------------------------------------------------------------+
| Current  | Network name | Connection info                                                         |
+----------+--------------+-------------------------------------------------------------------------+
| *        | baby-fleming | /home/chris/.safe/cli/networks/baby-fleming_node_connection_info.config |
+----------+--------------+-------------------------------------------------------------------------+
```

The local network is set as the current network, so any `safe` commands will now run against that. This network consists of 11 `sn_node` processes running on the local machine.

Before attempting to connect to a remote network, you may want to play around with this local network by issuing some commands against it. You could try storing some files with the `files put` command, then you could retrieve them using the `cat` command. You could also create some NRS entries. To get an idea of what you could do, you can read those sections of this guide, then come back here.

When you're satisfied with your local experimentation, you can stop the local network using the following command:
```
$ safe node killall
Success, all processes instances of sn_node were stopped!
```

However, you can feel free to keep this network running but still connect to another network. We'll do this in our examples here.

### Connect to a Remote Network

We'll describe the process for connecting to a remote network using a couple of example networks created by our [testnet tool](https://github.com/maidsafe/sn_testnet_tool). We'll call these networks 'alpha' and 'beta'.

To connect to a network, we need to obtain its node connection information. This can be provided using different mechanisms. Contact the owner(s)/administrator(s) of the network you're trying to connect to and ask them to provide you with the connection info.

#### Connection Info via HTTP

People who run networks can keep a copy of the connection info at some http location. In our case, we have this connection information hosted in an S3 bucket, and this is available via http. We can add a new network and use this http location for the connection info:
```
$ safe networks add alpha https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/alpha-node_connection_info.config
Network 'alpha' was added to the list. Connection information is located at 'https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/alpha-node_connection_info.config'
```

We've given this network the name 'alpha', but it can be named however you wish to refer to it. Run `safe networks` to see it in the networks list:
```
$ safe networks
+----------+--------------+----------------------------------------------------------------------------------------+
| Networks |              |                                                                                        |
+----------+--------------+----------------------------------------------------------------------------------------+
| Current  | Network name | Connection info                                                                        |
+----------+--------------+----------------------------------------------------------------------------------------+
|          | alpha        | https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/alpha-node_connection_info.config |
+----------+--------------+----------------------------------------------------------------------------------------+
| *        | baby-fleming | /home/chris/.safe/cli/networks/baby-fleming_node_connection_info.config                |
+----------+--------------+----------------------------------------------------------------------------------------+

```

Notice our new network isn't set as the current network, meaning any `safe` commands we run will still run against our local network. Make the remote network current by using the `switch` command:
```
$ safe networks switch alpha
Switching to 'alpha' network...
Fetching 'alpha' network connection information from 'https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/alpha-node_connection_info.config' ...
Successfully switched to 'alpha' network in your system!
```

You can then run `safe networks` to see the current network has changed:
```
+----------+--------------+----------------------------------------------------------------------------------------+
| Networks |              |                                                                                        |
+----------+--------------+----------------------------------------------------------------------------------------+
| Current  | Network name | Connection info                                                                        |
+----------+--------------+----------------------------------------------------------------------------------------+
| *        | alpha        | https://safe-testnet-tool.s3.eu-west-2.amazonaws.com/alpha-node_connection_info.config |
+----------+--------------+----------------------------------------------------------------------------------------+
|          | baby-fleming | /home/chris/.safe/cli/networks/baby-fleming_node_connection_info.config                |
+----------+--------------+----------------------------------------------------------------------------------------+
```

At this point, you can now start using this network. Try uploading some files and retrieving them.

#### Direct Connection Info

If for some reason connection info isn't available via http, the network owner can directly provide the network's genesis key and the list of IP and port pairs for each node. These can then be added as a network using the `networks set` command:
```
$ safe networks set beta \
    a857bf4e8cce3ab97a6e6c27c2308eebe640ccf57e0447182b732b41f6b04a9796edce5bf151fbfa522b01bcfbbfefa0 \
    178.62.57.53:12000 167.99.88.95:12000 178.128.41.85:12000 206.189.23.17:12000 \
    178.128.42.138:12000 178.128.173.109:12000 167.99.199.158:12000 142.93.41.250:12000 \
    206.189.117.149:12000 209.97.191.137:12000 104.248.171.51:12000 142.93.41.143:12000 \
    167.99.199.11:12000 104.248.174.35:12000 178.128.162.247:12000 209.97.189.244:12000
Network 'beta' was added to the list. Contacts: '(PublicKey(0857..aa81), {104.248.171.51:12000, 104.248.174.35:12000, 142.93.41.143:12000, 142.93.41.250:12000, 167.99.88.95:12000, 167.99.199.11:12000, 167.99.199.158:12000, 178.62.57.53:12000, 178.128.41.85:12000, 178.128.42.138:12000, 178.128.162.247:12000, 178.128.173.109:12000, 206.189.23.17:12000, 206.189.117.149:12000, 209.97.189.244:12000, 209.97.191.137:12000})'
```

As can be seen, this method isn't as convenient as http, but nonetheless, it's available if need be.

If you run `safe networks`, you'll now see 'beta' in the networks list, and you can run `safe networks switch beta` to start using this network.

Perhaps give that a try by uploading some files, then switch between each network.

### Provide a Node

With the remote networks added, we have the opportunity to launch our own node and participate in either of those networks. This will provide more storage space to the network. Let's join the 'alpha' network. We can do so using the `node join` command.

Successfully joining a remote network depends on your networking and routing configuration. The node implements automated port forwarding according to the [IGD Protocol](https://en.wikipedia.org/wiki/Internet_Gateway_Device_Protocol). If your router is IGD compatible, you should be able to join the network with no further configuration.

Try the `safe node join` command and see what happens. If this command runs without error and the node starts, you've sent a join request. If there's an error, it's likely to be related to UPnP port forwarding. If so, run `safe node join --skip-auto-port-forwarding`. Again, this may successfully start the node and send a join request, but the request could be rejected.

We need to inspect the node log to see the status of the join request. The CLI will tell you the location of the log, but it's usually under the `~/.safe/node/local-node` directory. If the join request is rejected, you would see something similar to the following:
```
➤ Node cannot join the network since it is not externally reachable: X.X.X.X:38854
Unfortunately we are unable to establish a connection to your machine (X.X.X.X:38854) either through a public IP address, or via IGD on your router. Please ensure that IGD is enabled on your router - if it is and you are still unable to add your node to the testnet, then skip adding a node for this testnet iteration. You can still use the testnet as a client, uploading and downloading content, etc. https://safenetforum.org/
```

The Xs will be your internet public IP address. If this is the case, you most likely need to configure port forwarding on your router. Obviously, this is something that varies depending on the router, so a step-by-step guide is beyond the scope of this document. However, we can say broadly, you need to access the admin console for your router and find a section on access control and port forwarding. What you want to do is add a UDP rule to forward any external requests on a port of your choosing, to an internal network address and port. The internal address will be the address of the machine where the node is running, and most likely in the form 192.168.X.X. The node uses port 12000 by default, so that's a good one to go with. The rule would look something like this:
```
+----------+---------------+---------------+---------------+---------------+
| Protocol | External Host | Internal Host | External Port | Internal Port |
+----------+---------------+---------------+---------------+---------------+
| UDP      | *             | 192.168.X.X   | 12000         | 12000         |
+----------+---------------+---------------+---------------+---------------+
```

With this configuration in place, you can try joining the network using the following command:
```
$ safe node join --public-addr <internet public IP>:12000 --local-addr <node device IP>:12000 --skip-auto-port-forwarding

```

Regardless of which `join` command you used, if the join request is successful, the log should contain something similar to the following:
```
 INFO 2021-12-24T00:52:18.319227Z [sn/src/routing/routing_api/mod.rs:L196]:
	 ➤ 59dc1f.. Joined the network!
 INFO 2021-12-24T00:52:18.319254Z [sn/src/routing/routing_api/mod.rs:L197]:
	 ➤ Our AGE: 5
 INFO 2021-12-24T00:52:18.319284Z [sn/src/routing/routing_api/dispatcher.rs:L87]:
	 ➤ Starting to probe network
 INFO 2021-12-24T00:52:18.319291Z [sn/src/routing/routing_api/dispatcher.rs:L115]:
	 ➤ Writing our PrefixMap to disk
 INFO 2021-12-24T00:52:18.319312Z [sn/src/routing/core/mod.rs:L212]:
	 ➤ Writing our latest PrefixMap to disk
 INFO 2021-12-24T00:52:18.319659Z [sn/src/node/node_api/mod.rs:L87]:
	 ➤ Node PID: 1065146, prefix: Prefix(0), name: 59dc1f(01011001).., age: 5, connection info: "79.71.42.38:12000"
```

It's also possible to join a network without adding a network to the networks list. You can use the `--contact-list` and `--genesis-key` arguments. Run `safe node join --help` for more details.

## XorUrl

Almost everything on the network involves the use of what we call an XOR-URL. You'll see these in
the form `safe://hoxm5aps8my8he8cpgdqh8k5wuox5p7kzed6bsbajayc3gc8pgp36s`. This isn't just a random
string: it's generated based on the content the URL points to. These URLs are then used to retrieve
said content.

Each piece of content is located at an address we call an XorName. This is a 256-bit number, so we
have an [absolutely enormous](https://www.youtube.com/watch?v=S9JGmA5_unY) 2^256 possible addresses.
The XOR-URL has the XorName encoded in it, plus some other information about the content, such as
its type, e.g., a file. As you'll see in the [Files](#files) section, each file you upload will have
its own XOR-URL, which can be determined before the file is uploaded.

As an example, I can use the `xorurl` command on this document:
```
$ safe xorurl sn_cli/README.md
1 file/s processed:
sn_cli/README.md  safe://hy8oyeyybi347nyusj15sfs73t4gkqzbi7tftpo33crgfgyijxh69jhedjepy
```

If we decided to upload that file, it would be located at this XOR-URL. This command can be useful
if you lost the URL of something you uploaded.

An XOR-URL can also be decoded to yield information about it:
```
$ safe xorurl decode safe://hy8oyeyybwsanc3ehnecyab9n3ufoip6x47e6553rb539aeqnej1xwadcbfdo
UrlType: XOR-URL
Xorname: a5b026651c12180c07e2cccb0ab7cfd751edef240ef3fc21c24264fa606c0947
Public Name: <unknown>
Sub names:
Type tag: 0
Native data type: File
Path:
QueryString:
QueryPairs: []
Fragment:
Content version: latest
```

You may be thinking these URLs seem a bit unwieldy. To deal with this, the network has a concept
similar to internet DNS, which enables us to alias human readable names to these addresses. See the
[NRS](#nrs) section for more information.

## Keys

Every message sent to the network is signed with a public/private keypair, which is used to assign
the owner of any data that was uploaded. If you're familiar with SSH keys, the keypair here is
analogous.

It's possible to use `safe` without a keypair, but in this case, it will generate a new one for each
command. So if you uploaded some files, the owner of them would be assigned the one-time keypair,
and effectively they would be read-only. If you want to subsequently write to them, you need to use
the same keypair. We'll need this for the `files sync` command in the next section, so let's use the
`keys create` command to get a persistent keypair:
```
$ safe keys create --for-cli
New SafeKey created: "safe://hyryyyyyym96apoapogoieau7yxkifo6xgppzxh66n1fh1hwnibz6xaku4tfy"
Key pair generated:
Public Key = 5ffd86c30d81a154627d03d552c3cf335b77f3de148bc97282a86fe7e153d44a
Secret Key = 70c1936dbb9143e8c4eeb335a2731b2fe93679c1e6c9b28a1f79ec710b21c9cc
Setting new SafeKey to be used by CLI...
New credentials were successfully stored in /home/chris/.safe/cli/credentials
Safe CLI now has write access to the network
```

We also need the keypair for writing NRS entries.

At the moment, this is all we're using the keypair for; however, in the future, it may be used for
many other things, including the encryption of data.

To avoid confusion when working with files, it's worth generating a persistent keypair to use with
all your commands.

## Files

We can use the CLI to upload files and folders and keep them in sync with local modifications.

Files are uploaded and stored as one or more chunks. When we upload a directory, its hierarchy is flattened and represented in a map, with each file's path mapped to a file XOR-URL. This map is maintained in a special container called `FilesContainer`. The data representation in the `FilesContainer` is planned to be implemented with [RDF](https://en.wikipedia.org/wiki/Resource_Description_Framework), and the corresponding `FilesContainer` RFC will be submitted; however, at this stage, the implementation is a simple serialised structure.

For brevity, in the rest of this section we'll refer to the `FilesContainer` just using the term "container".

As we describe the `files` commands, all the examples will work with the following directory:
```
to-upload/
├── file1.txt
├── myfolder
│   └── file2.txt
└── myotherfolder
    └── subfolder
        └── file3.txt
```

### Warning

At some point, the underlying files APIs used in the CLI (and perhaps much else of the CLI) will be deprecated to use [`sn_fs`](https://github.com/maidsafe/sn_fs). This is a POSIX-based filesystem which is more comprehensive and performant. The impact this may have, if any, on CLI commands, is not yet known. But if you're interested in a filesystem on Safe, `sn_fs` is most definitely where you should look.

### Put

The simplest thing we can do is upload a file using the `files put` command:
```
$ safe files put ./to-upload/file1.txt
FilesContainer created at: "safe://hyryyryynamznbfsgn7ccfquqmx6y8yzhq6tn7uzz775hrkyj4g8ipcy3ke6yeuy?v=h9jxxwwpy1cwf3pnb86ahkfk5ju8eb3miegnuehc99f5r5x83d9go"
+  ./to-upload/file1.txt  safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
```

This will create a container with a single file. The `safe://` URLs assigned here refer to the container and the file, respectively. This means the file is addressable using either `safe://hyryyryynamznbfsgn7ccfquqmx6y8yzhq6tn7uzz775hrkyj4g8ipcy3ke6yeuy/file1.txt` or its direct URL.

We can also recursively upload everything in a local directory, obtaining the XOR-URLs of the newly created container and each uploaded file:
```
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hyryyryynteexnr17a75mdptifno13kugqxdu8k39tecdm1dukm8kfidq9wpyeuy?v=ha83p9ai5be5cp1bajhfaydjyxe7hrhjb833qi9dsqugugsj3jxbo"
+  ./to-upload/file1.txt                          safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
+  ./to-upload/myfolder
+  ./to-upload/myfolder/file2.txt                 safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
+  ./to-upload/myotherfolder
+  ./to-upload/myotherfolder/subfolder
+  ./to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
```

**Note**: the `+` sign indicates the files were _added_ to the container, as opposed to _updated_ or _deleted_. This will be elaborated further when discussing the `files sync` command.

#### Base Path

When a container is created, its base path is set to `/`. Uploaded files have an absolute path stemming from the container's base.

To illustrate using our example directory, if we run `safe files put --recursive to-upload/`, this maps to:
```
/file1.txt
/myfolder/file2.txt
/myotherfolder/subfolder/file3.txt
```

If an alternative base is desired, we could use `safe files put --recursive to-upload/ /mychosenroot` and this would map to:
```
/mychosenroot/file1.txt
/mychosenroot/myfolder/file2.txt
/mychosenroot/myotherfolder/subfolder/file3.txt
```

### Ls

We can list the contents of a container using the `files ls` command.

Create a new container with the example directory:
```
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hyryyryyndnbzqc9zmuu6iggm7j5obyx3sj8idcpg7ds9jdiwtjs1gjipd3ioeuy?v=hub5nnrw5eq6sbc4do4d5oyndd7ijyw4q79zt3k3ocnnpfpfzkdjy"
+  ./to-upload/file1.txt                          safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
+  ./to-upload/file2.txt                          safe://hy8oycyybpgwwyx378g4b1da348kawo9i6xerxkot9w7xzwjht71awf55tj8o
+  ./to-upload/myfolder
+  ./to-upload/myfolder/file2.txt                 safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
+  ./to-upload/myotherfolder
+  ./to-upload/myotherfolder/subfolder
+  ./to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
```

Now list its contents:
```
$ safe files ls safe://hyryyryyndnbzqc9zmuu6iggm7j5obyx3sj8idcpg7ds9jdiwtjs1gjipd3ioeuy
Files of FilesContainer (version hub5nnrw5eq6sbc4do4d5oyndd7ijyw4q79zt3k3ocnnpfpfzkdjy) at "safe://hyryyryyndnbzqc9zmuu6iggm7j5obyx3sj8idcpg7ds9jdiwtjs1gjipd3ioeuy":
Files: 2   Size: 49   Total Files: 4   Total Size: 120
SIZE  CREATED     MODIFIED    NAME
29    1641566755  1641566755  file1.txt
20    1641566755  1641566755  file2.txt
35    1641566755  1641566755  myfolder/
36    1641566755  1641566755  myotherfolder/
```

**Note**: the size of the subdirectory is the sum of the sizes of all its files.

You can also list a subdirectory:
```
$ safe files ls safe://hyryyryyndnbzqc9zmuu6iggm7j5obyx3sj8idcpg7ds9jdiwtjs1gjipd3ioeuy/myfolder
Files of FilesContainer (version hub5nnrw5eq6sbc4do4d5oyndd7ijyw4q79zt3k3ocnnpfpfzkdjy) at "safe://hyryyryyndnbzqc9zmuu6iggm7j5obyx3sj8idcpg7ds9jdiwtjs1gjipd3ioeuy/myfolder":
Files: 1   Size: 35   Total Files: 1   Total Size: 35
SIZE  CREATED     MODIFIED    NAME
35    1641566755  1641566755  file2.txt
```

### Sync

When files or directories have been uploaded, local changes can be kept in sync using the `files sync` command. When the command completes, we'll have a new version of the container with the local modifications.

First, we'll upload our example directory:
```
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hyryyryynuffbauiq8jbnw4whc4kk7bkoz7e6e534ufb188c5ua4kg35yjh7oeuy?v=h63f7w71cws46ar7gxhbqntrp6ritx4wso3g1o6obw1uqgzpm14xo"
+  ./to-upload/file1.txt                          safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
+  ./to-upload/myfolder
+  ./to-upload/myfolder/file2.txt                 safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
+  ./to-upload/myotherfolder
+  ./to-upload/myotherfolder/subfolder
+  ./to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
```

Then we'll make some local changes:

* Update the content of `file1.txt`
* Add a `new.txt` file
* Delete `myfolder/file2.txt`
* Change the contents of `myotherfolder/subfolder/file3.txt`

Now, perform the `files sync` command using the container address:
```
$ safe files sync ./to-upload/ safe://hyryyryynuffbauiq8jbnw4whc4kk7bkoz7e6e534ufb188c5ua4kg35yjh7oeuy
FilesContainer synced up (version hkib4j7zukystawmi61ytw1cmrztdy6gd85n8u8pyr6ccoexoeu8y): "safe://hyryyryynuffbauiq8jbnw4whc4kk7bkoz7e6e534ufb188c5ua4kg35yjh7oeuy?v=hkib4j7zukystawmi61ytw1cmrztdy6gd85n8u8pyr6ccoexoeu8y"
*  ./to-upload/file1.txt  safe://hy8oycyybexj6wd9yr7r5dhf1x1un5ar8tkat1hpzm7zm7yr5m9u3dod4zjfy
+  ./to-upload/new.txt    safe://hy8oycyybkbwadw8m5d845dfwe3bgxm3ssjjtawqgoy66eh9fkhh3xbwxis9y
```

The `*` and `+` denote a _modification_ and an _addition_, respectively, and we have a new version hash, which is now the current version. Using the version hash from the initial `files put` command, it would be possible to work with the first version of the container, which still exists.

What of the deletion and change to `file3.txt`? Why were those not synchronised? By default, the command won't check for deletions and will only work with the top level of the directory.

Run again using the `--recursive` and `--delete` flags:
```
$ safe files sync ./to-upload/ safe://hyryyryynuffbauiq8jbnw4whc4kk7bkoz7e6e534ufb188c5ua4kg35yjh7oeuy --recursive --delete
FilesContainer synced up (version hb4bed1obb7j87q8jg9dengte66s14tjdi6dgpz1doknjiggfe14o): "safe://hyryyryynuffbauiq8jbnw4whc4kk7bkoz7e6e534ufb188c5ua4kg35yjh7oeuy?v=hb4bed1obb7j87q8jg9dengte66s14tjdi6dgpz1doknjiggfe14o"
-  /myfolder/file2.txt                            safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
*  ./to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyyb9iwiadpibqwae93feyw53e8o6swhwcqpq8m6yuydbahskjjurpyo
```

Again, the container has a new version, and this time we see the _deletion_, denoted by `-`, and the _modification_ to `file3.txt`, which was in a subdirectory.

**Note**: `--delete` will only apply when used in conjunction with `--recursive`.

When performing a sync, it's also possible to specify a location in the container. This is useful if you wanted to synchronise some other directory to the same container. To illustrate, we can make a copy of `to-upload` and sync it to `upload2` in the container:
```
$ safe files sync ./to-upload2/ safe://hyryyryynuffbauiq8jbnw4whc4kk7bkoz7e6e534ufb188c5ua4kg35yjh7oeuy/upload2 --recursive
FilesContainer synced up (version h6fpc6brw7a65zb5brwo6gigpryqeyothgpmgnwqy549ks4tfg1sy): "safe://hyryyryynuffbauiq8jbnw4whc4kk7bkoz7e6e534ufb188c5ua4kg35yjh7oeuy?v=h6fpc6brw7a65zb5brwo6gigpryqeyothgpmgnwqy549ks4tfg1sy"
+  ./to-upload2/file1.txt                          safe://hy8oycyybexj6wd9yr7r5dhf1x1un5ar8tkat1hpzm7zm7yr5m9u3dod4zjfy
+  ./to-upload2/file2.txt                          safe://hy8oycyybpaxr1qxmkxup5urtxz5xahxcw36jkf7n4fywx9i4mennnzwyxiso
+  ./to-upload2/myfolder
+  ./to-upload2/myotherfolder
+  ./to-upload2/myotherfolder/subfolder
+  ./to-upload2/myotherfolder/subfolder/file3.txt  safe://hy8oycyyb9iwiadpibqwae93feyw53e8o6swhwcqpq8m6yuydbahskjjurpyo
+  ./to-upload2/new.txt                            safe://hy8oycyybkbwadw8m5d845dfwe3bgxm3ssjjtawqgoy66eh9fkhh3xbwxis9y
```

### Add

We may want to add a file to an existing container rather than perform a full sync. We can use the `files add` command for this.

First, create a new container by uploading the example directory:
```
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hyryyryyn7y13rpg7jgitypn47f6dkprjbxh4j5skk3iafxayqo6pgh3om5poeuy?v=h3tipnhnf5iq89gfkqprs3hdg5hb3zyq8ton3ukjoakbzsqujiayo"
+  ./to-upload/file1.txt                          safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
+  ./to-upload/myfolder
+  ./to-upload/myfolder/file2.txt                 safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
+  ./to-upload/myotherfolder
+  ./to-upload/myotherfolder/subfolder
+  ./to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
```

Then create a new file at `to-upload/file2.txt` and add it to the container:
```
$ safe files add to-upload/file2.txt safe://hyryyryyn7y13rpg7jgitypn47f6dkprjbxh4j5skk3iafxayqo6pgh3om5poeuy
FilesContainer updated (version hwoqjfzhgjrjquuofwb1xj8zr574b4dh5jcatn4u9xfx6rxfw6qro): "safe://hyryyryyn7y13rpg7jgitypn47f6dkprjbxh4j5skk3iafxayqo6pgh3om5poeuy?v=hwoqjfzhgjrjquuofwb1xj8zr574b4dh5jcatn4u9xfx6rxfw6qro"
+  to-upload/file2.txt  safe://hy8oycyybpgwwyx378g4b1da348kawo9i6xerxkot9w7xzwjht71awf55tj8o
```

If we have previously uploaded a file, we can add it to another container by providing its XOR-URL as the `<location>` argument:
```
$ safe files add \
    safe://hy8oycyybkbwadw8m5d845dfwe3bgxm3ssjjtawqgoy66eh9fkhh3xbwxis9y \
    safe://hyryyryyn7y13rpg7jgitypn47f6dkprjbxh4j5skk3iafxayqo6pgh3om5poeuy/new.txt
FilesContainer updated (version hcd7koxakugzak5xfsq86dwntx18whp98zeegcbyi1n11q4za11ry): "safe://hyryyryyn7y13rpg7jgitypn47f6dkprjbxh4j5skk3iafxayqo6pgh3om5poeuy?v=hcd7koxakugzak5xfsq86dwntx18whp98zeegcbyi1n11q4za11ry"
+  /new.txt  safe://hy8oycyybkbwadw8m5d845dfwe3bgxm3ssjjtawqgoy66eh9fkhh3xbwxis9y
```

**Note**: the `<target>` must include a filename, which is `/new.txt` in the above example.

### Get

The `files get` command copies file(s) from the network to the local filesystem.

This command works similarly to Unix `cp` or `scp` or the windows `copy` command.

First, create a new container by uploading the example directory:
```
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hyryyryyn68cfxon3diif17w87nkj5mesc95f4noxnr85yqt6nj4qhbaaktjyeuy?v=htj4d87x47jnpyzkgyj5gm657eak33bpbaegfgtk56kznws654dxo"
+  ./to-upload/file1.txt                          safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
+  ./to-upload/file2.txt                          safe://hy8oycyybpgwwyx378g4b1da348kawo9i6xerxkot9w7xzwjht71awf55tj8o
+  ./to-upload/myfolder
+  ./to-upload/myfolder/file2.txt                 safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
+  ./to-upload/myotherfolder
+  ./to-upload/myotherfolder/subfolder
+  ./to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
```

We'll now show examples of the different ways `files get` can be used. The command uses text to indicate the progress of each file download. For brevity, this output will be omitted, and we also won't list the resulting output on the local file system. To see the results, try the examples for yourself.

Copy the contents of a container to the current directory:
```
$ safe files get safe://hyryyryyn68cfxon3diif17w87nkj5mesc95f4noxnr85yqt6nj4qhbaaktjyeuy
<progress output omitted>
Done. Retrieved 4 files to .
```

Copy the contents of a subdirectory to the current directory:
```
$ safe files get safe://hyryyryyn68cfxon3diif17w87nkj5mesc95f4noxnr85yqt6nj4qhbaaktjyeuy/myfolder
<progress output omitted>
Done. Retrieved 1 files to .
```

If it doesn't already exist, this will create the `myfolder` directory locally.

If you wish `myfolder` to have a different name locally, use the destination argument:
```
$ safe files get safe://hyryyryyn68cfxon3diif17w87nkj5mesc95f4noxnr85yqt6nj4qhbaaktjyeuy/myfolder other
<progress output omitted>
Done. Retrieved 1 files to other
```

Copy the contents of a container to an existing directory:
```
$ mkdir target
$ safe files get safe://hyryyryyn68cfxon3diif17w87nkj5mesc95f4noxnr85yqt6nj4qhbaaktjyeuy target
<progress output omitted>
Done. Retrieved 4 files to target
```

Copy a file:
```
$ safe files get safe://hyryyryyn68cfxon3diif17w87nkj5mesc95f4noxnr85yqt6nj4qhbaaktjyeuy/file1.txt
<progress output omitted>
Done. Retrieved 1 files to .
```

You could also add a target for the file, which could be either an existing or non-existent directory or a new filename.

**Note**: Wildcards, e.g. *.txt, and set/range expansion, e.g. photo{1-3}.jpg, in the source URL path, are not supported at this time, but are planned for a future release.

#### Performance

Subfolder or single-file downloads from a container with thousands of files may be slower than expected.

This is because the entire container is fetched and locally filtered to obtain the XorUrl for each file that matches the source URL path.

Future releases may operate differently.

### Tree

The `files tree` command displays a visual representation of an entire directory tree of a container.

First, create a new container by uploading the example directory:
```
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hyryyryynrqxhdosmk1xr9bsz1a8jkc6a9mwmhxueqoiwueicknboakwdk7toeuy?v=h8h3mrhkzr793pwxdwga6i31stcr35ckhkegr51rfcgmufkjcrz5y"
+  ./to-upload/file1.txt                          safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
+  ./to-upload/file2.txt                          safe://hy8oycyybpgwwyx378g4b1da348kawo9i6xerxkot9w7xzwjht71awf55tj8o
+  ./to-upload/myfolder
+  ./to-upload/myfolder/file2.txt                 safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
+  ./to-upload/myotherfolder
+  ./to-upload/myotherfolder/subfolder
+  ./to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
```

Run the command to visualise the container:
```
$ safe files tree safe://hyryyryynrqxhdosmk1xr9bsz1a8jkc6a9mwmhxueqoiwueicknboakwdk7toeuy
safe://hyryyryynrqxhdosmk1xr9bsz1a8jkc6a9mwmhxueqoiwueicknboakwdk7toeuy
├── file1.txt
├── file2.txt
├── myfolder
│   └── file2.txt
└── myotherfolder
    └── subfolder
        └── file3.txt

3 directories, 4 files
```

You can also list a subfolder of the tree:
```
❯ safe files tree safe://hyryyryynrqxhdosmk1xr9bsz1a8jkc6a9mwmhxueqoiwueicknboakwdk7toeuy/myotherfolder
safe://hyryyryynrqxhdosmk1xr9bsz1a8jkc6a9mwmhxueqoiwueicknboakwdk7toeuy/myotherfolder
└── subfolder
    └── file3.txt

1 directory, 1 file
```

**Note**: a `--details` flag can be supplied to output the file sizes.

### Rm

Files and directories can be removed from a container using `files sync`, but it's also possible with the `files rm` command.

First, create a new container by uploading the example directory:
```
$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hyryyryyny8xnytj1rgad3siak49cyeuzfxnd8ggafpifcna1jj55b86914uyeuy?v=ht9kwqnhoxcrq9z9gwkwdkfk3dgig3fxi4uk88ynkqzex4nyenagy"
+  ./to-upload/file1.txt                          safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
+  ./to-upload/file2.txt                          safe://hy8oycyybpgwwyx378g4b1da348kawo9i6xerxkot9w7xzwjht71awf55tj8o
+  ./to-upload/myfolder
+  ./to-upload/myfolder/file2.txt                 safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
+  ./to-upload/myotherfolder
+  ./to-upload/myotherfolder/subfolder
+  ./to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
```

Remove `file1.txt` from the container:
```
$ safe files rm safe://hyryyryyny8xnytj1rgad3siak49cyeuzfxnd8ggafpifcna1jj55b86914uyeuy/file1.txt
FilesContainer updated (version hsh1bc78zckusbj3y43fsh3hj8uwdwprm7r9qc1u9uy5p7yyb58go): "safe://hyryyryyny8xnytj1rgad3siak49cyeuzfxnd8ggafpifcna1jj55b86914uyeuy?v=hsh1bc78zckusbj3y43fsh3hj8uwdwprm7r9qc1u9uy5p7yyb58go"
-  /file1.txt  safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
```

To remove a directory, use the `--recursive` flag:
```
$ safe files rm safe://hyryyryyny8xnytj1rgad3siak49cyeuzfxnd8ggafpifcna1jj55b86914uyeuy/myotherfolder --recursive
FilesContainer updated (version h6zr4xmy7pw6bcpcat5ofs4rt9zfu3x4shjctm1mqx8it7ucda8bo): "safe://hyryyryyny8xnytj1rgad3siak49cyeuzfxnd8ggafpifcna1jj55b86914uyeuy?v=h6zr4xmy7pw6bcpcat5ofs4rt9zfu3x4shjctm1mqx8it7ucda8bo"
-  /myotherfolder/subfolder
-  /myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
```

## Cat

We can retrieve and display content using the `cat` command.

Create a files container to give us some material to work with:
```
$ safe keys create --for-cli
New SafeKey created: "safe://hyryyyyyyft14jnfkkjos96bu5qtkg6tttzjoskjj4jucj6gxcaedxgm5xgto"
Key pair generated:
Public Key = 2c65a488aa52616ff833dba2a37a318dd30b2929d266c4f8cf661037997b79a3
Secret Key = e984d5bbee24991357178abe2e90e31f6e39b08e59e4a03b3f05385362563c53
Setting new SafeKey to be used by CLI...
New credentials were successfully stored in /home/chris/.safe/cli/credentials
Safe CLI now has write access to the network

$ safe files put ./to-upload/ --recursive
FilesContainer created at: "safe://hyryyryynqxwh3aadnwn111mc4db53e677ccewemq9ighy1fkpytfip8bbfjyeuy?v=hpgzkdo1b5b45k8k815b55uzco669zprxm7oq797c3p1zneq33pro"
+  to-upload/file1.txt                          safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
+  to-upload/myfolder
+  to-upload/myfolder/file2.txt                 safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
+  to-upload/myotherfolder
+  to-upload/myotherfolder/subfolder
+  to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
```

We generate persistent keys because we'll later add a binary file to the same container. Refer back
to the [Keys](#keys) section to see why this is required.

### Retrieve Files and Containers

Use the `cat` command to retrieve `file3.txt` with its XOR-URL:
```
$ safe cat safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
A text file with other stuff in it.
```

The content displayed depends on the content the URL points to. In this case, it was pointing to a
file, so `safe` displayed the file contents.

The URL of the container has a different content type, so try `cat` with that:
```
$ safe cat safe://hyryyryynqxwh3aadnwn111mc4db53e677ccewemq9ighy1fkpytfip8bbfjyeuy
Files of FilesContainer (version hpgzkdo1b5b45k8k815b55uzco669zprxm7oq797c3p1zneq33pro) at "safe://hyryyryynqxwh3aadnwn111mc4db53e677ccewemq9ighy1fkpytfip8bbfjyeuy":
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| Name                               | Type            | Size | Created    | Modified   | Link                                                                 |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /file1.txt                         | text/plain      | 29   | 1645402864 | 1645402864 | safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myfolder                          | inode/directory | 0    | 1645402864 | 1645402864 |                                                                      |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myfolder/file2.txt                | text/plain      | 35   | 1645402864 | 1645402864 | safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder                     | inode/directory | 0    | 1645402864 | 1645402864 |                                                                      |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder/subfolder           | inode/directory | 0    | 1645402864 | 1645402864 |                                                                      |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder/subfolder/file3.txt | text/plain      | 36   | 1645402864 | 1645402864 | safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
```

As we can see, it lists the contents of the container in a table.

You can also address the files in the container by appending a path to its URL:
```
$ safe cat safe://hyryyryynqxwh3aadnwn111mc4db53e677ccewemq9ighy1fkpytfip8bbfjyeuy/myfolder/file2.txt
A text file with some stuff in it.
```

### Retrieve Binary Files

The previous example retrieved a text file. If we retrieve a binary file, `cat` will also print the
file directly to the terminal. Since binary content isn't human readable, this isn't very useful. We
can use the `--hexdump` argument to print the file in the same fashion as a hex editor.

Upload a binary file to our example container (any image file will do):
```
$ safe files add to-upload/island.jpg safe://hyryyryynqxwh3aadnwn111mc4db53e677ccewemq9ighy1fkpytfip8bbfjyeuy
FilesContainer updated (version h8e3s1ur5hek97jmsacu8yg6ska18kdtf7hai1uf38icgfpxrx5uo): "safe://hyryyryynqxwh3aadnwn111mc4db53e677ccewemq9ighy1fkpytfip8bbfjyeuy?v=h8e3s1ur5hek97jmsacu8yg6ska18kdtf7hai1uf38icgfpxrx5uo"
+  to-upload/island.jpg  safe://hygoygyyb11oaeofunmfyej6c6q9ximnuphtxasyueb5jdgp3i5rgrdtikzbo
```

Now retrieve it using `--hexdump`:
```
$ safe -- cat safe://hygoygyyb11oaeofunmfyej6c6q9ximnuphtxasyueb5jdgp3i5rgrdtikzbo --hexdump
Length: 62317 (0xf36d) bytes
0000:   ff d8 ff e0  00 10 4a 46  49 46 00 01  01 00 00 01   ......JFIF......
0010:   00 01 00 00  ff db 00 84  00 04 04 04  04 05 04 05   ................
0020:   06 06 05 07  08 07 08 07  0a 0a 09 09  0a 0a 10 0b   ................
0030:   0c 0b 0c 0b  10 18 0f 11  0f 0f 11 0f  18 15 19 15   ................
0040:   13 15 19 15  26 1e 1a 1a  1e 26 2c 25  23 25 2c 35   ....&....&,%#%,5
0050:   2f 2f 35 43  3f 43 57 57  75 01 04 04  04 04 05 04   //5C?CWWu.......
0060:   05 06 06 05  07 08 07 08  07 0a 0a 09  09 0a 0a 10   ................
0070:   0b 0c 0b 0c  0b 10 18 0f  11 0f 0f 11  0f 18 15 19   ................
0080:   15 13 15 19  15 26 1e 1a  1a 1e 26 2c  25 23 25 2c   .....&....&,%#%,
0090:   35 2f 2f 35  43 3f 43 57  57 75 ff c2  00 11 08 01   5//5C?CWWu......
00a0:   fa 03 84 03  01 22 00 02  11 01 03 11  01 ff c4 00   ....."..........
00b0:   35 00 00 02  03 01 01 01  01 00 00 00  00 00 00 00   5...............
<remaining output snipped>
```

We could also use standard Unix redirection to output to file:
```
$ safe cat safe://hygoygyyb11oaeofunmfyej6c6q9ximnuphtxasyueb5jdgp3i5rgrdtikzbo > island.jpg
```

### Versioning

When the binary file was added, a new version of the container was created. We can use this to
demonstrate retrieval of specific versions. You can see the output of the `files put` and
`files add` commands present a version string for the container. This string is a hash of the
container content. The reason the version doesn't just use consecutive integers is because it would
be possible for the container to be updated by two or more people at the same time. We would then be
able to ask the user to resolve the conflicts between the updates.

Retrieve the first version of the container by supplying this version using the `v` query parameter
on its XOR-URL:
```
$ safe cat "safe://hyryyryynqxwh3aadnwn111mc4db53e677ccewemq9ighy1fkpytfip8bbfjyeuy?v=hpgzkdo1b5b45k8k815b55uzco669zprxm7oq797c3p1zneq33pro"
Files of FilesContainer (version hpgzkdo1b5b45k8k815b55uzco669zprxm7oq797c3p1zneq33pro) at "safe://hyryyryynqxwh3aadnwn111mc4db53e677ccewemq9ighy1fkpytfip8bbfjyeuy?v=hpgzkdo1b5b45k8k815b55uzco669zprxm7oq797c3p1zneq33pro":
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| Name                               | Type            | Size | Created    | Modified   | Link                                                                 |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /file1.txt                         | text/plain      | 29   | 1645402864 | 1645402864 | safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myfolder                          | inode/directory | 0    | 1645402864 | 1645402864 |                                                                      |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myfolder/file2.txt                | text/plain      | 35   | 1645402864 | 1645402864 | safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder                     | inode/directory | 0    | 1645402864 | 1645402864 |                                                                      |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder/subfolder           | inode/directory | 0    | 1645402864 | 1645402864 |                                                                      |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder/subfolder/file3.txt | text/plain      | 36   | 1645402864 | 1645402864 | safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
```

As expected, notice it doesn't contain the new `island.jpg` file.

Retrieve the current version by using the XOR-URL as normal:
```
$ safe cat "safe://hyryyryynqxwh3aadnwn111mc4db53e677ccewemq9ighy1fkpytfip8bbfjyeuy"
Files of FilesContainer (version h8e3s1ur5hek97jmsacu8yg6ska18kdtf7hai1uf38icgfpxrx5uo) at "safe://hyryyryynqxwh3aadnwn111mc4db53e677ccewemq9ighy1fkpytfip8bbfjyeuy":
+------------------------------------+-----------------+-------+------------+------------+----------------------------------------------------------------------+
| Name                               | Type            | Size  | Created    | Modified   | Link                                                                 |
+------------------------------------+-----------------+-------+------------+------------+----------------------------------------------------------------------+
| /file1.txt                         | text/plain      | 29    | 1645402864 | 1645402864 | safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy |
+------------------------------------+-----------------+-------+------------+------------+----------------------------------------------------------------------+
| /island.jpg                        | image/jpeg      | 62317 | 1645403232 | 1645403232 | safe://hygoygyyb11oaeofunmfyej6c6q9ximnuphtxasyueb5jdgp3i5rgrdtikzbo |
+------------------------------------+-----------------+-------+------------+------------+----------------------------------------------------------------------+
| /myfolder                          | inode/directory | 0     | 1645402864 | 1645402864 |                                                                      |
+------------------------------------+-----------------+-------+------------+------------+----------------------------------------------------------------------+
| /myfolder/file2.txt                | text/plain      | 35    | 1645402864 | 1645402864 | safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy |
+------------------------------------+-----------------+-------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder                     | inode/directory | 0     | 1645402864 | 1645402864 |                                                                      |
+------------------------------------+-----------------+-------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder/subfolder           | inode/directory | 0     | 1645402864 | 1645402864 |                                                                      |
+------------------------------------+-----------------+-------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder/subfolder/file3.txt | text/plain      | 36    | 1645402864 | 1645402864 | safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o |
+------------------------------------+-----------------+-------+------------+------------+----------------------------------------------------------------------+
```

Note the addition of the binary file, and the version, which is the current version. It would
also be possible to supply this version in the XOR-URL, but if it isn't supplied, `cat` will fetch
the latest version by default.

### Symlinks

The CLI supports upload and retrieval of symlinks. It can also resolve relative symlinks in a
container, provided the target exists.

More details on symlinks are available [here](README-symlinks.md).

## NRS

As we've seen, content on the network is accessible via XOR-URLs, but these can be hard to keep
track of. For this reason, Safe has a Name Resolution System (NRS) which is analogous to the
internet DNS system.

The main aspects to be aware of are:

* A 'fully qualified domain name' in DNS, e.g., `maps.google.com`, is a 'public name' in NRS.
* A 'top level domain' in DNS, e.g., `google.com`, is a 'top name' in NRS.
* A 'sub domain' in DNS, e.g., the `maps` part of `maps.google.com`, is a 'sub name' in NRS.

As usual, let's use our example directory to give us something to work with:
```
$ safe keys create --for-cli
New SafeKey created: "safe://hyryyyyyy4m3odsss55kg5iwx66in5go5j5nwyh6a3cg9hifa4acsdo9exify"
Key pair generated:
Public Key = d2f301dad6ded46dd68ff7aa2d9a1b4ec54073d8cb0dfe54b8d61961c3e87d4a
Secret Key = 1339bcd172d88e92ab7cd70d7d0fa4196706dd89b515e7d40646ef32fe898f54
Setting new SafeKey to be used by CLI...
New credentials were successfully stored in /home/chris/.safe/cli/credentials
Safe CLI now has write access to the network

$ safe files put to-upload/ --recursive
FilesContainer created at: "safe://hyryyryyn6j99m1ar1mc6hub3mcfwcddrgtqho3pobg6k9fjs1en77rztrjjyeuy?v=hw3yk4y1yzop438imze96wajpedexecauewhs46u1mmxnqi534hyy"
+  to-upload/file1.txt                          safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
+  to-upload/myfolder
+  to-upload/myfolder/file2.txt                 safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
+  to-upload/myotherfolder
+  to-upload/myotherfolder/subfolder
+  to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o
```

We generate keys because we'll add sub name entries to the top name we register. As the
[Keys](#keys) section explained, we'll require persistent keys to keep writing to the same data
structure, which is owned by the key that created it. Refer to that section again if you wish to
re-familiarise yourself.

### Register a Top Name

First, let's register a new top name, and we'll link it to the container:
```
$ safe nrs register example --link "safe://hyryyryyn6j99m1ar1mc6hub3mcfwcddrgtqho3pobg6k9fjs1en77rztrjjyeuy?v=hw3yk4y1yzop438imze96wajpedexecauewhs46u1mmxnqi534hyy"
New NRS Map created for "safe://example"
The container for the map is located at safe://hyryygyynqncd44jxc1yam9i6piz4cb5dbmuec1xzh56rmqkgoyrsabzr9mpomzy
The entry points to safe://hyryyryyn6j99m1ar1mc6hub3mcfwcddrgtqho3pobg6k9fjs1en77rztrjjyeuy?v=hw3yk4y1yzop438imze96wajpedexecauewhs46u1mmxnqi534hyy
+  example  safe://example
```

Note that the `--link` argument used the version of the container. This is required because when
linking to versionable content, NRS requires us to specify the version we wish to link to. A files
container is a mutable structure and is therefore versionable.

Also important to note is, the output supplies an XOR-URL to a container for the NRS map. This is
where the data related to the top name is stored. The NRS map is a list of all the sub names for the
top name, and the content they point to. We'll come back to this concept when we create a sub name.

For now, let's retrieve the content using its NRS name:
```
$ safe cat safe://example
Files of FilesContainer (version hw3yk4y1yzop438imze96wajpedexecauewhs46u1mmxnqi534hyy) at "safe://example":
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| Name                               | Type            | Size | Created    | Modified   | Link                                                                 |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /file1.txt                         | text/plain      | 29   | 1645407121 | 1645407121 | safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myfolder                          | inode/directory | 0    | 1645407121 | 1645407121 |                                                                      |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myfolder/file2.txt                | text/plain      | 35   | 1645407121 | 1645407121 | safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder                     | inode/directory | 0    | 1645407121 | 1645407121 |                                                                      |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder/subfolder           | inode/directory | 0    | 1645407121 | 1645407121 |                                                                      |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
| /myotherfolder/subfolder/file3.txt | text/plain      | 36   | 1645407121 | 1645407121 | safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o |
+------------------------------------+-----------------+------+------------+------------+----------------------------------------------------------------------+
```

It lists the container, in the same way it would if we had used its XOR-URL directly.

It's also possible to register a top name without linking it to any content.

### Add a Sub Name

Let's add a sub name for the `example` top name, and link it to a file in the container:
```
$ safe nrs add file1.example --link safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
Existing NRS Map updated.
Now at version hf6zjb8j3d4nezh917extkumwf5f8nxry38inwq9z66cwyewuqh4o.
+  file1.example  safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
```

Note that we didn't use a version with the `--link` argument. On Safe, a file is immutable content
and therefore has no version.

Now retrieve the file:
```
$ safe cat safe://file1.example
A file with some text in it.
```

Add another sub name with a link to the second file, then retrieve it:
```
$ safe nrs add file2.example --link safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
Existing NRS Map updated.
Now at version hra3toibhg1zg6h9y4do4g6s177iejkqfx9enpbroq61z5zrnpryo.
+  file2.example  safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy

$ safe cat safe://file2.example
A text file with some stuff in it.
```

Finally, it's possible to add a sub name and register a top name in one step. Run `nrs add` with the
`--register-top-name` flag:
```
$ safe nrs add file2.example2 --link safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy --register-top-name
New NRS Map created.
The container for the map is located at safe://hyryygyynwfb5q7rzss18dnfqomgkzqfdr5hamzrn881ubh74mykwhk4g3mbomzy
Now at version h4dprogdr98hj9ecrygfoico5o66r1dfwgdnt9z13m57n3bnm9phy.
+  file2.example2  safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy

$ safe cat safe://file2.example2
A text file with some stuff in it.
```

Here we've registered `example2` and created a `file2` sub name. We linked it to the same file, just
to illustrate the point.

### List the NRS Map

We can see all the sub names for a registered top name by retrieving the content of the container
where the NRS map is stored.  Use the container address supplied when the top name was created:
```
$ safe cat safe://hyryygyynqncd44jxc1yam9i6piz4cb5dbmuec1xzh56rmqkgoyrsabzr9mpomzy
NRS Map Container at safe://hyryygyynqncd44jxc1yam9i6piz4cb5dbmuec1xzh56rmqkgoyrsabzr9mpomzy
Listing NRS map contents:
example: safe://hyryyryyn6j99m1ar1mc6hub3mcfwcddrgtqho3pobg6k9fjs1en77rztrjjyeuy?v=hw3yk4y1yzop438imze96wajpedexecauewhs46u1mmxnqi534hyy
file1.example: safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
file2.example: safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
```

The output has all the sub names for our `example` top name, and all the associated links.

## Dog

The `dog` command provides us with information about content on the network, namely, how it is
resolved and the data types that represent it.

Let's setup some context for illustrating `dog`. Create a container with the usual example directory
and associate some NRS entries with it:
```
$ safe keys create --for-cli
New SafeKey created: "safe://hyryyyyyy78fnjgmwzw118jicqc1b6c6917h7jf3t4xnpbioas9fqxabrba1y"
Key pair generated:
Public Key = e9ca249974bd2523a6ac73241f33df9779d49731d3c4d0d618b7cae7e0240e24
Secret Key = d8609a4e885199af4a4535bd87d94af309d843e2f647bb9da8f0f6f162f11baf
Setting new SafeKey to be used by CLI...
New credentials were successfully stored in /home/chris/.safe/cli/credentials
Safe CLI now has write access to the network

$ safe files put to-upload/ --recursive
FilesContainer created at: "safe://hyryyryyng6ymimc9yjrio1q6xgmdj73dj5gmnd999fgmw7swdrmtd364jzoyeuy?v=hj4i3bs677desbkkjkg683pmwqgzei1yfbtgf7t9mmj14ojt445yy"
+  to-upload/file1.txt                          safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
+  to-upload/myfolder
+  to-upload/myfolder/file2.txt                 safe://hy8oycyybrqkkwrnmneshqetpnzfoncfw9qznm331515xk936hm1gsrkkw1cy
+  to-upload/myotherfolder
+  to-upload/myotherfolder/subfolder
+  to-upload/myotherfolder/subfolder/file3.txt  safe://hy8oycyybut5ea65nec5q4s8tpouws8ax5ej1jazu9c9r8e5p3ry97xkhdp7o

$ safe nrs register example --link "safe://hyryyryyng6ymimc9yjrio1q6xgmdj73dj5gmnd999fgmw7swdrmtd364jzoyeuy?v=hj4i3bs677desbkkjkg683pmwqgzei1yfbtgf7t9mmj14ojt445yy"
New NRS Map created for "safe://example"
The container for the map is located at safe://hyryygyynqncd44jxc1yam9i6piz4cb5dbmuec1xzh56rmqkgoyrsabzr9mpomzy
The entry points to safe://hyryyryyng6ymimc9yjrio1q6xgmdj73dj5gmnd999fgmw7swdrmtd364jzoyeuy?v=hj4i3bs677desbkkjkg683pmwqgzei1yfbtgf7t9mmj14ojt445yy
+  example  safe://example

$ safe nrs add file1.example --link safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
Existing NRS Map updated.
Now at version hf6zjb8j3d4nezh917extkumwf5f8nxry38inwq9z66cwyewuqh4o.
+  file1.example  safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
```

Start by running the command against `file1.txt`:
```
$ safe dog safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
== URL resolution step 1 ==
Resolved from: safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
= File =
XOR-URL: safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
XOR name: 0xea4aeb538a8dc5fc738a97057de47c288856fb99d216622d3731890eed155360
Native data type: PublicFile
Media type: text/plain
```

This tells us the network resolved the link in one step and it refers to a `PublicFile`.

Similarly, if we run it against the container XOR-URL, it too has one resolution step:
```
$ safe dog safe://hyryyryyng6ymimc9yjrio1q6xgmdj73dj5gmnd999fgmw7swdrmtd364jzoyeuy
== URL resolution step 1 ==
Resolved from: safe://hyryyryyng6ymimc9yjrio1q6xgmdj73dj5gmnd999fgmw7swdrmtd364jzoyeuy
= FilesContainer =
XOR-URL: safe://hyryyryyng6ymimc9yjrio1q6xgmdj73dj5gmnd999fgmw7swdrmtd364jzoyeuy
Version: hj4i3bs677desbkkjkg683pmwqgzei1yfbtgf7t9mmj14ojt445yy
Type tag: 1100
XOR name: 0x3780baad9f02495849de799634f7234eccb10ffff94cba76d4191711e7da4de0
Native data type: Register
Native data XOR-URL: safe://hyryyyyyng6ymimc9yjrio1q6xgmdj73dj5gmnd999fgmw7swdrmtd364jzoyeuy
```

The output also tells us the container is represented by a `Register`, which is generic storage
designed to work with our versioning system and support mechanisms for resolving conflicts that can
occur during concurrent writes.

Let's try the command using an NRS-URL:
```
$ safe dog safe://file1.example
== URL resolution step 1 ==
Resolved from: safe://file1.example
= NrsEntry =
Public name: file1.example
Target XOR-URL: safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
Target native data type: File
Resolves into: safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
Version: none

== URL resolution step 2 ==
Resolved from: safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
= File =
XOR-URL: safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
XOR name: 0xea4aeb538a8dc5fc738a97057de47c288856fb99d216622d3731890eed155360
Native data type: PublicFile
Media type: text/plain
```

We get some insight into the NRS system. The first step was to resolve the `file1.example`
`NrsEntry`, which then points to the file. The second step was to retrieve the target file.

Finally, let's try `dog` against the NRS container:
```
$ safe dog safe://hyryygyynqncd44jxc1yam9i6piz4cb5dbmuec1xzh56rmqkgoyrsabzr9mpomzy
== URL resolution step 1 ==
= NRS Map Container =
XOR-URL: safe://hyryygyynqncd44jxc1yam9i6piz4cb5dbmuec1xzh56rmqkgoyrsabzr9mpomzy
Type tag: 1500
XOR name: 0x70983d692f648185febe6d6fa607630ae68649f7e6fc45b94680096c06e4fadb
Native data type: Register
Native data XOR-URL: safe://hyryyyyynqncd44jxc1yam9i6piz4cb5dbmuec1xzh56rmqkgoyrsabzr9mpomzy
Listing NRS map contents:
example: safe://hyryyryyng6ymimc9yjrio1q6xgmdj73dj5gmnd999fgmw7swdrmtd364jzoyeuy?v=hj4i3bs677desbkkjkg683pmwqgzei1yfbtgf7t9mmj14ojt445yy
file1.example: safe://hy8oycyyb7jfqswhktzn9ahhk1hnz53dhfnrfp6h34emgrmjzggro75eikpoy
```

Like the file container, the output tells us this container is also represented by a `Register`, and
it also prints the NRS map.

## Further Help

If you want further help or information related to using the CLI, or perhaps more details about the
internals of the network, please feel free to join us and ask questions on the [Safe Network
Forum](https://safenetforum.org/).

If you find any issues, or have ideas for improvements and/or new features for this application and
the project, please raise them by [creating a new issue in this
repository](https://github.com/maidsafe/sn_cli/issues).

## License

This Safe Network repository is licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [Guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
