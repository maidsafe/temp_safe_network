// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod nrs_map;

use nrs_map::NrsMap;
pub use safe_network::url::{ContentType, VersionHash};

use crate::{app::Safe, register::EntryHash, Error, Result, Url};

use bytes::{Buf, Bytes};
use log::{debug, info, warn};
use std::collections::{BTreeMap, BTreeSet};

// Type tag to use for the NrsMapContainer stored on Register
pub(crate) const NRS_MAP_TYPE_TAG: u64 = 1_500;

impl Safe {
    pub fn parse_url(url: &str) -> Result<Url> {
        let safe_url = Url::from_url(&sanitised_url(url))?;
        Ok(safe_url)
    }

    /// # Creates a nrs_map_container for a chosen top name
    /// Returns a versioned NRS Url (containing a VersionHash) pointing to the provided link
    pub async fn nrs_map_container_create(
        &self,
        top_name: &str,
        link: &Url,
        dry_run: bool,
    ) -> Result<Url> {
        info!("Creating an NRS map for: {}", top_name);

        // Check top_name
        let url_str = validate_nrs_top_name(top_name)?;
        let nrs_xorname = Url::from_nrsurl(&url_str)?.xorname();
        debug!("XorName for \"{:?}\" is \"{:?}\"", &url_str, &nrs_xorname);
        if self.nrs_map_container_get(top_name).await.is_ok() {
            return Err(Error::ContentError("NRS name already exists".to_string()));
        }

        if dry_run {
            return Err(Error::NotImplementedError(
                "No dry run for nrs_map_container_add. Version info cannot be determined. (Register operations need this functionality implemented first.)".to_string(),
            ));
        }

        // Create and store NrsMap for top_name in a Public Blob
        let mut nrs_map = NrsMap::default();
        nrs_map.associate(top_name, link)?;
        debug!("The new NRS Map: {:?}", nrs_map);
        let nrs_map_xorurl = self.store_nrs_map(&nrs_map).await?;

        // Create a new Register and reference the NRS map's blob in the Register
        let reg_xorurl = self
            .register_create(Some(nrs_xorname), NRS_MAP_TYPE_TAG, false)
            .await?;
        let reg_entry = Url::from_xorurl(&nrs_map_xorurl)?;
        let entry_hash = self
            .write_to_register(&reg_xorurl, reg_entry, BTreeSet::new())
            .await?;

        Ok(versionned_nmc_url(url_str, entry_hash)?)
    }

    /// # Associates a public name to a link
    /// The top name of the input public name needs to be registered first with `nrs_map_container_create`
    /// ```no_run
    /// safe://<subName>.<topName>/path/to/whatever?var=value
    ///        |-----------------|
    ///            Public Name
    /// ```
    /// Returns a versioned NRS Url (containing a VersionHash) pointing to the provided link
    pub async fn nrs_map_container_associate(
        &self,
        public_name: &str,
        link: &Url,
        dry_run: bool,
    ) -> Result<Url> {
        info!(
            "Associating public name \"{}\" to \"{}\" in NRS map container",
            public_name, link
        );

        // check public_name
        let url_str = validate_nrs_public_name(public_name)?;

        // fetch and edit nrs_map
        let (mut nrs_map, version) = self.fetch_latest_nrs_map(public_name).await?;
        nrs_map.associate(public_name, link)?;
        debug!("The new NRS Map: {:?}", nrs_map);

        if dry_run {
            return Err(Error::NotImplementedError(
                "No dry run for nrs_map_container_add. Version info cannot be determined. (Register operations need this functionality implemented first.)".to_string(),
            ));
        }

        // store updated nrs_map
        let nrs_map_xorurl = self.store_nrs_map(&nrs_map).await?;
        let old_values: BTreeSet<EntryHash> = [version.entry_hash()].iter().copied().collect();
        let reg_entry = Url::from_xorurl(&nrs_map_xorurl)?;
        let entry_hash = self
            .write_to_register(&url_str, reg_entry, old_values)
            .await?;

        Ok(versionned_nmc_url(url_str, entry_hash)?)
    }

    /// # Removes a public name
    /// The top name of the input public name needs to be registered first with `nrs_map_container_create`
    /// ```no_run
    /// safe://<subName>.<topName>/path/to/whatever?var=value
    ///        |-----------------|
    ///            Public Name
    /// ```
    /// Returns a versioned NRS Url (containing a VersionHash) pointing to the latest version for
    /// the provided public name.
    pub async fn nrs_map_container_remove(&self, public_name: &str, dry_run: bool) -> Result<Url> {
        info!(
            "Removing public name \"{}\" from NRS map container",
            public_name
        );

        // check public_name
        let url_str = validate_nrs_public_name(public_name)?;

        // fetch and edit nrs_map
        let (mut nrs_map, version) = self.fetch_latest_nrs_map(public_name).await?;
        nrs_map.remove(public_name)?;

        if dry_run {
            return Err(Error::NotImplementedError(
                "No dry run for nrs_map_container_remove. Version info cannot be determined. (Register operations need this functionality implemented first.)".to_string(),
            ));
        }

        // store updated nrs_map
        let nrs_map_xorurl = self.store_nrs_map(&nrs_map).await?;
        let old_values: BTreeSet<EntryHash> = [version.entry_hash()].iter().copied().collect();
        let reg_entry = Url::from_xorurl(&nrs_map_xorurl)?;
        let entry_hash = self
            .write_to_register(&url_str, reg_entry, old_values)
            .await?;

        Ok(versionned_nmc_url(url_str, entry_hash)?)
    }

    /// # Gets a public name's associated link
    /// The top name of the input public name needs to be registered first with `nrs_map_container_create`
    /// ```no_run
    /// safe://<subName>.<topName>/path/to/whatever?var=value
    ///        |-----------------|
    ///            Public Name
    /// ```
    /// Returns the associated Url for the given public name.
    pub async fn nrs_map_container_get(&self, public_name: &str) -> Result<Url> {
        info!("Getting link for public name: {}", public_name);

        let (nrs_map, _version) = self.fetch_latest_nrs_map(public_name).await?;
        nrs_map.get(public_name)
    }

    /// Get the mapping of all subNames and their associated Url for the Nrs Map Container at the given public name
    pub async fn nrs_get_subnames_map(&self, public_name: &str) -> Result<BTreeMap<String, Url>> {
        let (nrs_map, _version) = self.fetch_latest_nrs_map(public_name).await?;
        Ok(nrs_map.map)
    }

    // Private helper function to fetch the latest nrs_map from the network
    async fn fetch_latest_nrs_map(&self, public_name: &str) -> Result<(NrsMap, VersionHash)> {
        // fetch latest entries and wrap errors
        let safe_url = Safe::parse_url(public_name)?;
        let entries = self
            .fetch_register_entries(&safe_url)
            .await
            .map_err(|e| match e {
                Error::ContentNotFound(_) => {
                    Error::ContentNotFound("No NRS Map found at this address".to_string())
                }
                Error::VersionNotFound(msg) => Error::VersionNotFound(msg),
                err => Error::NetDataError(format!("Failed to get current version: {}", err)),
            })?;

        // take the 1st entry (TODO Multiple entries)
        if entries.len() > 1 {
            return Err(Error::RegisterFork("Multiple NRS map entries not managed, this happends when 2 clients write concurrently to a NRS map".to_string()));
        }
        let first_entry = entries.iter().next();
        let (version, nrs_map_url) = match first_entry {
            Some((entry_hash, url)) => (entry_hash.into(), url),
            None => {
                warn!(
                    "NRS map Register found at XOR name \"{:?}\" was empty",
                    safe_url.xorname()
                );
                return Ok((NrsMap::default(), VersionHash::default()));
            }
        };

        // Using the NrsMap url we can now fetch the NrsMap and deserialise it
        let serialised_nrs_map = self.fetch_public_data(nrs_map_url, None).await?;

        debug!("Nrs map v{} retrieved: {:?} ", version, &serialised_nrs_map);
        let nrs_map = serde_json::from_str(&String::from_utf8_lossy(serialised_nrs_map.chunk()))
            .map_err(|err| {
                Error::ContentError(format!(
                    "Couldn't deserialise the NrsMap stored in the NrsContainer: {:?}",
                    err
                ))
            })?;

        Ok((nrs_map, version))
    }

    // Private helper to serialise an NrsMap and store it in a Public Blob
    async fn store_nrs_map(&self, nrs_map: &NrsMap) -> Result<String> {
        // The NrsMapContainer is a Register where each NRS Map version is
        // an entry containing the XOR-URL of the Blob that contains the serialised NrsMap.
        let serialised_nrs_map = serde_json::to_string(nrs_map).map_err(|err| {
            Error::Serialisation(format!(
                "Couldn't serialise the NrsMap generated: {:?}",
                err
            ))
        })?;

        let nrs_map_xorurl = self
            .store_public_bytes(Bytes::from(serialised_nrs_map), None, false)
            .await?;

        Ok(nrs_map_xorurl)
    }
}

// Makes a versionned Nrs Map Container Url from a Url and EntryHash
fn versionned_nmc_url(url: String, entry_hash: EntryHash) -> Result<Url> {
    let mut url = Url::from_url(&url)?;
    url.set_content_version(Some(VersionHash::from(&entry_hash)));
    url.set_content_type(ContentType::NrsMapContainer)?;
    Ok(url)
}

// Checks top_name for invalid content, returns a sanitised url: "safe://<top_name>"
fn validate_nrs_top_name(top_name: &str) -> Result<String> {
    let sanitised_url = sanitised_url(top_name);
    let safe_url = Safe::parse_url(&sanitised_url)?;
    if safe_url.top_name() != top_name {
        return Err(Error::InvalidInput(
            format!("The NRS top name \"{}\" is invalid because it contains url parts, please remove any path, version, subnames, etc...", top_name)
        ));
    }
    Ok(sanitised_url)
}

// Checks public_name for invalid content, returns a sanitised url: "safe://<public_name>"
fn validate_nrs_public_name(public_name: &str) -> Result<String> {
    let sanitised_url = sanitised_url(public_name);
    let safe_url = Safe::parse_url(&sanitised_url)?;
    if safe_url.public_name() != public_name {
        return Err(Error::InvalidInput(
            format!("The NRS public name \"{}\" is invalid because it contains url parts, please remove any path, version, etc...", public_name)
        ));
    }
    Ok(sanitised_url)
}

// Makes sure there’s a (and only one) "safe://" in front of input name
fn sanitised_url(name: &str) -> String {
    format!("safe://{}", name.strip_prefix("safe://").unwrap_or(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app::test_helpers::{new_safe_instance, random_nrs_name},
        retry_loop, retry_loop_for_pattern, Url,
    };
    use anyhow::{anyhow, bail, Result};
    use std::str::FromStr;

    #[tokio::test]
    async fn test_nrs_map_container_create() -> Result<()> {
        let site_name = random_nrs_name();
        let safe = new_safe_instance().await?;

        let mut original_url = Url::from_url("safe://linked-from-site_name")?;
        original_url.set_content_version(Some(VersionHash::default()));

        let _nrs_map_url =
            retry_loop!(safe.nrs_map_container_create(&site_name, &original_url, false));

        let retrieved_url = retry_loop!(safe.nrs_map_container_get(&site_name));

        assert_eq!(original_url, retrieved_url);
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_map_container_associate() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link));
        let mut url_v0 = Url::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        let nrs_url =
            retry_loop!(safe.nrs_map_container_create(&site_name, &url_v0, false));

        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        // add subname and set it as the new default too
        let mut url_v1 = Url::from_url(&link)?;
        url_v1.set_content_version(Some(VersionHash::default()));
        let associated_name = format!("a.b.{}", site_name);

        let versionned_url =
            retry_loop!(safe.nrs_map_container_associate(&associated_name, &url_v1, false));

        assert_ne!(versionned_url.content_version(), Some(version0));
        assert_ne!(
            versionned_url.content_version(),
            Some(VersionHash::default())
        );

        // check that the retrieved url matches the expected
        let retrieved_url = retry_loop!(safe.nrs_map_container_get(&associated_name));
        assert_eq!(retrieved_url, url_v1);
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_map_container_add_or_remove_with_versioned_target() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link));
        let mut url_v0 = Url::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        let nrs_url =
            retry_loop!(safe.nrs_map_container_create(&site_name, &url_v0, false));

        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        let dummy_version = "hqt1zg7dwci3ze7dfqp48e3muqt4gkh5wqt1zg7dwci3ze7dfqp4y";
        let versioned_sitename = format!("a.b.{}?v={}", site_name, dummy_version);
        let mut dummy_url_v0 = Url::from_url("safe://linked-from-a_b_site_name")?;
        dummy_url_v0.set_content_version(Some(version0));
        match safe
            .nrs_map_container_associate(&versioned_sitename, &dummy_url_v0, false)
            .await
        {
            Ok(_) => {
                return Err(anyhow!(
                    "NRS map add was unexpectedly successful".to_string(),
                ))
            }
            Err(Error::InvalidInput(msg)) => assert_eq!(
                msg,
                format!(
                    "The NRS public name \"{}\" is invalid because it contains url parts, please remove any path, version, etc...",
                    versioned_sitename
                )
            ),
            other => bail!("Error returned is not the expected one: {:?}", other),
        };

        match safe
            .nrs_map_container_remove(&versioned_sitename, false)
            .await
        {
            Ok(_) => Err(anyhow!(
                "NRS map remove was unexpectedly successful".to_string(),
            )),
            Err(Error::InvalidInput(msg)) => {
                assert_eq!(
                    msg,
                    format!(
                        "The NRS public name \"{}\" is invalid because it contains url parts, please remove any path, version, etc...",
                        versioned_sitename
                    )
                );
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn test_nrs_map_container_remove_one_of_two() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link));

        // associate a first name
        let mut url_v0 = Url::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        let nrs_url =
            retry_loop!(safe.nrs_map_container_create(&site_name, &url_v0, false));
        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        // associate a second name
        let version1 =
            VersionHash::from_str("hqt1zg7dwci3ze7dfqp48e3muqt4gkh5wqt1zg7dwci3ze7dfqp4y")?;
        let mut url_v1 = Url::from_url(&link)?;
        url_v1.set_content_version(Some(version1));
        let associated_name1 = format!("a.b.{}", site_name);

        let versionned_url =
            retry_loop!(safe.nrs_map_container_associate(&associated_name1, &url_v1, false));
        assert!(versionned_url.content_version().is_some());

        // wait for them to be available
        let _ = retry_loop_for_pattern!(safe.nrs_map_container_get(&site_name), Ok(res_url) if res_url == &url_v0)?;
        let _ = retry_loop_for_pattern!(safe.nrs_map_container_get(&associated_name1), Ok(res_url) if res_url == &url_v1)?;

        // remove the first one
        let versionned_url = retry_loop!(safe.nrs_map_container_remove(&site_name, false));
        assert_ne!(versionned_url.content_version(), Some(version1));
        assert_ne!(versionned_url.content_version(), Some(version0));

        // check one is still present
        let _ = retry_loop_for_pattern!(safe.nrs_map_container_get(&associated_name1), Ok(res_url) if res_url == &url_v1)?;

        // check the other is gone
        let expected_err_msg = format!("Link not found in NRS Map Container for: {}", "");
        let _ = retry_loop_for_pattern!(safe.nrs_map_container_get(&site_name), Err(e) if e.to_string() == expected_err_msg)?;

        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_map_container_remove_default_soft_link() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link));
        let mut url_v0 = Url::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        let xorurl = retry_loop!(safe.nrs_map_container_create(
            &site_name,
            &url_v0,
            false,
        ));
        let _ = retry_loop!(safe.fetch(&xorurl.to_string(), None));

        // remove subname
        let versionned_url =
            retry_loop!(safe.nrs_map_container_remove(&site_name, false));
        assert!(versionned_url.content_version().is_some());

        // try to get it again
        let (version, _) = retry_loop!(safe.files_container_get(&link));
        assert_eq!(version, version0);
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_map_container_remove_default_hard_link() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link));
        let mut url_v0 = Url::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        let nrs_url =
            retry_loop!(safe.nrs_map_container_create(&site_name, &url_v0, false,));

        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        // remove subname
        let versionned_url = retry_loop!(safe.nrs_map_container_remove(&site_name, false));
        assert!(versionned_url.content_version().is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_no_scheme() -> Result<()> {
        let site_name = random_nrs_name();
        let url = Safe::parse_url(&site_name)?;
        assert_eq!(url.public_name(), site_name);
        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_topname() -> Result<()> {
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link));
        let mut url_v0 = Url::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        // test with invalid top name
        let invalid_top_name = "atffdgasd/d";
        let expected_err = format!("The NRS top name \"{}\" is invalid because it contains url parts, please remove any path, version, subnames, etc...", invalid_top_name);
        match safe
            .nrs_map_container_create(invalid_top_name, &url_v0, true)
            .await
        {
            Ok(_) => bail!("Unexpected NRS success when expected to fail with invalid top name"),
            Err(Error::InvalidInput(e)) => assert_eq!(e, expected_err),
            Err(_) => bail!("Expected an InvalidInput error kind, got smth else"),
        };
        Ok(())
    }
}
