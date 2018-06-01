// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! This build script downloads some diagrams from Cacoo and places them into
//! the docs directory, so they can be linked or embedded in the docs.
//!
//! Currently the diagrams are only downloaded when cargo is run with
//! the `generate-diagrams` feature enabled.

#![allow(unused)]

#[cfg(feature = "generate-diagrams")]
extern crate hyper;

#[cfg(feature = "generate-diagrams")]
mod generate_diagrams {
    use hyper::client::IntoUrl;
    use hyper::Client;
    use std::fs::{self, File};
    use std::io;
    use std::path::{Path, PathBuf};

    pub fn download_image<U: IntoUrl>(name: &str, src: U) {
        download(src, image_path(name))
    }

    fn download<U: IntoUrl, P: AsRef<Path>>(src: U, dst: P) {
        let client = Client::new();
        let mut res = client.get(src).send().unwrap();

        if let Some(dir) = dst.as_ref().parent() {
            fs::create_dir_all(dir).unwrap();
        }

        let mut file = File::create(dst).unwrap();
        io::copy(&mut res, &mut file).unwrap();
    }

    fn image_path(name: &str) -> PathBuf {
        let mut path = PathBuf::from("target/doc/safe_vault");
        path.push(name);
        path.set_extension("png");
        path
    }
}

// Only generate the diagrams when "generate-diagrams" feature is enabled.
// TODO: instead of this feature, detect that cargo is run in the "doc" profile.
#[cfg(feature = "generate-diagrams")]
fn main() {
    // List all diagram names and URLs to download them from.
    generate_diagrams::download_image(
        "personas",
        "https://cacoo.com/diagrams/wl6of3FUFriB0FWO-0BD19.png",
    );
    generate_diagrams::download_image(
        "immutable-data-put-flow",
        "https://cacoo.com/diagrams/SCHrwEhLRB86EGe1-EF9A0.png",
    );
    generate_diagrams::download_image(
        "immutable-data-get-flow",
        "https://cacoo.com/diagrams/ndcPMKC3WapABSaA-EF9A0.png",
    );
}

#[cfg(not(feature = "generate-diagrams"))]
fn main() {}
