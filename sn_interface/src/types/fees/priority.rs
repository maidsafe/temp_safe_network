// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// Used by client to choose how fast their spend will be processed.
/// The chosen variant will map to a fee using the spend queue stats fetched from Elders.
#[derive(
    Copy, Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum SpendPriority {
    Highest,    // -> `High` + 1 std dev.
    High,       // -> The highest fee in spend queue.
    MediumHigh, // -> Avg of `High` and `Normal`
    Normal,     // -> The avg fee in spend queue.
    MediumLow,  // -> Avg of `Normal` and `Low`.
    Low,        // -> The lowest fee in spend queue.
    Lowest,     // -> `Low` - 1 std dev
}
