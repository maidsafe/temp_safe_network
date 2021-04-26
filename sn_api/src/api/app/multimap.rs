// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    register::EntryHash,
    safeurl::{SafeContentType, SafeDataType, SafeUrl, XorUrl},
};
use crate::{Error, Result, Safe};
use log::debug;
use std::collections::BTreeSet;
use xor_name::XorName;

pub type MultimapKey = Vec<u8>;
pub type MultimapValue = Vec<u8>;
pub type MultimapKeyValue = (MultimapKey, MultimapValue);
pub type MultimapKeyValues = BTreeSet<(EntryHash, MultimapKeyValue)>;

const MULTIMAP_REMOVED_MARK: &[u8] = b"";

impl Safe {
    /// Create a Multimap on the network
    pub async fn multimap_create(
        &self,
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
    pub async fn multimap_get_by_key(
        &self,
        url: &str,
        key: &[u8],
    ) -> Result<Option<(EntryHash, MultimapValue)>> {
        debug!("Getting value by key from Multimap at: {}", url);
        let (safeurl, _) = self.parse_and_resolve_url(url).await?;

        self.fetch_multimap_value_by_key(&safeurl, key).await
    }

    /// Return the value of a Multimap on the network corresponding to the hash provided
    pub async fn multimap_get_by_hash(
        &self,
        url: &str,
        hash: EntryHash,
    ) -> Result<Option<MultimapKeyValue>> {
        debug!("Getting value by hash from Multimap at: {}", url);
        let (safeurl, _) = self.parse_and_resolve_url(url).await?;

        let entries = self
            .fetch_multimap_value(&safeurl, Some(hash), None)
            .await?;

        // Since we passed down a hash we know only one single entry should have been found
        Ok(entries.into_iter().next().map(|(_, key_val)| key_val))
    }

    // Return the value (by a provided key) of a Multimap on
    // the network without resolving the SafeUrl
    pub(crate) async fn fetch_multimap_value_by_key(
        &self,
        safeurl: &SafeUrl,
        key: &[u8],
    ) -> Result<Option<(EntryHash, MultimapValue)>> {
        let entries = self.fetch_multimap_value(safeurl, None, Some(key)).await?;

        // Since we passed down a key to filter with,
        // we know only one single entry should have been found
        Ok(entries
            .into_iter()
            .next()
            .map(|(hash, (_, val))| (hash, val)))
    }

    /// Insert a key-value pair into a Multimap on the network
    pub async fn multimap_insert(
        &self,
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
        &self,
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

    // Crate's helper to return the value of a Multimap on
    // the network without resolving the SafeUrl,
    // optionally filtering by hash and/or key.
    pub(crate) async fn fetch_multimap_value(
        &self,
        safeurl: &SafeUrl,
        hash: Option<EntryHash>,
        key_to_find: Option<&[u8]>,
    ) -> Result<MultimapKeyValues> {
        let entries = match self.fetch_register_value(safeurl, hash).await {
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
        let mut multimap_key_vals = MultimapKeyValues::new();
        for (hash, entry) in entries.iter() {
            if !entry.is_empty() {
                // ...this entry is not a MULTIMAP_REMOVED_MARK,
                // let's then try to parse it as a key-value
                let (current_key, current_value): MultimapKeyValue = rmp_serde::from_slice(entry)
                    .map_err(|err| {
                    Error::ContentError(format!(
                        "Couldn't parse the entry stored in the Multimap at {}: {:?}",
                        safeurl, err
                    ))
                })?;

                if let Some(key) = key_to_find {
                    if *key == current_key {
                        return Ok(vec![(*hash, (current_key, current_value))]
                            .into_iter()
                            .collect());
                    }
                }

                multimap_key_vals.insert((*hash, (current_key, current_value)));
            }
        }

        if key_to_find.is_some() && !multimap_key_vals.is_empty() {
            Ok(MultimapKeyValues::new())
        } else {
            Ok(multimap_key_vals)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{api::app::test_helpers::new_safe_instance, retry_loop, retry_loop_for_pattern};
    use anyhow::{anyhow, bail, Result};
    use std::collections::BTreeSet;

    #[tokio::test]
    async fn test_multimap_create() -> Result<()> {
        let safe = new_safe_instance().await?;

        let xorurl = safe.multimap_create(None, 25_000, false).await?;
        let xorurl_priv = safe.multimap_create(None, 25_000, true).await?;

        let key = b"".to_vec();
        let received_data = retry_loop!(safe.multimap_get_by_key(&xorurl, &key));
        let received_data_priv = retry_loop!(safe.multimap_get_by_key(&xorurl_priv, &key));

        assert_eq!(received_data, None);
        assert_eq!(received_data_priv, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_multimap_insert() -> Result<()> {
        let safe = new_safe_instance().await?;
        let key = b"key".to_vec();
        let val = b"value".to_vec();
        let key_val = (key.clone(), val.clone());

        let val2 = b"value2".to_vec();
        let key_val2 = (key.clone(), val2.clone());

        let xorurl = safe.multimap_create(None, 25_000, false).await?;
        let xorurl_priv = safe.multimap_create(None, 25_000, true).await?;

        let _ = retry_loop!(safe.multimap_get_by_key(&xorurl, &key));
        let _ = retry_loop!(safe.multimap_get_by_key(&xorurl_priv, &key));

        let hash = safe
            .multimap_insert(&xorurl, key_val.clone(), BTreeSet::new())
            .await?;
        let hash_priv = safe
            .multimap_insert(&xorurl_priv, key_val, BTreeSet::new())
            .await?;

        let received_data =
            retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl, &key), Ok(v) if v.is_some())?;
        let received_data_priv = retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl_priv, &key), Ok(v) if v.is_some())?;

        assert_eq!(received_data, Some((hash, val.clone())));
        assert_eq!(received_data_priv, Some((hash_priv, val.clone())));

        // Let's now test an insert which replace the previous value for a key
        let hashes_to_replace = vec![hash].into_iter().collect();
        let hash2 = safe
            .multimap_insert(&xorurl, key_val2.clone(), hashes_to_replace)
            .await?;
        let hashes_priv_to_replace = vec![hash_priv].into_iter().collect();
        let hash_priv2 = safe
            .multimap_insert(&xorurl_priv, key_val2, hashes_priv_to_replace)
            .await?;

        let received_data = retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl, &key), Ok(Some((_, v))) if *v != val)?;
        let received_data_priv = retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl_priv, &key), Ok(Some((_, v))) if *v != val)?;

        assert_eq!(received_data, Some((hash2, val2.clone())));
        assert_eq!(received_data_priv, Some((hash_priv2, val2)));

        Ok(())
    }

    #[tokio::test]
    async fn test_multimap_get_by_hash() -> Result<()> {
        let safe = new_safe_instance().await?;
        let key = b"key".to_vec();
        let val = b"value".to_vec();
        let key_val = (key.clone(), val.clone());
        let key2 = b"key2".to_vec();
        let val2 = b"value2".to_vec();
        let key_val2 = (key2.clone(), val2.clone());

        let xorurl = safe.multimap_create(None, 25_000, false).await?;
        let xorurl_priv = safe.multimap_create(None, 25_000, true).await?;

        let _ = retry_loop!(safe.multimap_get_by_key(&xorurl, &key));
        let _ = retry_loop!(safe.multimap_get_by_key(&xorurl_priv, &key));

        let hash = safe
            .multimap_insert(&xorurl, key_val.clone(), BTreeSet::new())
            .await?;
        let hash2 = safe
            .multimap_insert(&xorurl, key_val2.clone(), BTreeSet::new())
            .await?;

        let hash_priv = safe
            .multimap_insert(&xorurl_priv, key_val.clone(), BTreeSet::new())
            .await?;
        let hash_priv2 = safe
            .multimap_insert(&xorurl_priv, key_val2.clone(), BTreeSet::new())
            .await?;

        let received_data = retry_loop_for_pattern!(safe.multimap_get_by_hash(&xorurl, hash), Ok(v) if v.is_some())?;
        let received_data_priv = retry_loop_for_pattern!(safe.multimap_get_by_hash(&xorurl_priv, hash_priv), Ok(v) if v.is_some())?;

        assert_eq!(received_data, Some(key_val.clone()));
        assert_eq!(received_data_priv, Some(key_val));

        let received_data = retry_loop_for_pattern!(safe.multimap_get_by_hash(&xorurl, hash2), Ok(v) if v.is_some())?;
        let received_data_priv = retry_loop_for_pattern!(safe.multimap_get_by_hash(&xorurl_priv, hash_priv2), Ok(v) if v.is_some())?;

        assert_eq!(received_data, Some(key_val2.clone()));
        assert_eq!(received_data_priv, Some(key_val2));

        Ok(())
    }

    #[tokio::test]
    async fn test_multimap_remove() -> Result<()> {
        let safe = new_safe_instance().await?;
        let key = b"key".to_vec();
        let val = b"value".to_vec();
        let key_val = (key.clone(), val.clone());

        let xorurl = safe.multimap_create(None, 25_000, false).await?;
        let xorurl_priv = safe.multimap_create(None, 25_000, true).await?;

        let _ = retry_loop!(safe.multimap_get_by_key(&xorurl, &key));
        let _ = retry_loop!(safe.multimap_get_by_key(&xorurl_priv, &key));

        let hash = safe
            .multimap_insert(&xorurl, key_val.clone(), BTreeSet::new())
            .await?;
        let hash_priv = safe
            .multimap_insert(&xorurl_priv, key_val, BTreeSet::new())
            .await?;

        let received_data =
            retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl, &key), Ok(v) if v.is_some())?;
        let received_data_priv = retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl_priv, &key), Ok(v) if v.is_some())?;

        if let Some((read_hash, read_val)) = received_data {
            assert_eq!((read_hash, read_val), (hash, val.clone()));

            let hashes_to_remove = vec![hash].into_iter().collect();
            let removed_mark_hash = safe.multimap_remove(&xorurl, hashes_to_remove).await?;
            assert_ne!(removed_mark_hash, hash);

            assert_eq!(
                retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl, &key), Ok(None))?,
                None
            );
        } else {
            bail!("Unexpectedly failed to get key-value from public Multimap");
        }

        if let Some((read_hash, read_val)) = received_data_priv {
            assert_eq!((read_hash, read_val), (hash_priv, val));

            let hashes_to_remove = vec![hash_priv].into_iter().collect();
            let removed_mark_hash = safe.multimap_remove(&xorurl_priv, hashes_to_remove).await?;
            assert_ne!(removed_mark_hash, hash_priv);

            assert_eq!(
                retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl_priv, &key), Ok(None))?,
                None
            );

            Ok(())
        } else {
            Err(anyhow!(
                "Unexpectedly failed to get key-value from private Multimap"
            ))
        }
    }
}
