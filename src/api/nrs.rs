// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::constants::{
    CONTENT_ADDED_SIGN, /*CONTENT_DELETED_SIGN, CONTENT_ERROR_SIGN, CONTENT_UPDATED_SIGN,*/
    FAKE_RDF_PREDICATE_CREATED, FAKE_RDF_PREDICATE_DEFAULT, FAKE_RDF_PREDICATE_LINK,
    FAKE_RDF_PREDICATE_MODIFIED,
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

type PublicNameKey = String;
type NrsMapRawData = Vec<(Vec<u8>, Vec<u8>)>;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum PublicNameEntry {
    Definition(BTreeMap<String, String>),
    SubName(NrsMap),
}

impl PublicNameEntry {
    fn get(&self, key: &str) -> Option<String> {
        match self {
            PublicNameEntry::SubName { .. } => Some(self.get(&key)?),
            _ => None,
        }
    }
}

impl fmt::Display for PublicNameEntry {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PublicNameEntry::Definition(def_map) => Ok(write!(fmt, "{:?}", def_map)?),
            PublicNameEntry::SubName(map) => Ok(write!(fmt, "{:?}", map)?),
        }
    }
}

// Each PublicName contains metadata and the link to the target's XOR-URL
pub type PublicNameRDF = BTreeMap<PublicNameKey, PublicNameEntry>;

// To use for mapping sub names to PublicNames
#[derive(Debug, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct NrsMap {
    pub entries: PublicNameRDF,
    pub default: PublicNameKey,
}

impl NrsMap {
    #[allow(dead_code)]
    pub fn get_default(&self) -> ResultReturn<&str> {
        Ok(&self.default)
    }

    pub fn resolve_for_subnames(&self, mut sub_name_list: Vec<String>) -> ResultReturn<XorUrl> {
        debug!(
            "NRS: Attempting to resolve for subnames {:?}",
            sub_name_list
        );
        let mut nrs_map = self;
        let mut link = None;

        sub_name_list.reverse();
        for (_i, the_sub_name) in sub_name_list.iter().enumerate() {
            let next = nrs_map.entries.get(the_sub_name);

            if let Some(PublicNameEntry::SubName(nrs_sub_name)) = &next {
                nrs_map = nrs_sub_name;
            }

            if let Some(PublicNameEntry::Definition(def_map)) = next {
                debug!("NRS subname resolution done. Located: \"{:?}\"", def_map);
                link = def_map.get(FAKE_RDF_PREDICATE_LINK);
            }
        }

        match link {
            Some(the_link) => Ok(the_link.to_string()),
            None => Err(Error::ContentError(format!(
                "No link found for subnames: {:?}.",
                &sub_name_list.reverse()
            ))),
        }
    }

    pub fn get_default_link(&self) -> ResultReturn<XorUrl> {
        debug!("Attempting to get default link vis NRS....");
        let default = &self.default;

        let default_entry = self.entries.get(default);
        debug!("NRS found default link target: {:?}", &default_entry);

        let mut dereferenced_link: String;
        let link = match default_entry {
            Some(entry) => match entry {
                PublicNameEntry::Definition(def_map) => def_map.get(FAKE_RDF_PREDICATE_LINK),
                PublicNameEntry::SubName(nrs_sub_name) => {
                    warn!("Attempting to get a default link from a nested subname.");

                    dereferenced_link = nrs_sub_name.get_default_link()?;
                    Some(&dereferenced_link)
                }
            },
            None => {
                return Err(Error::ContentError(
                    "No default found for resolvable map.".to_string(),
                ))
            }
        };

        match link {
            Some(the_link) => {
                warn!("Default link retrieved: \"{:?}\"", the_link);
                Ok(the_link.to_string())
            }
            None => Err(Error::ContentError(format!(
                "No link found for default entry: {}.",
                &default
            ))),
        }
    }

    #[allow(dead_code)]
    pub fn get_link_for(&self, sub_name: &str) -> ResultReturn<XorUrl> {
        let the_entry = self.entries.get(sub_name);

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

// List of public names uploaded with details if they were added, updated or deleted from NrsMaps
type ProcessedEntries = BTreeMap<String, (String, String)>;

fn xorname_from_nrs_string(name: &str) -> ResultReturn<XorName> {
    let vec_hash = sha3_256(&name.to_string().into_bytes());
    let xorname = XorName(vec_hash);
    debug!("Resulting XorName for NRS: {} is, {}", name, xorname);
    Ok(xorname)
}

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
        // GET current NRS map from &name TLD
        // NOT via normal fetch
        let xorurl_encoder = self.parse_url(&sanitised_nrs_url(name))?;
        let xorurl = xorurl_encoder.to_string("")?;
        let (version, nrs_map) = self.nrs_map_container_get_latest(&xorurl)?;
        debug!("NRS, Existing data: {:?}", nrs_map);
        let (_, resolvable_container_raw_data, processed_entries, resulting_nrs_map) =
            nrs_map_update_or_create_data(name, destination, Some(nrs_map), default, dry_run)?;

        info!("The new dataaaaa..... {:?}", resulting_nrs_map);
        // Append new version of the NrsMap in the Published AppendOnlyData (NRS Map Container)
        let new_version = self.safe_app.append_seq_append_only_data(
            resolvable_container_raw_data,
            version + 1,
            xorurl_encoder.xorname(),
            xorurl_encoder.type_tag(),
        )?;

        Ok((new_version, xorurl, processed_entries, resulting_nrs_map))
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
        debug!("Creating an NRS map");
        match self.nrs_map_container_get_latest(&sanitised_nrs_url(name)) {
            Ok(_) => Err(Error::ContentError(
                "NRS name already exists. Please use 'nrs add' command to add sub names to it"
                    .to_string(),
            )),
            Err(_) => {
                let (nrs_xorname, resolvable_container_raw_data, processed_entries, nrs_map) =
                    nrs_map_update_or_create_data(&name, destination, None, default, dry_run)?;

                // Store the NrsMapContainer in a Published AppendOnlyData
                let xorname = self.safe_app.put_seq_append_only_data(
                    resolvable_container_raw_data,
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
    /// assert_eq!(nrs_map_container.get_default().unwrap(), "_default");
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

fn create_public_name_description(destination: &str) -> ResultReturn<PublicNameEntry> {
    let now = gen_timestamp_secs();

    let mut public_name = BTreeMap::new();

    public_name.insert(FAKE_RDF_PREDICATE_LINK.to_string(), destination.to_string());

    public_name.insert(FAKE_RDF_PREDICATE_MODIFIED.to_string(), now.clone());
    public_name.insert(FAKE_RDF_PREDICATE_CREATED.to_string(), now.clone());

    Ok(PublicNameEntry::Definition(public_name))
}

fn sanitised_nrs_url(name: &str) -> String {
    format!("safe://{}", name.replace("safe://", ""))
}

fn nrs_map_update_or_create_data(
    name: &str,
    destination: Option<&str>,
    existing_map: Option<NrsMap>,
    default: bool,
    _dry_run: bool,
) -> ResultReturn<(XorName, NrsMapRawData, ProcessedEntries, NrsMap)> {
    info!("Creating or updating an NRS map");

    // TODO: This should always add/overwrite existing entries at the moment

    // santize to a simple string
    let sanitised_name = str::replace(name, "safe://", "").to_string();
    let mut nrs_map = existing_map.unwrap_or_else(NrsMap::default);

    let name_vec: Vec<String> = sanitised_name.split('.').map(String::from).collect();
    // get the TLD
    let top_level_name = &name_vec[name_vec.len() - 1];
    // subnames
    let the_rest_sub_names = &name_vec[0..name_vec.len() - 1];
    // reverse list for resolving existing subname trees
    let mut the_reverse_sub_names = the_rest_sub_names.to_vec();
    the_reverse_sub_names.reverse();

    // by default top subname is...
    let top_subname = if !the_rest_sub_names.is_empty() {
        &the_rest_sub_names[the_rest_sub_names.len() - 1]
    } else {
        FAKE_RDF_PREDICATE_DEFAULT
    };

    let nrs_xorname = xorname_from_nrs_string(&top_level_name)?;

    debug!(
        "XorName for \"{:?}\" is \"{:?}\"",
        &top_level_name, &nrs_xorname
    );

    let final_destination = destination.unwrap_or_else(|| "");

    let mut public_name_rdf = create_public_name_description(final_destination)?;

    // TODO : Check if current subname exists...
    // check if its a description...
    // if so move that to default of next?
    // what if new one should be default?? .......

    // throw error if existing?

    debug!("Sub name target data: {:?}", public_name_rdf);

    // a.b.c

    // IN C

    // UPDATING a.b.c

    // subname A...
    if !the_rest_sub_names.is_empty() {
        debug!("Subnames will be added...");
        let mut prev_subname: &str = "";

        // let mut existing_subname_target = 0;
        let mut testing_map = PublicNameEntry::SubName(nrs_map.clone());

        // let's build a map of exiting subnames related to our target...
        let mut existing_tree: BTreeMap<usize, NrsMap> = BTreeMap::new();
        for (i, existing_sub_name) in the_reverse_sub_names.iter().enumerate() {
            debug!("Checking if subname already exists.");
            let test2_map = testing_map.clone();
            let actual_map = match test2_map {
                PublicNameEntry::SubName(sub_map) => sub_map,
                _ => NrsMap::default(),
            };

            if actual_map
                .entries
                .contains_key(&existing_sub_name.to_string())
            {
                debug!("{} already exists", existing_sub_name);
                let target_map = actual_map
                    .entries
                    .get(&existing_sub_name.to_string())
                    .ok_or_else(|| {
                        Error::ContentNotFound("Could not find subname in question".to_string())
                    })?;

                let another_default = NrsMap::default();
                // TODO: Impl get underlying map func.
                let target2_map = target_map.clone();
                let actual_target_map = match target2_map {
                    PublicNameEntry::SubName(sub_map) => sub_map,
                    _ => another_default,
                };
                let actual_target2 = actual_target_map.clone();
                existing_tree.insert(the_reverse_sub_names.len() - i, actual_target_map);
                let the_public_entry = PublicNameEntry::SubName(actual_target2);
                testing_map = the_public_entry;
            }
        }

        // let's loop through subnames from lowest up, building our subname tree...
        for (i, the_sub_name) in the_rest_sub_names.iter().enumerate() {
            debug!("Subname {} is {}", i + 1, &the_sub_name);

            // let mut existing_subname_target = public_name_rdf;

            // WE ONLY NEED TO KNOW IF IT EXISTS ALREADY!?!!!
            // OR AT WHAT LEVEL IT EXISTS ALREADY
            // let existing_sub_name_map = existing_tree[i];

            let default = NrsMap::default();
            // use the existing map at this level...

            let sub_nrs_map = match existing_tree.get(&i) {
                Some(map) => map,
                None => &default,
            };
            // .unwrap_or_else( || &mut default );

            if i == 0 {
                prev_subname = the_sub_name;
            }

            // if we have other subnames, we add them all up
            if i > 0 {
                let mut our_map = sub_nrs_map.clone();
                our_map
                    .entries
                    .insert(prev_subname.to_string(), public_name_rdf);
                our_map.default = prev_subname.to_string();

                public_name_rdf = PublicNameEntry::SubName(our_map);
            }
        }
    }

    // if we're not touching default
    // if top_subname != FAKE_RDF_PREDICATE_LINK && nrs_map.entries.contains_key(&top_subname) {
    // 	return Err(Error::ContentError(format!("The subname \"{}\" already exists", &top_subname) ))
    // }

    // if you have only default, and add a new default...
    nrs_map
        .entries
        .insert(top_subname.to_string(), public_name_rdf);

    // TODO: Enable source for funds / ownership

    // The NrsMapContainer is created as a AppendOnlyData with a single entry containing the
    // timestamp as the entry's key, and the serialised NrsMap as the entry's value
    // TODO: use RDF format

    debug!("Subname inserted with name {:?}", &top_subname);

    if default {
        debug!("Setting {:?} as default for NrsMap", &name);

        nrs_map.default = top_subname.to_string();
    }

    let mut processed_entries: BTreeMap<String, (String, String)> = BTreeMap::new();
    processed_entries.insert(
        name.to_string(),
        (
            CONTENT_ADDED_SIGN.to_string(),
            final_destination.to_string(),
        ),
    );

    let serialised_nrs_map = serde_json::to_string(&nrs_map).map_err(|err| {
        Error::Unexpected(format!(
            "Couldn't serialise the NrsMap generated: {:?}",
            err
        ))
    })?;
    let now = gen_timestamp_secs();
    let resolvable_container_raw_data = vec![(
        now.into_bytes().to_vec(),
        serialised_nrs_map.as_bytes().to_vec(),
    )];

    Ok((
        nrs_xorname,
        resolvable_container_raw_data,
        processed_entries,
        nrs_map,
    ))
}

// Unit Tests

#[test]
fn test_nrs_map_container_create() {
    use super::nrs::PublicNameEntry;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use unwrap::unwrap;

    let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();

    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", Some("fake-credentials")).unwrap();

    let nrs_xorname = xorname_from_nrs_string(&site_name).unwrap();

    let (xor_url, _entries, nrs_map) =
        unwrap!(safe.nrs_map_container_create(&site_name, Some("safe://top_xorurl"), true, false));
    assert_eq!(nrs_map.entries.len(), 1);

    let public_name = &nrs_map.entries[FAKE_RDF_PREDICATE_DEFAULT];

    if let PublicNameEntry::Definition(def_map) = public_name {
        assert_eq!(
            *def_map.get(FAKE_RDF_PREDICATE_LINK).unwrap(),
            "safe://top_xorurl".to_string()
        );
    } else {
        panic!("No definition map found...")
    }

    assert_eq!(nrs_map.get_default().unwrap(), FAKE_RDF_PREDICATE_DEFAULT);

    let decoder = XorUrlEncoder::from_url(&xor_url).unwrap();
    assert_eq!(nrs_xorname, decoder.xorname())
}
