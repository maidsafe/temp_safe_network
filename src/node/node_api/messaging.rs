// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    client::ClientMsg, node::NodeMsg, Aggregation, DstInfo, DstLocation, Itinerary, MessageType,
    SrcLocation,
};
use crate::node::{
    network::Network,
    node_ops::{MsgType, OutgoingLazyError, OutgoingMsg, OutgoingSupportingInfo},
    Error, Result,
};
use crate::routing::XorName;
use std::collections::BTreeSet;
use tracing::{error, trace};

pub(crate) async fn send(msg: OutgoingMsg, network: &Network) -> Result<()> {
    let our_prefix = network.our_prefix().await;
    trace!("{:?}, Sending msg: {:?}", our_prefix, msg);
    let src = if msg.section_source {
        SrcLocation::Section(our_prefix.name())
    } else {
        SrcLocation::Node(network.our_name().await)
    };
    let itinerary = Itinerary {
        src,
        dst: msg.dst,
        aggregation: msg.aggregation,
    };

    let dst_name = msg.dst.name().ok_or(Error::NoDestinationName)?;
    let dst_section_pk = network
        .get_section_pk_by_name(&dst_name)
        .await?
        .bls()
        .ok_or(Error::NoSectionPublicKeyKnown(dst_name))?;
    let dst_info = DstInfo {
        // this is overwritten by routing at the moment...
        dst: dst_name,
        dst_section_pk,
    };
    let content = match msg.msg.clone() {
        MsgType::Client(msg) => MessageType::Client { msg, dst_info },
        MsgType::Node(msg) => {
            let src_section_pk = if itinerary.aggregate_at_dst() {
                Some(
                    network
                        .section_public_key()
                        .await?
                        .bls()
                        .ok_or(Error::NoSectionPublicKey)?,
                )
            } else {
                None
            };
            MessageType::Node {
                msg,
                src_section_pk,
                dst_info,
            }
        }
    };

    network
        .send_message(itinerary, content.clone())
        .await
        .map_or_else(
            |err| {
                error!("Unable to send msg: {:?}", err);
                Err(Error::UnableToSend(content))
            },
            |()| Ok(()),
        )
}

pub(crate) async fn send_error(msg: OutgoingLazyError, network: &Network) -> Result<()> {
    trace!("Sending error msg: {:?}", msg);
    let src = SrcLocation::Node(network.our_name().await);
    let itinerary = Itinerary {
        src,
        dst: msg.dst,
        aggregation: Aggregation::None,
    };

    let dst_name = msg.dst.name().ok_or(Error::NoDestinationName)?;
    let dst_section_pk = network
        .get_section_pk_by_name(&dst_name)
        .await?
        .bls()
        .ok_or(Error::NoSectionPublicKeyKnown(dst_name))?;

    let message = ClientMsg::ProcessingError(msg.msg);
    let message = MessageType::Client {
        msg: message.clone(),
        dst_info: DstInfo {
            dst: dst_name,
            dst_section_pk,
        },
    };
    let result = network.send_message(itinerary, message.clone()).await;

    result.map_or_else(
        |err| {
            error!("Unable to send msg: {:?}", err);
            Err(Error::UnableToSend(message))
        },
        |()| Ok(()),
    )
}

// TODO: Refactor over support/error
pub(crate) async fn send_support(msg: OutgoingSupportingInfo, network: &Network) -> Result<()> {
    trace!("Sending support msg: {:?}", msg);
    let src = SrcLocation::Node(network.our_name().await);
    let itinerary = Itinerary {
        src,
        dst: msg.dst,
        aggregation: Aggregation::None,
    };

    let dst_name = msg.dst.name().ok_or(Error::NoDestinationName)?;
    let dst_section_pk = network
        .get_section_pk_by_name(&dst_name)
        .await?
        .bls()
        .ok_or(Error::NoSectionPublicKeyKnown(dst_name))?;

    let message = MessageType::Client {
        msg: ClientMsg::SupportingInfo(msg.msg),
        dst_info: DstInfo {
            dst: dst_name,
            dst_section_pk,
        },
    };

    let result = network.send_message(itinerary, message.clone()).await;

    result.map_or_else(
        |err| {
            error!("Unable to send msg: {:?}", err);
            Err(Error::UnableToSend(message))
        },
        |()| Ok(()),
    )
}

pub(crate) async fn send_to_nodes(
    msg: &NodeMsg,
    targets: BTreeSet<XorName>,
    aggregation: Aggregation,
    network: &Network,
) -> Result<()> {
    let our_prefix = network.our_prefix().await;
    trace!(
        "{:?}, Sending msg to nodes: {:?}: {:?}",
        our_prefix,
        targets,
        msg
    );

    let name = network.our_name().await;
    let src_section_pk = Some(network.matching_section(&name).await?);

    for target in targets {
        let dst_section_pk = network
            .get_section_pk_by_name(&target)
            .await?
            .bls()
            .ok_or(Error::NoSectionPublicKeyKnown(target))?;
        let message = MessageType::Node {
            msg: msg.clone(),
            dst_info: DstInfo {
                dst: target,
                dst_section_pk,
            },
            src_section_pk,
        };

        network
            .send_message(
                Itinerary {
                    src: SrcLocation::Node(name),
                    dst: DstLocation::Node(XorName(target.0)),
                    aggregation,
                },
                message,
            )
            .await
            .map_or_else(
                |err| {
                    error!("Unable to send Message to Peer: {:?}", err);
                },
                |()| {},
            );
    }
    Ok(())
}
