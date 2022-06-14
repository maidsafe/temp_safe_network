// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

pub use super::{ContentType, DataType, SafeUrl, VersionHash, XorUrlBase};
use crate::app::{
    files::{FileInfo, FilesMap},
    multimap::Multimap,
    nrs::NrsMap,
    register::{Entry, EntryHash},
    XorName,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// SafeData contains the data types fetchable using the Safe Network resolver
#[allow(clippy::large_enum_variant)]
// FilesContainer is significantly larger than the other variants
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub enum SafeData {
    SafeKey {
        xorurl: String,
        xorname: XorName,
        resolved_from: String,
    },
    FilesContainer {
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        version: Option<VersionHash>, // this is set to None if the container is found empty
        files_map: FilesMap,
        data_type: DataType,
        metadata: Option<FileInfo>,
        resolves_into: Option<SafeUrl>,
        resolved_from: String,
    },
    PublicFile {
        xorurl: String,
        xorname: XorName,
        data: Bytes,
        media_type: Option<String>,
        metadata: Option<FileInfo>,
        resolved_from: String,
    },
    NrsMapContainer {
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        nrs_map: NrsMap,
        data_type: DataType,
    },
    /// The xorurl and data_type are those the target entry points to.
    NrsEntry {
        xorurl: String,
        public_name: String,
        data_type: DataType,
        resolves_into: SafeUrl,
        resolved_from: String,
        version: Option<EntryHash>,
    },
    Multimap {
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        data: Multimap,
        resolved_from: String,
    },
    Register {
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        data: BTreeSet<(EntryHash, Entry)>,
        resolved_from: String,
    },
}

impl SafeData {
    pub fn xorurl(&self) -> String {
        use SafeData::*;
        match self {
            SafeKey { xorurl, .. }
            | FilesContainer { xorurl, .. }
            | PublicFile { xorurl, .. }
            | NrsMapContainer { xorurl, .. }
            | NrsEntry { xorurl, .. }
            | Multimap { xorurl, .. }
            | Register { xorurl, .. } => xorurl.clone(),
        }
    }

    pub fn resolved_from(&self) -> Option<String> {
        use SafeData::*;
        match self {
            SafeKey { resolved_from, .. }
            | FilesContainer { resolved_from, .. }
            | PublicFile { resolved_from, .. }
            | NrsEntry { resolved_from, .. }
            | Multimap { resolved_from, .. }
            | Register { resolved_from, .. } => Some(resolved_from.clone()),
            NrsMapContainer { .. } => None,
        }
    }

    pub fn resolves_into(&self) -> Option<SafeUrl> {
        use SafeData::*;
        match self {
            SafeKey { .. }
            | Multimap { .. }
            | NrsMapContainer { .. }
            | PublicFile { .. }
            | Register { .. } => None,
            FilesContainer { resolves_into, .. } => resolves_into.clone(),
            NrsEntry { resolves_into, .. } => Some(resolves_into.clone()),
        }
    }

    pub fn metadata(&self) -> Option<FileInfo> {
        use SafeData::*;
        match self {
            SafeKey { .. }
            | Multimap { .. }
            | Register { .. }
            | NrsMapContainer { .. }
            | NrsEntry { .. } => None,
            FilesContainer { metadata, .. } | PublicFile { metadata, .. } => metadata.clone(),
        }
    }
}
