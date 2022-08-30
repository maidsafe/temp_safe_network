// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{list_files_in, prefix_tree_path, Result};

use crate::UsedSpace;

use sn_interface::{
    messaging::data::{CreateRegister, SignedRegisterCreate},
    types::{
        register::Register,
        utils::{deserialise, serialise},
        RegisterAddress, RegisterCmd, SectionAuth,
    },
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

pub(super) type RegisterLog = Vec<RegisterCmd>;

#[derive(Clone, Debug)]
pub(super) struct StoredRegister {
    pub(super) state: Option<(Register, SectionAuth)>,
    pub(super) ops_log: RegisterLog,
    pub(super) ops_log_path: PathBuf,
}

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
    pub(super) fn new<P: AsRef<Path>>(root_path: P, used_space: UsedSpace) -> Result<Self> {
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

    pub(super) async fn list_all_reg_addrs(&self) -> Vec<RegisterAddress> {
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

    pub(super) fn can_add(&self, size: usize) -> bool {
        self.used_space.can_add(size)
    }

    pub(super) async fn delete_data(&self, addr: &RegisterAddress) -> Result<()> {
        let filepath = self.address_to_filepath(addr)?;
        let meta = metadata(filepath.clone()).await?;
        remove_file(filepath).await?;
        self.used_space.decrease(meta.len() as usize);
        Ok(())
    }

    /// Opens the log of RegisterCmds for a given register address.
    /// Creates a new log if no data is found
    pub(super) async fn open_reg_log_from_disk(
        &self,
        addr: &RegisterAddress,
    ) -> Result<StoredRegister> {
        let path = self.address_to_filepath(addr)?;
        let mut stored_reg = StoredRegister {
            state: None,
            ops_log: RegisterLog::new(),
            ops_log_path: path.clone(),
        };

        if !path.exists() {
            trace!(
                "Register log does not exist, creating a new one {}",
                path.display()
            );
            return Ok(stored_reg);
        }

        trace!("Register log path exists: {}", path.display());
        for filepath in list_files_in(&path) {
            match read(&filepath)
                .await
                .map(|serialized_data| deserialise::<RegisterCmd>(&serialized_data))
            {
                Ok(Ok(reg_cmd)) => {
                    stored_reg.ops_log.push(reg_cmd.clone());

                    if let RegisterCmd::Create {
                        cmd:
                            SignedRegisterCreate {
                                op: CreateRegister { name, tag, policy },
                                ..
                            },
                        section_auth,
                    } = reg_cmd
                    {
                        // TODO: if we already have read a RegisterCreate op, check if there
                        // is any difference with this other one,...if so perhaps log a warning?
                        if stored_reg.state.is_none() {
                            let register = Register::new(*policy.owner(), name, tag, policy);
                            stored_reg.state = Some((register, section_auth));
                        }
                    }
                }
                other => {
                    warn!(
                        "Ignoring corrupted register cmd from storage found at {}: {:?}",
                        filepath.display(),
                        other
                    )
                }
            }
        }

        Ok(stored_reg)
    }

    /// Persists a RegisterLog to disk
    pub(super) async fn write_log_to_disk(&self, log: &RegisterLog, path: &Path) -> Result<()> {
        trace!("Writing to register log at {}", path.display());

        create_dir_all(path).await?;

        for cmd in log {
            // TODO do we want to fail here if one entry fails?
            self.write_register_cmd(cmd, path).await?;
        }

        trace!("Log writing successful at {}", path.display());
        Ok(())
    }

    /// Persists a RegisterCmd to disk
    pub(super) async fn write_register_cmd(&self, cmd: &RegisterCmd, path: &Path) -> Result<()> {
        let reg_cmd_id = register_operation_id(cmd)?;
        let path = path.join(reg_cmd_id.clone());

        trace!("Writing cmd register log at {}", path.display());
        // it's deterministic, so they are exactly the same op so we can leave
        if path.exists() {
            trace!("RegisterCmd exists on disk, so was not written: {cmd:?}");
            // TODO: should we error?
            return Ok(());
        }

        let mut file = File::create(path).await?;

        let serialized_data = serialise(cmd)?;
        file.write_all(&serialized_data).await?;

        self.used_space.increase(std::mem::size_of::<RegisterCmd>());

        trace!("RegisterCmd writing successful for id {reg_cmd_id}");
        Ok(())
    }
}

// Gets an operation id, deterministic for a RegisterCmd, it takes
// the full Cmd and all signers into consideration
fn register_operation_id(cmd: &RegisterCmd) -> Result<RegisterCmdId> {
    let mut hasher = Sha3::v256();

    let bytes = serialise(cmd)?;
    let mut output = [0; 64];
    hasher.update(&bytes);
    hasher.finalize(&mut output);

    let id = hex::encode(output);
    Ok(id)
}
