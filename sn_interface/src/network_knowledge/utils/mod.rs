// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::network_knowledge::prefix_map::NetworkPrefixMap;
use crate::types::{Error, Result};
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::os::windows::fs::symlink_file;
use std::path::PathBuf;
use tokio::fs::{read_link, remove_file, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub const DEFAULT_PREFIX_SYMLINK_NAME: &str = "default";

pub async fn compare_and_write_prefix_map_to_disk(prefix_map: &NetworkPrefixMap) -> Result<()> {
    // Open or create `$User/.safe/prefix_maps` dir
    let prefix_map_dir = get_prefix_map_dir()?;
    tokio::fs::create_dir_all(prefix_map_dir.clone())
        .await
        .map_err(|_| {
            Error::DirectoryHandling("Could not read '.safe/prefix_maps' directory".to_string())
        })?;
    let prefix_map_file = prefix_map_dir.join(format!("{:?}", prefix_map.genesis_key()));

    // Check if the prefixMap is already present and is latest to the provided Map.
    let disk_map = read_prefix_map_from_disk().await.ok();
    let mut update_symlink: bool = false;
    if let Some(old_map) = disk_map {
        // if symlink points to a different PrefixMap
        if old_map.genesis_key() != prefix_map.genesis_key() {
            update_symlink = true;
        }
        // Return early as the PrefixMap in disk is the equivalent/latest already
        else if &old_map >= prefix_map {
            info!("Equivalent/Latest PrefixMap already in disk");
            return Ok(());
        }
    } else {
        // if symlink is not present
        update_symlink = true;
    }

    trace!("Writing prefix_map to disk at {:?}", prefix_map_file);
    let serialized =
        rmp_serde::to_vec(prefix_map).map_err(|e| Error::Serialisation(e.to_string()))?;

    let mut file = File::create(&prefix_map_file)
        .await
        .map_err(|e| Error::FileHandling(e.to_string()))?;

    file.write_all(&serialized)
        .await
        .map_err(|e| Error::FileHandling(e.to_string()))?;

    file.sync_all()
        .await
        .map_err(|e| Error::FileHandling(e.to_string()))?;

    if update_symlink {
        update_prefix_map_symlink(&prefix_map.genesis_key()).await?;
    }

    Ok(())
}

pub async fn read_prefix_map_from_disk() -> Result<NetworkPrefixMap> {
    // Read NetworkPrefixMap from disk
    let path = get_prefix_map_dir()?.join(DEFAULT_PREFIX_SYMLINK_NAME);
    match File::open(&path).await {
        Ok(mut prefix_map_file) => {
            let mut prefix_map_contents = vec![];
            let _ = prefix_map_file
                .read_to_end(&mut prefix_map_contents)
                .await
                .map_err(|err| {
                    Error::FileHandling(format!(
                        "Error reading PrefixMap from {:?}: {:?}",
                        path, err
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

pub async fn update_prefix_map_symlink(genesis_key: &bls::PublicKey) -> Result<()> {
    // point '.safe/prefix_maps/default' to the PrefixMap corresponding to the given genesis_key
    let prefix_map_dir = get_prefix_map_dir()?;
    let prefix_map_file = prefix_map_dir.join(format!("{:?}", genesis_key));
    let default_prefix = prefix_map_dir.join(DEFAULT_PREFIX_SYMLINK_NAME);

    if read_link(&default_prefix).await.is_ok() {
        trace!("Remove default_prefix symlink as it already exists");
        remove_file(&default_prefix).await.map_err(|e| {
            Error::FileHandling(format!(
                "Error removing previous PrefixMap symlink: {:?}",
                e
            ))
        })?;
    }

    trace!(
        "Creating symlink for PrefixMap from {:?} to {:?}",
        prefix_map_file,
        default_prefix
    );
    #[cfg(unix)]
    symlink(prefix_map_file, default_prefix)
        .map_err(|e| Error::FileHandling(format!("Error creating PrefixMap symlink: {:?}", e)))?;
    #[cfg(windows)]
    symlink_file(prefix_map_file, default_prefix)
        .map_err(|e| Error::FileHandling(format!("Error creating PrefixMap symlink: {:?}", e)))?;
    Ok(())
}

fn get_prefix_map_dir() -> Result<PathBuf> {
    Ok(dirs_next::home_dir()
        .ok_or_else(|| Error::DirectoryHandling("Could not read '.safe' directory".to_string()))?
        .join(".safe")
        .join("prefix_maps"))
}
