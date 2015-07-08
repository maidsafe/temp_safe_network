# maidsafe_client

[![](https://img.shields.io/badge/Project%20SAFE-Approved-green.svg)](http://maidsafe.net/applications) [![](https://img.shields.io/badge/License-GPL3-green.svg)](https://github.com/maidsafe/crust/blob/master/COPYING)


**Primary Maintainer:**     Spandan Sharma (spandan.sharma@maidsafe.net)

**Secondary Maintainer:**   Krishna Kumar (krishna.kumar@maidsafe.net)

|Crate|Linux|Windows|OSX|Coverage|
|:------:|:-------:|:-------:|:-------:|:-------:|
|[![](http://meritbadge.herokuapp.com/maidsafe_client)](https://crates.io/crates/maidsafe_client)|[![Build Status](https://travis-ci.org/maidsafe/maidsafe_client.svg?branch=master)](https://travis-ci.org/maidsafe/maidsafe_client)|[![Build Status](http://ci.maidsafe.net:8080/buildStatus/icon?job=maidsafe_client_win64_status_badge)](http://ci.maidsafe.net:8080/job/maidsafe_client_win64_status_badge/)|[![Build Status](http://ci.maidsafe.net:8080/buildStatus/icon?job=maidsafe_client_osx_status_badge)](http://ci.maidsafe.net:8080/job/maidsafe_client_osx_status_badge/)|[![Coverage Status](https://coveralls.io/repos/maidsafe/maidsafe_client/badge.svg?branch=master)](https://coveralls.io/r/maidsafe/maidsafe_client?branch=master)|

| [API Documentation - master branch](http://maidsafe.net/maidsafe_client/master) | [SAFE Network System Documentation](http://systemdocs.maidsafe.net) | [MaidSafe website](http://maidsafe.net) | [Safe Community site](https://forum.safenetwork.io) |
|:------:|:-------:|:-------:|:-------:|

###Pre-requisite:
libsodium is a native dependency for [sodiumxoide](https://github.com/dnaq/sodiumoxide). Thus, install sodium by following the instructions [here](http://doc.libsodium.org/installation/index.html).

For windows, download and use the [prebuilt mingw library](https://download.libsodium.org/libsodium/releases/libsodium-1.0.2-mingw.tar.gz).
Extract and place the libsodium.a file in "bin\x86_64-pc-windows-gnu" for 64bit System, or "bin\i686-pc-windows-gnu" for a 32bit system.

###Build Instructions:
Maidsafe-Client interfaces conditionally with either the actual routing crate or the Mock used for efficient local testing.

To use it with the Mock (default) do:
```
cargo build
cargo test
etc
```

To interface it with actual routing, do:
```
cargo build --features "USE_ACTUAL_ROUTING"
cargo test --features "USE_ACTUAL_ROUTING"
etc
```

##TODO (rust_3 sprint)
### [0.1.2]
- [ ] [MAID-1209](https://maidsafe.atlassian.net/browse/MAID-1209) Remove NFS API

### [0.1.3]
- [ ] [MAID-1219](https://maidsafe.atlassian.net/browse/MAID-1219) Implement Private and Public types

### [0.1.4]
- [ ] [MAID-1248](https://maidsafe.atlassian.net/browse/MAID-1248) Name the spawned rust threads
- [ ] [MAID-1218](https://maidsafe.atlassian.net/browse/MAID-1218) No restarting of routing-client

### [0.1.5]
- [ ] [MAID-1249](https://maidsafe.atlassian.net/browse/MAID-1249) Implement Unified Structured Datatype
    - [ ] [MAID-1252](https://maidsafe.atlassian.net/browse/MAID-1252) Mock Unified StructuredData and ImmutableData
    - [ ] [MAID-1253](https://maidsafe.atlassian.net/browse/MAID-1253) Update Mock Routing to support Mock Unified SturcturedData and ImmutableData
    - [ ] [MAID-1220](https://maidsafe.atlassian.net/browse/MAID-1220) Update the SessionPacket struct
    - [ ] [MAID-1221](https://maidsafe.atlassian.net/browse/MAID-1221) Update client API to get and set configuration directory ids
    - [ ] [MAID-1222](https://maidsafe.atlassian.net/browse/MAID-1222) Compute size of Structured Data
    - [ ] [MAID-1223](https://maidsafe.atlassian.net/browse/MAID-1223) Implement a handler for Storing UnVersioned Structured Data
    - [ ] [MAID-1224](https://maidsafe.atlassian.net/browse/MAID-1224) Implement a handler for Retrieving Content of UnVersioned Structured Data
    - [ ] [MAID-1225](https://maidsafe.atlassian.net/browse/MAID-1225) Write Test Cases for UnVersioned Structured Data handler
    - [ ] [MAID-1226](https://maidsafe.atlassian.net/browse/MAID-1226) Implement Session Packet as UnVersioned Structure DataType
    - [ ] [MAID-1227](https://maidsafe.atlassian.net/browse/MAID-1227) Update the test cases in Client API
    - [ ] [MAID-1228](https://maidsafe.atlassian.net/browse/MAID-1228) Update the test cases in mock routing framework
