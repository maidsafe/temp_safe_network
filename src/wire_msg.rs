// Copyright 2021MaidSafe.net limited.
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

/// Current version of the messaging protocol
const MESSAGING_PROTO_VERSION: u16 = 1u16;

#[derive(Debug)]
pub struct WireMsg {
    hdr: WireMsgHdr,
    payload: Vec<u8>,
}

impl WireMsg {
    pub fn new(msg: &MsgEnvelope, serialisation: PayloadSerType) -> Result<WireMsg> {
        let payload = match serialisation {
            PayloadSerType::Json => serde_json::to_string(msg).map_err(|err| {
                Error::Serialisation(format!(
                    "Could not serialize message payload (id: {}): {}",
                    msg.id(),
                    err
                ))
            })?,
            PayloadSerType::Bincode => "bincode".to_string(), // TODO
        };

        let hdr = WireMsgHdr {
            version: MESSAGING_PROTO_VERSION,
            payload_ser: serialisation,
        };

        Ok(Self {
            hdr,
            payload: payload.as_bytes().to_vec(),
        })
    }

    pub fn from(buf: &[u8]) -> Result<Self> {
        // Let's read the servialisation protocol version bytes first
        let mut version_bytes = [0; 2];
        version_bytes[0..].copy_from_slice(&buf[0..2]);
        // TODO: return error if version is not supported
        let version = u16::from_be_bytes(version_bytes);

        // ...now read the type of serialisation used for the payload
        let mut ser_type_bytes = [0; 2];
        ser_type_bytes[0..].copy_from_slice(&buf[2..4]);
        let payload_ser = PayloadSerType::from(u16::from_be_bytes(ser_type_bytes))?;

        // ...and finally read the bytes of the serialised payload
        let payload = buf[4..].to_vec();

        // We can now create a deserialised WireMsg using the read bytes
        Ok(Self {
            hdr: WireMsgHdr {
                version,
                payload_ser,
            },
            payload,
        })
    }

    pub fn size(&self) -> usize {
        WireMsgHdr::size() + self.payload.len()
    }

    pub fn serialise(&self) -> Vec<u8> {
        // First we create a buffer with the exact size
        // needed to serialise the wire msg
        let mut buf = vec![0u8; self.size()];

        // Let's write the serialisation protocol version bytes first
        let (buf2, _) = gen(be_u16(self.hdr.version), &mut buf[..]).unwrap();
        // ...then we write the type of servialisation used for the payload
        let (buf3, _) = gen(be_u16(self.hdr.payload_ser.into()), &mut buf2[..]).unwrap();
        // ...and finally we write the bytes of the serialised payload
        let (_, _) = gen(slice(self.payload.clone()), &mut buf3[..]).unwrap();

        // We can now return the buffer containing the written bytes
        buf
    }

    pub fn deserialise(&self) -> Result<MsgEnvelope> {
        match self.hdr.payload_ser {
            PayloadSerType::Json => serde_json::from_str(
                std::str::from_utf8(&self.payload)
                    .map_err(|err| Error::FailedToParse(err.to_string()))?,
            )
            .map_err(|err| Error::FailedToParse(err.to_string())),
            PayloadSerType::Bincode => Err(Error::FailedToParse("blabla".to_string())), // TODO
        }
    }
}

/// Header to be serialisied at the front of the wire message
#[derive(Debug)]
struct WireMsgHdr {
    version: u16,
    payload_ser: PayloadSerType,
}

impl WireMsgHdr {
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Type of serialisation used for the payload in a message
#[derive(Debug, Clone, Copy)]
pub enum PayloadSerType {
    Json,
    Bincode,
}

impl From<PayloadSerType> for u16 {
    fn from(s: PayloadSerType) -> u16 {
        match s {
            PayloadSerType::Json => 0,
            PayloadSerType::Bincode => 1,
        }
    }
}

impl PayloadSerType {
    fn from(t: u16) -> Result<Self> {
        match t {
            0 => Ok(PayloadSerType::Json),
            1 => Ok(PayloadSerType::Bincode),
            other => Err(Error::FailedToParse(format!(
                "Invalid payload serialiation type: {}",
                other
            ))),
        }
    }
}
