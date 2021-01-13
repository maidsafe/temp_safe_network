// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{Error, MsgEnvelope, Result};
use bytes::Bytes;
use cookie_factory::{bytes::be_u16, combinator::slice, gen};
use std::{fmt::Debug, mem::size_of};

// Current version of the messaging protocol.
// At this point this implementation supports only this version.
const MESSAGING_PROTO_VERSION: u16 = 1u16;

// In order to send a MsgEnvelope over the wire, it needs to be serialised
// along with a header (WireMsgHeader) which contains the information needed
// by the recipient to properly deserialise it.
// The WireMsg struct provides the utilities to serialise and deserialise MsgEnvelope.
#[derive(Debug)]
pub(crate) struct WireMsg {
    header: WireMsgHeader,
    payload: Bytes,
}

impl WireMsg {
    // Creates a new instance keeping a (serialised) copy of the MsgEnvelope provided.
    pub fn new(msg: &MsgEnvelope) -> Result<WireMsg> {
        let payload_vec = rmp_serde::to_vec_named(&msg).map_err(|err| {
            Error::Serialisation(format!(
                "Could not serialize message payload (id: {}) with Msgpack: {}",
                msg.id(),
                err
            ))
        })?;

        Ok(Self {
            header: WireMsgHeader::new(),
            payload: Bytes::from(payload_vec),
        })
    }

    // Attempts to create an instance of WireMsg by deserialising the bytes provided.
    // To succeed, the bytes should contain at least a valid WireMsgHeader.
    pub fn from(bytes: Bytes) -> Result<Self> {
        // Deserialise the header bytes first
        let (header, payload) = WireMsgHeader::from(bytes)?;

        // We can now create a deserialised WireMsg using the read bytes
        Ok(Self { header, payload })
    }

    // Return the serialised WireMsg, which contains the WireMsgHeader bytes,
    // followed by the payload bytes, i.e. the serialised MsgEnvelope.
    pub fn serialise(&self) -> Result<Bytes> {
        // First we create a buffer with the exact size
        // needed to serialise the wire msg
        let mut buffer = vec![0u8; self.size()];

        // Let's write the header size first
        let (buf_at_version, _) =
            gen(be_u16(self.header.header_size), &mut buffer[..]).map_err(|err| {
                Error::Serialisation(format!(
                    "header size value couldn't be serialised in header: {}",
                    err
                ))
            })?;

        // Now let's write the serialisation protocol version bytes
        let (buf_at_payload, _) = gen(be_u16(self.header.version), &mut buf_at_version[..])
            .map_err(|err| {
                Error::Serialisation(format!(
                    "version field couldn't be serialised in header: {}",
                    err
                ))
            })?;

        // ...and finally we write the bytes of the serialised payload
        let (_, _) = gen(slice(self.payload.clone()), &mut buf_at_payload[..]).map_err(|err| {
            Error::Serialisation(format!("message payload couldn't be serialised: {}", err))
        })?;

        // We can now return the buffer containing the written bytes
        Ok(Bytes::from(buffer))
    }

    // Convenience function which creates a temporary WireMsg from the provided
    // MsgEnvelope, returning the serialised WireMsg.
    pub fn serialise_msg(msg: &MsgEnvelope) -> Result<Bytes> {
        Self::new(msg)?.serialise()
    }

    // Deserialise the payload from this WireMsg returning a MsgEnvelope instance.
    pub fn deserialise(&self) -> Result<MsgEnvelope> {
        rmp_serde::from_slice(&self.payload)
            .map_err(|err| Error::FailedToParse(format!("message payload as Msgpack: {:?}", err)))
    }

    // Convenience function which creates a temporary WireMsg from the provided
    // bytes, returning the deserialised payload, i.e. a MsgEnvelope instance.
    pub fn deserialise_msg(bytes: Bytes) -> Result<MsgEnvelope> {
        Self::from(bytes)?.deserialise()
    }

    // Bytes size of this WireMsg taking into account current self-contained payload.
    fn size(&self) -> usize {
        WireMsgHeader::size() + self.payload.len()
    }
}

// Header to be serialisied at the front of the wire message.
// This header contains the information needed to deserialise the payload.
#[derive(Debug)]
struct WireMsgHeader {
    header_size: u16,
    version: u16,
}

impl WireMsgHeader {
    // Instantiate a WireMsgHeader as per current supported version.
    pub fn new() -> Self {
        Self {
            header_size: Self::size() as u16,
            version: MESSAGING_PROTO_VERSION,
        }
    }

    // Parses the provided bytes to deserialise a WireMsgHeader,
    // returning the created WireMsgHeader, as well as the remaining bytes which
    // correspond to the message payload. The caller shall then take care of
    // deserialising the payload using the information provided in the WireMsgHeader.
    pub fn from(mut bytes: Bytes) -> Result<(Self, Bytes)> {
        // We need at least 4 bytes as current version 1 only has
        // a header size field and the protocol version. Thus, let's
        // make sure there is this number of bytes as a minimum.
        if bytes.len() < Self::size() {
            return Err(Error::FailedToParse(
                "not enough bytes received to deserialise wire message header".to_string(),
            ));
        }

        // Let's read the bytes which gives us the header size
        let mut header_size_bytes = [0; 2];
        header_size_bytes[0..].copy_from_slice(&bytes[0..2]);
        let header_size = u16::from_be_bytes(header_size_bytes);

        // ...now let's read the serialisation protocol version bytes
        let mut version_bytes = [0; 2];
        version_bytes[0..].copy_from_slice(&bytes[2..4]);
        let version = u16::from_be_bytes(version_bytes);
        // Make sure we support this version
        if version != MESSAGING_PROTO_VERSION {
            return Err(Error::UnsupportedVersion(version));
        }

        let header = Self {
            header_size,
            version,
        };

        // Get a slice for the payload bytes, i.e. the bytes after the header bytes
        let payload_bytes = bytes.split_off(header_size.into());

        Ok((header, payload_bytes))
    }

    // Size in bytes of WireMsgHeader.
    pub fn size() -> usize {
        size_of::<Self>()
    }
}
