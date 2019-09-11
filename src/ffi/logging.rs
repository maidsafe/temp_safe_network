use crate::api::{ResultReturn, Error};
use config_file_handler::FileHandler;
use ffi_utils::{catch_unwind_cb, from_c_str, FfiResult, FFI_RESULT_OK};
use maidsafe_utilities::log;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};

/// This function should be called to enable logging to a file.
/// If `output_file_name_override` is provided, then this path will be used for
/// the log output file.
#[no_mangle]
pub unsafe extern "C" fn app_init_logging(
    output_file_name_override: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult),
) {
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        if output_file_name_override.is_null() {
            log::init(false)?;
        } else {
            let output_file_name_override = from_c_str(output_file_name_override)?;
            log::init_with_output_file(false, output_file_name_override)?;
        }
        o_cb(user_data, FFI_RESULT_OK);
        Ok(())
    });
}

/// This function should be called to find where log file will be created. It
/// will additionally create an empty log file in the path in the deduced
/// location and will return the file name along with complete path to it.
#[no_mangle]
pub unsafe extern "C" fn app_output_log_path(
    output_file_name: *const c_char,
    user_data: *mut c_void,
    o_cb: extern "C" fn(user_data: *mut c_void, result: *const FfiResult, log_path: *const c_char),
) {
    catch_unwind_cb(user_data, o_cb, || -> ResultReturn<()> {
        let op_file = from_c_str(output_file_name)?;
        let fh = FileHandler::<()>::new(&op_file, true)
            .map_err(|e| Error::Unexpected(format!("{}", e)))?;
        let op_file_path = CString::new(
            fh.path()
                .to_path_buf()
                .into_os_string()
                .into_string()
                .map_err(|_| Error::Unexpected("Couldn't convert OsString".to_string()))?
                .into_bytes(),
        )?;
        o_cb(user_data, FFI_RESULT_OK, op_file_path.as_ptr());
        Ok(())
    })
}
