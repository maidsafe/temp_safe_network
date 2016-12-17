# SAFE Authenticator

**Maintainer:** Spandan Sharma (spandan.sharma@maidsafe.net)

[![Documentation](https://docs.rs/safe_authenticator/badge.svg)](https://docs.rs/safe_authenticator)

This is the crate for interfacing with `Authenticator` frontend. It contains the business logic for the `Authenticator` UI and code for building the URI for communicating with [safe_app](../safe_app).

## Build Instructions

`safe_authenticator` can interface conditionally against either the routing crate or a mock used for local testing.

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

## License

Licensed under either of

* the MaidSafe.net Commercial License, version 1.0 or later ([LICENSE](LICENSE))
* the General Public License (GPL), version 3 ([COPYING](COPYING) or http://www.gnu.org/licenses/gpl-3.0.en.html)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the MaidSafe Contributor Agreement, version 1.1 ([CONTRIBUTOR]
(CONTRIBUTOR)), shall be dual licensed as above, and you agree to be bound by the terms of the
MaidSafe Contributor Agreement, version 1.1.
