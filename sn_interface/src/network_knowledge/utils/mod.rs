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
use std::{io::Write, path::PathBuf};
use tempfile::NamedTempFile;
use tokio::{fs, io::AsyncReadExt};
pub const DEFAULT_PREFIX_HARDLINK_NAME: &str = "default";
pub const SN_PREFIX_MAP_DIR: &str = "SN_PREFIX_MAP_DIR";

pub async fn write_prefix_map_to_disk(prefix_map: &NetworkPrefixMap) -> Result<()> {
    // Open or create `$User/.safe/prefix_maps` dir
    let prefix_map_dir = get_prefix_map_dir()?;
    fs::create_dir_all(prefix_map_dir.clone())
        .await
        .map_err(|_| {
            Error::DirectoryHandling("Could not read '.safe/prefix_maps' directory".to_string())
        })?;
    let prefix_map_file = prefix_map_dir.join(format!("{:?}", prefix_map.genesis_key()));
    let mut temp_file = NamedTempFile::new_in(prefix_map_dir)
        .map_err(|e| Error::FileHandling(format!("Error creating tempfile: {:?}", e)))?;

    let serialized =
        rmp_serde::to_vec(prefix_map).map_err(|e| Error::Serialisation(e.to_string()))?;
    temp_file
        .write_all(serialized.as_slice())
        .map_err(|e| Error::FileHandling(e.to_string()))?;
    fs::rename(temp_file.path(), &prefix_map_file)
        .await
        .map_err(|e| Error::FileHandling(e.to_string()))?;

    set_default_prefix_map(prefix_map.genesis_key()).await?;
    trace!("Wrote prefix_map to disk {:?}", prefix_map_file);
    Ok(())
}

pub async fn read_prefix_map_from_disk() -> Result<NetworkPrefixMap> {
    // Read the default NetworkPrefixMap from disk
    let path = get_prefix_map_dir()?.join(DEFAULT_PREFIX_HARDLINK_NAME);
    match fs::File::open(&path).await {
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

pub async fn set_default_prefix_map(genesis_key: &bls::PublicKey) -> Result<()> {
    // create hardlink '.safe/prefix_maps/default' that points to the PrefixMap corresponding to
    // the given genesis_key
    let prefix_map_dir = get_prefix_map_dir()?;
    let prefix_map_file = prefix_map_dir.join(format!("{:?}", genesis_key));
    let default_prefix = prefix_map_dir.join(DEFAULT_PREFIX_HARDLINK_NAME);

    if default_prefix.exists() {
        trace!("Remove default_prefix hardlink as it already exists");
        fs::remove_file(&default_prefix).await.map_err(|e| {
            Error::FileHandling(format!(
                "Error removing previous PrefixMap hardlink: {:?}",
                e
            ))
        })?;
    }

    trace!(
        "Creating hardlink for PrefixMap from {:?} to {:?}",
        prefix_map_file,
        default_prefix
    );
    fs::hard_link(prefix_map_file, default_prefix)
        .await
        .map_err(|e| {
            Error::FileHandling(format!(
                "Error creating default PrefixMap hardlink: {:?}",
                e
            ))
        })?;
    Ok(())
}

fn get_prefix_map_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var(SN_PREFIX_MAP_DIR) {
        Ok(PathBuf::from(dir))
    } else {
        Ok(dirs_next::home_dir()
            .ok_or_else(|| {
                Error::DirectoryHandling("Could not read '.safe' directory".to_string())
            })?
            .join(".safe")
            .join("prefix_maps"))
    }
}
