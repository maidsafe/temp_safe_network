// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::super::{utils, Error, PublicKey, Result, Signature};
use super::metadata::Entries;
use super::metadata::{Address, Entry, Index, Perm};
use crdts::{
    list::{List, Op},
    CmRDT,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Debug, Display},
    hash::Hash,
};

/// CRDT Data operation applicable to other Sequence replica.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrdtOperation<A: Ord, T> {
    /// Address of a Sequence object on the network.
    pub address: Address,
    /// The data operation to apply.
    pub crdt_op: Op<T, A>,
    /// The PublicKey of the entity that generated the operation
    pub source: PublicKey,
    /// The signature of source on the crdt_top, required to apply the op
    pub signature: Option<Signature>,
}

/// Sequence data type as a CRDT with Access Control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SequenceCrdt<A: Ord, P> {
    /// Actor of this piece of data
    pub(crate) actor: A,
    /// Address on the network of this piece of data
    address: Address,
    /// CRDT to store the actual data, i.e. the items of the Sequence.
    data: List<Entry, A>,
    /// The Policy matrix containing ownership and users permissions.
    policy: P,
}

impl<A, P> Display for SequenceCrdt<A, P>
where
    A: Ord + Clone + Display + Debug + Serialize,
    P: Perm + Hash + Clone + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, entry) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "<{}>", String::from_utf8_lossy(&entry),)?;
        }
        write!(f, "]")
    }
}

impl<A, P> SequenceCrdt<A, P>
where
    A: Ord + Clone + Debug + Serialize,
    P: Serialize,
{
    /// Constructs a new 'SequenceCrdt'.
    pub fn new(actor: A, address: Address, policy: P) -> Self {
        Self {
            actor,
            address,
            data: List::new(),
            policy,
        }
    }

    /// Returns the address.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the length of the sequence.
    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }

    /// Create crdt op to append a new item to the SequenceCrdt
    pub fn create_append_op(
        &self,
        entry: Entry,
        source: PublicKey,
    ) -> Result<CrdtOperation<A, Entry>> {
        let address = *self.address();

        // Append the entry to the LSeq
        let crdt_op = self.data.append(entry, self.actor.clone());

        // We return the operation as it may need to be broadcasted to other replicas
        Ok(CrdtOperation {
            address,
            crdt_op,
            source,
            signature: None,
        })
    }

    /// Apply a remote data CRDT operation to this replica of the Sequence.
    pub fn apply_op(&mut self, op: CrdtOperation<A, Entry>) -> Result<()> {
        // Let's first check the op is validly signed.
        // Note: Perms for the op are checked at the upper Sequence layer.

        let sig = op.signature.ok_or(Error::CrdtMissingOpSignature)?;
        let bytes_to_verify = utils::serialise(&op.crdt_op).map_err(|err| {
            Error::Serialisation(format!(
                "Could not serialise CRDT operation to verify signature: {}",
                err
            ))
        })?;
        op.source.verify(&sig, &bytes_to_verify)?;

        // Apply the CRDT operation to the LSeq data
        self.data.apply(op.crdt_op);

        Ok(())
    }

    /// Gets the entry at `index` if it exists.
    pub fn get(&self, index: Index) -> Option<&Entry> {
        let i = to_absolute_index(index, self.len() as usize)?;
        self.data.position(i)
    }

    /// Gets the last entry.
    pub fn last_entry(&self) -> Option<&Entry> {
        self.data.last()
    }

    /// Gets the Policy of the object.
    pub fn policy(&self) -> &P {
        &self.policy
    }

    /// Gets a list of items which are within the given indices.
    /// Note the range of items is [start, end), i.e. the end index is not inclusive.
    pub fn in_range(&self, start: Index, end: Index) -> Option<Entries> {
        let count = self.len() as usize;
        let start_index = to_absolute_index(start, count)?;
        if start_index >= count {
            return None;
        }
        let end_index = to_absolute_index(end, count)?;
        let items_to_take = end_index - start_index;

        let entries = self
            .data
            .iter()
            .skip(start_index)
            .take(items_to_take)
            .cloned()
            .collect::<Entries>();

        Some(entries)
    }
}

// Private helpers

fn to_absolute_index(index: Index, count: usize) -> Option<usize> {
    match index {
        Index::FromStart(index) if (index as usize) <= count => Some(index as usize),
        Index::FromStart(_) => None,
        Index::FromEnd(index) => count.checked_sub(index as usize),
    }
}
