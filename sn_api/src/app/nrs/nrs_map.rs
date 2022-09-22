// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{Error, Result, SafeUrl};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) type PublicName = String;

/// An NRS map is a description of a registered topname and all subnames associated with that.
///
/// Each subname will link to some content, e.g., a `FilesContainer`, and the topname can also
/// optionally link to something.
///
/// The struct is stored on the network using a Multimap. The entries are public name -> `SafeUrl`
/// mappings.
///
/// | `PublicName` Key    | Full Name        | `SafeUrl` Value            |
/// |-------------------|------------------|--------------------------|
/// | "example"         | "example"        | "safe://example"         |
/// | "sub.example"     | "sub.example"    | "safe://sub.example"     |
/// | "sub.sub.example" | "sub.sub.example"| "safe://sub.sub.example" |
///
/// The map also has a subname version field that optionally specifies a subname at a particular
/// version, since it's possible to have multiple entries for a given subname. If no version was
/// requested when the map is retrieved, it will be set to `None`.
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
pub struct NrsMap {
    pub map: BTreeMap<PublicName, SafeUrl>,
}

impl NrsMap {
    /// Get the `SafeUrl` associated with the given public name.
    ///
    /// There are 3 possible inputs for `public_name`:
    /// * The topname, e.g., "example".
    /// * A subname with a topname, e.g. "a.example".
    /// * An `XorUrl` string.
    ///
    /// The calling `nrs_get` function would have already returned if it couldn't find, say,
    /// "example2".
    ///
    /// If `public_name` isn't in the map, we then check to see if it contains subnames, in which
    /// case, we return a `ContentError`. If it doesn't, we return None. At this point, either the
    /// topname has no link associated, or we have an `XorUrl` string. In both cases, the resolver is
    /// going to return the `NrsMapContainer` content.
    ///
    /// We're doing this because we want to return no target link if the address of the container
    /// has been passed to `nrs_get`.
    pub fn get(&self, public_name: &str) -> Result<Option<SafeUrl>> {
        match self.map.get(public_name) {
            Some(link) => {
                debug!(
                    "NRS: public name resolution is: {} => {}",
                    public_name, link
                );
                Ok(Some(link.clone()))
            }
            None => {
                debug!("NRS: No link found for public name: {}", public_name);
                if self.public_name_contains_subname(public_name) {
                    return Err(Error::ContentError(format!(
                        "Link not found in NRS Map Container for public name: \"{}\"",
                        public_name
                    )));
                }
                Ok(None)
            }
        }
    }

    /// Prints a summary for the NRS map.
    ///
    /// This is used in the CLI for printing out the details of a map.
    ///
    /// It sorts by the length of the subname, so you'd end up with something like this:
    /// * example
    /// * a.example
    /// * a.b.example
    /// * subname.example
    pub fn get_map_summary(&self) -> Vec<(String, String)> {
        let mut v = self
            .map
            .iter()
            .map(|x| (x.0.clone(), x.1.to_string()))
            .collect::<Vec<(String, String)>>();
        v.sort_by(|a, b| a.0.len().cmp(&b.0.len()));
        v
    }

    fn public_name_contains_subname(&self, public_name: &str) -> bool {
        let mut parts = public_name.split('.');
        // pop the topname out.
        parts.next_back();
        let subnames = parts.collect::<Vec<&str>>().join(".");
        !subnames.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SafeUrl;
    use anyhow::{anyhow, Result};
    use assert_matches::assert_matches;

    #[test]
    fn get_should_return_link_for_subname() -> Result<()> {
        let mut nrs_map = NrsMap {
            map: BTreeMap::new(),
        };
        nrs_map
            .map
            .insert("example".to_string(), SafeUrl::from_url("safe://example")?);
        let subname_url = SafeUrl::from_url("safe://a.example")?;
        nrs_map
            .map
            .insert("a.example".to_string(), subname_url.clone());

        let url = nrs_map.get("a.example")?;

        assert_eq!(
            url.ok_or_else(|| anyhow!("url should not be None"))?,
            subname_url
        );
        Ok(())
    }

    #[test]
    fn get_should_return_link_for_multi_subname() -> Result<()> {
        let mut nrs_map = NrsMap {
            map: BTreeMap::new(),
        };
        nrs_map
            .map
            .insert("example".to_string(), SafeUrl::from_url("safe://example")?);
        nrs_map.map.insert(
            "a.example".to_string(),
            SafeUrl::from_url("safe://a.example")?,
        );
        let subname_url = SafeUrl::from_url("safe://a.b.example")?;
        nrs_map
            .map
            .insert("a.b.example".to_string(), subname_url.clone());

        let url = nrs_map.get("a.b.example")?;

        assert_eq!(
            url.ok_or_else(|| anyhow!("url should not be None"))?,
            subname_url
        );
        Ok(())
    }

    #[test]
    fn get_should_return_link_for_topname() -> Result<()> {
        let mut nrs_map = NrsMap {
            map: BTreeMap::new(),
        };
        let topname_url = SafeUrl::from_url("safe://example")?;
        nrs_map
            .map
            .insert("example".to_string(), topname_url.clone());
        nrs_map.map.insert(
            "a.example".to_string(),
            SafeUrl::from_url("safe://a.example")?,
        );

        let url = nrs_map.get("example")?;

        assert_eq!(
            url.ok_or_else(|| anyhow!("url should not be None"))?,
            topname_url
        );
        Ok(())
    }

    #[test]
    fn get_should_return_error_for_non_existent_subname() -> Result<()> {
        let mut nrs_map = NrsMap {
            map: BTreeMap::new(),
        };
        nrs_map
            .map
            .insert("example".to_string(), SafeUrl::from_url("safe://example")?);
        nrs_map.map.insert(
            "a.example".to_string(),
            SafeUrl::from_url("safe://a.example")?,
        );
        nrs_map.map.insert(
            "a.b.example".to_string(),
            SafeUrl::from_url("safe://a.b.example")?,
        );

        assert_matches!(
            nrs_map.get("a.b.c.example"), Err(Error::ContentError(err))
            if err.as_str() == "Link not found in NRS Map Container for public name: \"a.b.c.example\""
        );

        Ok(())
    }

    #[test]
    fn get_should_return_none_for_container_xorurl() -> Result<()> {
        let mut nrs_map = NrsMap {
            map: BTreeMap::new(),
        };
        let topname_url = SafeUrl::from_url("safe://example")?;
        nrs_map
            .map
            .insert("example".to_string(), topname_url.clone());
        nrs_map.map.insert(
            "a.example".to_string(),
            SafeUrl::from_url("safe://a.example")?,
        );
        nrs_map.map.insert(
            "a.b.example".to_string(),
            SafeUrl::from_url("safe://a.b.example")?,
        );

        let container_xorurl = SafeUrl::from_url(&topname_url.to_xorurl_string())?;
        let url = nrs_map.get(container_xorurl.public_name())?;
        assert!(url.is_none());
        Ok(())
    }

    #[test]
    fn get_should_return_none_for_topname_when_topname_has_no_link() -> Result<()> {
        let mut nrs_map = NrsMap {
            map: BTreeMap::new(),
        };
        nrs_map.map.insert(
            "a.example".to_string(),
            SafeUrl::from_url("safe://a.example")?,
        );
        nrs_map.map.insert(
            "a.b.example".to_string(),
            SafeUrl::from_url("safe://a.b.example")?,
        );

        let url = nrs_map.get("example")?;
        assert!(url.is_none());
        Ok(())
    }

    #[test]
    fn get_map_summary_should_return_map_entries() -> Result<()> {
        let mut nrs_map = NrsMap {
            map: BTreeMap::new(),
        };
        let topname_url = SafeUrl::from_url("safe://example")?;
        let a_url = SafeUrl::from_url("safe://a.example")?;
        let a_b_url = SafeUrl::from_url("safe://a.b.example")?;

        nrs_map
            .map
            .insert("example".to_string(), topname_url.clone());
        nrs_map.map.insert("a.example".to_string(), a_url.clone());
        nrs_map
            .map
            .insert("a.b.example".to_string(), a_b_url.clone());

        let summary = nrs_map.get_map_summary();
        assert_eq!(summary.len(), 3);
        assert_eq!(summary[0].0, "example");
        assert_eq!(summary[0].1, topname_url.to_string());
        assert_eq!(summary[1].0, "a.example");
        assert_eq!(summary[1].1, a_url.to_string());
        assert_eq!(summary[2].0, "a.b.example");
        assert_eq!(summary[2].1, a_b_url.to_string());
        Ok(())
    }
}
