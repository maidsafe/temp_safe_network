// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// use super::helpers::{parse_coins_amount, pk_from_hex, pk_to_hex, sk_from_hex, KeyPair};
use super::xorurl::{xorname_to_xorurl, xorurl_to_xorname, XorUrl};
// use super::scl_mock::{xorname_to_xorurl, xorurl_to_xorname, XorUrl};
use super::{BlsKeyPair, Safe};
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use threshold_crypto::SecretKey;
use unwrap::unwrap;

// Each FileItem contains file metadata and the link to the file's ImmutableData XOR-URL
pub type FileItem = BTreeMap<String, String>;

// To use for mapping files names (with path in a flattened hierarchy) to FileItems
pub type FilesMap = BTreeMap<String, FileItem>;

pub type FilesContainer = String; //json serialised
pub type FilesMap = Vec<(DateTime<Utc>, FilesContainer)>;

impl Safe {
    /// # Create a map of paths to xorurls
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use safe_cli::Safe;
    /// # use unwrap::unwrap;
    /// # use std::collections::BTreeMap;
    /// # let mut safe = Safe::new("base32".to_string());
    /// let top = b"Something top level";
    /// let top_xorurl = safe.put_published_immutable(top).unwrap();
    /// let second = b"Something second level";
    /// let second_xorurl = safe.put_published_immutable(second).unwrap();
    /// let mut content_map = BTreeMap::new();
    /// content_map.insert("./tests/testfolder/test.md".to_string(), top_xorurl);
    /// content_map.insert("./tests/testfolder/subfolder/subexists.md".to_string(), second_xorurl);
    /// let file_map = safe.create_files_map( content_map ).unwrap();
    /// # assert_eq!(true, file_map.contains("\"md\""));
    /// # assert_eq!(true, file_map.contains("\"./tests/testfolder/test.md\""));
    /// ```
    pub fn create_files_map(&mut self, content: ContentMap) -> Result<String, String> {
        let mut files_map = FilesMap::default();
        let now = Utc::now().to_string().to_string();

        for (key, value) in content.iter() {
            let mut file = FileItem::new();
            let metadata = fs::metadata(&key).map_err(|err| {
                format!(
                    "Couldn't obtain metadata information for local file: {:?}",
                    err,
                )
            })?;

            file.insert("link".to_string(), value.to_string());

            let file_type = Path::new(&key).extension().ok_or("unknown")?;
            file.insert(
                "type".to_string(),
                file_type.to_str().unwrap_or_else(|| "unknown").to_string(),
            );

            let file_size = &metadata.len().to_string();
            file.insert("size".to_string(), file_size.to_string());

            // file.insert("permissions", metadata.permissions().to_string());
            file.insert("modified".to_string(), now.clone());
            file.insert("created".to_string(), now.clone());

            debug!("FileItem item: {:?}", file);

            &files_map.insert(key.to_string(), file);
        }

        // TODO: use RDF format and serialise it
        let serialised_rdf = serde_json::to_string(&files_map)
            .map_err(|err| format!("Couldn't serialise the FilesMap generated: {:?}", err))?;

        Ok(serialised_rdf)
    }

    /// # Create versioned data.
    ///
    /// ## Example
    ///
    /// ```rust
    /// # use safe_cli::Safe;
    /// # use unwrap::unwrap;
    /// # use std::collections::BTreeMap;
    /// # let mut safe = Safe::new("base32".to_string());
    /// let top = b"Something top level";
    /// let top_xorurl = safe.put_published_immutable(top).unwrap();
    /// let second = b"Something second level";
    /// let second_xorurl = safe.put_published_immutable(second).unwrap();
    /// let mut content_map = BTreeMap::new();
    /// content_map.insert("./tests/testfolder/test.md".to_string(), top_xorurl);
    /// content_map.insert("./tests/testfolder/subfolder/subexists.md".to_string(), second_xorurl);
    /// let file_map = safe.create_files_map( content_map ).unwrap();
    /// # assert!(file_map.contains("\"md\""));
    /// # assert!(file_map.contains("\"./tests/testfolder/test.md\""));
    /// let xor_url = safe.put_versioned_data(file_map.into_bytes().to_vec(), 21321 ).unwrap();
    /// assert!(xor_url.contains("safe://"))
    /// ```
    pub fn put_versioned_data(&mut self, data: Vec<u8>, type_tag: u64) -> Result<XorUrl, String> {
        // let mut file_map : FilesMap =
        let now = Utc::now().to_string().to_string();

        let appendable_data = vec![(now.into_bytes().to_vec(), data)];

        //create this data!.
        let xorname = self
            .safe_app
            .put_seq_appendable_data(appendable_data, None, type_tag, None);

        xorname_to_xorurl(&xorname.unwrap(), &self.xorurl_base)
    }

    // TODO:
    // Upload files as ImmutableData
    // Check if file or dir
    // if dir, grab and do many.
    // upload individual file
    // get file metadata?
    // if not now... when?

    /// # Put Published ImmutableData
    /// Put data blobs onto the network.
    ///
    /// ## Example
    /// ```
    /// # use safe_cli::Safe;
    /// # use unwrap::unwrap;
    /// # let mut safe = Safe::new("base32".to_string());
    /// let data = b"Something super good";
    /// let xorurl = safe.put_published_immutable(data).unwrap();
    /// # let received_data = safe.get_published_immutable(xorurl).unwrap();
    /// # assert_eq!(received_data, data);
    /// ```
    pub fn put_published_immutable(&mut self, data: &[u8]) -> Result<XorUrl, String> {
        // TODO: do we want ownership from other PKs yet?
        let xorname = self.safe_app.put_published_immutable(&data);

        xorname_to_xorurl(&xorname.unwrap(), &self.xorurl_base)
    }

    /// # Get Published ImmutableData
    /// Put data blobs onto the network.
    ///
    /// ## Example
    /// ```
    /// # use safe_cli::Safe;
    /// # use unwrap::unwrap;
    /// # let mut safe = Safe::new("base32".to_string());
    /// # let data = b"Something super good";
    /// let xorurl = safe.put_published_immutable(data).unwrap();
    /// let received_data = safe.get_published_immutable(xorurl).unwrap();
    /// # assert_eq!(received_data, data);
    /// ```
    pub fn get_published_immutable(&mut self, xorurl: XorUrl) -> Result<Vec<u8>, String> {
        // TODO: do we want ownership from other PKs yet?
        let xorname = xorurl_to_xorname(&xorurl).unwrap();
        self.safe_app.get_published_immutable(xorname)
    }
}

// Unit Tests

#[test]
fn test_keys_create_preload_test_coins() {}
