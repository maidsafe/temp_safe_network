// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Data messages and their possible responses.

use crate::messaging::MsgId;

use bls_ringct::ringct::{Amount, RingCtTransaction};
use serde::{Deserialize, Serialize};
use sn_dbc::{Dbc, KeyImage, OwnerOnce, SpentProofShare};
use std::fmt;

/// Network service messages that clients or nodes send in order to use the services,
/// communicate and carry out the Dbc tasks.
#[derive(Clone, Serialize, Deserialize)]
pub enum ServiceMsg {
    /// Mutate the dbc.
    Cmd(DbcMessage),
    /// Query for the DBC related information from the network.
    Query(DbcMessage),
    /// Response to to a Cmd or Query.
    Response {
        /// ID of the cmd message.
        dkg_msg: DbcMessage,
        /// ID of the cmd message.
        correlation_id: MsgId,
    },
}

/// Messages used for running DBC.
#[derive(Clone, Deserialize, Serialize)]
pub enum DbcMessage {
    Issue {
        starting_dbc: Dbc,
        amount: Amount,
        receive_owner: OwnerOnce,
    },
    AddSpentProof(SpentProofShare),
    CreateTransaction {
        input_dbc: Dbc,
        amount: Amount,
        output_owner: OwnerOnce,
    },
    WriteTransaction {
        key_image: KeyImage,
        transaction: RingCtTransaction,
    },
}

impl fmt::Debug for DbcMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            DbcMessage::Issue {
                starting_dbc,
                amount,
                receive_owner,
            } => write!(
                formatter,
                "Issue amount {:?} from {:?} to {:?}",
                amount, starting_dbc, receive_owner
            ),
            DbcMessage::AddSpentProof(spent_proof_share) => {
                write!(formatter, "AddSpentProof({:?})", spent_proof_share)
            }
            DbcMessage::CreateTransaction {
                input_dbc,
                amount,
                output_owner,
            } => write!(
                formatter,
                "CreateTransaction of amount {:?} from {:?} to {:?}",
                amount, input_dbc, output_owner
            ),
            DbcMessage::WriteTransaction {
                key_image,
                transaction,
            } => {
                write!(
                    formatter,
                    "WriteTransaction {:?} of {:?}",
                    transaction, key_image
                )
            }
        }
    }
}
