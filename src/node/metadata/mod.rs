// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod adult_liveness;
pub(crate) mod adult_reader;
mod chunk_records;
mod elder_stores;
mod register_storage;

use crate::messaging::{
    data::{CmdError, DataCmd, DataExchange, DataQuery, QueryResponse, ServiceMsg},
    node::NodeMsg,
    AuthorityProof, DstLocation, EndUser, MessageId, ServiceAuth, WireMsg,
};

use crate::node::{
    capacity::Capacity,
    network::Network,
    node_ops::{MsgType, NodeDuties, NodeDuty, OutgoingMsg},
    Error, Result,
};
use crate::routing::Prefix;
use crate::types::{Chunk, Keypair, PublicKey};
use bls::PublicKey as BlsPublicKey;
use chunk_records::ChunkRecords;
use elder_stores::ElderStores;
use rand::rngs::OsRng;
pub use register_storage::RegisterStorage;
use std::{
    collections::BTreeSet,
    fmt::{self, Display, Formatter},
};
use xor_name::XorName;

/// This module is called `Metadata`
/// as a preparation for the responsibilities
/// it will have eventually, after `Data Hierarchy Refinement`
/// has been implemented; where the data types are all simply
/// the structures + their metadata - handled at `Elders` - with
/// all underlying data being chunks stored at `Adults`.
pub(crate) struct Metadata {
    elder_stores: ElderStores,
}

impl Metadata {
    pub(crate) async fn new(capacity: Capacity, network: Network) -> Result<Self> {
        let chunk_records = ChunkRecords::new(capacity);
        let elder_stores = ElderStores::new(chunk_records, network);
        Ok(Self { elder_stores })
    }

    pub(crate) async fn read(
        &self,
        query: DataQuery,
        id: MessageId,
        auth: AuthorityProof<ServiceAuth>,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        match self.elder_stores.read(query, id, auth, origin).await {
            Err(Error::NoSuchData(_)) => Ok(NodeDuty::NoOp),
            res => res,
        }
    }

    pub(crate) async fn record_adult_read_liveness(
        &self,
        correlation_id: MessageId,
        result: QueryResponse,
        src: XorName,
        network: &Network,
    ) -> Result<NodeDuties> {
        self.elder_stores
            .chunk_records()
            .record_adult_read_liveness(correlation_id, result, src, network)
            .await
    }

    pub(crate) async fn retain_members_only(&self, members: BTreeSet<XorName>) -> Result<()> {
        self.elder_stores
            .chunk_records()
            .retain_members_only(members)
            .await?;
        Ok(())
    }

    pub(crate) async fn write(
        &mut self,
        cmd: DataCmd,
        id: MessageId,
        auth: AuthorityProof<ServiceAuth>,
        origin: EndUser,
    ) -> Result<NodeDuty> {
        self.elder_stores.write(cmd, id, auth, origin).await
    }

    /// Adds a given node to the list of full nodes.
    pub(crate) async fn increase_full_node_count(&self, node_id: PublicKey) {
        self.elder_stores
            .chunk_records()
            .increase_full_node_count(node_id)
            .await
    }

    // When receiving the chunk from remaining holders, we ask new holders to store it.
    pub(crate) async fn republish_chunk(&self, chunk: Chunk) -> Result<NodeDuty> {
        self.elder_stores
            .chunk_records()
            .republish_chunk(chunk)
            .await
    }

    pub(crate) async fn get_data_exchange_packet(&self, prefix: Prefix) -> Result<DataExchange> {
        self.elder_stores.get_data_of(prefix).await
    }

    pub(crate) async fn update(&mut self, data: DataExchange) -> Result<()> {
        self.elder_stores.update(data).await
    }
}

impl Display for Metadata {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "Metadata")
    }
}

fn build_client_query_response(
    response: QueryResponse,
    correlation_id: MessageId,
    origin: EndUser,
) -> OutgoingMsg {
    OutgoingMsg {
        id: MessageId::in_response_to(&correlation_id),
        msg: MsgType::Client(ServiceMsg::QueryResponse {
            response,
            correlation_id,
        }),
        dst: DstLocation::EndUser(origin),
        aggregation: false,
    }
}

fn build_forward_query_response(
    response: QueryResponse,
    correlation_id: MessageId,
    origin: EndUser,
    section_pk: BlsPublicKey,
) -> Result<OutgoingMsg> {
    let msg = ServiceMsg::QueryResponse {
        response,
        correlation_id,
    };

    let mut rng = OsRng;
    let keypair = Keypair::new_ed25519(&mut rng);
    let payload = WireMsg::serialize_msg_payload(&msg)?;
    let signature = keypair.sign(&payload);

    let auth = ServiceAuth {
        node_pk: keypair.public_key(),
        signature,
    };

    Ok(OutgoingMsg {
        id: MessageId::in_response_to(&correlation_id),
        msg: MsgType::Node(NodeMsg::ForwardServiceMsg {
            msg,
            user: origin,
            auth,
        }),
        dst: DstLocation::Section {
            name: origin.xorname,
            section_pk,
        },
        aggregation: false,
    })
}

fn build_client_error_response(error: CmdError, msg_id: MessageId, origin: EndUser) -> OutgoingMsg {
    OutgoingMsg {
        id: MessageId::in_response_to(&msg_id),
        msg: MsgType::Client(ServiceMsg::CmdError {
            error,
            correlation_id: msg_id,
        }),
        dst: DstLocation::EndUser(origin),
        aggregation: false,
    }
}

// TODO: verify earlier so that this isn't needed
fn verify_op(auth: ServiceAuth, cmd: DataCmd) -> Result<AuthorityProof<ServiceAuth>> {
    let message = ServiceMsg::Cmd(cmd);
    let payload = WireMsg::serialize_msg_payload(&message)?;
    Ok(AuthorityProof::verify(auth, &payload)?)
}
