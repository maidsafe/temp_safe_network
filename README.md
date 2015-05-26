# maidsafe_client

**Primary Maintainer:**     Spandan Sharma (spandan.sharma@maidsafe.net)

**Secondary Maintainer:**   Krishna Kumar (krishna.kumar@maidsafe.net)

|Crate|Travis|Windows|OSX|Coverage|
|:------:|:-------:|:-------:|:-------:|:-------:|
|[![](http://meritbadge.herokuapp.com/maidsafe_client)](https://crates.io/crates/maidsafe_client)|[![Build Status](https://travis-ci.org/maidsafe/maidsafe_client.svg?branch=master)](https://travis-ci.org/maidsafe/maidsafe_client)|[![Build Status](http://ci.maidsafe.net:8080/buildStatus/icon?job=maidsafe_client_win64_status_badge)](http://ci.maidsafe.net:8080/job/maidsafe_client_win64_status_badge/)|[![Build Status](http://ci.maidsafe.net:8080/buildStatus/icon?job=maidsafe_client_osx_status_badge)](http://ci.maidsafe.net:8080/job/maidsafe_client_osx_status_badge/)|[![Coverage Status](https://coveralls.io/repos/maidsafe/maidsafe_client/badge.svg?branch=master)](https://coveralls.io/r/maidsafe/maidsafe_client?branch=master)|

| [ API Documentation](http://maidsafe.github.io/maidsafe_client/)| [MaidSafe System Documention](http://systemdocs.maidsafe.net/) | [MaidSafe web site](http://www.maidsafe.net) | [Safe Community site](https://forum.safenetwork.io) |

#Todo
- [ ] [MAID-1077](https://maidsafe.atlassian.net/browse/MAID-1077) Account Creation
    - [ ] [MAID-1078](https://maidsafe.atlassian.net/browse/MAID-1078) Register
    - [ ] [MAID-1079](https://maidsafe.atlassian.net/browse/MAID-1079) Login
- [ ] Implement Storage API
    - [X] [MAID-1080](https://maidsafe.atlassian.net/browse/MAID-1080) Implement types
        - [X] Implement MetaData, File and DirectoryListing types
        - [X] Implement wrapper traits
    - [ ] Implement Helpers
        - [ ] [MAID-1081](https://maidsafe.atlassian.net/browse/MAID-1081) Directory Helper
            - [ ] Save DirectoryListing
            - [ ] Get Directory
            - [ ] Get Directory Versions
        - [ ] File Helper
            - [ ] [MAID-1082](https://maidsafe.atlassian.net/browse/MAID-1082) Create File, update file and Metatdata
            - [ ] [MAID-1083](https://maidsafe.atlassian.net/browse/MAID-1083) Get Versions
            - [ ] [MAID-1084](https://maidsafe.atlassian.net/browse/MAID-1084) Read File
        - [ ] [MAID-1085](https://maidsafe.atlassian.net/browse/MAID-1085) Unit test cases for Directory and File Helpers
    - [ ] Implement REST DataTypes
        - [ ] [MAID-1086](https://maidsafe.atlassian.net/browse/MAID-1086) Container & Blob types
            - [ ] Implement Blob and Container types
            - [ ] Implement FileWrapper trait for Blob
            - [ ] Implement DirectoryListingWrapper trait for Container
            - [ ] Unit test cases
        - [ ] REST API methods in Container
            - [ ] [MAID-1087](https://maidsafe.atlassian.net/browse/MAID-1087) Create Container & Get Container
            - [ ] [MAID-1088](https://maidsafe.atlassian.net/browse/MAID-1088) List Containers, Update / Get Container Metadata
            - [ ] [MAID-1089](https://maidsafe.atlassian.net/browse/MAID-1089) Delete Container
            - [ ] [MAID-1090](https://maidsafe.atlassian.net/browse/MAID-1090) Create Blob
            - [ ] [MAID-1091](https://maidsafe.atlassian.net/browse/MAID-1091) List Blobs
            - [ ] [MAID-1092](https://maidsafe.atlassian.net/browse/MAID-1092) Get Blob
            - [ ] [MAID-1098](https://maidsafe.atlassian.net/browse/MAID-1098) Update Blob Content
            - [ ] [MAID-1093](https://maidsafe.atlassian.net/browse/MAID-1093) Get Blob Content
            - [ ] [MAID-1094](https://maidsafe.atlassian.net/browse/MAID-1094) List Blob Version
            - [ ] [MAID-1095](https://maidsafe.atlassian.net/browse/MAID-1095) Delete Blob
            - [ ] [MAID-1096](https://maidsafe.atlassian.net/browse/MAID-1096) Copy Blob
            - [ ] [MAID-1097](https://maidsafe.atlassian.net/browse/MAID-1097) Update / Get Blob Metadata
        - [ ] [MAID-1099](https://maidsafe.atlassian.net/browse/MAID-1099) Unit test cases for API
    - [ ] [MAID-1101](https://maidsafe.atlassian.net/browse/MAID-1101) Implement Version Cache (cache key,(blob/container) info to reduce network traffic)
- [ ] Create Example:
    - [ ] [MAID-1102](https://maidsafe.atlassian.net/browse/MAID-1102) Self authentication Example
    - [ ] [MAID-1103](https://maidsafe.atlassian.net/browse/MAID-1103) Example to demonstrate Storage API
    - [ ] [MAID-1104](https://maidsafe.atlassian.net/browse/MAID-1104) Validate above example against Local network / droplet
