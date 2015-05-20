# maidsafe_client

**Primary Maintainer:**     Spandan Sharma (spandan.sharma@maidsafe.net)

**Secondary Maintainer:**   Krishna Kumar (krishna.kumar@maidsafe.net)

|Crate|Travis|Windows|OSX|Coverage|
|:------:|:-------:|:-------:|:-------:|:-------:|
|[![](http://meritbadge.herokuapp.com/maidsafe_client)](https://crates.io/crates/maidsafe_client)|[![Build Status](https://travis-ci.org/maidsafe/maidsafe_client.svg?branch=master)](https://travis-ci.org/maidsafe/maidsafe_client)|[![Build Status](http://ci.maidsafe.net:8080/buildStatus/icon?job=maidsafe_client_win64_status_badge)](http://ci.maidsafe.net:8080/job/maidsafe_client_win64_status_badge/)|[![Build Status](http://ci.maidsafe.net:8080/buildStatus/icon?job=maidsafe_client_osx_status_badge)](http://ci.maidsafe.net:8080/job/maidsafe_client_osx_status_badge/)|[![Coverage Status](https://coveralls.io/repos/maidsafe/maidsafe_client/badge.svg?branch=master)](https://coveralls.io/r/maidsafe/maidsafe_client?branch=master)|

| [ API Documentation](http://maidsafe.github.io/maidsafe_client/)| [MaidSafe System Documention](http://systemdocs.maidsafe.net/) | [MaidSafe web site](http://www.maidsafe.net) | [Safe Community site](https://forum.safenetwork.io) |

#Todo
- [ ] Example:
  - [ ] Self Authentication Example
  - [ ] Validate above example against Local network / droplet
- [ ] API Version 0.0.8
- [ ] Implement Storage API (think about all of this as one unit when designing!)
    - [ ] Implement Routing Client Interface
        - [ ] Put
        - [ ] Get
        - [ ] Post
    - [ ] Define API traits
        - [ ] Define Metadata API trait (common for Container and Blob)
        - [ ] Define Container API trait
        - [ ] Define Blob API trait
        - [ ] Define Storage API trait
    - [ ] Implement NFS types
        - [ ] Implement NFS_Metadata
            - [ ] Implement Metatdata API trait
            - [ ] Implement serialisation
            - [ ] Write test cases
        - [ ] Implement NFS_Container
            - [ ] Implement Container API trait
            - [ ] Implement serialisation
            - [ ] Write test cases
        - [ ] Implement NFS_Blob
            - [ ] Implement Blob API trait
            - [ ] Implement serialisation
            - [ ] Write test cases
    - [ ] Implement NFS_Storage API
        - [ ] Implement Storage API trait
        - [ ] Write test cases
    - [ ] Implement Version Cache (cache key,(blob/container) info to reduce network traffic)
- [ ] Create Example:
    - [ ] Example to demonstrate Container API usage
    - [ ] Example to demonstrate Blob API usage




- [ ] Password Change
- [ ] Implement Modify (Post) for mutable data
- [ ] API Version 0.1.0
