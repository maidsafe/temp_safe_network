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

pub(crate) use pac_man::{encrypt_blob, to_chunk, DataMapLevel};

use crate::client::{Error, Result};

use bytes::Bytes;
use self_encryption::MIN_ENCRYPTABLE_BYTES;

/// Data of size more than 0 bytes less than [`MIN_ENCRYPTABLE_BYTES`] bytes.
///
/// A `Spot` cannot be self-encrypted, thus is encrypted using the client encryption keys instead.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub(crate) struct Spot {
    bytes: Bytes,
}

/// Data of size larger than or equal to [`MIN_ENCRYPTABLE_BYTES`] bytes.
///
/// A `Blob` is spread across multiple chunks in the network.
/// This is done using self-encryption, which produces at least 4 chunks (3 for the contents, 1 for the `DataMap`).
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub(crate) struct Blob {
    bytes: Bytes,
}

impl Spot {
    /// Enforces size > 0 and size < [`MIN_ENCRYPTABLE_BYTES`] bytes.
    pub(crate) fn new(bytes: Bytes) -> Result<Self> {
        if bytes.len() >= MIN_ENCRYPTABLE_BYTES {
            Err(Error::TooLargeToBeSpot)
        } else if bytes.is_empty() {
            Err(Error::EmptyBytesProvided)
        } else {
            Ok(Self { bytes })
        }
    }

    /// Returns the bytes.
    pub(crate) fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }
}

impl Blob {
    /// Enforces size >= [`MIN_ENCRYPTABLE_BYTES`] bytes.
    pub(crate) fn new(bytes: Bytes) -> Result<Self> {
        if MIN_ENCRYPTABLE_BYTES > bytes.len() {
            Err(Error::TooSmallToBeBlob)
        } else {
            Ok(Self { bytes })
        }
    }

    /// Returns the bytes.
    pub(crate) fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }
}

pub(crate) use batching::{Batch, Batching, BatchingConfig};

use crate::types::Token;

pub(crate) type ItemKey = String;

/// A stash of tokens
pub(crate) trait Stash: Clone {
    /// The total value of the stash.
    fn value(&self) -> Token;
    /// Removes and returns dbcs up to the requested
    /// value, if exists.
    fn take(&self, value: Token) -> Vec<Dbc>;
}

#[derive(Debug)]
pub(crate) struct Dbc {
    #[allow(unused)]
    pub(crate) value: Token,
}

#[cfg(test)]
mod tests {
    use super::{Batch, Batching, BatchingConfig, Stash};

    use crate::client::{Error, Result};
    use crate::types::{utils::random_bytes, Scope};
    use crate::UsedSpace;

    use std::{collections::BTreeMap, iter::FromIterator};
    use tempfile::tempdir;
    use tokio::time::{sleep, Duration};

    //
    #[tokio::test(flavor = "multi_thread")]
    async fn basic() -> Result<()> {
        let temp_dir = tempdir().map_err(|_| Error::CouldNotCreateRootDir)?;
        let root_dir = temp_dir.path().to_path_buf();
        let cfg = BatchingConfig {
            pool_count: 4,
            pool_limit: 10,
            root_dir,
            used_space: UsedSpace::new(usize::MAX),
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
