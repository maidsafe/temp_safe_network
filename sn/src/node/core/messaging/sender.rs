// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::messaging::{
    data::{CmdError, ServiceMsg},
    DstLocation, EndUser, MsgId, MsgKind, ServiceAuth, WireMsg,
};
use crate::node::{
    api::cmds::Cmd, core::delivery_group, error::Result, network_knowledge::NetworkKnowledge,
    NodeInfo,
};
use crate::types::{Peer, PublicKey, Signature};

use bytes::Bytes;
use signature::Signer;
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) struct MsgSender {
    network_knowledge: NetworkKnowledge,
    our_info: Arc<RwLock<NodeInfo>>,
}

impl MsgSender {
    pub(crate) async fn new(
        network_knowledge: NetworkKnowledge,
        our_info: Arc<RwLock<NodeInfo>>,
    ) -> Self {
        Self {
            network_knowledge,
            our_info,
        }
    }

    /// Forms a CmdError msg to send back to the client
    pub(crate) async fn send_cmd_error_response(
        &self,
        error: CmdError,
        target: Peer,
        msg_id: MsgId,
    ) -> Result<Vec<Cmd>> {
        let the_error_msg = ServiceMsg::CmdError {
            error,
            correlation_id: msg_id,
        };
        self.send_cmd_response(target, the_error_msg).await
    }

    /// Forms a CmdAck msg to send back to the client
    pub(crate) async fn send_cmd_ack(&self, target: Peer, msg_id: MsgId) -> Result<Vec<Cmd>> {
        let the_ack_msg = ServiceMsg::CmdAck {
            correlation_id: msg_id,
        };
        self.send_cmd_response(target, the_ack_msg).await
    }

    /// Currently using node's Ed key. May need to use bls key share for concensus purpose.
    pub(crate) async fn ed_sign_service_msg(&self, msg: &ServiceMsg) -> Result<(MsgKind, Bytes)> {
        let keypair = self.our_info.read().await.keypair.clone();
        let payload = WireMsg::serialize_msg_payload(msg)?;
        let signature = keypair.sign(&payload);

        let msg = MsgKind::ServiceMsg(ServiceAuth {
            public_key: PublicKey::Ed25519(keypair.public),
            signature: Signature::Ed25519(signature),
        });

        Ok((msg, payload))
    }

    // Send message to peers on the network.
    pub(crate) async fn send_msg_to_nodes(&self, mut wire_msg: WireMsg) -> Result<Option<Cmd>> {
        let dst_location = wire_msg.dst_location();
        let (targets, dg_size) = delivery_group::delivery_targets(
            dst_location,
            &self.our_info.read().await.name(),
            &self.network_knowledge,
        )
        .await?;

        trace!(
            "relay {:?} to first {:?} of {:?} (Section PK: {:?})",
            wire_msg,
            dg_size,
            targets,
            wire_msg.src_section_pk(),
        );

        let target_name = dst_location.name();

        // To avoid loop: if destination is to Node, targets are multiple, self is an elder,
        //     self section prefix matches the destination name, then don't carry out a relay.
        if self.is_elder().await
            && targets.len() > 1
            && dst_location.is_to_node()
            && self.network_knowledge.prefix().await.matches(&target_name)
        {
            // This actually means being an elder, but we don't know the member yet. Which most likely
            // happens during the join process that a node's name is changed.
            // we just drop the message
            return Ok(None);
        }

        let dst_pk = self
            .network_knowledge
            .section_key_by_name(&target_name)
            .await;
        wire_msg.set_dst_section_pk(dst_pk);

        let cmd = Cmd::SendMsgDeliveryGroup {
            recipients: targets.into_iter().collect(),
            delivery_group_size: dg_size,
            wire_msg,
        };

        Ok(Some(cmd))
    }

    /// Forms a cmd to send a cmd response error/ack to the client
    async fn send_cmd_response(&self, target: Peer, msg: ServiceMsg) -> Result<Vec<Cmd>> {
        let dst = DstLocation::EndUser(EndUser(target.name()));

        let (msg_kind, payload) = self.ed_sign_service_msg(&msg).await?;
        let wire_msg = WireMsg::new_msg(MsgId::new(), payload, msg_kind, dst)?;

        let cmd = Cmd::SendMsg {
            recipients: vec![target],
            wire_msg,
        };

        Ok(vec![cmd])
    }

    async fn is_elder(&self) -> bool {
        self.network_knowledge
            .is_elder(&self.our_info.read().await.name())
            .await
    }
}
