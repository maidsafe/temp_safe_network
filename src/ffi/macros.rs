// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

macro_rules! ffi_error_code {
    ($result:expr) => {{
        let decorator = ::std::iter::repeat('-').take(50).collect::<String>();
        let err_str = format!("{:?}", $result);
        let err_code: i32 = $result.into();
        info!("\nFFI cross-boundary error propagation:\n {}\n| **ERRNO: {}** {}\n {}\n\n",
              decorator, err_code, err_str, decorator);
        err_code
    }}
}

macro_rules! ffi_result_code {
    ($result:expr) => {
        match $result {
            Ok(_) => 0,
            Err(error) => ffi_error_code!(error),
        }
    }
}

macro_rules! ffi_try {
    ($result:expr) => {
        match $result {
            Ok(value)  => value,
            Err(error) => {
                return ffi_error_code!(error)
            },
        }
    }
}

macro_rules! ffi_ptr_try {
    ($result:expr, $out:expr) => {
        match $result {
            Ok(value)  => value,
            Err(error) => {
                let _ = ffi_error_code!(error);
                return ::std::ptr::null();
            },
        }
    }
}
