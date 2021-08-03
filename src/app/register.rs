// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

pub use safe_network::types::register::{Entry, EntryHash};

use crate::{Error, Result, Safe};
use log::debug;
use safe_network::url::{SafeContentType, SafeUrl, XorUrl};
use std::collections::BTreeSet;
use xor_name::XorName;

impl Safe {
    /// Create a Register on the network
    pub async fn register_create(
        &self,
        name: Option<XorName>,
        type_tag: u64,
        private: bool,
    ) -> Result<XorUrl> {
        let xorname = self
            .safe_client
            .store_register(name, type_tag, None, private)
            .await?;

        let xorurl = SafeUrl::encode_register(
            xorname,
            type_tag,
            SafeContentType::Raw,
            self.xorurl_base,
            private,
        )?;

        Ok(xorurl)
    }

    /// Read value from a Register on the network
    pub async fn register_read(&self, url: &str) -> Result<BTreeSet<(EntryHash, Entry)>> {
        debug!("Getting Public Register data from: {:?}", url);
        let (safeurl, _) = self.parse_and_resolve_url(url).await?;

        self.fetch_register_entries(&safeurl).await
    }

    /// Read value from a Register on the network by its hash
    pub async fn register_read_entry(&self, url: &str, hash: EntryHash) -> Result<Entry> {
        debug!("Getting Public Register data from: {:?}", url);
        let (safeurl, _) = self.parse_and_resolve_url(url).await?;

        self.fetch_register_entry(&safeurl, hash).await
    }

    /// Fetch a Register from a SafeUrl without performing any type of URL resolution
    /// Works with version hashes
    /// e.g. safe://mysafeurl?v=ce56a3504c8f27bfeb13bdf9051c2e91409230ea
    pub(crate) async fn fetch_register_entries(
        &self,
        safeurl: &SafeUrl,
    ) -> Result<BTreeSet<(EntryHash, Entry)>> {
        // take entry with version hash
        if let Some(v) = safeurl.content_version() {
            let hash = v.entry_hash();
            let entry = self
                .fetch_register_entry(&safeurl, hash)
                .await
                .map_err(|e| match e {
                    Error::EmptyContent(_) => {
                        Error::EmptyContent(format!("Register found at \"{}\" was empty", safeurl))
                    }
                    Error::ContentNotFound(_) => {
                        Error::ContentNotFound("No Register found at this address".to_string())
                    }
                    other => other,
                })?;
            let mut set = BTreeSet::new();
            set.insert((hash, entry));
            return Ok(set);
        }

        // else take latest entry
        let address = safeurl.register_address()?;

        match self.safe_client.read_register(address).await {
            Ok(data) => {
                debug!("Register retrieved...");
                Ok(data)
            }
            Err(Error::EmptyContent(_)) => Err(Error::EmptyContent(format!(
                "Register found at \"{}\" was empty",
                safeurl
            ))),
            Err(Error::ContentNotFound(_)) => Err(Error::ContentNotFound(
                "No Register found at this address".to_string(),
            )),
            other => other,
        }
    }

    /// Fetch a Register from a SafeUrl without performing any type of URL resolution
    pub(crate) async fn fetch_register_entry(
        &self,
        safeurl: &SafeUrl,
        hash: EntryHash,
    ) -> Result<Entry> {
        // TODO: allow to specify the hash with the SafeUrl as well: safeurl.content_hash(),
        // e.g. safe://mysafeurl#ce56a3504c8f27bfeb13bdf9051c2e91409230ea
        let address = safeurl.register_address()?;

        // We fetch a specific entry with provided hash
        self.safe_client.get_register_entry(address, hash).await
    }

    /// Write value to a Register on the network
    pub async fn write_to_register(
        &self,
        url: &str,
        data: Vec<u8>,
        parents: BTreeSet<EntryHash>,
    ) -> Result<EntryHash> {
        /*
        let safeurl = Safe::parse_url(url)?;
        if safeurl.content_hash().is_some() {
            // TODO: perhaps we can allow this, and that's how an
            // application can specify the parent entry in the Register.
            return Err(Error::InvalidInput(format!(
                "The target URL cannot contain a content hash: {}",
                url
            )));
        };
        */

        let (safeurl, _) = self.parse_and_resolve_url(url).await?;
        let address = safeurl.register_address()?;

        // write the data to the Register
        self.safe_client
            .write_to_register(address, data, parents)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::{app::test_helpers::new_safe_instance, retry_loop};
    use anyhow::Result;

    #[tokio::test]
    async fn test_register_create() -> Result<()> {
        let safe = new_safe_instance().await?;

        let xorurl = safe.register_create(None, 25_000, false).await?;
        let xorurl_priv = safe.register_create(None, 25_000, true).await?;

        let received_data = retry_loop!(safe.register_read(&xorurl));
        let received_data_priv = retry_loop!(safe.register_read(&xorurl_priv));

        assert!(received_data.is_empty());
        assert!(received_data_priv.is_empty());

        let initial_data = b"initial data";
        let hash = safe
            .write_to_register(&xorurl, initial_data.to_vec(), Default::default())
            .await?;
        let hash_priv = safe
            .write_to_register(&xorurl_priv, initial_data.to_vec(), Default::default())
            .await?;

        let received_entry = retry_loop!(safe.register_read_entry(&xorurl, hash));
        let received_entry_priv = retry_loop!(safe.register_read_entry(&xorurl_priv, hash_priv));

        assert_eq!(received_entry, initial_data.to_vec());
        assert_eq!(received_entry_priv, initial_data.to_vec());

        Ok(())
    }
}
