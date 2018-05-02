# SAFE core

**Maintainer:** Spandan Sharma (spandan.sharma@maidsafe.net)

| [![](http://meritbadge.herokuapp.com/safe_core)](https://crates.io/crates/safe_core) | [![Documentation](https://docs.rs/safe_core/badge.svg)](https://docs.rs/safe_core) |
|:----------:|:----------:|


## Build Instructions

`safe_core` can interface conditionally against either the routing crate or a mock used for local testing.

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

Licensed under the General Public License (GPL), version 3 ([LICENSE](LICENSE) http://www.gnu.org/licenses/gpl-3.0.en.html).
