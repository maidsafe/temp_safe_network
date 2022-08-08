// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    network_knowledge::prefix_map::NetworkPrefixMap,
    types::{Error, Result},
};
use std::{io::Write, path::Path};
use tempfile::NamedTempFile;
use tokio::{fs, io::AsyncReadExt};

pub async fn write_prefix_map_to_disk(prefix_map: &NetworkPrefixMap, path: &Path) -> Result<()> {
    trace!("Writing prefix_map to disk at {}", path.display());
    let parent_path = if let Some(parent_path) = path.parent() {
        fs::create_dir_all(parent_path).await.map_err(|err| {
            Error::DirectoryHandling(format!(
                "Could not create '{}' parent directory path: {}",
                path.display(),
                err,
            ))
        })?;
        parent_path
    } else {
        Path::new(".")
    };

    let mut temp_file = NamedTempFile::new_in(parent_path).map_err(|e| {
        Error::FileHandling(format!(
            "Error creating tempfile at {}: {:?}",
            parent_path.display(),
            e
        ))
    })?;

    let serialized =
        rmp_serde::to_vec(prefix_map).map_err(|e| Error::Serialisation(e.to_string()))?;

    temp_file
        .write_all(serialized.as_slice())
        .map_err(|e| Error::FileHandling(e.to_string()))?;

    fs::rename(temp_file.path(), &path)
        .await
        .map_err(|e| Error::FileHandling(e.to_string()))?;

    trace!("Wrote prefix_map to disk: {}", path.display());

    Ok(())
}

/// Read the default NetworkPrefixMap from disk
pub async fn read_prefix_map_from_disk(path: &Path) -> Result<NetworkPrefixMap> {
    match fs::File::open(path).await {
        Ok(mut prefix_map_file) => {
            let mut prefix_map_contents = vec![];
            let _ = prefix_map_file
                .read_to_end(&mut prefix_map_contents)
                .await
                .map_err(|err| {
                    Error::FileHandling(format!(
                        "Error reading PrefixMap from {}: {:?}",
                        path.display(),
                        err
                    ))
                })?;

            let prefix_map: NetworkPrefixMap = rmp_serde::from_slice(&prefix_map_contents)
                .map_err(|err| {
                    Error::FileHandling(format!(
                        "Error deserializing PrefixMap from disk: {:?}",
                        err
                    ))
                })?;
            Ok(prefix_map)
        }
        Err(e) => Err(Error::FailedToParse(e.to_string())),
    }
}
