# maidsafe_client

|Travis build status|Appveyor build status (Windows)|Code Coverage|
|:-----------------:|:-------------------:|:---------------------:|
|[![Build Status](https://travis-ci.org/dirvine/maidsafe_client.svg?branch=master)](https://travis-ci.org/dirvine/maidsafe_client)|[![Build status](https://ci.appveyor.com/api/projects/status/kp7liadkt0uwm7fs?svg=true)](https://ci.appveyor.com/project/dirvine/maidsafe-client)|[![Coverage Status](https://coveralls.io/repos/dirvine/maidsafe_client/badge.svg?branch=master)](https://coveralls.io/r/dirvine/maidsafe_client?branch=master)|

[Documentation](http://dirvine.github.io/maidsafe_client/)

#Todo
- [ ] Implement client façade
  - [ ] Implement Basic Read (get):
    - [ ] for immutable chunk
      - [ ] test
    - [ ] for mutable data
      - [ ] test
  - [ ] Implement Basic Write (Put):
    - [ ] for immmutable chunk
      - [ ] test
    - [ ] for mutable data
      - [ ] test
  - [ ] Implement Modify (Post) for mutable data
    - [ ] test
- [ ] Implement Account:
  - [x] Serialisation
    - [x] test
  - [x] Encryption
    - [x] test
  - [ ] Creation
    - [ ] test
  - [ ] Retrieval
    - [ ] test
  - [ ] Password Change
    - [ ] test
- [ ] Implement Storage API (think about all of this as one unit when designing!)
    - [ ] Implement Metadata (for Container and Blob)
      - [ ] test
    - [ ] Implement Directory (Container):
      - [ ] Creation
        - [ ] test
      - [ ] Sub-Directory (Container) Creation
        - [ ] test
      - [ ] Sub-Directory (Container) Removal
        - [ ] test
      - [ ] Directory (Container) Listing:
        - [ ] Sub-Directories (Containers)
          - [ ] test
        - [ ] Files (Blobs)
          - [ ] test
      - [ ] File (Blob) History at a specified key
        - [ ] test
      - [ ] Implement Version Cache (cache key,(blob/container) info to reduce network traffic)
        - [ ] test
    - [ ] Implement File (Blob) :
      - [ ] File (Blob) Creation
        - [ ] test
      - [ ] File (Blob) Modification
        - [ ] test
      - [ ] File (Blob) Removal
        - [ ] test
      - [ ] File (Blob) Copying
        - [ ] test

