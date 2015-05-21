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
    - [ ] [MAID-1080](https://maidsafe.atlassian.net/browse/MAID-1080) Define API traits
        - [ ] Define Metadata trait (common for Container and Blob)
        - [ ] Define Container trait
        - [ ] Define Blob trait
        - [ ] Define Storage trait
    - [ ] Implement NFS types
        - [ ] [MAID-1081](https://maidsafe.atlassian.net/browse/MAID-1081) Implement NFS_Metadata
            - [ ] Implement Metatdata trait
            - [ ] Implement serialisation
            - [ ] Unit test cases
        - [ ] [MAID-1082](https://maidsafe.atlassian.net/browse/MAID-1082) Implement NFS_Blob
            - [ ] Implement Blob trait
            - [ ] Implement serialisation
            - [ ] Unit test cases
        - [ ] Implement NFS_Container
            - [ ] [MAID-1083](https://maidsafe.atlassian.net/browse/MAID-1083) Implement Container trait
                - [ ] [MAID-1086](https://maidsafe.atlassian.net/browse/MAID-1086) Create Container
                - [ ] [MAID-1084](https://maidsafe.atlassian.net/browse/MAID-1084) List Containers
                - [ ] [MAID-1085](https://maidsafe.atlassian.net/browse/MAID-1085) Get Container
                - [ ] [MAID-1087](https://maidsafe.atlassian.net/browse/MAID-1087) Delete Container
                - [ ] [MAID-1088](https://maidsafe.atlassian.net/browse/MAID-1088) Update / Get Container Metadata
                - [ ] [MAID-1090](https://maidsafe.atlassian.net/browse/MAID-1090) Create Blob
                - [ ] [MAID-1089](https://maidsafe.atlassian.net/browse/MAID-1089) List Blobs
                - [ ] [MAID-1091](https://maidsafe.atlassian.net/browse/MAID-1091) Get Blob
                - [ ] [MAID-1092](https://maidsafe.atlassian.net/browse/MAID-1092) Update Blob Content
                - [ ] [MAID-1093](https://maidsafe.atlassian.net/browse/MAID-1093) Get Blob Content
                - [ ] [MAID-1094](https://maidsafe.atlassian.net/browse/MAID-1094) List Blob Version
                - [ ] [MAID-1095](https://maidsafe.atlassian.net/browse/MAID-1095) Delete Blob
                - [ ] [MAID-1096](https://maidsafe.atlassian.net/browse/MAID-1096) Copy Blob
                - [ ] [MAID-1097](https://maidsafe.atlassian.net/browse/MAID-1097) Update / Get Blob Metadata
            - [ ] [MAID-1098](https://maidsafe.atlassian.net/browse/MAID-1098) Implement serialisation
            - [ ] [MAID-1099](https://maidsafe.atlassian.net/browse/MAID-1099) Unit test cases
    - [ ] [MAID-1100](https://maidsafe.atlassian.net/browse/MAID-1100) Implement NFS_Storage API
    - [ ] [MAID-1101](https://maidsafe.atlassian.net/browse/MAID-1101) Implement Version Cache (cache key,(blob/container) info to reduce network traffic)
- [ ] Create Example:
    - [ ] [MAID-1102](https://maidsafe.atlassian.net/browse/MAID-1102) Self authentication Example
    - [ ] [MAID-1103](https://maidsafe.atlassian.net/browse/MAID-1103) Example to demonstrate Storage API
    - [ ] [MAID-1104](https://maidsafe.atlassian.net/browse/MAID-1104) Validate above example against Local network / droplet    
