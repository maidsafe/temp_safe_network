|Documentation|Linux/macOS/Windows|
|:-----------:|:-----------------:|
| [![Documentation](https://docs.rs/safe-cli/badge.svg)](https://docs.rs/safe-cli) | [![Build Status](https://travis-ci.com/maidsafe/safe-cli.svg?branch=master)](https://travis-ci.com/maidsafe/safe-cli) |

| [MaidSafe website](https://maidsafe.net) | [SAFE Dev Forum](https://forum.safedev.org) | [SAFE Network Forum](https://safenetforum.org) |
|:----------------------------------------:|:-------------------------------------------:|:----------------------------------------------:|

# SAFE CLI
This crate implements a CLI (Command Line Interface) for the SAFE Network.

For further information please see https://safenetforum.org/t/safe-cli-high-level-design-document/28690


## Table of contents

1. [Build](#build)
1. [Using the CLI](#using-the-cli)
	- [Keys](#keys)
		- [Balance](#keys-balance)
		- [Create](#keys-create)
	- [Wallet](#wallet)
		- [Balance](#wallet-balance)
		- [Create](#wallet-create)
		- [Transfer](#wallet-transfer)
1. [Further Help](#further-help)
1. [License](#license)


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

```shell
    # --dry-run    Dry run of command. No data will be written. No coins spent.
    -h, --help       Prints help information
    --pretty     Print human readable responses. (Alias to --output human-readable.)
    #--root       The account's Root Container address
    -V, --version    Prints version information
    #-v, --verbose    Increase output verbosity. (More logs!)
```
### `--help`

All commands have a `--help` function which will list args, options and subcommands.

### Keys
#### Keys Balance

Retrieve a given key's balance from a target XorUrl.

Target can be passed as an arg, or retrieved from `stdin`
```
$ safe_cli keys balance <target>
> 33
```

#### Keys Create

Creates a new `Key` on the network. A `payee` address is needed to pay for PUTs
```shell
$ safe_cli keys create <payee>

## or via cargo and with test-coins added
$ cargo run -- keys create --test-coins --preload 15.342 --pretty

New Key created at XOR-URL: "safe://bbkulcbnrmdzhdkrfb6zbbf7fisbdn7ggztdvgcxueyq2iys272koaplks"
Key pair generated: pk="b62c1e4e3544a1f64212fca89046df98d998ea615e84c4348c4b5fd29c07ad52a970539df819e31990c1edf09b882e61", sk="c4cc596d7321a3054d397beff82fe64f49c3896a07a349d31f29574ac9f56965"
```

Other optional args includes:
```shell
--test-coins    Create a Key and allocate test-coins onto it
--pk <pk>       Don\'t generate a key pair and just use the provided public key
--preload <preload>  Preload the key with a coin balance
```
// TODO: Do we need to enable `--anon` functionality here?


### Wallet

Manage a wallet container and safecoin funds therein.

```shell
SUBCOMMANDS:
    balance     Query a new Wallet or PublicKeys CoinBalance
    #check-tx    Check the status of a given transaction
    create      Create a new Wallet/CoinBalance
    help        Prints this message or the help of the given subcommand(s)
    insert      Insert a spendable balance into a wallet
    #sweep       Move all coins within a wallet to a given balance
    transfer    Transfer safecoins from one wallet, spendable balance or pk to another.
```

#### Wallet Creates

Create a new wallet container.

```shell
$ safe_cli wallet create --pretty
> Wallet created at XOR-URL: "safe://bbkulcbthsrih6ot7mfwus6oa4xeonv5y7wwm2ucjeypgtwrmdk5db7fqy"
```
#### Wallet Balance

Retrieve a given wallet's balance from the wallet XorUrl. This returns the balance of the whole wallet, including any contains spendable balances, or child wallets.

Target can be passed as an arg, or retrieved from `stdin`
```shell
$ safe_cli wallet balance <target>
> 33
```

// TODO: stdin functionality not yet in place.

#### Wallet Transfer

Transfer an `<amount>` of safecoin `<to>` another wallet, `<from>` a wallet. This currently must be passed as a XorUrl, but `stdin` will be supported later.

Both wallets must have _default_ entries set so far.

```shell
$ safe_cli wallet transfer <amount> <to> <from> --pretty
# eg:
safe://7e0ae5e6ed15a8065ea03218a0903b0be7c9d78384998817331b309e9d23566e safe://6221785c1a20163bbefaf523af15fa525d83b00be7502d28cae5b09ac54f4e75 --pretty
Transaction Success. Tx_id: 44dcd919-0703-4f23-a9a2-6b6be8da0bcc
```

#### Wallet Insert

Insert a public key into a wallet to make it a spendable balance.

- The `<payee>` is the wallet paying for data creation.
- The `--name` is an optional nickname to give a wallet for easy reference,
- The `--default` flag sets _this_ new spendable balance as the default for the containing wallet.
- The `--target` is the wallet to insert the spendable balance too.
- The `--key` is the existing key on the network we'll be using to generate the spendable balance.

With the above options, the user will be prompted to input the secret key associated with the public key. This is stored in the wallet.

Otherwise, there's also the `--secret-key` argument which can pass the key as part of the command line instruction itself.

```shell
$ safe_cli wallet insert safe://pk-xor-url safe://wallet-xorurl safe://pk-xor-url --name my_wallet --default --pretty
Enter secret key corresponding to public key at XOR-URL "safe://pk-xor-url": b493a84e3b35239cbffdf10b8ebfa49c0013a5d1b59e5ef3c000320e2d303311
Spendable balance added with name 'my_wallet' in wallet located at XOR-URL "safe://wallet-xorurl"

```

## Further Help

You can discuss development-related questions on the [SAFE Dev Forum](https://forum.safedev.org/).
If you are just starting to develop an application for the SAFE Network, it's very advisable to visit the [SAFE Network Dev Hub](https://hub.safedev.org) where you will find a lot of relevant information.

## License
This SAFE Network application is licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).

## Contribute
Copyrights in the SAFE Network are retained by their contributors. No copyright assignment is required to contribute to this project.
