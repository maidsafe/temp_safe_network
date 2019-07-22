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
use super::xorurl::SafeContentType;
use super::{Error, ResultReturn, Safe, XorUrl, XorUrlEncoder};
use safe_nd::XorName;
use serde::{Deserialize, Serialize};

use chrono::{SecondsFormat, Utc};
use log::{debug, warn};
use std::collections::BTreeMap;
use tiny_keccak::sha3_256;

// Type tag to use for the FilesContainer stored on AppendOnlyData
pub static RESOLVABLE_MAP_TYPE_TAG: u64 = 1500;
// Informative string of the SAFE native data type behind a FilesContainer
pub static RESOLVABLE_MAP_TYPE_TAG_NATIVE_TYPE: &str = "AppendOnlyData";

static ERROR_MSG_NO_RESOLVABLE_MAP_FOUND: &str = "No Resolvable Map found at this address";

// Each ResolvableItem contains item metadata and the link to the item's XOR-URL
pub type ResolvableItem = BTreeMap<String, String>;


// To use for mapping domain names (with path in a flattened hierarchy) to ResolvableItems
#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ResolvableMap {
    // #[derive(PartialEq)]
    pub entries: BTreeMap<String, ResolvableItem>,
    pub default: String,
}

impl ResolvableMap {
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

// List of public names uploaded with details if they were added, updated or deleted from ResolvableMaps
type ProcessedEntries = BTreeMap<String, (String, String)>;

pub fn xorname_from_nrs_string(name: &str) -> ResultReturn<XorName> {
    let vec_hash = sha3_256(&name.to_string().into_bytes());

    let xorname = XorName(vec_hash);
    debug!("Resulting XornName for NRS: {} is, {}", &name, &xorname);

    Ok(xorname)
}

#[allow(dead_code)]
impl Safe {
    /// # Create a ResolvableMapContainer.
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
    /// let (xorurl, _processed_entries, resolvable_map_container) = safe.resolvable_map_container_create(&rand_string, "safe://somewhere", true, false).unwrap();
    /// assert!(xorurl.contains("safe://"))
    /// ```
    pub fn resolvable_map_container_create(
        &mut self,
        name: &str,
        destination: &str,
        default: bool,
		_dry_run: bool,
    ) -> ResultReturn<(XorUrl, ProcessedEntries, ResolvableMap)> {
        let sanitized_name = str::replace(&name, "safe://", "").to_string();

        let nrs_xorname = xorname_from_nrs_string(&sanitized_name)?;

        debug!("XorName for \"{:?}\" is \"{:?}\"", &name, &nrs_xorname);

        // TODO: Enable source for funds / ownership

        // The ResolvableMapContainer is created as a AppendOnlyData with a single entry containing the
        // timestamp as the entry's key, and the serialised ResolvableMap as the entry's value
        // TODO: use RDF format
        let resolvable_map = resolvable_map_create(&name, &destination, default)?;

        let mut processed_entries = BTreeMap::new();
        processed_entries.insert(
            name.to_string(),
            (CONTENT_ADDED_SIGN.to_string(), destination.to_string()),
        );

        let serialised_resolvable_map = serde_json::to_string(&resolvable_map).map_err(|err| {
            Error::Unexpected(format!(
                "Couldn't serialise the ResolvableMap generated: {:?}",
                err
            ))
        })?;
        let now = gen_timestamp();
        let resolvable_container_data = vec![(
            now.into_bytes().to_vec(),
            serialised_resolvable_map.as_bytes().to_vec(),
        )];

        // Store the ResolvableMapContainer in a Published AppendOnlyData
        let xorname = self.safe_app.put_seq_append_only_data(
            resolvable_container_data,
            Some(nrs_xorname),
            RESOLVABLE_MAP_TYPE_TAG,
            None,
        )?;

        let xorurl = XorUrlEncoder::encode(
            xorname,
            RESOLVABLE_MAP_TYPE_TAG,
            SafeContentType::ResolvableMapContainer,
			None,
            &self.xorurl_base,
        )?;

        Ok((xorurl, processed_entries, resolvable_map))
    }

    /// # Fetch an existing ResolvableMapContainer.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use safe_cli::Safe;
	/// # use rand::distributions::Alphanumeric;
	/// # use rand::{thread_rng, Rng};
    /// # let mut safe = Safe::new("base32z".to_string());
	/// # safe.connect("", Some("fake-credentials")).unwrap();
    /// # const FAKE_RDF_PREDICATE_LINK: &str = "link";
	/// let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
    /// let (xorurl, _processed_entries, _resolvable_map) = safe.resolvable_map_container_create(&rand_string, "somewhere", true, false).unwrap();
    /// let (version, resolvable_map_container, native_type) = safe.resolvable_map_container_get_latest(&xorurl).unwrap();
	/// assert_eq!(version, 1);
    /// assert_eq!(resolvable_map_container.entries[&rand_string][FAKE_RDF_PREDICATE_LINK], "somewhere");
    /// assert_eq!(resolvable_map_container.get_default_link().unwrap(), "somewhere");
    /// assert_eq!(resolvable_map_container.get_default().unwrap(), &rand_string);
    /// ```
    pub fn resolvable_map_container_get_latest(
        &self,
        xorurl: &str,
    ) -> ResultReturn<(u64, ResolvableMap, String)> {
        debug!("Getting latest resolvable map container from: {:?}", xorurl);

        let xorurl_encoder = XorUrlEncoder::from_url(xorurl)?;
        match self
            .safe_app
            .get_latest_seq_append_only_data(xorurl_encoder.xorname(), RESOLVABLE_MAP_TYPE_TAG)
        {
            Ok((version, (_key, value))) => {
                debug!(
                    "Resolvable map retrieved.... v{:?}, value {:?} ",
                    &version, &value
                );
                // TODO: use RDF format and deserialise it
                let resolvable_map = serde_json::from_str(&String::from_utf8_lossy(&value.as_slice()))
                    .map_err(|err| {
                    Error::ContentError(format!(
                        "Couldn't deserialise the ResolvableMap stored in the ResolvableContainer: {:?}",
                        err
                    ))
                })?;
                Ok((
                    version,
                    resolvable_map,
                    RESOLVABLE_MAP_TYPE_TAG_NATIVE_TYPE.to_string(),
                ))
            }
            Err(Error::EmptyContent(_)) => {
                warn!("Resolvable container found at {:?} was empty", &xorurl);
                Ok((
                    0,
                    ResolvableMap::default(),
                    RESOLVABLE_MAP_TYPE_TAG_NATIVE_TYPE.to_string(),
                ))
            }
            Err(Error::ContentNotFound(_)) => Err(Error::ContentNotFound(
                ERROR_MSG_NO_RESOLVABLE_MAP_FOUND.to_string(),
            )),
            Err(err) => Err(Error::NetDataError(format!(
                "Failed to get current version: {}",
                err
            ))),
        }
    }
}

// Helper functions

// TODO: Move to helper func
fn gen_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

// From the provided list of resolovable items
// create a ResolvableMap with metadata and their corresponding links
fn resolvable_map_create(
    name: &str,
    destination: &str,
    set_as_defualt: bool,
) -> ResultReturn<ResolvableMap> {
    let mut resolvable_map = ResolvableMap::default();
    let now = gen_timestamp();

    // TODO: Split name w/ .

    debug!("ResolvableMap for name:{:?}", &name);
    let mut resolvable_item = ResolvableItem::new();

    resolvable_item.insert(FAKE_RDF_PREDICATE_LINK.to_string(), destination.to_string());

    resolvable_item.insert(FAKE_RDF_PREDICATE_MODIFIED.to_string(), now.clone());
    resolvable_item.insert(FAKE_RDF_PREDICATE_CREATED.to_string(), now.clone());
    resolvable_item.insert(FAKE_RDF_PREDICATE_CREATED.to_string(), now.clone());

    debug!("ResolvableItem item: {:?}", resolvable_item);

    debug!("ResolvableItem item inserted with name {:?}", &name);
    resolvable_map
        .entries
        .insert(name.to_string(), resolvable_item);

    if set_as_defualt {
        debug!("Setting {:?} as default for ResolvableMap", &name);

        resolvable_map.default = name.to_string();
    }

    Ok(resolvable_map)
}

// Unit Tests

#[test]
fn test_resolvable_map_container_create() {
    use unwrap::unwrap;

    let mut safe = Safe::new("base32z".to_string());
	safe.connect("", Some("fake-credentials")).unwrap();

    let nrs_xorname = xorname_from_nrs_string("some_site").unwrap();
    let SITE_NAME = "some_site";
    let (xor_url, _entries, resolvable_map) =
        unwrap!(safe.resolvable_map_container_create(SITE_NAME, "safe://top_xorurl", true, false));
    assert_eq!(resolvable_map.entries.len(), 1);
    let resolvable_item = &resolvable_map.entries[SITE_NAME];
    assert_eq!(
        resolvable_item[FAKE_RDF_PREDICATE_LINK],
        "safe://top_xorurl"
    );
    assert_eq!(resolvable_map.get_default().unwrap(), SITE_NAME);

    let decoder = XorUrlEncoder::from_url(&xor_url).unwrap();
    assert_eq!(nrs_xorname, decoder.xorname())
}

#[test]
fn test_resolvable_map_create() {
    use unwrap::unwrap;
    let mut safe = Safe::new("base32z".to_string());
    let resolvable_map = unwrap!(resolvable_map_create("site1", "safe://top_xorurl", true));
    assert_eq!(resolvable_map.entries.len(), 1);
    let resolvable_item = &resolvable_map.entries["site1"];
    assert_eq!(
        resolvable_item[FAKE_RDF_PREDICATE_LINK],
        "safe://top_xorurl"
    );
    assert_eq!(resolvable_map.get_default().unwrap(), "site1");
    assert_eq!(
        resolvable_map.get_default_link().unwrap(),
        "safe://top_xorurl"
    );
}
