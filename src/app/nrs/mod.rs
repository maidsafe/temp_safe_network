// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod nrs_map;

pub use nrs_map::{DefaultRdf, NrsMap};
pub use safe_network::url::VersionHash;

use crate::{
    app::{
        consts::{CONTENT_ADDED_SIGN, CONTENT_DELETED_SIGN},
        Safe,
    },
    register::EntryHash,
    Error, Result, SafeUrl, XorUrl,
};
use log::{debug, info, warn};
use std::collections::{BTreeMap, BTreeSet};

// Type tag to use for the NrsMapContainer stored on Register
pub(crate) const NRS_MAP_TYPE_TAG: u64 = 1_500;

const ERROR_MSG_NO_NRS_MAP_FOUND: &str = "No NRS Map found at this address";

// List of public names uploaded with details if they were added, updated or deleted from NrsMaps
pub type ProcessedEntries = BTreeMap<String, (String, String)>;

impl Safe {
    pub fn parse_url(url: &str) -> Result<SafeUrl> {
        let safe_url = SafeUrl::from_url(&sanitised_url(url))?;
        Ok(safe_url)
    }

    // Parses a safe:// URL and returns all the info in a SafeUrl instance.
    // It also returns a second SafeUrl if the URL was resolved from an NRS-URL,
    // this second SafeUrl instance contains the information of the parsed NRS-URL.
    // *Note* this is not part of the public API, but an internal helper function used by API impl.
    pub(crate) async fn parse_and_resolve_url(
        &self,
        url: &str,
    ) -> Result<(SafeUrl, Option<SafeUrl>)> {
        let safe_url = Safe::parse_url(url)?;
        let orig_path = safe_url.path_decoded()?;

        // Obtain the resolution chain without resolving the URL's path
        let mut resolution_chain = self
            .retrieve_from_url(
                // TODO take a look at safe url code where its used, ask gab
                &safe_url.to_string(),
                false,
                None,
                false, // don't resolve the URL's path
            )
            .await?;

        // The resolved content is the last item in the resolution chain we obtained
        let safe_data = resolution_chain
            .pop()
            .ok_or_else(|| Error::ContentNotFound(format!("Failed to resolve {}", url)))?;

        // Set the original path so we return the SafeUrl with it
        let mut safe_url = SafeUrl::from_url(&safe_data.xorurl())?;
        safe_url.set_path(&orig_path);

        // If there is still one item in the chain, the first item is the NRS Map Container
        // targeted by the URL and where the whole resolution started from
        if resolution_chain.is_empty() {
            Ok((safe_url, None))
        } else {
            let nrsmap_xorul_encoder = SafeUrl::from_url(&resolution_chain[0].resolved_from())?;
            Ok((safe_url, Some(nrsmap_xorul_encoder)))
        }
    }

    pub async fn nrs_map_container_add(
        &self,
        name: &str,
        link: &str,
        default: bool,
        hard_link: bool,
        dry_run: bool,
    ) -> Result<(VersionHash, XorUrl, ProcessedEntries, NrsMap)> {
        info!("Adding to NRS map...");
        // GET current NRS map from name's TLD
        let (safe_url, _) = validate_nrs_name(name)?;
        let xorurl = safe_url.to_string();
        let (version, mut nrs_map) = self.nrs_map_container_get(&xorurl).await?;
        debug!("NRS, Existing data: {:?}", nrs_map);

        let link = nrs_map.update(name, link, default, hard_link)?;
        let mut processed_entries = ProcessedEntries::new();
        processed_entries.insert(name.to_string(), (CONTENT_ADDED_SIGN.to_string(), link));
        debug!("The new NRS Map: {:?}", nrs_map);

        if dry_run {
            return Ok((version, xorurl, processed_entries, nrs_map));
        }

        let nrs_map_xorurl = self.store_nrs_map(&nrs_map).await?;
        let old_values: BTreeSet<EntryHash> = self
            .fetch_multimap_values(&safe_url)
            .await?
            .iter()
            .map(|(hash, _)| hash.to_owned())
            .collect();
        let entry = (
            name.as_bytes().to_owned(),
            nrs_map_xorurl.as_bytes().to_owned(),
        );
        let entry_hash = &self.multimap_insert(&xorurl, entry, old_values).await?;
        let new_version:VersionHash = entry_hash.into();

        Ok((new_version, xorurl, processed_entries, nrs_map))
    }

    /// # Create a NrsMapContainer.
    ///
    /// ## Example
    ///
    /// ```rust_no_run
    /// # use rand::distributions::Alphanumeric;
    /// # use rand::{thread_rng, Rng};
    /// # use sn_api::Safe;
    /// # let mut safe = Safe::default();
    /// # async_std::task::block_on(async {
    /// #   safe.connect("", Some("fake-credentials")).await.unwrap();
    ///     let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    ///     let file_xorurl = safe.files_store_public_blob(&vec![], None, false).await.unwrap();
    ///     let (xorurl, _processed_entries, nrs_map_container) = safe.nrs_map_container_create(&rand_string, &file_xorurl, true, false, false).await.unwrap();
    ///     assert!(xorurl.contains("safe://"))
    /// # });
    /// ```
    pub async fn nrs_map_container_create(
        &mut self,
        name: &str,
        link: &str,
        default: bool,
        hard_link: bool,
        dry_run: bool,
    ) -> Result<(XorUrl, ProcessedEntries, NrsMap)> {
        info!("Creating an NRS map");
        let (_, nrs_url) = validate_nrs_name(name)?;
        if self.nrs_map_container_get(&nrs_url).await.is_ok() {
            return Err(Error::ContentError(
                "NRS name already exists. Please use 'nrs add' command to add sub names to it"
                    .to_string(),
            ));
        }

        let mut nrs_map = NrsMap::default();
        let link = nrs_map.update(&name, link, default, hard_link)?;
        let mut processed_entries = ProcessedEntries::new();
        processed_entries.insert(name.to_string(), (CONTENT_ADDED_SIGN.to_string(), link));
        debug!("The new NRS Map: {:?}", nrs_map);

        if dry_run {
            return Ok(("".to_string(), processed_entries, nrs_map));
        }

        let nrs_xorname = SafeUrl::from_nrsurl(&nrs_url)?.xorname();
        debug!("XorName for \"{:?}\" is \"{:?}\"", &nrs_url, &nrs_xorname);

        // Store the serialised NrsMap in a Public Blob
        let nrs_map_xorurl = self.store_nrs_map(&nrs_map).await?;

        // Create a new multimap
        let xorurl = self
            .multimap_create(Some(nrs_xorname), NRS_MAP_TYPE_TAG, false)
            .await?;

        // Store the NRS map in the multimap
        let entry = (
            name.as_bytes().to_owned(),
            nrs_map_xorurl.as_bytes().to_owned(),
        );
        let _ = self
            .multimap_insert(&xorurl, entry, BTreeSet::new())
            .await?;

        Ok((xorurl, processed_entries, nrs_map))
    }

    pub async fn nrs_map_container_remove(
        &self,
        name: &str,
        dry_run: bool,
    ) -> Result<(VersionHash, XorUrl, ProcessedEntries, NrsMap)> {
        info!("Removing from NRS map...");
        // GET current NRS map from &name TLD
        let (safe_url, _) = validate_nrs_name(name)?;
        let xorurl = safe_url.to_string();
        let (version, mut nrs_map) = self.nrs_map_container_get(&xorurl).await?;
        debug!("NRS, Existing data: {:?}", nrs_map);

        let removed_link = nrs_map.nrs_map_remove_subname(name)?;
        let mut processed_entries = ProcessedEntries::new();
        processed_entries.insert(
            name.to_string(),
            (CONTENT_DELETED_SIGN.to_string(), removed_link),
        );

        if dry_run {
            return Ok((version, xorurl, processed_entries, nrs_map));
        }

        debug!("Removing from multimap");
        let old_values: BTreeSet<EntryHash> = self
            .fetch_multimap_values(&safe_url)
            .await?
            .iter()
            .map(|(hash, _)| hash.to_owned())
            .collect();

        // TODO use remove
        // self.multimap_remove(&xorurl, old_values).await?;
        // tmp retro-compatible workaround with insert
        let nrs_map_xorurl = self.store_nrs_map(&nrs_map).await?;
        let entry = (
            name.as_bytes().to_owned(),
            nrs_map_xorurl.as_bytes().to_owned(),
        );
        let entry_hash = &self.multimap_insert(&xorurl, entry, old_values).await?;
        let new_version:VersionHash = entry_hash.into();

        Ok((new_version, xorurl, processed_entries, nrs_map))
    }

    /// # Fetch an existing NrsMapContainer.
    ///
    /// ## Example
    ///
    /// ```rust_no_run
    /// # use sn_api::Safe;
    /// # use rand::distributions::Alphanumeric;
    /// # use rand::{thread_rng, Rng};
    /// # let mut safe = Safe::default();
    /// # async_std::task::block_on(async {
    /// #   safe.connect("", Some("fake-credentials")).await.unwrap();
    ///     let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    ///     let file_xorurl = safe.files_store_public_blob(&vec![], Some("text/plain"), false).await.unwrap();
    ///     let (xorurl, _processed_entries, _nrs_map) = safe.nrs_map_container_create(&rand_string, &file_xorurl, true, false, false).await.unwrap();
    ///     let (version, nrs_map_container) = safe.nrs_map_container_get(&xorurl).await.unwrap();
    ///     assert_eq!(version, 0);
    ///     assert_eq!(nrs_map_container.get_default_link().unwrap(), file_xorurl);
    /// # });
    /// ```
    pub async fn nrs_map_container_get(&self, url: &str) -> Result<(VersionHash, NrsMap)> {
        debug!("Getting latest resolvable map container from: {:?}", url);
        let safe_url = Safe::parse_url(url)?;

        // fetch multimap latest values
        // TODO: manage multiple resolutions currently only returns the 1st one
        let data = match self.fetch_multimap_values(&safe_url).await?.iter().next() {
            Some((register_entry_hash, (_name, nrs_map_xorurl_bytes))) => {
                Ok((register_entry_hash.into(), nrs_map_xorurl_bytes.to_owned()))
            }
            None => Err(Error::EmptyContent(format!(
                "Empty Register found at XoR name {}",
                safe_url.xorname()
            ))),
        };

        match data {
            Ok((version, nrs_map_xorurl_bytes)) => {
                // We first parse the NrsMap XOR-URL from the Register
                let url = String::from_utf8(nrs_map_xorurl_bytes.to_owned()).map_err(|err| {
                    Error::ContentError(format!(
                        "Couldn't parse the NrsMap link stored in the NrsMapContainer: {:?}",
                        err
                    ))
                })?;
                debug!("Deserialised NrsMap XOR-URL: {}", url);
                let nrs_map_xorurl = SafeUrl::from_url(&url)?;

                // Using the NrsMap XOR-URL we can now fetch the NrsMap and deserialise it
                let serialised_nrs_map = self.fetch_public_blob(&nrs_map_xorurl, None).await?;

                debug!("Nrs map v{} retrieved: {:?} ", version, &serialised_nrs_map);
                let nrs_map =
                    serde_json::from_str(&String::from_utf8_lossy(&serialised_nrs_map.as_slice()))
                        .map_err(|err| {
                            Error::ContentError(format!(
                                "Couldn't deserialise the NrsMap stored in the NrsContainer: {:?}",
                                err
                            ))
                        })?;

                Ok((version, nrs_map))
            }
            Err(Error::EmptyContent(_)) => {
                warn!("Nrs container found at {:?} was empty", &url);
                Ok((VersionHash::default(), NrsMap::default()))
            }
            Err(Error::ContentNotFound(_)) => Err(Error::ContentNotFound(
                ERROR_MSG_NO_NRS_MAP_FOUND.to_string(),
            )),
            Err(Error::VersionNotFound(msg)) => Err(Error::VersionNotFound(msg)),
            Err(err) => Err(Error::NetDataError(format!(
                "Failed to get current version: {}",
                err
            ))),
        }
    }

    // Private helper to serialise an NrsMap and store it in a Public Blob
    async fn store_nrs_map(&self, nrs_map: &NrsMap) -> Result<String> {
        // The NrsMapContainer is a Register where each NRS Map version is
        // an entry containing the XOR-URL of the Blob that contains the serialised NrsMap.
        // TODO: use RDF format
        let serialised_nrs_map = serde_json::to_string(nrs_map).map_err(|err| {
            Error::Serialisation(format!(
                "Couldn't serialise the NrsMap generated: {:?}",
                err
            ))
        })?;

        let nrs_map_xorurl = self
            .files_store_public_blob(serialised_nrs_map.as_bytes(), None, false)
            .await?;

        Ok(nrs_map_xorurl)
    }
}

fn validate_nrs_name(name: &str) -> Result<(SafeUrl, String)> {
    // validate no slashes in name.
    if name.find('/').is_some() {
        let msg = "The NRS name/subname cannot contain a slash".to_string();
        return Err(Error::InvalidInput(msg));
    }
    // parse the name into a url
    let sanitised_url = sanitised_url(name);
    let safe_url = Safe::parse_url(&sanitised_url)?;
    if safe_url.content_version().is_some() {
        return Err(Error::InvalidInput(format!(
            "The NRS name/subname URL cannot contain a version: {}",
            sanitised_url
        )));
    };
    Ok((safe_url, sanitised_url))
}

fn sanitised_url(name: &str) -> String {
    // FIXME: make sure we remove the starting 'safe://'
    format!("safe://{}", name.replace("safe://", ""))
}

#[cfg(test)]
mod tests {
    use super::nrs_map::DefaultRdf;
    use super::*;
    use crate::{
        app::{
            consts::PREDICATE_LINK,
            test_helpers::{new_safe_instance, random_nrs_name},
        },
        retry_loop, retry_loop_for_pattern,
    };
    use anyhow::{anyhow, bail, Result};

    #[tokio::test]
    async fn test_nrs_map_container_create() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        let nrs_xorname = Safe::parse_url(&site_name)?.xorname();

        let (xor_url, _, nrs_map) = retry_loop!(safe.nrs_map_container_create(
            &site_name,
            "safe://linked-from-site_name?v=0",
            true,
            false,
            false
        ));

        assert_eq!(nrs_map.sub_names_map.len(), 0);
        assert_eq!(
            nrs_map.get_default_link()?,
            "safe://linked-from-site_name?v=0"
        );

        if let DefaultRdf::OtherRdf(def_data) = &nrs_map.default {
            let link = def_data
                .get(PREDICATE_LINK)
                .ok_or_else(|| anyhow!("Entry not found with key '{}'", PREDICATE_LINK))?;

            assert_eq!(*link, "safe://linked-from-site_name?v=0".to_string());
            assert_eq!(
                nrs_map.get_default()?,
                &DefaultRdf::OtherRdf(def_data.clone())
            );
            let decoder = SafeUrl::from_url(&xor_url)?;
            assert_eq!(nrs_xorname, decoder.xorname());
            Ok(())
        } else {
            Err(anyhow!("No default definition map found...".to_string(),))
        }
    }

    #[tokio::test]
    async fn test_nrs_map_container_add() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let link_v0 = format!("{}?v=0", link);

        let (xorurl, _, nrs_map) = retry_loop!(safe.nrs_map_container_create(
            &format!("b.{}", site_name),
            &link_v0,
            true,
            false,
            false
        ));

        assert_eq!(nrs_map.sub_names_map.len(), 1);
        assert_eq!(nrs_map.get_default_link()?, link_v0);
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        // add subname and set it as the new default too
        let link_v1 = format!("{}?v=1", link);
        let (_version, _, _, updated_nrs_map) = retry_loop!(safe.nrs_map_container_add(
            &format!("a.b.{}", site_name),
            &link_v1,
            true,
            false,
            false
        ));

        // assert_eq!(version, 1); // Versions features disabled temporarily (TODO replace with hash)
        assert_eq!(updated_nrs_map.sub_names_map.len(), 1);
        assert_eq!(updated_nrs_map.get_default_link()?, link_v1);

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
        let link_v0 = format!("{}?v=0", link);

        let (xorurl, _, _) = retry_loop!(safe
            .nrs_map_container_create(&format!("b.{}", site_name), &link_v0, true, false, false));

        let _ = retry_loop!(safe.fetch(&xorurl, None));

        let versioned_sitename = format!("a.b.{}?v=6", site_name);
        match safe
            .nrs_map_container_add(
                &versioned_sitename,
                "safe://linked-from-a_b_site_name?v=0",
                true,
                false,
                false,
            )
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
                    "The NRS name/subname URL cannot contain a version: safe://{}",
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
                        "The NRS name/subname URL cannot contain a version: safe://{}",
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
        let link_v0 = format!("{}?v=0", link);

        let (xorurl, _, nrs_map) = retry_loop!(safe
            .nrs_map_container_create(
                &format!("a.b.{}", site_name),
                &link_v0,
                true,
                false,
                false,
            )
        );
        assert_eq!(nrs_map.sub_names_map.len(), 1);
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        let link_v1 = format!("{}?v=1", link);
        let (version, _, _, _) = retry_loop!(safe
            .nrs_map_container_add(&format!("a2.b.{}", site_name), &link_v1, true, false, false)
        );

        // TODO use hash for version, this is a placeholder
        let _ = retry_loop_for_pattern!(safe.nrs_map_container_get(&xorurl), Ok((version, _)) if *version == *version)?;

        // remove subname
        let (version, _, _, updated_nrs_map) = retry_loop!(safe
            .nrs_map_container_remove(&format!("a.b.{}", site_name), false)
        );

        // TODO use hash for version
        // assert_eq!(version, 2);
        assert_eq!(updated_nrs_map.sub_names_map.len(), 1);
        assert_eq!(updated_nrs_map.get_default_link()?, link_v1);

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
        let link_v0 = format!("{}?v=0", link);

        let (xorurl, _, nrs_map) = retry_loop!(safe
            .nrs_map_container_create(
                &format!("a.b.{}", site_name),
                &link_v0,
                true,
                false,
                false,
            )
        );
        assert_eq!(nrs_map.sub_names_map.len(), 1);
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        // remove subname
        let (_version, _, _, updated_nrs_map) = retry_loop!(safe
            .nrs_map_container_remove(&format!("a.b.{}", site_name), false)
        );
        // assert_eq!(version, 1);
        assert_eq!(updated_nrs_map.sub_names_map.len(), 0);
        match updated_nrs_map.get_default_link() {
            Ok(link) => Err(anyhow!("Unexpectedly retrieved a default link: {}", link)),
            Err(Error::ContentError(msg)) => {
                assert_eq!(
                    msg,
                    "Default found for resolvable map (set to sub names 'a.b') cannot be resolved."
                        .to_string()
                );
                Ok(())
            }
            Err(err) => Err(anyhow!("Error returned is not the expected one: {}", err)),
        }
    }

    #[tokio::test]
    async fn test_nrs_map_container_remove_default_hard_link() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let link_v0 = format!("{}?v=0", link);

        let (xorurl, _, nrs_map) = retry_loop!(safe
            .nrs_map_container_create(
                &format!("a.b.{}", site_name),
                &link_v0,
                true,
                true, // this sets the default to be a hard-link
                false,
            )
        );
        assert_eq!(nrs_map.sub_names_map.len(), 1);
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        // remove subname
        let (version, _, _, updated_nrs_map) = retry_loop!(safe
            .nrs_map_container_remove(&format!("a.b.{}", site_name), false)
        );

        // TODO use version hash
        // assert_eq!(version, 1);
        assert_eq!(updated_nrs_map.sub_names_map.len(), 0);
        assert_eq!(updated_nrs_map.get_default_link()?, link_v0);
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
    async fn test_nrs_validate_name() -> Result<()> {
        let nrs_name = random_nrs_name();
        let (_, nrs_url) = validate_nrs_name(&nrs_name)?;
        assert_eq!(nrs_url, format!("safe://{}", nrs_name));
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_validate_name_with_slash() -> Result<()> {
        let nrs_name = "name/with/slash";
        match validate_nrs_name(&nrs_name) {
            Ok(_) => Err(anyhow!(
                "Unexpectedly validated nrs name with slashes {}",
                nrs_name
            )),
            Err(Error::InvalidInput(msg)) => {
                assert_eq!(
                    msg,
                    "The NRS name/subname cannot contain a slash".to_string()
                );
                Ok(())
            }
            Err(err) => Err(anyhow!("Error returned is not the expected one: {}", err)),
        }
    }
}
