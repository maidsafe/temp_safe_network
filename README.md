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

1. [Download](#download)
2. [Build](#build)
3. [Using the CLI](#using-the-cli)
  - [Auth](#auth)
    - [Prerequisite: run the Authenticator](#prerequisite-run-the-authenticator)
    - [Authorise the safe CLI app](#authorise-the-safe-cli-app)
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
  - [Cat](#cat)
  - [NRS](#nrs-name-resolution-system)
    - [Create](#nrs-create)
    - [Add](#nrs-add)
    - [Remove](#nrs-remove)
  - [SAFE-URLs](#safe-urls)
  - [Update](#update)
4. [Further Help](#further-help)
5. [License](#license)

## Download

The latest version of the SAFE CLI can be downloaded from the [releases page](https://github.com/maidsafe/safe-cli/releases/latest). Once it's downloaded and unpacked, you can follow the steps in this User Guide by starting from the [Using the CLI](#using-the-cli) section below in this document.

If otherwise you prefer to build the SAFE CLI from source code, please follow the instructions in the next two section below.

## Build

In order to build this CLI from source code you need to make sure you have `rustc v1.37.0` (or higher) installed. Please take a look at this [notes about Rust installation](https://www.rust-lang.org/tools/install) if you need help with installing it. We recommend you install it with `rustup` which will install the `cargo` tool which this guide makes use of.

Once Rust and its toolchain are installed, run the following commands to clone this repository and build the `safe-cli` crate (the build process may take several minutes the first time you run it on this crate):
```shell
$ git clone https://github.com/maidsafe/safe-cli.git
$ cd safe-cli
$ cargo build
```

### Using the Mock or Non-Mock SAFE Network

By default, the `safe-cli` is built with [Non-Mock libraries](https://github.com/maidsafe/safe_client_libs/wiki/Mock-vs.-non-mock). If you are intending to use it with the `Mock` network you'll need to specify the `mock-network` feature in every command you run with `cargo`, e.g. to build it for the `Mock` network you can run:
```
$ cargo build --features mock-network
```

Keep in mind that when running the safe-cli with `cargo run`, please also make sure to set the `mock-network` feature if you want to use the `Mock` network, e.g. with the following command the `safe-cli` will try to create a `SafeKey` with test-coins on the `Mock` network:
```
$ cargo run --features mock-network -- keys create --test-coins
```

## Using the CLI

Right now the CLI is under active development. Here we're listing commands ready to be tested (against mock).

The base command, if built is `$ safe`, or all commands can be run via `$ cargo run -- <command>`.

Various global flags are available (those commented out are not yet implemented):

```
--dry-run                  Dry run of command. No data will be written. No coins spent.
-h, --help                 Prints help information
--json                     Sets JSON as output serialisation format (alias of '--output json')
# --root                   The account's Root Container address
-V, --version              Prints version information
# -v, --verbose            Increase output verbosity. (More logs!)
-o, --output <output_fmt>  Output data serialisation. Currently only supported 'json'
# -q, --query <query>      Enable to query the output via SPARQL eg.
--xorurl <xorurl_base>     Base encoding to be used for XOR-URLs generated. Currently supported: base32z
                           (default), base32 and base64
```

#### `--help`

All commands have a `--help` function which lists args, options and subcommands.

### Auth

The CLI is just another client SAFE application, therefore it needs to be authorised by the user to gain access to the SAFE Network on behalf of the user. The `auth` command allows us to obtain such authorisation from the account owner (the user) via the SAFE Authenticator.

This command simply sends an authorisation request to the Authenticator available, e.g. the `safe_auth` CLI daemon (see further bellow for explanation of how to run it), and it then stores the authorisation response (credentials) in `<user's home directory>/.safe/credentials` file. Any subsequent CLI command will read the `~/.safe/credentials` file, to obtain the credentials and connect to the network for the corresponding operation.

#### Prerequisite: run the Authenticator

You need the [SAFE Authenticator CLI](https://github.com/maidsafe/safe-authenticator-cli) running locally and exposing its WebService interface for authorising applications, and also be logged in to a SAFE account created on the mock network (i.e. `MockVault` file), making sure the port number you set is `41805`, and enabling the `mock-network` feature.

Please open a second/separate terminal console, and follow the instructions found in the [SAFE Authenticator guide](https://github.com/maidsafe/safe-authenticator-cli/blob/master/README.md). Once you have an account created on the `Mock` network, and have the `safe_auth` CLI running with the `--daemon` flag:
```shell
$ safe_auth --daemon 41805
Secret:
Password:
Exposing service on 127.0.0.1:41805
```

You can then continue with the next step below.

#### Authorise the safe CLI app

Now that the Authenticator is running and ready to authorise applications, we can simply invoke the `auth` command:
```shell
$ safe auth
Authorising CLI application...
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

Please go ahead and allow the authorisation by entering `y`. The Authenticator will send a response back to the `safe-cli` with the corresponding credentials it can use to connect directly with the Network. When the `safe-cli` receives the authorisation response, it will display a message like the following:
```shell
SAFE CLI app was successfully authorised
Credentials were stored in <home directory>/.safe/credentials
```

We are now ready to start using the CLI to operate with the network, via its commands and supported operations!.

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

A `Wallet` effectively contains links to `SafeKey`s which have safecoin balances attached to them, but the `Wallet` also stores the secret keys needed to spend them. `Wallet`s are stored encrypted and only accessible to the owner by default.

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
        --test-coins    Create a Key, allocate test-coins onto it, and add the SafeKey to the Wallet
    -V, --version       Prints version information

OPTIONS:
        --keyurl <keyurl>         An existing SafeKey's safe://xor-url. If this is not supplied, a new SafeKey will be
                                  automatically generated and inserted. The corresponding secret key will be prompted if
                                  not provided with '--sk'
        --name <name>             The name to give the spendable balance
    -o, --output <output_fmt>     Output data serialisation. Currently only supported 'json'
    -w, --pay-with <pay_with>     The secret key of a SafeKey for paying the operation costs
        --preload <preload>       Preload the key with a balance
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

The balance of a given `Wallet` can be queried using its XorUrl. This returns the balance of the whole `Wallet`, including the contained spendable balances, or any child/nested `Wallet`s (this is not implemented just yet).

The target `Wallet` can be passed as an argument (or it will be retrieved from `stdin`):
```shell
$ safe wallet balance safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e
Wallet at "safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e" has a total balance of 0 safecoins
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

When using a `Wallet` either as the source or destination of a transfer, it must have a _default_ spendable balance for the transfer to succeed. In the future different type of logics will be implemented for using different Wallet's balances and not just the default one.

```shell
$ safe wallet transfer <amount> --from <source Wallet URL> --to <destination Wallet/SafeKey URL>
```
E.g.:
```shell
$ safe wallet transfer 323.23 --from safe://hnyybyqbp8d4u79f9sqhcxtdczgb76iif74cdsjif1wegik9t38diuk1yny9e --to safe://hbyek1io7m6we5ges83fcn16xd51bqrrjjea4yyhu4hbu9yunyc5mucjao
Success. TX_ID: 6183829450183485238
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

### Cat

The `cat` command is probably the most straight forward command, it allows users to fetch data from the Network using a URL, and render it according to the type of data being fetched:
```shell
$ safe cat safe://<XOR-URL>
```

If the XOR-URL targets a published `FilesContainer`, the `cat` command will fetch its contents render it showing the list of files contained (linked) from it, along with the corresponding XOR-URLs for each of the linked files.

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

In order to get additional information about the native data type holding the data of a specific content, we can pass the `--info` flag to the `cat` command:
```shell
$ safe cat safe://hbyit4fq3pwk9yzcytrstcgbi68q7yr9o8j1mnrxh194m6jmjanear1j5w --info
Native data type: PublishedSeqAppendOnlyData
Type tag: 1100
XOR name: 0x346b0335f55f3dbd4d89ecb792bc76460f6dcc8627b35c429a11d940cb15a492

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

We've got some additional information about the content we are retrieving. In this case we see the location where this data is stored on the Network (this is called the XOR name), a type tag number associated with the content (1100 was set for this particular type of container), and the native SAFE Network data type where this data is being held on (PublishedSeqAppendOnlyData).

And of course this flag can be used also with other type of content like files (`ImmutableData`):
```shell
$ safe cat safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc/subfolder/subexists.md --info
Native data type: ImmutableData (published)
XOR name: 0xc343e62e9127559583a336ffd2e5f9e658b11387646725eec3dbda3d3cf55da1

Raw content of the file:
hello from a subfolder!
```

#### Retrieving older versions of content

As we've seen above, we can use `cat` command to retrieve the latest/current version of any type of content from the Network using their XOR-URL. But every change made to content that is uploaded to the Network as Published data is perpetual, and therefore a new version is generated when performing any amendments to it, keeping older versions also available forever.

We can use the `cat` command to also retrieve any version of content that was uploaded as Published data by appending a query param to the URL. E.g. given the XOR-URL of the `FilesContainer` we created in previous sections (`safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w`), which reached version 2 after a couple of amendments we made with `files sync` command, we can retrieve the very first version (version 0) by using `v=<version>` query param:
```shell
$ safe cat safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w?v=0
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
$ safe nrs create mywebsite --link safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0
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
$ safe nrs create blog.mywebsite --link safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0
New NRS Map for "safe://blog.mywebsite" created at: "safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh"
+  blog.mywebsite  safe://hnyynyie8kccparz3pcxj9uisdc4gyzcpem9dfhehhjd6hpzwf8se5w1zobnc?v=0
```

As the NRS CLI advances, you'll be able to individually add to both `blog.mywebsite`, or indeed just `mywebsite`, as well as change what the `default` resource to retrieve is for both.

#### NRS Add

Once a public name has been created with `nrs create` command, more sub names can be added to it using the `nrs add` command. This command expects the same arguments as `nrs create` command but it only requires and assumes that the public name already exists.

Let's add `profile` sub name to the `mywebsite` NRS name we created before:
```shell
$ safe nrs add profile.mywebsite --link safe://hnyynyipybem7ihnzqya3w31seezj4i6u8ckg9d7s39exz37z3nxue3cnkbnc?v=0
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
- `$ safe cat safe://hnyydyz7utb6npt9kg3aksgorfwmkphet8u8z3or4nsu8n3bj8yiep4a91bqh?v=1`
- `$ safe cat safe://mywebsite?v=1`

In both cases the NRS Map Container will be found (from above URLs) by decoding the XOR-URL or by resolving NRS public name. Once that's done, and since the content is an NRS Map, following the rules defined by NRS and the map found in it the target link will be resolved from it. In some circumstances, it may be useful to get information about the resolution of a URL, which can be obtained using the `cat` command.

We've seen before that we can provide `--info` flag to the `cat` command to obtain more information about the content being retrieved, but it's also possible to request a higher level of information by passing `-ii` or `-iii` (which are equivalent to pass `--info` twice or thrice respectively) to the `cat` command:
```shell
$ safe cat safe://mywebsite/contact/form.html -iii
Native data type: ImmutableData (published)
XOR name: 0x8fa90a8234747a9672d22d030984b94f1cd640136e8b659e23249a664eb70e71

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

Raw content of the file:
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

### Update

The CLI can update itself to the latest available version. If you run `safe update`, the application will check if a newer release is available on [GitHub](https://github.com/maidsafe/safe-cli/releases). After prompting to confirm if you want to take the latest version, it will be downloaded and the binary will be updated.

## Further Help

You can discuss development-related topics on the [SAFE Dev Forum](https://forum.safedev.org/).

If you are just starting to develop an application for the SAFE Network, it's very advisable to visit the [SAFE Network Dev Hub](https://hub.safedev.org) where you will find a lot of relevant information.

If you find any issues, or have ideas for improvements and/or new features for this application and the project, please raise them by [creating a new issue in this repository](https://github.com/maidsafe/safe-cli/issues).

## License
This SAFE Network application is licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).

## Contribute
Copyrights in the SAFE Network are retained by their contributors. No copyright assignment is required to contribute to this project.
