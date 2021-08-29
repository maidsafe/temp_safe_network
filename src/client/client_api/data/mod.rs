// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod batching;
mod pac_man;
mod upload;

#[allow(unused)]
pub(crate) use batching::{Batch, Batching, BatchingConfig};
#[allow(unused)]
pub(crate) use pac_man::{get_data_chunks, get_file_chunks, SecretKeyLevel};

use crate::dbs::{KvStore, ToDbKey};
use crate::types::Token;

pub(crate) type ItemKey = String;
pub(crate) type Db<K, V> = KvStore<K, V>;

/// A stash of tokens
pub(crate) trait Stash: Clone {
    /// The total value of the stash.
    fn value(&self) -> Token;
    /// Removes and returns dbcs up to the requested
    /// value, if exists.
    fn take(&self, value: Token) -> Vec<Dbc>;
}

pub(crate) struct Dbc {
    #[allow(unused)]
    pub(crate) value: Token,
}

#[cfg(test)]
mod tests {
    use super::{Batch, Batching, BatchingConfig, Stash};
    use crate::client::{utils::random_bytes, Error, Result};
    use crate::url::Scope;
    use crate::UsedSpace;
    use std::{collections::BTreeMap, iter::FromIterator};
    use tempfile::tempdir;
    use tokio::time::{sleep, Duration};

    //
    #[tokio::test(flavor = "multi_thread")]
    async fn basic() -> Result<()> {
        let temp_dir = tempdir().map_err(|e| Error::Generic(e.to_string()))?;
        let root_dir = temp_dir.path().to_path_buf();
        let cfg = BatchingConfig {
            pool_count: 4,
            pool_limit: 10,
            root_dir,
            used_space: UsedSpace::new(u64::MAX),
        };
        let b = Batching::new(cfg, TestStash {})?;

        let size = 1234567;

        for _ in 0..4 {
            let bytes_0 = random_bytes(size);
            let bytes_1 = random_bytes(size);
            let bytes_2 = random_bytes(size);
            let bytes_3 = random_bytes(size);
            let bytes_4 = random_bytes(size);
            let bytes_5 = random_bytes(size);
            let bytes_6 = random_bytes(size);
            let bytes_7 = random_bytes(size);
            let bytes_8 = random_bytes(size);
            let bytes_9 = random_bytes(size);

            let batch = Batch {
                files: BTreeMap::new(),
                values: BTreeMap::from_iter(vec![
                    ("A".to_string(), (bytes_0, Scope::Public)),
                    ("B".to_string(), (bytes_1, Scope::Public)),
                    ("C".to_string(), (bytes_2, Scope::Public)),
                    ("D".to_string(), (bytes_3, Scope::Public)),
                    ("E".to_string(), (bytes_4, Scope::Public)),
                    ("F".to_string(), (bytes_5, Scope::Public)),
                    ("G".to_string(), (bytes_6, Scope::Public)),
                    ("H".to_string(), (bytes_7, Scope::Public)),
                    ("I".to_string(), (bytes_8, Scope::Public)),
                    ("J".to_string(), (bytes_9, Scope::Public)),
                ]),
                reg_ops: BTreeMap::new(),
                msg_quotas: BTreeMap::new(),
            };

            b.push(batch);

            sleep(Duration::from_secs(3)).await;
        }

        Ok(())
    }

    #[derive(Clone)]
    struct TestStash {}

    impl Stash for TestStash {
        fn value(&self) -> crate::types::Token {
            crate::types::Token::from_nano(1_000_000_000)
        }

        fn take(&self, _value: crate::types::Token) -> Vec<super::Dbc> {
            vec![]
        }
    }
}
