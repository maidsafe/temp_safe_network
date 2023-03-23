// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::wire_msg_header::WireMsgHeader;

use crate::messaging::{
    data::{ClientMsg, DataResponse},
    system::NodeMsg,
    AntiEntropyMsg, AuthorityProof, Dst, Error, MsgId, MsgKind, NetworkMsg, Result,
};

use bytes::{BufMut, Bytes, BytesMut};
use custom_debug::Debug;
use serde::Serialize;

/// In order to send a message over the wire, it needs to be serialized
/// along with a header (`WireMsgHeader`) which contains the information needed
/// by the recipient to properly deserialize it.
/// The `WireMsg` struct provides the utilities to serialize and deserialize messages.
#[derive(Clone, Debug)]
pub struct WireMsg {
    /// Message header
    pub header: WireMsgHeader,
    #[debug(skip)]
    /// Serialized Message header
    pub serialized_header: Option<Bytes>,
    /// Serialised message
    #[debug(skip)]
    pub payload: Bytes,
    /// The target dst
    pub dst: Dst,
    #[debug(skip)]
    /// Serialized Message dst
    pub serialized_dst: Option<Bytes>,
}

impl PartialEq for WireMsg {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header && self.payload == other.payload
    }
}

impl WireMsg {
    /// Serializes the message provided. This function shall be used to
    /// obtain the serialized payload which is needed to create the `MsgKind`.
    /// Once the caller obtains both the serialized payload and `MsgKind`,
    /// it can invoke the `new_msg` function to instantiate a `WireMsg`.
    pub fn serialize_msg_payload<T: Serialize>(msg: &T) -> Result<Bytes> {
        let mut bytes = BytesMut::new().writer();
        rmp_serde::encode::write(&mut bytes, &msg).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize message payload with Msgpack: {err}",
            ))
        })?;

        Ok(bytes.into_inner().freeze())
    }
    /// Serializes the dst provided.
    fn serialize_dst_payload(dst: &Dst) -> Result<Bytes> {
        let mut bytes = BytesMut::new().writer();

        rmp_serde::encode::write(&mut bytes, dst).map_err(|err| {
            Error::Serialisation(format!(
                "could not serialize dst payload with Msgpack: {err}",
            ))
        })?;

        Ok(bytes.into_inner().freeze())
    }
    /// Serializes the dst on the WireMsg
    pub fn serialize_msg_dst(&self) -> Result<Bytes> {
        Self::serialize_dst_payload(&self.dst)
    }

    /// Creates a new `WireMsg` with the provided serialized payload and `MsgKind`.
    pub fn new_msg(msg_id: MsgId, payload: Bytes, kind: MsgKind, dst: Dst) -> Self {
        Self {
            header: WireMsgHeader::new(msg_id, kind),
            dst,
            payload,
            serialized_dst: None,
            serialized_header: None,
        }
    }

    /// Attempts to create an instance of `WireMsg` by deserialising the bytes provided.
    /// To succeed, the bytes should contain at least a valid `WireMsgHeader`.
    pub fn from(bytes: (Bytes, Bytes, Bytes)) -> Result<Self> {
        let (header_bytes, dst_bytes, payload) = bytes;
        // Deserialize the header bytes first
        let header = WireMsgHeader::from(header_bytes.clone())?;
        let dst: Dst = rmp_serde::from_slice(&dst_bytes).map_err(|err| {
            Error::FailedToParse(format!(
                "Message dst couldn't be deserialized from the dst bytes: {err}",
            ))
        })?;

        // We can now create a deserialized WireMsg using the read bytes
        Ok(Self {
            header,
            dst,
            payload,
            serialized_dst: Some(dst_bytes),
            serialized_header: Some(header_bytes),
        })
    }

    /// Return the serialized `WireMsgHeader`, the Dst and the Payload bytes contained
    /// on the WireMsg
    pub fn serialize(&self) -> Result<(Bytes, Bytes, Bytes)> {
        let header = if let Some(bytes) = &self.serialized_header {
            bytes.clone()
        } else {
            // First we create a buffer with the capacity
            // needed to serialize the wire msg
            self.header.serialize()?
        };

        let dst = if let Some(bytes) = &self.serialized_dst {
            bytes.clone()
        } else {
            self.serialize_msg_dst()?
        };

        // We can now return the buffer containing the written bytes
        Ok((header, dst, self.payload.clone()))
    }

    /// Return the serialized `WireMsgHeader`, the Dst and the Payload bytes
    /// Caching the bytes to the WireMsg itself
    pub fn serialize_and_cache_bytes(&mut self) -> Result<(Bytes, Bytes, Bytes)> {
        // if we've already serialized, grab those header bytes
        let header = if let Some(hdr_bytes) = &self.serialized_header {
            hdr_bytes.clone()
        } else {
            let hdr_bytes = self.header.serialize()?;
            self.serialized_header = Some(hdr_bytes.clone());
            hdr_bytes
        };

        let dst = if let Some(dst_bytes) = &self.serialized_dst {
            dst_bytes.clone()
        } else {
            let dst_bytes = self.serialize_msg_dst()?;
            self.serialized_dst = Some(dst_bytes.clone());
            dst_bytes
        };

        Ok((header, dst, self.payload.clone()))
    }

    /// Return the serialized `WireMsg`, which contains the `WireMsgHeader` bytes,
    /// followed by the provided dst and payload bytes, i.e. the serialized Message.
    pub fn serialize_with_new_dst(&self, dst: &Dst) -> Result<(Bytes, Bytes, Bytes)> {
        // if we've already serialized, grab those header bytes
        let header = if let Some(bytes) = &self.serialized_header {
            bytes.clone()
        } else {
            self.header.serialize()?
        };

        let dst = Self::serialize_dst_payload(dst)?;

        Ok((header, dst, self.payload.clone()))
    }

    /// Deserialize the payload from this `WireMsg` returning a `NetworkMsg` instance.
    pub fn into_msg(&self) -> Result<NetworkMsg> {
        match self.header.msg_envelope.kind.clone() {
            MsgKind::AntiEntropy(_) => {
                let msg: AntiEntropyMsg = rmp_serde::from_slice(&self.payload).map_err(|err| {
                    Error::FailedToParse(format!("Ae message payload as Msgpack: {err}"))
                })?;
                Ok(NetworkMsg::AntiEntropy(msg))
            }
            MsgKind::Client { auth, .. } => {
                let msg: ClientMsg = rmp_serde::from_slice(&self.payload).map_err(|err| {
                    Error::FailedToParse(format!("Data message payload as Msgpack: {err}"))
                })?;

                let auth = AuthorityProof::verify(auth, &self.payload)?;

                Ok(NetworkMsg::Client { auth, msg })
            }
            MsgKind::DataResponse(_) => {
                let msg: DataResponse = rmp_serde::from_slice(&self.payload).map_err(|err| {
                    Error::FailedToParse(format!("Data message payload as Msgpack: {err}"))
                })?;
                Ok(NetworkMsg::DataResponse(msg))
            }
            MsgKind::Node { .. } => {
                let msg: NodeMsg = rmp_serde::from_slice(&self.payload).map_err(|err| {
                    Error::FailedToParse(format!("Node signed message payload as Msgpack: {err}"))
                })?;
                Ok(NetworkMsg::Node(msg))
            }
        }
    }

    /// Return the message id of this message
    pub fn msg_id(&self) -> MsgId {
        self.header.msg_envelope.msg_id
    }

    /// Return the auth of this message
    pub fn kind(&self) -> &MsgKind {
        &self.header.msg_envelope.kind
    }

    /// Return the dst of this msg
    pub fn dst(&self) -> &Dst {
        &self.dst
    }

    /// Convenience function which creates a temporary `WireMsg` from the provided
    /// bytes, returning the deserialized message.
    pub fn deserialize(bytes: (Bytes, Bytes, Bytes)) -> Result<(MsgId, NetworkMsg)> {
        let msg = Self::from(bytes)?;
        Ok((msg.msg_id(), msg.into_msg()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        messaging::{
            data::{ClientMsg, DataQuery},
            system::NodeMsg,
            AuthorityProof, ClientAuth, MsgId,
        },
        types::{ChunkAddress, Keypair},
    };
    use bls::SecretKey;
    use eyre::Result;

    #[test]
    fn serialisation_node_msg() -> Result<()> {
        let dst = Dst {
            name: xor_name::rand::random(),
            section_key: SecretKey::random().public_key(),
        };

        let msg_id = MsgId::new();

        let msg = NodeMsg::HandoverAE(100);

        let payload = WireMsg::serialize_msg_payload(&msg)?;
        let kind = MsgKind::Node {
            name: Default::default(),
            is_join: true,
        };
        let wire_msg = WireMsg::new_msg(msg_id, payload, kind, dst);
        let serialized = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dst(), &dst);
        assert_eq!(deserialized.dst().section_key, dst.section_key);

        // test deserialisation of payload
        assert_eq!(deserialized.into_msg()?, NetworkMsg::Node(msg),);

        Ok(())
    }

    #[test]
    fn serialisation_client_msg() -> Result<()> {
        let src_client_keypair = Keypair::new_ed25519();

        let dst = Dst {
            name: xor_name::rand::random(),
            section_key: SecretKey::random().public_key(),
        };

        let msg_id = MsgId::new();

        let client_msg =
            ClientMsg::Query(DataQuery::GetChunk(ChunkAddress(xor_name::rand::random())));

        let payload = WireMsg::serialize_msg_payload(&client_msg)?;
        let auth = ClientAuth {
            public_key: src_client_keypair.public_key(),
            signature: src_client_keypair.sign(&payload),
        };
        let auth_proof = AuthorityProof::verify(auth.clone(), &payload)?;
        let kind = MsgKind::Client {
            auth,
            is_spend: false,
            query_index: None,
        };

        let wire_msg = WireMsg::new_msg(msg_id, payload, kind, dst);
        let serialized = wire_msg.serialize()?;

        // test deserialisation of header
        let deserialized = WireMsg::from(serialized)?;
        assert_eq!(deserialized, wire_msg);
        assert_eq!(deserialized.msg_id(), wire_msg.msg_id());
        assert_eq!(deserialized.dst(), &dst);
        assert_eq!(deserialized.dst().section_key, dst.section_key);

        // test deserialisation of payload
        assert_eq!(
            deserialized.into_msg()?,
            NetworkMsg::Client {
                auth: auth_proof,
                msg: client_msg,
            }
        );

        Ok(())
    }
}
