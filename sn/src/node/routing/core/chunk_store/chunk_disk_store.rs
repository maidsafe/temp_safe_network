use xor_name::{Prefix, XorName};

use bytes::Bytes;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use std::io::Read;
use tokio::io::AsyncWriteExt;

use super::{Error, Result};
use crate::dbs::UsedSpace;
use crate::types::{Chunk, ChunkAddress};

const BIT_TREE_DEPTH: usize = 20;
const CHUNK_DB_DIR: &str = "chunkdb";

/// A disk store for chunks
#[derive(Clone)]
pub(crate) struct ChunkDiskStore {
    bit_tree_depth: usize,
    chunk_store_path: PathBuf,
    used_space: UsedSpace,
}

impl ChunkDiskStore {
    /// Creates a new `ChunkDiskStore` at location `root/CHUNK_DB_DIR`
    ///
    /// If the location specified already contains a ChunkDiskStore, it is simply used
    ///
    /// Used space of the dir is tracked
    pub(crate) fn new<P: AsRef<Path>>(root: P, used_space: UsedSpace) -> Result<Self> {
        let dir = root.as_ref().join(CHUNK_DB_DIR);
        used_space.add_dir(&dir);

        Ok(ChunkDiskStore {
            bit_tree_depth: BIT_TREE_DEPTH,
            chunk_store_path: dir,
            used_space,
        })
    }

    // ---------------------- helper methods ----------------------

    // Helper that returns the prefix tree path of depth `bit_count` for a given xorname
    // Example:
    // - with a xorname with starting bits `010001110110....`
    // - and a bit_count of `6`
    // returns the path `CHUNK_STORE_PATH/0/1/0/0/0/1`
    // If the provided bit count is larger than `self.bit_tree_depth`, uses `self.bit_tree_depth`
    // to stay within the prefix tree path
    fn prefix_tree_path(&self, xorname: XorName, bit_count: usize) -> PathBuf {
        let bin = format!("{:b}", xorname);
        let prefix_dir_list: Vec<String> = bin
            .chars()
            .take(std::cmp::min(bit_count, self.bit_tree_depth))
            .map(|c| format!("{}", c))
            .collect();
        let prefix_dir_path: PathBuf = prefix_dir_list.iter().collect();

        let mut path = self.chunk_store_path.clone();
        path.push(prefix_dir_path);
        path
    }

    fn address_to_filepath(&self, addr: &ChunkAddress) -> Result<PathBuf> {
        let xorname = *addr.name();
        let filename = addr.encode_to_zbase32()?;
        let mut path = self.prefix_tree_path(xorname, self.bit_tree_depth);
        path.push(filename);
        Ok(path)
    }

    fn filepath_to_address(&self, path: &str) -> Result<ChunkAddress> {
        let filename = Path::new(path)
            .file_name()
            .ok_or(Error::NoFilename)?
            .to_str()
            .ok_or(Error::InvalidFilename)?;
        Ok(ChunkAddress::decode_from_zbase32(filename)?)
    }

    // ---------------------- public (crate) methods ----------------------

    pub(crate) async fn total_used_space(&self) -> u64 {
        self.used_space.total().await
    }

    pub(crate) async fn used_space_ratio(&self) -> f64 {
        self.used_space.ratio().await
    }

    pub(crate) async fn write_chunk(&self, data: &Chunk) -> Result<()> {
        if !self.used_space.can_consume(data.value().len() as u64).await {
            return Err(Error::NotEnoughSpace);
        }

        let addr = data.address();
        let filepath = self.address_to_filepath(addr)?;
        if let Some(dirs) = filepath.parent() {
            tokio::fs::create_dir_all(dirs).await?;
        }

        let mut file = tokio::fs::File::create(filepath).await?;
        file.write_all(data.value()).await?;
        Ok(())
    }

    pub(crate) fn delete_chunk(&self, addr: &ChunkAddress) -> Result<()> {
        let filepath = self.address_to_filepath(addr)?;
        std::fs::remove_file(filepath)?;
        Ok(())
    }

    pub(crate) fn read_chunk(&self, addr: &ChunkAddress) -> Result<Chunk> {
        let filepath = self.address_to_filepath(addr)?;

        let mut f = std::fs::File::open(filepath)?;
        let mut buffer = Vec::new();
        let _bytes_read = f.read_to_end(&mut buffer)?;

        let bytes = Bytes::from(buffer);
        let chunk = Chunk::new(bytes);
        Ok(chunk)
    }

    pub(crate) fn chunk_file_exists(&self, addr: &ChunkAddress) -> Result<bool> {
        let filepath = self.address_to_filepath(addr)?;
        Ok(filepath.exists())
    }

    pub(crate) fn list_all_files(&self) -> Result<Vec<String>> {
        list_files_in(&self.chunk_store_path)
    }

    pub(crate) fn list_all_chunk_addresses(&self) -> Result<Vec<ChunkAddress>> {
        let all_files = self.list_all_files()?;
        let all_addrs = all_files
            .iter()
            .map(|filepath| self.filepath_to_address(filepath))
            .collect();
        all_addrs
    }

    pub(crate) fn list_files_without_prefix(&self, prefix: Prefix) -> Result<Vec<String>> {
        let all_files = self.list_all_files()?;
        let prefix_path = self.prefix_tree_path(prefix.name(), prefix.bit_count());
        let outside_prefix = all_files
            .into_iter()
            .filter(|p| !Path::new(&p).starts_with(&prefix_path.as_path()))
            .collect();
        Ok(outside_prefix)
    }

    pub(crate) fn list_files_with_prefix(&self, prefix: Prefix) -> Result<Vec<String>> {
        let prefix_path = self.prefix_tree_path(prefix.name(), prefix.bit_count());
        list_files_in(prefix_path.as_path())
    }
}

fn list_files_in(path: &Path) -> Result<Vec<String>> {
    let files = WalkDir::new(path)
        .into_iter()
        .filter_map(|e| match e {
            Ok(direntry) => Some(direntry),
            Err(err) => {
                warn!("ChunkDiskStore: failed to process file entry: {}", err);
                None
            }
        })
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().display().to_string())
        .collect();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn init_chunk_disk_store() -> ChunkDiskStore {
        let root = tempdir().expect("Failed to create temporary directory for chunk disk store");
        let used_space = UsedSpace::new(u64::MAX);
        ChunkDiskStore::new(root.path(), used_space).expect("Failed to create chunk disk store")
    }

    #[tokio::test]
    async fn test_write_read_chunk() {
        let cds = init_chunk_disk_store();

        let chunk = Chunk::new(Bytes::from("test"));
        let addr = &chunk.address();

        cds.write_chunk(&chunk)
            .await
            .expect("Failed to write chunk.");
        let read_chunk = cds.read_chunk(addr).expect("Failed to read chunk.");

        assert_eq!(chunk.value(), read_chunk.value());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_write_read_async_multiple_chunks() {
        let cds = init_chunk_disk_store();

        let chunk1 = Chunk::new(Bytes::from("test1"));
        let chunk2 = Chunk::new(Bytes::from("test2"));
        let chunk3 = Chunk::new(Bytes::from("test3"));
        let chunk4 = Chunk::new(Bytes::from("test4"));
        let addr1 = &chunk1.address();
        let addr2 = &chunk2.address();
        let addr3 = &chunk3.address();
        let addr4 = &chunk4.address();

        let (res1, res2, res3, res4) = tokio::join!(
            cds.write_chunk(&chunk1),
            cds.write_chunk(&chunk2),
            cds.write_chunk(&chunk3),
            cds.write_chunk(&chunk4),
        );
        res1.expect("error writing chunk1");
        res2.expect("error writing chunk2");
        res3.expect("error writing chunk3");
        res4.expect("error writing chunk4");

        let (read_chunk1, read_chunk2, read_chunk3, read_chunk4) = (
            cds.read_chunk(addr1),
            cds.read_chunk(addr2),
            cds.read_chunk(addr3),
            cds.read_chunk(addr4),
        );

        assert_eq!(
            chunk1.value(),
            read_chunk1.expect("error reading chunk 1").value()
        );
        assert_eq!(
            chunk2.value(),
            read_chunk2.expect("error reading chunk 2").value()
        );
        assert_eq!(
            chunk3.value(),
            read_chunk3.expect("error reading chunk 3").value()
        );
        assert_eq!(
            chunk4.value(),
            read_chunk4.expect("error reading chunk 4").value()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_write_read_async_multiple_identical_chunks() {
        let cds = init_chunk_disk_store();

        let chunk1 = Chunk::new(Bytes::from("test_concurrent"));
        let chunk2 = Chunk::new(Bytes::from("test_concurrent"));
        let chunk3 = Chunk::new(Bytes::from("test_concurrent"));
        let chunk4 = Chunk::new(Bytes::from("test_concurrent"));
        let addr1 = &chunk1.address();
        let addr2 = &chunk2.address();
        let addr3 = &chunk3.address();
        let addr4 = &chunk4.address();

        let (res1, res2, res3, res4) = tokio::join!(
            cds.write_chunk(&chunk1),
            cds.write_chunk(&chunk2),
            cds.write_chunk(&chunk3),
            cds.write_chunk(&chunk4),
        );
        res1.expect("error writing chunk1");
        res2.expect("error writing chunk2");
        res3.expect("error writing chunk3");
        res4.expect("error writing chunk4");

        let (read_chunk1, read_chunk2, read_chunk3, read_chunk4) = (
            cds.read_chunk(addr1),
            cds.read_chunk(addr2),
            cds.read_chunk(addr3),
            cds.read_chunk(addr4),
        );

        assert_eq!(
            chunk1.value(),
            read_chunk1.expect("error reading chunk 1").value()
        );
        assert_eq!(
            chunk2.value(),
            read_chunk2.expect("error reading chunk 2").value()
        );
        assert_eq!(
            chunk3.value(),
            read_chunk3.expect("error reading chunk 3").value()
        );
        assert_eq!(
            chunk4.value(),
            read_chunk4.expect("error reading chunk 4").value()
        );
    }
}
