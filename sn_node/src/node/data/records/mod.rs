// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod capacity;

pub(crate) use self::capacity::{Capacity, MIN_LEVEL_WHEN_FULL};

use crate::node::{
    error::convert_to_error_msg, messages::WireMsgUtils, Cmd, Error, Node, Prefix, Result,
    MAX_WAITING_PEERS_PER_QUERY,
};

use itertools::Itertools;
use sn_dysfunction::IssueType;
#[cfg(feature = "traceroute")]
use sn_interface::messaging::Entity;
use sn_interface::{
    data_copy_count,
    messaging::{
        data::{CmdError, DataQuery, MetadataExchange, StorageLevel},
        system::{NodeCmd, NodeQuery, SystemMsg},
        AuthorityProof, DstLocation, EndUser, MsgId, ServiceAuth, WireMsg,
    },
    types::{log_markers::LogMarker, Peer, PublicKey, ReplicatedData},
};
use std::{cmp::Ordering, collections::BTreeSet};
use tracing::info;
use xor_name::XorName;

impl Node {
    // Locate ideal holders for this data, line up wiremsgs for those to instruct them to store the data
    pub(crate) fn replicate_data(
        &self,
        data: ReplicatedData,
        #[cfg(feature = "traceroute")] mut traceroute: Vec<Entity>,
    ) -> Result<Vec<Cmd>> {
        trace!("{:?}: {:?}", LogMarker::DataStoreReceivedAtElder, data);
        if self.is_elder() {
            let targets = self.get_adults_who_should_store_data(data.name());

            info!(
                "Replicating data {:?} to holders {:?}",
                data.name(),
                &targets,
            );

            let msg = SystemMsg::NodeCmd(NodeCmd::ReplicateData(vec![data]));
            self.send_node_msg_to_nodes(
                msg,
                targets,
                #[cfg(feature = "traceroute")]
                &mut traceroute,
            )
        } else {
            Err(Error::InvalidState)
        }
    }

    pub(crate) async fn read_data_from_adults(
        &mut self,
        query: DataQuery,
        msg_id: MsgId,
        auth: AuthorityProof<ServiceAuth>,
        origin: Peer,
        #[cfg(feature = "traceroute")] traceroute: Vec<Entity>,
    ) -> Result<Vec<Cmd>> {
        let address = query.variant.address();
        let operation_id = query.variant.operation_id()?;
        trace!(
            "{:?} preparing to query adults for data at {:?} with op_id: {:?}",
            LogMarker::DataQueryReceviedAtElder,
            address,
            operation_id
        );

        let targets = self.get_adults_holding_data_including_full(address.name());

        // Query only the nth adult
        let targets = BTreeSet::from_iter(
            targets
                .iter()
                .nth(query.adult_index) // Grab only nth adult
                .iter() // Get Iter of length 0 (if nth adult does not exists) or 1
                .copied() // &&XorName -> &XorName
                .copied(), // &XorName -> XorName
        );

        if targets.is_empty() {
            let error = convert_to_error_msg(Error::NoAdults(self.network_knowledge().prefix()));

            debug!("No targets found for {msg_id:?}");
            return self.send_cmd_error_response(
                CmdError::Data(error),
                origin,
                msg_id,
                #[cfg(feature = "traceroute")]
                traceroute,
            );
        }

        let mut op_was_already_underway = false;
        let waiting_peers =
            if let Some(mut peers) = self.pending_data_queries.get(&operation_id).await {
                op_was_already_underway = peers.insert(origin);

                peers
            } else {
                let mut peers = BTreeSet::new();
                let _false_as_fresh = peers.insert(origin);
                peers
            };

        // drop if we exceed
        if waiting_peers.len() > MAX_WAITING_PEERS_PER_QUERY {
            warn!("Dropping query from {origin:?}, there are more than {MAX_WAITING_PEERS_PER_QUERY} waiting already");
            return Ok(vec![]);
        }

        // only set pending data query cache if non existed.
        // otherwise we've appended to the Peers above
        // we rely on the data query cache timeout to decide as/when we'll be re-sending a query to adults
        if !op_was_already_underway {
            // ensure we only add a pending request when we're actually sending out requests.
            for target in &targets {
                trace!("adding pending req for {target:?} in dysfunction tracking");
                self.dysfunction_tracking.track_issue(
                    target.name(),
                    IssueType::PendingRequestOperation(Some(operation_id)),
                )?;
            }

            trace!(
                "Adding to pending data queries for op id: {:?}",
                operation_id
            );

            let _prior_value = self
                .pending_data_queries
                .set(operation_id, waiting_peers, None)
                .await;

            let msg = SystemMsg::NodeQuery(NodeQuery::Data {
                query: query.variant,
                auth: auth.into_inner(),
                origin: EndUser(origin.name()),
                correlation_id: msg_id,
            });

            self.send_node_msg_to_nodes(
                msg,
                targets,
                #[cfg(feature = "traceroute")]
                &mut traceroute.clone(),
            )
        } else {
            // we don't do anything as we're still within data query timeout
            Ok(vec![])
        }
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
    fn get_adults_holding_data_including_full(&self, target: &XorName) -> BTreeSet<Peer> {
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
            "Chunk holders of {:?} are non-full adults: {:?} and full adults: {:?}",
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
                        for adult in adults.iter() {
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
    fn get_adults_who_should_store_data(&self, target: XorName) -> BTreeSet<Peer> {
        let full_adults = self.full_adults();
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

    // Takes a message for specified targets, and builds internal send cmds
    // for sending to each of the targets.
    // Targets are XorName specified so must be within the section
    fn send_node_msg_to_nodes(
        &self,
        msg: SystemMsg,
        targets: BTreeSet<Peer>,
        #[cfg(feature = "traceroute")] traceroute: &mut Vec<Entity>,
    ) -> Result<Vec<Cmd>> {
        // we create a dummy/random dst location,
        // we will set it correctly for each msg and target
        let section_pk = self.network_knowledge().section_key();
        let our_name = self.info().name();
        let dummy_dst_location = DstLocation::Node {
            name: our_name,
            section_pk,
        };

        // separate this into form_wire_msg based on agg
        #[allow(unused_mut)]
        let mut wire_msg = WireMsg::single_src(&self.info(), dummy_dst_location, msg, section_pk)?;

        #[cfg(feature = "traceroute")]
        {
            traceroute.push(Entity::Elder(PublicKey::Ed25519(
                self.info().keypair.public,
            )));
            wire_msg.add_trace(traceroute);
        }

        let mut cmds = vec![];

        for target in targets {
            debug!("Queueing Send of {:?} to {:?}", wire_msg, target);
            let mut wire_msg = wire_msg.clone();
            let dst_section_pk = self.section_key_by_name(&target.name());
            wire_msg.set_dst_section_pk(dst_section_pk);
            wire_msg.set_dst_xorname(target.name());

            cmds.push(Cmd::SendMsg {
                recipients: vec![target],
                wire_msg,
            });
        }

        Ok(cmds)
    }
}
