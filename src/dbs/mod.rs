// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod data_store;
mod encoding;
mod errors;
mod register_cmd_event_store;

use data_store::to_db_key::ToDbKey;
pub(crate) use data_store::{
    data::{Data, DataId},
    used_space::UsedSpace,
    DataStore, Subdir,
};
pub(crate) use errors::Error;
use errors::Result;
pub(crate) use register_cmd_event_store::RegisterCmdEventStore;
