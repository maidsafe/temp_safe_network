# safe_client

[![](https://img.shields.io/badge/Project%20SAFE-Approved-green.svg)](http://maidsafe.net/applications) [![](https://img.shields.io/badge/License-GPL3-green.svg)](https://github.com/maidsafe/crust/blob/master/COPYING)


**Primary Maintainer:**     Spandan Sharma (spandan.sharma@maidsafe.net)

**Secondary Maintainer:**   Krishna Kumar (krishna.kumar@maidsafe.net)

|Crate|Linux/OS X|Windows|Coverage|Issues|
|:---:|:--------:|:-----:|:------:|:----:|
|[![](http://meritbadge.herokuapp.com/safe_client)](https://crates.io/crates/safe_client)|[![Build Status](https://travis-ci.org/maidsafe/safe_client.svg?branch=master)](https://travis-ci.org/maidsafe/safe_client)|[![Build status](https://ci.appveyor.com/api/projects/status/pqjc5v2ww7k0qx88/branch/master?svg=true)](https://ci.appveyor.com/project/MaidSafe-QA/safe-client/branch/master)|[![Coverage Status](https://coveralls.io/repos/maidsafe/safe_client/badge.svg?branch=master)](https://coveralls.io/r/maidsafe/safe_client?branch=master)|[![Stories in Ready](https://badge.waffle.io/maidsafe/safe_client.png?label=ready&title=Ready)](https://waffle.io/maidsafe/safe_client)|

| [API Documentation - master branch](http://maidsafe.net/safe_client/master) | [SAFE Network System Documentation](http://systemdocs.maidsafe.net) | [MaidSafe website](http://maidsafe.net) | [Safe Community site](https://forum.safenetwork.io) |
|:------:|:-------:|:-------:|:-------:|

###Pre-requisite:
libsodium is a native dependency for [sodiumxoide](https://github.com/dnaq/sodiumoxide). Thus, install sodium by following the instructions [here](http://doc.libsodium.org/installation/index.html).

For windows:

- Download [prebuilt libsodium library](https://download.libsodium.org/libsodium/releases/libsodium-1.0.2-mingw.tar.gz)
- Extract `libsodium.a` for x86/x64 from the corresponding folder in the archive to your local filesystem
- Add this local path to `%PATH%`. (`PATH=%PATH%;<path to extracted libsodium.a dir>`)

###Build Instructions:
`safe_client` can interface conditionally against either the routing crate or a mock used for local testing.

To use it with the Mock:
```
cargo build --features "use-mock-routing"
cargo test --features "use-mock-routing"
```

To interface it with actual routing (default):
```
cargo build
cargo test
```

## TODO

### [0.1.4]
- [X] [MAID-1219](https://maidsafe.atlassian.net/browse/MAID-1219) Implement Private and Public types
- [X] [MAID-1249](https://maidsafe.atlassian.net/browse/MAID-1249) Implement Unified Structured Datatype
    - [X] [MAID-1252](https://maidsafe.atlassian.net/browse/MAID-1252) Mock Unified StructuredData and ImmutableData
    - [X] [MAID-1253](https://maidsafe.atlassian.net/browse/MAID-1253) Update Mock Routing to support Mock Unified SturcturedData and ImmutableData
    - [X] [MAID-1222](https://maidsafe.atlassian.net/browse/MAID-1222) Compute size of Structured Data
    - [X] [MAID-1223](https://maidsafe.atlassian.net/browse/MAID-1223) Implement a handler for Storing UnVersioned Structured Data
    - [X] [MAID-1224](https://maidsafe.atlassian.net/browse/MAID-1224) Implement a handler for Retrieving Content of UnVersioned Structured Data
    - [X] [MAID-1225](https://maidsafe.atlassian.net/browse/MAID-1225) Write Test Cases for UnVersioned Structured Data handler
    - [X] [MAID-1230](https://maidsafe.atlassian.net/browse/MAID-1230) Implement a handler for Storing Versioned Structured Data
    - [X] [MAID-1231](https://maidsafe.atlassian.net/browse/MAID-1231) Create MaidSafe Specific configuration directory
    - [X] [MAID-1232](https://maidsafe.atlassian.net/browse/MAID-1232) Write Test Cases for Versioned Structured Data handler
    - [X] [MAID-1226](https://maidsafe.atlassian.net/browse/MAID-1226) Implement Session Packet as UnVersioned Structure DataType
    - [X] [MAID-1227](https://maidsafe.atlassian.net/browse/MAID-1227) Update the test cases in Client API
    - [X] [MAID-1228](https://maidsafe.atlassian.net/browse/MAID-1228) Update the test cases in mock routing framework
    - [X] [MAID-1234](https://maidsafe.atlassian.net/browse/MAID-1234) Update Hybrid Encrypt and Decrypt
