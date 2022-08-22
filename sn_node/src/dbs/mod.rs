// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod errors;
mod file_store;
mod reg_op_store;
mod used_space;

pub(crate) use errors::{convert_to_error_msg, Error, Result};
pub(crate) use file_store::FileStore;
pub(crate) use reg_op_store::RegOpStore;
use std::path::Path;
pub use used_space::UsedSpace;

pub(crate) trait Subdir {
    fn subdir() -> &'static Path;
}
