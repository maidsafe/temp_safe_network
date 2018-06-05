// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Java/JNI utilities.

use jni::errors::Error as JniError;
use jni::objects::{GlobalRef, JObject};
use jni::sys::{jobject, jsize};
use jni::JNIEnv;
use std::os::raw::c_void;

/// Result returning JNI errors
pub type JniResult<T> = Result<T, JniError>;

/// Unwraps the results and checks for Java exceptions.
/// Required for exceptions pass-through (simplifies debugging).
#[macro_export]
macro_rules! jni_unwrap {
    ($res:expr) => {{
        let res: Result<_, JniError> = $res;
        if let Err(JniError(ErrorKind::JavaException, _)) = res {
            return;
        } else {
            res.unwrap()
        }
    }};
}

/// Generates a `user_data` context containing a reference to a single or several Java callbacks
#[macro_export]
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
#[macro_export]
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
    };
}

#[macro_export]
macro_rules! gen_object_array_converter {
    ($native_type:ident, $java_ty_name:expr) => {
        impl<'a, 'b> ToJava<'a, JObject<'a>> for &'b [$native_type] {
            fn to_java(&self, env: &'a JNIEnv) -> JniResult<JObject<'a>> {
                object_array_to_java($native_type::to_java, self, env, $java_ty_name)
            }
        }
    };
}

#[macro_export]
macro_rules! gen_byte_array_converter {
    ($arr_type:ty, $size:expr) => {
        impl<'a> FromJava<JObject<'a>> for [$arr_type; $size] {
            fn from_java(env: &JNIEnv, input: JObject) -> JniResult<Self> {
                let input = input.into_inner() as jbyteArray;
                let mut output = [0; $size];

                let len = env.get_array_length(input)? as usize;
                env.get_byte_array_region(input, 0, &mut output[0..cmp::min(len, $size)])?;

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
    };
}

/// Converts object arrays into Java arrays
pub fn object_array_to_java<
    'a,
    T,
    F: Fn(&T, &'a JNIEnv) -> JniResult<U>,
    U: Into<JObject<'a>> + 'a,
>(
    transform_fn: F,
    list: &[T],
    env: &'a JNIEnv,
    class: &str,
) -> JniResult<JObject<'a>> {
    let output = env.new_object_array(list.len() as jsize, class, JObject::null())?;

    for (idx, entry) in list.iter().enumerate() {
        let jentry = transform_fn(entry, env)?.into();
        env.set_object_array_element(output, idx as i32, jentry)?;
    }

    Ok(JObject::from(output))
}

/// Converts `user_data` back into a Java callback object
pub unsafe fn convert_cb_from_java(env: &JNIEnv, ctx: *mut c_void) -> GlobalRef {
    GlobalRef::from_raw(unwrap!(env.get_java_vm()), ctx as jobject)
}
