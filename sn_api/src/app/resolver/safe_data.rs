// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

pub use super::{ContentType, DataType, SafeUrl, VersionHash, XorUrlBase};
use crate::app::{
    files::{FileInfo, FilesMap},
    multimap::MultimapKeyValues,
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
        public_name: Option<String>,
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        version: VersionHash,
        nrs_map: NrsMap,
        data_type: DataType,
        resolves_into: Option<SafeUrl>,
        resolved_from: String,
    },
    Multimap {
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        data: MultimapKeyValues,
        resolved_from: String,
    },
    PublicRegister {
        xorurl: String,
        xorname: XorName,
        type_tag: u64,
        data: BTreeSet<(EntryHash, Entry)>,
        resolved_from: String,
    },
    PrivateRegister {
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
            | Multimap { xorurl, .. }
            | PublicRegister { xorurl, .. }
            | PrivateRegister { xorurl, .. } => xorurl.clone(),
        }
    }

    pub fn resolved_from(&self) -> String {
        use SafeData::*;
        match self {
            SafeKey { resolved_from, .. }
            | FilesContainer { resolved_from, .. }
            | PublicFile { resolved_from, .. }
            | NrsMapContainer { resolved_from, .. }
            | Multimap { resolved_from, .. }
            | PublicRegister { resolved_from, .. }
            | PrivateRegister { resolved_from, .. } => resolved_from.clone(),
        }
    }

    pub fn resolves_into(&self) -> Option<SafeUrl> {
        use SafeData::*;
        match self {
            SafeKey { .. }
            | PublicFile { .. }
            | Multimap { .. }
            | PublicRegister { .. }
            | PrivateRegister { .. } => None,
            FilesContainer { resolves_into, .. } | NrsMapContainer { resolves_into, .. } => {
                resolves_into.clone()
            }
        }
    }

    pub fn metadata(&self) -> Option<FileInfo> {
        use SafeData::*;
        match self {
            SafeKey { .. }
            | Multimap { .. }
            | PublicRegister { .. }
            | PrivateRegister { .. }
            | NrsMapContainer { .. } => None,
            FilesContainer { metadata, .. } | PublicFile { metadata, .. } => metadata.clone(),
        }
    }
}
