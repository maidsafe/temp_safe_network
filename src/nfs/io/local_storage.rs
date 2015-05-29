// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.
use self_encryption;

use std::fmt;
use std::fs;
use std::fs::{File};
use std::io::prelude::*;
use std::path::Path;
use std::string::String;

pub struct LocalStorage {
    pub storage_path : String
}

impl LocalStorage {

    fn to_hex(&self, ch: u8) -> String {
        let hex = fmt::format(format_args!("{:x}", ch));
        if hex.len() == 1 {
            let s = "0".to_string();
            s + &hex
        } else {
            hex
        }
    }

    pub fn file_name(&self, name: &Vec<u8>) -> String {
        let mut string = String::new();
        for i in 0..name.len() {
            string.push_str(&self.to_hex(name[i]));
        }
        string
    }
}

impl self_encryption::Storage for LocalStorage {
    fn get(&self, name: Vec<u8>) -> Vec<u8> {
        let pathstr = self.file_name(&name);
        let tmpname = self.storage_path.clone() + &pathstr;
        let path = Path::new(&tmpname);
        let display = path.display();
        let mut file = match File::open(&path) {
            Err(_) => panic!("couldn't open {}", display),
            Ok(f) => f,
        };
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        data
    }

    fn put(&self, name: Vec<u8>, data: Vec<u8>) {
        let pathstr = self.file_name(&name);
        let tmpname = self.storage_path.clone() + &pathstr;
        let path = Path::new(&tmpname);
        let mut file = match File::create(&path) {
               Err(r) => panic!("couldn't create { } - { }", r, tmpname),
               Ok(f) => f
           };
        match file.write_all(&data[..]) {
                 Err(_) => panic!("couldn't write "),
                 Ok(_) => println!("chunk  written")
            };
    }
}
