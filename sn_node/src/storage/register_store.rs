// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{list_files_in, prefix_tree_path, Error, Result};

use crate::UsedSpace;

use sn_interface::types::{
    utils::{deserialise, serialise},
    RegisterAddress, RegisterCmd,
};

use bincode::serialize;
use std::{
    collections::{btree_map::Entry, BTreeMap},
    path::{Path, PathBuf},
};
use tiny_keccak::{Hasher, Sha3};
use tokio::fs::{create_dir_all, metadata, read, remove_file, File};
use tokio::io::AsyncWriteExt;
use xor_name::XorName;

// Deterministic Id for a register Cmd, takes into account the underlying cmd, and all sigs
type RegisterCmdId = String;

pub(super) type RegisterLog = BTreeMap<RegisterCmdId, RegisterCmd>;

/// A disk store for Registers
#[derive(Clone, Debug)]
pub(super) struct RegisterStore {
    file_store_path: PathBuf,
    used_space: UsedSpace,
}

impl RegisterStore {
    /// Creates a new `RegisterStore` at the specified root location
    ///
    /// If the location specified already contains a `RegisterStore`, it is simply used
    ///
    /// Used space of the dir is tracked
    pub(crate) fn new<P: AsRef<Path>>(root_path: P, used_space: UsedSpace) -> Result<Self> {
        Ok(Self {
            file_store_path: root_path.as_ref().to_path_buf(),
            used_space,
        })
    }

    fn address_to_filepath(&self, addr: &RegisterAddress) -> Result<PathBuf> {
        // this is a unique identifier of the Register,
        // since it encodes both the xorname and tag.
        let reg_id = XorName::from_content(&serialize(addr)?);
        let path = prefix_tree_path(&self.file_store_path, reg_id);

        // we need to append a folder for the file specifically so bit depth is an issue when low.
        // we use hex to get full id, not just first bytes
        Ok(path.join(hex::encode(reg_id)))
    }

    pub(crate) async fn list_all_reg_addrs(&self) -> Vec<RegisterAddress> {
        let iter = list_files_in(&self.file_store_path)
            .into_iter()
            .filter_map(|e| e.parent().map(|parent| (parent.to_path_buf(), e.clone())));

        let mut addrs = BTreeMap::<PathBuf, RegisterAddress>::new();
        for (parent, op_file) in iter {
            if let Entry::Vacant(vacant) = addrs.entry(parent) {
                if let Ok(Ok(cmd)) = read(op_file)
                    .await
                    .map(|serialized_data| deserialise::<RegisterCmd>(&serialized_data))
                {
                    let _existing = vacant.insert(cmd.dst_address());
                }
            }
        }

        addrs.into_iter().map(|(_, addr)| addr).collect()
    }

    pub(crate) fn can_add(&self, size: usize) -> bool {
        self.used_space.can_add(size)
    }

    pub(crate) async fn delete_data(&self, addr: &RegisterAddress) -> Result<()> {
        let filepath = self.address_to_filepath(addr)?;
        let meta = metadata(filepath.clone()).await?;
        remove_file(filepath).await?;
        self.used_space.decrease(meta.len() as usize);
        Ok(())
    }

    pub(crate) fn data_file_exists(&self, addr: &RegisterAddress) -> Result<bool> {
        let filepath = self.address_to_filepath(addr)?;
        Ok(filepath.exists())
    }

    /// Opens the log of RegisterCmds for a given register address.
    /// Creates a new log if no data is found
    pub(crate) async fn open_reg_log_from_disk(
        &self,
        addr: &RegisterAddress,
    ) -> Result<(RegisterLog, PathBuf)> {
        let mut register_log = RegisterLog::new();

        let path = self.address_to_filepath(addr)?;
        if path.exists() {
            trace!("Register log path exists: {}", path.display());
            for filepath in list_files_in(&path) {
                let serialized_data = read(filepath).await?;
                let cmd: RegisterCmd = deserialise(&serialized_data)?;
                let _existing = register_log.insert(register_operation_id(&cmd)?, cmd);
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
    pub(crate) async fn append_and_write_log_to_disk(
        &self,
        cmd: RegisterCmd,
        mut log: RegisterLog,
        path: &Path,
    ) -> Result<()> {
        trace!(
            "Appending cmd and writing to register log at {}",
            path.display()
        );

        let reg_cmd_id = register_operation_id(&cmd)?;

        if log.contains_key(&reg_cmd_id) {
            return Err(Error::RegCmdOperationExists(reg_cmd_id));
        }

        let _old_cmd = log.insert(reg_cmd_id, cmd);

        create_dir_all(&path).await?;

        for (reg_cmd_id, cmd) in log {
            // TODO do we want to fail here if one entry fails?
            self.write_register_cmd(&reg_cmd_id, &cmd, path).await?;
        }

        trace!("Log writing successful at {}", path.display());
        Ok(())
    }

    /// Persists a RegisterCmd to disk
    pub(crate) async fn write_register_cmd(
        &self,
        reg_cmd_id: &RegisterCmdId,
        cmd: &RegisterCmd,
        path: &Path,
    ) -> Result<()> {
        let serialized_data = serialise(cmd)?;

        let path = path.join(reg_cmd_id.clone());
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

        trace!("RegisterCmd writing successful for id {reg_cmd_id}");
        Ok(())
    }
}

/// Gets an operation id, deterministic for a RegisterCmd, it takes
/// the full Cmd and all signers into consideration
fn register_operation_id(cmd: &RegisterCmd) -> Result<RegisterCmdId> {
    let mut hasher = Sha3::v256();

    let bytes = serialise(cmd)?;
    let mut output = [0; 64];
    hasher.update(&bytes);
    hasher.finalize(&mut output);

    let id = hex::encode(output);
    Ok(id)
}
