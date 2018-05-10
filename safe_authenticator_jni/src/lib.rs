// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#![allow(non_snake_case, missing_docs, unsafe_code, unused_results, trivial_casts,
         trivial_numeric_casts, unused, unused_qualifications)]

#[macro_use]
extern crate ffi_utils;
extern crate jni;
extern crate safe_authenticator;
extern crate safe_core;
#[macro_use]
extern crate unwrap;

use ffi_utils::*;
use ffi_utils::java::{JniResult, convert_cb_from_java, object_array_to_java};
use jni::{JNIEnv, JavaVM};
use jni::errors::{Error as JniError, ErrorKind};
use jni::objects::{GlobalRef, JClass, JObject, JString};
use jni::strings::JNIStr;
use jni::sys::{jbyte, jbyteArray, jint, jlong, jobject, jsize};
use safe_authenticator::*;
use safe_core::arrays::*;
use safe_core::ffi::*;
use safe_core::ffi::ipc::req::{AppExchangeInfo, AuthReq, ContainerPermissions, ContainersReq,
                               PermissionSet, ShareMData, ShareMDataReq};
use safe_core::ffi::ipc::resp::{AccessContInfo, AccessContainerEntry, AppAccess, AppKeys,
                                AuthGranted, ContainerInfo, MDataEntry, MDataKey, MDataValue,
                                MetadataResponse};
use safe_core::ffi::nfs::File;
use std::{cmp, mem, slice};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

#[repr(C)]
struct Authenticator(*mut c_void);

static mut JVM: Option<JavaVM> = None;

// This is called when `loadLibrary` is called on the Java side.
#[no_mangle]
pub unsafe extern "C" fn JNI_OnLoad(vm: *mut jni::sys::JavaVM, _reserved: *mut c_void) -> jint {
    JVM = Some(unwrap!(JavaVM::from_raw(vm)));
    jni::sys::JNI_VERSION_1_4
}

// Trait for conversion of rust value to java value.
trait ToJava<'a, T: Sized + 'a> {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<T>;
}

// Trait for conversion of java value to rust value.
trait FromJava<T> {
    fn from_java(env: &JNIEnv, input: T) -> JniResult<Self>
    where
        Self: Sized;
}

gen_primitive_type_converter!(u8, jbyte);
gen_primitive_type_converter!(i32, jint);
gen_primitive_type_converter!(u32, jint);
gen_primitive_type_converter!(i64, jlong);
gen_primitive_type_converter!(u64, jlong);

gen_byte_array_converter!(i8, 8);
gen_byte_array_converter!(u8, 24);
gen_byte_array_converter!(u8, 32);
gen_byte_array_converter!(u8, 64);

impl<'a> ToJava<'a, bool> for bool {
    fn to_java(&self, _env: &JNIEnv) -> JniResult<bool> {
        Ok(*self)
    }
}

impl<'a> ToJava<'a, jlong> for usize {
    fn to_java(&self, _env: &JNIEnv) -> JniResult<jlong> {
        Ok(*self as jlong)
    }
}

impl<'a> FromJava<JString<'a>> for *const c_char {
    fn from_java(env: &JNIEnv, input: JString) -> JniResult<Self> {
        Ok(CString::from_java(env, input)?.into_raw())
    }
}

impl<'a> ToJava<'a, JString<'a>> for *const c_char {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JString<'a>> {
        Ok(unsafe {
            unwrap!(env.new_string(JNIStr::from_ptr(*self).to_owned()))
        })
    }
}

impl<'a> FromJava<JString<'a>> for *mut c_char {
    fn from_java(env: &JNIEnv, input: JString) -> JniResult<Self> {
        Ok(<*const _>::from_java(env, input)? as *mut _)
    }
}

impl<'a> ToJava<'a, JString<'a>> for *mut c_char {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JString<'a>> {
        Ok((*self as *const c_char).to_java(env)?)
    }
}


impl<'a> FromJava<JString<'a>> for CString {
    fn from_java(env: &JNIEnv, input: JString) -> JniResult<Self> {
        let tmp: &CStr = &*unwrap!(env.get_string(input));
        Ok(tmp.to_owned())
    }
}

// TODO: implement this for all primitive types (consider defining a `PrimitiveType`
// trait and implement it for all rust types that correspond to primitive java types)
impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [i32] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        let output = env.new_int_array(self.len() as jsize)?;
        env.set_int_array_region(output, 0, self)?;
        Ok(JObject::from(output as jobject))
    }
}

impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [u8] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        let output = env.new_byte_array(self.len() as jsize)?;
        env.set_byte_array_region(output, 0, unsafe {
            slice::from_raw_parts(self.as_ptr() as *const i8, self.len())
        })?;
        Ok(JObject::from(output as jobject))
    }
}

impl<'a> FromJava<JObject<'a>> for Vec<u8> {
    fn from_java(env: &JNIEnv, input: JObject) -> JniResult<Self> {
        let input = input.into_inner() as jbyteArray;
        Ok(env.convert_byte_array(input)?)
    }
}

gen_object_array_converter!(
    RegisteredApp,
    "net/maidsafe/safe_authenticator/RegisteredApp"
);
gen_object_array_converter!(AppAccess, "net/maidsafe/safe_authenticator/AppAccess");
gen_object_array_converter!(
    AppExchangeInfo,
    "net/maidsafe/safe_authenticator/AppExchangeInfo"
);
gen_object_array_converter!(MDataKey, "net/maidsafe/safe_authenticator/MDataKey");
gen_object_array_converter!(MDataValue, "net/maidsafe/safe_authenticator/MDataValue");
gen_object_array_converter!(MDataEntry, "net/maidsafe/safe_authenticator/MDataEntry");
gen_object_array_converter!(
    ContainerPermissions,
    "net/maidsafe/safe_authenticator/ContainerPermissions"
);

include!("../../bindings/java/safe_authenticator/jni.rs");
