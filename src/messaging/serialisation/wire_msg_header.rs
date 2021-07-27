// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{DstLocation, Error, MessageId, MsgKind, Result};
use bincode::config::{Options, WithOtherEndian, WithOtherTrailing};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, mem::size_of};

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

// The header skeleton lets us deserialize and check the version and header size before
// deserializing the rest of the contents. This defines the outer message format (serialized with
// bincode).
#[derive(Deserialize, Serialize)]
struct HeaderSkeleton<'buf> {
    version: u16,
    #[serde(with = "serde_bytes")]
    header_bytes: &'buf [u8],
}

// Maximum header size. If header ends up larger, serialization will fail.
const HDR_CONTENT_MAX_SIZE: usize = u16::MAX as usize;

lazy_static::lazy_static! {
    static ref BINCODE_OPTS: WithOtherEndian<
        WithOtherTrailing<bincode::DefaultOptions, bincode::config::AllowTrailing>,
        bincode::config::BigEndian,
    > = {
        use bincode::Options;
        bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .with_big_endian()
    };

    static ref HDR_MAX_SIZE: u16 = {
        BINCODE_OPTS
            .serialized_size(&u16::MAX)
            .expect("serialize u16 should be infallible") as u16
            + BINCODE_OPTS
                .serialized_size(&HDR_CONTENT_MAX_SIZE)
                .expect("serialize usize should be infallible") as u16
            + size_of::<MsgEnvelope>() as u16
    };
}

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
        // Deserialize the header skeleton
        let skeleton: HeaderSkeleton = BINCODE_OPTS
            .deserialize(&bytes)
            .map_err(|err| Error::FailedToParse(format!("invalid message header: {}", err)))?;

        // Make sure we support this version
        if skeleton.version != MESSAGING_PROTO_VERSION {
            return Err(Error::UnsupportedVersion(skeleton.version));
        }

        // Check that the header's not too large
        if skeleton.header_bytes.len() > HDR_CONTENT_MAX_SIZE {
            return Err(Error::FailedToParse(format!(
                "header too large to deserialize: {}",
                skeleton.header_bytes.len()
            )));
        }

        // ...finally, we read the message envelope bytes
        let msg_envelope: MsgEnvelope =
            rmp_serde::from_slice(skeleton.header_bytes).map_err(|err| {
                Error::FailedToParse(format!(
                    "source authority couldn't be deserialized from the header: {}",
                    err
                ))
            })?;

        let header = Self {
            //header_size,
            version: skeleton.version,
            msg_envelope,
        };

        // Get a slice for the payload bytes, i.e. the bytes after the header bytes
        let serialized_size = BINCODE_OPTS.serialized_size(&skeleton).map_err(|err| {
            Error::FailedToParse(format!("failed to calculate header size: {}", err))
        })?;
        let payload_bytes = bytes.split_off(serialized_size as usize);

        Ok((header, payload_bytes))
    }

    pub fn write<'a>(&self, mut buffer: &'a mut [u8]) -> Result<(&'a mut [u8], u16)> {
        // first serialise the msg envelope so we can figure out the total header size
        let msg_envelope_vec = rmp_serde::to_vec_named(&self.msg_envelope).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize message envelope with Msgpack: {}",
                err
            ))
        })?;

        if msg_envelope_vec.len() > HDR_CONTENT_MAX_SIZE {
            return Err(Error::Serialisation(format!(
                "header too large to serialise: {}",
                msg_envelope_vec.len()
            )));
        }

        let buffer_len = buffer.len();

        let skeleton = HeaderSkeleton {
            version: self.version,
            header_bytes: &msg_envelope_vec,
        };
        BINCODE_OPTS
            .serialize_into(&mut buffer, &skeleton)
            .map_err(|err| Error::Serialisation(format!("failed to serialize header: {}", err)))?;

        let serialized_size = u16::try_from(buffer_len - buffer.len()).unwrap();
        Ok((buffer, serialized_size))
    }

    // Maximum size in bytes a WireMsgHeader can occupied when serialized.
    // this may be innaccurate since we use `size_of` which may not match the serialized form
    pub fn max_size() -> u16 {
        *HDR_MAX_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::{MsgEnvelope, WireMsgHeader, MESSAGING_PROTO_VERSION};
    use crate::messaging::{DstLocation, EndUser, MessageId, MsgKind};
    use bytes::Bytes;
    use std::convert::TryFrom;
    use xor_name::XorName;

    #[test]
    fn serialization() {
        let header = WireMsgHeader {
            version: MESSAGING_PROTO_VERSION,
            msg_envelope: MsgEnvelope {
                msg_id: MessageId::new(),
                msg_kind: MsgKind::SectionInfoMsg,
                dst_location: DstLocation::EndUser(EndUser {
                    xorname: XorName::random(),
                    socket_id: XorName::random(),
                }),
            },
        };
        let serialized_envelope = rmp_serde::to_vec_named(&header.msg_envelope).unwrap();

        let expected_size = u16::try_from(2 + serialized_envelope.len()).unwrap();
        let mut expected = vec![
            MESSAGING_PROTO_VERSION as u8,
            serialized_envelope.len() as u8,
        ];
        expected.extend(&serialized_envelope);

        let mut actual = vec![0; WireMsgHeader::max_size().into()];
        let (_, header_size) = header.write(&mut actual).unwrap();

        assert_eq!(header_size, expected_size);
        assert_eq!(&actual[..header_size.into()], &expected);

        assert_eq!(WireMsgHeader::from(Bytes::from(actual)).unwrap().0, header);
    }
}
