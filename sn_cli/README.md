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
- [Xorurl](#xorurl)
- [Xorurl decode](#xorurl-decode)
- [Cat](#cat)
  - [Retrieving Binary Files](#retrieving-binary-files)
  - [Retrieving Older Versions](#retrieving-older-versions)
- [Safe-URLs](#safe-urls)
  - [Symlinks](#symlinks)
- [Dog](#dog)
- [Shell Completions](#shell-completions)
  - [Bash Completions](#bash-completions)
  - [Windows PowerShell Completions](#windows-powershell-completions)
- [Further Help](#further-help)
- [License](#license)
- [Contributing](#contributing)

## Description

This crate implements a CLI (Command Line Interface) for the Safe Network.

The Safe CLI provides all the tools necessary to interact with the Safe Network, including storing and browsing data of any kind, following links contained in the data, using their addresses on the network, and much more. Using the CLI, users have access to any type of operation that can be made on the Safe Network and the data stored on it. Due to it being a CLI, it can also be used in automated scripts and Unix-style piping and redirection.

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

## Xorurl

As we've seen, when uploading files to the network, each file is uploaded as one or more chunks using the [self-encryption algorithm](https://github.com/maidsafe/self_encryption) in the client, splitting the files into encrypted chunks, and the resulting file's XOR-URL is linked from a `FilesContainer`.

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

### Decode

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

## Cat

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

As seen above, the `safe cat` command can be used to fetch any type of content from the Safe Network. At this point it only supports files, `FilesContainer`s, `Wallet`s, and `NRS-Container`s (see further below about NRS Containers and commands), but it will be expanded as more types are supported by the CLI and its API.

### Retrieving Binary Files

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

### Retrieving Older Versions

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

## Safe-URLs

In previous sections of this guide we explained how we can create two types of safe:// URLs, XOR-URLs and NRS-URLs. It has been explained that safe:// URLs can contain a path as well, if they target a `FilesContainer`, and they can also be post-fixed with `v=<version>` query param in order to target a specific version of the content rather than the latest/current version when this query param is omitted.

All these types of safe:// URLs can be used in any of the supported CLI commands interchangeably as the argument of any command which expects safe:// URL.

E.g. we can retrieve the content of a website with the `cat` command using either its XOR-URL or its NRS-URL, and either fetching the latest version of it or supplying the query param to get a specific version of it. Thus, if we wanted to fetch `version #1` of the site we published at `safe://mywebsite` (which NRS Map Container XOR-URL is `safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh`), the following two commands would be equivalent:
- `$ safe cat "safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh?v=1"`
- `$ safe cat "safe://mywebsite?v=1"`

In both cases the NRS Map Container will be found (from above URLs) by decoding the XOR-URL or by resolving NRS public name. Once that's done, and since the content is an NRS Map, following the rules defined by NRS and the map found in it the target link will be resolved from it. In some circumstances, it may be useful to get information about the resolution of a URL, which can be obtained using the `dog` command.

## Symlinks

The sn_cli supports upload and retrieval of symlinks using the above commands. It can also resolve relative symlinks in a FileContainer provided that the target exists in the FileContainer.

[More Details](README-symlinks.md)

## Dog

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

Of course the `safe dog` command can be used also with other type of content like files, e.g. if we use it with a `FilesContainer`'s XOR-URL and the path of one of the files it contains:
```shell
$ safe dog safe://hnyynywttiyr6tf3qk811b3rto9azx8579h95ewbs3ikwpctxdhtqesmwnbnc/subfolder/index.html
Native data type: PublicFile
XOR name: 0xda4ce4aa59889874921817e79c2b98dc3dbede7fd9a9808a60aa2d35efaa05f4
XOR-URL: safe://hbhybyds1ch1ifunraq1jbof98uoi3tzb7z5x89spjonfgbktpgzz4wbxw
Media type: text/html
```

But how about using the `dog` command with an NRS URL, as we now know it's resolved using the NRS rules and following the links found in the NRS Map Container:
```shell
$ safe dog safe://mywebsite/contact/form.html
Native data type: PublicFile
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

## Shell Completions

Automatic command completions via <tab> are available for popular shells such as bash and PowerShell (Windows). Completions are also provided for the shells fish, zsh, and elvish.

Until an installer becomes available, these completions must be manually enabled as per below.

### Bash Completions

To enable bash completions in the current bash session, use the following command:
```shell
SC=/tmp/safe.rc && safe setup completions bash > $SC && source $SC
```

To enable bash completions always for the current user:
```shell
SC=~/.bash_sn_cli CL="source $SC" RC=~/.bashrc; safe setup completions bash > $SC && grep -qxF "$CL" $RC || echo $CL >> $RC
```

### Windows PowerShell Completions

To enable completions in the current PowerShell session, use the following commands:
```shell
safe setup completions bash > sn_cli.ps1
sn_cli.ps1
```

To enable PowerShell completions permanently, generate the sn_cli.ps1 file as per above and then see this [stackoverflow answer](<https://stackoverflow.com/questions/20575257/how-do-i-run-a-powershell-script-when-the-computer-starts#32189430>).

## Further Help

You can discuss development-related topics on the [Safe Dev Forum](https://forum.safedev.org/).

If you are just starting to develop an application for the Safe Network, it's very advisable to visit the [Safe Network Dev Hub](https://hub.safedev.org) where you will find a lot of relevant information.

If you find any issues, or have ideas for improvements and/or new features for this application and the project, please raise them by [creating a new issue in this repository](https://github.com/maidsafe/sn_cli/issues).

## License

This Safe Network repository is licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).

## Contributing

Want to contribute? Great :tada:

There are many ways to give back to the project, whether it be writing new code, fixing bugs, or just reporting errors. All forms of contributions are encouraged!

For instructions on how to contribute, see our [Guide to contributing](https://github.com/maidsafe/QA/blob/master/CONTRIBUTING.md).
