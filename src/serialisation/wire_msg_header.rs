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
    gen,
};
use std::{convert::TryFrom, fmt::Debug, mem::size_of};

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
}

// Bytes length in the header for the 'header_size' field
const HDR_SIZE_BYTES_LEN: usize = size_of::<u16>();

// Bytes index and size in the header for the 'version' field
const HDR_VERSION_BYTES_START: usize = 2;
const HDR_VERSION_BYTES_END: usize = 4;
const HDR_VERSION_BYTES_LEN: usize = size_of::<u16>();

// Bytes index in the header for the 'kind' field
const HDR_KIND_BYTES_START: usize = 4;

impl WireMsgHeader {
    // Instantiate a WireMsgHeader as per current supported version.
    pub fn new(kind: MessageKind) -> Self {
        Self {
            header_size: Self::size() as u16,
            version: MESSAGING_PROTO_VERSION,
            kind,
        }
    }

    pub fn kind(&self) -> MessageKind {
        self.kind
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

        // ...and finally let's read the message kind value (only 1 byte)
        let kind = MessageKind::try_from(bytes[HDR_KIND_BYTES_START])?;

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

        // ...and finally, let's write the value signaling the message kind
        let (buf_at_payload, _) = gen(be_u8(self.kind.into()), buf_at_msg_kind).map_err(|err| {
            Error::Serialisation(format!(
                "message kind field couldn't be serialized in header: {}",
                err
            ))
        })?;

        Ok(buf_at_payload)
    }

    // Size in bytes of WireMsgHeader when serialized.
    pub fn size() -> usize {
        // We don't use 'std::mem::size_of' since for the
        // 'MessageKind' enum it reports 2 bytes mem size,
        // and we want to serialize that field using 1 byte only.
        // HDR_SIZE_BYTES_LEN + HDR_VERSION_BYTES_LEN + 1 == 5
        5
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
