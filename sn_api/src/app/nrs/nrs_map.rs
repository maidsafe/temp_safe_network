// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{
    app::resolver::{ContentType, DataType},
    Error, Result, Url,
};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) type SubName = String;

/// Mapping SubNames to Urls
/// For a given Top Name : "example"
///
/// | SubName Key   | Full Name        | Url Value    |
/// |---------------|------------------|--------------|
/// | ""            | "example"        | "safe://eg1" |
/// | "sub"         | "sub.example"    | "safe://eg2" |
/// | "sub.sub"     | "sub.sub.example"| "safe://eg3" |
///
#[derive(Debug, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct NrsMap {
    pub map: BTreeMap<SubName, Url>,
}

impl NrsMap {
    /// Associates a link with a public name in the NrsMap
    pub fn associate(&mut self, public_name: &str, link: &Url) -> Result<String> {
        info!("Updating NRS map for: {}", public_name);
        // NRS resolver doesn't allow unversioned links
        validate_nrs_url(link)?;
        let subname = parse_out_subnames(public_name);
        self.map.insert(subname, link.to_owned());
        Ok(link.to_string())
    }

    /// Get the Url associated with the input public name in the NrsMap
    pub fn get(&self, public_name: &str) -> Result<Url> {
        let subname = parse_out_subnames(public_name);
        self.get_for_subname(&subname)
    }

    /// Get the Url associated with the input sub name in the NrsMap
    pub fn get_for_subname(&self, sub_name: &str) -> Result<Url> {
        match self.map.get(sub_name) {
            Some(link) => {
                debug!("NRS: Subname resolution is: {} => {}", sub_name, link);
                Ok(link.to_owned())
            }
            None => {
                debug!("NRS: No link found for subname(s): {}", sub_name);
                Err(Error::ContentError(format!(
                    "Link not found in NRS Map Container for: {}",
                    sub_name
                )))
            }
        }
    }

    /// Remove a public name from the NrsMap
    pub fn remove(&mut self, public_name: &str) -> Result<()> {
        info!("Removing public name from NRS map: {}", public_name);
        let sub_name = parse_out_subnames(public_name);
        match self.map.remove(&sub_name) {
            Some(_link) => Ok(()),
            None => Err(Error::ContentError(
                "Sub name not found in NRS Map Container".to_string(),
            )),
        }
    }
}

/// removes top name from a given name
/// "sub.sub.topname" -> "sub.sub"
/// "sub.cooltopname" -> "sub"
/// "lonetopname" -> ""
fn parse_out_subnames(name: &str) -> String {
    let sanitized_name = str::replace(name, "safe://", "");
    let mut parts = sanitized_name.split('.');
    // pop out the topname (last part)
    let _ = parts.next_back();
    parts.collect::<Vec<&str>>().join(".")
}

// helper function to check a xorurl used for NRS
// - checks if the url is valid
// - checks if it has a version if its data is versionable
fn validate_nrs_url(link: &Url) -> Result<()> {
    if link.content_version().is_none() {
        let content_type = link.content_type();
        let data_type = link.data_type();
        if content_type == ContentType::FilesContainer
            || content_type == ContentType::NrsMapContainer
        {
            return Err(Error::InvalidInput(format!(
                "The linked content ({}) is versionable, therefore NRS requires the link to specify a hash: {}",
                content_type, link.to_string()
            )));
        } else if data_type == DataType::Register {
            return Err(Error::InvalidInput(format!(
                "The linked content ({}) is versionable, therefore NRS requires the link to specify a hash: {}",
                data_type, link.to_string()
            )));
        }
    }

    Ok(())
}
