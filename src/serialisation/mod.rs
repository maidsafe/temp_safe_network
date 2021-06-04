// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

pub mod wire_msg_header;

use self::wire_msg_header::{MessageKind, WireMsgHeader};
#[cfg(not(feature = "client-only"))]
use super::node;
use super::{client, section_info, DestInfo, Error, MessageId, MessageType, Result};
use bytes::Bytes;
use cookie_factory::{combinator::slice, gen_simple};
use std::fmt::Debug;
use threshold_crypto::PublicKey;
use xor_name::XorName;

// In order to send a message over the wire, it needs to be serialized
// along with a header (WireMsgHeader) which contains the information needed
// by the recipient to properly deserialize it.
// The WireMsg struct provides the utilities to serialize and deserialize messages.
#[derive(Debug, PartialEq, Clone)]
pub struct WireMsg {
    header: WireMsgHeader,
    payload: Bytes,
}

impl WireMsg {
    /// Creates a new instance keeping a (serialized) copy of the 'SectionInfo' message provided.
    pub fn new_section_info_msg(
        query: &section_info::Message,
        dest: XorName,
        dest_section_pk: PublicKey,
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
                dest,
                dest_section_pk,
                None,
            ),
            payload: Bytes::from(payload_vec),
        })
    }

    /// Creates a new instance keeping a (serialized) copy of the client 'Message' message provided.
    pub fn new_client_msg(
        msg: &client::ClientMsg,
        dest: XorName,
        dest_section_pk: PublicKey,
    ) -> Result<Self> {
        let payload_vec = rmp_serde::to_vec_named(&msg).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize client message payload (id: {}) with Msgpack: {}",
                msg.id(),
                err
            ))
        })?;

        Ok(Self {
            header: WireMsgHeader::new(msg.id(), MessageKind::Client, dest, dest_section_pk, None),
            payload: Bytes::from(payload_vec),
        })
    }

    /// Creates a new instance keeping a (serialized) copy of the node 'Message' message provided.
    #[cfg(not(feature = "client-only"))]
    pub fn new_routing_msg(
        msg: &node::RoutingMsg,
        dest: XorName,
        dest_section_pk: PublicKey,
    ) -> Result<Self> {
        let payload_vec = rmp_serde::to_vec_named(&msg).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize node message payload with Msgpack: {}",
                err
            ))
        })?;

        Ok(Self {
            header: WireMsgHeader::new(msg.id(), MessageKind::Routing, dest, dest_section_pk, None),
            payload: Bytes::from(payload_vec),
        })
    }

    /// Creates a new instance keeping a (serialized) copy of the node 'Message' message provided.
    #[cfg(not(feature = "client-only"))]
    pub fn new_node_msg(
        msg: &node::NodeMsg,
        dest: XorName,
        dest_section_pk: PublicKey,
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
                dest,
                dest_section_pk,
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
        let dest_info = DestInfo {
            dest: self.dest(),
            dest_section_pk: self.dest_section_pk(),
        };

        match self.header.kind() {
            MessageKind::SectionInfo => {
                let msg: section_info::Message =
                    rmp_serde::from_slice(&self.payload).map_err(|err| {
                        Error::FailedToParse(format!(
                            "Client message payload as Msgpack: {}",
                            err
                        ))
                    })?;

                Ok(MessageType::SectionInfo{msg, dest_info})
            }
            MessageKind::Client => {
                let msg: client::ClientMsg =
                    rmp_serde::from_slice(&self.payload).map_err(|err| {
                        Error::FailedToParse(format!(
                            "Client message payload as Msgpack: {}",
                            err
                        ))
                    })?;

                Ok(MessageType::Client{msg, dest_info})
            }
            #[cfg(feature = "client-only")]
            MessageKind::Routing => {
                Err(Error::FailedToParse("Message payload is a Node message which is not supported when feature 'client-only' is set".to_string()))
            }
            #[cfg(not(feature = "client-only"))]
            MessageKind::Routing => {
                let msg: node::RoutingMsg =
                    rmp_serde::from_slice(&self.payload).map_err(|err| {
                        Error::FailedToParse(format!("Node message payload as Msgpack: {}", err))
                    })?;

                Ok(MessageType::Routing{msg, dest_info})
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
                    dest_info,
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
    pub fn dest_section_pk(&self) -> PublicKey {
        self.header.dest_section_pk()
    }

    /// Return the destination for this message
    pub fn dest(&self) -> XorName {
        self.header.dest()
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
        query: &section_info::Message,
        dest: XorName,
        dest_section_pk: PublicKey,
    ) -> Result<Bytes> {
        Self::new_section_info_msg(query, dest, dest_section_pk)?.serialize()
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// Message, returning the serialized WireMsg.
    pub fn serialize_client_msg(
        msg: &client::ClientMsg,
        dest: XorName,
        dest_section_pk: PublicKey,
    ) -> Result<Bytes> {
        Self::new_client_msg(msg, dest, dest_section_pk)?.serialize()
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// node::Messsage, returning the serialized WireMsg.
    #[cfg(not(feature = "client-only"))]
    pub fn serialize_routing_msg(
        msg: &node::RoutingMsg,
        dest: XorName,
        dest_section_pk: PublicKey,
    ) -> Result<Bytes> {
        Self::new_routing_msg(msg, dest, dest_section_pk)?.serialize()
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// node::Node, returning the serialized WireMsg.
    #[cfg(not(feature = "client-only"))]
    pub fn serialize_node_msg(
        msg: &node::NodeMsg,
        dest: XorName,
        dest_section_pk: PublicKey,
        src_section_pk: Option<PublicKey>,
    ) -> Result<Bytes> {
        Self::new_node_msg(msg, dest, dest_section_pk, src_section_pk)?.serialize()
    }

    // Private function which returns the bytes size of this WireMsg
    // taking into account current self-contained payload.
    fn size(&self) -> usize {
        self.header.size() as usize + self.payload.len()
    }

    /// Update dest_pk and or dest in the WireMsg
    pub fn update_dest_info(&mut self, dest_pk: Option<PublicKey>, dest: Option<XorName>) {
        if let Some(dest) = dest {
            self.header.dest = dest
        }
        if let Some(dest_pk) = dest_pk {
            self.header.dest_section_pk = dest_pk
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use threshold_crypto::SecretKey;
    use xor_name::XorName;

    #[test]
    fn serialisation_section_info_msg() -> Result<()> {
        let dest = XorName::random();
        let dest_section_pk = SecretKey::random().public_key();

        let query = section_info::Message::GetSectionQuery {
            public_key: dest_section_pk.into(),
            is_node: true,
        };
        let wire_msg = WireMsg::new_section_info_msg(&query, dest, dest_section_pk)?;
        let serialized = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dest(), dest);
        assert_eq!(deserialized.dest_section_pk(), dest_section_pk);
        assert_eq!(deserialized.src_section_pk(), None);

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::SectionInfo {
                msg: query,
                dest_info: DestInfo {
                    dest,
                    dest_section_pk
                }
            }
        );

        Ok(())
    }

    #[test]
    fn serialisation_and_update_dest_for_section_info_msg() -> Result<()> {
        let dest = XorName::random();
        let dest_section_pk = SecretKey::random().public_key();

        let query = section_info::Message::GetSectionQuery {
            public_key: dest_section_pk.into(),
            is_node: true,
        };
        let mut wire_msg = WireMsg::new_section_info_msg(&query, dest, dest_section_pk)?;
        let serialized = wire_msg.serialize()?;

        let wire_msg2 = wire_msg.clone();
        let dest_new = XorName::random();
        wire_msg.update_dest_info(None, Some(dest_new));
        let serialised_second_msg = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialised_second_msg.clone())?;

        assert_ne!(serialized, serialised_second_msg);
        assert_ne!(wire_msg2, wire_msg);
        assert_eq!(deserialized.dest(), dest_new);
        assert_eq!(deserialized.dest_section_pk(), dest_section_pk);
        assert_eq!(deserialized.src_section_pk(), None);

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::SectionInfo {
                msg: query,
                dest_info: DestInfo {
                    dest: dest_new,
                    dest_section_pk
                }
            }
        );

        Ok(())
    }

    #[test]
    #[cfg(not(feature = "client-only"))]
    fn serialisation_node_msg() -> Result<()> {
        use crate::MessageId;
        use node::{NodeCmd, NodeMsg, NodeSystemCmd};

        let dest = XorName::random();
        let src_section_pk = SecretKey::random().public_key();
        let dest_section_pk = SecretKey::random().public_key();

        let node_cmd = NodeMsg::NodeCmd {
            cmd: NodeCmd::System(NodeSystemCmd::RegisterWallet(dest_section_pk.into())),
            id: MessageId::new(),
        };

        // first test without including a source section public key in the header
        let wire_msg = WireMsg::new_node_msg(&node_cmd, dest, dest_section_pk, None)?;
        let serialized = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dest(), dest);
        assert_eq!(deserialized.dest_section_pk(), dest_section_pk);
        assert_eq!(deserialized.src_section_pk(), None);

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::Node {
                msg: node_cmd.clone(),
                dest_info: DestInfo {
                    dest,
                    dest_section_pk
                },
                src_section_pk: None
            }
        );

        // let's now test including a source section public key in the header
        let wire_msg_with_src_pk =
            WireMsg::new_node_msg(&node_cmd, dest, dest_section_pk, Some(src_section_pk))?;
        let serialized = wire_msg_with_src_pk.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg_with_src_pk);
        assert_eq!(deserialized.msg_id(), wire_msg_with_src_pk.msg_id());
        assert_eq!(deserialized.dest(), dest);
        assert_eq!(deserialized.dest_section_pk(), dest_section_pk);
        assert_eq!(deserialized.src_section_pk(), Some(src_section_pk));

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::Node {
                msg: node_cmd,
                dest_info: DestInfo {
                    dest,
                    dest_section_pk
                },
                src_section_pk: Some(src_section_pk)
            }
        );

        Ok(())
    }
}
