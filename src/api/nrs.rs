// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::constants::{
    CONTENT_ADDED_SIGN, /*CONTENT_DELETED_SIGN, CONTENT_ERROR_SIGN, CONTENT_UPDATED_SIGN,*/
    FAKE_RDF_PREDICATE_CREATED, FAKE_RDF_PREDICATE_LINK, FAKE_RDF_PREDICATE_MODIFIED,
};

use super::helpers::gen_timestamp_secs;
use super::xorurl::{SafeContentType, SafeDataType};
use super::{Error, ResultReturn, Safe, XorUrl, XorUrlEncoder};
use log::{debug, warn};
use safe_nd::XorName;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tiny_keccak::sha3_256;

// Type tag to use for the FilesContainer stored on AppendOnlyData
pub const NRS_MAP_TYPE_TAG: u64 = 1500;

const ERROR_MSG_NO_NRS_MAP_FOUND: &str = "No NRS Map found at this address";

// Each PublicName contains metadata and the link to the target's XOR-URL
pub type PublicName = BTreeMap<String, String>;

// To use for mapping domain names (with path in a flattened hierarchy) to PublicNames
#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct NrsMap {
    // #[derive(PartialEq)]
    pub entries: BTreeMap<String, PublicName>,
    pub default: String,
}

impl NrsMap {
    #[allow(dead_code)]
    pub fn get_default(&self) -> ResultReturn<&str> {
        Ok(&self.default)
    }
    pub fn get_default_link(&self) -> ResultReturn<XorUrl> {
        let default = &self.default;

        let default_entry = self.entries.get(default);

        let link = match default_entry {
            Some(entry) => entry.get(FAKE_RDF_PREDICATE_LINK),
            None => {
                return Err(Error::ContentError(
                    "No default found for resolvable map.".to_string(),
                ))
            }
        };

        match link {
            Some(the_link) => Ok(the_link.to_string()),
            None => Err(Error::ContentError(format!(
                "No link found for default entry: {}.",
                &default
            ))),
        }
    }
}

// List of public names uploaded with details if they were added, updated or deleted from NrsMaps
type ProcessedEntries = BTreeMap<String, (String, String)>;

pub fn xorname_from_nrs_string(name: &str) -> ResultReturn<XorName> {
    let vec_hash = sha3_256(&name.to_string().into_bytes());

    let xorname = XorName(vec_hash);
    debug!("Resulting XornName for NRS: {} is, {}", &name, &xorname);

    Ok(xorname)
}

#[allow(dead_code)]
impl Safe {
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
        _dry_run: bool,
    ) -> ResultReturn<(XorUrl, ProcessedEntries, NrsMap)> {
        let sanitized_name = str::replace(&name, "safe://", "").to_string();

        let nrs_xorname = xorname_from_nrs_string(&sanitized_name)?;

        debug!("XorName for \"{:?}\" is \"{:?}\"", &name, &nrs_xorname);

        let final_destination = destination.unwrap_or_else(|| "");
        // TODO: Enable source for funds / ownership

        // The NrsMapContainer is created as a AppendOnlyData with a single entry containing the
        // timestamp as the entry's key, and the serialised NrsMap as the entry's value
        // TODO: use RDF format
        let nrs_map = nrs_map_create(&name, &final_destination, default)?;

        let mut processed_entries = BTreeMap::new();
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
        let resolvable_container_data = vec![(
            now.into_bytes().to_vec(),
            serialised_nrs_map.as_bytes().to_vec(),
        )];

        // Store the NrsMapContainer in a Published AppendOnlyData
        let xorname = self.safe_app.put_seq_append_only_data(
            resolvable_container_data,
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
            &self.xorurl_base,
        )?;

        Ok((xorurl, processed_entries, nrs_map))
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
    /// assert_eq!(nrs_map_container.entries[&rand_string]["link"], "somewhere");
    /// assert_eq!(nrs_map_container.get_default_link().unwrap(), "somewhere");
    /// assert_eq!(nrs_map_container.get_default().unwrap(), &rand_string);
    /// ```
    pub fn nrs_map_container_get_latest(&self, xorurl: &str) -> ResultReturn<(u64, NrsMap)> {
        debug!("Getting latest resolvable map container from: {:?}", xorurl);

        let xorurl_encoder = XorUrlEncoder::from_url(xorurl)?;
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
                warn!("Nrs container found at {:?} was empty", &xorurl);
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

// From the provided list of resolovable public names
// create a NrsMap with metadata and their corresponding links
fn nrs_map_create(name: &str, destination: &str, set_as_defualt: bool) -> ResultReturn<NrsMap> {
    let mut nrs_map = NrsMap::default();
    let now = gen_timestamp_secs();

    // TODO: Split name w/ .

    debug!("NrsMap for name:{:?}", &name);
    let mut public_name = PublicName::new();

    public_name.insert(FAKE_RDF_PREDICATE_LINK.to_string(), destination.to_string());

    public_name.insert(FAKE_RDF_PREDICATE_MODIFIED.to_string(), now.clone());
    public_name.insert(FAKE_RDF_PREDICATE_CREATED.to_string(), now.clone());
    public_name.insert(FAKE_RDF_PREDICATE_CREATED.to_string(), now.clone());

    debug!("PublicName: {:?}", public_name);

    debug!("PublicName inserted with name {:?}", &name);
    nrs_map.entries.insert(name.to_string(), public_name);

    if set_as_defualt {
        debug!("Setting {:?} as default for NrsMap", &name);

        nrs_map.default = name.to_string();
    }

    Ok(nrs_map)
}

// Unit Tests

#[test]
fn test_nrs_map_container_create() {
    use unwrap::unwrap;

    let mut safe = Safe::new("base32z".to_string());
    safe.connect("", Some("fake-credentials")).unwrap();

    let nrs_xorname = xorname_from_nrs_string("some_site").unwrap();
    let site_name = "some_site";
    let (xor_url, _entries, nrs_map) =
        unwrap!(safe.nrs_map_container_create(site_name, Some("safe://top_xorurl"), true, false));
    assert_eq!(nrs_map.entries.len(), 1);
    let public_name = &nrs_map.entries[site_name];
    assert_eq!(public_name[FAKE_RDF_PREDICATE_LINK], "safe://top_xorurl");
    assert_eq!(nrs_map.get_default().unwrap(), site_name);

    let decoder = XorUrlEncoder::from_url(&xor_url).unwrap();
    assert_eq!(nrs_xorname, decoder.xorname())
}

#[test]
fn test_nrs_map_create() {
    use unwrap::unwrap;
    let _safe = Safe::new("base32z".to_string());
    let nrs_map = unwrap!(nrs_map_create("site1", "safe://top_xorurl", true));
    assert_eq!(nrs_map.entries.len(), 1);
    let public_name = &nrs_map.entries["site1"];
    assert_eq!(public_name[FAKE_RDF_PREDICATE_LINK], "safe://top_xorurl");
    assert_eq!(nrs_map.get_default().unwrap(), "site1");
    assert_eq!(nrs_map.get_default_link().unwrap(), "safe://top_xorurl");
}
