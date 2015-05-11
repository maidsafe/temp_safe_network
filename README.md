# maidsafe_client

|Crate|Travis|Appveyor|Coverage|
|:-------:|:-------:|:------:|:------:|
|[![](http://meritbadge.herokuapp.com/maidsafe_client)](https://crates.io/crates/maidsafe_client)|[![Build Status](https://travis-ci.org/maidsafe/maidsafe_client.svg?branch=master)](https://travis-ci.org/maidsafe/maidsafe_client)|[![Build status](https://ci.appveyor.com/api/projects/status/1rnsp7l44y0nvbmt/branch/master?svg=true)](https://ci.appveyor.com/project/dirvine/maidsafe-client-r8y3h/branch/master)|[![Coverage Status](https://coveralls.io/repos/maidsafe/maidsafe_client/badge.svg?branch=master)](https://coveralls.io/r/maidsafe/maidsafe_client?branch=master)|

| [ API Documentation](http://maidsafe.github.io/maidsafe_client/)| [MaidSafe System Documention](http://systemdocs.maidsafe.net/) | [MaidSafe web site](http://www.maidsafe.net) | [Safe Community site](https://forum.safenetwork.io) |

#Todo
- [ ] Implement client Interface
  - [ ] Implement Basic Read (get):
  - [ ] Implement Basic Write (Put):
- [ ] Implement Account:
  - [x] Serialisation
  - [x] Encryption
  - [ ] Creation
  - [ ] Retrieval
- [ ] Example:
  - [ ] Self Authentication Example
  - [ ] Validate above example against Local network / droplet
- [ ] API Version 0.0.8
- [ ] Password Change
- [ ] Implement Modify (Post) for mutable data
- [ ] Implement Storage API (think about all of this as one unit when designing!)
    - [ ] Implement Metadata (for Container and Blob)
    - [ ] Implement Directory (Container):
      - [ ] Creation
      - [ ] Sub-Directory (Container) Creation
      - [ ] Sub-Directory (Container) Removal
      - [ ] Directory (Container) Listing:
        - [ ] Sub-Directories (Containers)
        - [ ] Files (Blobs)
      - [ ] File (Blob) History at a specified key
      - [ ] Implement Version Cache (cache key,(blob/container) info to reduce network traffic)
    - [ ] Implement File (Blob) :
      - [ ] File (Blob) Creation
      - [ ] File (Blob) Modification
      - [ ] File (Blob) Removal
      - [ ] File (Blob) Copying
- [ ] API Version 0.1.0
