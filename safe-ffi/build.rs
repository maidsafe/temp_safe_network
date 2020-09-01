// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#[cfg(not(feature = "bindings"))]
fn main() {}

#[cfg(feature = "bindings")]
fn main() {
    bindings::main();
}

#[cfg(feature = "bindings")]
mod bindings {
    use sn_bindgen::{Bindgen, FilterMode, LangC, LangCSharp};
    use std::collections::HashMap;
    use std::env;
    use std::path::Path;
    use unwrap::unwrap;
    use xor_name::XOR_NAME_LEN;

    const BLS_PUBLIC_KEY_LEN: usize = 48;

    const BSD_MIT_LICENSE: &str = "// Copyright 2019 MaidSafe.net limited.\n\
         //\n\
         // This SAFE Network Software is licensed to you under the MIT license\n\
         // <LICENSE-MIT or https://opensource.org/licenses/MIT> or the Modified\n\
         // BSD license <LICENSE-BSD or https://opensource.org/licenses/BSD-3-Clause>,\n\
         // at your option. This file may not be copied, modified, or distributed\n\
         // except according to those terms. Please review the Licences for the\n\
         // specific language governing permissions and limitations relating to use\n\
         // of the SAFE Network Software.";

    const FFI_UTILS_CODE: &str =
        "#[repr(C)] pub struct FfiResult { error_code: i32, description: *const c_char }";

    pub fn main() {
        gen_bindings_c();
        gen_bindings_csharp();
    }

    fn gen_bindings_c() {
        let target_dir = Path::new("bindings/c/safe-api");
        let mut outputs = HashMap::new();

        let mut bindgen = unwrap!(Bindgen::new());
        let mut lang = LangC::new();

        lang.set_lib_name("ffi_utils");
        bindgen.source_code("ffi_utils/src/lib.rs", FFI_UTILS_CODE);
        bindgen.compile_or_panic(&mut lang, &mut outputs, false);

        lang.add_custom_code("typedef void* Safe;\n");
        lang.set_lib_name("safe_ffi");
        bindgen.source_file("lib.rs");
        bindgen.compile_or_panic(&mut lang, &mut outputs, true);

        add_license_headers(&mut outputs);
        bindgen.write_outputs_or_panic(target_dir, &outputs);
    }

    fn gen_bindings_csharp() {
        let target_dir = Path::new("bindings/csharp/safe-api");

        let mut bindgen = unwrap!(Bindgen::new());
        let mut lang = LangCSharp::new();

        lang.set_lib_name(unwrap!(env::var("CARGO_PKG_NAME")));

        lang.set_interface_section(
            "SafeApp.AppBindings/IAppBindings.cs",
            "SafeApp.AppBindings",
            "IAppBindings",
        );
        lang.set_functions_section(
            "SafeApp.AppBindings/AppBindings.cs",
            "SafeApp.AppBindings",
            "AppBindings",
        );
        lang.set_consts_section(
            "SafeApp.Core/AppConstants.cs",
            "SafeApp.Core",
            "AppConstants",
        );
        lang.set_types_section("SafeApp.Core/AppTypes.cs", "SafeApp.Core");
        lang.set_utils_section(
            "SafeApp.Core/BindingUtils.cs",
            "SafeApp.Core",
            "BindingUtils",
        );

        lang.add_const("ulong", "XOR_NAME_LEN", XOR_NAME_LEN);
        lang.add_const("ulong", "BLS_PUBLIC_KEY_LEN", BLS_PUBLIC_KEY_LEN);
        lang.add_opaque_type("SafeAuthenticator");
        lang.add_opaque_type("Safe");

        lang.reset_filter(FilterMode::Blacklist);

        let mut outputs = HashMap::new();
        bindgen.source_file("lib.rs");
        bindgen.compile_or_panic(&mut lang, &mut outputs, true);
        apply_patches(&mut outputs);
        bindgen.write_outputs_or_panic(target_dir, &outputs);

        lang.set_consts_enabled(false);
        lang.set_types_enabled(false);
        lang.set_utils_enabled(false);

        lang.reset_filter(FilterMode::Whitelist);

        outputs.clear();
        bindgen.compile_or_panic(&mut lang, &mut outputs, true);
        add_license_headers(&mut outputs);
        bindgen.write_outputs_or_panic(target_dir, &outputs);
    }

    fn add_license_headers(outputs: &mut HashMap<String, String>) {
        for content in outputs.values_mut() {
            add_license_header(content);
        }
    }

    fn add_license_header(content: &mut String) {
        *content = format!("{}\n{}", BSD_MIT_LICENSE, content);
    }

    fn apply_patches(outputs: &mut HashMap<String, String>) {
        {
            let content = fetch_mut(outputs, "SafeApp.AppBindings/AppBindings.cs");
            insert_using_utilities(content);
            insert_using_obj_c_runtime(content);
            insert_guard(content);
        }

        // todo : uncomment when adding structures
        //insert_internals_visible_to(fetch_mut(outputs, "SafeApp.Core/AppTypes.cs"));

        for content in outputs.values_mut() {
            fix_names(content);
        }
    }

    fn insert_guard(content: &mut String) {
        content.insert_str(0, "#if !NETSTANDARD1_2 || __DESKTOP__\n");
        content.push_str("#endif\n");
    }

    fn insert_using_utilities(content: &mut String) {
        content.insert_str(0, "using SafeApp.Core;\n");
    }

    fn insert_using_obj_c_runtime(content: &mut String) {
        content.insert_str(0, "#if __IOS__\nusing ObjCRuntime;\n#endif\n");
    }

    #[allow(dead_code)]
    fn insert_internals_visible_to(content: &mut String) {
        content.insert_str(0, "using System.Runtime.CompilerServices;\n");

        // HACK: The [assembly: ...] annotation must go after all using declaration.
        *content = content.replace(
            "namespace SafeApp.Core",
            "[assembly: InternalsVisibleTo(\"SafeApp.AppBindings\")]\n\
             namespace SafeApp.Core",
        );
    }

    fn fix_names(content: &mut String) {
        *content = content.replace("Idata", "IData").replace("Mdata", "MData");
    }

    fn fetch_mut<'a>(outputs: &'a mut HashMap<String, String>, key: &str) -> &'a mut String {
        unwrap!(outputs.get_mut(key), "key {:?} not found in outputs", key)
    }
}
