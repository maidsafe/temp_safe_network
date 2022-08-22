// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{Error, Result};

use crate::UsedSpace;

use sn_interface::{
    messaging::data::DataCmd,
    types::{
        utils::{deserialise, serialise},
        Chunk, ChunkAddress, RegisterAddress, RegisterCmd, RegisterCmdId,
        ReplicatedDataAddress as DataAddress,
    },
};

use crate::dbs::reg_op_store::RegisterLog;

use bytes::Bytes;
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, metadata, read, remove_file, File};
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;
use xor_name::{Prefix, XorName};

const BIT_TREE_DEPTH: usize = 20;
const FILE_DB_DIR: &str = "filedb";

/// A disk store for chunks
#[derive(Clone, Debug)]
pub(crate) struct FileStore {
    bit_tree_depth: usize,
    file_store_path: PathBuf,
    used_space: UsedSpace,
}

impl FileStore {
    /// Creates a new `FileStore` at location `root/CHUNK_DB_DIR`
    ///
    /// If the location specified already contains a `FileStore`, it is simply used
    ///
    /// Used space of the dir is tracked
    pub(crate) fn new<P: AsRef<Path>>(root: P, used_space: UsedSpace) -> Result<Self> {
        let chunk_store_path = root.as_ref().join(FILE_DB_DIR);

        Ok(FileStore {
            bit_tree_depth: BIT_TREE_DEPTH,
            file_store_path: chunk_store_path,
            used_space,
        })
    }

    // ---------------------- helper methods ----------------------

    // Helper that returns the prefix tree path of depth `bit_count` for a given xorname
    // Example:
    // - with a xorname with starting bits `010001110110....`
    // - and a bit_count of `6`
    // returns the path `FILE_STORE_PATH/0/1/0/0/0/1`
    // If the provided bit count is larger than `self.bit_tree_depth`, uses `self.bit_tree_depth`
    // to stay within the prefix tree path
    fn prefix_tree_path(&self, xorname: XorName, bit_count: usize) -> PathBuf {
        let bin = format!("{:b}", xorname);
        let prefix_dir_path: PathBuf = bin
            .chars()
            .take(std::cmp::min(bit_count, self.bit_tree_depth))
            .map(|c| format!("{}", c))
            .collect();

        let mut path = self.file_store_path.clone();
        path.push(prefix_dir_path);
        path
    }

    fn address_to_filepath(&self, addr: &DataAddress) -> Result<PathBuf> {
        let path = if let DataAddress::Register(reg_addr) = addr {
            self.prefix_tree_path(reg_addr.id()?, self.bit_tree_depth)
        } else {
            let xorname = *addr.name();
            let mut path = self.prefix_tree_path(xorname, self.bit_tree_depth);
            let filename = addr.encode_to_zbase32()?;
            path.push(filename);
            path
        };

        Ok(path)
    }

    pub(crate) fn list_all_chunk_addrs(&self) -> Vec<ChunkAddress> {
        self.list_all_files()
            .iter()
            .filter_map(|filepath| self.chunk_filepath_to_address(filepath).ok())
            .collect()
    }

    pub(crate) fn list_all_reg_addrs(&self) -> Vec<RegisterAddress> {
        self.list_all_files()
            .iter()
            .filter_map(|filepath| self.reg_filepath_to_address(filepath).ok())
            .collect()
    }

    fn chunk_filepath_to_address(&self, path: &Path) -> Result<ChunkAddress> {
        let filename = path
            .file_name()
            .ok_or(Error::NoFilename)?
            .to_str()
            .ok_or(Error::InvalidFilename)?;

        Ok(ChunkAddress::decode_from_zbase32(filename)?)
    }

    fn reg_filepath_to_address(&self, path: &Path) -> Result<RegisterAddress> {
        let parent_path = path.parent().ok_or(Error::InvalidFilename)?;

        futures::executor::block_on(async {
            let mut ops_files = tokio::fs::read_dir(&parent_path).await?;
            while let Some(entry) = ops_files.next_entry().await? {
                if entry.metadata().await?.is_file() {
                    let serialized_data = read(entry.path()).await?;
                    let cmd: RegisterCmd = deserialise(&serialized_data)?;
                    return Ok(cmd.dst_address());
                }
            }

            Err(Error::NoFilename)
        })
    }

    // ---------------------- api methods ----------------------

    pub(crate) fn can_add(&self, size: usize) -> bool {
        self.used_space.can_add(size)
    }

    pub(crate) async fn write_data(&self, data: DataCmd) -> Result<DataAddress> {
        let addr = data.address();
        let filepath = self.address_to_filepath(&addr)?;
        if let Some(dirs) = filepath.parent() {
            create_dir_all(dirs).await?;
        }

        let mut file = File::create(filepath).await?;

        // Only chunk go through here
        if let DataCmd::StoreChunk(chunk) = data {
            file.write_all(chunk.value()).await?;
            self.used_space.increase(chunk.value().len());
        }

        Ok(addr)
    }

    #[allow(dead_code)]
    pub(crate) async fn delete_data(&self, addr: &DataAddress) -> Result<()> {
        let filepath = self.address_to_filepath(addr)?;
        let meta = metadata(filepath.clone()).await?;
        remove_file(filepath).await?;
        self.used_space.decrease(meta.len() as usize);
        Ok(())
    }

    pub(crate) async fn read_data(&self, addr: &DataAddress) -> Result<Chunk> {
        let file_path = self.address_to_filepath(addr)?;
        let bytes = Bytes::from(read(file_path).await?);
        let chunk = Chunk::new(bytes);
        Ok(chunk)
    }

    pub(crate) fn data_file_exists(&self, addr: &DataAddress) -> Result<bool> {
        let filepath = self.address_to_filepath(addr)?;
        Ok(filepath.exists())
    }

    pub(crate) fn list_all_files(&self) -> Vec<PathBuf> {
        list_files_in(&self.file_store_path)
    }

    #[allow(unused)]
    /// quickly find chunks related or not to a section, might be useful when adults change sections
    /// not used yet
    pub(crate) fn list_files_without_prefix(&self, prefix: Prefix) -> Vec<PathBuf> {
        let prefix_path = self.prefix_tree_path(prefix.name(), prefix.bit_count());
        self.list_all_files()
            .into_iter()
            .filter(|path| !path.starts_with(&prefix_path.as_path()))
            .collect()
    }

    #[allow(unused)]
    /// quickly find chunks related or not to a section, might be useful when adults change sections
    /// not used yet
    pub(crate) fn list_files_with_prefix(&self, prefix: Prefix) -> Vec<PathBuf> {
        let prefix_path = self.prefix_tree_path(prefix.name(), prefix.bit_count());
        list_files_in(prefix_path.as_path())
    }

    /// Opens the log of RegisterCmds for a given register address. Creates a new log if no data is found
    pub(crate) async fn open_log_from_disk(
        &self,
        addr: &RegisterAddress,
    ) -> Result<(RegisterLog, PathBuf)> {
        let mut register_log = RegisterLog::new();

        let path = self.address_to_filepath(&DataAddress::Register(*addr))?;
        if path.exists() {
            trace!("Register log path exists: {}", path.display());

            let mut ops_files = tokio::fs::read_dir(&path).await?;
            while let Some(entry) = ops_files.next_entry().await? {
                if entry.metadata().await?.is_file() {
                    let serialized_data = read(entry.path()).await?;
                    let cmd: RegisterCmd = deserialise(&serialized_data)?;
                    let _existing = register_log.insert(cmd.register_operation_id()?, cmd.clone());
                }
            }
        } else {
            trace!(
                "Register log does not exist, creating a new one {}",
                path.display()
            );
        }

        Ok((register_log, path))
    }

    /// Persists a RegisterLog to disk
    pub(crate) async fn write_log_to_disk(&self, log: &RegisterLog, path: &Path) -> Result<()> {
        trace!("Writing to register log at {}", path.display());

        create_dir_all(&path).await?;

        for (reg_id, cmd) in log {
            // TODO do we want to fail here if one entry fails?
            self.write_register_cmd(reg_id, cmd, path).await?;
        }

        trace!("Log writing successful at {}", path.display());
        Ok(())
    }

    /// Persists a RegisterCmd to disk
    pub(crate) async fn write_register_cmd(
        &self,
        reg_id: &RegisterCmdId,
        cmd: &RegisterCmd,
        path: &Path,
    ) -> Result<()> {
        let serialized_data = serialise(cmd)?;

        let reg_id = hex::encode(reg_id);
        let path = path.join(reg_id.clone());
        trace!("Writing cmd register log at {}", path.display());
        // it's deterministic, so they are exactly the same op so we can leave
        if path.exists() {
            trace!("RegisterCmd exists on disk, so was not written: {cmd:?}");
            // TODO: should we error?
            return Ok(());
        }

        let mut file = File::create(path).await?;

        file.write_all(&serialized_data).await?;

        self.used_space.increase(std::mem::size_of::<RegisterCmd>());

        trace!("RegisterCmd writing successful for id {reg_id}");
        Ok(())
    }
}

fn list_files_in(path: &Path) -> Vec<PathBuf> {
    if !path.exists() {
        return vec![];
    }
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| match e {
            Ok(direntry) => Some(direntry),
            Err(err) => {
                warn!("FileStore: failed to process file entry: {}", err);
                None
            }
        })
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sn_interface::types::utils::random_bytes;

    use futures::future::join_all;
    use rayon::prelude::*;
    use tempfile::tempdir;

    fn init_file_store() -> FileStore {
        let root = tempdir().expect("Failed to create temporary directory for chunk disk store");
        FileStore::new(root.path(), UsedSpace::new(usize::MAX))
            .expect("Failed to create chunk disk store")
    }

    #[tokio::test]
    #[ignore]
    async fn test_write_read_chunk() {
        let store = init_file_store();
        // test that a range of different chunks return the written chunk
        for _ in 0..10 {
            let chunk = Chunk::new(random_bytes(100));

            let addr = store
                .write_data(DataCmd::StoreChunk(chunk.clone()))
                .await
                .expect("Failed to write chunk.");

            let read_chunk = store.read_data(&addr).await.expect("Failed to read chunk.");

            assert_eq!(chunk.value(), read_chunk.value());
        }
    }

    #[tokio::test]
    async fn test_write_read_async_multiple_chunks() {
        let store = init_file_store();
        let size = 100;
        let chunks: Vec<Chunk> = std::iter::repeat_with(|| Chunk::new(random_bytes(size)))
            .take(7)
            .collect();
        write_and_read_chunks(&chunks, store).await;
    }

    #[tokio::test]
    async fn test_write_read_async_multiple_identical_chunks() {
        let store = init_file_store();
        let chunks: Vec<Chunk> = std::iter::repeat(Chunk::new(Bytes::from("test_concurrent")))
            .take(7)
            .collect();
        write_and_read_chunks(&chunks, store).await;
    }

    async fn write_and_read_chunks(chunks: &[Chunk], store: FileStore) {
        // write all chunks
        let tasks = chunks
            .iter()
            .map(|c| store.write_data(DataCmd::StoreChunk(c.clone())));
        let results = join_all(tasks).await;

        // read all chunks
        let tasks = results.iter().flatten().map(|addr| store.read_data(addr));
        let results = join_all(tasks).await;
        let read_chunks: Vec<&Chunk> = results.iter().flatten().collect();

        // verify all written were read
        assert!(chunks
            .par_iter()
            .all(|c| read_chunks.iter().any(|r| r.value() == c.value())))
    }
}
