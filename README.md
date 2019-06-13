|Documentation|Linux/macOS/Windows|
|:-----------:|:-----------------:|
| [![Documentation](https://docs.rs/safe-cli/badge.svg)](https://docs.rs/safe-cli) | [![Build Status](https://travis-ci.com/maidsafe/safe-cli.svg?branch=master)](https://travis-ci.com/maidsafe/safe-cli) |

| [MaidSafe website](https://maidsafe.net) | [SAFE Dev Forum](https://forum.safedev.org) | [SAFE Network Forum](https://safenetforum.org) |
|:----------------------------------------:|:-------------------------------------------:|:----------------------------------------------:|

# SAFE CLI
This crate implements a CLI (Command Line Interface) for the SAFE Network.

The SAFE CLI provides all the tools necessary to interact with the SAFE Network, including storing and browsing data of any kind, following links that are contained in the data and using their addresses on the network. Thus using the CLI users have access to any type of operation that can be made on SAFE Network data, allowing them to also use it for automated scripts in a piped chain of commands.

For further information please see https://safenetforum.org/t/safe-cli-high-level-design-document/28690


## Table of contents

1. [Build](#build)
2. [Using the CLI](#using-the-cli)
    - [Auth](#auth)
        - [Prerequisite: run the Authenticator](#prerequisite-run-the-authenticator)
        - [Authorise the safe_cli app](#authorise-the-safe_cli-app)
	- [Keys](#keys)
		- [Create](#keys-creation)
		- [Balance](#keys-balance)
	- [Wallet](#wallet)
		- [Create](#wallet-creation)
		- [Insert](#wallet-insert)
		- [Balance](#wallet-balance)
		- [Transfer](#wallet-transfer)
3. [Further Help](#further-help)
4. [License](#license)


## Build

In order to build this CLI from source code you need to make sure you have `rustc v1.35.0` (or higher) installed. Please take a look at this [notes about Rust installation](https://www.rust-lang.org/tools/install) if you need help with installing it. We recommend you install it with `rustup` which will install `cargo` tool since this guide makes use of it.

Once Rust and its toolchain are installed, run the following commands to clone this repository and build the `safe_cli` crate (the build process may take several minutes the first time you run it on this crate):
```
$ git clone https://github.com/maidsafe/safe-cli.git
$ cd safe-cli
$ cargo build
```

## Using the CLI

Right now the CLI is under active development. Here we're listing commands ready to be tested (against mock).

The base command, if built is `$ safe_cli`, or all commands can be run via `$ cargo run -- <command>`.

Various global flags are available (those commented out are not yet implemented):

```bash
# --dry-run              Dry run of command. No data will be written. No coins spent.
-h, --help               Prints help information
--pretty                 Print human readable responses. (Alias to --output human-readable.)
# --root                 The account's Root Container address
-V, --version            Prints version information
# -v, --verbose          Increase output verbosity. (More logs!)
# -o, --output <output>  Output data serlialisation
# -q, --query <query>    Enable to query the output via SPARQL eg.
--xorurl <xorurl_base>   Base encoding to be used for XOR-URLs generated. Currently supported: base32 (default) and base32z
```

#### `--help`

All commands have a `--help` function which will list args, options and subcommands.

### Auth

The CLI is just another client SAFE application, therefore it needs to be authorised by the user to gain access to the SAFE Network on behalf of the user. The CLI `auth` command allows us to obtain such authorisation from the account owner (the user) via the SAFE Authenticator.

This command simply sends an authorisation request to the Authenticator available, e.g. the safe_auth CLI daemon (see further bellow for explanation of how to run it), and it then stores the authorisation response (credentials) in `<user's home directory>/.safe/credentials` file. Any subsequent CLI command will read the `~/.safe/credentials` file, to obtain the credentials and connect to the network for the corresponding operation.

#### Prerequisite: run the Authenticator

You need the [SAFE Authenticator CLI](https://github.com/maidsafe/safe-authenticator-cli) running locally and exposing its WebService interface for authorising applications, and also be logged in to a SAFE account created on the mock network (i.e. `MockVault` file), making sure the port number you set is `41805`, and enabling the `mock-network` feature.

Please open a second/separate terminal console to execute the following commands (again, please make sure you have `rustc v1.35.0` or higher):
```
$ git clone https://github.com/maidsafe/safe-authenticator-cli.git
$ cd safe-authenticator-cli
$ cargo run --features mock-network -- --daemon 41805
```

#### Authorise the safe_cli app

Now that the Authenticator is running and ready to authorise applications, we can simply invoke the `auth` command:
```bash
$ safe_cli auth
```

At this point, you need to authorise the application on the Authenticator, you should see a prompt like the following:
```bash
The following application authorisation request was received:
+------------------+----------+------------------+------------------------+
| Id               | Name     | Vendor           | Permissions requested  |
+------------------+----------+------------------+------------------------+
| net.maidsafe.cli | SAFE CLI | MaidSafe.net Ltd | Own container: false   |
|                  |          |                  | Default containers: {} |
+------------------+----------+------------------+------------------------+
Allow authorisation? [y/N]:
```


### Keys

`Key` management allows users to generate sign/encryption key pairs that can be used for different type of operations, like choosing which sign key to use for uploading files (and therefore paying for the storage used), or signing a message posted on some social application when a `Key` is linked from a public profile (e.g. a WebID/SAFE-ID), or even for encrypting messages that are privately sent to another party so it can verify the authenticity of the sender.

Users can record `Key`'s in a `Wallet` (see further below for more details about `Wallet`'s), having friendly names to refer to them, but they can also be created as throw away `Key`'s which are not linked from any `Wallet`, container, or any other type of data on the network.

Note that even though the key pair is automatically generated by the CLI, `Key`s don’t hold the secret key on the network but just the public key, and `Key`s optionally can have a safecoin balance attached to it. Thus `Key`'s can also be used for safecoin transactions. In this sense, a `Key` can be compared to a Bitcoin address, it has a coin balance associated to it, such balance can be queried using the public key (since its location on the network is based on the public key itself), but in order to spend its balance the corresponding secret key needs to be provided in the `transfer` request. The secret key can be provided by the user, or retrieved from a `Wallet`, at the moment of creating the transaction (again, see the [`Wallet` section](#wallet) below for more details)

#### Keys Creation

To generate a key pair and create a new `Key` on the network, a `source` address is needed to pay for storage costs:
```shell
$ safe_cli keys create <source>
```

But we can also create a `Key` with test-coins since we are using the mock network:
```shell
$ safe_cli keys create --test-coins --preload 15.342 --pretty
New Key created at: "safe://bbkulcbnrmdzhdkrfb6zbbf7fisbdn7ggztdvgcxueyq2iys272koaplks"
Key pair generated: pk="b62c1e4e3544a1f64212fca89046df98d998ea615e84c4348c4b5fd29c07ad52a970539df819e31990c1edf09b882e61", sk="c4cc596d7321a3054d397beff82fe64f49c3896a07a349d31f29574ac9f56965"
```

Once we have some `Key`'s with some test-coins we can use them as the `source` for the creation of new `Key`'s, thus if we use the `Key` we just created with test-coins we can create a second `Key`:
```shell
$ safe_cli keys create --preload 8.15 safe://bbkulcbnrmdzhdkrfb6zbbf7fisbdn7ggztdvgcxueyq2iys272koaplks --pretty
Enter secret key corresponding to public key at XOR-URL "safe://bbkulcbnrmdzhdkrfb6zbbf7fisbdn7ggztdvgcxueyq2iys272koaplks":
New Key created at: "safe://bbkulcbf2uuqwawvuonevraqa4ieu375qqrdpwvzi356edwkdjhwgd4dum"
Key pair generated:
pk="9754a42c0b568e692b10401c4129bff61088df6ae51bef883b28693d8c3e0e8ce23054e236bd64edc45791549ef60ce1"
sk="2f211ad4606c716c2c2965e8ea2bd76a63bfc5a5936b792cda448ddea70a031c"
```

Other optional args that can be used with `keys create` sub-command are:
```shell
--pk <pk>            Don't generate a key pair and just use the provided public key
--preload <preload>  Preload the Key with a coin balance
```

#### Key's Balance

We can retrieve a given `Key`'s balance using its XorUrl.

The target `Key` can be passed as an argument (or it will be retrieved from `stdin`)
```
$ safe_cli keys balance safe://bbkulcbnrmdzhdkrfb6zbbf7fisbdn7ggztdvgcxueyq2iys272koaplks --pretty
Key's current balance: 15.342
```

### Wallet

A `Wallet` is a specific type of Container on the network, holding a set of spendable safecoin balances.

A `Wallet` effectively contains links to `Key`'s which have safecoin balances attached to them, but the `Wallet` also stores the secret keys needed to spend them. `Wallet`'s are stored encrypted and only accessible to the owner by default.

There are several sub-commands that can be used to manage the `Wallet`'s with the `safe_cli wallet` command (those commented out are not yet implemented):

```shell
SUBCOMMANDS:
    balance       Query a Wallet's total balance
    # check-tx    Check the status of a given transaction
    create        Create a new Wallet
    help          Prints this message or the help of the given subcommand(s)
    insert        Insert a spendable balance into a Wallet
    # sweep       Move all coins within a Wallet to a second given Wallet or Key
    transfer      Transfer safecoins from one Wallet, Key or pk, to another.
```

#### Wallet Creation

We can create a new `Wallet` by simply running:
```shell
$ safe_cli wallet create --pretty
Wallet created at: "safe://bbkulcbthsrih6ot7mfwus6oa4xeonv5y7wwm2ucjeypgtwrmdk5db7fqy"
```

#### Wallet Balance

The balance of a given `Wallet` can be queried using its XorUrl. This returns the balance of the whole `Wallet`, including the contained spendable balances, or any child wallets (this is not implemented just yet).

The target `Wallet` can be passed as an argument (or it will be retrieved from `stdin`):
```shell
$ safe_cli wallet balance safe://bbkulcbthsrih6ot7mfwus6oa4xeonv5y7wwm2ucjeypgtwrmdk5db7fqy --pretty
Wallet at "safe://bbkulcakdcx2jxw2gfyvh7klkacht652c2pog3pohhpmiri73qjjpd2vks" has a total balance of 0 safecoins
```

#### Wallet Insert

As mentioned before, a `Key` doesn't hold the secret key on the network, therefore even if it has some non-zero coin balance, it cannot be spent. This is where the `Wallet` comes into play, holding the links to `Key`'s, and making their balances spendable by storing the correspondig secret keys.

We achieve this by `insert`-ing into a `Wallet` a link to a `Key`, making it a spendable balance.

```shell
USAGE:
    safe_cli wallet insert [FLAGS] [OPTIONS] <source> [ARGS]

OPTIONS:
    --name <name>              The name to give this spendable balance
    -s, --secret-key <secret>  Optionally pass the secret key to make the balance spendable

ARGS:
    <source>     The source Wallet for funds
    <target>     The target Wallet to insert the spendable balance
    <key>        An existing `Key`'s safe://xor-url. If this is not supplied, a new `Key` will be automatically generated and inserted
```

- The `<source>` is the `Wallet` paying for data creation/mutation.
- The `<target>` is the `Wallet` to insert the spendable balance to.
- The `<key>` allows passing an existing `Key` XorUrl, which we'll be used to generate the spendable balance.
- The `--name` is an optional nickname to give a spendable balance for easy reference,
- The `--default` flag sets _this_ new spendable balance as the default for the containing `Wallet`. This can be used by wallet applications to apply some logic on how to spend and/or choose the balances for a transaction.

With the above options, the user will be prompted to input the secret key associated with the public key. This is stored in the `Wallet`.

Otherwise, there's also the `--secret-key` argument, which when combined with `--key` can pass the `Key` XorUrl as part of the command line instruction itself, e.g.:

```shell
$ safe_cli wallet insert <source wallet> --target safe://wallet-xorurl --key safe://key-xor-url --name my_default_balance --default --pretty
Enter secret key corresponding to public key at safe://key-xor-url:
b493a84e3b35239cbffdf10b8ebfa49c0013a5d1b59e5ef3c000320e2d303311
Spendable balance inserted with name 'my_default_balance' in Wallet located at "safe://wallet-xorurl"
```

#### Wallet Transfer

Once a `Wallet` contains some spendable balance/s, we can transfer `<from>` a `Wallet` an `<amount>` of safecoins `<to>` another `Wallet` or `Key`. The destination `Wallet`/`Key` currently must be passed as a XorUrl in the arguments list, but reading it from `stdin` will be supported later on.

Both the `<from>` and `<to>` Wallets must have a _default_ spendable balance for the transfer to succeed. In the future different type of logics will be implemented for using different Wallet's balances and not just the default one.

```shell
$ safe_cli wallet transfer <amount> <to> <from>
```
E.g.:
```shell
$ safe_cli wallet transfer 323.23 safe://7e0ae5e6ed15a8065ea03218a0903b0be7c9d78384998817331b309e9d23566e safe://6221785c1a20163bbefaf523af15fa525d83b00be7502d28cae5b09ac54f4e75 --pretty
Transaction Success. Tx_id: 44dcd919-0703-4f23-a9a2-6b6be8da0bcc
```

## Further Help

You can discuss development-related topics on the [SAFE Dev Forum](https://forum.safedev.org/).
If you are just starting to develop an application for the SAFE Network, it's very advisable to visit the [SAFE Network Dev Hub](https://hub.safedev.org) where you will find a lot of relevant information.

## License
This SAFE Network application is licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).

## Contribute
Copyrights in the SAFE Network are retained by their contributors. No copyright assignment is required to contribute to this project.
