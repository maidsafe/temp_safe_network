// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

pub use sn_data_types::register::{Entry, EntryHash};

use super::safeurl::{SafeContentType, SafeDataType, SafeUrl, XorUrl};
use crate::{Error, Result, Safe};
use log::debug;
use std::collections::BTreeSet;
use xor_name::XorName;

pub type MultimapKey = Vec<u8>;
pub type MultimapValue = Vec<u8>;
pub type MultimapKeyValue = (MultimapKey, MultimapValue);

const MULTIMAP_REMOVED_MARK: &[u8] = b"";

impl Safe {
    /// Create a Multimap on the network
    pub async fn multimap_create(
        &mut self,
        name: Option<XorName>,
        type_tag: u64,
        private: bool,
    ) -> Result<XorUrl> {
        debug!("Creating a Multimap");
        let xorname = self
            .safe_client
            .store_register(name, type_tag, None, private)
            .await?;

        let xorurl = SafeUrl::encode_register(
            xorname,
            type_tag,
            SafeContentType::Multimap,
            self.xorurl_base,
            private,
        )?;

        Ok(xorurl)
    }

    /// Return the value of a Multimap on the network corresponding to the key provided
    pub async fn multimap_get(
        &mut self,
        url: &str,
        key: MultimapKey,
    ) -> Result<Option<(EntryHash, MultimapValue)>> {
        debug!("Getting Multimap from: {}", url);
        let safeurl = Safe::parse_url(url)?;

        self.fetch_multimap_value(&safeurl, key).await
    }

    /// Return the value of a Multimap on the network without resolving the SafeUrl
    pub(crate) async fn fetch_multimap_value(
        &mut self,
        safeurl: &SafeUrl,
        key: MultimapKey,
    ) -> Result<Option<(EntryHash, MultimapValue)>> {
        let entries = match self.fetch_register_value(safeurl, None).await {
            Ok(data) => {
                debug!("Multimap retrieved...");
                Ok(data)
            }
            Err(Error::EmptyContent(_)) => Err(Error::EmptyContent(format!(
                "Multimap found at \"{}\" was empty",
                safeurl
            ))),
            Err(Error::ContentNotFound(_)) => Err(Error::ContentNotFound(
                "No Multimap found at this address".to_string(),
            )),
            other => other,
        }?;

        // We parse each entry in the Register as a 'MultimapKeyValue'
        for (hash, entry) in entries.iter() {
            let (current_key, current_value): MultimapKeyValue = rmp_serde::from_slice(entry)
                .map_err(|err| {
                    Error::ContentError(format!(
                        "Couldn't parse the entry stored in the Multimap at {}: {:?}",
                        safeurl, err
                    ))
                })?;

            if key == current_key {
                return Ok(Some((*hash, current_value)));
            }
        }

        Ok(None)
    }

    /// Insert a key-value pair into a Multimap on the network
    pub async fn multimap_insert(
        &mut self,
        url: &str,
        entry: MultimapKeyValue,
        replace: BTreeSet<EntryHash>,
    ) -> Result<EntryHash> {
        debug!("Inserting '{:?}' into Multimap at {}", entry, url);
        let serialised_entry = rmp_serde::to_vec_named(&entry).map_err(|err| {
            Error::Serialisation(format!(
                "Couldn't serialise the Multimap entry '{:?}': {:?}",
                entry, err
            ))
        })?;

        let safeurl = Safe::parse_url(url)?;
        let is_private = safeurl.data_type() == SafeDataType::PrivateRegister;

        self.safe_client
            .write_to_register(
                &serialised_entry,
                safeurl.xorname(),
                safeurl.type_tag(),
                is_private,
                replace,
            )
            .await
    }

    /// Remove a key from a Multimap on the network
    pub async fn multimap_remove(
        &mut self,
        url: &str,
        to_remove: BTreeSet<EntryHash>,
    ) -> Result<EntryHash> {
        debug!("Removing from Multimap at {}: {:?}", url, to_remove);
        let safeurl = Safe::parse_url(url)?;
        let is_private = safeurl.data_type() == SafeDataType::PrivateRegister;

        let hash = self
            .safe_client
            .write_to_register(
                MULTIMAP_REMOVED_MARK,
                safeurl.xorname(),
                safeurl.type_tag(),
                is_private,
                to_remove,
            )
            .await?;

        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use crate::{api::app::test_helpers::new_safe_instance, retry_loop, retry_loop_for_pattern};
    use anyhow::{anyhow, Result};

    #[tokio::test]
    async fn test_multimap_create() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let key = b"key".to_vec();
        let val = b"value".to_vec();
        let initial_key_val = (key.clone(), val.clone());

        let (xorurl, hash) = safe
            .multimap_create(vec![initial_key_val], None, 25_000, false)
            .await?;
        //let (xorurl_priv, _) = safe
        //    .multimap_create(vec![initial_key_val], None, 25_000, true)
        //    .await?;

        let received_data = retry_loop!(safe.multimap_get(&xorurl, &key));
        //let received_data_priv = retry_loop!(safe.multimap_get(&xorurl_priv, &key));
        assert_eq!(received_data, Some((hash, val)));
        //assert_eq!(received_data_priv, Some(val));

        Ok(())
    }

    #[tokio::test]
    async fn test_multimap_insert() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let key1 = b"key1".to_vec();
        let key2 = b"key2".to_vec();
        let val1_v1 = b"value1".to_vec();
        let val1_v2 = b"value2".to_vec();
        let val2 = b"value3".to_vec();
        let initial_key_val = (key1.clone(), val1_v1.clone());

        let (xorurl, initial_hash) = safe
            .multimap_create(vec![initial_key_val.clone()], None, 25_000, false)
            .await?;
        //let (xorurl_priv, _) = safe
        //    .multimap_create(vec![initial_key_val], None, 25_000, true)
        //    .await?;

        let _ = retry_loop!(safe.multimap_get(&xorurl, &key1));
        //let _ = retry_loop!(safe.multimap_get(&xorurl_priv, &key1));

        let hash = safe
            .multimap_insert(&xorurl, (key1.clone(), val1_v2.clone()))
            .await?;

        let received_data = retry_loop_for_pattern!(safe.multimap_get(&xorurl, &key1), Ok(Some((h, _))) if *h == hash)?;
        assert_eq!(received_data, Some((hash, val1_v2)));
        //assert_eq!(received_data_priv, Some((hash, val1_v2)));

        let hash = safe
            .multimap_insert(&xorurl, (key2.clone(), val2.clone()))
            .await?;

        let received_data =
            retry_loop_for_pattern!(safe.multimap_get(&xorurl, &key2), Ok(Some(_)))?;
        assert_eq!(received_data, Some((hash, val2)));
        //assert_eq!(received_data_priv, Some((hash, val2)));

        Ok(())
    }

    #[tokio::test]
    async fn test_multimap_remove() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let key = b"key".to_vec();
        let val = b"value".to_vec();
        let initial_key_val = (key.clone(), val.clone());

        let (xorurl, initial_hash) = safe
            .multimap_create(vec![initial_key_val.clone()], None, 25_000, false)
            .await?;
        //let (xorurl_priv, _) = safe
        //    .multimap_create(vec![initial_key_val], None, 25_000, true)
        //    .await?;

        if let Some((hash, _)) = retry_loop!(safe.multimap_get(&xorurl, &key)) {
            let received_data = safe.multimap_remove(&xorurl, &key).await?;
            assert_eq!(received_data, Some((hash, val)));

            assert_eq!(
                retry_loop_for_pattern!(safe.multimap_get(&xorurl, &key), Ok(None))?,
                None
            );

            Ok(())
        } else {
            Err(anyhow!(
                "Unexpectedly failed to get key-value from Multimap"
            ))
        }
    }
}
