// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod nrs_map;

pub use crate::safeurl::{ContentType, DataType, VersionHash};
pub use nrs_map::NrsMap;

use nrs_map::parse_out_subnames;

use crate::{app::Safe, register::EntryHash, Error, Result, SafeUrl};

use log::{debug, info};
use std::collections::{BTreeMap, BTreeSet};
use std::str;

// Type tag to use for the NrsMapContainer stored on Register
pub(crate) const NRS_MAP_TYPE_TAG: u64 = 1_500;

impl Safe {
    pub fn parse_url(url: &str) -> Result<SafeUrl> {
        let safe_url = SafeUrl::from_url(&sanitised_url(url))?;
        Ok(safe_url)
    }

    /// # Creates a nrs_map_container for a chosen top name
    /// ```no_run
    /// safe://<subName>.<topName>/path/to/whatever?var=value
    ///        |-----------------|
    ///            Public Name
    /// ```
    /// Registers the given NRS top name on the network.
    /// Returns the NRS SafeUrl: `safe://{top_name}
    /// Note that this NRS SafeUrl is not linked to anything yet. You just registered the topname here.
    /// You can now associate public_names (with that topname) to links using `nrs_associate` or `nrs_add`
    pub async fn nrs_create(&self, top_name: &str) -> Result<SafeUrl> {
        info!("Creating an NRS map for: {}", top_name);

        // Check top_name, check if there is an NrsMapContainer there already
        let url_str = validate_nrs_top_name(top_name)?;
        let nrs_xorname = SafeUrl::from_nrsurl(&url_str)?.xorname();
        debug!("XorName for \"{:?}\" is \"{:?}\"", &url_str, &nrs_xorname);
        if self.nrs_get_subnames_map(top_name, None).await.is_ok() {
            return Err(Error::NrsNameAlreadyExists(top_name.to_owned()));
        }

        // Create a new empty Multimap
        let _ = self
            .multimap_create(Some(nrs_xorname), NRS_MAP_TYPE_TAG, false)
            .await?;

        let mut url = SafeUrl::from_url(&url_str)?;
        url.set_content_type(ContentType::NrsMapContainer)?;
        Ok(url)
    }

    /// # Associates a public name to a link
    /// The top name of the input public name needs to be registered first with `nrs_create`
    ///
    /// ```no_run
    /// safe://<subName>.<topName>/path/to/whatever?var=value
    ///        |-----------------|
    ///            Public Name
    /// ```
    /// Associates the given public_name to the link.
    /// Errors out if the topname is not registered.
    /// Returns the versioned NRS SafeUrl (containing a VersionHash) now pointing to the provided link:
    /// `safe://{public_name}?v={version_hash}`
    pub async fn nrs_associate(&self, public_name: &str, link: &SafeUrl) -> Result<SafeUrl> {
        info!(
            "Associating public name \"{}\" to \"{}\" in NRS map container",
            public_name, link
        );

        // check input
        let url_str = validate_nrs_public_name(public_name)?;
        validate_nrs_url(link)?;

        // get current latest for subname
        let safe_url = Safe::parse_url(public_name)?;
        let subname = parse_out_subnames(public_name);
        let current_versions = self
            .fetch_multimap_value_by_key(&safe_url, subname.as_bytes())
            .await?
            .into_iter()
            .map(|(hash, _)| hash)
            .collect();

        // update with new entry
        let entry = (
            subname.as_bytes().to_vec(),
            link.to_string().as_bytes().to_vec(),
        );
        let entry_hash = self
            .multimap_insert(&url_str, entry, current_versions)
            .await?;

        Ok(get_versioned_nrs_url(url_str, entry_hash)?)
    }

    /// # Associates any public name to a link
    ///
    /// Associates the given public_name to the link registering the topname on the way if needed.
    /// Returns the versioned NRS SafeUrl (containing a VersionHash) now pointing to the provided link:
    /// `safe://{public_name}?v={version_hash}`
    /// Also returns a bool to indicate whether it registered the topname in the process or not.
    pub async fn nrs_add(&self, public_name: &str, link: &SafeUrl) -> Result<(SafeUrl, bool)> {
        info!(
            "Adding public name \"{}\" to \"{}\" in an NRS map container",
            public_name, link
        );

        // make sure the topname is registered
        let sanitised_url = sanitised_url(public_name);
        let safe_url = Safe::parse_url(&sanitised_url)?;
        let top_name = safe_url.top_name();
        let creation_result = self.nrs_create(top_name).await;
        let did_register_topname = match creation_result {
            Ok(_) => Ok(true),
            Err(Error::NrsNameAlreadyExists(_)) => Ok(false),
            Err(e) => Err(e),
        }?;

        // associate with peace of mind
        let new_url = self.nrs_associate(public_name, link).await?;
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
    /// Returns a versioned NRS SafeUrl (containing a VersionHash) pointing to the latest version
    /// (including the deletion) for the provided public name.
    /// `safe://{public_name}?v={version_hash}`
    pub async fn nrs_remove(&self, public_name: &str) -> Result<SafeUrl> {
        info!(
            "Removing public name \"{}\" from NRS map container",
            public_name
        );

        // check public_name
        let url_str = validate_nrs_public_name(public_name)?;

        // get current latest for subname
        let safe_url = Safe::parse_url(public_name)?;
        let subname = parse_out_subnames(public_name);
        let current_versions = self
            .fetch_multimap_value_by_key(&safe_url, subname.as_bytes())
            .await?
            .into_iter()
            .map(|(hash, _)| hash)
            .collect();

        // remove
        let entry_hash = self.multimap_remove(&url_str, current_versions).await?;

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
    /// Finds the SafeUrl associated with the given public name on the network.
    /// If multiple entries are found for the same public name, there's a conflict.
    /// If there are conflicts for subnames other than the one requested, get proceeds as usual,
    /// but the NrsMap returned will ignore those conflicts.
    /// Otherwise, it returns an error.
    /// Returns the associated SafeUrl for the given public name for that version along with an NrsMap
    pub async fn nrs_get(
        &self,
        public_name: &str,
        version: Option<VersionHash>,
    ) -> Result<(SafeUrl, NrsMap)> {
        info!(
            "Getting link for public name: {} for version: {:?}",
            public_name, version
        );

        // get nrs_map, ignoring conflicting entries if they are not the ones we're getting
        let nrs_map = match self.nrs_get_subnames_map(public_name, version).await {
            Ok(map) => Ok(map),
            Err(Error::ConflictingNrsEntries(str, conflicting_entries, map)) => {
                let subname = parse_out_subnames(public_name);
                if conflicting_entries.iter().any(|(sub, _)| sub == &subname) {
                    Err(Error::ConflictingNrsEntries(str, conflicting_entries, map))
                } else {
                    Ok(map)
                }
            }
            Err(e) => Err(e),
        }?;

        let url = nrs_map.get(public_name)?;
        Ok((url, nrs_map))
    }

    /// Get the mapping of all subNames and their associated SafeUrl for the Nrs Map Container at the given public name
    pub async fn nrs_get_subnames_map(
        &self,
        public_name: &str,
        version: Option<VersionHash>,
    ) -> Result<NrsMap> {
        // fetch multimap entries
        let safe_url = Safe::parse_url(public_name)?;
        let res = self.fetch_multimap_values(&safe_url).await;
        let multimap_keyvals = match res {
            Ok(s) => Ok(s),
            Err(Error::EmptyContent(_)) => Ok(BTreeSet::new()),
            Err(Error::ContentNotFound(e)) => Err(Error::ContentNotFound(format!(
                "No Nrs Map entry found at {}: {}",
                safe_url, e
            ))),
            Err(e) => Err(Error::NetDataError(format!(
                "Failed to get Nrs Map entries: {}",
                e
            ))),
        }?;

        // collect a raw map with serialized data, get specific version if needed
        let raw_set = match version {
            Some(v) => {
                let hash = v.entry_hash();
                if multimap_keyvals
                    .iter()
                    .any(|(h, _)| VersionHash::from(h) == v)
                {
                    // just return the key val set if we have the version we're looking for
                    multimap_keyvals
                        .into_iter()
                        .map(|(_hash, key_val)| key_val)
                        .collect()
                } else {
                    // manually fetch the missing versionned entry
                    let mut key_vals: BTreeSet<_> = multimap_keyvals
                        .into_iter()
                        .map(|(_hash, key_val)| key_val)
                        .collect();
                    let key_val = self.fetch_multimap_value_by_hash(&safe_url, hash).await?;
                    let fetched_key = &key_val.0;

                    // remove other entries with the same key
                    key_vals.retain(|(k, _)| k != fetched_key);

                    // insert the versioned entry
                    key_vals.insert(key_val);
                    key_vals
                }
            }
            None => multimap_keyvals
                .into_iter()
                .map(|(_hash, key_val)| key_val)
                .collect(),
        };

        // deserialize
        let clean_set: BTreeSet<(String, SafeUrl)> = raw_set
            .into_iter()
            .map(|(subname_bytes, url_bytes)| {
                let subname = str::from_utf8(&subname_bytes)?;
                let url = SafeUrl::from_url(str::from_utf8(&url_bytes)?)?;
                Ok((subname.to_owned(), url))
            })
            .collect::<Result<BTreeSet<(String, SafeUrl)>>>()?;

        // turn into map
        let subnames_map: BTreeMap<String, SafeUrl> = clean_set.clone().into_iter().collect();
        let nrs_map = NrsMap {
            map: subnames_map.clone(),
        };

        // check for conflicting entries (same subname, different url)
        let set_len = clean_set.len();
        let map_len = subnames_map.len();
        if map_len != set_len {
            let set_from_map: BTreeSet<(String, SafeUrl)> = subnames_map.into_iter().collect();
            let conflicting_entries: Vec<(String, SafeUrl)> =
                clean_set.difference(&set_from_map).cloned().collect();
            return Err(Error::ConflictingNrsEntries("Found multiple entries for the same name, this happens when 2 clients write concurrently to the same NRS mapping. It can be fixed by simply associating a new link to the conflicting names.".to_string(), conflicting_entries, nrs_map));
        }

        Ok(nrs_map)
    }
}

// Makes a versionned Nrs Map Container SafeUrl from a SafeUrl and EntryHash
fn get_versioned_nrs_url(url: String, entry_hash: EntryHash) -> Result<SafeUrl> {
    let mut url = SafeUrl::from_url(&url)?;
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
            format!("The NRS top name \"{}\" is invalid because it contains url parts. Please remove any path, version or subnames.", top_name)
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
            format!("The NRS public name \"{}\" is invalid because it contains url parts. Please remove any path or version.", public_name)
        ));
    }
    Ok(sanitised_url)
}

// helper function to check a SafeUrl used for NRS
// - checks if the url is valid
// - checks if it has a version if its data is versionable
fn validate_nrs_url(link: &SafeUrl) -> Result<()> {
    if link.content_version().is_none() {
        let content_type = link.content_type();
        let data_type = link.data_type();
        if content_type == ContentType::FilesContainer
            || content_type == ContentType::NrsMapContainer
        {
            return Err(Error::UnversionedContentError(format!(
                "The linked content ({}) is versionable, therefore NRS requires the link to specify a hash: {}",
                content_type, link
            )));
        } else if data_type == DataType::Register {
            return Err(Error::UnversionedContentError(format!(
                "The linked content ({}) is versionable, therefore NRS requires the link to specify a hash: {}",
                data_type, link
            )));
        }
    }

    Ok(())
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
        retry_loop, retry_loop_for_pattern, Error, SafeUrl,
    };
    use anyhow::{anyhow, bail, Result};
    use rand::Rng;
    use std::{matches, str::FromStr};

    const TEST_DATA_FILE: &str = "./testdata/test.md";

    #[tokio::test]
    async fn test_nrs_create() -> Result<()> {
        let site_name = random_nrs_name();
        let safe = new_safe_instance().await?;

        let mut original_url = SafeUrl::from_url("safe://linked-from-site_name")?;
        let random_hash = EntryHash(rand::thread_rng().gen::<[u8; 32]>());
        let version_hash = VersionHash::from(&random_hash);
        original_url.set_content_version(Some(version_hash));

        let _nrs_map_url = retry_loop!(safe.nrs_create(&site_name));

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
            .files_container_create_from(TEST_DATA_FILE, None, false, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link))
            .ok_or(anyhow!("files container was unexpectedly empty"))?;

        let mut url_v0 = SafeUrl::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        let (nrs_url, did_create) = retry_loop!(safe.nrs_add(&site_name, &url_v0));
        assert!(did_create);

        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        // add subname and set it as the new default too
        let random_hash = EntryHash(rand::thread_rng().gen::<[u8; 32]>());
        let version_hash = VersionHash::from(&random_hash);
        let mut url_v1 = SafeUrl::from_url(&link)?;
        url_v1.set_content_version(Some(version_hash));
        let associated_name = format!("a.b.{}", site_name);

        let versionned_url = retry_loop!(safe.nrs_associate(&associated_name, &url_v1));

        assert_ne!(versionned_url.content_version(), Some(version0));

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
            .files_container_create_from(TEST_DATA_FILE, None, false, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link))
            .ok_or(anyhow!("files container was unexpectedly empty"))?;

        let mut url_v0 = SafeUrl::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        let (nrs_url, did_create) = retry_loop!(safe.nrs_add(&site_name, &url_v0));
        assert!(did_create);

        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        let dummy_version = "hqt1zg7dwci3ze7dfqp48e3muqt4gkh5wqt1zg7dwci3ze7dfqp4y";
        let versioned_sitename = format!("a.b.{}?v={}", site_name, dummy_version);
        let mut dummy_url_v0 = SafeUrl::from_url("safe://linked-from-a_b_site_name")?;
        dummy_url_v0.set_content_version(Some(version0));
        match safe
            .nrs_associate(&versioned_sitename, &dummy_url_v0)
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
                    "The NRS public name \"{}\" is invalid because it contains url parts. Please remove any path or version.",
                    versioned_sitename
                )
            ),
            other => bail!("Error returned is not the expected one: {:?}", other),
        };

        match safe.nrs_remove(&versioned_sitename).await {
            Ok(_) => Err(anyhow!(
                "NRS map remove was unexpectedly successful".to_string(),
            )),
            Err(Error::InvalidInput(msg)) => {
                assert_eq!(
                    msg,
                    format!(
                        "The NRS public name \"{}\" is invalid because it contains url parts. Please remove any path or version.",
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
            .files_container_create_from(TEST_DATA_FILE, None, false, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link))
            .ok_or(anyhow!("files container was unexpectedly empty"))?;

        // associate a first name
        let mut url_v0 = SafeUrl::from_url(&link)?;
        url_v0.set_content_version(Some(version0));

        let (nrs_url, did_create) = retry_loop!(safe.nrs_add(&site_name, &url_v0));
        assert!(did_create);

        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        // associate a second name
        let version1 =
            VersionHash::from_str("hqt1zg7dwci3ze7dfqp48e3muqt4gkh5wqt1zg7dwci3ze7dfqp4y")?;
        let mut url_v1 = SafeUrl::from_url(&link)?;
        url_v1.set_content_version(Some(version1));
        let associated_name1 = format!("a.b.{}", site_name);

        let versionned_url = retry_loop!(safe.nrs_associate(&associated_name1, &url_v1));
        assert!(versionned_url.content_version().is_some());

        // wait for them to be available
        let _ = retry_loop_for_pattern!(safe.nrs_get(&site_name, None), Ok((res_url, _)) if res_url == &url_v0)?;
        let _ = retry_loop_for_pattern!(safe.nrs_get(&associated_name1, None), Ok((res_url, _)) if res_url == &url_v1)?;

        // remove the first one
        let versionned_url = retry_loop!(safe.nrs_remove(&site_name));
        assert_ne!(versionned_url.content_version(), Some(version1));
        assert_ne!(versionned_url.content_version(), Some(version0));

        // check one is still present
        let _ = retry_loop_for_pattern!(safe.nrs_get(&associated_name1, None), Ok((res_url, _)) if res_url == &url_v1)?;

        // check the other is gone
        let expected_err_msg = "Link not found in NRS Map Container";
        let result = retry_loop_for_pattern!(safe.nrs_get(&site_name, None), Err(e) if e.to_string().contains(expected_err_msg));
        if let Err(err) = result {
            assert!(err.to_string().contains(expected_err_msg));
        } else {
            bail!(
                "Got unexpected result when expecting an error containing \"{}\": The result: {:?}",
                expected_err_msg,
                result
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_remove_default() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create_from(TEST_DATA_FILE, None, false, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link))
            .ok_or(anyhow!("files container was unexpectedly empty"))?;

        let mut url_v0 = SafeUrl::from_url(&link)?;
        url_v0.set_content_version(Some(version0));
        let (nrs_url, did_create) = retry_loop!(safe.nrs_add(&site_name, &url_v0));
        assert!(did_create);

        // check it's there
        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));
        let nrs_map = safe.nrs_get_subnames_map(&site_name, None).await?;
        assert!(nrs_map.map.len() == 1);

        // remove subname
        let versionned_url = retry_loop!(safe.nrs_remove(&site_name));
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
    async fn test_nrs_invalid_topname() -> Result<()> {
        let safe = new_safe_instance().await?;

        // test with invalid top name
        let invalid_top_name = "atffdgasd/d";
        let expected_err = format!("The NRS top name \"{}\" is invalid because it contains url parts. Please remove any path, version or subnames.", invalid_top_name);
        match safe.nrs_create(invalid_top_name).await {
            Ok(_) => bail!("Unexpected NRS success when expected to fail with invalid top name"),
            Err(Error::InvalidInput(e)) => assert_eq!(e, expected_err),
            Err(_) => bail!("Expected an InvalidInput error kind, got smth else"),
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_conflicting_names() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create_from(TEST_DATA_FILE, None, false, false)
            .await?;
        let (version0, _) = retry_loop!(safe.files_container_get(&link))
            .ok_or(anyhow!("files container was unexpectedly empty"))?;

        // associate a first name
        let mut valid_link = SafeUrl::from_url(&link)?;
        valid_link.set_content_version(Some(version0));

        let (nrs_url, did_create) = retry_loop!(safe.nrs_add(&site_name, &valid_link));
        assert!(did_create);

        let _ = retry_loop!(safe.fetch(&nrs_url.to_string(), None));

        // associate a second name
        let second_valid_link = SafeUrl::from_url(&link)?;
        valid_link.set_content_version(Some(version0));
        let site_name2 = format!("sub.{}", &site_name);

        let (nrs_url2, did_create) = retry_loop!(safe.nrs_add(&site_name2, &second_valid_link));
        assert!(!did_create);

        let _ = retry_loop!(safe.fetch(&nrs_url2.to_string(), None));

        // manually add a conflicting name
        let another_valid_url = nrs_url;
        let url_str = validate_nrs_top_name(&site_name)?;
        let entry = (
            "".as_bytes().to_vec(),
            another_valid_url.to_string().as_bytes().to_vec(),
        );
        let _ = safe
            .multimap_insert(&url_str, entry, BTreeSet::new())
            .await?;

        // get of other name should be ok
        let _ = retry_loop_for_pattern!(safe.nrs_get(&site_name2, None), Ok((res_url, _)) if res_url == &second_valid_link)?;

        // get of conflicting name should error out
        let conflict_error =
            retry_loop_for_pattern!(safe.nrs_get(&site_name, None), Err(_) if true);
        assert!(matches!(
            conflict_error,
            Err(Error::ConflictingNrsEntries { .. })
        ));

        // check for the error content
        if let Err(Error::ConflictingNrsEntries(_, dups, _)) = conflict_error {
            let got_entries: Result<()> = dups.into_iter().try_for_each(|(subname, url)| {
                assert_eq!(subname, "");
                assert!(url == valid_link || url == another_valid_url);
                Ok(())
            });
            assert!(got_entries.is_ok());
        }

        // resolve the error
        let _ = retry_loop!(safe.nrs_associate(&site_name, &valid_link));

        // get should work now
        let _ = retry_loop_for_pattern!(safe.nrs_get(&site_name, None), Ok((res_url, _)) if res_url == &valid_link)?;
        Ok(())
    }
}
