// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::constants::{
    FAKE_RDF_PREDICATE_CREATED, FAKE_RDF_PREDICATE_LINK, FAKE_RDF_PREDICATE_MODIFIED,
};
use super::helpers::gen_timestamp_secs;
use super::{Error, ResultReturn, Safe, SafeContentType, XorUrl};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::iter::FromIterator;

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
    ExistingRdf(SubName),
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
    pub fn get_default(&self) -> ResultReturn<&DefaultRdf> {
        Ok(&self.default)
    }

    pub fn resolve_for_subnames(&self, mut sub_names: Vec<SubName>) -> ResultReturn<XorUrl> {
        debug!("NRS: Attempting to resolve for subnames {:?}", sub_names);
        let mut nrs_map = self;
        let mut dereferenced_link: String;
        let sub_names_str = sub_names_vec_to_str(&sub_names);
        let mut link = if sub_names.is_empty() {
            match &self.default {
                DefaultRdf::OtherRdf(def_data) => {
                    debug!(
                        "NRS subname resolution done from default. Located: \"{:?}\"",
                        def_data
                    );
                    def_data.get(FAKE_RDF_PREDICATE_LINK)
                }
                DefaultRdf::ExistingRdf(sub_name) => {
                    let sub_names = Vec::from_iter(sub_name.split('.').map(String::from));
                    dereferenced_link = self.resolve_for_subnames(sub_names)?;
                    Some(&dereferenced_link)
                }
                DefaultRdf::NotSet => None,
            }
        } else {
            None
        };

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
                    return Err(Error::ContentError(
                        "Sub name not found in NRS Map Container".to_string(),
                    ));
                }
            };
        }

        match link {
            Some(the_link) => {
                // Let's make sure it's a versioned link
                validate_nrs_link(the_link)?;
                Ok(the_link.to_string())
            }
            None => Err(Error::ContentError(format!(
                "No link found for subname/s \"{}\"",
                sub_names_str
            ))),
        }
    }

    #[allow(dead_code)]
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
            DefaultRdf::ExistingRdf(sub_name) => {
                let sub_names = Vec::from_iter(sub_name.split('.').map(String::from));
                dereferenced_link = self.resolve_for_subnames(sub_names).map_err(|_| Error::ContentError(
                    format!("Default found for resolvable map (set to sub names '{}') cannot be resolved.", sub_name),
                ))?;
                Some(&dereferenced_link)
            }
        }
        .ok_or_else(|| {
            Error::ContentError(format!(
                "No link found for default entry: {:?}.",
                self.default
            ))
        })?;

        debug!("Default link retrieved: \"{}\"", link);
        // Let's make sure it's a versioned link
        validate_nrs_link(link)?;
        Ok(link.to_string())
    }

    pub fn nrs_map_remove_subname(&mut self, name: &str) -> ResultReturn<String> {
        info!("Removing sub name \"{}\" from NRS map", name);
        let sub_names = parse_nrs_name(name)?;

        // let's walk the NRS Map tree to find the sub name we need to remove
        let (updated_nrs_map, removed_link) = remove_nrs_sub_tree(&self, sub_names)?;
        self.sub_names_map = updated_nrs_map.sub_names_map;
        self.default = updated_nrs_map.default;

        Ok(removed_link)
    }

    pub fn nrs_map_update_or_create_data(
        &mut self,
        name: &str,
        link: &str,
        default: bool,
        hard_link: bool,
    ) -> ResultReturn<String> {
        info!("Updating NRS map for: {}", name);

        // NRS resolver doesn't allow unversioned links
        validate_nrs_link(link)?;

        // Update NRS Map with new names
        let sub_names: Vec<String> = parse_nrs_name(name)?;
        let updated_nrs_map = setup_nrs_tree(&self, sub_names.clone(), link)?;
        self.sub_names_map = updated_nrs_map.sub_names_map;

        // Set (top level) default if was requested
        if default {
            debug!("Setting {:?} as default for NrsMap", &name);
            let definition_data = create_public_name_description(link)?;
            if hard_link || sub_names.is_empty() {
                self.default = DefaultRdf::OtherRdf(definition_data);
            } else {
                let sub_names_str = sub_names_vec_to_str(&sub_names);
                self.default = DefaultRdf::ExistingRdf(sub_names_str);
            }
        } else {
            self.default = updated_nrs_map.default;
        }

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

fn create_public_name_description(link: &str) -> ResultReturn<DefinitionData> {
    let now = gen_timestamp_secs();
    let mut public_name = DefinitionData::new();
    public_name.insert(FAKE_RDF_PREDICATE_LINK.to_string(), link.to_string());
    public_name.insert(FAKE_RDF_PREDICATE_MODIFIED.to_string(), now.clone());
    public_name.insert(FAKE_RDF_PREDICATE_CREATED.to_string(), now.clone());
    Ok(public_name)
}

fn sub_names_vec_to_str(sub_names: &[SubName]) -> String {
    if !sub_names.is_empty() {
        let length = sub_names.len() - 1;
        sub_names
            .iter()
            .enumerate()
            .map(|(i, n)| {
                if i < length {
                    format!("{}.", n)
                } else {
                    n.to_string()
                }
            })
            .collect()
    } else {
        "".to_string()
    }
}

fn parse_nrs_name(name: &str) -> ResultReturn<Vec<String>> {
    // santize to a simple string
    let sanitized_name = str::replace(&name, "safe://", "").to_string();

    let mut sub_names: Vec<String> = sanitized_name.split('.').map(String::from).collect();
    // get the TLD
    let _ = sub_names
        .pop()
        .ok_or_else(|| Error::Unexpected("Failed to parse NRS name".to_string()))?;

    Ok(sub_names)
}

fn validate_nrs_link(link: &str) -> ResultReturn<()> {
    let link_encoder = Safe::parse_url(link)?;
    if link_encoder.content_version().is_none() {
        // We could try to automatically set the latest/current version,
        // but NRSMap currently doesn't have a connection to do so.
        match link_encoder.content_type() {
            SafeContentType::FilesContainer | SafeContentType::NrsMapContainer => {
                Err(Error::InvalidInput(format!(
                    "The link is unversioned, but the linked content is versionable. NRS resolver doesn\'t allow unversioned links for this type of content: \"{}\"",
                    link
                )))
            }
            _ => Ok(()),
        }
    } else {
        Ok(())
    }
}

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
            if updated_sub_map.sub_names_map.is_empty()
                && updated_sub_map.default == DefaultRdf::NotSet
            {
                // there are no more sub names at this level now, so let's remove it
                updated_nrs_map.sub_names_map.remove(&curr_sub_name);
            } else {
                updated_nrs_map
                    .sub_names_map
                    .insert(curr_sub_name, SubNameRDF::SubName(updated_sub_map));
            }
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
