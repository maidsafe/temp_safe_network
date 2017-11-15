# FFI utils - Change Log

## [0.4.0]
- Use pointers to `FfiResult` instead of passing by value
- Change type of `FFI_RESULT_OK` to a static reference
- Don't add padding to URIs
- Update base64 version
- Add support for using a single user data parameter for multiple callbacks
- Add tests for the `catch_unwind` family of functions

## [0.3.0]
- Improve documentation and fix bugs
- Fix compiler errors on rustc-nightly

## [0.2.0]
- Change the log output for FFI errors - remove the decoration and reduce the log level

## [0.1.0]
- Provide FFI utility functions for safe_client_libs
