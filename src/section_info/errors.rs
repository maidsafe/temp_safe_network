// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::section_info::SectionInfo;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Error {
    /// Target section key provided with message is out of date. Current PK Set is provided in error
    #[error("Target section's public key is outdated. New information has been provided.")]
    TargetSectionInfoOutdated(SectionInfo),
    /// Target section is undergoing churn, a new key set will be agreed upon shortly
    #[error("DKG is in process. New key set will be agreed upon shortly.")]
    DkgInProgress,
    /// Target section is unrecognized
    #[error("Target section key provided is unrecognized")]
    UnrecognizedSectionKey,
    /// No PublicKeySet found at this section
    #[error("No PublicKey found at this section")]
    NoSectionPkSet,
    /// Invalid data in the bootstrap cmd
    #[error("Invalid data in the bootstrap cmd")]
    InvalidBootstrap(String),
}
