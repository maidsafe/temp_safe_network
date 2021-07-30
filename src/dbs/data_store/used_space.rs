// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Error;
use async_recursion::async_recursion;
use dashmap::DashSet;
use futures::future::join_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

#[derive(Clone, Debug)]
/// Tracking used space in supplied dirs, and providing checks to ensure max capacity isn't exceeded
pub struct UsedSpace {
    /// the maximum (inclusive) allocated space for storage
    max_capacity: u64,
    dirs: Arc<DashSet<PathBuf>>,
}

impl UsedSpace {
    /// Create new UsedSpace tracker
    pub fn new(max_capacity: u64) -> Self {
        Self {
            max_capacity,
            dirs: Arc::new(DashSet::new()),
        }
    }

    pub(crate) fn add_dir(&self, dir: &Path) {
        let _ = self.dirs.insert(dir.to_path_buf());
    }

    // pub(crate) fn remove_dir(&self, dir: &Path) {
    //     let _ = self.dirs.remove(&dir.to_path_buf());
    // }

    pub(crate) fn max_capacity(&self) -> u64 {
        self.max_capacity
    }

    pub(crate) async fn can_consume(&self, space: u64) -> bool {
        self.total()
            .await
            .checked_add(space)
            .map_or(false, |new_total| self.max_capacity >= new_total)
    }

    pub(crate) async fn total(&self) -> u64 {
        // todo: handle the error
        let handles = self
            .dirs
            .iter()
            .map(|d| d.clone())
            .map(|path| tokio::spawn(async move { get_size(path).await.map_err(Error::from) }));
        join_all(handles).await.iter().flatten().flatten().sum()
    }
}

/// Returns size in bytes of a file or directory. For directories, it recursively gets all of the
/// sizes of its children.
#[async_recursion]
async fn get_size(path: PathBuf) -> tokio::io::Result<u64> {
    let metadata = fs::metadata(&path).await?;
    let mut size = metadata.len();
    if metadata.is_dir() {
        let mut dir = fs::read_dir(&path).await?;
        while let Some(entry) = dir.next_entry().await? {
            size += get_size(entry.path()).await?;
        }
    }
    Ok(size)
}
