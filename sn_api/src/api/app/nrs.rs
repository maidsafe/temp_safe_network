// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    consts::{CONTENT_ADDED_SIGN, CONTENT_DELETED_SIGN},
    nrs_map::NrsMap,
    xorurl::SafeContentType,
    Safe,
};
use crate::{
    xorurl::{XorUrl, XorUrlEncoder},
    Error, Result,
};
use log::{debug, info, warn};
use std::collections::BTreeMap;

// Type tag to use for the NrsMapContainer stored on Sequence
pub(crate) const NRS_MAP_TYPE_TAG: u64 = 1_500;

const ERROR_MSG_NO_NRS_MAP_FOUND: &str = "No NRS Map found at this address";

// List of public names uploaded with details if they were added, updated or deleted from NrsMaps
pub type ProcessedEntries = BTreeMap<String, (String, String)>;

impl Safe {
    pub fn parse_url(url: &str) -> Result<XorUrlEncoder> {
        XorUrlEncoder::from_url(&sanitised_url(url))
    }

    // Parses a safe:// URL and returns all the info in a XorUrlEncoder instance.
    // It also returns a second XorUrlEncoder if the URL was resolved from an NRS-URL,
    // this second XorUrlEncoder instance contains the information of the parsed NRS-URL.
    // *Note* this is not part of the public API, but an internal helper function used by API impl.
    pub(crate) async fn parse_and_resolve_url(
        &mut self,
        url: &str,
    ) -> Result<(XorUrlEncoder, Option<XorUrlEncoder>)> {
        let xorurl_encoder = Safe::parse_url(url)?;
        let orig_path = xorurl_encoder.path_decoded()?;

        // Obtain the resolution chain without resolving the URL's path
        let mut resolution_chain = self
            .retrieve_from_url(
                &xorurl_encoder.to_string(),
                false,
                None,
                false, // don't resolve the URL's path
            )
            .await?;

        // The resolved content is the last item in the resolution chain we obtained
        let safe_data = resolution_chain
            .pop()
            .ok_or_else(|| Error::ContentNotFound(format!("Failed to resolve {}", url)))?;

        // Set the original path so we return the XorUrlEncoder with it
        let mut xorurl_encoder = XorUrlEncoder::from_url(&safe_data.xorurl())?;
        xorurl_encoder.set_path(&orig_path);

        // If there is still one item in the chain, the first item is the NRS Map Container
        // targeted by the URL and where the whole resolution started from
        if resolution_chain.is_empty() {
            Ok((xorurl_encoder, None))
        } else {
            let nrsmap_xorul_encoder =
                XorUrlEncoder::from_url(&resolution_chain[0].resolved_from())?;
            Ok((xorurl_encoder, Some(nrsmap_xorul_encoder)))
        }
    }

    pub async fn nrs_map_container_add(
        &mut self,
        name: &str,
        link: &str,
        default: bool,
        hard_link: bool,
        dry_run: bool,
    ) -> Result<(u64, XorUrl, ProcessedEntries, NrsMap)> {
        info!("Adding to NRS map...");
        // GET current NRS map from name's TLD
        let (xorurl_encoder, _) = validate_nrs_name(name)?;
        let xorurl = xorurl_encoder.to_string();
        let (version, mut nrs_map) = self.nrs_map_container_get(&xorurl).await?;
        debug!("NRS, Existing data: {:?}", nrs_map);

        let link = nrs_map.nrs_update_map_or_create_data(name, link, default, hard_link)?;
        let mut processed_entries = ProcessedEntries::new();
        processed_entries.insert(name.to_string(), (CONTENT_ADDED_SIGN.to_string(), link));

        debug!("The new NRS Map: {:?}", nrs_map);
        if !dry_run {
            // Append new version of the NrsMap in the Public Sequence (NRS Map Container)
            let nrs_map_raw_data = gen_nrs_map_raw_data(&nrs_map)?;
            self.safe_client
                .append_to_sequence(
                    &nrs_map_raw_data,
                    xorurl_encoder.xorname(),
                    xorurl_encoder.type_tag(),
                    false,
                )
                .await?;
        }

        Ok((version + 1, xorurl, processed_entries, nrs_map))
    }

    /// # Create a NrsMapContainer.
    ///
    /// ## Example
    ///
    /// ```rust
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
            Err(Error::ContentError(
                "NRS name already exists. Please use 'nrs add' command to add sub names to it"
                    .to_string(),
            ))
        } else {
            let mut nrs_map = NrsMap::default();
            let link = nrs_map.nrs_update_map_or_create_data(&name, link, default, hard_link)?;
            let mut processed_entries = ProcessedEntries::new();
            processed_entries.insert(name.to_string(), (CONTENT_ADDED_SIGN.to_string(), link));

            debug!("The new NRS Map: {:?}", nrs_map);
            if dry_run {
                Ok(("".to_string(), processed_entries, nrs_map))
            } else {
                let nrs_xorname = XorUrlEncoder::from_nrsurl(&nrs_url)?.xorname();
                debug!("XorName for \"{:?}\" is \"{:?}\"", &nrs_url, &nrs_xorname);

                // Store the NrsMapContainer in a Public Sequence
                let nrs_map_raw_data = gen_nrs_map_raw_data(&nrs_map)?;
                let xorname = self
                    .safe_client
                    .store_sequence(
                        &nrs_map_raw_data,
                        Some(nrs_xorname),
                        NRS_MAP_TYPE_TAG,
                        None,
                        false,
                    )
                    .await?;

                let xorurl = XorUrlEncoder::encode_sequence_data(
                    xorname,
                    NRS_MAP_TYPE_TAG,
                    SafeContentType::NrsMapContainer,
                    self.xorurl_base,
                    false,
                )?;

                Ok((xorurl, processed_entries, nrs_map))
            }
        }
    }

    pub async fn nrs_map_container_remove(
        &mut self,
        name: &str,
        dry_run: bool,
    ) -> Result<(u64, XorUrl, ProcessedEntries, NrsMap)> {
        info!("Removing from NRS map...");
        // GET current NRS map from &name TLD
        let (xorurl_encoder, _) = validate_nrs_name(name)?;
        let xorurl = xorurl_encoder.to_string();
        let (version, mut nrs_map) = self.nrs_map_container_get(&xorurl).await?;
        debug!("NRS, Existing data: {:?}", nrs_map);

        let removed_link = nrs_map.nrs_map_remove_subname(name)?;
        let mut processed_entries = ProcessedEntries::new();
        processed_entries.insert(
            name.to_string(),
            (CONTENT_DELETED_SIGN.to_string(), removed_link),
        );

        debug!("The new NRS Map: {:?}", nrs_map);
        if !dry_run {
            // Append new version of the NrsMap in the Public Sequence (NRS Map Container)
            let nrs_map_raw_data = gen_nrs_map_raw_data(&nrs_map)?;
            self.safe_client
                .append_to_sequence(
                    &nrs_map_raw_data,
                    xorurl_encoder.xorname(),
                    xorurl_encoder.type_tag(),
                    false,
                )
                .await?;
        }

        Ok((version + 1, xorurl, processed_entries, nrs_map))
    }

    /// # Fetch an existing NrsMapContainer.
    ///
    /// ## Example
    ///
    /// ```rust
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
    pub async fn nrs_map_container_get(&mut self, url: &str) -> Result<(u64, NrsMap)> {
        debug!("Getting latest resolvable map container from: {:?}", url);
        let xorurl_encoder = Safe::parse_url(url)?;

        // Check if the URL specified a specific version of the content or simply the latest available
        let data = match xorurl_encoder.content_version() {
            None => {
                self.safe_client
                    .sequence_get_last_entry(xorurl_encoder.xorname(), NRS_MAP_TYPE_TAG, false)
                    .await
            }
            Some(content_version) => {
                let serialised_nrs_map = self
                    .safe_client
                    .sequence_get_entry(
                        xorurl_encoder.xorname(),
                        NRS_MAP_TYPE_TAG,
                        content_version,
                        false,
                    )
                    .await
                    .map_err(|_| {
                        Error::VersionNotFound(format!(
                            "Version '{}' is invalid for NRS Map Container found at \"{}\"",
                            content_version, url,
                        ))
                    })?;

                Ok((content_version, serialised_nrs_map))
            }
        };

        match data {
            Ok((version, serialised_nrs_map)) => {
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
                Ok((0, NrsMap::default()))
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
}

fn validate_nrs_name(name: &str) -> Result<(XorUrlEncoder, String)> {
    let sanitised_url = sanitised_url(name);
    let xorurl_encoder = Safe::parse_url(&sanitised_url)?;
    if xorurl_encoder.content_version().is_some() {
        return Err(Error::InvalidInput(format!(
            "The NRS name/subname URL cannot contain a version: {}",
            sanitised_url
        )));
    };
    Ok((xorurl_encoder, sanitised_url))
}

fn sanitised_url(name: &str) -> String {
    // FIXME: make sure we remove the starting 'safe://'
    format!("safe://{}", name.replace("safe://", ""))
}

fn gen_nrs_map_raw_data(nrs_map: &NrsMap) -> Result<Vec<u8>> {
    // The NrsMapContainer is a Sequence where each NRS Map version is
    // an entry containing the serialised NrsMap
    // TODO: use RDF format
    let serialised_nrs_map = serde_json::to_string(nrs_map).map_err(|err| {
        Error::Serialisation(format!(
            "Couldn't serialise the NrsMap generated: {:?}",
            err
        ))
    })?;

    Ok(serialised_nrs_map.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::app::test_helpers::{new_safe_instance, random_nrs_name};
    use anyhow::{anyhow, Result};

    #[tokio::test]
    async fn test_nrs_map_container_create() -> Result<()> {
        use crate::api::app::consts::FAKE_RDF_PREDICATE_LINK;
        use crate::nrs_map::DefaultRdf;
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        let nrs_xorname = Safe::parse_url(&site_name)?.xorname();

        let (xor_url, _entries, nrs_map) = safe
            .nrs_map_container_create(
                &site_name,
                "safe://linked-from-<site_name>?v=0",
                true,
                false,
                false,
            )
            .await?;

        assert_eq!(nrs_map.sub_names_map.len(), 0);
        assert_eq!(
            nrs_map.get_default_link()?,
            "safe://linked-from-<site_name>?v=0"
        );

        if let DefaultRdf::OtherRdf(def_data) = &nrs_map.default {
            let link = def_data
                .get(FAKE_RDF_PREDICATE_LINK)
                .ok_or_else(|| anyhow!("Entry not found with key '{}'", FAKE_RDF_PREDICATE_LINK))?;

            assert_eq!(*link, "safe://linked-from-<site_name>?v=0".to_string());
            assert_eq!(
                nrs_map.get_default()?,
                &DefaultRdf::OtherRdf(def_data.clone())
            );
            let decoder = XorUrlEncoder::from_url(&xor_url)?;
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
        let (_xor_url, _entries, nrs_map) = safe
            .nrs_map_container_create(
                &format!("b.{}", site_name),
                "safe://linked-from-<b.site_name>?v=0",
                true,
                false,
                false,
            )
            .await?;
        assert_eq!(nrs_map.sub_names_map.len(), 1);
        assert_eq!(
            nrs_map.get_default_link()?,
            "safe://linked-from-<b.site_name>?v=0"
        );

        // add subname and set it as the new default too
        let (version, _xorurl, _entries, updated_nrs_map) = safe
            .nrs_map_container_add(
                &format!("a.b.{}", site_name),
                "safe://linked-from-<a.b.site_name>?v=0",
                true,
                false,
                false,
            )
            .await?;
        assert_eq!(version, 1);
        assert_eq!(updated_nrs_map.sub_names_map.len(), 1);
        assert_eq!(
            updated_nrs_map.get_default_link()?,
            "safe://linked-from-<a.b.site_name>?v=0"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_map_container_add_or_remove_with_versioned_target() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;
        let _ = safe
            .nrs_map_container_create(
                &format!("b.{}", site_name),
                "safe://linked-from-<b.site_name>?v=0",
                true,
                false,
                false,
            )
            .await?;

        let versioned_sitename = format!("safe://a.b.{}?v=6", site_name);
        match safe
            .nrs_map_container_add(
                &versioned_sitename,
                "safe://linked-from-<a.b.site_name>?v=0",
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
            Err(err) => assert_eq!(
                err,
                Error::InvalidInput(format!(
                    "The NRS name/subname URL cannot contain a version: {}",
                    versioned_sitename
                ))
            ),
        };

        match safe
            .nrs_map_container_remove(&versioned_sitename, false)
            .await
        {
            Ok(_) => Err(anyhow!(
                "NRS map remove was unexpectedly successful".to_string(),
            )),
            Err(err) => {
                assert_eq!(
                    err,
                    Error::InvalidInput(format!(
                        "The NRS name/subname URL cannot contain a version: {}",
                        versioned_sitename
                    ))
                );
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_nrs_map_container_remove_one_of_two() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;
        let (_xor_url, _entries, nrs_map) = safe
            .nrs_map_container_create(
                &format!("a.b.{}", site_name),
                "safe://linked-from-<a.b.site_name>?v=0",
                true,
                false,
                false,
            )
            .await?;
        assert_eq!(nrs_map.sub_names_map.len(), 1);

        let (_version, _xorurl, _entries, _updated_nrs_map) = safe
            .nrs_map_container_add(
                &format!("a2.b.{}", site_name),
                "safe://linked-from-<a2.b.site_name>?v=0",
                true,
                false,
                false,
            )
            .await?;

        // remove subname
        let (version, _xorurl, _entries, updated_nrs_map) = safe
            .nrs_map_container_remove(&format!("a.b.{}", site_name), false)
            .await?;
        assert_eq!(version, 2);
        assert_eq!(updated_nrs_map.sub_names_map.len(), 1);
        assert_eq!(
            updated_nrs_map.get_default_link()?,
            "safe://linked-from-<a2.b.site_name>?v=0"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_map_container_remove_default_soft_link() -> Result<()> {
        let site_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;
        let (_xor_url, _entries, nrs_map) = safe
            .nrs_map_container_create(
                &format!("a.b.{}", site_name),
                "safe://linked-from-<a.b.site_name>?v=0",
                true,
                false,
                false,
            )
            .await?;
        assert_eq!(nrs_map.sub_names_map.len(), 1);

        // remove subname
        let (version, _xorurl, _entries, updated_nrs_map) = safe
            .nrs_map_container_remove(&format!("a.b.{}", site_name), false)
            .await?;
        assert_eq!(version, 1);
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
        let (_xor_url, _entries, nrs_map) = safe
            .nrs_map_container_create(
                &format!("a.b.{}", site_name),
                "safe://linked-from-<a.b.site_name>?v=0",
                true,
                true, // this sets the default to be a hard-link
                false,
            )
            .await?;
        assert_eq!(nrs_map.sub_names_map.len(), 1);

        // remove subname
        let (version, _xorurl, _entries, updated_nrs_map) = safe
            .nrs_map_container_remove(&format!("a.b.{}", site_name), false)
            .await?;
        assert_eq!(version, 1);
        assert_eq!(updated_nrs_map.sub_names_map.len(), 0);
        assert_eq!(
            updated_nrs_map.get_default_link()?,
            "safe://linked-from-<a.b.site_name>?v=0"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_no_scheme() -> Result<()> {
        let site_name = random_nrs_name();
        let url = Safe::parse_url(&site_name)?;
        assert_eq!(url.public_name(), site_name);
        Ok(())
    }
}
