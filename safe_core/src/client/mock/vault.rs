// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::Account;
use super::DataId;
use crate::client::mock::routing::unlimited_muts;
use crate::client::XorNameConverter;
use crate::config_handler::{Config, DevConfig};
use fs2::FileExt;
use maidsafe_utilities::serialisation::{deserialise, serialise};
use routing::{Authority, ClientError, ImmutableData, MutableData as OldMutableData, XorName};
use rust_sodium::crypto::sign;
use safe_nd::mutable_data::{
    MutableData as NewMutableData, MutableDataRef, SeqMutableData, UnseqMutableData,
};
use safe_nd::request::{Request, Requester};
use safe_nd::response::{Response, Transaction};
use safe_nd::{Coins, Error, Message, MessageId};
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;
use std::time::SystemTime;
use tiny_keccak::sha3_256;

const FILE_NAME: &str = "MockVault";

pub struct Vault {
    cache: Cache,
    config: Config,
    store: Box<Store>,
}

// Initializes mock-vault path with the following precedence:
// 1. "SAFE_MOCK_VAULT_PATH" env var
// 2. DevConfig `mock_vault_path` option
// 3. default temp dir
fn init_vault_path(devconfig: Option<&DevConfig>) -> PathBuf {
    match env::var("SAFE_MOCK_VAULT_PATH") {
        Ok(path) => PathBuf::from(path),
        Err(_) => match devconfig.and_then(|dev| dev.mock_vault_path.clone()) {
            Some(path) => PathBuf::from(path),
            None => env::temp_dir(),
        },
    }
}

// Initializes vault storage. The type of storage is chosen with the following precedence:
// 1. "SAFE_MOCK_IN_MEMORY_STORAGE" env var => in-memory storage
// 2. DevConfig `mock_in_memory_storage` option => in-memory storage
// 3. Else => file storage, use path from `init_vault_path`
fn init_vault_store(config: &Config) -> Box<Store> {
    match env::var("SAFE_MOCK_IN_MEMORY_STORAGE") {
        Ok(_) => {
            // If the env var is set, override config file option.
            trace!("Mock vault: using memory store");
            Box::new(MemoryStore)
        }
        Err(_) => match config.dev {
            Some(ref dev) if dev.mock_in_memory_storage => {
                trace!("Mock vault: using memory store");
                Box::new(MemoryStore)
            }
            Some(ref dev) => {
                trace!("Mock vault: using file store");
                Box::new(FileStore::new(&init_vault_path(Some(dev))))
            }
            None => {
                trace!("Mock vault: using file store");
                Box::new(FileStore::new(&init_vault_path(None)))
            }
        },
    }
}

impl Vault {
    pub fn new(config: Config) -> Self {
        let store = init_vault_store(&config);

        Vault {
            cache: Cache {
                client_manager: HashMap::new(),
                nae_manager: HashMap::new(),
            },
            config,
            store,
        }
    }

    // Get account for the client manager name.
    pub fn get_account(&self, name: &XorName) -> Option<&Account> {
        self.cache.client_manager.get(name)
    }

    // Get mutable reference to account for the client manager name.
    pub fn get_account_mut(&mut self, name: &XorName) -> Option<&mut Account> {
        self.cache.client_manager.get_mut(name)
    }

    // Get the config for this vault.
    pub fn config(&self) -> Config {
        self.config.clone()
    }

    // Create account for the given client manager name.
    pub fn insert_account(&mut self, name: XorName) {
        let _ = self
            .cache
            .client_manager
            .insert(name, Account::new(self.config.clone()));
    }

    // Authorise read (non-mutation) operation.
    pub fn authorise_read(
        &self,
        dst: &Authority<XorName>,
        data_name: &XorName,
    ) -> Result<(), ClientError> {
        match *dst {
            Authority::NaeManager(name) if name == *data_name => Ok(()),
            x => {
                debug!("Unexpected authority for read: {:?}", x);
                Err(ClientError::InvalidOperation)
            }
        }
    }

    // Authorise coin operation.
    pub fn authorise_coin_operation(
        &self,
        _dst: &Authority<XorName>,
        _requester: &Requester,
    ) -> Result<(), Error> {
        // let dst_name = match *dst {
        //     Authority::ClientManager(name) => name,
        //     x => {
        //         debug!("Unexpected authority: {:?}", x);
        //         return Err(ClientError::InvalidOperation);
        //     }
        // };

        // let account = match self.get_account(&dst_name) {
        //     Some(account) => account,
        //     None => {
        //         debug!("Account not found for {:?}", dst);
        //         return Err(ClientError::NoSuchAccount);
        //     }
        // };

        // Check if we are the owner or app.
        // match requester {
        //     Requester::Owner(..) => true,
        //     Requester::Key(sign_pk) => match account.auth_keys().get(sign_pk) {
        //         Some(perms) => {
        //             if !perms.transfer_coins {
        //                 debug!("Mutation not authorised");
        //                 return Err(ClientError::AccessDenied);
        //             }
        //         }
        //     },
        // }

        Ok(())
    }

    // Authorise mutation operation.
    pub fn authorise_mutation(
        &self,
        dst: &Authority<XorName>,
        sign_pk: &sign::PublicKey,
    ) -> Result<(), ClientError> {
        let dst_name = match *dst {
            Authority::ClientManager(name) => name,
            x => {
                debug!("Unexpected authority for mutation: {:?}", x);
                return Err(ClientError::InvalidOperation);
            }
        };

        let account = match self.get_account(&dst_name) {
            Some(account) => account,
            None => {
                debug!("Account not found for {:?}", dst);
                return Err(ClientError::NoSuchAccount);
            }
        };

        // Check if we are the owner or app.
        let owner_name = XorName(sha3_256(&sign_pk[..]));
        if owner_name != dst_name && !account.auth_keys().contains_key(sign_pk) {
            debug!("Mutation not authorised");
            return Err(ClientError::AccessDenied);
        }

        let unlimited_mut = unlimited_muts(&self.config);
        if !unlimited_mut && account.account_info().mutations_available == 0 {
            return Err(ClientError::LowBalance);
        }

        Ok(())
    }

    // Authorise mutation operation.
    pub fn authorise_mutation1(&self, dst: &Authority<XorName>) -> Result<(), Error> {
        let dst_name = match *dst {
            Authority::ClientManager(name) => name,
            x => {
                debug!("Unexpected authority for mutation: {:?}", x);
                return Err(Error::InvalidOperation);
            }
        };

        let account = match self.get_account(&dst_name) {
            Some(account) => account,
            None => {
                debug!("Account not found for {:?}", dst);
                return Err(Error::NoSuchAccount);
            }
        };
        // TODO: Check if we are the owner or app once account keys are changed to threshold_crypto

        let unlimited_mut = unlimited_muts(&self.config);
        if !unlimited_mut && account.account_info().mutations_available == 0 {
            return Err(Error::LowBalance);
        }

        Ok(())
    }

    // Commit a mutation.
    pub fn commit_mutation(&mut self, dst: &Authority<XorName>) {
        {
            let account = unwrap!(self.get_account_mut(&dst.name()));
            account.increment_mutations_counter();
        }
    }

    // Check if data with the given name is in the storage.
    pub fn contains_data(&self, name: &DataId) -> bool {
        self.cache.nae_manager.contains_key(name)
    }

    // Load data with the given name from the storage.
    pub fn get_data(&self, name: &DataId) -> Option<Data> {
        self.cache.nae_manager.get(name).cloned()
    }

    // Save the data to the storage.
    pub fn insert_data(&mut self, name: DataId, data: Data) {
        let _ = self.cache.nae_manager.insert(name, data);
    }

    // Delete the data from the storage.
    pub fn delete_data(&mut self, name: DataId) {
        let _ = self.cache.nae_manager.remove(&name);
    }

    fn transfer_coins(
        &mut self,
        src: Authority<XorName>,
        _dest: Authority<XorName>,
        destination: XorName,
        amount: Coins,
        transaction_id: u64,
        _requester: Requester,
    ) -> Result<(), Error> {
        let client_id = if let Authority::Client { client_id, .. } = src {
            client_id
        } else {
            return Err(Error::AccessDenied); // wrong authority
        };

        match self.get_account_mut(client_id.name()) {
            Some(account) => account.credit_balance(amount, transaction_id)?,
            None => return Err(Error::NoSuchAccount),
        };
        match self.get_account_mut(&destination) {
            Some(account) => account.debit_balance(amount)?,
            None => return Err(Error::NoSuchAccount),
        };
        Ok(())
    }

    fn get_transaction(
        &self,
        src: Authority<XorName>,
        coins_balance_id: &XorName,
        transaction_id: u64,
        _requester: Requester,
    ) -> Result<Transaction, Error> {
        let client_id = if let Authority::Client { client_id, .. } = src {
            client_id
        } else {
            return Err(Error::AccessDenied); // wrong authority
        };

        // Check if we're the owner of the account
        if coins_balance_id != client_id.name() {
            return Err(Error::AccessDenied);
        }

        let account = match self.get_account(coins_balance_id) {
            Some(account) => account,
            None => return Ok(Transaction::NoSuchCoinBalance),
        };

        match account.find_transaction(transaction_id) {
            Some(amount) => Ok(Transaction::Success(amount)),
            None => Ok(Transaction::NoSuchTransaction),
        }
    }

    fn get_balance(
        &self,
        src: Authority<XorName>,
        coins_balance_id: &XorName,
        requester: Requester,
    ) -> Result<Coins, Error> {
        let client_id = if let Authority::Client { client_id, .. } = src {
            client_id
        } else {
            return Err(Error::AccessDenied); // wrong authority
        };

        // Check if we're the owner of the account
        match requester {
            Requester::Key(_pk) => {}
            Requester::Owner(_sig) => {
                // TODO: verify owner signature

                if coins_balance_id != client_id.name() {
                    return Err(Error::AccessDenied);
                }
            }
        }

        let account = match self.get_account(coins_balance_id) {
            Some(account) => account,
            None => return Err(Error::NoSuchAccount),
        };

        Ok(account.balance())
    }

    pub fn process_request(
        &mut self,
        src: Authority<XorName>,
        dest: Authority<XorName>,
        payload: Vec<u8>,
    ) -> Result<(Authority<XorName>, Vec<u8>), Error> {
        let (request, message_id, requester) = if let Message::Request {
            request,
            message_id,
            requester,
        } = unwrap!(deserialise(&payload))
        {
            (request, message_id, requester)
        } else {
            return Err(Error::from("Unexpected Message type"));
        };

        let response = match request {
            Request::TransferCoins {
                destination,
                amount,
                transaction_id,
            } => {
                let res = self.transfer_coins(
                    src,
                    dest,
                    XorName::from_new(destination),
                    amount,
                    transaction_id,
                    requester,
                );
                Response::TransferCoins(res)
            }
            Request::GetBalance { coins_balance_id } => {
                self.authorise_coin_operation(&dest, &requester)?;
                let res = self.get_balance(src, &XorName::from_new(coins_balance_id), requester);
                Response::GetBalance(res)
            }
            Request::GetTransaction {
                coins_balance_id,
                transaction_id,
            } => {
                let transaction = self.get_transaction(
                    src,
                    &XorName::from_new(coins_balance_id),
                    transaction_id,
                    requester,
                );
                Response::GetTransaction(transaction)
            }
            Request::PutUnseqMData { data } => {
                let result =
                    self.put_mdata(dest, MutableDataKind::Unsequenced(data.clone()), requester);
                Response::PutUnseqMData(result)
            }
            Request::GetSeqMData { address } => {
                let result = self
                    .get_mdata(
                        dest,
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        Some(true),
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Sequenced(mdata) => Ok(mdata),
                        _ => Err(Error::from("Unexpected data")),
                    });
                Response::GetSeqMData(result)
            }
            Request::GetUnseqMData { address } => {
                let result = self
                    .get_mdata(
                        dest,
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        Some(false),
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Unsequenced(mdata) => Ok(mdata),
                        _ => Err(Error::from("Unexpected data")),
                    });
                Response::GetUnseqMData(result)
            }
            Request::PutSeqMData { data } => {
                let result =
                    self.put_mdata(dest, MutableDataKind::Sequenced(data.clone()), requester);
                Response::PutSeqMData(result)
            }
            Request::GetSeqMDataValue { address, ref key } => {
                let result = self
                    .get_mdata(
                        dest,
                        address,
                        requester.clone(),
                        request.clone(),
                        message_id,
                        Some(true),
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Sequenced(mdata) => Ok(mdata.get(&key).unwrap().clone()),
                        _ => Err(Error::from("Unexpected data returned")),
                    });
                Response::GetSeqMDataValue(result)
            }
            Request::GetUnseqMDataValue { address, ref key } => {
                let result = self
                    .get_mdata(
                        dest,
                        address,
                        requester.clone(),
                        request.clone(),
                        message_id,
                        Some(false),
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Unsequenced(mdata) => Ok(mdata.get(&key).unwrap().clone()),
                        _ => Err(Error::from("Unexpected data returned")),
                    });
                Response::GetUnseqMDataValue(result)
            }
            Request::GetSeqMDataShell { address } => {
                let result = self
                    .get_mdata(
                        dest,
                        address,
                        requester.clone(),
                        request,
                        message_id,
                        Some(true),
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Sequenced(mdata) => Ok(mdata.shell()),
                        _ => Err(Error::from("Unexpected data returned")),
                    });
                Response::GetSeqMDataShell(result)
            }
            Request::GetUnseqMDataShell { address } => {
                let result = self
                    .get_mdata(
                        dest,
                        address,
                        requester.clone(),
                        request,
                        message_id,
                        Some(false),
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Unsequenced(mdata) => Ok(mdata.shell()),
                        _ => Err(Error::from("Unexpected data returned")),
                    });
                Response::GetUnseqMDataShell(result)
            }
            Request::GetMDataVersion { address } => {
                let result = self
                    .get_mdata(dest, address, requester.clone(), request, message_id, None)
                    .and_then(|data| match data {
                        MutableDataKind::Sequenced(mdata) => Ok(mdata.version()),
                        MutableDataKind::Unsequenced(mdata) => Ok(mdata.version()),
                    });
                Response::GetMDataVersion(result)
            }
            Request::ListUnseqMDataEntries { address } => {
                let result = self
                    .get_mdata(
                        dest,
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        Some(false),
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Unsequenced(mdata) => Ok(mdata.entries().clone()),
                        _ => Err(Error::from("Unexpected data returned")),
                    });
                Response::ListUnseqMDataEntries(result)
            }
            Request::ListSeqMDataEntries { address } => {
                let result = self
                    .get_mdata(
                        dest,
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        Some(true),
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Sequenced(mdata) => Ok(mdata.entries().clone()),
                        _ => Err(Error::from("Unexpected data returned")),
                    });
                Response::ListSeqMDataEntries(result)
            }
            Request::ListMDataKeys { address } => {
                let result = self
                    .get_mdata(
                        dest,
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        None,
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Sequenced(mdata) => Ok(mdata.keys().clone()),
                        MutableDataKind::Unsequenced(mdata) => Ok(mdata.keys().clone()),
                    });
                Response::ListMDataKeys(result)
            }
            Request::ListSeqMDataValues { address } => {
                let result = self
                    .get_mdata(
                        dest,
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        Some(true),
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Sequenced(mdata) => Ok(mdata.values()),
                        _ => Err(Error::from("Unexpected data returned")),
                    });
                Response::ListSeqMDataValues(result)
            }
            Request::ListUnseqMDataValues { address } => {
                let result = self
                    .get_mdata(
                        dest,
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        Some(false),
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Unsequenced(mdata) => Ok(mdata.values()),
                        _ => Err(Error::from("Unexpected data returned")),
                    });
                Response::ListUnseqMDataValues(result)
            }
            Request::DeleteMData { address } => {
                // let res = self.delete_mdata(dest, address.clone(), requester);
                let res = self.authorise_mutation1(&dest).and_then(|_| {
                    self.get_mdata(
                        dest,
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        None,
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Sequenced(mdata) => {
                            self.delete_data(DataId::mutable(
                                XorName::from_new(*mdata.name()),
                                mdata.tag(),
                            ));
                            self.commit_mutation(&dest);
                            Ok(())
                        }
                        MutableDataKind::Unsequenced(mdata) => {
                            self.delete_data(DataId::mutable(
                                XorName::from_new(*mdata.name()),
                                mdata.tag(),
                            ));
                            self.commit_mutation(&dest);
                            Ok(())
                        }
                    })
                });
                Response::DeleteMData(res)
            }
            Request::SetMDataUserPermissions {
                address,
                ref user,
                ref permissions,
                version,
            } => {
                let permissions = permissions.clone();
                let user = user.clone();

                let result = self
                    .get_mdata(
                        Authority::NaeManager(XorName::from_new(address.clone().name())),
                        address.clone(),
                        requester.clone(),
                        request.clone(),
                        message_id,
                        None,
                    )
                    .and_then(|data| {
                        let data_name = DataId::mutable(data.name(), data.tag());
                        match data.clone() {
                            MutableDataKind::Unsequenced(mut mdata) => {
                                unwrap!(mdata.set_user_permissions(user, permissions, version));
                                self.insert_data(
                                    data_name,
                                    Data::NewMutable(MutableDataKind::Unsequenced(mdata)),
                                );
                                self.commit_mutation(&dest);
                                Ok(())
                            }
                            MutableDataKind::Sequenced(mut mdata) => {
                                unwrap!(mdata.set_user_permissions(user, permissions, version));
                                self.insert_data(
                                    data_name,
                                    Data::NewMutable(MutableDataKind::Sequenced(mdata)),
                                );
                                self.commit_mutation(&dest);
                                Ok(())
                            }
                        }
                    });
                Response::SetMDataUserPermissions(result)
            }
            Request::DelMDataUserPermissions {
                address,
                ref user,
                version,
            } => {
                let user = user.clone();

                let result = self
                    .get_mdata(
                        Authority::NaeManager(XorName::from_new(address.clone().name())),
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        None,
                    )
                    .and_then(|data| {
                        let data_name = DataId::mutable(data.name(), data.tag());
                        match data.clone() {
                            MutableDataKind::Unsequenced(mut mdata) => {
                                unwrap!(mdata.del_user_permissions(user, version));
                                self.insert_data(
                                    data_name,
                                    Data::NewMutable(MutableDataKind::Unsequenced(mdata)),
                                );
                                self.commit_mutation(&dest);
                                Ok(())
                            }
                            MutableDataKind::Sequenced(mut mdata) => {
                                unwrap!(mdata.del_user_permissions(user, version));
                                self.insert_data(
                                    data_name,
                                    Data::NewMutable(MutableDataKind::Sequenced(mdata)),
                                );
                                self.commit_mutation(&dest);
                                Ok(())
                            }
                        }
                    });
                Response::DelMDataUserPermissions(result)
            }
            Request::ListMDataUserPermissions { address, ref user } => {
                let user = user.clone();

                let result = self
                    .get_mdata(
                        dest,
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        None,
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Unsequenced(mdata) => {
                            Ok((*unwrap!(mdata.user_permissions(user))).clone())
                        }
                        MutableDataKind::Sequenced(mdata) => {
                            Ok((*unwrap!(mdata.user_permissions(user))).clone())
                        }
                    });
                Response::ListMDataUserPermissions(result)
            }
            Request::ListMDataPermissions { address } => {
                let result = self
                    .get_mdata(
                        dest,
                        address.clone(),
                        requester.clone(),
                        request,
                        message_id,
                        None,
                    )
                    .and_then(|data| match data {
                        MutableDataKind::Unsequenced(mdata) => Ok(mdata.permissions()),
                        MutableDataKind::Sequenced(mdata) => Ok(mdata.permissions()),
                    });
                Response::ListMDataPermissions(result)
            }
            Request::MutateSeqMDataEntries {
                address,
                ref actions,
            } => {
                let request = request.clone();

                let result = self
                    .get_mdata(
                        Authority::NaeManager(XorName::from_new(address.clone().name())),
                        address.clone(),
                        requester.clone(),
                        request.clone(),
                        message_id,
                        Some(true),
                    )
                    .and_then(move |data| {
                        let data_name = DataId::mutable(data.name(), data.tag());
                        match data.clone() {
                            MutableDataKind::Sequenced(mut mdata) => {
                                unwrap!(mdata.mutate_entries(
                                    actions.clone(),
                                    request,
                                    requester,
                                    message_id
                                ));
                                self.insert_data(
                                    data_name,
                                    Data::NewMutable(MutableDataKind::Sequenced(mdata)),
                                );
                                self.commit_mutation(&dest);
                                Ok(())
                            }
                            _ => Err(Error::from("Unexpected data returned")),
                        }
                    });
                Response::MutateSeqMDataEntries(result)
            }
            Request::MutateUnseqMDataEntries {
                address,
                ref actions,
            } => {
                let request = request.clone();
                let actions = actions.clone();

                let result = self
                    .get_mdata(
                        Authority::NaeManager(XorName::from_new(address.clone().name())),
                        address.clone(),
                        requester.clone(),
                        request.clone(),
                        message_id,
                        Some(false),
                    )
                    .and_then(move |data| {
                        let data_name = DataId::mutable(data.name(), data.tag());
                        match data.clone() {
                            MutableDataKind::Unsequenced(mut mdata) => {
                                unwrap!(mdata.mutate_entries(
                                    actions.clone(),
                                    request,
                                    requester,
                                    message_id
                                ));
                                self.insert_data(
                                    data_name,
                                    Data::NewMutable(MutableDataKind::Unsequenced(mdata)),
                                );
                                self.commit_mutation(&dest);
                                Ok(())
                            }
                            _ => Err(Error::from("Unexpected data returned")),
                        }
                    });
                Response::MutateUnseqMDataEntries(result)
            }
            _ => {
                // Dummy return
                // other requests to be handled by their data type impls
                return Ok((dest, payload));
            }
        };

        Ok((
            dest,
            unwrap!(serialise(&Message::Response {
                response,
                message_id,
            })),
        ))
    }

    pub fn get_mdata(
        &mut self,
        _dst: Authority<XorName>,
        address: MutableDataRef,
        requester: Requester,
        request: Request,
        msg_id: MessageId,
        sequenced: Option<bool>,
    ) -> Result<MutableDataKind, Error> {
        let data_name = DataId::mutable(XorName::from_new(address.name()), address.tag());
        // self.authorise_read(&dst, &XorName::from_new(address.name()))
        // .map_err(|err| Error::from(err.description()))
        // .and_then(|_| match self.get_data(&data_name) {
        dbg!(request.clone());
        match self.get_data(&data_name) {
            Some(data_type) => match data_type {
                Data::NewMutable(data) => match data.clone() {
                    MutableDataKind::Sequenced(mdata) => {
                        if sequenced.is_some() && !unwrap!(sequenced) {
                            Err(Error::from("Unexpected data returned"))
                        } else if mdata.check_permissions(request, requester, msg_id).is_err() {
                            Err(Error::AccessDenied)
                        } else {
                            Ok(data)
                        }
                    }
                    MutableDataKind::Unsequenced(mdata) => {
                        if sequenced.is_some() && unwrap!(sequenced) {
                            Err(Error::from("Unexpected data returned"))
                        } else if mdata.check_permissions(request, requester, msg_id).is_err() {
                            Err(Error::AccessDenied)
                        } else {
                            Ok(data)
                        }
                    }
                },
                _ => Err(Error::NoSuchData),
            },
            None => Err(Error::NoSuchData),
        }
        // })
    }

    pub fn put_mdata(
        &mut self,
        dst: Authority<XorName>,
        data: MutableDataKind,
        _requester: Requester,
    ) -> Result<(), Error> {
        let data_name = DataId::mutable(data.name(), data.tag());
        self.authorise_mutation1(&dst)
            .and_then(|_| {
                if self.contains_data(&data_name) {
                    Err(Error::DataExists)
                } else {
                    self.insert_data(data_name, Data::NewMutable(data));
                    Ok(())
                }
            })
            .map(|_| self.commit_mutation(&dst))
    }
}

pub struct VaultGuard<'a>(MutexGuard<'a, Vault>);

impl<'a> Deref for VaultGuard<'a> {
    type Target = Vault;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a> DerefMut for VaultGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

impl<'a> Drop for VaultGuard<'a> {
    fn drop(&mut self) {
        let vault = &mut *self.0;
        vault.store.save(&vault.cache)
    }
}

pub fn lock(vault: &Mutex<Vault>, writing: bool) -> VaultGuard {
    let mut inner = unwrap!(vault.lock());

    if let Some(cache) = inner.store.load(writing) {
        inner.cache = cache;
    }

    VaultGuard(inner)
}

#[derive(Deserialize, Serialize)]
struct Cache {
    client_manager: HashMap<XorName, Account>,
    nae_manager: HashMap<DataId, Data>,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum Data {
    Immutable(ImmutableData),
    OldMutable(OldMutableData),
    NewMutable(MutableDataKind),
}

#[derive(Clone, Deserialize, Serialize)]
pub enum MutableDataKind {
    Sequenced(SeqMutableData),
    Unsequenced(UnseqMutableData),
}

impl MutableDataKind {
    fn name(&self) -> XorName {
        match self {
            MutableDataKind::Sequenced(data) => XorName::from_new(*data.name()),
            MutableDataKind::Unsequenced(data) => XorName::from_new(*data.name()),
        }
    }
    fn tag(&self) -> u64 {
        match self {
            MutableDataKind::Sequenced(data) => data.tag(),
            MutableDataKind::Unsequenced(data) => data.tag(),
        }
    }
}

trait Store: Send {
    fn load(&mut self, writing: bool) -> Option<Cache>;
    fn save(&mut self, cache: &Cache);
}

struct MemoryStore;

impl Store for MemoryStore {
    fn load(&mut self, _: bool) -> Option<Cache> {
        None
    }

    fn save(&mut self, _: &Cache) {}
}

struct FileStore {
    // `bool` element indicates whether the store is being written to.
    file: Option<(File, bool)>,
    sync_time: Option<SystemTime>,
    path: PathBuf,
}

impl FileStore {
    fn new(path: &PathBuf) -> Self {
        FileStore {
            file: None,
            sync_time: None,
            path: path.join(FILE_NAME),
        }
    }
}

impl Store for FileStore {
    fn load(&mut self, writing: bool) -> Option<Cache> {
        // Create the file if it doesn't exist yet.
        let mut file = unwrap!(OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.path));

        if writing {
            unwrap!(file.lock_exclusive());
        } else {
            unwrap!(file.lock_shared());
        };

        let metadata = unwrap!(file.metadata());
        let mtime = unwrap!(metadata.modified());
        let mtime_duration = if let Some(sync_time) = self.sync_time {
            mtime
                .duration_since(sync_time)
                .unwrap_or_else(|_| Duration::from_millis(0))
        } else {
            Duration::from_millis(1)
        };

        // Update vault only if it's not already synchronised
        let mut result = None;
        if mtime_duration > Duration::new(0, 0) {
            let mut raw_data = Vec::with_capacity(metadata.len() as usize);
            match file.read_to_end(&mut raw_data) {
                Ok(0) => (),
                Ok(_) => match deserialise::<Cache>(&raw_data) {
                    Ok(cache) => {
                        self.sync_time = Some(mtime);
                        result = Some(cache);
                    }
                    Err(e) => {
                        warn!("Can't read the mock vault: {:?}", e);
                    }
                },
                Err(e) => {
                    warn!("Can't read the mock vault: {:?}", e);
                    return None;
                }
            }
        }

        self.file = Some((file, writing));

        result
    }

    fn save(&mut self, cache: &Cache) {
        // Write the data to the storage file (if in write mode) and remove
        // the lock.
        if let Some((mut file, writing)) = self.file.take() {
            if writing {
                let raw_data = unwrap!(serialise(&cache));
                unwrap!(file.set_len(0));
                let _ = unwrap!(file.seek(SeekFrom::Start(0)));
                unwrap!(file.write_all(&raw_data));
                unwrap!(file.sync_all());

                let mtime = unwrap!(unwrap!(file.metadata()).modified());
                self.sync_time = Some(mtime);
            }

            let _ = file.unlock();
        }
    }
}

/// Path to the mock vault store file.
pub fn mock_vault_path(config: &Config) -> PathBuf {
    init_vault_path(config.dev.as_ref()).join(FILE_NAME)
}
