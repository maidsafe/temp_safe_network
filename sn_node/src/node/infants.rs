// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_interface::{
    messaging::system::{
        InfantJoinRejectionReason as RejectionReason, InfantJoinResponse as Response,
    },
    types::Peer,
};
use std::collections::BTreeSet;
use xor_name::XorName;

const MAX_NUM_INFANTS: usize = 200;
const MIN_INFANT_AGE: u8 = 5;

/// The handling of the infant nodes,
/// i.e. the newly joined nodes in the network.
///
/// NB: This is a side-by-side impl and
/// does not interfere with current join logic.
#[derive(Clone, Debug)]
pub(crate) struct Infants {
    set: BTreeSet<Peer>,
}

impl Infants {
    /// Returns a new empty instance of `Infants`.
    pub(super) fn new() -> Self {
        Self {
            set: BTreeSet::new(),
        }
    }

    /// Adds an infant.
    pub(super) fn add(&mut self, infant: Peer) -> Response {
        if self.is_full() {
            return Response::Rejected(RejectionReason::Full);
        } else if infant.age() != MIN_INFANT_AGE {
            return Response::Rejected(RejectionReason::InvalidAge {
                provided: infant.age(),
                expected: MIN_INFANT_AGE,
            });
        }

        let _ = self.set.insert(infant);

        Response::Approved
    }

    /// Removes an infant.
    pub(super) fn remove(&mut self, name: XorName) {
        self.set.retain(|peer| peer.name() != name);
    }

    /// Returns if we've already got the max number of infants.
    pub(super) fn is_full(&self) -> bool {
        self.set.len() >= MAX_NUM_INFANTS
    }

    /// Returns if the name exists among our infants.
    pub(super) fn exists(&self, name: XorName) -> bool {
        self.set.iter().any(|peer| peer.name() == name)
    }
}
