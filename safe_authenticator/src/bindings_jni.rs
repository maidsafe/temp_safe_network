// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

#![allow(non_snake_case, missing_docs, unsafe_code, unused_results, trivial_casts,
         trivial_numeric_casts, unused, unused_qualifications)]


use super::*;
use ffi_utils::*;
use jni::{self, JNIEnv, JavaVM};
use jni::errors::{Error as JniError, ErrorKind};
use jni::objects::{GlobalRef, JClass, JObject, JString};
use jni::strings::JNIStr;
use jni::sys::{jbyte, jbyteArray, jint, jlong, jobject, jsize};
use safe_core;
use safe_core::arrays::*;
use safe_core::ffi::*;
use safe_core::ffi::ipc::req::{AppExchangeInfo, AuthReq, ContainerPermissions, ContainersReq,
                               PermissionSet, ShareMData, ShareMDataReq};
use safe_core::ffi::ipc::resp::{AccessContInfo, AccessContainerEntry, AppAccess, AppKeys,
                                AuthGranted, ContainerInfo, MDataEntry, MDataKey, MDataValue,
                                MetadataResponse};
use safe_core::ffi::nfs::File;
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_void};
use std::slice;

type JniResult<T> = Result<T, JniError>;

/// Generates a `user_data` context containing a reference to a single or several Java callbacks
macro_rules! gen_ctx {
    ($env:ident, $cb:ident) => {
        {
            let ctx = $env.new_global_ref($cb).unwrap();
            $env.delete_local_ref($cb).unwrap();
            let ptr = *ctx.as_obj() as *mut c_void;
            mem::forget(ctx);
            ptr
        }
    };

    ($env:ident, $cb0:ident, $($cb_rest:ident),+ ) => {
        {
            let ctx = [
                Some($env.new_global_ref($cb0).unwrap()),
                $(
                    Some($env.new_global_ref($cb_rest).unwrap()),
                )+
            ];
            let ctx = Box::into_raw(Box::new(ctx)) as *mut c_void;
            $env.delete_local_ref($cb0).unwrap();
            $(
                $env.delete_local_ref($cb_rest).unwrap();
            )+
            ctx
        }
    }
}

/// Generates primitive type converters
macro_rules! gen_primitive_type_converter {
    ($native_type:ty, $java_type:ty) => {
        impl FromJava<$java_type> for $native_type {
            fn from_java(_env: &JNIEnv, input: $java_type) -> JniResult<Self> {
                Ok(input as Self)
            }
        }

        impl<'a> ToJava<'a, $java_type> for $native_type {
            fn to_java(&self, _env: &JNIEnv) -> JniResult<$java_type> {
                Ok(*self as $java_type)
            }
        }
    }
}

macro_rules! gen_byte_array_converter {
    ($arr_type:ty, $size:expr) => {
        impl<'a> FromJava<JObject<'a>> for [$arr_type; $size] {
            fn from_java(env: &JNIEnv, input: JObject) -> JniResult<Self> {
                let input = input.into_inner() as jbyteArray;
                let mut output = [0; $size];
                env.get_byte_array_region(input, 0, &mut output)?;

                Ok(unsafe { mem::transmute(output) })
            }
        }

        impl<'a> ToJava<'a, JObject<'a>> for [$arr_type; $size] {
            fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
                let output = env.new_byte_array(self.len() as jsize)?;
                env.set_byte_array_region(output, 0, unsafe {
                    slice::from_raw_parts(self.as_ptr() as *const i8, self.len())
                })?;
                Ok(JObject::from(output as jobject))
            }
        }
    }
}

macro_rules! jni_unwrap {
    ($res:expr) => {{
        let res: Result<_, JniError> = $res;
        if let Err(JniError(ErrorKind::JavaException, _)) = res {
            return;
        } else {
            res.unwrap()
        }
    }}
}

/// Converts `user_data` back into a Java callback object
unsafe fn convert_cb_from_java(env: &JNIEnv, ctx: *mut c_void) -> GlobalRef {
    GlobalRef::from_raw(unwrap!(env.get_java_vm()), ctx as jobject)
}

static mut JVM: Option<JavaVM> = None;

#[no_mangle]
// This is called when `loadLibrary` is called on the Java side.
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

impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [RegisteredApp] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        object_array_to_java(self, env, "net/maidsafe/safe_authenticator/RegisteredApp")
    }
}

impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [AppAccess] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        object_array_to_java(self, env, "net/maidsafe/safe_authenticator/AppAccess")
    }
}

impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [AppExchangeInfo] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        object_array_to_java(self, env, "net/maidsafe/safe_authenticator/AppExchangeInfo")
    }
}

impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [MDataKey] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        object_array_to_java(self, env, "net/maidsafe/safe_authenticator/MDataKey")
    }
}

impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [MDataValue] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        object_array_to_java(self, env, "net/maidsafe/safe_authenticator/MDataValue")
    }
}

impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [MDataEntry] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        object_array_to_java(self, env, "net/maidsafe/safe_authenticator/MDataEntry")
    }
}

impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [ContainerPermissions] {
    fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
        object_array_to_java(
            self,
            env,
            "net/maidsafe/safe_authenticator/ContainerPermissions",
        )
    }
}

/// Converts object arrays into Java arrays
fn object_array_to_java<'a, T: ToJava<'a, U>, U: Into<JObject<'a>> + 'a>(
    list: &[T],
    env: &'a JNIEnv,
    class: &str,
) -> JniResult<JObject<'a>> {
    let output = env.new_object_array(
        list.len() as jsize,
        class,
        JObject::null(),
    )?;

    for (idx, entry) in list.iter().enumerate() {
        let jentry = entry.to_java(env)?.into();
        env.set_object_array_element(output, idx as i32, jentry);
    }

    Ok(JObject::from(output as jobject))
}

include!("../../bindings/java/safe_authenticator/jni.rs");
