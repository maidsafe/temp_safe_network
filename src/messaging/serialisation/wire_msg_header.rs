// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{DstLocation, Error, MessageId, MsgKind, Result};
use bytes::Bytes;
use cookie_factory::{bytes::be_u16, combinator::slice, gen, gen_simple};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, mem::size_of};

// Current version of the messaging protocol.
// At this point this implementation supports only this version.
const MESSAGING_PROTO_VERSION: u16 = 1u16;

// Header to be serialisied at the front of the wire message.
// This header contains the information needed to deserialize the payload.
#[derive(Debug, PartialEq, Clone)]
pub struct WireMsgHeader {
    // We serialise a header size field, but we don't know it up front until we serialise it.
    // header_size: u16,
    version: u16,
    pub msg_envelope: MsgEnvelope,
}

// The message envelope contains the ID of the message, the authority
// signing this message (if any), as well as destination information
// This is all part of the message header, and it gets deserialize
// when the `WireMsgHeader` is deserialise, allowing the caller to read
// all this information before deciding to deserialise the actual message payload.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MsgEnvelope {
    pub msg_id: MessageId,
    pub msg_kind: MsgKind,
    pub dst_location: DstLocation,
}

// Bytes length in the header for the 'header_size' field
const HDR_SIZE_BYTES_LEN: usize = size_of::<u16>();

// Bytes index and size in the header for the 'version' field
const HDR_VERSION_BYTES_START: usize = HDR_SIZE_BYTES_LEN;
const HDR_VERSION_BYTES_LEN: usize = size_of::<u16>();
const HDR_VERSION_BYTES_END: usize = HDR_VERSION_BYTES_START + HDR_VERSION_BYTES_LEN;

impl WireMsgHeader {
    // Instantiate a WireMsgHeader as per current supported version.
    pub fn new(msg_id: MessageId, msg_kind: MsgKind, dst_location: DstLocation) -> Self {
        Self {
            //header_size: Self::max_size(),
            version: MESSAGING_PROTO_VERSION,
            msg_envelope: MsgEnvelope {
                msg_id,
                msg_kind,
                dst_location,
            },
        }
    }

    // Parses the provided bytes to deserialize a WireMsgHeader,
    // returning the created WireMsgHeader, as well as the remaining bytes which
    // correspond to the message payload. The caller shall then take care of
    // deserializing the payload using the information provided in the `WireMsgHeader`.
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

        // We check that at least we have the minimum number of bytes
        // for the header of any kind of message to be deserialised.
        if length < header_size.into() {
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

        // ...finally, we read the message envelope bytes
        let msg_envelope_bytes = &bytes[HDR_VERSION_BYTES_END..header_size.into()];
        let msg_envelope: MsgEnvelope =
            rmp_serde::from_slice(msg_envelope_bytes).map_err(|err| {
                Error::FailedToParse(format!(
                    "source authority couldn't be deserialized from the header: {}",
                    err
                ))
            })?;

        let header = Self {
            //header_size,
            version,
            msg_envelope,
        };

        // Get a slice for the payload bytes, i.e. the bytes after the header bytes
        let payload_bytes = bytes.split_off(header_size.into());

        Ok((header, payload_bytes))
    }

    pub fn write<'a>(&self, buffer: &'a mut [u8]) -> Result<(&'a mut [u8], u16)> {
        // first serialise the msg envelope so we can figure out the total header size
        let msg_envelope_vec = rmp_serde::to_vec_named(&self.msg_envelope).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize message envelope with Msgpack: {}",
                err
            ))
        })?;

        // real header size based on the length of serialised msg envelope
        let header_size =
            (HDR_SIZE_BYTES_LEN + HDR_VERSION_BYTES_LEN + msg_envelope_vec.len()) as u16;

        // Let's write the header size first
        let (buf_at_version, _) = gen(be_u16(header_size), buffer).map_err(|err| {
            Error::Serialisation(format!(
                "header size value couldn't be serialized into the header: {}",
                err
            ))
        })?;

        // Now let's write the serialisation protocol version bytes
        let (buf_at_msg_envelope, _) =
            gen(be_u16(self.version), buf_at_version).map_err(|err| {
                Error::Serialisation(format!(
                    "version field couldn't be serialized into the header: {}",
                    err
                ))
            })?;

        // ...now write the message envelope
        let buf_at_payload =
            gen_simple(slice(&msg_envelope_vec), buf_at_msg_envelope).map_err(|err| {
                Error::Serialisation(format!(
                    "message envelope couldn't be serialized into the header: {}",
                    err
                ))
            })?;

        Ok((buf_at_payload, header_size))
    }

    // Maximum size in bytes a WireMsgHeader can occupied when serialized.
    pub fn max_size() -> u16 {
        // We don't use 'std::mem::size_of' since we don't necesserally
        // serialise them in the same way as they are represented in this struct.
        (HDR_SIZE_BYTES_LEN + HDR_VERSION_BYTES_LEN + size_of::<MsgEnvelope>()) as u16
    }
}
