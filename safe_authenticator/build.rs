// Copyright 2017 MaidSafe.net limited.
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

//! Build script for generating C header files from FFI modules.

extern crate ffi_utils;
extern crate rust_sodium;
extern crate routing;
extern crate safe_bindgen;
#[macro_use]
extern crate unwrap;

use routing::XOR_NAME_LEN;
use rust_sodium::crypto::{box_, secretbox, sign};
use safe_bindgen::{Bindgen, FilterMode, LangCSharp};
use std::collections::HashMap;
use std::env;
use std::path::Path;

fn main() {
    if env::var("CARGO_FEATURE_BINDINGS").is_err() {
        return;
    }

    gen_bindings_c();
    gen_bindings_csharp();
}

fn gen_bindings_c() {
    unwrap!(ffi_utils::header_gen::gen_headers(
        &unwrap!(env::var("CARGO_PKG_NAME")),
        "../bindings/c/",
        "src/lib.rs",
    ));
}

fn gen_bindings_csharp() {
    let target_dir = Path::new("../bindings/csharp/safe_authenticator");

    let mut bindgen = unwrap!(Bindgen::new());
    let mut lang = LangCSharp::new();

    lang.set_lib_name(unwrap!(env::var("CARGO_PKG_NAME")));
    lang.set_namespace("SafeAuth");
    lang.set_class_name("AuthBindings");
    lang.set_consts_class_name("AuthConstants");
    lang.set_types_file_name("AuthTypes");
    lang.set_utils_class_name("BindingUtils");
    lang.add_const("ulong", "ASYM_PUBLIC_KEY_LEN", box_::PUBLICKEYBYTES);
    lang.add_const("ulong", "ASYM_SECRET_KEY_LEN", box_::SECRETKEYBYTES);
    lang.add_const("ulong", "ASYM_NONCE_LEN", box_::NONCEBYTES);
    lang.add_const("ulong", "SYM_KEY_LEN", secretbox::KEYBYTES);
    lang.add_const("ulong", "SYM_NONCE_LEN", secretbox::NONCEBYTES);
    lang.add_const("ulong", "SIGN_PUBLIC_KEY_LEN", sign::PUBLICKEYBYTES);
    lang.add_const("ulong", "SIGN_SECRET_KEY_LEN", sign::SECRETKEYBYTES);
    lang.add_const("ulong", "XOR_NAME_LEN", XOR_NAME_LEN);
    lang.add_opaque_type("Authenticator");

    lang.reset_filter(FilterMode::Blacklist);
    lang.filter("AuthFuture");

    bindgen.source_file("../safe_core/src/lib.rs");
    unwrap!(bindgen.compile(&mut lang, &mut HashMap::new(), false));

    bindgen.source_file("src/lib.rs");
    bindgen.run_build(&mut lang, target_dir);

    // Hand-written code.
    let resource_path = Path::new("resource");
    if resource_path.is_dir() {
        unwrap!(ffi_utils::bindgen_utils::copy_files(
            resource_path,
            target_dir,
            ".cs",
        ));
    }
}
