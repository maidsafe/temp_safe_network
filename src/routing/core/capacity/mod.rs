// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod adult_storage_info;

use crate::routing::{Prefix, XorName};
pub(super) use adult_storage_info::AdultsStorageInfo;
use std::collections::BTreeSet;

// The number of separate copies of a chunk which should be maintained.
pub(crate) const CHUNK_COPY_COUNT: usize = 4;

/// A util for sharing the
/// info on data capacity among the
/// chunk storing nodes in the section.
#[derive(Clone)]
pub(crate) struct Capacity {
    reader: CapacityReader,
    writer: CapacityWriter,
}

impl Capacity {
    /// Pass in adult_storage_info with info on chunk holders.
    pub(super) fn new(reader: CapacityReader, writer: CapacityWriter) -> Self {
        Self { reader, writer }
    }

    /// Whether the adult is recorded as full
    pub(super) async fn is_full(&self, adult: XorName) -> bool {
        self.reader.is_full(&adult).await
    }

    /// Number of full chunk storing nodes in the section.
    pub(super) async fn full_adults_count(&self) -> u8 {
        self.reader.full_adults_count().await
    }

    /// Full chunk storing nodes in the section.
    pub(super) async fn full_adults(&self) -> BTreeSet<XorName> {
        self.reader.full_adults().await
    }

    /// Number of full chunk storing nodes in the section.
    pub(super) async fn full_adults_matching(&self, prefix: Prefix) -> BTreeSet<XorName> {
        self.reader.full_adults_matching(prefix).await
    }

    pub(super) async fn insert_full_adults(&self, full_adults: BTreeSet<XorName>) {
        self.writer.insert_full_adults(full_adults).await
    }

    pub(super) async fn remove_full_adults(&self, full_adults: BTreeSet<XorName>) {
        self.writer.remove_full_adults(full_adults).await
    }

    /// Registered holders not present in provided list of members
    /// will be removed from adult_storage_info and no longer tracked for liveness.
    pub(super) async fn retain_members_only(&self, members: &BTreeSet<XorName>) {
        self.writer.retain_members_only(members).await
    }
}

#[derive(Clone)]
pub(super) struct CapacityReader {
    // reader: AdultReader,
    adult_storage_info: AdultsStorageInfo,
}

#[derive(Clone)]
pub(super) struct CapacityWriter {
    // reader: AdultReader,
    adult_storage_info: AdultsStorageInfo,
}

impl CapacityReader {
    /// Pass in adult_storage_info with info on chunk holders.
    pub(super) fn new(adult_storage_info: AdultsStorageInfo, //, reader: AdultReader
    ) -> Self {
        Self {
            // reader,
            adult_storage_info,
        }
    }

    /// Whether the adult is recorded as full
    pub(super) async fn is_full(&self, adult: &XorName) -> bool {
        self.adult_storage_info
            .full_adults
            .read()
            .await
            .contains(adult)
    }

    /// Number of full chunk storing nodes in the section.
    pub(super) async fn full_adults_count(&self) -> u8 {
        self.adult_storage_info.full_adults.read().await.len() as u8
    }

    /// Number of full chunk storing nodes in the section.
    pub(super) async fn full_adults_matching(&self, prefix: Prefix) -> BTreeSet<XorName> {
        self.adult_storage_info
            .full_adults
            .read()
            .await
            .iter()
            .filter(|name| prefix.matches(name))
            .copied()
            .collect()
    }

    /// Get full adults.
    pub(super) async fn full_adults(&self) -> BTreeSet<XorName> {
        self.adult_storage_info.full_adults.read().await.clone()
    }
}

impl CapacityWriter {
    /// Pass in adult_storage_info with info on chunk holders.
    pub(super) fn new(adult_storage_info: AdultsStorageInfo) -> Self {
        Self { adult_storage_info }
    }

    pub(super) async fn insert_full_adults(&self, full_adults: BTreeSet<XorName>) {
        let mut orig_full_adults = self.adult_storage_info.full_adults.write().await;

        for adult in full_adults {
            let _ = orig_full_adults.insert(adult);
        }
    }

    pub(super) async fn remove_full_adults(&self, full_adults: BTreeSet<XorName>) {
        let mut orig_full_adults = self.adult_storage_info.full_adults.write().await;

        for adult in full_adults {
            let _ = orig_full_adults.remove(&adult);
        }
    }

    /// Registered holders not present in provided list of members
    /// will be removed from adult_storage_info and no longer tracked for liveness.
    pub(super) async fn retain_members_only(&self, members: &BTreeSet<XorName>) {
        // full adults
        let mut full_adults = self.adult_storage_info.full_adults.write().await;
        let absent_adults = full_adults
            .iter()
            .filter(|&key| !members.contains(key))
            .cloned()
            .collect::<Vec<_>>();

        for adult in &absent_adults {
            let _ = full_adults.remove(adult);
        }
    }
}
