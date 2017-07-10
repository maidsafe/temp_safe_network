# SAFE Authenticator - Change Log

## [0.1.0]
- Implement RFC 46 ([New Auth Flow](https://github.com/maidsafe/rfcs/blob/master/text/0046-new-auth-flow/0046-new-auth-flow.md))
- Allow users to create accounts and login into the SAFE Network
- Allow applications to be authenticated to use the network on behalf of the user, with an option for users to subsequently revoke the given permissions
- Introduce the concept of an access container, which allows to set fine-grained permissions for apps to access various MutableData instances on the network
- Provide a Foreign Function Interface to use the Authenticator API from other languages (JavaScript and Node.js, Java, etc.)
