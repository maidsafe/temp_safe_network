// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{Error, MsgEnvelope, Result};
use cookie_factory::{bytes::be_u16, combinator::slice, gen};
use std::fmt::Debug;

// Current version of the messaging protocol
const MESSAGING_PROTO_VERSION: u16 = 1u16;

#[derive(Debug)]
pub(crate) struct WireMsg {
    header: WireMsgHeader,
    payload: Vec<u8>,
}

impl WireMsg {
    pub fn new(msg: &MsgEnvelope, serialisation: PayloadSerialisationType) -> Result<WireMsg> {
        let payload = match serialisation {
            PayloadSerialisationType::Json => {
                let str = serde_json::to_string(msg).map_err(|err| {
                    Error::Serialisation(format!(
                        "Could not serialize message payload (id: {}) with Json: {}",
                        msg.id(),
                        err
                    ))
                })?;
                str.as_bytes().to_vec()
            }
            PayloadSerialisationType::Msgpack => rmp_serde::to_vec(&msg).map_err(|err| {
                Error::Serialisation(format!(
                    "Could not serialize message payload (id: {}) with Msgpack: {}",
                    msg.id(),
                    err
                ))
            })?,
        };

        let header = WireMsgHeader {
            version: MESSAGING_PROTO_VERSION,
            payload_serialisation: serialisation,
        };

        Ok(Self { header, payload })
    }

    pub fn from(bytes: &[u8]) -> Result<Self> {
        // Deserialise the header bytes firstg
        let header = WireMsgHeader::from(&bytes[..4])?;

        // Read the bytes of the serialised payload now
        let payload = bytes[4..].to_vec();

        // We can now create a deserialised WireMsg using the read bytes
        Ok(Self { header, payload })
    }

    /// Return the serialised WireMsg.
    pub fn serialise(&self) -> Result<Vec<u8>> {
        // First we create a buffer with the exact size
        // needed to serialise the wire msg
        let mut buffer = vec![0u8; self.size()];

        // Let's write the serialisation protocol version bytes first
        let (buf2, _) = gen(be_u16(self.header.version), &mut buffer[..]).map_err(|err| {
            Error::Serialisation(format!(
                "version field couldn't be serialised in header: {}",
                err
            ))
        })?;
        // ...then we write the type of serialisation used for the payload
        let (buf3, _) = gen(
            be_u16(self.header.payload_serialisation.into()),
            &mut buf2[..],
        )
        .map_err(|err| {
            Error::Serialisation(format!(
                "payload serialisation type value couldn't be serialised in header: {}",
                err
            ))
        })?;

        // ...and finally we write the bytes of the serialised payload
        let (_, _) = gen(slice(self.payload.clone()), &mut buf3[..]).map_err(|err| {
            Error::Serialisation(format!("message payload couldn't be serialised: {}", err))
        })?;

        // We can now return the buffer containing the written bytes
        Ok(buffer)
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// MsgEnvelope and serialisation type, returning the serialised WireMsg.
    pub fn serialise_msg(
        msg: &MsgEnvelope,
        serialisation: PayloadSerialisationType,
    ) -> Result<Vec<u8>> {
        Self::new(msg, serialisation)?.serialise()
    }

    /// Deserialise the payload returning a MsgEnvelope instance
    pub fn deserialise(&self) -> Result<MsgEnvelope> {
        match self.header.payload_serialisation {
            PayloadSerialisationType::Json => {
                serde_json::from_str(std::str::from_utf8(&self.payload).map_err(|err| {
                    Error::FailedToParse(format!("message payload as Json: {}", err))
                })?)
                .map_err(|err| Error::FailedToParse(err.to_string()))
            }
            PayloadSerialisationType::Msgpack => {
                rmp_serde::from_slice(&self.payload).map_err(|err| {
                    Error::FailedToParse(format!("message payload as Msgpack: {}", err))
                })?
            }
        }
    }

    /// Convenience function which creates a temporary WireMsg from the provided
    /// bytes, returning the deserialised MsgEnvelope.
    pub fn deserialise_msg(bytes: &[u8]) -> Result<MsgEnvelope> {
        Self::from(bytes)?.deserialise()
    }

    // Bytes size of this instance with current content
    fn size(&self) -> usize {
        WireMsgHeader::size() + self.payload.len()
    }
}

/// Header to be serialisied at the front of the wire message
#[derive(Debug)]
struct WireMsgHeader {
    version: u16,
    payload_serialisation: PayloadSerialisationType,
}

impl WireMsgHeader {
    pub fn from(bytes: &[u8]) -> Result<Self> {
        // Let's read the serialisation protocol version bytes first
        let mut version_bytes = [0; 2];
        version_bytes[0..].copy_from_slice(&bytes[0..2]);
        let version = u16::from_be_bytes(version_bytes);
        // make sure we support this version
        if version != MESSAGING_PROTO_VERSION {
            return Err(Error::UnsupportedVersion(version));
        }

        // ...now read the type of serialisation used for the payload
        let mut serialisation_type_bytes = [0; 2];
        serialisation_type_bytes[0..].copy_from_slice(&bytes[2..4]);
        let payload_serialisation =
            PayloadSerialisationType::from(u16::from_be_bytes(serialisation_type_bytes))?;

        Ok(Self {
            version,
            payload_serialisation,
        })
    }

    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Type of serialisation used for the payload in a message
#[derive(Debug, Clone, Copy)]
pub enum PayloadSerialisationType {
    Json,
    Msgpack,
}

impl From<PayloadSerialisationType> for u16 {
    fn from(t: PayloadSerialisationType) -> u16 {
        match t {
            PayloadSerialisationType::Json => 0,
            PayloadSerialisationType::Msgpack => 1,
        }
    }
}

impl PayloadSerialisationType {
    fn from(t: u16) -> Result<Self> {
        match t {
            0 => Ok(PayloadSerialisationType::Json),
            1 => Ok(PayloadSerialisationType::Msgpack),
            other => Err(Error::UnsupportedSerialisation(other)),
        }
    }
}
