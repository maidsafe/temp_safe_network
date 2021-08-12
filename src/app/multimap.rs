// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::register::EntryHash;
use crate::{Error, Result, Safe};
use log::debug;
use safe_network::url::{ContentType, NativeUrl, Scope, XorUrl};
use std::collections::BTreeSet;
use xor_name::XorName;

pub type MultimapKey = Vec<u8>;
pub type MultimapValue = Vec<u8>;
pub type MultimapKeyValue = (MultimapKey, MultimapValue);
pub type MultimapKeyValues = BTreeSet<(EntryHash, MultimapKeyValue)>;

const MULTIMAP_REMOVED_MARK: &str = "safe://MULTIMAP_REMOVED_MARK";

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

        let scope = if private {
            Scope::Private
        } else {
            Scope::Public
        };
        let xorurl = NativeUrl::encode_register(
            xorname,
            type_tag,
            scope,
            ContentType::Multimap,
            self.xorurl_base,
        )?;

        Ok(xorurl)
    }

    /// Return the value of a Multimap on the network corresponding to the key provided
    pub async fn multimap_get_by_key(&self, url: &str, key: &[u8]) -> Result<MultimapKeyValues> {
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

        self.fetch_multimap_value_by_hash(&safeurl, hash).await
    }

    // Return the value (by a provided key) of a Multimap on
    // the network without resolving the NativeUrl
    pub(crate) async fn fetch_multimap_value_by_key(
        &self,
        safeurl: &NativeUrl,
        key: &[u8],
    ) -> Result<MultimapKeyValues> {
        let entries = self.fetch_multimap_values(safeurl).await?;
        Ok(entries
            .into_iter()
            .filter(|(_, (entry_key, _))| entry_key == key)
            .collect())
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
        let entry_xorname = self
            .safe_client
            .store_public_blob(&serialised_entry, false)
            .await?;
        let entry_xorurl = NativeUrl::encode_blob(
            entry_xorname,
            Scope::Public,
            ContentType::Raw,
            self.xorurl_base,
        )?;
        let entry_ptr = NativeUrl::from_xorurl(&entry_xorurl)?;
        let safeurl = Safe::parse_url(url)?;
        let address = safeurl.register_address()?;

        self.safe_client
            .write_to_register(address, entry_ptr, replace)
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
        let address = safeurl.register_address()?;
        let hash = self
            .safe_client
            .write_to_register(
                address,
                NativeUrl::from_url(MULTIMAP_REMOVED_MARK)?,
                to_remove,
            )
            .await?;

        Ok(hash)
    }

    // Crate's helper to return the value of a Multimap on
    // the network without resolving the NativeUrl,
    // optionally filtering by hash and/or key.
    pub(crate) async fn fetch_multimap_values(
        &self,
        safeurl: &NativeUrl,
    ) -> Result<MultimapKeyValues> {
        let entries = match self.fetch_register_entries(safeurl).await {
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
        for (hash, entry_ptr) in entries.iter() {
            if entry_ptr.to_string() == MULTIMAP_REMOVED_MARK {
                // this is a tombstone entry created to delete some old entries
                continue;
            }
            let entry = self.fetch_public_blob(entry_ptr, None).await?;
            let key_val = Self::decode_multimap_entry(&entry)?;
            multimap_key_vals.insert((*hash, key_val));
        }
        Ok(multimap_key_vals)
    }

    // Crate's helper to return the value of a Multimap on
    // the network without resolving the NativeUrl,
    // optionally filtering by hash and/or key.
    pub(crate) async fn fetch_multimap_value_by_hash(
        &self,
        safeurl: &NativeUrl,
        hash: EntryHash,
    ) -> Result<Option<MultimapKeyValue>> {
        let entry_ptr = match self.fetch_register_entry(safeurl, hash).await {
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
            Err(other) => Err(other),
        }?;

        // We parse each entry in the Register as a 'MultimapKeyValue'
        if entry_ptr.to_string() == MULTIMAP_REMOVED_MARK {
            Ok(None)
        } else {
            let entry = self.fetch_public_blob(&entry_ptr, None).await?;
            let key_val = Self::decode_multimap_entry(&entry)?;
            Ok(Some(key_val))
        }
    }

    fn decode_multimap_entry(entry: &[u8]) -> Result<MultimapKeyValue> {
        rmp_serde::from_slice(entry)
            .map_err(|err| Error::ContentError(format!("Couldn't parse Multimap entry: {:?}", err)))
    }
}

#[cfg(test)]
mod tests {
    use crate::{app::test_helpers::new_safe_instance, retry_loop, retry_loop_for_pattern};
    use anyhow::Result;
    use std::collections::BTreeSet;

    #[tokio::test]
    async fn test_multimap_create() -> Result<()> {
        let safe = new_safe_instance().await?;

        let xorurl = safe.multimap_create(None, 25_000, false).await?;
        let xorurl_priv = safe.multimap_create(None, 25_000, true).await?;

        let key = b"".to_vec();
        let received_data = retry_loop!(safe.multimap_get_by_key(&xorurl, &key));
        let received_data_priv = retry_loop!(safe.multimap_get_by_key(&xorurl_priv, &key));

        assert_eq!(received_data, Default::default());
        assert_eq!(received_data_priv, Default::default());

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
            .multimap_insert(&xorurl_priv, key_val.clone(), BTreeSet::new())
            .await?;

        let received_data = retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl, &key), Ok(v) if !v.is_empty())?;
        let received_data_priv = retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl_priv, &key), Ok(v) if !v.is_empty())?;

        assert_eq!(
            received_data,
            vec![(hash, key_val.clone())].into_iter().collect()
        );
        assert_eq!(
            received_data_priv,
            vec![(hash_priv, key_val.clone())].into_iter().collect()
        );

        // Let's now test an insert which replace the previous value for a key
        let hashes_to_replace = vec![hash].into_iter().collect();
        let hash2 = safe
            .multimap_insert(&xorurl, key_val2.clone(), hashes_to_replace)
            .await?;
        let hashes_priv_to_replace = vec![hash_priv].into_iter().collect();
        let hash_priv2 = safe
            .multimap_insert(&xorurl_priv, key_val2.clone(), hashes_priv_to_replace)
            .await?;

        let received_data = retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl, &key),
                                                    Ok(v) if v.iter().all(|(_, kv)| *kv != key_val))?;
        let received_data_priv = retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl_priv, &key),
                                                         Ok(v) if v.iter().all(|(_, kv)| *kv != key_val))?;

        assert_eq!(
            received_data,
            vec![(hash2, key_val2.clone())].into_iter().collect()
        );
        assert_eq!(
            received_data_priv,
            vec![(hash_priv2, key_val2.clone())].into_iter().collect()
        );

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

        let received_data = retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl, &key), Ok(v) if !v.is_empty())?;
        let received_data_priv = retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl_priv, &key), Ok(v) if !v.is_empty())?;

        assert_eq!(received_data.len(), 1);
        let (read_hash, read_key_val) = received_data.into_iter().next().unwrap();
        assert_eq!(
            (read_hash, read_key_val),
            (hash, (key.clone(), val.clone()))
        );

        let hashes_to_remove = vec![hash].into_iter().collect();
        let removed_mark_hash = safe.multimap_remove(&xorurl, hashes_to_remove).await?;
        assert_ne!(removed_mark_hash, hash);

        assert_eq!(
            retry_loop_for_pattern!(safe.multimap_get_by_key(&xorurl, &key),
                                    Ok(entries) if entries.is_empty())?,
            Default::default()
        );

        assert_eq!(received_data_priv.len(), 1);
        let (read_hash, read_key_val) = received_data_priv.into_iter().next().unwrap();
        assert_eq!((read_hash, read_key_val), (hash_priv, (key.clone(), val)));

        let hashes_to_remove = vec![hash_priv].into_iter().collect();
        let removed_mark_hash = safe.multimap_remove(&xorurl_priv, hashes_to_remove).await?;
        assert_ne!(removed_mark_hash, hash_priv);

        assert_eq!(
            retry_loop_for_pattern!(
                safe.multimap_get_by_key(&xorurl_priv, &key),
                Ok(entries) if entries.is_empty())?,
            Default::default()
        );

        Ok(())
    }
}
