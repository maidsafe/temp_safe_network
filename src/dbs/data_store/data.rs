// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::ToDbKey;
use crate::types::DataAddress;
use serde::{de::DeserializeOwned, Serialize};

pub(crate) trait Data: Serialize + DeserializeOwned {
    type Id: DataId;
    fn id(&self) -> &Self::Id;
}

pub(crate) trait DataId: ToDbKey + PartialEq + Eq + DeserializeOwned {
    fn to_data_address(&self) -> DataAddress;
}
