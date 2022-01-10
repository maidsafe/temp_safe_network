// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod encoding;
mod errors;
mod event_store;
mod lru_cache;
mod used_space;

pub(crate) use encoding::{deserialise, serialise};
pub(crate) use errors::{convert_to_error_message, Error, Result};
pub(crate) use event_store::EventStore;
pub(crate) use lru_cache::LruCache;
use std::path::Path;
pub use used_space::UsedSpace;

pub(crate) const SLED_FLUSH_TIME_MS: Option<u64> = Some(10000);

pub(crate) trait Subdir {
    fn subdir() -> &'static Path;
}
