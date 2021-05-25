// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{Error, MessageId, Result, MESSAGE_ID_LEN};
use bytes::Bytes;
use cookie_factory::{
    bytes::{be_u16, be_u8},
    combinator::slice,
    gen,
};
use std::{convert::TryFrom, fmt::Debug, mem::size_of};
use threshold_crypto::{PublicKey, PK_SIZE};
use xor_name::{XorName, XOR_NAME_LEN};

// Current version of the messaging protocol.
// At this point this implementation supports only this version.
const MESSAGING_PROTO_VERSION: u16 = 1u16;

// Header to be serialisied at the front of the wire message.
// This header contains the information needed to deserialize the payload.
#[derive(Debug, PartialEq)]
pub(crate) struct WireMsgHeader {
    msg_id: MessageId,
    header_size: u16,
    version: u16,
    kind: MessageKind,
    dest: XorName,
    dest_section_pk: PublicKey,
    src_section_pk: Option<PublicKey>,
}

// Bytes length in the header for the 'header_size' field
const HDR_SIZE_BYTES_LEN: usize = size_of::<u16>();

// Bytes index and size in the header for the 'version' field
const HDR_VERSION_BYTES_START: usize = HDR_SIZE_BYTES_LEN;
const HDR_VERSION_BYTES_LEN: usize = size_of::<u16>();
const HDR_VERSION_BYTES_END: usize = HDR_VERSION_BYTES_START + HDR_VERSION_BYTES_LEN;

// Bytes index in the header for the 'msg_id' field
const HDR_MSG_ID_BYTES_START: usize = HDR_VERSION_BYTES_END;
const HDR_MSG_ID_BYTES_LEN: usize = MESSAGE_ID_LEN;
const HDR_MSG_ID_BYTES_END: usize = HDR_MSG_ID_BYTES_START + HDR_MSG_ID_BYTES_LEN;

// Bytes index in the header for the 'kind' field
const HDR_KIND_BYTES_START: usize = HDR_MSG_ID_BYTES_END;
const HDR_KIND_BYTES_LEN: usize = 1;

// Bytes index in the header for the 'dest' field
const HDR_DEST_BYTES_START: usize = HDR_KIND_BYTES_START + HDR_KIND_BYTES_LEN;
const HDR_DEST_BYTES_LEN: usize = XOR_NAME_LEN;
const HDR_DEST_BYTES_END: usize = HDR_DEST_BYTES_START + HDR_DEST_BYTES_LEN;

// Bytes index in the header for the 'dest_section_pk' field
const HDR_DEST_PK_BYTES_START: usize = HDR_DEST_BYTES_END;
const HDR_DEST_PK_BYTES_LEN: usize = PK_SIZE;
const HDR_DEST_PK_BYTES_END: usize = HDR_DEST_PK_BYTES_START + HDR_DEST_PK_BYTES_LEN;

// Bytes index in the header for the 'src_section_pk' field
const HDR_SRC_PK_BYTES_START: usize = HDR_DEST_PK_BYTES_END;
const HDR_SRC_PK_BYTES_LEN: usize = PK_SIZE;
const HDR_SRC_PK_BYTES_END: usize = HDR_SRC_PK_BYTES_START + HDR_SRC_PK_BYTES_LEN;

const HEADER_MIN_SIZE: usize = HDR_SIZE_BYTES_LEN
    + HDR_VERSION_BYTES_LEN
    + HDR_MSG_ID_BYTES_LEN
    + HDR_KIND_BYTES_LEN
    + HDR_DEST_BYTES_LEN
    + HDR_DEST_PK_BYTES_LEN;

impl WireMsgHeader {
    // Instantiate a WireMsgHeader as per current supported version.
    pub fn new(
        msg_id: MessageId,
        kind: MessageKind,
        dest: XorName,
        dest_section_pk: PublicKey,
        src_section_pk: Option<PublicKey>,
    ) -> Self {
        Self {
            msg_id,
            header_size: Self::bytes_size(src_section_pk.is_some()) as u16,
            version: MESSAGING_PROTO_VERSION,
            kind,
            dest,
            dest_section_pk,
            src_section_pk,
        }
    }

    // Return the message id of this message
    pub fn msg_id(&self) -> MessageId {
        self.msg_id
    }

    // Return the kind of this message
    pub fn kind(&self) -> MessageKind {
        self.kind
    }

    // Return the destination section PublicKey for this message
    pub fn dest_section_pk(&self) -> PublicKey {
        self.dest_section_pk
    }

    // Return the destination for this message
    pub fn dest(&self) -> XorName {
        self.dest
    }

    // Return the source section PublicKey for this message
    // if it's a NodeMsg and it was included in the header
    pub fn src_section_pk(&self) -> Option<PublicKey> {
        self.src_section_pk
    }

    // Parses the provided bytes to deserialize a WireMsgHeader,
    // returning the created WireMsgHeader, as well as the remaining bytes which
    // correspond to the message payload. The caller shall then take care of
    // deserializing the payload using the information provided in the WireMsgHeader.
    pub fn from(mut bytes: Bytes) -> Result<(Self, Bytes)> {
        // Let's make sure there is a minimum number of bytes to parse the header size part.
        let length = bytes.len();
        if length < HDR_SIZE_BYTES_LEN {
            return Err(Error::FailedToParse(format!(
                "not enough bytes received ({}) to even read the wire message header length field",
                length
            )));
        }

        // Let's read the bytes which gives us the header size
        let mut header_size_bytes = [0; HDR_SIZE_BYTES_LEN];
        header_size_bytes[0..].copy_from_slice(&bytes[0..HDR_SIZE_BYTES_LEN]);
        let header_size = u16::from_be_bytes(header_size_bytes);

        // TODO: since the header is currently (and temporarily) of a
        // varian length, we check that at least we have the minimum number of bytes
        // for the header of any kind of message to be deserialised.
        if length < header_size.into() || HEADER_MIN_SIZE > header_size.into() {
            return Err(Error::FailedToParse(format!(
                "not enough bytes received ({}) to deserialize wire message header",
                length
            )));
        }

        // ...now let's read the serialization protocol version bytes
        let mut version_bytes = [0; HDR_VERSION_BYTES_LEN];
        version_bytes[0..].copy_from_slice(&bytes[HDR_VERSION_BYTES_START..HDR_VERSION_BYTES_END]);
        let version = u16::from_be_bytes(version_bytes);
        // Make sure we support this version
        if version != MESSAGING_PROTO_VERSION {
            return Err(Error::UnsupportedVersion(version));
        }

        // ...read the message id bytes
        let mut msg_id_bytes = [0; HDR_MSG_ID_BYTES_LEN];
        msg_id_bytes[0..].copy_from_slice(&bytes[HDR_MSG_ID_BYTES_START..HDR_MSG_ID_BYTES_END]);
        let msg_id = MessageId::with(msg_id_bytes);

        // ...read the message kind value (only 1 byte)
        let kind = MessageKind::try_from(bytes[HDR_KIND_BYTES_START])?;

        // ...now let's read the destination bytes
        let mut dest_bytes = [0; HDR_DEST_BYTES_LEN];
        dest_bytes[0..].copy_from_slice(&bytes[HDR_DEST_BYTES_START..HDR_DEST_BYTES_END]);
        let dest = XorName(dest_bytes);

        // ...read the destination section pubic key bytes
        let mut dest_pk_bytes = [0; HDR_DEST_PK_BYTES_LEN];
        dest_pk_bytes[0..].copy_from_slice(&bytes[HDR_DEST_PK_BYTES_START..HDR_DEST_PK_BYTES_END]);
        let dest_section_pk = PublicKey::from_bytes(&dest_pk_bytes).map_err(|err| {
            Error::FailedToParse(format!(
                "destination section PublicKey couldn't be deserialized from header: {}",
                err
            ))
        })?;

        // ...finally, we read the source section pubic key bytes if it's a NodeMsg
        // and if the header size has the exact number of bytes to read a PublicKey from.
        // Once we move back to fixed-length header we won't need this check.
        let src_section_pk = if kind == MessageKind::Node
            && HEADER_MIN_SIZE + HDR_SRC_PK_BYTES_LEN == header_size.into()
        {
            let mut src_pk_bytes = [0; HDR_SRC_PK_BYTES_LEN];
            src_pk_bytes[0..].copy_from_slice(&bytes[HDR_SRC_PK_BYTES_START..HDR_SRC_PK_BYTES_END]);
            let src_section_pk = PublicKey::from_bytes(&src_pk_bytes).map_err(|err| {
                Error::FailedToParse(format!(
                    "source section PublicKey couldn't be deserialized from header: {}",
                    err
                ))
            })?;
            Some(src_section_pk)
        } else {
            None
        };

        let header = Self {
            msg_id,
            header_size,
            version,
            kind,
            dest,
            dest_section_pk,
            src_section_pk,
        };

        // Get a slice for the payload bytes, i.e. the bytes after the header bytes
        let payload_bytes = bytes.split_off(header_size.into());

        Ok((header, payload_bytes))
    }

    pub fn write<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a mut [u8]> {
        // Let's write the header size first
        let (buf_at_version, _) = gen(be_u16(self.header_size), buffer).map_err(|err| {
            Error::Serialisation(format!(
                "header size value couldn't be serialized in header: {}",
                err
            ))
        })?;

        // Now let's write the serialisation protocol version bytes
        let (buf_at_msg_id, _) = gen(be_u16(self.version), buf_at_version).map_err(|err| {
            Error::Serialisation(format!(
                "version field couldn't be serialized in header: {}",
                err
            ))
        })?;

        // Write the message id bytes
        let (buf_at_msg_kind, _) =
            gen(slice(self.msg_id.as_ref()), buf_at_msg_id).map_err(|err| {
                Error::Serialisation(format!(
                    "message id field couldn't be serialized in header: {}",
                    err
                ))
            })?;

        // ...now let's write the value signaling the message kind
        let (buf_at_dest, _) = gen(be_u8(self.kind.into()), buf_at_msg_kind).map_err(|err| {
            Error::Serialisation(format!(
                "message kind field couldn't be serialized in header: {}",
                err
            ))
        })?;

        // ...write the destination bytes
        let (buf_at_dest_pk, _) = gen(slice(&self.dest), buf_at_dest).map_err(|err| {
            Error::Serialisation(format!(
                "destination field couldn't be serialized in header: {}",
                err
            ))
        })?;

        // ...now let's write the destination section public key
        let (buf_at_src_pk, _) = gen(slice(self.dest_section_pk.to_bytes()), buf_at_dest_pk)
            .map_err(|err| {
                Error::Serialisation(format!(
                    "destination section public key field couldn't be serialized in header: {}",
                    err
                ))
            })?;

        // ...now write the source section public key if it's a NodeMsg
        // and a source section public key was provided
        let buf_at_payload = if let Some(src_section_pk) = self.src_section_pk {
            if self.kind != MessageKind::Node {
                return Err(Error::Serialisation(format!(
                    "source section public key field couldn't be serialized in header since it's not a NodeMsg but a {:?}",
                    self.kind
                )));
            }

            let (buf, _) = gen(slice(src_section_pk.to_bytes()), buf_at_src_pk).map_err(|err| {
                Error::Serialisation(format!(
                    "source section public key field couldn't be serialized in header: {}",
                    err
                ))
            })?;

            buf
        } else {
            buf_at_src_pk
        };

        Ok(buf_at_payload)
    }

    // Size in bytes of this WireMsgHeader when serialized.
    pub fn size(&self) -> u16 {
        self.header_size
    }

    // Size in bytes when serialized if a WireMsgHeader
    // depending if a source section public key is included.
    fn bytes_size(with_src_section_pk: bool) -> usize {
        // We don't use 'std::mem::size_of' since, for example, the
        // 'MessageKind' enum it reports 2 bytes mem size,
        // and we want to serialize that field using 1 byte only.
        if with_src_section_pk {
            HEADER_MIN_SIZE + HDR_SRC_PK_BYTES_LEN
        } else {
            HEADER_MIN_SIZE
        }
    }
}

// Type of message being sent over the wire, this type
// is part of the WireMsgHeader.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MessageKind {
    SectionInfo,
    Client,
    Routing,
    Node,
}

// Bytes values used for each of the kind of messages
// when written to the message header
const SECTION_INFO_KIND: u8 = 0x00;
const CLIENT_MSG_KIND: u8 = 0x01;
const ROUTING_MSG_KIND: u8 = 0x02;
const NODE_MSG_KIND: u8 = 0x03;

impl TryFrom<u8> for MessageKind {
    type Error = super::Error;

    fn try_from(input: u8) -> Result<Self, Self::Error> {
        match input {
            SECTION_INFO_KIND => Ok(Self::SectionInfo),
            CLIENT_MSG_KIND => Ok(Self::Client),
            ROUTING_MSG_KIND => Ok(Self::Routing),
            NODE_MSG_KIND => Ok(Self::Node),
            other => Err(Error::UnsupportedMessageKind(other)),
        }
    }
}

impl From<MessageKind> for u8 {
    fn from(kind: MessageKind) -> u8 {
        match kind {
            MessageKind::SectionInfo => SECTION_INFO_KIND,
            MessageKind::Client => CLIENT_MSG_KIND,
            MessageKind::Routing => ROUTING_MSG_KIND,
            MessageKind::Node => NODE_MSG_KIND,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn message_kind_from_u8() -> Result<()> {
        for &(kind, byte) in &[
            (MessageKind::SectionInfo, SECTION_INFO_KIND),
            (MessageKind::Client, CLIENT_MSG_KIND),
            (MessageKind::Routing, ROUTING_MSG_KIND),
            (MessageKind::Node, NODE_MSG_KIND),
        ] {
            assert_eq!(kind as u8, byte);
            assert_eq!(MessageKind::try_from(byte)?, kind);
        }

        for byte in 5..u8::MAX {
            assert!(MessageKind::try_from(byte).is_err());
        }

        Ok(())
    }
}
