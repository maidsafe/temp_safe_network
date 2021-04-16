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

use crate::{
    api::app::{
        consts::{CONTENT_ADDED_SIGN, CONTENT_DELETED_SIGN, PREDICATE_LINK},
        safeurl::{SafeContentType, SafeUrl, XorUrl},
        Safe,
    },
    Error, Result,
};
use hex::encode;
use log::{debug, info, warn};
use nrs_map::{parse_nrs_name, validate_nrs_link, DefinitionData, SubName, SubNameRdf};
use std::collections::{BTreeMap, BTreeSet};

// Type tag to use for the NrsMapContainer stored on Register
pub(crate) const NRS_MAP_TYPE_TAG: u64 = 1_500;

const NRS_MAP_CURRENT_SUB_NAME: &str = ".";

// List of public names uploaded with details if they were added, updated or deleted from NrsMaps
pub type ProcessedEntries = BTreeMap<String, (String, String)>;

impl Safe {
    /// Parse a Safe URL returning a SafeUrl instance
    pub fn parse_url(url: &str) -> Result<SafeUrl> {
        SafeUrl::from_url(&sanitised_url(url))
    }

    // Parses a safe:// URL and returns all the info in a SafeUrl instance.
    // It also returns a second SafeUrl if the URL was resolved from an NRS-URL,
    // this second SafeUrl instance contains the information of the parsed NRS-URL.
    // *Note* this is not part of the public API, but an internal helper function used by API impl.
    pub(crate) async fn parse_and_resolve_url(
        &mut self,
        url: &str,
    ) -> Result<(SafeUrl, Option<SafeUrl>)> {
        let safeurl = Safe::parse_url(url)?;
        let orig_path = safeurl.path_decoded()?;

        // Obtain the resolution chain without resolving the URL's path
        let mut resolution_chain = self
            .retrieve_from_url(
                &safeurl.to_string(),
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
        let mut safeurl = SafeUrl::from_url(&safe_data.xorurl())?;
        safeurl.set_path(&orig_path);

        // If there is still one item in the chain, the first item is the NRS Map Container
        // targeted by the URL and where the whole resolution started from
        if resolution_chain.is_empty() {
            Ok((safeurl, None))
        } else {
            let nrsmap_xorul_encoder = SafeUrl::from_url(&resolution_chain[0].resolved_from())?;
            Ok((safeurl, Some(nrsmap_xorul_encoder)))
        }
    }

    /// # Create a NrsMapContainer.
    pub async fn nrs_map_container_create(
        &mut self,
        name: &str,
        link: &str,
        default: bool,
        hard_link: bool,
        dry_run: bool,
    ) -> Result<(XorUrl, ProcessedEntries, NrsMap, EntryHash)> {
        info!("Creating an NRS map");
        let (_, nrs_url) = validate_nrs_name(name)?;
        /*if self.nrs_map_container_get(&nrs_url).await.is_ok() {
            return Err(Error::ContentError(
                "NRS name already exists. Please use 'nrs add' to add sub names to it"
                    .to_string(),
            ));
        }*/

        // NRS resolver doesn't allow links with no content hash (fragment '#')
        validate_nrs_link(link)?;

        let (top_name, mut nrs_names) = parse_nrs_name(name)?;
        nrs_names.push(top_name);

        let mut processed_entries = ProcessedEntries::new();
        processed_entries.insert(
            name.to_string(),
            (CONTENT_ADDED_SIGN.to_string(), link.to_string()),
        );

        let (xorurl, nrs_map, hash) = if dry_run {
            // TODO: review what would be the best to return for a dry-run
            ("".to_string(), NrsMap::default(), EntryHash::default())
        } else {
            // Each subname resolutions map is store in its own Public Register,
            // we will therefore create one Register for each subname, and
            // link them accordingly, creating thee in-memory NrsMap as well.
            // E.g. for safe://a.b.c we would have:
            // - Register at xorurl1: [ "a:<user-provided-link>" ]
            // - Register at xorurl2: [ "b:<xorurl1>" ]
            // - Register at hash(c): [ "c:<xorurl2>" ]
            let mut target_url = link.to_string();
            let mut hash = EntryHash::default();
            let mut current_nrs_map = NrsMap::default();
            let mut current_sub_name = NRS_MAP_CURRENT_SUB_NAME;

            for (i, sub_name) in nrs_names.iter().enumerate() {
                let (nrs_map, metadata) = if i == 0 {
                    NrsMap::with_default(&target_url)
                } else {
                    NrsMap::with_sub_map(
                        current_sub_name.to_string(),
                        current_nrs_map.clone(),
                        &target_url,
                    )
                };

                let entry = (current_sub_name.to_string(), metadata);
                let serialised_entry = rmp_serde::to_vec_named(&entry).map_err(|err| {
                    Error::Serialisation(format!(
                        "Couldn't serialise the NrsMap entry '({}, {})': {:?}",
                        sub_name, target_url, err
                    ))
                })?;

                let (xorname, new_hash) = if i < nrs_names.len() - 1 {
                    self.safe_client
                        .store_register(&serialised_entry, None, NRS_MAP_TYPE_TAG, None, false)
                        .await?
                } else {
                    // this is the top level NRS name, thus we store its Register
                    // at a specific xorname as per our NRS convention (i.e. sha3(<nrs name>))
                    let nrs_xorname = SafeUrl::from_nrsurl(&nrs_url)?.xorname();
                    debug!("XorName for \"{}\" is \"{:?}\"", &nrs_url, &nrs_xorname);

                    let (xorname, new_hash) = self
                        .safe_client
                        .store_register(
                            &serialised_entry,
                            Some(nrs_xorname),
                            NRS_MAP_TYPE_TAG,
                            None,
                            false,
                        )
                        .await?;

                    if default {
                        // FIXME: the register may not be fully written in all replicas yet,
                        // we need to be able to write both entries together upon creation of Register
                        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

                        let default_link_entry = rmp_serde::to_vec_named(&(
                            NRS_MAP_CURRENT_SUB_NAME,
                            target_url.clone(),
                        ))
                        .map_err(|err| {
                            Error::Serialisation(format!(
                                "Couldn't serialise the NrsMap default link entry '({}, {})': {:?}",
                                NRS_MAP_CURRENT_SUB_NAME, target_url, err
                            ))
                        })?;
                        let _ = self
                            .safe_client
                            .write_to_register(
                                &default_link_entry,
                                xorname,
                                NRS_MAP_TYPE_TAG,
                                false,
                                BTreeSet::new(),
                            )
                            .await?;
                    }

                    (xorname, new_hash)
                };

                let xorurl = SafeUrl::encode_register_url(
                    xorname,
                    NRS_MAP_TYPE_TAG,
                    SafeContentType::NrsMapContainer,
                    self.xorurl_base,
                    false,
                )?;

                target_url = xorurl;
                hash = new_hash;
                current_nrs_map = nrs_map;
                current_sub_name = sub_name;
            }

            (target_url, current_nrs_map, hash)
        };

        debug!("The new NRS Map: {:?}", nrs_map);
        Ok((xorurl, processed_entries, nrs_map, hash))
    }

    /// # Fetch an existing NrsMapContainer.
    pub async fn nrs_map_container_get(&mut self, url: &str) -> Result<(EntryHash, NrsMap)> {
        debug!("Getting latest resolvable map container from: {:?}", url);
        let mut current_safeurl = Safe::parse_url(url)?;
        let mut nrs_map_info = Vec::new();
        let mut top_nrs_map_hash = EntryHash::default();

        // Read each of the Registers by followinng the links resolved by each NRS sub-name
        while current_safeurl.content_type() == SafeContentType::NrsMapContainer {
            // Check if the URL specifies a hash or simply the latest available
            let data = match current_safeurl.content_hash() {
                None => {
                    let data = super::helpers::temp_pick_first_leaf(
                        self.safe_client
                            .read_register(
                                current_safeurl.xorname(),
                                NRS_MAP_TYPE_TAG,
                                /*is_private =*/ false,
                            )
                            .await?,
                    )?;
                    Ok(data.clone())
                }
                Some(content_hash) => {
                    let nrs_map = self
                        .safe_client
                        .get_register_entry(
                            current_safeurl.xorname(),
                            NRS_MAP_TYPE_TAG,
                            content_hash,
                            false,
                        )
                        .await
                        .map_err(|_| {
                            Error::HashNotFound(format!(
                                "Hash '{}' is invalid for NRS Map Container found at \"{}\"",
                                encode(content_hash),
                                url,
                            ))
                        })?;

                    Ok((content_hash, nrs_map))
                }
            };

            let (hash, xorurl) = match data {
                Ok((hash, nrs_map_entry_bytes)) => {
                    // We first parse the NrsMap entry from the Register
                    let (sub_name, definition): (SubName, DefinitionData) =
                        rmp_serde::from_slice(&nrs_map_entry_bytes).map_err(|err| {
                            Error::ContentError(format!(
                                "Couldn't parse the NrsMap entry stored in the NrsMapContainer {}: {:?}",
                                url, err
                            ))
                        })?;

                    debug!("Deserialised NrsMap entry: {} - {:?}", sub_name, definition);
                    match definition.get(PREDICATE_LINK) {
                        Some(link) => {
                            let nrs_map_xorurl = SafeUrl::from_url(link)?;
                            nrs_map_info.push((sub_name, definition));

                            Ok((hash, nrs_map_xorurl))
                        },
                        None => Err(Error::ContentError(format!(
                                    "Couldn't get link from NrsMap entry stored in the NrsMapContainer of {}",
                                    url
                                ))),
                    }
                }
                /*Err(Error::EmptyContent(_)) => {
                    warn!("Nrs container found at {:?} was empty", &url);
                    Ok((EntryHash::default(), NrsMap::default()))
                }*/
                Err(Error::ContentNotFound(_)) => Err(Error::ContentNotFound(
                    "No NRS Map found at this address".to_string(),
                )),
                Err(Error::HashNotFound(msg)) => Err(Error::HashNotFound(msg)),
                Err(err) => Err(Error::NetDataError(format!(
                    "Failed to get current version: {}",
                    err
                ))),
            }?;

            if top_nrs_map_hash == EntryHash::default() {
                top_nrs_map_hash = hash;
            }
            current_safeurl = xorurl;
        }

        let mut top_nrs_map = NrsMap::default();
        let mut i = 0;
        while let Some((sub_name, definition)) = nrs_map_info.pop() {
            if i == 0 && sub_name == NRS_MAP_CURRENT_SUB_NAME {
                top_nrs_map.default = DefaultRdf::OtherRdf(definition);
            } else if i == 0 {
                top_nrs_map
                    .sub_names_map
                    .insert(sub_name, SubNameRdf::Definition(definition));
            } else {
                let mut new_nrs_map = NrsMap::default();
                new_nrs_map
                    .sub_names_map
                    .insert(sub_name, SubNameRdf::SubName(top_nrs_map));
                top_nrs_map = new_nrs_map;
            }
            i += 1;
        }

        Ok((top_nrs_map_hash, top_nrs_map))
    }

    pub async fn nrs_map_container_add(
        &mut self,
        name: &str,
        link: &str,
        _default: bool,
        _hard_link: bool,
        dry_run: bool,
    ) -> Result<(XorUrl, ProcessedEntries, NrsMap, EntryHash)> {
        info!("Adding {} to NRS Map...", name);

        // GET current NRS map for NRS name provided
        let (safeurl, _) = validate_nrs_name(name)?;

        // NRS resolver doesn't allow links with no content hash (fragment '#')
        validate_nrs_link(link)?;

        let (_, mut nrs_names) = parse_nrs_name(name)?;

        let mut processed_entries = ProcessedEntries::new();
        processed_entries.insert(
            name.to_string(),
            (CONTENT_ADDED_SIGN.to_string(), link.to_string()),
        );

        let (xorurl, nrs_map, hash) = if dry_run {
            // TODO: review what would be the best to return for a dry-run
            ("".to_string(), NrsMap::default(), EntryHash::default())
        } else {
            //let mut target_url = link.to_string();
            //let mut hash = EntryHash::default();
            //let mut current_nrs_map = NrsMap::default();
            //let mut current_sub_name = NRS_MAP_CURRENT_SUB_NAME;
            let mut current_reg_safeurl = safeurl;

            while let Some(curr_sub_name) = nrs_names.pop() {
                // Let's read the current entries from the current sub-name's Register
                let nrs_map_entries = self.fetch_register_value(&current_reg_safeurl).await?;

                // find the current sub_name in the entries
                let mut link_found = false;
                for (hash, nrs_map_entry_bytes) in nrs_map_entries {
                    // We first parse the NrsMap entry from the Register
                    let (sub_name, definition): (SubName, DefinitionData) =
                        rmp_serde::from_slice(&nrs_map_entry_bytes).map_err(|err| {
                            Error::ContentError(format!(
                            "Couldn't parse the NrsMap entry stored in the NrsMapContainer of {}: {:?}",
                            name, err
                        ))
                        })?;

                    println!("Deserialised NrsMap entry: {} - {:?}", sub_name, definition);
                    match (sub_name == curr_sub_name, definition.get(PREDICATE_LINK)) {
                        (true, Some(link)) => {
                            current_reg_safeurl = SafeUrl::from_url(link)?;
                            link_found = true;
                            break;
                        }
                        _ => {}
                    }
                }

                if !link_found {
                    // this is where we need to add the new sub-name to
                    let mut target_url = link.to_string();
                    let mut hash = EntryHash::default();
                    let mut current_nrs_map = NrsMap::default();
                    let mut current_sub_name = NRS_MAP_CURRENT_SUB_NAME;
                    let mut new_nrs_names = nrs_names.clone();
                    new_nrs_names.push(curr_sub_name);
                    for (i, sub_name) in new_nrs_names.iter().enumerate() {
                        let (nrs_map, metadata) = if i == 0 {
                            NrsMap::with_default(&target_url)
                        } else {
                            NrsMap::with_sub_map(
                                current_sub_name.to_string(),
                                current_nrs_map.clone(),
                                &target_url,
                            )
                        };

                        let entry = (current_sub_name.to_string(), metadata);
                        let serialised_entry = rmp_serde::to_vec_named(&entry).map_err(|err| {
                            Error::Serialisation(format!(
                                "Couldn't serialise the NrsMap entry '({}, {})': {:?}",
                                sub_name, target_url, err
                            ))
                        })?;

                        let (xorurl, new_hash) = if i < new_nrs_names.len() - 1 {
                            let (xorname, hash) = self
                                .safe_client
                                .store_register(
                                    &serialised_entry,
                                    None,
                                    NRS_MAP_TYPE_TAG,
                                    None,
                                    false,
                                )
                                .await?;

                            let xorurl = SafeUrl::encode_register_url(
                                xorname,
                                NRS_MAP_TYPE_TAG,
                                SafeContentType::NrsMapContainer,
                                self.xorurl_base,
                                false,
                            )?;
                            (xorurl, hash)
                        } else {
                            let hash = self
                                .safe_client
                                .write_to_register(
                                    &serialised_entry,
                                    current_reg_safeurl.xorname(),
                                    NRS_MAP_TYPE_TAG,
                                    false,
                                    BTreeSet::new(),
                                )
                                .await?;

                            (current_reg_safeurl.to_string(), hash)
                        };

                        target_url = xorurl;
                        hash = new_hash;
                        current_nrs_map = nrs_map;
                        current_sub_name = sub_name;
                    }

                    break;
                }
            }

            //(target_url, current_nrs_map, hash)
            ("".to_string(), NrsMap::default(), EntryHash::default())
        };

        debug!("The new NRS Map: {:?}", nrs_map);
        Ok((xorurl, processed_entries, nrs_map, hash))
    }

    pub async fn nrs_map_container_remove(
        &mut self,
        name: &str,
        dry_run: bool,
    ) -> Result<(EntryHash, XorUrl, ProcessedEntries, NrsMap)> {
        info!("Removing from NRS map...");
        unimplemented!();
    }
}

// Private helper that verifies the NRS name neither contains `/`
// nor references a specific content hash with a fragment`#` part
fn validate_nrs_name(name: &str) -> Result<(SafeUrl, String)> {
    // validate no slashes in name.
    if name.find('/').is_some() {
        let msg = "The NRS name/subname cannot contain a slash".to_string();
        return Err(Error::InvalidInput(msg));
    }

    // parse the name into a url
    let sanitised_url = sanitised_url(name);

    let safeurl = Safe::parse_url(&sanitised_url)?;
    if safeurl.content_hash().is_some() {
        return Err(Error::InvalidInput(format!(
            "The NRS name/subname URL cannot contain a hash value: {}",
            sanitised_url
        )));
    };

    Ok((safeurl, sanitised_url))
}

fn sanitised_url(name: &str) -> String {
    // FIXME: make sure we remove the starting 'safe://'
    format!("safe://{}", name.replace("safe://", ""))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::app::{
            consts::PREDICATE_LINK,
            test_helpers::{new_safe_instance, random_nrs_name},
        },
        retry_loop, retry_loop_for_pattern, Safe,
    };
    use anyhow::{anyhow, bail, Result};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    async fn new_random_blob(safe: &mut Safe) -> Result<XorUrl> {
        let random_blob_content: String =
            thread_rng().sample_iter(&Alphanumeric).take(20).collect();

        let xorurl = safe
            .files_store_public_blob(random_blob_content.as_bytes(), None, false)
            .await?;

        Ok(xorurl)
    }

    #[tokio::test]
    async fn test_nrs_map_container_create() -> Result<()> {
        let nrs_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        let blob_xorurl = new_random_blob(&mut safe).await?;
        let nrs_xorname = Safe::parse_url(&nrs_name)?.xorname();

        let (xorurl, _, nrs_map, _) = safe
            .nrs_map_container_create(&nrs_name, &blob_xorurl, false, false, false)
            .await?;

        assert_eq!(nrs_map.sub_names_map.len(), 0);
        assert_eq!(nrs_map.get_default_link()?, blob_xorurl);

        if let DefaultRdf::OtherRdf(def_data) = &nrs_map.default {
            let link = def_data.get(PREDICATE_LINK).ok_or_else(|| {
                anyhow!("Entry with key '{}' not found in NrsMap", PREDICATE_LINK)
            })?;

            assert_eq!(*link, blob_xorurl);
            assert_eq!(
                nrs_map.get_default()?,
                &DefaultRdf::OtherRdf(def_data.clone())
            );
            let decoder = SafeUrl::from_url(&xorurl)?;
            assert_eq!(nrs_xorname, decoder.xorname());
            Ok(())
        } else {
            Err(anyhow!("No default definition map found...".to_string()))
        }
    }

    #[tokio::test]
    async fn test_nrs_map_container_get() -> Result<()> {
        let nrs_name = format!("a.b.c.{}", random_nrs_name());
        let mut safe = new_safe_instance().await?;

        let blob_xorurl = new_random_blob(&mut safe).await?;
        let nrs_xorname = Safe::parse_url(&nrs_name)?.xorname();

        let (xorurl, _, nrs_map, hash) = safe
            .nrs_map_container_create(&nrs_name, &blob_xorurl, false, false, false)
            .await?;

        let (fetched_hash, fetched_nrs_map) = retry_loop!(safe.nrs_map_container_get(&nrs_name));

        assert_eq!(fetched_hash, hash);
        assert_eq!(fetched_nrs_map, nrs_map);

        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_map_container_add() -> Result<()> {
        let nrs_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        let blob1_xorurl = new_random_blob(&mut safe).await?;
        let blob2_xorurl = new_random_blob(&mut safe).await?;

        let (xorurl, _, nrs_map, _) = safe
            .nrs_map_container_create(
                &format!("c.{}", nrs_name),
                &blob1_xorurl,
                true,
                false,
                false,
            )
            .await?;
        assert_eq!(nrs_map.sub_names_map.len(), 1);
        //assert_eq!(nrs_map.get_default_link()?, blob1_xorurl);
        println!("NRS MAP: {:?}", nrs_map);

        let _ = retry_loop!(safe.nrs_map_container_get(&xorurl));

        // add subname and set it as the new default too
        let (_, _, updated_nrs_map, new_hash) = safe
            .nrs_map_container_add(
                &format!("b.c.{}", nrs_name),
                &blob2_xorurl,
                true,
                false,
                false,
            )
            .await?;

        println!("UPDATED NRS MAP: {:?}", updated_nrs_map);
        //assert_eq!(updated_nrs_map.sub_names_map.len(), 1);
        //assert_eq!(updated_nrs_map.get_default_link()?, blob2_xorurl);

        let (fetched_hash, fetched_nrs_map) = retry_loop!(safe.nrs_map_container_get(&nrs_name));

        //assert_ne!(fetched_hash, new_hash);
        //assert_eq!(fetched_nrs_map.get_default_link()?, blob2_xorurl);
        assert_eq!(
            fetched_nrs_map.resolve_for_subnames(&["c".to_string(),])?,
            blob1_xorurl
        );
        assert_eq!(
            fetched_nrs_map.resolve_for_subnames(&["b".to_string(), "c".to_string(),])?,
            blob2_xorurl
        );

        Ok(())
    }
    /*
    #[tokio::test]
    async fn test_nrs_map_container_add_or_remove_with_versioned_target() -> Result<()> {
        let nrs_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let link_v0 = format!("{}?v=0", link);

        let (xorurl, _, _) = safe
            .nrs_map_container_create(&format!("b.{}", nrs_name), &link_v0, true, false, false)
            .await?;

        let _ = retry_loop!(safe.fetch(&xorurl, None));

        let versioned_sitename = format!("a.b.{}?v=6", nrs_name);
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
        let nrs_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let link_v0 = format!("{}?v=0", link);

        let (xorurl, _, nrs_map) = safe
            .nrs_map_container_create(&format!("a.b.{}", nrs_name), &link_v0, true, false, false)
            .await?;
        assert_eq!(nrs_map.sub_names_map.len(), 1);
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        let link_v1 = format!("{}?v=1", link);
        let _ = safe
            .nrs_map_container_add(&format!("a2.b.{}", nrs_name), &link_v1, true, false, false)
            .await?;

        let _ = retry_loop_for_pattern!(safe.nrs_map_container_get(&xorurl), Ok((version, _)) if *version == 1)?;

        // remove subname
        let (version, _, _, updated_nrs_map) = safe
            .nrs_map_container_remove(&format!("a.b.{}", nrs_name), false)
            .await?;

        assert_eq!(version, 2);
        assert_eq!(updated_nrs_map.sub_names_map.len(), 1);
        assert_eq!(updated_nrs_map.get_default_link()?, link_v1);

        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_map_container_remove_default_soft_link() -> Result<()> {
        let nrs_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let link_v0 = format!("{}?v=0", link);

        let (xorurl, _, nrs_map) = safe
            .nrs_map_container_create(&format!("a.b.{}", nrs_name), &link_v0, true, false, false)
            .await?;
        assert_eq!(nrs_map.sub_names_map.len(), 1);
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        // remove subname
        let (version, _, _, updated_nrs_map) = safe
            .nrs_map_container_remove(&format!("a.b.{}", nrs_name), false)
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
        let nrs_name = random_nrs_name();
        let mut safe = new_safe_instance().await?;

        // let's create an empty files container so we have a valid to link
        let (link, _, _) = safe
            .files_container_create(None, None, true, true, false)
            .await?;
        let link_v0 = format!("{}?v=0", link);

        let (xorurl, _, nrs_map) = safe
            .nrs_map_container_create(
                &format!("a.b.{}", nrs_name),
                &link_v0,
                true,
                true, // this sets the default to be a hard-link
                false,
            )
            .await?;
        assert_eq!(nrs_map.sub_names_map.len(), 1);
        let _ = retry_loop!(safe.fetch(&xorurl, None));

        // remove subname
        let (version, _, _, updated_nrs_map) = safe
            .nrs_map_container_remove(&format!("a.b.{}", nrs_name), false)
            .await?;
        assert_eq!(version, 1);
        assert_eq!(updated_nrs_map.sub_names_map.len(), 0);
        assert_eq!(updated_nrs_map.get_default_link()?, link_v0);
        Ok(())
    }

    #[tokio::test]
    async fn test_nrs_no_scheme() -> Result<()> {
        let nrs_name = random_nrs_name();
        let url = Safe::parse_url(&nrs_name)?;
        assert_eq!(url.public_name(), nrs_name);
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
    */
}
