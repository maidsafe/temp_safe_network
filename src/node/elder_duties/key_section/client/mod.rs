// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod client_msg_handling;

use self::client_msg_handling::ClientMsgHandling;
use crate::{
    node::node_ops::{GatewayDuty, Msg, NodeMessagingDuty, NodeOperation},
    ElderState, Error, Result,
};
use log::{error, trace};
use sn_routing::Event as RoutingEvent;
use std::fmt::{self, Display, Formatter};

/// A client gateway routes messages
/// back and forth between a client and the network.
pub struct ClientGateway {
    client_msg_handling: ClientMsgHandling,
    elder_state: ElderState,
}

impl ClientGateway {
    pub async fn new(elder_state: ElderState) -> Result<Self> {
        let client_msg_handling = ClientMsgHandling::new(elder_state.clone());

        let gateway = Self {
            client_msg_handling,
            elder_state,
        };

        Ok(gateway)
    }

    pub async fn process_as_gateway(&self, cmd: GatewayDuty) -> Result<NodeOperation> {
        trace!("Processing as gateway");
        use GatewayDuty::*;
        match cmd {
            FindClientFor(msg) => self.try_find_client(&msg).await,
            ProcessClientEvent(event) => self.process_client_event(event).await,
            NoOp => Ok(NodeOperation::NoOp),
        }
    }

    async fn try_find_client(&self, msg: &Msg) -> Result<NodeOperation> {
        trace!("trying to find client...");
        if let Some(xorname) = msg.dst.name() {
            if self.elder_state.prefix().matches(&xorname) {
                trace!("Message matches gateway prefix");
                self.client_msg_handling.match_outgoing(&msg.msg).await?;
                return Ok(NodeOperation::NoOp);
            }
        }
        Ok(NodeMessagingDuty::SendToSection {
            msg: msg.clone(),
            as_node: true,
        }
        .into())
    }

    /// This is where client input is parsed.
    async fn process_client_event(&self, event: RoutingEvent) -> Result<NodeOperation> {
        trace!("Processing client event");
        match event {
            RoutingEvent::ClientMessageReceived { content, src, .. } => {
                trace!("Deserialized client msg is {:?}", content);

                // if !validate_client_sig(&content) {
                //     return Err(Error::NetworkData(DtError::InvalidSignature));
                // }

                match self
                    .client_msg_handling
                    .track_incoming_message(&content, src)
                    .await
                {
                    Ok(()) => unimplemented!(),
                    // Ok(()) => Ok(KeySectionDuty::EvaluateUserMsg {
                    //     msg: *content,
                    //     user: User::Client { public_key, socket_id },
                    // }
                    // .into()),
                    Err(e) => Err(e),
                }
            }
            other => {
                error!("NOT SUPPORTED YET: {:?}", other);
                Err(Error::Logic(
                    "Event not supported in Client event processing".to_string(),
                ))
            }
        }
    }
}

// fn validate_client_sig(msg: &Message) -> bool {
//     if !msg.origin.is_client() {
//         return false;
//     }
//     let verification = msg.verify();
//     if let Ok(true) = verification {
//         true
//     } else {
//         warn!(
//             "Msg {:?} from {:?} is invalid. Verification: {:?}",
//             msg.id(),
//             msg.origin.address().xorname(),
//             verification
//         );
//         false
//     }
// }

impl Display for ClientGateway {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "ClientGateway")
    }
}
