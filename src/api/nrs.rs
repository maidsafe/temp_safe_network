// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::constants::{
    CONTENT_ADDED_SIGN, CONTENT_DELETED_SIGN, /*CONTENT_ERROR_SIGN, CONTENT_UPDATED_SIGN,*/
    FAKE_RDF_PREDICATE_CREATED, FAKE_RDF_PREDICATE_LINK, FAKE_RDF_PREDICATE_MODIFIED,
};

use super::helpers::{gen_timestamp_secs, get_subnames_host_and_path};
use super::xorurl::{SafeContentType, SafeDataType};
use super::{Error, ResultReturn, Safe, SafeApp, XorUrl, XorUrlEncoder};
use log::{debug, info, warn};
use safe_nd::XorName;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use tiny_keccak::sha3_256;

// Type tag to use for the NrsMapContainer stored on AppendOnlyData
const NRS_MAP_TYPE_TAG: u64 = 1_500;

const ERROR_MSG_NO_NRS_MAP_FOUND: &str = "No NRS Map found at this address";

type SubName = String;
type DefinitionData = BTreeMap<String, String>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum SubNameRDF {
    Definition(DefinitionData),
    SubName(NrsMap),
}

impl SubNameRDF {
    fn get(&self, key: &str) -> Option<String> {
        match self {
            SubNameRDF::SubName { .. } => Some(self.get(&key)?),
            _ => None,
        }
    }
}

impl fmt::Display for SubNameRDF {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SubNameRDF::Definition(def_data) => Ok(write!(fmt, "{:?}", def_data)?),
            SubNameRDF::SubName(map) => Ok(write!(fmt, "{:?}", map)?),
        }
    }
}

// The default for a sub name can be unset (NotSet), reference to the same mapping as
// another existing sub name (ExistingRdf), or just a different mapping (OtherRdf)
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum DefaultRdf {
    NotSet,
    ExistingRdf(SubName), // Not supported yet
    OtherRdf(DefinitionData),
}

impl std::default::Default for DefaultRdf {
    fn default() -> Self {
        DefaultRdf::NotSet
    }
}

// Each PublicName contains metadata and the link to the target's XOR-URL
pub type SubNamesMap = BTreeMap<SubName, SubNameRDF>;

// To use for mapping sub names to PublicNames
#[derive(Debug, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct NrsMap {
    pub sub_names_map: SubNamesMap,
    pub default: DefaultRdf,
}

impl NrsMap {
    #[allow(dead_code)]
    pub fn get_default(&self) -> ResultReturn<&DefaultRdf> {
        Ok(&self.default)
    }

    pub fn resolve_for_subnames(&self, mut sub_names: Vec<String>) -> ResultReturn<XorUrl> {
        debug!("NRS: Attempting to resolve for subnames {:?}", sub_names);
        let mut nrs_map = self;
        let mut link = None;

        while !sub_names.is_empty() {
            let curr_sub_name = sub_names
                .pop()
                .ok_or_else(|| Error::Unexpected("Failed to parse NRS name".to_string()))?;

            match nrs_map.sub_names_map.get(&curr_sub_name) {
                Some(SubNameRDF::SubName(nrs_sub_map)) => {
                    if nrs_sub_map.sub_names_map.is_empty() {
                        // we need default one then
                        if let DefaultRdf::OtherRdf(def_data) = &nrs_sub_map.default {
                            debug!("NRS subname resolution done. Located: \"{:?}\"", def_data);
                            link = def_data.get(FAKE_RDF_PREDICATE_LINK);
                        } else {
                            return Err(Error::ContentError(
                                "Sub name not found in NRS Map Container".to_string(),
                            ));
                        }
                    }
                    nrs_map = nrs_sub_map;
                }
                Some(SubNameRDF::Definition(def_data)) => {
                    debug!("NRS subname resolution done. Located: \"{:?}\"", def_data);
                    if sub_names.is_empty() {
                        // cool, we've gone through all subnames and we found a Definition (tree leaf)
                        link = def_data.get(FAKE_RDF_PREDICATE_LINK);
                    } else {
                        // oops...we haven't gone through all subnames and we reached a Definition (tree leaf)
                        return Err(Error::ContentError(
                            "Not all sub names were found in NRS Map Container".to_string(),
                        ));
                    };
                }
                None => {
                    if sub_names.is_empty() {
                        // we need default one then
                        if let DefaultRdf::OtherRdf(def_data) = &nrs_map.default {
                            debug!("NRS subname resolution done. Located: \"{:?}\"", def_data);
                            link = def_data.get(FAKE_RDF_PREDICATE_LINK);
                        } else {
                            return Err(Error::ContentError(
                                "Sub name not found in NRS Map Container".to_string(),
                            ));
                        }
                    } else {
                        return Err(Error::ContentError(
                            "Sub name not found in NRS Map Container".to_string(),
                        ));
                    };
                }
            };
        }

        match link {
            Some(the_link) => Ok(the_link.to_string()),
            None => Err(Error::ContentError(format!(
                "No link found for subnames: {:?}.",
                &sub_names.reverse()
            ))),
        }
    }

    pub fn get_default_link(&self) -> ResultReturn<XorUrl> {
        debug!("Attempting to get default link vis NRS....");
        let mut dereferenced_link: String;
        let link = match &self.default {
            DefaultRdf::NotSet => {
                return Err(Error::ContentError(
                    "No default found for resolvable map.".to_string(),
                ))
            }
            DefaultRdf::OtherRdf(def_data) => def_data.get(FAKE_RDF_PREDICATE_LINK),
            DefaultRdf::ExistingRdf(sub_name) => match self.sub_names_map.get(sub_name) {
                Some(entry) => match entry {
                    SubNameRDF::Definition(def_data) => def_data.get(FAKE_RDF_PREDICATE_LINK),
                    SubNameRDF::SubName(nrs_sub_name) => {
                        warn!("Attempting to get a default link from a nested subname.");
                        // FIXME: we need to stop looping if there is a crossed ref with defaults
                        dereferenced_link = nrs_sub_name.get_default_link()?;
                        Some(&dereferenced_link)
                    }
                },
                None => {
                    return Err(Error::ContentError(
                        "Default found in resolvable map seems corrupted.".to_string(),
                    ))
                }
            },
        }
        .ok_or_else(|| {
            Error::ContentError(format!(
                "No link found for default entry: {:?}.",
                self.default
            ))
        })?;

        debug!("Default link retrieved: \"{}\"", link);
        Ok(link.to_string())
    }

    #[allow(dead_code)]
    pub fn get_link_for(&self, sub_name: &str) -> ResultReturn<XorUrl> {
        let the_entry = self.sub_names_map.get(sub_name);

        let link = match the_entry {
            Some(entry) => entry.get(FAKE_RDF_PREDICATE_LINK),
            None => {
                return Err(Error::ContentError(format!(
                    "No entry \"{}\" found for resolvable map.",
                    &sub_name
                )))
            }
        };
        match link {
            Some(the_link) => Ok(the_link.to_string()),
            None => Err(Error::ContentError(format!(
                "No link found for entry: {}.",
                &sub_name
            ))),
        }
    }
}

// Raw data stored in the SAFE native data type for a NRS Map Container
type NrsMapRawData = Vec<(Vec<u8>, Vec<u8>)>;

// List of public names uploaded with details if they were added, updated or deleted from NrsMaps
type ProcessedEntries = BTreeMap<String, (String, String)>;

#[allow(dead_code)]
impl Safe {
    pub fn parse_url(&self, url: &str) -> ResultReturn<XorUrlEncoder> {
        debug!("Attempting to decode url: {}", url);
        XorUrlEncoder::from_url(url).or_else(|err| {
            info!(
                "Falling back to NRS. XorUrl decoding failed with: {:?}",
                err
            );

            let (sub_names, host_str, path) = get_subnames_host_and_path(url)?;
            let hashed_host = xorname_from_nrs_string(&host_str)?;

            let encoded_xor = XorUrlEncoder::new(
                hashed_host,
                NRS_MAP_TYPE_TAG,
                SafeDataType::PublishedSeqAppendOnlyData,
                SafeContentType::NrsMapContainer,
                Some(&path),
                Some(sub_names),
            );

            Ok(encoded_xor)
        })
    }

    pub fn nrs_map_container_add(
        &mut self,
        name: &str,
        destination: Option<&str>,
        default: bool,
        dry_run: bool,
    ) -> ResultReturn<(u64, XorUrl, ProcessedEntries, NrsMap)> {
        info!("Adding to NRS map...");
        // GET current NRS map from name's TLD
        let xorurl_encoder = self.parse_url(&sanitised_nrs_url(name))?;
        let xorurl = xorurl_encoder.to_string("")?;
        let (version, nrs_map) = self.nrs_map_container_get_latest(&xorurl)?;
        debug!("NRS, Existing data: {:?}", nrs_map);

        let (_, processed_entries, resulting_nrs_map, nrs_map_raw_data) =
            nrs_map_update_or_create_data(name, destination, Some(nrs_map), default)?;

        debug!("The new dataaaaa..... {:?}", resulting_nrs_map);
        if !dry_run {
            // Append new version of the NrsMap in the Published AppendOnlyData (NRS Map Container)
            self.safe_app.append_seq_append_only_data(
                nrs_map_raw_data,
                version + 1,
                xorurl_encoder.xorname(),
                xorurl_encoder.type_tag(),
            )?;
        }

        Ok((version + 1, xorurl, processed_entries, resulting_nrs_map))
    }

    /// # Create a NrsMapContainer.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use rand::distributions::Alphanumeric;
    /// # use rand::{thread_rng, Rng};
    /// # use unwrap::unwrap;
    /// # use safe_cli::Safe;
    /// # let mut safe = Safe::new("base32z".to_string());
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    /// let (xorurl, _processed_entries, nrs_map_container) = safe.nrs_map_container_create(&rand_string, Some("safe://somewhere"), true, false).unwrap();
    /// assert!(xorurl.contains("safe://"))
    /// ```
    pub fn nrs_map_container_create(
        &mut self,
        name: &str,
        destination: Option<&str>,
        default: bool,
        dry_run: bool,
    ) -> ResultReturn<(XorUrl, ProcessedEntries, NrsMap)> {
        info!("Creating an NRS map");
        if self
            .nrs_map_container_get_latest(&sanitised_nrs_url(name))
            .is_ok()
        {
            Err(Error::ContentError(
                "NRS name already exists. Please use 'nrs add' command to add sub names to it"
                    .to_string(),
            ))
        } else {
            let (nrs_xorname, processed_entries, nrs_map, nrs_map_raw_data) =
                nrs_map_update_or_create_data(&name, destination, None, default)?;

            if dry_run {
                Ok(("".to_string(), processed_entries, nrs_map))
            } else {
                // Store the NrsMapContainer in a Published AppendOnlyData
                let xorname = self.safe_app.put_seq_append_only_data(
                    nrs_map_raw_data,
                    Some(nrs_xorname),
                    NRS_MAP_TYPE_TAG,
                    None,
                )?;

                let xorurl = XorUrlEncoder::encode(
                    xorname,
                    NRS_MAP_TYPE_TAG,
                    SafeDataType::PublishedSeqAppendOnlyData,
                    SafeContentType::NrsMapContainer,
                    None,
                    None,
                    &self.xorurl_base,
                )?;

                Ok((xorurl, processed_entries, nrs_map))
            }
        }
    }

    pub fn nrs_map_container_remove(
        &mut self,
        name: &str,
        dry_run: bool,
    ) -> ResultReturn<(u64, XorUrl, ProcessedEntries, NrsMap)> {
        info!("Removing from NRS map...");
        // GET current NRS map from &name TLD
        let xorurl_encoder = self.parse_url(&sanitised_nrs_url(name))?;
        let xorurl = xorurl_encoder.to_string("")?;
        let (version, nrs_map) = self.nrs_map_container_get_latest(&xorurl)?;
        debug!("NRS, Existing data: {:?}", nrs_map);

        let (_, processed_entries, resulting_nrs_map, nrs_map_raw_data) =
            nrs_map_remove_subname(name, nrs_map)?;

        debug!("The new dataaaaa..... {:?}", resulting_nrs_map);
        if !dry_run {
            // Append new version of the NrsMap in the Published AppendOnlyData (NRS Map Container)
            self.safe_app.append_seq_append_only_data(
                nrs_map_raw_data,
                version + 1,
                xorurl_encoder.xorname(),
                xorurl_encoder.type_tag(),
            )?;
        }

        Ok((version + 1, xorurl, processed_entries, resulting_nrs_map))
    }

    /// # Fetch an existing NrsMapContainer.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use safe_cli::Safe;
    /// # use rand::distributions::Alphanumeric;
    /// # use rand::{thread_rng, Rng};
    /// # let mut safe = Safe::new("base32z".to_string());
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    /// let (xorurl, _processed_entries, _nrs_map) = safe.nrs_map_container_create(&rand_string, Some("somewhere"), true, false).unwrap();
    /// let (version, nrs_map_container) = safe.nrs_map_container_get_latest(&xorurl).unwrap();
    /// assert_eq!(version, 1);
    /// assert_eq!(nrs_map_container.get_default_link().unwrap(), "somewhere");
    /// ```
    pub fn nrs_map_container_get_latest(&self, url: &str) -> ResultReturn<(u64, NrsMap)> {
        debug!("Getting latest resolvable map container from: {:?}", url);

        let xorurl_encoder = self.parse_url(url)?;
        match self
            .safe_app
            .get_latest_seq_append_only_data(xorurl_encoder.xorname(), NRS_MAP_TYPE_TAG)
        {
            Ok((version, (_key, value))) => {
                debug!("Nrs map retrieved.... v{:?}, value {:?} ", &version, &value);
                // TODO: use RDF format and deserialise it
                let nrs_map = serde_json::from_str(&String::from_utf8_lossy(&value.as_slice()))
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
            Err(err) => Err(Error::NetDataError(format!(
                "Failed to get current version: {}",
                err
            ))),
        }
    }
}

fn xorname_from_nrs_string(name: &str) -> ResultReturn<XorName> {
    let vec_hash = sha3_256(&name.to_string().into_bytes());
    let xorname = XorName(vec_hash);
    debug!("Resulting XorName for NRS \"{}\" is: {}", name, xorname);
    Ok(xorname)
}

fn create_public_name_description(destination: &str) -> ResultReturn<DefinitionData> {
    let now = gen_timestamp_secs();
    let mut public_name = DefinitionData::new();
    public_name.insert(FAKE_RDF_PREDICATE_LINK.to_string(), destination.to_string());
    public_name.insert(FAKE_RDF_PREDICATE_MODIFIED.to_string(), now.clone());
    public_name.insert(FAKE_RDF_PREDICATE_CREATED.to_string(), now.clone());
    Ok(public_name)
}

fn sanitised_nrs_url(name: &str) -> String {
    // FIXME: make sure we remove the starting 'safe://'
    format!("safe://{}", name.replace("safe://", ""))
}

fn parse_nrs_name(name: &str) -> ResultReturn<(XorName, Vec<String>)> {
    // santize to a simple string
    let sanitized_name = str::replace(&name, "safe://", "").to_string();

    let mut sub_names: Vec<String> = sanitized_name.split('.').map(String::from).collect();
    // get the TLD
    let top_level_name = sub_names
        .pop()
        .ok_or_else(|| Error::Unexpected("Failed to parse NRS name".to_string()))?;

    let nrs_xorname = xorname_from_nrs_string(&top_level_name)?;
    debug!(
        "XorName for \"{:?}\" is \"{:?}\"",
        &top_level_name, &nrs_xorname
    );

    Ok((nrs_xorname, sub_names))
}

fn gen_nrs_map_raw_data(nrs_map: &NrsMap) -> ResultReturn<NrsMapRawData> {
    // The NrsMapContainer is an AppendOnlyData where each NRS Map version is an entry containing
    // the timestamp as the entry's key, and the serialised NrsMap as the entry's value
    // TODO: use RDF format
    let serialised_nrs_map = serde_json::to_string(nrs_map).map_err(|err| {
        Error::Unexpected(format!(
            "Couldn't serialise the NrsMap generated: {:?}",
            err
        ))
    })?;
    let now = gen_timestamp_secs();

    Ok(vec![(
        now.into_bytes().to_vec(),
        serialised_nrs_map.as_bytes().to_vec(),
    )])
}

fn nrs_map_remove_subname(
    name: &str,
    nrs_map: NrsMap,
) -> ResultReturn<(XorName, ProcessedEntries, NrsMap, NrsMapRawData)> {
    info!("Removing sub name \"{}\" from NRS map", name);
    let (nrs_xorname, sub_names) = parse_nrs_name(name)?;

    // let's walk the NRS Map tree to find the sub name we need to remove
    let (updated_nrs_map, link) = remove_nrs_sub_tree(&nrs_map, sub_names)?;

    let mut processed_entries = ProcessedEntries::new();
    processed_entries.insert(name.to_string(), (CONTENT_DELETED_SIGN.to_string(), link));

    let nrs_map_raw_data = gen_nrs_map_raw_data(&updated_nrs_map)?;
    Ok((nrs_xorname, processed_entries, nrs_map, nrs_map_raw_data))
}

fn nrs_map_update_or_create_data(
    name: &str,
    destination: Option<&str>,
    existing_map: Option<NrsMap>,
    default: bool,
) -> ResultReturn<(XorName, ProcessedEntries, NrsMap, NrsMapRawData)> {
    info!("Creating or updating NRS map for: {}", name);

    let nrs_map = existing_map.unwrap_or_else(NrsMap::default);
    let (nrs_xorname, sub_names) = parse_nrs_name(name)?;
    let link = destination.unwrap_or_else(|| "");

    // Update NRS Map with new names
    let mut updated_nrs_map = setup_nrs_tree(&nrs_map, sub_names, link)?;

    // Set (top level) default if was requested
    if default {
        debug!("Setting {:?} as default for NrsMap", &name);
        // TODO: support DefaultRdf::ExistingRdf
        let definition_data = create_public_name_description(link)?;
        updated_nrs_map.default = DefaultRdf::OtherRdf(definition_data);
    }

    let mut processed_entries = ProcessedEntries::new();
    processed_entries.insert(
        name.to_string(),
        (CONTENT_ADDED_SIGN.to_string(), link.to_string()),
    );

    let nrs_map_raw_data = gen_nrs_map_raw_data(&updated_nrs_map)?;
    Ok((
        nrs_xorname,
        processed_entries,
        updated_nrs_map,
        nrs_map_raw_data,
    ))
}

// fix: default of subnames (e.g. c.n) not set when adding
fn setup_nrs_tree(
    nrs_map: &NrsMap,
    mut sub_names: Vec<String>,
    link: &str,
) -> ResultReturn<NrsMap> {
    let mut updated_nrs_map = nrs_map.clone();
    let curr_sub_name = if sub_names.is_empty() {
        let definition_data = create_public_name_description(link)?;
        updated_nrs_map.default = DefaultRdf::OtherRdf(definition_data);
        return Ok(updated_nrs_map);
    } else {
        sub_names
            .pop()
            .ok_or_else(|| Error::Unexpected("Failed to generate NRS Map".to_string()))?
    };

    match nrs_map.sub_names_map.get(&curr_sub_name) {
        Some(SubNameRDF::SubName(nrs_sub_map)) => {
            let updated_sub_map = setup_nrs_tree(nrs_sub_map, sub_names, link)?;
            updated_nrs_map
                .sub_names_map
                .insert(curr_sub_name, SubNameRDF::SubName(updated_sub_map));
            Ok(updated_nrs_map)
        }
        Some(SubNameRDF::Definition(def_data)) => {
            // we need to add the new sub nrs tree but as a sibling
            let mut new_nrs_map = NrsMap::default();
            new_nrs_map.default = DefaultRdf::OtherRdf(def_data.clone());
            let updated_new_nrs_map = setup_nrs_tree(&new_nrs_map, sub_names, link)?;
            updated_nrs_map
                .sub_names_map
                .insert(curr_sub_name, SubNameRDF::SubName(updated_new_nrs_map));
            Ok(updated_nrs_map)
        }
        None => {
            // Sub name not found in NRS Map Container
            // we need to add the new sub nrs tree
            let new_nrs_map = NrsMap::default();
            let updated_new_nrs_map = setup_nrs_tree(&new_nrs_map, sub_names, link)?;
            updated_nrs_map
                .sub_names_map
                .insert(curr_sub_name, SubNameRDF::SubName(updated_new_nrs_map));
            Ok(updated_nrs_map)
        }
    }
}

fn remove_nrs_sub_tree(
    nrs_map: &NrsMap,
    mut sub_names: Vec<String>,
) -> ResultReturn<(NrsMap, String)> {
    let mut updated_nrs_map = nrs_map.clone();
    let curr_sub_name = if sub_names.is_empty() {
        match nrs_map.get_default()? {
            DefaultRdf::NotSet => {
                return Err(Error::ContentError(
                    "Sub name not found in NRS Map Container".to_string(),
                ))
            }
            DefaultRdf::OtherRdf(def_data) => {
                let link = match def_data.get(FAKE_RDF_PREDICATE_LINK) {
                    Some(link) => link.to_string(),
                    None => "".to_string(),
                };
                updated_nrs_map.default = DefaultRdf::NotSet;
                return Ok((updated_nrs_map, link));
            }
            DefaultRdf::ExistingRdf(sub_name) => sub_name.to_string(),
        }
    } else {
        sub_names
            .pop()
            .ok_or_else(|| Error::Unexpected("Failed to generate NRS Map".to_string()))?
    };

    match nrs_map.sub_names_map.get(&curr_sub_name) {
        Some(SubNameRDF::SubName(nrs_sub_map)) => {
            let (updated_sub_map, link) = remove_nrs_sub_tree(nrs_sub_map, sub_names)?;
            updated_nrs_map
                .sub_names_map
                .insert(curr_sub_name, SubNameRDF::SubName(updated_sub_map));
            Ok((updated_nrs_map, link))
        }
        Some(SubNameRDF::Definition(def_data)) => {
            println!("NRS subname resolution done. Located: \"{:?}\"", def_data);
            if sub_names.is_empty() {
                // cool, we've gone through all subnames and we found a Definition (tree leaf)
                let link = match def_data.get(FAKE_RDF_PREDICATE_LINK) {
                    Some(link) => link.to_string(),
                    None => "".to_string(),
                };
                let _ = updated_nrs_map.sub_names_map.remove(&curr_sub_name);
                if updated_nrs_map.default == DefaultRdf::ExistingRdf(curr_sub_name) {
                    // unset the default as it's currently pointing to the sub name being removed
                    updated_nrs_map.default = DefaultRdf::NotSet;
                }
                Ok((updated_nrs_map, link))
            } else {
                // oops...we haven't gone through all subnames and we reached a Definition (tree leaf)
                Err(Error::ContentError(
                    "Not all sub names were found in NRS Map Container".to_string(),
                ))
            }
        }
        None => Err(Error::ContentError(
            "Sub name not found in NRS Map Container".to_string(),
        )),
    }
}

// Unit Tests

#[test]
fn test_nrs_map_container_create() {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;

    let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", Some("fake-credentials")).unwrap();

    let nrs_xorname = xorname_from_nrs_string(&site_name).unwrap();

    let (xor_url, _entries, nrs_map) =
        unwrap!(safe.nrs_map_container_create(&site_name, Some("safe://top_xorurl"), true, false));
    assert_eq!(nrs_map.sub_names_map.len(), 0);

    if let DefaultRdf::OtherRdf(def_data) = &nrs_map.default {
        assert_eq!(
            *def_data.get(FAKE_RDF_PREDICATE_LINK).unwrap(),
            "safe://top_xorurl".to_string()
        );
        assert_eq!(
            nrs_map.get_default().unwrap(),
            &DefaultRdf::OtherRdf(def_data.clone())
        );
    } else {
        panic!("No default definition map found...")
    }

    let decoder = XorUrlEncoder::from_url(&xor_url).unwrap();
    assert_eq!(nrs_xorname, decoder.xorname())
}
