// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

/// WireMsg Header
mod wire_msg_header;

pub use self::wire_msg_header::MsgEnvelope;
pub(crate) use self::wire_msg_header::WireMsgHeader;
use super::node::{self, Variant};
use super::{
    client, section_info, DstLocation, Error, MessageId, MessageType, MsgAuthority, Result,
};
use bls::PublicKey;
use bytes::Bytes;
use cookie_factory::{combinator::slice, gen_simple};
use serde::Serialize;
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
    /// Serializes the message provided. This function shall be used to
    /// obtain the serialized payload which is needed to create the `MsgAuthority`.
    /// Once the caller contains both the serialized payload and `MsgAuthority`,
    /// it can invoke the `new_msg` function to instantiate a `WireMsg`.
    pub fn serialize_msg_payload<T: Serialize>(msg: &T) -> Result<Bytes> {
        let payload_vec = rmp_serde::to_vec_named(&msg).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize message payload with Msgpack: {}",
                err
            ))
        })?;

        Ok(Bytes::from(payload_vec))
    }

    /// Creates a new `WireMsg` with the provided serialized payload and `MsgAuthority`.
    pub fn new_msg(
        payload: Bytes,
        msg_authority: MsgAuthority,
        dst_location: DstLocation,
        dst_section_pk: PublicKey,
    ) -> Result<Self> {
        Ok(Self {
            header: WireMsgHeader::new(
                MessageId::new(),
                msg_authority,
                dst_location,
                dst_section_pk,
            ),
            payload,
        })
    }

    /// Convenience function to create a new 'SectionInfoMsg'.
    /// This function serializes the payload and assumes there is no need of a message authority.
    pub fn new_section_info_msg(
        query: &section_info::SectionInfoMsg,
        dst_location: DstLocation,
        dst_section_pk: PublicKey,
    ) -> Result<Self> {
        let payload = Self::serialize_msg_payload(query)?;

        Self::new_msg(payload, MsgAuthority::None, dst_location, dst_section_pk)
    }

    /// Attempts to create an instance of WireMsg by deserialising the bytes provided.
    /// To succeed, the bytes should contain at least a valid WireMsgHeader.
    pub fn from(bytes: Bytes) -> Result<Self> {
        // Deserialize the header bytes first
        let (header, payload) = WireMsgHeader::from(bytes)?;

        // We can now create a deserialized WireMsg using the read bytes
        Ok(Self { header, payload })
    }

    /// Returns `true` if the message is a JoinRequest.
    pub fn is_join_request(&self) -> Result<bool> {
        unimplemented!()
        /*        match self.header.src_authority() {
            MsgAuthority::Node(_) | MsgAuthority::BlsShare(_) | MsgAuthority::Section(_) => {
                if let MessageType::Routing { msg, .. } = self.to_message()? {
                    if let Variant::JoinRequest(_) = msg.variant {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }*/
    }

    /// Return the serialized WireMsg, which contains the WireMsgHeader bytes,
    /// followed by the payload bytes, i.e. the serialized Message.
    pub fn serialize(&self) -> Result<Bytes> {
        // First we create a buffer with the capacity
        // needed to serialize the wire msg
        let max_length = WireMsgHeader::max_size() as usize + self.payload.len();
        let mut buffer = vec![0u8; max_length];

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
        let msg_envelope = self.header.msg_envelope().clone();

        match self.header.msg_authority() {
            MsgAuthority::None => {
                let msg: section_info::SectionInfoMsg = rmp_serde::from_slice(&self.payload)
                    .map_err(|err| {
                        Error::FailedToParse(format!(
                            "Section info message payload as Msgpack: {}",
                            err
                        ))
                    })?;

                Ok(MessageType::SectionInfo { msg_envelope, msg })
            }
            MsgAuthority::Client(_) => {
                let msg: client::ClientMsg =
                    rmp_serde::from_slice(&self.payload).map_err(|err| {
                        Error::FailedToParse(format!("Client message payload as Msgpack: {}", err))
                    })?;

                Ok(MessageType::Client { msg_envelope, msg })
            }
            MsgAuthority::Node(_) | MsgAuthority::BlsShare(_) | MsgAuthority::Section(_) => {
                let msg: node::NodeMsg = rmp_serde::from_slice(&self.payload).map_err(|err| {
                    Error::FailedToParse(format!("Node message payload as Msgpack: {}", err))
                })?;

                Ok(MessageType::Node { msg_envelope, msg })
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
    pub fn dst_location(&self) -> &DstLocation {
        self.header.dst_location()
    }

    /// Return the source section PublicKey for this
    /// message if it's a NodeMsg and it was
    /// provided in the header of message.
    pub fn src_section_pk(&self) -> Option<PublicKey> {
        unimplemented!();
        //self.header.src_section_pk()
    }

    // The following functions are just for convenience, which allow users to
    // not need to create an instance of WireMsg beforehand.

    /// Convenience function which creates a temporary WireMsg from the provided
    /// bytes, returning the deserialized message.
    pub fn deserialize(bytes: Bytes) -> Result<MessageType> {
        Self::from(bytes)?.to_message()
    }
    /*
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
        /// node::Node, returning the serialized WireMsg.
        pub fn serialize_node_msg(
            msg: &node::NodeMsg,
            dst: XorName,
            dst_section_pk: PublicKey,
            src_section_pk: Option<PublicKey>,
        ) -> Result<Bytes> {
            Self::new_node_msg(msg, dst, dst_section_pk, src_section_pk)?.serialize()
        }
    */

    /// Update dst_pk and or dst in the WireMsg
    pub fn update_dst_info(&mut self, dst_pk: Option<PublicKey>, dst: Option<XorName>) {
        unimplemented!();
        /*
        if let Some(dst) = dst {
            self.header.dst = dst
        }
        if let Some(dst_pk) = dst_pk {
            self.header.dst_section_pk = dst_pk
        }
        */
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
        let dst_name = XorName::random();
        let dst_section_pk = SecretKey::random().public_key();
        let dst_location = DstLocation::Node(dst_name);

        let query = section_info::SectionInfoMsg::GetSectionQuery(dst_section_pk.into());

        let wire_msg = WireMsg::new_section_info_msg(&query, dst_location, dst_section_pk)?;
        let serialized = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dst_location(), &dst_location);
        assert_eq!(deserialized.dst_section_pk(), dst_section_pk);
        /*
                // test deserialisation of payload
                assert_eq!(
                    deserialized.to_message()?,
                    MessageType::SectionInfo {
                        msg: query,
                        msg_envelope: MsgEnvelope {
                            msg_id: wire_msg.msg_id(),
                            msg_authority: MsgAuthority::None,
                            dst_location: dst_location,
                            dst_section_pk: dst_section_pk
                        }
                    }
                );
        */
        Ok(())
    }
    /*
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
    */
}
