// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{client, infrastructure, node, Error, MessageType, Result};
use bytes::Bytes;
use cookie_factory::{
    bytes::{be_u16, be_u8},
    combinator::slice,
    gen,
};
use std::{convert::TryFrom, fmt::Debug};

// Current version of the messaging protocol.
// At this point this implementation supports only this version.
const MESSAGING_PROTO_VERSION: u16 = 1u16;

// In order to send a MsgEnvelope over the wire, it needs to be serialized
// along with a header (WireMsgHeader) which contains the information needed
// by the recipient to properly deserialise it.
// The WireMsg struct provides the utilities to serialise and deserialise MsgEnvelope.
#[derive(Debug, PartialEq)]
pub struct WireMsg {
    header: WireMsgHeader,
    payload: Bytes,
}

impl WireMsg {
    /// Creates a new instance of a 'Ping' message.
    pub fn new_ping_msg() -> WireMsg {
        Self {
            header: WireMsgHeader::new(MessageKind::Ping),
            payload: Bytes::new(),
        }
    }

    /// Creates a new instance keeping a (serialised) copy of the 'InfrastructureQuery' message provided.
    pub fn new_infrastructure_query(query: &infrastructure::Query) -> Result<WireMsg> {
        let payload_vec = rmp_serde::to_vec_named(&query).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialise infrastructure query payload with Msgpack: {}",
                err
            ))
        })?;

        Ok(Self {
            header: WireMsgHeader::new(MessageKind::InfrastructureQuery),
            payload: Bytes::from(payload_vec),
        })
    }

    /// Creates a new instance keeping a (serialised) copy of the client 'MsgEnvelope' message provided.
    pub fn new_client_msg(msg: &client::MsgEnvelope) -> Result<WireMsg> {
        let payload_vec = rmp_serde::to_vec_named(&msg).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialise client message payload (id: {}) with Msgpack: {}",
                msg.id(),
                err
            ))
        })?;

        Ok(Self {
            header: WireMsgHeader::new(MessageKind::ClientMessage),
            payload: Bytes::from(payload_vec),
        })
    }

    /// Creates a new instance keeping a (serialised) copy of the node 'Message' message provided.
    pub fn new_node_msg(msg: &node::NodeMessage) -> Result<WireMsg> {
        let payload_vec = rmp_serde::to_vec_named(&msg).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialise node message payload with Msgpack: {}",
                err
            ))
        })?;

        Ok(Self {
            header: WireMsgHeader::new(MessageKind::NodeMessage),
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

    /// Return the serialised WireMsg, which contains the WireMsgHeader bytes,
    /// followed by the payload bytes, i.e. the serialised MsgEnvelope.
    pub fn serialise(&self) -> Result<Bytes> {
        // First we create a buffer with the exact size
        // needed to serialize the wire msg
        let mut buffer = vec![0u8; self.size()];

        let buf_at_payload = self.header.write(&mut buffer)?;

        // ...and finally we write the bytes of the serialised payload
        let _ = gen(slice(self.payload.clone()), &mut buf_at_payload[..]).map_err(|err| {
            Error::Serialisation(format!("message payload couldn't be serialised: {}", err))
        })?;

        // We can now return the buffer containing the written bytes
        Ok(Bytes::from(buffer))
    }

    /// Deserialise the payload from this WireMsg returning a Message instance.
    pub fn to_message(&self) -> Result<MessageType> {
        match self.header.kind {
            MessageKind::Ping => Ok(MessageType::Ping),
            MessageKind::InfrastructureQuery => {
                let query: infrastructure::Query =
                    rmp_serde::from_slice(&self.payload).map_err(|err| {
                        Error::FailedToParse(format!(
                            "Client message payload as Msgpack: {:?}",
                            err
                        ))
                    })?;
                Ok(MessageType::InfrastructureQuery(query))
            }
            MessageKind::ClientMessage => {
                let client_msg: client::MsgEnvelope = rmp_serde::from_slice(&self.payload)
                    .map_err(|err| {
                        Error::FailedToParse(format!(
                            "Client message payload as Msgpack: {:?}",
                            err
                        ))
                    })?;
                Ok(MessageType::ClientMessage(client_msg))
            }
            MessageKind::NodeMessage => {
                let node_msg: node::NodeMessage =
                    rmp_serde::from_slice(&self.payload).map_err(|err| {
                        Error::FailedToParse(format!("Node message payload as Msgpack: {:?}", err))
                    })?;
                Ok(MessageType::NodeMessage(node_msg))
            }
        }
    }

    // The following functions are just for convinience, which allow users to
    // not needing to create an instance of WireMsg beforehand.

    /// Convenience function which creates a temporary WireMsg from the provided
    /// bytes, returning the deserialised message.
    pub fn deserialise(bytes: Bytes) -> Result<MessageType> {
        Self::from(bytes)?.to_message()
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// MsgEnvelope, returning the serialised WireMsg.
    pub fn serialise_infrastructure_query(query: &infrastructure::Query) -> Result<Bytes> {
        Self::new_infrastructure_query(query)?.serialise()
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// MsgEnvelope, returning the serialised WireMsg.
    pub fn serialise_client_msg(msg: &client::MsgEnvelope) -> Result<Bytes> {
        Self::new_client_msg(msg)?.serialise()
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// node::Messsage, returning the serialised WireMsg.
    pub fn serialise_node_msg(msg: &node::NodeMessage) -> Result<Bytes> {
        Self::new_node_msg(msg)?.serialise()
    }

    // Private function which returns the bytes size of this WireMsg
    // taking into account current self-contained payload.
    fn size(&self) -> usize {
        WireMsgHeader::size() + self.payload.len()
    }
}

// Type of message being sent over the wire, this type
// is part of the WireMsgHeader.
#[derive(Debug, Clone, Copy, PartialEq)]
enum MessageKind {
    Ping,
    InfrastructureQuery,
    ClientMessage,
    NodeMessage,
}

impl TryFrom<u8> for MessageKind {
    type Error = super::Error;

    fn try_from(input: u8) -> Result<Self, Self::Error> {
        match input {
            0 => Ok(Self::Ping),
            1 => Ok(Self::InfrastructureQuery),
            2 => Ok(Self::ClientMessage),
            3 => Ok(Self::NodeMessage),
            other => Err(Error::UnsupportedMessageKind(other)),
        }
    }
}

impl From<MessageKind> for u8 {
    fn from(kind: MessageKind) -> u8 {
        match kind {
            MessageKind::Ping => 0,
            MessageKind::InfrastructureQuery => 1,
            MessageKind::ClientMessage => 2,
            MessageKind::NodeMessage => 3,
        }
    }
}

// Header to be serialisied at the front of the wire message.
// This header contains the information needed to deserialise the payload.
#[derive(Debug, PartialEq)]
struct WireMsgHeader {
    header_size: u16,
    version: u16,
    kind: MessageKind,
}

impl WireMsgHeader {
    // Instantiate a WireMsgHeader as per current supported version.
    pub fn new(kind: MessageKind) -> Self {
        Self {
            header_size: Self::size() as u16,
            version: MESSAGING_PROTO_VERSION,
            kind,
        }
    }

    // Parses the provided bytes to deserialize a WireMsgHeader,
    // returning the created WireMsgHeader, as well as the remaining bytes which
    // correspond to the message payload. The caller shall then take care of
    // deserializing the payload using the information provided in the WireMsgHeader.
    pub fn from(mut bytes: Bytes) -> Result<(Self, Bytes)> {
        // We need at least 4 bytes as current version 1 only has
        // a header size field and the protocol version. Thus, let's
        // make sure there is this number of bytes as a minimum.
        let length = bytes.len();
        if length < Self::size() {
            return Err(Error::FailedToParse(format!(
                "not enough bytes received ({}) to deserialise wire message header",
                length
            )));
        }

        // Let's read the bytes which gives us the header size
        let mut header_size_bytes = [0; 2];
        header_size_bytes[0..].copy_from_slice(&bytes[0..2]);
        let header_size = u16::from_be_bytes(header_size_bytes);

        // ...now let's read the serialization protocol version bytes
        let mut version_bytes = [0; 2];
        version_bytes[0..].copy_from_slice(&bytes[2..4]);
        let version = u16::from_be_bytes(version_bytes);
        // Make sure we support this version
        if version != MESSAGING_PROTO_VERSION {
            return Err(Error::UnsupportedVersion(version));
        }

        // ...and finally let's read the message kind value (only 1 byte)
        let kind = MessageKind::try_from(bytes[4])?;

        let header = Self {
            header_size,
            version,
            kind,
        };

        // Get a slice for the payload bytes, i.e. the bytes after the header bytes
        let payload_bytes = bytes.split_off(header_size.into());

        Ok((header, payload_bytes))
    }

    pub fn write<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a mut [u8]> {
        // Let's write the header size first
        let (buf_at_version, _) =
            gen(be_u16(self.header_size), &mut buffer[..]).map_err(|err| {
                Error::Serialisation(format!(
                    "header size value couldn't be serialised in header: {}",
                    err
                ))
            })?;

        // Now let's write the serialisation protocol version bytes
        let (buf_at_msg_kind, _) =
            gen(be_u16(self.version), &mut buf_at_version[..]).map_err(|err| {
                Error::Serialisation(format!(
                    "version field couldn't be serialised in header: {}",
                    err
                ))
            })?;

        // ...and finally, let's write the value signaling the message kind
        let (buf_at_payload, _) =
            gen(be_u8(self.kind.into()), &mut buf_at_msg_kind[..]).map_err(|err| {
                Error::Serialisation(format!(
                    "message kind field couldn't be serialised in header: {}",
                    err
                ))
            })?;

        Ok(buf_at_payload)
    }

    // Size in bytes of WireMsgHeader when serialised.
    pub fn size() -> usize {
        // We don't use 'std::mem::size_of' since for the
        // 'MessageKind' enum it reports 2 bytes mem size,
        // and we want to serialise that field using 1 byte only.
        5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use xor_name::XorName;

    #[test]
    fn message_kind_from_u8() -> Result<()> {
        for &(kind, byte) in &[
            (MessageKind::Ping, 0),
            (MessageKind::InfrastructureQuery, 1),
            (MessageKind::ClientMessage, 2),
            (MessageKind::NodeMessage, 3),
        ] {
            assert_eq!(kind as u8, byte);
            assert_eq!(MessageKind::try_from(byte)?, kind);
        }

        for byte in 4..u8::MAX {
            assert!(MessageKind::try_from(byte).is_err());
        }

        Ok(())
    }

    #[test]
    fn serialisation_ping() -> Result<()> {
        let wire_msg = WireMsg::new_ping_msg();
        let serialised = wire_msg.serialise()?;
        let deserialised = WireMsg::from(serialised)?;
        assert_eq!(deserialised, wire_msg);
        assert_eq!(wire_msg.to_message()?, MessageType::Ping);

        Ok(())
    }

    #[test]
    fn serialisation_infrastructure_query() -> Result<()> {
        let random_xor = XorName::random();
        let query = infrastructure::Query::GetSectionRequest(random_xor);
        let wire_msg = WireMsg::new_infrastructure_query(&query)?;
        let serialised = wire_msg.serialise()?;
        let deserialised = WireMsg::from(serialised)?;
        assert_eq!(deserialised, wire_msg);
        assert_eq!(
            wire_msg.to_message()?,
            MessageType::InfrastructureQuery(query)
        );

        Ok(())
    }
}
