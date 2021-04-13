// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{Error, Result};
use bytes::Bytes;
use cookie_factory::{
    bytes::{be_u16, be_u8},
    combinator::slice,
    gen,
};
use std::{convert::TryFrom, fmt::Debug, mem::size_of};
use threshold_crypto::PublicKey;
use xor_name::{XorName, XOR_NAME_LEN};

// Current version of the messaging protocol.
// At this point this implementation supports only this version.
const MESSAGING_PROTO_VERSION: u16 = 1u16;

// Header to be serialisied at the front of the wire message.
// This header contains the information needed to deserialize the payload.
#[derive(Debug, PartialEq)]
pub(crate) struct WireMsgHeader {
    header_size: u16,
    version: u16,
    kind: MessageKind,
    dest: XorName,
    dest_section_pk: PublicKey,
}

// Bytes length in the header for the 'header_size' field
const HDR_SIZE_BYTES_LEN: usize = size_of::<u16>();

// Bytes index and size in the header for the 'version' field
const HDR_VERSION_BYTES_START: usize = HDR_SIZE_BYTES_LEN;
const HDR_VERSION_BYTES_LEN: usize = size_of::<u16>();
const HDR_VERSION_BYTES_END: usize = HDR_VERSION_BYTES_START + HDR_VERSION_BYTES_LEN;

// Bytes index in the header for the 'kind' field
const HDR_KIND_BYTES_START: usize = HDR_VERSION_BYTES_END;
const HDR_KIND_BYTES_LEN: usize = 1;

// Bytes index in the header for the 'dest' field
const HDR_DEST_BYTES_START: usize = HDR_KIND_BYTES_START + HDR_KIND_BYTES_LEN;
const HDR_DEST_BYTES_LEN: usize = XOR_NAME_LEN;
const HDR_DEST_BYTES_END: usize = HDR_DEST_BYTES_START + HDR_DEST_BYTES_LEN;

// Bytes index in the header for the 'dest_section_pk' field
const HDR_DEST_PK_BYTES_START: usize = HDR_DEST_BYTES_END;
const HDR_DEST_PK_BYTES_LEN: usize = 48;
const HDR_DEST_PK_BYTES_END: usize = HDR_DEST_PK_BYTES_START + HDR_DEST_PK_BYTES_LEN;

impl WireMsgHeader {
    // Instantiate a WireMsgHeader as per current supported version.
    pub fn new(kind: MessageKind, dest: XorName, dest_section_pk: PublicKey) -> Self {
        Self {
            header_size: Self::size() as u16,
            version: MESSAGING_PROTO_VERSION,
            kind,
            dest,
            dest_section_pk,
        }
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

    // Parses the provided bytes to deserialize a WireMsgHeader,
    // returning the created WireMsgHeader, as well as the remaining bytes which
    // correspond to the message payload. The caller shall then take care of
    // deserializing the payload using the information provided in the WireMsgHeader.
    pub fn from(mut bytes: Bytes) -> Result<(Self, Bytes)> {
        // Let's make sure there is a minimum number of bytes to parse the header.
        let length = bytes.len();
        if length < Self::size() {
            return Err(Error::FailedToParse(format!(
                "not enough bytes received ({}) to deserialize wire message header",
                length
            )));
        }

        // Let's read the bytes which gives us the header size
        let mut header_size_bytes = [0; HDR_SIZE_BYTES_LEN];
        header_size_bytes[0..].copy_from_slice(&bytes[0..HDR_SIZE_BYTES_LEN]);
        let header_size = u16::from_be_bytes(header_size_bytes);

        // ...now let's read the serialization protocol version bytes
        let mut version_bytes = [0; HDR_VERSION_BYTES_LEN];
        version_bytes[0..].copy_from_slice(&bytes[HDR_VERSION_BYTES_START..HDR_VERSION_BYTES_END]);
        let version = u16::from_be_bytes(version_bytes);
        // Make sure we support this version
        if version != MESSAGING_PROTO_VERSION {
            return Err(Error::UnsupportedVersion(version));
        }

        // ...read the message kind value (only 1 byte)
        let kind = MessageKind::try_from(bytes[HDR_KIND_BYTES_START])?;

        // ...now let's read the destination bytes
        let mut dest_bytes = [0; HDR_DEST_BYTES_LEN];
        dest_bytes[0..].copy_from_slice(&bytes[HDR_DEST_BYTES_START..HDR_DEST_BYTES_END]);
        let dest = XorName(dest_bytes);

        // ...finally, let's read the destination section pubic key bytes
        let mut dest_pk_bytes = [0; HDR_DEST_PK_BYTES_LEN];
        dest_pk_bytes[0..].copy_from_slice(&bytes[HDR_DEST_PK_BYTES_START..HDR_DEST_PK_BYTES_END]);
        let dest_section_pk = PublicKey::from_bytes(&dest_pk_bytes).map_err(|err| {
            Error::FailedToParse(format!(
                "destination section PublicKey couldn't be deserialized from header: {}",
                err
            ))
        })?;

        let header = Self {
            header_size,
            version,
            kind,
            dest,
            dest_section_pk,
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
        let (buf_at_msg_kind, _) = gen(be_u16(self.version), buf_at_version).map_err(|err| {
            Error::Serialisation(format!(
                "version field couldn't be serialized in header: {}",
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

        // ...finally let's write the destination section public key
        let (buf_at_payload, _) = gen(slice(self.dest_section_pk.to_bytes()), buf_at_dest_pk)
            .map_err(|err| {
                Error::Serialisation(format!(
                    "destination section public key field couldn't be serialized in header: {}",
                    err
                ))
            })?;

        Ok(buf_at_payload)
    }

    // Size in bytes of WireMsgHeader when serialized.
    pub fn size() -> usize {
        // We don't use 'std::mem::size_of' since, for example, the
        // 'MessageKind' enum it reports 2 bytes mem size,
        // and we want to serialize that field using 1 byte only.
        HDR_SIZE_BYTES_LEN
            + HDR_VERSION_BYTES_LEN
            + HDR_KIND_BYTES_LEN
            + HDR_DEST_BYTES_LEN
            + HDR_DEST_PK_BYTES_LEN
    }
}

// Type of message being sent over the wire, this type
// is part of the WireMsgHeader.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MessageKind {
    Ping,
    SectionInfo,
    ClientMessage,
    NodeMessage,
}

impl TryFrom<u8> for MessageKind {
    type Error = super::Error;

    fn try_from(input: u8) -> Result<Self, Self::Error> {
        match input {
            0 => Ok(Self::Ping),
            1 => Ok(Self::SectionInfo),
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
            MessageKind::SectionInfo => 1,
            MessageKind::ClientMessage => 2,
            MessageKind::NodeMessage => 3,
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
            (MessageKind::Ping, 0),
            (MessageKind::SectionInfo, 1),
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
}
