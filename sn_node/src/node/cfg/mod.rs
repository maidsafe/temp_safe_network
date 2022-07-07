// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// Configuration
pub mod config_handler;

/// File storage for keypairs
pub(crate) mod keypair_storage;

pub use test_utils::*;

mod test_utils {
    use super::config_handler::Config;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    /// Create a register store for routing examples
    pub fn create_test_max_capacity_and_root_storage() -> eyre::Result<(usize, PathBuf)> {
        let random_filename: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        let root_dir = tempdir().map_err(|e| eyre::eyre!(e.to_string()))?;
        let storage_dir = Path::new(root_dir.path()).join(random_filename);
        let config = Config::default();

        Ok((config.max_capacity(), storage_dir))
    }
}
