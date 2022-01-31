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

pub(crate) type Subname = String;

/// An NRS map is a description of a registered topname and all subnames associated with that.
///
/// Each subname will link to some content, e.g., a `FilesContainer`, and the topname can also
/// optionally link to something.
///
/// The struct is stored on the network using a Multimap. The entries are subname -> SafeUrl
/// mappings.
///
/// | Subname Key       | Full Name        | SafeUrl Value            |
/// |-------------------|------------------|--------------------------|
/// | "example"         | "example"        | "safe://example"         |
/// | "sub.example"     | "sub.example"    | "safe://sub.example"     |
/// | "sub.sub.example" | "sub.sub.example"| "safe://sub.sub.example" |
///
/// The map also has a subname version field that optionally specifies a subname at a particular
/// version, since it's possible to have multiple entries for a given subname. If no version was
/// requested when the map is retrieved, it will be set to `None`.
#[derive(Debug, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct NrsMap {
    pub map: BTreeMap<Subname, SafeUrl>,
    pub subname_version: Option<VersionHash>,
}

impl NrsMap {
    /// Get the SafeUrl associated with the given public name.
    pub fn get(&self, public_name: &str) -> Result<SafeUrl> {
        match self.map.get(public_name) {
            Some(link) => {
                debug!("NRS: Subname resolution is: {} => {}", public_name, link);
                Ok(link.to_owned())
            }
            None => {
                debug!("NRS: No link found for subname(s): {}", public_name);
                Err(Error::ContentError(format!(
                    "Link not found in NRS Map Container for subname(s): \"{}\"",
                    public_name
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
