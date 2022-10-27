// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{messaging::Peers, Cmd, Error, MyNode, Prefix, Result};

use bytes::Bytes;
use itertools::Itertools;
use qp2p::SendStream;
#[cfg(feature = "traceroute")]
use sn_interface::messaging::{Traceroute, WireMsg};
use sn_interface::{
    data_copy_count,
    messaging::{
        data::{DataQuery, MetadataExchange, StorageLevel},
        system::{NodeCmd, NodeMsg, NodeQuery, OperationId},
        AuthorityProof, ClientAuth, Dst, MsgId, MsgKind,
    },
    types::{log_markers::LogMarker, Peer, PublicKey, ReplicatedData},
};
use std::{cmp::Ordering, collections::BTreeSet, sync::Arc};
use tokio::sync::Mutex;
use tracing::info;
use xor_name::XorName;

impl MyNode {
    // Locate ideal holders for this data, instruct them to store the data
    pub(crate) fn replicate_data(
        &self,
        data: ReplicatedData,
        targets: BTreeSet<Peer>,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Cmd {
        info!(
            "Replicating data {:?} to holders {:?}",
            data.name(),
            &targets,
        );

        self.trace_system_msg(
            NodeMsg::NodeCmd(NodeCmd::ReplicateData(vec![data])),
            Peers::Multiple(targets),
            #[cfg(feature = "traceroute")]
            traceroute,
        )
    }

    /// Find target adult, sends a bidi msg, awaiting response, and then sends this on to the client
    pub(crate) async fn read_data_from_adults(
        &self,
        query: DataQuery,
        msg_id: MsgId,
        auth: AuthorityProof<ClientAuth>,
        source_client: Peer,
        client_response_stream: Option<Arc<Mutex<SendStream>>>,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Result<Vec<Cmd>> {
        // We generate the operation id to track the response from the Adult
        // by using the query msg id, which shall be unique per query.
        let operation_id = OperationId::from(&Bytes::copy_from_slice(msg_id.as_ref()));
        let address = query.variant.address();
        trace!(
            "{:?} preparing to query adults for data at {:?} with op_id: {:?}",
            LogMarker::DataQueryReceviedAtElder,
            address,
            operation_id
        );

        let targets = self.target_data_holders_including_full(address.name());

        // Query only the nth adult
        let target = if let Some(peer) = targets.iter().nth(query.adult_index) {
            *peer
        } else {
            debug!("No targets found for {msg_id:?}");
            let error = Error::InsufficientAdults {
                prefix: self.network_knowledge().prefix(),
                expected: query.adult_index as u8 + 1,
                found: targets.len() as u8,
            };

            self.query_error_response(
                error,
                &query.variant,
                source_client,
                msg_id,
                client_response_stream,
                #[cfg(feature = "traceroute")]
                traceroute,
            )
            .await?;
            // TODO: do error processing
            return Ok(vec![]);
        };

        // Form a msg to our adult
        let msg = NodeMsg::NodeQuery(NodeQuery::Data {
            query: query.variant,
            auth: auth.into_inner(),
            operation_id,
        });

        let (kind, payload) = self.serialize_node_msg(msg)?;

        let (bytes_to_adult, msg_id) = self.form_usr_msg_bytes_to_node(
            payload,
            kind,
            target,
            #[cfg(feature = "traceroute")]
            traceroute.clone(),
        )?;

        let cmd = Cmd::SendToAdultsAndReturnToClient {
            msg_id,
            operation_id,
            target,
            bytes_to_adult,
            client_response_stream,
            #[cfg(feature = "traceroute")]
            traceroute,
        };

        Ok(vec![cmd])
    }

    /// Send an OutgoingMsg on a given stream
    pub(crate) async fn send_msg_on_stream(
        &self,
        payload: Bytes,
        kind: MsgKind,
        send_stream: Arc<Mutex<SendStream>>,
        source_peer: Peer,
        original_msg_id: MsgId,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Result<()> {
        // TODO why do we need dst here?
        let (bytes, _new_msg_id) = self.form_usr_msg_bytes_to_node(
            payload,
            kind,
            source_peer,
            #[cfg(feature = "traceroute")]
            traceroute,
        )?;
        trace!("USING BIDI to send to client! OH DEAR, FASTEN SEATBELTS");
        let stream_prio = 10;
        let mut send_stream = send_stream.lock().await;

        debug!("stream locked");
        send_stream.set_priority(stream_prio);
        if let Err(error) = send_stream.send_user_msg(bytes).await {
            error!(
                        "Could not send query response {original_msg_id:?} to client {source_peer:?} over response stream: {error:?}",

                    );
        }
        if let Err(error) = send_stream.finish().await {
            error!(
                        "Could not close response stream for {original_msg_id:?} to client {source_peer:?}: {error:?}",
                    );
        }

        debug!("sent");

        Ok(())
    }

    pub(crate) fn form_usr_msg_bytes_to_node(
        &self,
        payload: Bytes,
        kind: MsgKind,
        target: Peer,
        #[cfg(feature = "traceroute")] traceroute: Traceroute,
    ) -> Result<(qp2p::UsrMsgBytes, MsgId)> {
        let msg_id = MsgId::new();
        // let (kind, payload) = self.serialize_node_msg(msg)?;
        // we first generate the XorName
        let dst = Dst {
            name: target.name(),
            section_key: self.network_knowledge().section_key(),
        };

        #[allow(unused_mut)]
        let mut wire_msg = WireMsg::new_msg(msg_id, payload, kind, dst);

        // TODO: do we still need traceroute now?
        #[cfg(feature = "traceroute")]
        {
            let mut traceroute = traceroute;
            traceroute.0.push(self.identity());
            wire_msg.append_trace(&mut traceroute);
        }

        #[cfg(feature = "test-utils")]
        let wire_msg = wire_msg.set_payload_debug(msg);

        Ok((
            wire_msg
                .serialize_and_cache_bytes()
                .map_err(|_| Error::InvalidMessage)?,
            msg_id,
        ))
    }

    pub(crate) fn get_metadata_of(&self, prefix: &Prefix) -> MetadataExchange {
        // Load tracked adult_levels
        let adult_levels = self.capacity.levels_matching(*prefix);
        MetadataExchange { adult_levels }
    }

    pub(crate) fn set_adult_levels(&mut self, adult_levels: MetadataExchange) {
        let MetadataExchange { adult_levels } = adult_levels;
        self.capacity.set_adult_levels(adult_levels)
    }

    /// Registered holders not present in provided list of members
    /// will be removed from `adult_storage_info` and no longer tracked for liveness.
    pub(crate) fn liveness_retain_only(&mut self, members: BTreeSet<XorName>) -> Result<()> {
        // full adults
        self.capacity.retain_members_only(&members);
        // stop tracking liveness of absent holders
        self.dysfunction_tracking.retain_members_only(members);
        Ok(())
    }

    /// Adds the new adult to the Capacity and Liveness trackers.
    pub(crate) fn add_new_adult_to_trackers(&mut self, adult: XorName) {
        info!("Adding new Adult: {adult} to trackers");
        self.capacity.add_new_adult(adult);
        self.dysfunction_tracking.add_new_node(adult);
    }

    /// Set storage level of a given node.
    /// Returns whether the level changed or not.
    pub(crate) fn set_storage_level(&mut self, node_id: &PublicKey, level: StorageLevel) -> bool {
        info!("Setting new storage level..");
        let changed = self
            .capacity
            .set_adult_level(XorName::from(*node_id), level);
        let avg_usage = self.capacity.avg_usage();
        info!(
            "Avg storage usage among Adults is between {}-{} %",
            avg_usage * 10,
            (avg_usage + 1) * 10
        );
        changed
    }

    pub(crate) fn full_adults(&self) -> BTreeSet<XorName> {
        self.capacity.full_adults()
    }

    /// Construct list of adults that hold target data, including full nodes.
    /// List is sorted by distance from `target`.
    fn target_data_holders_including_full(&self, target: &XorName) -> BTreeSet<Peer> {
        let full_adults = self.full_adults();
        let adults = self.network_knowledge().adults();

        let mut candidates = adults
            .clone()
            .into_iter()
            .sorted_by(|lhs, rhs| target.cmp_distance(&lhs.name(), &rhs.name()))
            .filter(|peer| !full_adults.contains(&peer.name()))
            .take(data_copy_count())
            .collect::<BTreeSet<_>>();

        trace!(
            "Data holders of {:?} are non-full adults: {:?} and full adults: {:?}",
            target,
            candidates,
            full_adults
        );

        // Full adults that are close to the chunk, shall still be considered as candidates
        // to allow chunks stored to non-full adults can be queried when nodes become full.
        let candidates_clone = candidates.clone();
        let close_full_adults = if let Some(closest_not_full) = candidates_clone.iter().next() {
            full_adults
                .iter()
                .filter_map(|name| {
                    if target.cmp_distance(name, &closest_not_full.name()) == Ordering::Less {
                        // get the actual peer if closer
                        let mut the_closer_peer = None;
                        for adult in &adults {
                            if &adult.name() == name {
                                the_closer_peer = Some(adult)
                            }
                        }
                        the_closer_peer
                    } else {
                        None
                    }
                })
                .collect::<BTreeSet<_>>()
        } else {
            // In case there is no empty candidates, query all full_adults
            adults
                .iter()
                .filter(|peer| !full_adults.contains(&peer.name()))
                .collect::<BTreeSet<_>>()
        };

        candidates.extend(close_full_adults);
        candidates
    }

    /// Used to fetch the list of holders for given name of data. Excludes full nodes
    pub(crate) fn target_data_holders(&self, target: XorName) -> BTreeSet<Peer> {
        let full_adults = self.full_adults();
        trace!("full_adults = {}", full_adults.len());
        // TODO: reuse our_adults_sorted_by_distance_to API when core is merged into upper layer
        let adults = self.network_knowledge().adults();

        trace!("Total adults known about: {:?}", adults.len());

        let candidates = adults
            .into_iter()
            .sorted_by(|lhs, rhs| target.cmp_distance(&lhs.name(), &rhs.name()))
            .filter(|peer| !full_adults.contains(&peer.name()))
            .take(data_copy_count())
            .collect::<BTreeSet<_>>();

        trace!(
            "Target holders of {:?} are non-full adults: {:?} and full adults that were ignored: {:?}",
            target,
            candidates,
            full_adults
        );

        candidates
    }
}
