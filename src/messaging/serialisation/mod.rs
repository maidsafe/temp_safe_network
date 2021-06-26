// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod wire_msg_header;

pub use self::wire_msg_header::MsgEnvelope;
pub(crate) use self::wire_msg_header::WireMsgHeader;
use super::node::NodeMsg;
use super::{
    client, section_info, DstLocation, Error, MessageId, MessageType, MsgAuthority, Result,
};
use bls::PublicKey as BlsPublicKey;
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
    pub payload: Bytes,
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
        msg_id: MessageId,
        payload: Bytes,
        msg_authority: MsgAuthority,
        dst_location: DstLocation,
    ) -> Result<Self> {
        Ok(Self {
            header: WireMsgHeader::new(msg_id, msg_authority, dst_location),
            payload,
        })
    }

    /// Convenience function to create a new 'SectionInfoMsg'.
    /// This function serializes the payload and assumes there is no need of a message authority.
    pub fn new_section_info_msg(
        query: &section_info::SectionInfoMsg,
        dst_location: DstLocation,
    ) -> Result<Self> {
        let payload = Self::serialize_msg_payload(query)?;

        Self::new_msg(MessageId::new(), payload, MsgAuthority::None, dst_location)
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
        // First we create a buffer with the capacity
        // needed to serialize the wire msg
        let max_length = WireMsgHeader::max_size() as usize + self.payload.len();
        let mut buffer = vec![0u8; max_length];

        let (buf_at_payload, bytes_written) = self.header.write(&mut buffer)?;

        // ...and finally we write the bytes of the serialized payload to the original buffer
        let _ = gen_simple(slice(self.payload.clone()), buf_at_payload).map_err(|err| {
            Error::Serialisation(format!("message payload couldn't be serialized: {}", err))
        })?;

        // We can now return the buffer containing the written bytes
        buffer.truncate(bytes_written as usize + self.payload.len());
        Ok(Bytes::from(buffer))
    }

    /// Deserialize the payload from this WireMsg returning a MessageType instance.
    pub fn to_message(&self) -> Result<MessageType> {
        let msg_envelope = self.header.msg_envelope().clone();

        match self.header.msg_envelope.msg_authority {
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
                let msg: NodeMsg = rmp_serde::from_slice(&self.payload).map_err(|err| {
                    Error::FailedToParse(format!("Node message payload as Msgpack: {}", err))
                })?;

                Ok(MessageType::Node { msg_envelope, msg })
            }
        }
    }

    /// Return the message id of this message
    pub fn msg_id(&self) -> MessageId {
        self.header.msg_envelope.msg_id
    }

    /// Return the message id of this message
    pub fn msg_authority(&self) -> &MsgAuthority {
        &self.header.msg_envelope.msg_authority
    }

    /// Return the destination section PublicKey for this message
    pub fn dst_section_pk(&self) -> Option<BlsPublicKey> {
        self.header.msg_envelope.dst_location.section_pk()
    }

    /// Return the destination for this message
    pub fn dst_location(&self) -> &DstLocation {
        &self.header.msg_envelope.dst_location
    }

    /// Return the source section PublicKey for this
    /// message if it's a NodeMsg
    pub fn src_section_pk(&self) -> Option<BlsPublicKey> {
        match &self.header.msg_envelope.msg_authority {
            MsgAuthority::Node(node_signed) => Some(node_signed.section_pk),
            MsgAuthority::BlsShare(bls_share_signed) => Some(bls_share_signed.section_pk),
            MsgAuthority::Section(section_signed) => Some(section_signed.section_pk),
            _ => None,
        }
    }

    // The following functions are just for convenience, which allow users to
    // not need to create an instance of WireMsg beforehand.

    /// Convenience function which creates a temporary WireMsg from the provided
    /// bytes, returning the deserialized message.
    pub fn deserialize(bytes: Bytes) -> Result<MessageType> {
        Self::from(bytes)?.to_message()
    }

    /// Update dst_location in the WireMsg
    pub fn update_dst_location(&mut self, dst_location: DstLocation) {
        self.header.set_dst_location(dst_location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::{
        client::{ChunkRead, DataQuery},
        node::{NodeCmd, NodeMsg, NodeSystemCmd},
        ClientSigned, MessageId, NodeSigned,
    };
    use crate::types::{ChunkAddress, Keypair};
    use anyhow::Result;
    use bls::SecretKey;
    use client::{ClientMsg, ProcessMsg, Query};
    use ed25519_dalek::Signer;
    use rand::rngs::OsRng;
    use xor_name::XorName;

    #[test]
    fn serialisation_section_info_msg() -> Result<()> {
        let dst_name = XorName::random();
        let dst_section_pk = SecretKey::random().public_key();
        let dst_location = DstLocation::Node {
            name: dst_name,
            section_pk: dst_section_pk,
        };

        let query = section_info::SectionInfoMsg::GetSectionQuery(dst_section_pk.into());

        let wire_msg = WireMsg::new_section_info_msg(&query, dst_location)?;
        let serialized = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dst_location(), &dst_location);
        assert_eq!(deserialized.dst_section_pk(), Some(dst_section_pk));
        assert_eq!(deserialized.src_section_pk(), None);

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::SectionInfo {
                msg_envelope: MsgEnvelope {
                    msg_id: wire_msg.msg_id(),
                    msg_authority: MsgAuthority::None,
                    dst_location,
                },
                msg: query,
            }
        );

        Ok(())
    }

    #[test]
    fn serialisation_and_update_dst_location_section_info_msg() -> Result<()> {
        let dst_name = XorName::random();
        let dst_section_pk = SecretKey::random().public_key();
        let dst_location = DstLocation::Node {
            name: dst_name,
            section_pk: dst_section_pk,
        };

        let query = section_info::SectionInfoMsg::GetSectionQuery(dst_section_pk.into());

        let mut wire_msg = WireMsg::new_section_info_msg(&query, dst_location)?;
        let serialized = wire_msg.serialize()?;

        let new_wire_msg = wire_msg.clone();
        let new_dst_location = DstLocation::Section {
            name: XorName::random(),
            section_pk: dst_section_pk,
        };
        wire_msg.update_dst_location(new_dst_location);
        let serialised_new_dst_location = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialised_new_dst_location.clone())?;

        assert_ne!(serialized, serialised_new_dst_location);
        assert_ne!(new_wire_msg, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dst_location(), &new_dst_location);
        assert_eq!(deserialized.dst_section_pk(), Some(dst_section_pk));
        assert_eq!(deserialized.src_section_pk(), None);

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::SectionInfo {
                msg_envelope: MsgEnvelope {
                    msg_id: wire_msg.msg_id(),
                    msg_authority: MsgAuthority::None,
                    dst_location: new_dst_location,
                },
                msg: query,
            }
        );

        Ok(())
    }

    #[test]
    fn serialisation_node_msg() -> Result<()> {
        let src_section_pk = SecretKey::random().public_key();
        let mut rng = OsRng;
        let src_node_keypair = ed25519_dalek::Keypair::generate(&mut rng);

        let dst_name = XorName::random();
        let dst_section_pk = SecretKey::random().public_key();
        let dst_location = DstLocation::Node {
            name: dst_name,
            section_pk: dst_section_pk,
        };

        let msg_id = MessageId::new();
        let pk = crate::types::PublicKey::Bls(dst_section_pk);

        let node_msg = NodeMsg::NodeCmd(NodeCmd::System(NodeSystemCmd::StorageFull {
            node_id: pk,
            section: pk.into(),
        }));

        let payload = WireMsg::serialize_msg_payload(&node_msg)?;
        let node_signed = NodeSigned {
            section_pk: src_section_pk,
            public_key: src_node_keypair.public,
            signature: src_node_keypair.sign(&payload),
        };

        let msg_authority = MsgAuthority::Node(node_signed);

        let wire_msg = WireMsg::new_msg(msg_id, payload, msg_authority.clone(), dst_location)?;
        let serialized = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dst_location(), &dst_location);
        assert_eq!(deserialized.dst_section_pk(), Some(dst_section_pk));
        assert_eq!(deserialized.src_section_pk(), Some(src_section_pk));

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::Node {
                msg_envelope: MsgEnvelope {
                    msg_id: wire_msg.msg_id(),
                    msg_authority,
                    dst_location,
                },
                msg: node_msg.clone(),
            }
        );

        Ok(())
    }

    #[test]
    fn serialisation_client_msg() -> Result<()> {
        let mut rng = OsRng;
        let src_client_keypair = Keypair::new_ed25519(&mut rng);

        let dst_name = XorName::random();
        let dst_section_pk = SecretKey::random().public_key();
        let dst_location = DstLocation::Node {
            name: dst_name,
            section_pk: dst_section_pk,
        };

        let msg_id = MessageId::new();

        let client_msg = ClientMsg::Process(ProcessMsg::Query {
            id: msg_id,
            query: Query::Data(DataQuery::Blob(ChunkRead::Get(ChunkAddress::Private(
                XorName::random(),
            )))),
        });

        let payload = WireMsg::serialize_msg_payload(&client_msg)?;
        let client_signed = ClientSigned {
            public_key: src_client_keypair.public_key(),
            signature: src_client_keypair.sign(&payload),
        };

        let msg_authority = MsgAuthority::Client(client_signed);

        let wire_msg = WireMsg::new_msg(msg_id, payload, msg_authority.clone(), dst_location)?;
        let serialized = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dst_location(), &dst_location);
        assert_eq!(deserialized.dst_section_pk(), Some(dst_section_pk));
        assert_eq!(deserialized.src_section_pk(), None);

        // test deserialisation of payload
        assert_eq!(
            deserialized.to_message()?,
            MessageType::Client {
                msg_envelope: MsgEnvelope {
                    msg_id: wire_msg.msg_id(),
                    msg_authority,
                    dst_location,
                },
                msg: client_msg.clone(),
            }
        );

        Ok(())
    }
}
