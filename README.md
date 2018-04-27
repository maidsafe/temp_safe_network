# safe_client_libs

**Maintainer:** Nikita Baksalyar (nikita.baksalyar@maidsafe.net)

|Linux/OS X|Windows|Issues|
|:--------:|:-----:|:----:|
|[![Build Status](https://travis-ci.org/maidsafe/safe_client_libs.svg?branch=dev)](https://travis-ci.org/maidsafe/safe_client_libs)|[![Build status](https://ci.appveyor.com/api/projects/status/qyvxnojplcwcey4l/branch/dev?svg=true)](https://ci.appveyor.com/project/MaidSafe-QA/safe-client-libs/branch/dev)|[![Stories in Ready](https://badge.waffle.io/maidsafe/safe_client_libs.png?label=ready&title=Ready)](https://waffle.io/maidsafe/safe_client_libs)|

| [MaidSafe website](https://maidsafe.net) | [SAFE Dev Forum](https://forum.safedev.org) | [SAFE Network Forum](https://safenetforum.org) |
|:-------:|:-------:|:-------:|

This is the project workspace. Please refer to individual members for details:

- [safe_core](safe_core/README.md)
- [ffi_utils](ffi_utils/README.md)
- [safe_authenticator](safe_authenticator/README.md)
- [safe_app](safe_app/README.md)

## Testing

When making changes, please run the appropriate tests, described below, before submitting a PR.

### Mock routing

If a change is minor or does not involve potential breakage in data or API compatibility, it is not necessary to test against the actual network. In this case, it is enough to run unit tests using mock routing and make sure that they all pass. To do this, you will have to go to the crate you wish to test (e.g. the `safe_core` directory) and execute the following:

`cargo test --release --features=use-mock-routing`

This will run all unit tests for the crate. We run tests in release mode (indicated by the `--release` flag) in order to catch rare FFI bugs that may not manifest themselves in debug mode. Debug mode is also unoptimized and can take an inordinate amount of time to run tests.

### Real network

When in doubt, perform an integration test against the real network, in addition to mock routing tests above. This will test various network operations, so please make sure you have some available account mutations. The steps:

* You will need a SAFE network account. You can register one from within the [SAFE Browser](https://github.com/maidsafe/safe_browser/releases).
* Navigate to the `tests` directory.
* Open `tests.config` and set your account locator and password. **Do not commit this.** Alternatively, you can set the environment variables `TEST_ACC_LOCATOR` and `TEST_ACC_PASSWORD`.
* Run `cargo test --release -- --nocapture` and make sure that no errors are reported.

### Viewing debug logs

The codebase contains instrumentation statements that log potentially useful debugging information, at different priorities. Such statements include the macros `debug!` and `trace!`, and more can be found at the documentation for the [log crate](https://docs.rs/log). Feel free to add trace calls to the code when debugging.

In order to view the output of trace calls you will need to initialize logging at the beginning of your test:

```rust
unwrap!(maidsafe_utilities::log::init(true));
```

Then you will need to set the `RUST_LOG` environment variable to the desired logging level for the desired modules or crates. To view trace calls for `safe_authenticator`, for example, you may do this:

```shell
export RUST_LOG=safe_authenticator=trace
```

You could also set `RUST_LOG=trace` which will output *all* trace logs, but this may produce far more data than desired. For more information please see the documentation for the `log` module in our crate [maidsafe_utilities](https://docs.rs/maidsafe_utilities).

## License

Licensed under either of

* the MaidSafe.net Commercial License, version 1.0 or later ([LICENSE](LICENSE))
* the General Public License (GPL), version 3 ([COPYING](COPYING) or http://www.gnu.org/licenses/gpl-3.0.en.html)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the MaidSafe Contributor Agreement ([CONTRIBUTOR](CONTRIBUTOR)), shall be
dual licensed as above, and you agree to be bound by the terms of the MaidSafe Contributor Agreement.
