// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod nrs_map;

pub use nrs_map::NrsMap;
pub use safe_network::url::{ContentType, VersionHash};

use crate::{app::Safe, register::EntryHash, Error, Result, Url};

use bytes::{Buf, Bytes};
use log::{debug, info, warn};
use std::collections::BTreeSet;

// Type tag to use for the NrsMapContainer stored on Register
pub(crate) const NRS_MAP_TYPE_TAG: u64 = 1_500;

impl Safe {
    pub fn parse_url(url: &str) -> Result<Url> {
        let safe_url = Url::from_url(&sanitised_url(url))?;
        Ok(safe_url)
    }

    /// # Creates a nrs_map_container for a chosen top name
    /// ```no_run
    /// safe://<subName>.<topName>/path/to/whatever?var=value
    ///        |-----------------|
    ///            Public Name
    /// ```
    /// Registers the given NRS top name on the network.
    /// Returns the versioned NRS Url (containing a VersionHash)
    /// `safe://{top_name}?v={version_hash}`
    /// Note that this NRS Url is not linked yet to anything. You just registered the topname here.
    /// You can now associate public_names (with that topname) to links using `nrs_associate` or `nrs_add`
    pub async fn nrs_create(&self, top_name: &str, dry_run: bool) -> Result<Url> {
        info!("Creating an NRS map for: {}", top_name);

        // Check top_name, check if there is an NrsMapContainer there already
        let url_str = validate_nrs_top_name(top_name)?;
        let nrs_xorname = Url::from_nrsurl(&url_str)?.xorname();
        debug!("XorName for \"{:?}\" is \"{:?}\"", &url_str, &nrs_xorname);
        if self.nrs_get_subnames_map(top_name, None).await.is_ok() {
            return Err(Error::NrsNameAlreadyExists(top_name.to_owned()));
        }

        if dry_run {
            return Err(Error::NotImplementedError(
                "No dry run for nrs_create. Version info cannot be determined. (Register operations need this functionality implemented first.)".to_string(),
            ));
        }

        // Create and store NrsMap for top_name in a Public Blob
        let nrs_map = NrsMap::default();
        let nrs_map_xorurl = self.store_nrs_map(&nrs_map).await?;

        // Create a new Register and reference the NRS map's blob in the Register
        let reg_xorurl = self
            .register_create(Some(nrs_xorname), NRS_MAP_TYPE_TAG, false)
            .await?;
        let reg_entry = Url::from_xorurl(&nrs_map_xorurl)?;
        // Note that we can use the higher level register API here
        // because reg_xorurl is not an NRS url
        // (high level register API uses resolution that uses NRS!)
        let entry_hash = self
            .write_to_register(&reg_xorurl, reg_entry, BTreeSet::new())
            .await?;

        Ok(get_versioned_nrs_url(url_str, entry_hash)?)
    }

    /// # Associates a public name to a link
    /// The top name of the input public name needs to be registered first with `nrs_create`
    /// ```no_run
    /// safe://<subName>.<topName>/path/to/whatever?var=value
    ///        |-----------------|
    ///            Public Name
    /// ```
    /// Associates the given public_name to the link, stores it in the NrsMap registered for the
    /// public name's top name on the network.
    /// Errors out if the topname is not registered.
    /// Returns the versioned NRS Url (containing a VersionHash) now pointing to the provided link:
    /// `safe://{public_name}?v={version_hash}`
    pub async fn nrs_associate(&self, public_name: &str, link: &Url, dry_run: bool) -> Result<Url> {
        info!(
            "Associating public name \"{}\" to \"{}\" in NRS map container",
            public_name, link
        );

        // check public_name
        let url_str = validate_nrs_public_name(public_name)?;

        // fetch and edit nrs_map
        let (mut nrs_map, version) = self.fetch_nrs_map(public_name, None).await?;
        nrs_map.associate(public_name, link)?;
        debug!("The new NRS Map: {:?}", nrs_map);

        if dry_run {
            return Err(Error::NotImplementedError(
                "No dry run for nrs_add. Version info cannot be determined. (Register operations need this functionality implemented first.)".to_string(),
            ));
        }

        // store updated nrs_map
        let nrs_map_xorurl = self.store_nrs_map(&nrs_map).await?;
        let old_values: BTreeSet<EntryHash> = [version.entry_hash()].iter().copied().collect();
        let reg_entry = Url::from_xorurl(&nrs_map_xorurl)?;
        let safe_url = Safe::parse_url(&url_str)?;
        let address = self.get_register_address(&safe_url)?;
        let entry_hash = self
            .safe_client
            .write_to_register(address, reg_entry, old_values)
            .await?;

        Ok(get_versioned_nrs_url(url_str, entry_hash)?)
    }

    /// # Associates any public name to a link
    ///
    /// Associates the given public_name to the link registering the topname on the way if needed.
    /// Returns the versioned NRS Url (containing a VersionHash) now pointing to the provided link:
    /// `safe://{public_name}?v={version_hash}`
    /// Also returns a bool to indicate whether it registered the topname in the process or not.
    pub async fn nrs_add(
        &self,
        public_name: &str,
        link: &Url,
        dry_run: bool,
    ) -> Result<(Url, bool)> {
        info!(
            "Adding public name \"{}\" to \"{}\" in an NRS map container",
            public_name, link
        );

        // make sure the topname is registered
        let sanitised_url = sanitised_url(public_name);
        let safe_url = Safe::parse_url(&sanitised_url)?;
        let top_name = safe_url.top_name();
        let creation_result = self.nrs_create(top_name, dry_run).await;
        let did_register_topname = match creation_result {
            Ok(_) => Ok(true),
            Err(Error::NrsNameAlreadyExists(_)) => Ok(false),
            Err(e) => Err(e),
        }?;

        // associate with peace of mind
        let new_url = self.nrs_associate(public_name, link, dry_run).await?;
        Ok((new_url, did_register_topname))
    }

    /// # Removes a public name
    /// The top name of the input public name needs to be registered first with `nrs_create`
    /// ```no_run
    /// safe://<subName>.<topName>/path/to/whatever?var=value
    ///        |-----------------|
    ///            Public Name
    /// ```
    /// Removes the given public_name from the NrsMap registered for the public name's top name
    /// on the network.
    /// Returns a versioned NRS Url (containing a VersionHash) pointing to the latest version
    /// (including the deletion) for the provided public name.
    /// `safe://{public_name}?v={version_hash}`
    pub async fn nrs_remove(&self, public_name: &str, dry_run: bool) -> Result<Url> {
        info!(
            "Removing public name \"{}\" from NRS map container",
            public_name
        );

        // check public_name
        let url_str = validate_nrs_public_name(public_name)?;

        // fetch and edit nrs_map
        let (mut nrs_map, version) = self.fetch_nrs_map(public_name, None).await?;
        nrs_map.remove(public_name)?;

        if dry_run {
            return Err(Error::NotImplementedError(
                "No dry run for nrs_remove. Version info cannot be determined. (Register operations need this functionality implemented first.)".to_string(),
            ));
        }

        // store updated nrs_map
        let nrs_map_xorurl = self.store_nrs_map(&nrs_map).await?;
        let old_values: BTreeSet<EntryHash> = [version.entry_hash()].iter().copied().collect();
        let reg_entry = Url::from_xorurl(&nrs_map_xorurl)?;
        let safe_url = Safe::parse_url(&url_str)?;
        let address = self.get_register_address(&safe_url)?;
        let entry_hash = self
            .safe_client
            .write_to_register(address, reg_entry, old_values)
            .await?;

        Ok(get_versioned_nrs_url(url_str, entry_hash)?)
    }

    /// # Gets a public name's associated link
    /// If no version is specified, returns the latest.
    /// The top name of the input public name needs to be registered first with `nrs_create`
    /// ```no_run
    /// safe://<subName>.<topName>/path/to/whatever?var=value
    ///        |-----------------|
    ///            Public Name
    /// ```
    /// Finds the Url associated with the given public name on the network.
    /// Returns the associated Url for the given public name for that version along with the NrsMap
    pub async fn nrs_get(
        &self,
        public_name: &str,
        version: Option<VersionHash>,
    ) -> Result<(Url, NrsMap)> {
        info!(
            "Getting link for public name: {} for version: {:?}",
            public_name, version
        );

        let (nrs_map, _version) = self.fetch_nrs_map(public_name, version).await?;
        let url = nrs_map.get(public_name)?;
        Ok((url, nrs_map))
    }

    /// Get the mapping of all subNames and their associated Url for the Nrs Map Container at the given public name
    pub async fn nrs_get_subnames_map(
        &self,
        public_name: &str,
        version: Option<VersionHash>,
    ) -> Result<NrsMap> {
        let (nrs_map, _version) = self.fetch_nrs_map(public_name, version).await?;
        Ok(nrs_map)
    }

    // Private helper function to fetch the nrs_map from the network
    // If no version is provided, fetches the latest
    // Always returns the version of the fetched content along with the NrsMap
    async fn fetch_nrs_map(
        &self,
        public_name: &str,
        version: Option<VersionHash>,
    ) -> Result<(NrsMap, VersionHash)> {
        // assign version
        let mut safe_url = Safe::parse_url(public_name)?;
        safe_url.set_content_version(version);
        let safe_url = safe_url;

        // fetch entries and wrap errors
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
fn get_versioned_nrs_url(url: String, entry_hash: EntryHash) -> Result<Url> {
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
    async fn test_nrs_create() -> Result<()> {
        let site_name = random_nrs_name();
        let safe = new_safe_instance().await?;

        let mut original_url = Url::from_url("safe://linked-from-site_name")?;
        original_url.set_content_version(Some(VersionHash::default()));

        let _nrs_map_url = retry_loop!(safe.nrs_create(&site_name, false));

        let nrs_map = retry_loop!(safe.nrs_get_subnames_map(&site_name, None));

        assert_eq!(nrs_map.map.len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_associate() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link));
        let mut url_v0 = Url::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        let (nrs_url, did_create) = retry_loop!(safe.nrs_add(&site_name, &url_v0, false));
        assert!(did_create);

        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        // add subname and set it as the new default too
        let mut url_v1 = Url::from_url(&link)?;
        url_v1.set_content_version(Some(VersionHash::default()));
        let associated_name = format!("a.b.{}", site_name);

        let versionned_url = retry_loop!(safe.nrs_associate(&associated_name, &url_v1, false));

        assert_ne!(versionned_url.content_version(), Some(version0));
        assert_ne!(
            versionned_url.content_version(),
            Some(VersionHash::default())
        );

        // check that the retrieved url matches the expected
        let (retrieved_url, nrs_map) = retry_loop!(safe.nrs_get(&associated_name, None));
        assert_eq!(nrs_map.map.len(), 2);
        assert_eq!(retrieved_url, url_v1);
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_add_or_remove_with_versioned_target() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link));
        let mut url_v0 = Url::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        let (nrs_url, did_create) = retry_loop!(safe.nrs_add(&site_name, &url_v0, false));
        assert!(did_create);

        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        let dummy_version = "hqt1zg7dwci3ze7dfqp48e3muqt4gkh5wqt1zg7dwci3ze7dfqp4y";
        let versioned_sitename = format!("a.b.{}?v={}", site_name, dummy_version);
        let mut dummy_url_v0 = Url::from_url("safe://linked-from-a_b_site_name")?;
        dummy_url_v0.set_content_version(Some(version0));
        match safe
            .nrs_associate(&versioned_sitename, &dummy_url_v0, false)
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

        match safe.nrs_remove(&versioned_sitename, false).await {
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
    async fn test_nrs_remove_one_of_two() -> Result<()> {
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

        let (nrs_url, did_create) = retry_loop!(safe.nrs_add(&site_name, &url_v0, false));
        assert!(did_create);

        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        // associate a second name
        let version1 =
            VersionHash::from_str("hqt1zg7dwci3ze7dfqp48e3muqt4gkh5wqt1zg7dwci3ze7dfqp4y")?;
        let mut url_v1 = Url::from_url(&link)?;
        url_v1.set_content_version(Some(version1));
        let associated_name1 = format!("a.b.{}", site_name);

        let versionned_url = retry_loop!(safe.nrs_associate(&associated_name1, &url_v1, false));
        assert!(versionned_url.content_version().is_some());

        // wait for them to be available
        let _ = retry_loop_for_pattern!(safe.nrs_get(&site_name, None), Ok((res_url, _)) if res_url == &url_v0)?;
        let _ = retry_loop_for_pattern!(safe.nrs_get(&associated_name1, None), Ok((res_url, _)) if res_url == &url_v1)?;

        // remove the first one
        let versionned_url = retry_loop!(safe.nrs_remove(&site_name, false));
        assert_ne!(versionned_url.content_version(), Some(version1));
        assert_ne!(versionned_url.content_version(), Some(version0));

        // check one is still present
        let _ = retry_loop_for_pattern!(safe.nrs_get(&associated_name1, None), Ok((res_url, _)) if res_url == &url_v1)?;

        // check the other is gone
        let expected_err_msg = format!("Link not found in NRS Map Container for: {}", "");
        let _ = retry_loop_for_pattern!(safe.nrs_get(&site_name, None), Err(e) if e.to_string() == expected_err_msg)?;

        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_remove_default() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link));
        let mut url_v0 = Url::from_url(&link)?;
        url_v0.set_content_version(Some(version0));
        let (nrs_url, did_create) = retry_loop!(safe.nrs_add(&site_name, &url_v0, false,));
        assert!(did_create);

        // check it's there
        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));
        let nrs_map = safe.nrs_get_subnames_map(&site_name, None).await?;
        assert!(nrs_map.map.len() == 1);

        // remove subname
        let versionned_url = retry_loop!(safe.nrs_remove(&site_name, false));
        assert!(versionned_url.content_version().is_some());

        // check it's gone
        let _ = retry_loop_for_pattern!(safe.nrs_get_subnames_map(&site_name, None), Ok(nrs_map) if nrs_map.map.is_empty());

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
        let safe = new_safe_instance().await?;

        // test with invalid top name
        let invalid_top_name = "atffdgasd/d";
        let expected_err = format!("The NRS top name \"{}\" is invalid because it contains url parts, please remove any path, version, subnames, etc...", invalid_top_name);
        match safe.nrs_create(invalid_top_name, true).await {
            Ok(_) => bail!("Unexpected NRS success when expected to fail with invalid top name"),
            Err(Error::InvalidInput(e)) => assert_eq!(e, expected_err),
            Err(_) => bail!("Expected an InvalidInput error kind, got smth else"),
        };
        Ok(())
    }
}
