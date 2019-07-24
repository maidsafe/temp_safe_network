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
    - [Authorise the safe CLI app](#authorise-the-safe-cli-app)
  - [Keys](#keys)
    - [Create](#keys-creation)
    - [Balance](#keys-balance)
  - [Key pair](#key-pair)
  - [Wallet](#wallet)
    - [Create](#wallet-creation)
    - [Insert](#wallet-insert)
    - [Balance](#wallet-balance)
    - [Transfer](#wallet-transfer)
  - [Files](#files)
    - [Put](#files-put)
    - [Sync](#files-sync)
  - [Cat](#cat)
3. [Further Help](#further-help)
4. [License](#license)


## Build

In order to build this CLI from source code you need to make sure you have `rustc v1.35.0` (or higher) installed. Please take a look at this [notes about Rust installation](https://www.rust-lang.org/tools/install) if you need help with installing it. We recommend you install it with `rustup` which will install `cargo` tool since this guide makes use of it.

Once Rust and its toolchain are installed, run the following commands to clone this repository and build the `safe_cli` crate (the build process may take several minutes the first time you run it on this crate):
```shell
$ git clone https://github.com/maidsafe/safe-cli.git
$ cd safe-cli
$ cargo build
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
-o, --output <output_fmt>  Output data serlialisation. Currently only supported 'json'
# -q, --query <query>      Enable to query the output via SPARQL eg.
--xorurl <xorurl_base>     Base encoding to be used for XOR-URLs generated. Currently supported: base32z
                           (default), base32 and base64
```

#### `--help`

All commands have a `--help` function which will list args, options and subcommands.

### Auth

The CLI is just another client SAFE application, therefore it needs to be authorised by the user to gain access to the SAFE Network on behalf of the user. The CLI `auth` command allows us to obtain such authorisation from the account owner (the user) via the SAFE Authenticator.

This command simply sends an authorisation request to the Authenticator available, e.g. the safe_auth CLI daemon (see further bellow for explanation of how to run it), and it then stores the authorisation response (credentials) in `<user's home directory>/.safe/credentials` file. Any subsequent CLI command will read the `~/.safe/credentials` file, to obtain the credentials and connect to the network for the corresponding operation.

#### Prerequisite: run the Authenticator

You need the [SAFE Authenticator CLI](https://github.com/maidsafe/safe-authenticator-cli) running locally and exposing its WebService interface for authorising applications, and also be logged in to a SAFE account created on the mock network (i.e. `MockVault` file), making sure the port number you set is `41805`, and enabling the `mock-network` feature.

Please open a second/separate terminal console to execute the following commands (again, please make sure you have `rustc v1.35.0` or higher):
```shell
$ git clone https://github.com/maidsafe/safe-authenticator-cli.git
$ cd safe-authenticator-cli
$ cargo run --features mock-network -- --daemon 41805
```

#### Authorise the safe CLI app

Now that the Authenticator is running and ready to authorise applications, we can simply invoke the `auth` command:
```shell
$ safe auth
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
$ safe keys create <source>
```

But we can also create a `Key` with test-coins since we are using the mock network:
```shell
$ safe keys create --test-coins --preload 15.342
New Key created at: "safe://bbkulcbnrmdzhdkrfb6zbbf7fisbdn7ggztdvgcxueyq2iys272koaplks"
Key pair generated:
pk="b62c1e4e3544a1f64212fca89046df98d998ea615e84c4348c4b5fd29c07ad52a970539df819e31990c1edf09b882e61"
sk="c4cc596d7321a3054d397beff82fe64f49c3896a07a349d31f29574ac9f56965"
```

Once we have some `Key`'s with some test-coins we can use them as the `source` for the creation of new `Key`'s, thus if we use the `Key` we just created with test-coins we can create a second `Key`:
```shell
$ safe keys create --preload 8.15 safe://bbkulcbnrmdzhdkrfb6zbbf7fisbdn7ggztdvgcxueyq2iys272koaplks
Enter secret key corresponding to public key at XOR-URL "safe://bbkulcbnrmdzhdkrfb6zbbf7fisbdn7ggztdvgcxueyq2iys272koaplks":
New Key created at: "safe://bbkulcbf2uuqwawvuonevraqa4ieu375qqrdpwvzi356edwkdjhwgd4dum"
Key pair generated:
pk="9754a42c0b568e692b10401c4129bff61088df6ae51bef883b28693d8c3e0e8ce23054e236bd64edc45791549ef60ce1"
sk="2f211ad4606c716c2c2965e8ea2bd76a63bfc5a5936b792cda448ddea70a031c"
```

Other optional args that can be used with `keys create` sub-command are:
```
--pk <pk>            Don't generate a key pair and just use the provided public key
--preload <preload>  Preload the Key with a coin balance
```

#### Key's Balance

We can retrieve a given `Key`'s balance simply using its secret key, which we can pass to `keys balance` subcommand with `--sk <secret key>` argument, or we can enter it when the CLI prompts us.

We can optionally also pass the `Key`'s XorUrl to have the CLI to verify they correspond to each other, i.e. if the `Key`'s XorUrl is provided, the CLI will check if it corresponds to the public key derived from the passed secret key, and throw an error in it doesn't.

The target `Key`'s secret key can be passed as an argument (or it will be retrieved from `stdin`), let's check the balance of the `Key` we created in previous section:
```bash
$ safe keys balance
Enter secret key corresponding to the Key to query the balance from: c4cc596d7321a3054d397beff82fe64f49c3896a07a349d31f29574ac9f56965
Key's current balance: 15.342
```

### Key-pair

There are some scenarios that being able to generate a sign/encryption key-pair, without creating and/or storing a `Key` on the network, is required.

As an example, if we want to have a friend to create a `Key` for us, and preload it with some safecoins, we can generate a key-pair, and share with our friend only the public key so they can generate the `Key` to be owned by it (this is where we can use the `--pk` argument on the `keys create` sub-command).

Thus, let's see how this use case would work. First we create a key-pair:
```shell
$ safe keypair
Key pair generated:
pk="b2371df48684dc9456988f45b56d7640df63895fea3d7cee45c79b26ba268d259b864330b83fa28669ab910a1725b833"
sk="62e323615235122f7e20c7f05ddf56c5e5684853d21f65fca686b0bfb2ed851a"
```

We now take note of both the public key, and the secret key. Now, we only share the public key with our friend, who can use it to generate a `Key` to be owned by it and preload it with some test-coins:
```shell
$ safe keys create --test-coins --preload 64.24 --pk b2371df48684dc9456988f45b56d7640df63895fea3d7cee45c79b26ba268d259b864330b83fa28669ab910a1725b833
New Key created at: "safe://hodby8y3qgina9nqzxmsoi8ytjfh6gwnia7hdupo49ibt8yy3ytgdq"
```

Finally, our friend gives us the XOR-URL of the `Key` they have created for us, and we can now use the `Key` for any other operation, we own the balance it contains since we have the secret key associated to it.

### Wallet

A `Wallet` is a specific type of Container on the network, holding a set of spendable safecoin balances.

A `Wallet` effectively contains links to `Key`'s which have safecoin balances attached to them, but the `Wallet` also stores the secret keys needed to spend them. `Wallet`'s are stored encrypted and only accessible to the owner by default.

There are several sub-commands that can be used to manage the `Wallet`'s with the `safe wallet` command (those commented out are not yet implemented):

```
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

```shell
USAGE:
    safe wallet create [FLAGS] [OPTIONS] [ARGS]

FLAGS:
        --no-balance    If true, do not create a spendable balance
        --test-coins    Create a Key, allocate test-coins onto it, and add the Key to the Wallet

OPTIONS:
        --keyurl <keyurl>         An existing Key's safe://xor-url. If this is not supplied, a new Key will be
                                  automatically generated and inserted. The corresponding secret key will be prompted if
                                  not provided with '--sk'.
        --name <name>             The name to give the spendable balance
    -o, --output <output_fmt>     Output data serlialisation. Currently only supported 'json'
        --preload <preload>       Preload the key with a balance
        --sk <secret>             Pass the secret key to make the balance spendable, it will be prompted if not provided
        --xorurl <xorurl_base>    Base encoding to be used for XOR-URLs generated. Currently supported: base32z
                                  (default), base32 and base64

ARGS:
    <source>    The secret key of a Key for paying the storage costs.
```

Right now, only a secret key (of a `Key` with coins) can be used as the `<source>` to pay for the costs, but in the future a `Wallet` will be also allowed as the `<source>`.

For example, if we use the secret key we obtained when creating a `Key` in our example in previous section to pay for the costs, we can create a `Wallet` with a new spendable balance by simply running:

```shell
$ safe wallet create 62e323615235122f7e20c7f05ddf56c5e5684853d21f65fca686b0bfb2ed851a
New Key created at: "safe://hodqmc6ht5ezpprkh1cbw54n3mjyckcpm95qmygon897ft5dq8oxpc"
Key pair generated:
pk=a7086bbc7f7dad7db400a99ace99fd46abfef652d04788dbc3b9d1b6e45dec08806ee9cd318ee914577fae6a58009cae
sk=65f7cd252d3b66456239611f293325f94f4f89e1eda0b3b1d5bc41743999003c
Wallet created at: "safe://hbymipwqmm3ityq3ox5xuu6j7mjm8aw11nhnjnzpy1dib4cgmr63rc1jao"
```

#### Wallet Balance

The balance of a given `Wallet` can be queried using its XorUrl. This returns the balance of the whole `Wallet`, including the contained spendable balances, or any child wallets (this is not implemented just yet).

The target `Wallet` can be passed as an argument (or it will be retrieved from `stdin`):
```shell
$ safe wallet balance safe://hbymipwqmm3ityq3ox5xuu6j7mjm8aw11nhnjnzpy1dib4cgmr63rc1jao
Wallet at "safe://hbymipwqmm3ityq3ox5xuu6j7mjm8aw11nhnjnzpy1dib4cgmr63rc1jao" has a total balance of 0 safecoins
```

#### Wallet Insert

As mentioned before, a `Key` doesn't hold the secret key on the network, therefore even if it has some non-zero coin balance, it cannot be spent. This is where the `Wallet` comes into play, holding the links to `Key`'s, and making their balances spendable by storing the corresponding secret keys.

Aside from at wallet creation, we can add _more_ keys to use as spendable balances by `insert`-ing into a `Wallet` a link to a `Key`, making it a spendable balance.

```shell
USAGE:
    safe wallet insert [FLAGS] [OPTIONS] [ARGS]

FLAGS:
        --default    Set the inserted Key as the default one in the target Wallet

OPTIONS:
        --keyurl <keyurl>         The Key's safe://xor-url to verify it matches/corresponds to the secret key provided.
                                  The corresponding secret key will be prompted if not provided with '--sk'.
        --name <name>             The name to give this spendable balance
    -o, --output <output_fmt>     Output data serlialisation. Currently only supported 'json'
        --sk <secret>             Pass the secret key to make the balance spendable, it will be prompted if not provided
        --xorurl <xorurl_base>    Base encoding to be used for XOR-URLs generated. Currently supported: base32z
                                  (default), base32 and base64

ARGS:
    <target>     The target Wallet to insert the spendable balance
    <source>     The secret key of a Key for paying the storage costs
```

- The `<target>` is the `Wallet` to insert the spendable balance to
- The `<source>` is the secret key of a `Key` paying for the costs. Right now, only a secret key (of a `Key` with coins) can be used as the `<source>` to pay for the costs, but in the future a `Wallet` will be also allowed as the `<source>`
- The `--name` is an optional nickname to give a spendable balance for easy reference
- The `--default` flag sets _this_ new spendable balance as the default for the containing `Wallet`. This can be used by wallet applications to apply some logic on how to spend and/or choose the balances for a transaction

With the above options, the user will be prompted to input the secret key associated with the public key, unless it was already provided with `--sk`. This is stored in the `Wallet`.

The `--sk` argument can also be combined with `--keyurl` to pass the `Key`'s XorUrl as part of the command line instruction itself, e.g.:

```shell
$ safe wallet insert safe://<wallet-xorurl> --keyurl safe://<key-xor-url> --name my_default_balance --default
Enter secret key corresponding to public key at safe://key-xor-url:
b493a84e3b35239cbffdf10b8ebfa49c0013a5d1b59e5ef3c000320e2d303311
Spendable balance inserted with name 'my_default_balance' in Wallet located at "safe://wallet-xorurl"
```

#### Wallet Transfer

Once a `Wallet` contains some spendable balance/s, we can transfer `<from>` a `Wallet` an `<amount>` of safecoins `<to>` another `Wallet` or `Key`. The destination `Wallet`/`Key` currently must be passed as a XorUrl in the arguments list, but reading it from `stdin` will be supported later on.

Both the `<from>` and `<to>` Wallets must have a _default_ spendable balance for the transfer to succeed. In the future different type of logics will be implemented for using different Wallet's balances and not just the default one.

```shell
$ safe wallet transfer <amount> <to> <from>
```
E.g.:
```shell
$ safe wallet transfer 323.23 safe://hbyek1io7m6we5ges83fcn16xd51bqrrjjea4yyhu4hbu9yunyc5mucjao safe://hodn6ny9jwhrnokdrgrfmn1jyksh7exctuuzh9w35bpuw5wmpp7hhp
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

The `files sync` command follows a very similar logic to the well known `rsync` command supporting a subset of the functionality provided by it. The subset of features supported will be gradually expanded with more features. Users knowing how to use `rsync` can easily start using the SAFE CLI and the SAFE Network for uploading files and folders, making it also easy to integrate existing automated systems which are currently making use of `rsync`.

As an example, let's suppose we uploaded all files and subfolders found within the `./to-upload/` local directory, recursively, using `files put` command:
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
FilesContainer synced up (version 2): "safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w"
*  ./to-upload/another.md     safe://hox6jstso13b7wzfkw1wbs3kwn9gpssudqunk6sw5yt3d6pnmaec53
+  ./to-upload/new.md         safe://hoxpdc8ywz18twkg7wboarj45hem3pq6ou6sati9i3dud68tzutw34
-  /test.md                   safe://hoxibhqth9awkjgi35sz73u35wyyscuht65m3ztrznb6thd5z8hepx
```

The `*`, `+` and `-` signs mean that the files were updated, added, and removed respectively.

Also, please note we provided the optional `--delete` flag to the command above, this forces the deletion of those files which are found at the targeted `FilesContainer` that are not found in the source location, like the case of `./to-upload/test.md` file in our example above. If we didn't provide such flag, only the modification and creation of files would have been updated on the `FilesContainer`, like the case of `./to-upload/another.md` and `./to-upload/new` files in our example above. Note that `--delete` is only allowed if the `--recursive` flag is also provided.

The `files sync` command also supports to be passed a destination path as the `files put` command, but in this case the destination path needs to be provided as part of the target XOR-URL. E.g., we can sync a `FilesContainer` using the local path and provide a specific destination path `new-files` in the target XOR-URL:
```shell
$ safe files sync ./other-folder/ safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w/new-files
FilesContainer synced up (version 3): "safe://hbyw8kkqr3tcwfqiiqh4qeaehzr1e9boiuyfw5bqqx1adyh9sawdhboj5w"
+  ./other-folder/file1.txt     safe://hoqi7papyp7c6riyxiez6y5fh5ugj4xc7syqhmex774ws4g4b1z1xq
```

The `./other-folder/file1.txt` file will be uploaded and published in the `FilesContainer` with path `/new-files/file1.txt`.

#### Cat

The `cat` command is probably the most straight forward command, it allows users to fetch data from the Network using a URL, and render it according to the type of data being fetched:
```shell
$ safe cat safe://<XOR-URL>
```

If the XOR-URL targets a published `FilesContainer`, the `cat` command will fetch the content of it and render it showing the list of files contained (linked) from it, along with the corresponding XOR-URLs for each of the linked files.

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
Files of FilesContainer (version 1) at "safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc":
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

We could also take any of the XOR-URLs of the individual files and have the `cat` command to fetch the content of the file and show it in the output, e.g. let's use the XOR-URL of the `/test.md` file to fetch its content:
```shell
$ safe cat safe://hbyyyydx1c168rwuqi6hcctwfbf1ihf9dfhr4bkmb6kzacs96uyj7bp4n6
hello tests!
```

Alternatively, we could use the XOR-URL of the `FilesContainer` and provide the path of the file we are trying to fetch, in this case the `cat` command will resolve the path and follow the corresponding link to read the file's content directly for us. E.g. we can also read the content of the `/test.md` file with the following command:
```shell
$ safe cat safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc/test.md
hello tests!
```

As seen above, the `safe cat` command can be used to fetch any type of content from the SAFE Network, at this point it only supports files (`ImmutableData`), `FilesContainer`'s and `NRS-Container`'s (see further below about NRS Containers and commands), but it will be expanded as more types are supported by the CLI and its API.

In order to get additional information about the native data type holding the data of a specific content, we can pass the `--info` flag to the `cat` command:
```shell
$ safe cat safe://hbyit4fq3pwk9yzcytrstcgbi68q7yr9o8j1mnrxh194m6jmjanear1j5w --info
Native data type: PublishedSeqAppendOnlyData
Type tag: 1100
XOR name: 0x346b0335f55f3dbd4d89ecb792bc76460f6dcc8627b35c429a11d940cb15a492

Files of FilesContainer (version 1) at "safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc":
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

And of course that can be used also with other type of content like `ImmutableData` files:
```shell
$ safe cat safe://hnyynyw4gsy3i6ixu5xkpt8smxrihq3dy65qcoau5gznnuee71ogmns1jrbnc/subfolder/subexists.md --info
Native data type: ImmutableData (published)
XOR name: 0xc343e62e9127559583a336ffd2e5f9e658b11387646725eec3dbda3d3cf55da1

Raw content of the file:
hello from a subfolder!
```

## Further Help

You can discuss development-related topics on the [SAFE Dev Forum](https://forum.safedev.org/).

If you are just starting to develop an application for the SAFE Network, it's very advisable to visit the [SAFE Network Dev Hub](https://hub.safedev.org) where you will find a lot of relevant information.

If you find any issues, or have ideas for improvements and/or new features for this application and the project, please raise them by [creating a new issue in this repository](https://github.com/maidsafe/safe-cli/issues).

## License
This SAFE Network application is licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).

## Contribute
Copyrights in the SAFE Network are retained by their contributors. No copyright assignment is required to contribute to this project.
