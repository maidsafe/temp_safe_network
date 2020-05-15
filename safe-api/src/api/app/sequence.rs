// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{xorurl::SafeContentType, Safe, SafeApp};
use crate::{
    xorurl::{XorUrl, XorUrlEncoder},
    Error, Result,
};
use log::debug;
use safe_nd::XorName;

impl Safe {
    /// Create a Public Sequence on the network
    ///
    /// ## Example
    /// ```
    /// # use safe_api::Safe;
    /// # let mut safe = Safe::default();
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// # async_std::task::block_on(async {
    ///     let data = b"First in the sequence";
    ///     let xorurl = safe.sequence_create(data, None, 20_000).await.unwrap();
    ///     let received_data = safe.sequence_get(&xorurl).await.unwrap();
    ///     assert_eq!(received_data, (0, data.to_vec()));
    /// # });
    /// ```
    pub async fn sequence_create(
        &mut self,
        data: &[u8],
        name: Option<XorName>,
        type_tag: u64,
    ) -> Result<XorUrl> {
        let xorname = self
            .safe_app
            .store_sequence_data(data, name, type_tag, None)
            .await?;

        XorUrlEncoder::encode_sequence_data(
            xorname,
            type_tag,
            SafeContentType::Raw,
            self.xorurl_base,
        )
    }

    /// Get data from a Public Sequence on the network
    ///
    /// ## Example
    /// ```
    /// # use safe_api::Safe;
    /// # let mut safe = Safe::default();
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// # async_std::task::block_on(async {
    ///     let data = b"First in the sequence";
    ///     let xorurl = safe.sequence_create(data, None, 20_000).await.unwrap();
    ///     let received_data = safe.sequence_get(&xorurl).await.unwrap();
    ///     assert_eq!(received_data, (0, data.to_vec()));
    /// # });
    /// ```
    pub async fn sequence_get(&self, url: &str) -> Result<(u64, Vec<u8>)> {
        debug!("Getting Public Sequence data from: {:?}", url);
        let (xorurl_encoder, _) = self.parse_and_resolve_url(url).await?;

        self.fetch_sequence(&xorurl_encoder).await
    }

    /// Fetch a Sequence from a XorUrlEncoder without performing any type of URL resolution
    pub(crate) async fn fetch_sequence(
        &self,
        xorurl_encoder: &XorUrlEncoder,
    ) -> Result<(u64, Vec<u8>)> {
        let data = match xorurl_encoder.content_version() {
            Some(version) => {
                // We fetch a specific entry since the URL specifies a specific version
                let data = self
                    .safe_app
                    .get_sequence_entry(
                        xorurl_encoder.xorname(),
                        xorurl_encoder.type_tag(),
                        version,
                    )
                    .await
                    .map_err(|err| {
                        if let Error::VersionNotFound(_) = err {
                            Error::VersionNotFound(format!(
                                "Version '{}' is invalid for the Sequence found at \"{}\"",
                                version, xorurl_encoder,
                            ))
                        } else {
                            err
                        }
                    })?;
                Ok((version, data))
            }
            None => {
                // ...then get last entry in the Sequence
                self.safe_app
                    .get_sequence_last_entry(xorurl_encoder.xorname(), xorurl_encoder.type_tag())
                    .await
            }
        };

        match data {
            Ok((version, value)) => {
                debug!("Sequence retrieved... v{}", &version);
                Ok((version, value))
            }
            Err(Error::EmptyContent(_)) => Err(Error::EmptyContent(format!(
                "Sequence found at \"{}\" was empty",
                xorurl_encoder
            ))),
            Err(Error::ContentNotFound(_)) => Err(Error::ContentNotFound(
                "No Sequence found at this address".to_string(),
            )),
            other => other,
        }
    }

    /// Append data to a Public Sequence on the network
    ///
    /// ## Example
    /// ```
    /// # use safe_api::Safe;
    /// # let mut safe = Safe::default();
    /// # safe.connect("", Some("fake-credentials")).unwrap();
    /// # async_std::task::block_on(async {
    ///     let data1 = b"First in the sequence";
    ///     let xorurl = safe.sequence_create(data1, None, 20_000).await.unwrap();
    ///     let data2 = b"Second in the sequence";
    ///     safe.sequence_append(&xorurl, data2).await.unwrap();
    ///     let received_data = safe.sequence_get(&xorurl).await.unwrap();
    ///     assert_eq!(received_data, (1, data2.to_vec()));
    /// # });
    /// ```
    pub async fn sequence_append(&mut self, url: &str, data: &[u8]) -> Result<()> {
        let xorurl_encoder = Safe::parse_url(url)?;
        if xorurl_encoder.content_version().is_some() {
            return Err(Error::InvalidInput(format!(
                "The target URL cannot cannot contain a version: {}",
                url
            )));
        };

        let (xorurl_encoder, _) = self.parse_and_resolve_url(url).await?;

        let xorname = xorurl_encoder.xorname();
        let type_tag = xorurl_encoder.type_tag();
        self.safe_app.sequence_append(data, xorname, type_tag).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::app::test_helpers::new_safe_instance;

    #[tokio::test]
    async fn test_sequence_create() -> Result<()> {
        let mut safe = new_safe_instance()?;
        let initial_data = b"initial data";
        let xorurl = safe.sequence_create(initial_data, None, 25_000).await?;
        let received_data = safe.sequence_get(&xorurl).await?;
        assert_eq!(received_data, (0, initial_data.to_vec()));
        Ok(())
    }

    #[tokio::test]
    async fn test_sequence_append() -> Result<()> {
        let mut safe = new_safe_instance()?;
        let data1 = b"First in the sequence";
        let xorurl = safe.sequence_create(data1, None, 25_000).await?;
        let data2 = b"Second in the sequence";
        safe.sequence_append(&xorurl, data2).await?;
        let received_data = safe.sequence_get(&format!("{}?v=1", xorurl)).await?;
        assert_eq!(received_data, (1, data2.to_vec()));
        Ok(())
    }
}
