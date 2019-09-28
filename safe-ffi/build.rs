// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#[cfg(not(feature = "bindings"))]
fn main() {}

#[cfg(feature = "bindings")]
fn main() {
    bindings::main();
}

#[cfg(feature = "bindings")]
mod bindings {
    use rust_sodium::crypto::{box_, secretbox, sign};
    use safe_bindgen::{Bindgen, FilterMode, LangCSharp};
    use safe_nd::XOR_NAME_LEN;
    use std::collections::HashMap;
    use std::env;
    use std::path::Path;
    use unwrap::unwrap;

    const BSD_MIT_LICENSE: &str =
        "// Copyright 2019 MaidSafe.net limited.\n\
         //\n\
         // This SAFE Network Software is licensed to you under the MIT license\n\
         // <LICENSE-MIT or https://opensource.org/licenses/MIT> or the Modified\n\
         // BSD license <LICENSE-BSD or https://opensource.org/licenses/BSD-3-Clause>,\n\
         // at your option. This file may not be copied, modified, or distributed\n\
         // except according to those terms. Please review the Licences for the\n\
         // specific language governing permissions and limitations relating to use\n\
         // of the SAFE Network Software.";

    pub fn main() {
        gen_bindings_csharp();
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

        lang.add_const("ulong", "ASYM_PUBLIC_KEY_LEN", box_::PUBLICKEYBYTES);
        lang.add_const("ulong", "ASYM_SECRET_KEY_LEN", box_::SECRETKEYBYTES);
        lang.add_const("ulong", "ASYM_NONCE_LEN", box_::NONCEBYTES);
        lang.add_const("ulong", "SYM_KEY_LEN", secretbox::KEYBYTES);
        lang.add_const("ulong", "SYM_NONCE_LEN", secretbox::NONCEBYTES);
        lang.add_const("ulong", "SIGN_PUBLIC_KEY_LEN", sign::PUBLICKEYBYTES);
        lang.add_const("ulong", "SIGN_SECRET_KEY_LEN", sign::SECRETKEYBYTES);
        lang.add_const("ulong", "XOR_NAME_LEN", XOR_NAME_LEN);
        lang.add_const("ulong", "BLS_PUBLIC_KEY_LEN", "48");
        lang.add_opaque_type("App");

        lang.reset_filter(FilterMode::Blacklist);

        let mut outputs = HashMap::new();
        bindgen.source_file("lib.rs");
        bindgen.compile_or_panic(&mut lang, &mut outputs, true);
        apply_patches(&mut outputs);
        bindgen.write_outputs_or_panic(target_dir, &outputs);

        lang.set_consts_enabled(false);
        lang.set_types_enabled(false);
        lang.set_utils_enabled(false);
        lang.add_opaque_type("App");

        lang.reset_filter(FilterMode::Whitelist);

        outputs.clear();
        bindgen.compile_or_panic(&mut lang, &mut outputs, true);
        // apply_patches_testing(&mut outputs);
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
            insert_resharper_disable_inconsistent_naming(content);
        }

        // todo : uncomment when adding structures
        //insert_internals_visible_to(fetch_mut(outputs, "SafeApp.Core/AppTypes.cs"));

        for content in outputs.values_mut() {
            fix_names(content);
        }
    }

    fn apply_patches_testing(outputs: &mut HashMap<String, String>) {
        insert_using_utilities(fetch_mut(
            outputs,
            "SafeApp.MockAuthBindings/MockAuthBindings.cs",
        ));

        insert_using_utilities(fetch_mut(
            outputs,
            "SafeApp.MockAuthBindings/IMockAuthBindings.cs",
        ));

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

    fn insert_internals_visible_to(content: &mut String) {
        content.insert_str(0, "using System.Runtime.CompilerServices;\n");

        // HACK: The [assembly: ...] annotation must go after all using declaration.
        *content = content.replace(
            "namespace SafeApp.Core",
            "[assembly: InternalsVisibleTo(\"SafeApp.AppBindings\")]\n\
             [assembly: InternalsVisibleTo(\"SafeApp.MockAuthBindings\")]\n\n\
             namespace SafeApp.Core",
        );
    }

    fn insert_resharper_disable_inconsistent_naming(content: &mut String) {
        content.insert_str(0, "// ReSharper disable InconsistentNaming\n");
    }

    fn fix_names(content: &mut String) {
        *content = content.replace("Idata", "IData").replace("Mdata", "MData");
    }

    fn fetch_mut<'a>(outputs: &'a mut HashMap<String, String>, key: &str) -> &'a mut String {
        unwrap!(outputs.get_mut(key), "key {:?} not found in outputs", key)
    }
}
