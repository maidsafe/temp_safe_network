// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    client::ClientMsg, node::NodeMsg, ClientSigned, DstLocation, MessageId, MsgKind, WireMsg,
};
use crate::node::{
    network::Network,
    node_ops::{MsgType, OutgoingLazyError, OutgoingMsg, OutgoingSupportingInfo},
    Error, Result,
};
use crate::routing::XorName;
use crate::types::Keypair;
use rand::rngs::OsRng;
use std::collections::BTreeSet;
use tracing::{error, trace};

pub(crate) async fn send(msg: OutgoingMsg, network: &Network) -> Result<()> {
    let our_prefix = network.our_prefix().await;
    trace!("{:?}, Sending msg: {:?}", our_prefix, msg);

    let wire_msg = match msg.msg {
        MsgType::Client(client_msg) => {
            let payload = WireMsg::serialize_msg_payload(&client_msg)?;
            // FIXME: define which signature/authority this message should really carry,
            // perhaps it needs to carry Node signature. Giving a random sig temporarily
            let client_signed = random_client_signature();
            let msg_kind = MsgKind::ClientMsg(client_signed);
            WireMsg::new_msg(msg.id, payload, msg_kind, msg.dst)?
        }
        MsgType::Node(node_msg) => {
            let mut wire_msg = if msg.aggregation {
                network
                    .sign_msg_for_dst_accumulation(node_msg, msg.dst)
                    .await?
            } else {
                network.sign_single_src_msg(node_msg, msg.dst).await?
            };
            wire_msg.set_msg_id(msg.id);
            wire_msg
        }
    };

    network
        .send_message(wire_msg)
        .await
        .map_err(|err| err.into())
}

pub(crate) async fn send_error(msg: OutgoingLazyError, network: &Network) -> Result<()> {
    trace!("Sending error msg: {:?}", msg);
    let payload = WireMsg::serialize_msg_payload(&ClientMsg::ProcessingError(msg.msg))?;
    // FIXME: define which signature/authority this message should really carry,
    // perhaps it needs to carry Node signature. Giving a random sig temporarily
    let client_signed = random_client_signature();
    let msg_kind = MsgKind::ClientMsg(client_signed);

    let wire_msg = WireMsg::new_msg(MessageId::new(), payload, msg_kind, msg.dst)?;

    network
        .send_message(wire_msg)
        .await
        .map_err(|err| err.into())
}

// TODO: Refactor over support/error
pub(crate) async fn send_support(msg: OutgoingSupportingInfo, network: &Network) -> Result<()> {
    trace!("Sending support msg: {:?}", msg);
    let payload = WireMsg::serialize_msg_payload(&ClientMsg::SupportingInfo(msg.msg))?;
    // FIXME: define which signature/authority this message should really carry,
    // perhaps it needs to carry Node signature. Giving a random sig temporarily
    let client_signed = random_client_signature();
    let msg_kind = MsgKind::ClientMsg(client_signed);

    let wire_msg = WireMsg::new_msg(MessageId::new(), payload, msg_kind, msg.dst)?;

    network
        .send_message(wire_msg)
        .await
        .map_err(|err| err.into())
}

pub(crate) async fn send_to_nodes(
    msg_id: MessageId,
    node_msg: NodeMsg,
    targets: BTreeSet<XorName>,
    aggregation: bool,
    network: &Network,
) -> Result<()> {
    let our_prefix = network.our_prefix().await;
    trace!(
        "{:?}, Sending msg ({}) to nodes: {:?}: {:?}",
        our_prefix,
        msg_id,
        targets,
        node_msg
    );

    // we create a dummy/random dst location,
    // we will set it correctly for each msg and target
    let dummy_dst_location = DstLocation::Node {
        name: network.our_name().await,
        section_pk: network
            .section_public_key()
            .await?
            .bls()
            .ok_or(Error::NoSectionPublicKey)?,
    };

    let mut wire_msg = if aggregation {
        network
            .sign_msg_for_dst_accumulation(node_msg, dummy_dst_location)
            .await?
    } else {
        network
            .sign_single_src_msg(node_msg, dummy_dst_location)
            .await?
    };
    wire_msg.set_msg_id(msg_id);

    for target in targets {
        let dst_section_pk = network
            .get_section_pk_by_name(&target)
            .await?
            .bls()
            .ok_or(Error::NoSectionPublicKeyKnown(target))?;

        wire_msg.set_dst_section_pk(dst_section_pk);
        wire_msg.set_dst_xorname(target);

        network.send_message(wire_msg.clone()).await.map_or_else(
            |err| {
                error!("Unable to send Message to Peer: {:?}", err);
            },
            |()| {},
        );
    }

    Ok(())
}

fn random_client_signature() -> ClientSigned {
    let mut rng = OsRng;
    let keypair = Keypair::new_ed25519(&mut rng);
    let signature = keypair.sign(b"FIXME");

    ClientSigned {
        public_key: keypair.public_key(),
        signature,
    }
}
