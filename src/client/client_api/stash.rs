// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::data::{Dbc, Stash as StashTrait};
use crate::types::Token;

/// Temporary dummy impl

#[derive(Clone, Debug)]
pub(crate) struct Stash {}

impl StashTrait for Stash {
    fn value(&self) -> Token {
        Token::from_nano(u32::MAX as u64)
    }

    fn take(&self, value: Token) -> Vec<Dbc> {
        vec![Dbc { value }]
    }
}
