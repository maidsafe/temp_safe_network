# Changelog

All notable changes to this project will be documented in this file.

### [0.3.0] (4-09-2019)

### Bug Fixes

    **files-sync:** when sync-ing a FilesContainer using an NRS name it was not correctly realising the latest version, closes #230
    **wallet:** change wallet transfer args from being positional to --from and --to
    **ci:** remove dir structure from zips

### Features

    **transfers:** allow to pass a --tx-id to the keys/wallet transfer cmds to specify a TX ID
    **user-guide:** add details about the safe keys transfer command
    **safekeys:** implementation of a safe keys transfer cmd
    **SafeKey:** cat cmd to show information when targeting a SafeKey
    **ci:** produce tar.gz assets
    **ci:** add the community contributed safe_completion.sh as a release asset and provide some instructions in the release description for setting it up
    **ci:** distribute zips
    **ci:** sha-256 checksums in release description
    **ci:** perform strip correctly


### [0.2.2] (29-08-2019)

### Bug Fixes

    **wallet:** add test and check in fake-scl for scenario when transferring 0 amount
    **wallet:** update default when set in wallet insert cmd, plus add details to User Guide about fetching Wallets and subfolders from FilesContainers
    **lib:** use the client instance's transfer_coin instead of the client independent wallet_transfer_coins API
    **wallet:** make use of the --sk when provided without a --keyurl in a wallet create cmd
    **lib:** catch the correct error for insufficient balance from SCL, plus cosmetic improvement to CLI output when generating a key pair

### Features

    **wallet:** support for fetching the content of a Wallet and listing it with cat cmd
    **fetch:** support for fetching a FilesContainer with a subfolder path
    **cli:** display version in the xorurl for files sync feedback information
    **lib:** handle access denied error from wallet transfer API


### [0.1.0] (22-08-2019)

### Features

    **auth:** support to send/receive authorisation requests to/from safe_auth CLI
    **safekeys:** support for creating SafeKeys and checking their coins balance
    **keypair:** utilities to generate BLS key pair
    **wallet:** commands to create wallets, check total balance, transfer coins between them, and insert SafeKeys to them
    **files:** upload entire folders and files onto the network, as well as sync-ing local changes with uploaded files/folders
    **nrs:** create and update NRS (Name Resolution System) names/subname, to link them to any type of content
    **cat:** allow to fetch any type of content, fetching also additional information and metadata about their native data type and NRS Resolution
    **cat:** support for fetching specific versions of published data from the Network
