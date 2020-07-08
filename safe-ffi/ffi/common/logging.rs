// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::errors::{Error, Result};
use ffi_utils::{catch_unwind_cb, FfiResult, ReprC, FFI_RESULT_OK};
use safe_api::Error as NativeError;
use safe_core::{config_handler, utils::logging};
use std::ffi::{CString, OsStr};
use std::os::raw::{c_char, c_void};

/// Sets the path from which the `safe_core.config` file will be read.
#[no_mangle]
pub unsafe extern "C" fn set_config_dir_path(
    new_path: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<_> {
        let new_path = String::clone_from_repr_c(new_path)?;
        config_handler::set_config_dir_path(OsStr::new(&new_path));
        o_cb(user_data, FFI_RESULT_OK);
        Ok(())
    });
}

/// This function should be called to enable logging to a file.
/// If `output_file_name_override` is provided, then this path will be used for
/// the log output file.
#[no_mangle]
pub unsafe extern "C" fn init_logging(
    output_file_name_override: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        if output_file_name_override.is_null() {
            logging::init(false).map_err(|err| Error::from(NativeError::Unexpected(err)))?;
        } else {
            let output_file_name_override = String::clone_from_repr_c(output_file_name_override)?;
            logging::init_with_output_file(false, output_file_name_override)
                .map_err(|err| Error::from(NativeError::Unexpected(err)))?;
        }
        o_cb(user_data, FFI_RESULT_OK);
        Ok(())
    });
}

/// Returns the path at which the the configuration files are expected.
#[no_mangle]
pub unsafe extern "C" fn config_dir_path(
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, log_path: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> Result<()> {
        let config_dir = safe_core::config_dir()
            .map_err(|err| Error::from(NativeError::Unexpected(err.to_string())))?;
        let config_dir_path = CString::new(
            config_dir
                .into_os_string()
                .into_string()
                .map_err(|_| {
                    Error::from(NativeError::Unexpected(
                        "Couldn't convert OsString".to_string(),
                    ))
                })
                .map_err(Error::from)?
                .into_bytes(),
        )?;
        o_cb(user_data, FFI_RESULT_OK, config_dir_path.as_ptr());
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffi_utils::test_utils::{call_0, call_1};
    use log::{debug, error};
    use safe_core::config_dir;
    use std::env;
    use std::fs::{self, File};
    use std::io::Read;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::thread;
    use std::time::Duration;
    use unwrap::unwrap;

    // Test path where log file is created.
    #[test]
    fn test_config_dir_path() {
        let path: String = unsafe { unwrap!(call_1(|ud, cb| config_dir_path(ud, cb),)) };
        let expected_path = unwrap!(unwrap!(config_dir()).into_os_string().into_string());

        assert_eq!(path, expected_path);
    }

    // Test logging errors to file.
    #[test]
    fn file_logging() {
        setup_log_config();

        let log_file_name = unwrap!(PathBuf::from_str("AppClient.log"));

        let file_name = unwrap!(CString::new(unwrap!(log_file_name
            .clone()
            .into_os_string()
            .into_string())));
        unsafe {
            unwrap!(call_0(|ud, cb| init_logging(file_name.as_ptr(), ud, cb),));
        }

        let debug_msg = "This is a sample debug message".to_owned();
        let junk_msg = "This message should not exist in the log file".to_owned();

        error!("{}", debug_msg);
        debug!("{}", junk_msg);

        // Give some time to the async logging to flush in the background thread
        thread::sleep(Duration::from_secs(1));

        let log_path = unwrap!(config_dir()).join(log_file_name);
        let mut log_file = unwrap!(File::open(log_path));
        let mut file_content = String::new();

        let written = unwrap!(log_file.read_to_string(&mut file_content));
        assert!(written > 0);

        assert!(file_content.contains(&debug_msg[..]));
        assert!(!file_content.contains(&junk_msg[..]));
    }

    fn setup_log_config() {
        let mut current_dir = unwrap!(env::current_dir());
        let mut config_dir = unwrap!(config_dir());
        unwrap!(fs::create_dir_all(config_dir.clone()));

        if current_dir.as_path() != config_dir.as_path() {
            // Try to copy log.toml from the current dir to config dir
            // so that the config_handler can find it
            current_dir.push("sample_log_file/log.toml");
            config_dir.push("log.toml");

            let _ = unwrap!(fs::copy(current_dir, config_dir));
        }
    }
}
