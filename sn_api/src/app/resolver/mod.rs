// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

mod handlers;
mod safe_data;

use super::{files::FileInfo, Safe};
pub use super::{ContentType, DataType, SafeUrl, VersionHash, XorUrlBase};
use crate::{Error, Result};
use log::{debug, info};
pub use safe_data::SafeData;

pub type Range = Option<(Option<u64>, Option<u64>)>;

// Maximum number of indirections allowed when resolving a safe:// URL following links
const INDIRECTION_LIMIT: usize = 10;

impl Safe {
    /// Parses a string URL "safe://url" and returns a safe URL
    /// Resolves until it reaches the final URL
    pub async fn parse_and_resolve_url(&self, url: &str) -> Result<SafeUrl> {
        let safe_url = Safe::parse_url(url)?;
        let orig_path = safe_url.path_decoded()?;

        // Obtain the resolution chain without resolving the URL's path
        let mut resolution_chain = self
            .fully_resolve_url(
                safe_url, None, false, None, false, // don't resolve the URL's path
            )
            .await?;

        // The resolved content is the last item in the resolution chain we obtained
        let safe_data = resolution_chain
            .pop()
            .ok_or_else(|| Error::ContentNotFound(format!("Failed to resolve {}", url)))?;

        // Set the original path so we return the SafeUrl with it
        let mut new_safe_url = SafeUrl::from_url(&safe_data.xorurl())?;
        new_safe_url.set_path(&orig_path);

        Ok(new_safe_url)
    }

    /// # Retrieve data from a safe:// URL
    ///
    /// ## Examples
    ///
    /// ### Fetch FilesContainer relative path file
    /// ```no_run
    /// # use sn_api::{Safe, resolver::SafeData};
    /// # use std::collections::BTreeMap;
    /// # let mut safe = Safe::default();
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// #   safe.connect(None, None, None).await.unwrap();
    ///     let (xorurl, _, _) = safe.files_container_create_from("./testdata/", None, true, false).await.unwrap();
    ///
    ///     let safe_data = safe.fetch( &format!( "{}/test.md", &xorurl.replace("?v=0", "") ), None ).await.unwrap();
    ///     let data_string = match safe_data {
    ///         SafeData::PublicFile { data, .. } => {
    ///             match String::from_utf8(data.to_vec()) {
    ///                 Ok(string) => string,
    ///                 Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    ///             }
    ///         }
    ///         other => panic!(
    ///             "Content type '{:?}' should not have been found. This should be immutable data.",
    ///             other
    ///         )
    ///     };
    ///
    ///     assert!(data_string.starts_with("hello tests!"));
    /// # });
    /// ```
    pub async fn fetch(&self, url: &str, range: Range) -> Result<SafeData> {
        let safe_url = Safe::parse_url(url)?;
        info!("URL parsed successfully, fetching: {}", url);

        let mut resolution_chain = self
            .fully_resolve_url(safe_url, None, true, range, true)
            .await?;

        resolution_chain
            .pop()
            .ok_or_else(|| Error::ContentNotFound(format!("Failed to resolve {}", url)))
    }

    /// # Inspect a safe:// URL and retrieve metadata information but the actual target content
    /// # As opposed to 'fetch' function, the actual target content won't be fetched, and only
    /// # the URL will be inspected resolving it as necessary to find the target location.
    /// # This is helpful if you are interested in knowing about the target content,
    /// # and/or each of the SafeUrl resolution steps taken to the target content, rather than
    /// # trying to revieve the actual content.
    ///
    /// ## Examples
    ///
    /// ### Inspect FilesContainer relative path file
    /// ```no_run
    /// # use sn_api::{Safe, resolver::SafeData};
    /// # use std::collections::BTreeMap;
    /// # let rt = tokio::runtime::Runtime::new().unwrap();
    /// # rt.block_on(async {
    /// #   let mut safe = Safe::default();
    /// #   safe.connect(None, None, None).await.unwrap();
    ///     let (container_xorurl, _, _) = safe.files_container_create_from("./testdata/", None, true, false).await.unwrap();
    ///
    ///     let inspected_content = safe.inspect( &format!( "{}/test.md", &container_xorurl.replace("?v=0", "") ) ).await.unwrap();
    ///     match &inspected_content[0] {
    ///         SafeData::FilesContainer { xorurl, .. } => {
    ///             assert_eq!(*xorurl, container_xorurl);
    ///         }
    ///         other => panic!(
    ///             "Content type '{:?}' should not have been found. This should be a Files Container.",
    ///             other
    ///         )
    ///     };
    ///     match &inspected_content[1] {
    ///         SafeData::PublicFile { data, media_type, .. } => {
    ///             assert_eq!(*media_type, Some("text/markdown".to_string()));
    ///             assert!(data.is_empty());
    ///         }
    ///         other => panic!(
    ///             "Content type '{:?}' should not have been found. This should be an Immutable Data.",
    ///             other
    ///         )
    ///     };
    ///
    /// # });
    /// ```
    pub async fn inspect(&mut self, url: &str) -> Result<Vec<SafeData>> {
        let safe_url = Safe::parse_url(url)?;
        info!("URL parsed successfully, inspecting: {}", url);
        self.fully_resolve_url(safe_url, None, false, None, true)
            .await
    }

    // Retrieves all pieces of data that resulted from resolving the given URL,
    // keeping a copy of the intermediary resolution steps when indirections occur.
    // Resolves the given URL until
    // - it reaches the final piece of data
    // - or reaches the indirection limit
    // Returns a Vector with the data for all resolution steps
    //
    // NB: When resolving a file, metadata can be attached to it (attached_metadata)
    // Files don't have metadata on SAFE but the FileContainers linking to them have it
    // attached_metadata is used to attach metadata to files linked by their FilesContainers
    // URL -> FileContainer (has metadata..) -> Actual data in a file (..that we attach here)
    // devs can leave a None there when using this function
    //
    // NB: recursive (resolutions that resolve to themselves) aren't managed but since the
    // indirections are limited, it's probably not worth the overhead check.
    // Will need it if we allow infinite indirections though.
    async fn fully_resolve_url(
        &self,
        input_url: SafeUrl,
        attached_metadata: Option<FileInfo>,
        retrieve_data: bool,
        range: Range,
        resolve_path: bool,
    ) -> Result<Vec<SafeData>> {
        debug!(
            "Fetching URL: {} with content of type: {:?}, data type: {:?}",
            input_url,
            input_url.content_type(),
            input_url.data_type()
        );

        let mut indirections_limit = INDIRECTION_LIMIT;
        let mut safe_data_vec = vec![];
        let mut next_step = Some(input_url);
        let mut metadata = attached_metadata;
        while let Some(next_url) = next_step {
            // fetch safe_data from URL
            let safe_data = self
                .resolve_url(next_url, metadata, retrieve_data, range, resolve_path)
                .await?;

            next_step = safe_data.resolves_into();
            metadata = safe_data.metadata();
            safe_data_vec.push(safe_data);

            if indirections_limit == 0 {
                return Err(Error::ContentError(format!("The maximum number of indirections ({}) was reached when trying to resolve the URL provided", INDIRECTION_LIMIT)));
            }

            indirections_limit -= 1;
        }

        Ok(safe_data_vec)
    }

    // Private helper that resolves an URL to some data, but not recursively
    // it stops at the first resolution
    async fn resolve_url(
        &self,
        input_url: SafeUrl,
        attached_metadata: Option<FileInfo>,
        retrieve_data: bool,
        range: Range,
        resolve_path: bool,
    ) -> Result<SafeData> {
        debug!(
            "Resolving URL: {}, of content type: {:?}, and data type: {:?}, address {:?}",
            input_url.to_xorurl_string(),
            input_url.content_type(),
            input_url.data_type(),
            input_url.address()
        );

        match input_url.content_type() {
            ContentType::FilesContainer => {
                self.resolve_file_container(input_url, resolve_path).await
            }
            ContentType::NrsMapContainer => self.resolve_nrs_map_container(input_url).await,
            ContentType::Multimap => self.resolve_multimap(input_url, retrieve_data).await,
            ContentType::Raw => {
                self.resolve_raw(input_url, attached_metadata, retrieve_data, range)
                    .await
            }
            ContentType::MediaType(media_type_str) => {
                self.resolve_mediatype(
                    input_url,
                    attached_metadata,
                    retrieve_data,
                    range,
                    media_type_str,
                )
                .await
            }
            ContentType::Wallet { .. } => Err(Error::EmptyContent(
                "Temporarily disabled feature".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app::files, app::test_helpers::new_safe_instance, retry_loop, SafeUrl, Scope};
    use anyhow::{anyhow, bail, Context, Result};
    use bytes::Bytes;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use safe_network::types::DataAddress;
    use std::io::Read;

    #[tokio::test]
    async fn test_fetch_files_container() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let (fc_xorurl, _, original_files_map) = safe
            .files_container_create_from("./testdata/", None, true, false)
            .await?;

        let safe_url = SafeUrl::from_url(&fc_xorurl)?;
        let content = retry_loop!(safe.fetch(&fc_xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&fc_xorurl))
            .ok_or(anyhow!("files container was unexpectedly empty"))?;

        match content.clone() {
            SafeData::FilesContainer {
                xorurl,
                xorname,
                type_tag,
                version,
                files_map,
                data_type,
                metadata,
                resolves_into,
                resolved_from,
            } => {
                assert_eq!(xorurl, fc_xorurl.clone());
                assert_eq!(xorname, safe_url.xorname());
                assert_eq!(type_tag, files::FILES_CONTAINER_TYPE_TAG);
                assert_eq!(version, Some(version0));
                assert_eq!(files_map, original_files_map);
                assert_eq!(data_type, DataType::Register);
                assert!(metadata.is_none()); // no path so no metadata
                assert!(resolves_into.is_none()); // no path so no next resolution
                assert_eq!(resolved_from, fc_xorurl.clone());
            }
            _ => bail!("Invalid SafeData type! Expected SafeData::FileContainer!"),
        }

        let mut safe_url_with_path = safe_url.clone();
        safe_url_with_path.set_path("/subfolder/subexists.md");
        assert_eq!(safe_url_with_path.path(), "/subfolder/subexists.md");
        assert_eq!(safe_url_with_path.xorname(), safe_url.xorname());
        assert_eq!(safe_url_with_path.type_tag(), safe_url.type_tag());
        assert_eq!(safe_url_with_path.content_type(), safe_url.content_type());

        // let's also compare it with the result from inspecting the URL
        let inspected_content = safe.inspect(&fc_xorurl).await?;
        assert_eq!(inspected_content.len(), 1);
        assert_eq!(content, inspected_content[0]);
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_resolvable_container() -> Result<()> {
        let mut safe = new_safe_instance().await?;

        // create file container
        let (xorurl, _, the_files_map) = safe
            .files_container_create_from("./testdata/", None, true, false)
            .await?;
        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl))
            .ok_or(anyhow!("files container was unexpectedly empty"))?;

        // link to an nrs map
        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_content_version(Some(version0));
        let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
        let _ = safe.nrs_add(&site_name, &safe_url).await?;
        let nrs_url = format!("safe://{}", site_name);

        let content = retry_loop!(safe.fetch(&nrs_url, None));

        safe_url.set_sub_names("")?;
        let xorurl_without_subname = safe_url.to_string();

        // this should resolve to a FilesContainer until we enable prevent resolution.
        match &content {
            SafeData::FilesContainer {
                xorurl,
                xorname,
                type_tag,
                version,
                files_map,
                data_type,
                metadata,
                resolves_into,
                resolved_from,
            } => {
                assert_eq!(*xorurl, xorurl_without_subname);
                assert_eq!(*xorname, safe_url.xorname());
                assert_eq!(*type_tag, 1_100);
                assert_eq!(*version, Some(version0));
                assert_eq!(*data_type, DataType::Register);
                assert_eq!(*files_map, the_files_map);
                assert!(metadata.is_none());
                assert!(resolves_into.is_none());
                assert_eq!(resolved_from, &safe_url.to_string());
            }
            _ => {
                bail!("FilesContainer was not returned".to_string())
            }
        }

        // let's also compare it with the result from inspecting the URL
        let inspected_content = safe.inspect(&nrs_url).await?;
        assert_eq!(inspected_content.len(), 2);
        assert_eq!(content, inspected_content[1]);

        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_resolvable_map_data() -> Result<()> {
        let mut safe = new_safe_instance().await?;

        // create file container
        let (xorurl, _, _the_files_map) = safe
            .files_container_create_from("./testdata/", None, true, false)
            .await?;
        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl))
            .ok_or(anyhow!("files container was unexpectedly empty"))?;

        // link to an nrs map
        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_content_version(Some(version0));
        let files_container_url = safe_url;
        let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
        let (nrs_resolution_url, did_create) =
            safe.nrs_add(&site_name, &files_container_url).await?;
        assert!(did_create);
        let nrs_url = format!("safe://{}", site_name);

        // this should resolve to a FilesContainer
        let content = retry_loop!(safe.fetch(&nrs_url, None));
        match &content {
            SafeData::FilesContainer {
                xorurl,
                resolved_from,
                resolves_into,
                ..
            } => {
                assert_eq!(*xorurl, files_container_url.to_string());
                assert_eq!(*resolved_from, files_container_url.to_string());
                assert!(resolves_into.is_none());
            }
            _ => {
                bail!("FilesContainer was not returned".to_string());
            }
        }

        // let's also compare it with the result from inspecting the URL
        let inspected_content = safe.inspect(&nrs_url).await?;
        assert_eq!(inspected_content.len(), 2);
        assert_eq!(&content, &inspected_content[1]);

        // check NRS map container step
        match &inspected_content[0] {
            SafeData::NrsMapContainer {
                xorurl,
                resolved_from,
                resolves_into,
                ..
            } => {
                let mut unversionned_nrs_res_url = nrs_resolution_url.clone();
                unversionned_nrs_res_url.set_content_version(None);
                assert_eq!(*xorurl, unversionned_nrs_res_url.to_xorurl_string());
                assert_eq!(*resolved_from, unversionned_nrs_res_url.to_string());
                assert_eq!(*resolves_into, Some(files_container_url));
            }
            _ => {
                bail!("Nrs map container was not returned".to_string());
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_public_file() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let data = Bytes::from("Something super immutable");
        let xorurl = safe
            .store_public_file(data.clone(), Some("text/plain"))
            .await?;

        let safe_url = SafeUrl::from_url(&xorurl)?;
        let content = retry_loop!(safe.fetch(&xorurl, None));
        assert!(
            content
                == SafeData::PublicFile {
                    xorurl: xorurl.clone(),
                    xorname: safe_url.xorname(),
                    data: data.clone(),
                    resolved_from: xorurl.clone(),
                    media_type: Some("text/plain".to_string()),
                    metadata: None,
                }
        );

        // let's also compare it with the result from inspecting the URL
        let inspected_content = safe.inspect(&xorurl).await?;
        assert!(
            inspected_content[0]
                == SafeData::PublicFile {
                    xorurl: xorurl.clone(),
                    xorname: safe_url.xorname(),
                    data: Bytes::new(),
                    resolved_from: xorurl,
                    media_type: Some("text/plain".to_string()),
                    metadata: None,
                }
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_range_public_file() -> Result<()> {
        let safe = new_safe_instance().await?;
        let saved_data = Bytes::from("Something super immutable");
        let size = saved_data.len();
        let xorurl = safe
            .store_public_file(saved_data.clone(), Some("text/plain"))
            .await?;

        // Fetch first half and match
        let fetch_first_half = Some((None, Some(size as u64 / 2)));
        let content = retry_loop!(safe.fetch(&xorurl, fetch_first_half));

        if let SafeData::PublicFile { data, .. } = content {
            assert_eq!(data, saved_data.slice(0..size / 2));
        } else {
            bail!("Content fetched is not a PublicFile: {:?}", content);
        }

        // Fetch second half and match
        let fetch_second_half = Some((Some(size as u64 / 2), None));
        let content = safe.fetch(&xorurl, fetch_second_half).await?;

        if let SafeData::PublicFile { data, .. } = content {
            assert_eq!(data, saved_data[size / 2..]);
            Ok(())
        } else {
            Err(anyhow!(
                "Content fetched is not a PublicFile: {:?}",
                content
            ))
        }
    }

    #[tokio::test]
    async fn test_fetch_range_from_files_container() -> Result<()> {
        use std::fs::File;
        let mut safe = new_safe_instance().await?;

        // create file container
        let (xorurl, _, _files_map) = safe
            .files_container_create_from("./testdata/", None, true, false)
            .await?;
        let _ = retry_loop!(safe.fetch(&xorurl, None));
        let (version0, _) = retry_loop!(safe.files_container_get(&xorurl))
            .ok_or(anyhow!("files container was unexpectedly empty"))?;

        // map to nrs name
        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_content_version(Some(version0));
        let site_name: String = thread_rng().sample_iter(&Alphanumeric).take(15).collect();
        let _ = safe.nrs_add(&site_name, &safe_url).await?;
        let nrs_url = format!("safe://{}/test.md", site_name);

        // read a local file content (for comparison)
        let mut file = File::open("./testdata/test.md")
            .context("Failed to open local file: ./testdata/test.md".to_string())?;
        let mut file_data = Vec::new();
        file.read_to_end(&mut file_data)
            .context("Failed to read local file: ./testdata/test.md".to_string())?;
        let file_size = file_data.len();

        // fetch full file and compare
        let content = retry_loop!(safe.fetch(&nrs_url, None));
        if let SafeData::PublicFile { data, .. } = &content {
            assert_eq!(data.clone(), file_data.clone());
        } else {
            bail!("Content fetched is not a PublicFile: {:?}", content);
        }

        // fetch first half and match
        let fetch_first_half = Some((Some(0), Some(file_size as u64 / 2)));
        let content = safe.fetch(&nrs_url, fetch_first_half).await?;
        if let SafeData::PublicFile { data, .. } = content {
            assert_eq!(data, file_data[0..file_size / 2]);
        } else {
            bail!("Content fetched is not a PublicFile: {:?}", content);
        }

        // fetch second half and match
        let fetch_second_half = Some((Some(file_size as u64 / 2), Some(file_size as u64)));
        let content = safe.fetch(&nrs_url, fetch_second_half).await?;
        if let SafeData::PublicFile { data, .. } = content {
            assert_eq!(data, file_data[file_size / 2..]);
        } else {
            bail!("Content fetched is not a PublicFile: {:?}", content);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_unsupported_with_media_type() -> Result<()> {
        let mut safe = new_safe_instance().await?;
        let xorname = rand::random();
        let type_tag = 575_756_443;
        let xorurl = SafeUrl::encode(
            DataAddress::register(xorname, Scope::Public, type_tag),
            None,
            type_tag,
            ContentType::MediaType("text/html".to_string()),
            None,
            None,
            None,
            None,
            None,
            XorUrlBase::Base32z,
        )?;

        match safe.fetch(&xorurl, None).await {
            Ok(c) => {
                bail!("Unxpected fetched content: {:?}", c)
            }
            Err(Error::ContentError(msg)) => {
                assert_eq!(msg, "Data type 'Register' not supported yet".to_string())
            }
            other => bail!("Error returned is not the expected one: {:?}", other),
        };

        match safe.inspect(&xorurl).await {
            Ok(c) => Err(anyhow!("Unxpected fetched content: {:?}", c)),
            Err(Error::ContentError(msg)) => {
                assert_eq!(msg, "Data type 'Register' not supported yet".to_string());
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }

    #[tokio::test]
    async fn test_fetch_public_file_with_path() -> Result<()> {
        let safe = new_safe_instance().await?;
        let data = Bytes::from("Something super immutable");
        let xorurl = safe.store_public_file(data.clone(), None).await?;

        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        let path = "/some_relative_filepath";
        safe_url.set_path(path);
        match safe.fetch(&safe_url.to_string(), None).await {
            Ok(c) => {
                bail!("Unxpected fetched content: {:?}", c)
            }
            Err(Error::ContentError(msg)) => assert_eq!(
                msg,
                format!("Cannot get relative path of Immutable Data \"{}\"", path)
            ),
            other => bail!("Error returned is not the expected one: {:?}", other),
        };

        // test the same but a file with some media type
        let xorurl = safe
            .store_public_file(data.clone(), Some("text/plain"))
            .await?;

        let mut safe_url = SafeUrl::from_url(&xorurl)?;
        safe_url.set_path("/some_relative_filepath");
        let url_with_path = safe_url.to_string();
        match safe.fetch(&url_with_path, None).await {
            Ok(c) => Err(anyhow!("Unxpected fetched content: {:?}", c)),
            Err(Error::ContentError(msg)) => {
                assert_eq!(
                    msg,
                    format!("Cannot get relative path of Immutable Data \"{}\"", path)
                );
                Ok(())
            }
            other => Err(anyhow!(
                "Error returned is not the expected one: {:?}",
                other
            )),
        }
    }
}
