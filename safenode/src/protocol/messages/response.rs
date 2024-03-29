// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{
    node::NodeId,
    protocol::{
        chunk::Chunk,
        error::Result,
        fees::RequiredFee,
        register::{Entry, EntryHash, Permissions, Policy, Register, User},
    },
};

#[allow(unused_imports)] // needed by rustdocs links
use crate::protocol::messages::RegisterQuery;

use sn_dbc::SignedSpend;

use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt::Debug};

/// The response to a query, containing the query result.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryResponse {
    //
    // ===== DBC Data =====
    //
    /// Response to [`GetFees`].
    ///
    /// [`GetFees`]: crate::protocol::messages::SpendQuery::GetFees
    GetFees(Result<(NodeId, RequiredFee)>),
    /// If the queried node has validated a corresponding spend
    /// request, it will return the SignedSpend.
    /// It is up to the Client to get this SignedSpend from enough
    /// nodes as to consider it a valid spend. The specific rules
    /// on how many nodes are enough, are found here: (TODO).
    ///
    /// Response to [`GetDbcSpend`]
    ///
    /// [`GetDbcSpend`]: crate::protocol::messages::SpendQuery::GetDbcSpend
    GetDbcSpend(Result<SignedSpend>),
    //
    // ===== Chunk =====
    //
    /// Response to [`GetChunk`]
    ///
    /// [`GetChunk`]: crate::protocol::messages::Query::GetChunk
    GetChunk(Result<Chunk>),
    //
    // ===== Register Data =====
    //
    /// Response to [`RegisterQuery::Get`].
    GetRegister(Result<Register>),
    /// Response to [`RegisterQuery::GetEntry`].
    GetRegisterEntry(Result<Entry>),
    /// Response to [`RegisterQuery::GetOwner`].
    GetRegisterOwner(Result<User>),
    /// Response to [`RegisterQuery::Read`].
    ReadRegister(Result<BTreeSet<(EntryHash, Entry)>>),
    /// Response to [`RegisterQuery::GetPolicy`].
    GetRegisterPolicy(Result<Policy>),
    /// Response to [`RegisterQuery::GetUserPermissions`].
    GetRegisterUserPermissions(Result<Permissions>),
}

/// The response to a Cmd, containing the query result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CmdResponse {
    //
    // ===== Dbc Spends =====
    //
    /// Response to DbcCmd::Spend.
    Spend(Result<()>),
    //
    // ===== Chunk =====
    //
    /// Response to Cmd::StoreChunk
    StoreChunk(Result<()>),
    //
    // ===== Register Data =====
    //
    /// Response to RegisterCmd::Create.
    CreateRegister(Result<()>),
    /// Response to RegisterCmd::Edit.
    EditRegister(Result<()>),
}
