// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::msg::Message;
use safe_nd::{MessageId, PublicId, Request, Response, XorName};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Node internal cmds, about what requests to make.

/// Any network node
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum NodeCmd {
    Elder(ElderCmd),
    Adult(AdultCmd),
}

/// Only Adults can issue
/// these cmds.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum AdultCmd {
    /// Send a response as an Adult to our section's Elders.
    RespondToOurElders(Message),
    /// Send the same request to each individual Adult.
    SendToAdultPeers {
        targets: BTreeSet<XorName>,
        msg: Message,
    },
}

/// Elder only cmds.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum ElderCmd {
    Gateway(GatewayCmd),
    Metadata(MetadataCmd),
    Transfer(TransferCmd),
    Payment(PaymentCmd),
}

/// The Gateway consists of
/// the Elders in a section.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum GatewayCmd {
    /// Vote for a cmd so we can process the deferred action on consensus.
    /// (Currently immediately.)
    VoteFor(ConsensusAction),
    /// Send a validated client request from Gateway to the appropriate destination nodes.
    ForwardClientRequest(Message),
    /// Send a response back to client.
    RespondToClient {
        message_id: MessageId,
        response: Response,
    },
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum TransferCmd {
    /// Send data to section after payment.
    SendToSection(Message),
    /// Send a response from an Elder
    /// node to Gateway nodes.
    RespondToGateway { sender: XorName, msg: Message },
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum PaymentCmd {
    /// Send data to section after payment.
    SendToSection(Message),
    /// Send a response from an Elder
    /// node to Gateway nodes.
    RespondToGateway { sender: XorName, msg: Message },
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum MetadataCmd {
    /// Send the same request to each individual Adult.
    SendToAdults {
        targets: BTreeSet<XorName>,
        msg: Message,
    },
    /// Send a response from an Elder
    /// node to Gateway nodes.
    RespondToGateway { sender: XorName, msg: Message },
}

// Need to Serialize/Deserialize to go through the consensus process.
/// A ConsensusAction is something only
/// taking place at the network Gateways.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum ConsensusAction {
    /// When Gateway nodes consider a request
    /// valid, they will vote for it to be forwarded.
    /// As they reach consensus, this is then carried out.
    Forward {
        request: Request,
        client_public_id: PublicId,
        message_id: MessageId,
    },
}

//     /// Send a response to
//     /// our section's Elders, i.e. our peers.
//     RespondToElderPeers(Message),
