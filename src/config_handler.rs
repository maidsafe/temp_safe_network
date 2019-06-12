// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::Result;
use directories::ProjectDirs;
use safe_nd::XorName;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::fs;
use std::{
    fs::File,
    io::{self, BufReader},
    path::PathBuf,
};

const CONFIG_DIR_QUALIFIER: &str = "net";
const CONFIG_DIR_ORGANIZATION: &str = "MaidSafe";
const CONFIG_DIR_APPLICATION: &str = "vault";
const CONFIG_FILE: &str = "vault.config";
const LOG_CONFIG_FILE: &str = "vault.log.config";

/// Lets a vault configure a wallet address and storage limit.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    /// Used to store the address where SafeCoin will be sent.
    pub wallet_address: Option<XorName>,
    /// Upper limit for allowed network storage on this vault.
    pub max_capacity: Option<u64>, // measured by Bytes
    /// Root directory for chunk_store directories.
    pub chunk_store_root: Option<String>,
}

/// Reads the default vault config file.
pub fn read_config_file() -> Result<Config> {
    let path = dirs()?.config_dir().join(CONFIG_FILE);
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

/// Returns the path to the logging configuration file.
pub fn log_config_file_path() -> Option<PathBuf> {
    Some(dirs().ok()?.config_dir().join(LOG_CONFIG_FILE))
}

/// Writes a Vault config file **for use by tests and examples**.
///
/// The file is written to the `current_bin_dir()` with the appropriate file name.
///
/// N.B. This method should only be used as a utility for test and examples.  In normal use cases,
/// the config file should be created by the Vault's installer.
#[cfg(test)]
#[allow(dead_code)]
pub fn write_config_file(config: &Config) -> Result<PathBuf> {
    let dirs = dirs()?;
    let dir = dirs.config_dir();
    fs::create_dir_all(dir)?;

    let path = dir.join(CONFIG_FILE);
    let mut file = File::create(&path)?;
    serde_json::to_writer_pretty(&mut file, config)?;
    file.sync_all()?;

    Ok(path)
}

fn dirs() -> Result<ProjectDirs> {
    ProjectDirs::from(
        CONFIG_DIR_QUALIFIER,
        CONFIG_DIR_ORGANIZATION,
        CONFIG_DIR_APPLICATION,
    )
    .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found").into())
}

#[cfg(test)]
mod test {
    use super::Config;
    use serde_json;
    use std::{fs::File, io::Read, path::Path};
    use unwrap::unwrap;

    #[ignore]
    #[test]
    fn parse_sample_config_file() {
        let path = Path::new("installer/common/sample.vault.config").to_path_buf();
        let mut file = unwrap!(File::open(&path), "Error opening {}:", path.display());
        let mut encoded_contents = String::new();
        let _ = unwrap!(
            file.read_to_string(&mut encoded_contents),
            "Error reading {}:",
            path.display()
        );
        let config: Config = unwrap!(
            serde_json::from_str(&encoded_contents),
            "Error parsing {} as JSON:",
            path.display()
        );

        assert!(
            config.wallet_address.is_some(),
            "{} is missing `wallet_address` field.",
            path.display()
        );
        assert!(
            config.max_capacity.is_some(),
            "{} is missing `max_capacity` field.",
            path.display()
        );
        assert!(
            config.chunk_store_root.is_some(),
            "{} is missing `chunk_store_root` field.",
            path.display()
        );
    }
}
