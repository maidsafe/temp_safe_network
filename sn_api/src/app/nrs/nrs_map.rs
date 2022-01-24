// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::safeurl::VersionHash;
use crate::{Error, Result, SafeUrl};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) type SubName = String;

/// Mapping Subnames to SafeUrls
/// For a given Top Name : "example"
///
/// | SubName Key   | Full Name        | SafeUrl Value|
/// |---------------|------------------|--------------|
/// | ""            | "example"        | "safe://eg1" |
/// | "sub"         | "sub.example"    | "safe://eg2" |
/// | "sub.sub"     | "sub.sub.example"| "safe://eg3" |
///
/// The map also has a subname_version, which can be used if you want a subname at a particular
/// version.
#[derive(Debug, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct NrsMap {
    pub map: BTreeMap<SubName, SafeUrl>,
    pub subname_version: Option<VersionHash>,
}

impl NrsMap {
    /// Get the SafeUrl associated with the input public name in the NrsMap
    pub fn get(&self, public_name: &str) -> Result<SafeUrl> {
        let subname = parse_out_subnames(public_name);
        self.get_for_subname(&subname)
    }

    /// Get the SafeUrl associated with the input sub name in the NrsMap
    pub fn get_for_subname(&self, sub_name: &str) -> Result<SafeUrl> {
        match self.map.get(sub_name) {
            Some(link) => {
                debug!("NRS: Subname resolution is: {} => {}", sub_name, link);
                Ok(link.to_owned())
            }
            None => {
                debug!("NRS: No link found for subname(s): {}", sub_name);
                Err(Error::ContentError(format!(
                    "Link not found in NRS Map Container for subname(s): \"{}\"",
                    sub_name
                )))
            }
        }
    }

    /// Prints a summary for the NRS map.
    ///
    /// This is used in the CLI for printing out the details of a map.
    /// TODO: remove this placeholder func now that RDF is dropped, fix CLI accordingly
    pub fn get_map_summary(&self) -> BTreeMap<String, BTreeMap<String, String>> {
        BTreeMap::new()
    }
}

/// removes top name from a given name
/// "sub.sub.topname" -> "sub.sub"
/// "sub.cooltopname" -> "sub"
/// "lonetopname" -> ""
pub(super) fn parse_out_subnames(name: &str) -> String {
    let sanitized_name = str::replace(name, "safe://", "");
    let mut parts = sanitized_name.split('.');
    // pop out the topname (last part)
    let _ = parts.next_back();
    parts.collect::<Vec<&str>>().join(".")
}
