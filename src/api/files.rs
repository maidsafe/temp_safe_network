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
use std::collections::BTreeMap;
use threshold_crypto::SecretKey;
use unwrap::unwrap;

// To use for mapping path to xorurl
pub type ContentMap = BTreeMap<String, XorUrl>;

pub type FilesMapData = String; //json serialised
pub type FilesMap = Vec<(DateTime<Utc>, FilesMapData)>;

impl Safe {
    /// # Create a map of paths to xorurls
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// # use safe_cli::Safe;
    /// # use unwrap::unwrap;
    /// # let mut safe = Safe::new("base32".to_string());
    /// let top = b"Something top level";
    /// let top_xorurl = safe.put_published_immutable(top).unwrap();
    /// let second = b"Something second level";
    /// let second_xorurl = safe.put_published_immutable(second).unwrap();
    /// let mut content_map: ContentMap = Default::default();
    /// content_map.insert("./folder/file.txt".to_string(), top_xorurl);
    /// content_map.insert("./folder/subfolder/anotherfile.txt".to_string(), second_xorurl);
    /// let fileMap = create_files_map( content_map ).unwrap();
    /// assert_eq!("what", fileMap);
    /// ```
    pub fn create_files_map(&mut self, content: ContentMap) -> Result<(), String> {
        // TODO: take content map
        // iterate over. Put into timestamp for order...
        // PUT that onto the network.
        //
        Ok(())
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
