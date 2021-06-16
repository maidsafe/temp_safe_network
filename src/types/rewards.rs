// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{CreditId, SignatureShare, SignedCredit, SignedCreditShare};
use serde::{Deserialize, Serialize};

/// Node age, the number of times
/// it has been relocated between sections.
pub type NodeAge = u8;

/// Proposed credits resulting from a churn.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct RewardProposal {
    /// The section paying out the rewards.
    pub section_key: super::PublicKey,
    /// Any proposed rewards
    pub rewards: Vec<SignedCreditShare>,
}

/// Accumulation of proof for the churn credits.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct RewardAccumulation {
    /// The section paying out the rewards.
    pub section_key: super::PublicKey,
    /// Any agreed rewards
    pub rewards: Vec<AccumulatingReward>,
}

///
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct AccumulatingReward {
    /// A credit .
    pub signed_credit: SignedCredit,
    /// An individual Elder's sig share.
    pub sig: SignatureShare,
}

impl AccumulatingReward {
    /// Returns the id of the signed credit.
    pub fn id(&self) -> &CreditId {
        self.signed_credit.id()
    }
}
