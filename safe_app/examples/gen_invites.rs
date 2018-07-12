// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Generate invitations example.

// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(
    exceeding_bitshifts, mutable_transmutes, no_mangle_const_items, unknown_crate_types, warnings
)]
#![deny(
    bad_style, deprecated, improper_ctypes, missing_docs, non_shorthand_field_patterns,
    overflowing_literals, plugin_as_library, private_no_mangle_fns, private_no_mangle_statics,
    stable_features, unconditional_recursion, unknown_lints, unsafe_code, unused, unused_allocation,
    unused_attributes, unused_comparisons, unused_features, unused_parens, while_true
)]
#![warn(
    trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
    unused_qualifications, unused_results
)]
#![allow(
    box_pointers, missing_copy_implementations, missing_debug_implementations,
    variant_size_differences
)]
#![cfg_attr(
    feature = "cargo-clippy",
    deny(clippy, unicode_not_nfc, wrong_pub_self_convention, option_unwrap_used)
)]
#![cfg_attr(feature = "cargo-clippy", allow(implicit_hasher, too_many_arguments, use_debug))]

#[macro_use]
extern crate unwrap;

extern crate docopt;
extern crate futures;
extern crate maidsafe_utilities;
#[macro_use]
extern crate safe_core;
extern crate rand;
extern crate routing;
extern crate rustc_serialize;
extern crate safe_app;
extern crate safe_authenticator;
extern crate tiny_keccak;

use docopt::Docopt;
use futures::Future;
use rand::{thread_rng, Rng};
use routing::{Action, MutableData, PermissionSet, User, XorName};
use safe_app::App;
use safe_authenticator::Authenticator;
use safe_core::utils::seed;
use safe_core::{Client, FutureExt};
use std::fs::File;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::time::UNIX_EPOCH;
use tiny_keccak::sha3_256;

const INVITE_TOKEN_SIZE: usize = 90;
const INVITE_TOKEN_TYPE_TAG: u64 = 8;

static USAGE: &'static str = "
Usage:
  gen_invites [--gen-seed SIZE | --get-pk | --check-invite INVITE | -c [-n INVITES] | \
               -n INVITES | -h]

Options:
  --gen-seed SIZE            Only generate a random seed of given size, writing into input file.
  --get-pk                   Only get the public sign key given the seed, don't do anything
                             extra.
  --check-invite INVITE      Only check the status of the given invite (exists, consumed etc.).
  -c, --create               Create account using seed from input file. By default it will login.
  -n, --num-invites INVITES  Number of invites to generate (will populate the Network too).
  -h, --help                 Display this help message and exit.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_gen_seed: Option<usize>,
    flag_get_pk: bool,
    flag_check_invite: Option<String>,
    flag_create: bool,
    flag_num_invites: Option<usize>,
    flag_help: bool,
}

fn main() {
    unwrap!(maidsafe_utilities::log::init(true));

    let args: Args = Docopt::new(USAGE)
        .and_then(|docopt| docopt.decode())
        .unwrap_or_else(|error| error.exit());

    if let Some(size) = args.flag_gen_seed {
        let mut input = unwrap!(File::create("./input"), "Unable to create input file");
        let seed = generate_random_printable(size);
        unwrap!(write!(input, "{}", seed));
        return println!("----------- Done -----------");
    }

    let mut seed = String::with_capacity(100);
    {
        let mut input = unwrap!(File::open("./input"), "Unable to open input file");
        let _ = unwrap!(input.read_to_string(&mut seed));
    }

    if args.flag_get_pk {
        let sign_pk = unwrap!(seed::sign_pk_from_seed(&seed));
        return println!("Public Signing Key: {:?}", sign_pk.0);
    }

    // Check a single invite.

    if let Some(invite) = args.flag_check_invite {
        let app = unwrap!(App::unregistered(|| (), None,));
        let (tx, rx) = mpsc::channel();

        unwrap!(app.send(move |client, _| {
            let id = XorName(sha3_256(invite.as_bytes()));

            client
                .get_mdata_version(id, INVITE_TOKEN_TYPE_TAG)
                .then(move |res| -> Result<(), ()> {
                    match res {
                        Ok(version) => println!("Invite version: {}", version),
                        Err(e) => println!("Can't find invite: {:?}", e),
                    }
                    unwrap!(tx.send(()));
                    Ok(())
                })
                .map_err(|e| panic!("{:?}", e))
                .into_box()
                .into()
        }));

        unwrap!(rx.recv());
        println!("----------- Done -----------");
        return;
    }

    // Generate invites.

    let output = {
        let name = format!("./output-{}", unwrap!(UNIX_EPOCH.elapsed()).as_secs());
        unwrap!(File::create(&name))
    };

    let flag_create = args.flag_create;

    let auth = unwrap!(if flag_create {
        println!("\nTrying to create an account using given seed from file...");
        Authenticator::create_acc_with_seed(seed.as_str(), || ())
    } else {
        println!("\nTrying to log into the created account using given seed from file...");
        Authenticator::login_with_seed(seed.as_str(), || ())
    });

    println!("Success !");

    let num_invites = args.flag_num_invites.unwrap_or_else(|| {
        // Get number of invites to generate, if not passed in.

        println!("\n----------- Enter number of invitations to generate ---------------");

        let mut num = String::new();
        let _ = std::io::stdin().read_line(&mut num);
        num = num.trim().to_string();
        unwrap!(num.parse::<usize>())
    });

    println!("\nGenerating {} invites...", num_invites);

    for i in 0..num_invites {
        let inv = generate_random_printable(INVITE_TOKEN_SIZE);
        let id = XorName(sha3_256(inv.as_bytes()));

        let perms = btree_map![User::Anyone => PermissionSet::new()
                                       .allow(Action::Insert)
                                       .allow(Action::Update)
                                       .allow(Action::Delete)
                                       .allow(Action::ManagePermissions)];
        let data = btree_map![];
        let (tx, rx) = mpsc::channel();

        unwrap!(auth.send(move |client| {
            let owner_key = unwrap!(client.owner_key());
            let md = unwrap!(MutableData::new(
                id,
                INVITE_TOKEN_TYPE_TAG,
                perms,
                data,
                btree_set![owner_key]
            ));

            client
                .put_mdata(md)
                .then(move |_| {
                    unwrap!(tx.send(()));
                    Ok(())
                })
                .into_box()
                .into()
        }));

        unwrap!(rx.recv());
        let mut output2 = unwrap!(output.try_clone());
        unwrap!(writeln!(output2, "{}", inv));
        println!("Generated invite {} / {}", i + 1, num_invites);
    }

    println!("----------- Done -----------");
}

fn generate_random_printable(len: usize) -> String {
    thread_rng().gen_ascii_chars().take(len).collect()
}
