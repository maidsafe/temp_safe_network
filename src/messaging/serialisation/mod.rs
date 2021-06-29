// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// WireMsg Header
pub mod wire_msg_header;

use self::wire_msg_header::{MessageKind, WireMsgHeader};
#[cfg(not(feature = "client-only"))]
use super::node;
use super::{client, section_info, DstInfo, Error, MessageId, MessageType, Result};
use crate::messaging::node::Variant;
use bls::PublicKey;
use bytes::Bytes;
use cookie_factory::{combinator::slice, gen_simple};
use std::fmt::Debug;
use xor_name::XorName;

/// In order to send a message over the wire, it needs to be serialized
/// along with a header (WireMsgHeader) which contains the information needed
/// by the recipient to properly deserialize it.
/// The WireMsg struct provides the utilities to serialize and deserialize messages.
#[derive(Debug, PartialEq, Clone)]
pub struct WireMsg {
    header: WireMsgHeader,
    payload: Bytes,
}

impl WireMsg {
    /// Creates a new instance keeping a (serialized) copy of the 'SectionInfo' message provided.
    pub fn new_section_info_msg(
        query: &section_info::SectionInfoMsg,
        dst: XorName,
        dst_section_pk: PublicKey,
    ) -> Result<Self> {
        let payload_vec = rmp_serde::to_vec_named(&query).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize network info payload with Msgpack: {}",
                err
            ))
        })?;

        Ok(Self {
            header: WireMsgHeader::new(
                MessageId::new(),
                MessageKind::SectionInfo,
                dst,
                dst_section_pk,
                None,
            ),
            payload: Bytes::from(payload_vec),
        })
    }

    /// Creates a new instance keeping a (serialized) copy of the client 'Message' message provided.
    pub fn new_client_msg(
        msg: &client::ClientMsg,
        dst: XorName,
        dst_section_pk: PublicKey,
    ) -> Result<Self> {
        let payload_vec = rmp_serde::to_vec_named(&msg).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize client message payload (id: {}) with Msgpack: {}",
                msg.id(),
                err
            ))
        })?;

        Ok(Self {
            header: WireMsgHeader::new(msg.id(), MessageKind::Client, dst, dst_section_pk, None),
            payload: Bytes::from(payload_vec),
        })
    }

    /// Creates a new instance keeping a (serialized) copy of the node 'Message' message provided.
    #[cfg(not(feature = "client-only"))]
    pub fn new_routing_msg(
        msg: &node::RoutingMsg,
        dst: XorName,
        dst_section_pk: PublicKey,
        is_join_request: bool,
    ) -> Result<Self> {
        let payload_vec = rmp_serde::to_vec_named(&msg).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize node message payload with Msgpack: {}",
                err
            ))
        })?;

        let kind = if is_join_request {
            MessageKind::JoinRequest
        } else {
            MessageKind::Routing
        };

        Ok(Self {
            header: WireMsgHeader::new(msg.id, kind, dst, dst_section_pk, None),
            payload: Bytes::from(payload_vec),
        })
    }

    /// Returns 'true' if kind is `MessageKind::JoinsRequest`
    #[cfg(not(feature = "client-only"))]
    pub fn is_join_request(&self) -> bool {
        matches!(self.header.kind(), MessageKind::JoinRequest)
    }

    /// Creates a new instance keeping a (serialized) copy of the node 'Message' message provided.
    #[cfg(not(feature = "client-only"))]
    pub fn new_node_msg(
        msg: &node::NodeMsg,
        dst: XorName,
        dst_section_pk: PublicKey,
        src_section_pk: Option<PublicKey>,
    ) -> Result<Self> {
        let payload_vec = rmp_serde::to_vec_named(&msg).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize a node command message payload with Msgpack: {}",
                err
            ))
        })?;

        Ok(Self {
            header: WireMsgHeader::new(
                msg.id(),
                MessageKind::Node,
                dst,
                dst_section_pk,
                src_section_pk,
            ),
            payload: Bytes::from(payload_vec),
        })
    }

    /// Attempts to create an instance of WireMsg by deserialising the bytes provided.
    /// To succeed, the bytes should contain at least a valid WireMsgHeader.
    pub fn from(bytes: Bytes) -> Result<Self> {
        // Deserialize the header bytes first
        let (header, payload) = WireMsgHeader::from(bytes)?;

        // We can now create a deserialized WireMsg using the read bytes
        Ok(Self { header, payload })
    }

    /// Return the serialized WireMsg, which contains the WireMsgHeader bytes,
    /// followed by the payload bytes, i.e. the serialized Message.
    pub fn serialize(&self) -> Result<Bytes> {
        // First we create a buffer with the exact size
        // needed to serialize the wire msg
        let mut buffer = vec![0u8; self.size()];

        let buf_at_payload = self.header.write(&mut buffer)?;

        // ...and finally we write the bytes of the serialized payload to the original buffer
        let _ = gen_simple(slice(self.payload.clone()), buf_at_payload).map_err(|err| {
            Error::Serialisation(format!("message payload couldn't be serialized: {}", err))
        })?;

        // We can now return the buffer containing the written bytes
        Ok(Bytes::from(buffer))
    }

    /// Deserialize the payload from this WireMsg returning a Message instance.
    pub fn to_message(&self) -> Result<MessageType> {
        let dst_info = DstInfo {
            dst: self.dst(),
            dst_section_pk: self.dst_section_pk(),
        };

        match self.header.kind() {
            MessageKind::SectionInfo => {
                let msg: section_info::SectionInfoMsg =
                    rmp_serde::from_slice(&self.payload).map_err(|err| {
                        Error::FailedToParse(format!(
                            "Client message payload as Msgpack: {}",
                            err
                        ))
                    })?;

                Ok(MessageType::SectionInfo{msg, dst_info})
            }
            MessageKind::Client => {
                let msg: client::ClientMsg =
                    rmp_serde::from_slice(&self.payload).map_err(|err| {
                        Error::FailedToParse(format!(
                            "Client message payload as Msgpack: {}",
                            err
                        ))
                    })?;

                Ok(MessageType::Client{msg, dst_info})
            }
            #[cfg(feature = "client-only")]
            MessageKind::Routing
            | MessageKind::JoinRequest => {
                Err(Error::FailedToParse("Message payload is a Node message which is not supported when feature 'client-only' is set".to_string()))
            }
            #[cfg(not(feature = "client-only"))]
            MessageKind::Routing
            | MessageKind::JoinRequest => {
                let msg: node::RoutingMsg =
                    rmp_serde::from_slice(&self.payload).map_err(|err| {
                        Error::FailedToParse(format!("Node message payload as Msgpack: {}", err))
                    })?;

                Ok(MessageType::Routing{msg, dst_info})
            }
            #[cfg(feature = "client-only")]
            MessageKind::Node => {
                Err(Error::FailedToParse("Message payload is a NodeCmd message which is not supported when feature 'client-only' is set".to_string()))
            }
            #[cfg(not(feature = "client-only"))]
            MessageKind::Node => {
                let node_cmd: node::NodeMsg =
                    rmp_serde::from_slice(&self.payload).map_err(|err| {
                        Error::FailedToParse(format!("NodeCmd message payload as Msgpack: {}", err))
                    })?;

                Ok(MessageType::Node{
                    msg: node_cmd,
                    dst_info,
                    src_section_pk: self.src_section_pk()
                })
            }
        }
    }

    /// Return the message id of this message
    pub fn msg_id(&self) -> MessageId {
        self.header.msg_id()
    }

    /// Return the destination section PublicKey for this message
    pub fn dst_section_pk(&self) -> PublicKey {
        self.header.dst_section_pk()
    }

    /// Return the destination for this message
    pub fn dst(&self) -> XorName {
        self.header.dst()
    }

    /// Return the source section PublicKey for this
    /// message if it's a NodeMsg and it was
    /// provided in the header of message.
    pub fn src_section_pk(&self) -> Option<PublicKey> {
        self.header.src_section_pk()
    }

    // The following functions are just for convenience, which allow users to
    // not need to create an instance of WireMsg beforehand.

    /// Convenience function which creates a temporary WireMsg from the provided
    /// bytes, returning the deserialized message.
    pub fn deserialize(bytes: Bytes) -> Result<MessageType> {
        Self::from(bytes)?.to_message()
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// MsgEnvelope, returning the serialized WireMsg.
    pub fn serialize_section_info_msg(
        query: &section_info::SectionInfoMsg,
        dst: XorName,
        dst_section_pk: PublicKey,
    ) -> Result<Bytes> {
        Self::new_section_info_msg(query, dst, dst_section_pk)?.serialize()
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// Message, returning the serialized WireMsg.
    pub fn serialize_client_msg(
        msg: &client::ClientMsg,
        dst: XorName,
        dst_section_pk: PublicKey,
    ) -> Result<Bytes> {
        Self::new_client_msg(msg, dst, dst_section_pk)?.serialize()
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// node::Messsage, returning the serialized WireMsg.
    #[cfg(not(feature = "client-only"))]
    pub fn serialize_routing_msg(
        msg: &node::RoutingMsg,
        dst: XorName,
        dst_section_pk: PublicKey,
    ) -> Result<Bytes> {
        Self::new_routing_msg(
            msg,
            dst,
            dst_section_pk,
            matches!(msg.variant, Variant::JoinRequest(_)),
        )?
        .serialize()
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// node::Node, returning the serialized WireMsg.
    #[cfg(not(feature = "client-only"))]
    pub fn serialize_node_msg(
        msg: &node::NodeMsg,
        dst: XorName,
        dst_section_pk: PublicKey,
        src_section_pk: Option<PublicKey>,
    ) -> Result<Bytes> {
        Self::new_node_msg(msg, dst, dst_section_pk, src_section_pk)?.serialize()
    }

    // Private function which returns the bytes size of this WireMsg
    // taking into account current self-contained payload.
    fn size(&self) -> usize {
        self.header.size() as usize + self.payload.len()
    }

    /// Update dst_pk and or dst in the WireMsg
    pub fn update_dst_info(&mut self, dst_pk: Option<PublicKey>, dst: Option<XorName>) {
        if let Some(dst) = dst {
            self.header.dst = dst
        }
        if let Some(dst_pk) = dst_pk {
            self.header.dst_section_pk = dst_pk
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use bls::SecretKey;
    use xor_name::XorName;

    #[test]
    fn serialisation_section_info_msg() -> Result<()> {
        let dst = XorName::random();
        let dst_section_pk = SecretKey::random().public_key();

        let query = section_info::SectionInfoMsg::GetSectionQuery(dst_section_pk.into());
        let wire_msg = WireMsg::new_section_info_msg(&query, dst, dst_section_pk)?;
        let serialized = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dst(), dst);
        assert_eq!(deserialized.dst_section_pk(), dst_section_pk);
        assert_eq!(deserialized.src_section_pk(), None);

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::SectionInfo {
                msg: query,
                dst_info: DstInfo {
                    dst,
                    dst_section_pk
                }
            }
        );

        Ok(())
    }

    #[test]
    fn serialisation_and_update_dst_for_section_info_msg() -> Result<()> {
        let dst = XorName::random();
        let dst_section_pk = SecretKey::random().public_key();

        let query = section_info::SectionInfoMsg::GetSectionQuery(dst_section_pk.into());
        let mut wire_msg = WireMsg::new_section_info_msg(&query, dst, dst_section_pk)?;
        let serialized = wire_msg.serialize()?;

        let wire_msg2 = wire_msg.clone();
        let dst_new = XorName::random();
        wire_msg.update_dst_info(None, Some(dst_new));
        let serialised_second_msg = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialised_second_msg.clone())?;

        assert_ne!(serialized, serialised_second_msg);
        assert_ne!(wire_msg2, wire_msg);
        assert_eq!(deserialized.dst(), dst_new);
        assert_eq!(deserialized.dst_section_pk(), dst_section_pk);
        assert_eq!(deserialized.src_section_pk(), None);

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::SectionInfo {
                msg: query,
                dst_info: DstInfo {
                    dst: dst_new,
                    dst_section_pk
                }
            }
        );

        Ok(())
    }

    #[test]
    #[cfg(not(feature = "client-only"))]
    fn serialisation_node_msg() -> Result<()> {
        use crate::messaging::MessageId;
        use node::{NodeCmd, NodeMsg, NodeSystemCmd};

        let dst = XorName::random();
        let src_section_pk = SecretKey::random().public_key();
        let dst_section_pk = SecretKey::random().public_key();

        let pk = crate::types::PublicKey::Bls(dst_section_pk);

        let node_cmd = NodeMsg::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::StorageFull {
                node_id: pk,
                section: pk.into(),
            }),
            id: MessageId::new(),
        };

        // first test without including a source section public key in the header
        let wire_msg = WireMsg::new_node_msg(&node_cmd, dst, dst_section_pk, None)?;
        let serialized = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dst(), dst);
        assert_eq!(deserialized.dst_section_pk(), dst_section_pk);
        assert_eq!(deserialized.src_section_pk(), None);

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::Node {
                msg: node_cmd.clone(),
                dst_info: DstInfo {
                    dst,
                    dst_section_pk
                },
                src_section_pk: None
            }
        );

        // let's now test including a source section public key in the header
        let wire_msg_with_src_pk =
            WireMsg::new_node_msg(&node_cmd, dst, dst_section_pk, Some(src_section_pk))?;
        let serialized = wire_msg_with_src_pk.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg_with_src_pk);
        assert_eq!(deserialized.msg_id(), wire_msg_with_src_pk.msg_id());
        assert_eq!(deserialized.dst(), dst);
        assert_eq!(deserialized.dst_section_pk(), dst_section_pk);
        assert_eq!(deserialized.src_section_pk(), Some(src_section_pk));

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::Node {
                msg: node_cmd,
                dst_info: DstInfo {
                    dst,
                    dst_section_pk
                },
                src_section_pk: Some(src_section_pk)
            }
        );

        Ok(())
    }
}
