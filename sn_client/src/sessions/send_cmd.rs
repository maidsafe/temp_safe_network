// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{MsgResponse, Session};
use crate::{Error, Result};

use qp2p::UsrMsgBytes;
use sn_interface::{
    at_least_one_correct_elder, data_copy_count,
    messaging::{ClientAuth, Dst, MsgId, MsgKind, WireMsg},
    network_knowledge::supermajority,
    types::{DataError, Peer},
};

use bytes::Bytes;
use std::{collections::BTreeSet, net::SocketAddr};
use tokio::task::JoinSet;
use tracing::{debug, error, trace, warn};
use xor_name::XorName;

impl Session {
    #[instrument(skip(self, auth, payload), level = "debug", name = "session send cmd")]
    pub(crate) async fn send_cmd(
        &self,
        dst_address: XorName,
        auth: ClientAuth,
        payload: Bytes,
        needs_super_majority: bool,
    ) -> Result<()> {
        let endpoint = self.endpoint.clone();
        // TODO: Consider other approach: Keep a session per section!
        let (section_pk, elders) = self
            .get_cmd_elders(dst_address, needs_super_majority)
            .await?;

        let elders_len = elders.len();
        let msg_id = MsgId::new();
        debug!(
            "Sending cmd with {msg_id:?}, dst: {dst_address:?}, from {}, \
            to {elders_len} Elders: {elders:?}",
            endpoint.local_addr(),
        );

        let dst = Dst {
            name: dst_address,
            section_key: section_pk,
        };
        let kind = MsgKind::Client(auth);
        let wire_msg = WireMsg::new_msg(msg_id, payload, kind, dst);

        let log_line = |elders_len_s: String| {
            debug!(
                "Sending cmd w/id {msg_id:?}, from {}, to {elders_len_s} w/ dst: {dst_address:?}",
                endpoint.local_addr(),
            )
        };

        if needs_super_majority {
            log_line(format!("{elders_len}"));
            self.send_cmd_msg(elders.clone(), wire_msg).await
        } else {
            #[cfg(feature = "cmd-happy-path")]
            {
                log_line(format!("1 Elder (or at most {elders_len})"));
                self.send_to_one_or_more(dst_address, elders.clone(), wire_msg)
                    .await
            }
            #[cfg(not(feature = "cmd-happy-path"))]
            {
                log_line(format!("{elders_len}"));
                self.send_cmd_msg(elders.clone(), wire_msg).await
            }
        }
    }

    #[instrument(skip(self), level = "debug", name = "session setup conns")]
    /// Make a best effort to pre connect to only relevant nodes for a set of dst addresses
    /// This should reduce the number of connections attempts to the same elder set
    pub(crate) async fn prepare_connections(
        &self,
        dst_addresses: Vec<XorName>,
        needs_super_majority: bool,
    ) -> Result<()> {
        let mut relevant_elders = BTreeSet::new();
        for address in dst_addresses {
            let (_, elders) = self.get_cmd_elders(address, needs_super_majority).await?;
            for elder in elders {
                let _existed = relevant_elders.insert(elder);
            }
        }

        let mut tasks = vec![];
        for peer in relevant_elders {
            let session = self.clone();

            let task = async move {
                let connect_now = true;
                // We don't retry here.. if we fail it will be retried on a per message basis
                let _ = session
                    .peer_links
                    .get_or_create_link(&peer, connect_now, None)
                    .await;
            };
            tasks.push(task);
        }

        let _ = futures::future::join_all(tasks).await;

        Ok(())
    }

    /// This function will try a happy path,
    /// successively expanding to all the other elders in case of failure.
    ///
    /// 1st attempt: Closest Elder (take 1) (take index 0)
    /// 2nd attempt: Next closest (skip 1, take 1) (skip idx 0, take idx 1)
    /// 3rd attempt: Next 2 closest (skip 2, take 2) (skip idx 0 and 1, take idx 2 and 3)
    /// 4th attempt: Next 3 closest (skip 4, take 3) (skip idx 0-3, take index 4, 5 and 6)
    #[cfg(feature = "cmd-happy-path")]
    async fn send_to_one_or_more(
        &self,
        target: XorName,
        all_elders: Vec<Peer>,
        wire_msg: WireMsg,
    ) -> Result<()> {
        let msg_id = wire_msg.msg_id();
        // On happy path, we only require 1 ack.
        // NB: we are for now expecting as many acks as we are calling Elders. To be fixed.
        let _expected_acks = 1;

        // This will do at most 4 attempts, to: 1, 1, 2, and finally 3 elders,
        // thus eventually calling all 7 elders.
        for skip in 0..3 {
            let take = if skip == 0 { 1 } else { skip };

            let elders = self
                .pick_elders(target, all_elders.clone(), skip, take)
                .await;

            trace!("Sending cmd {msg_id:?}, skipping {skip}, sending to {take} elders..");

            // We only require one ack, we wait for it to get received.
            // Any AE message is handled by the tasks, hence no extra wait is required.
            if self
                .send_cmd_msg(elders.clone(), wire_msg.clone())
                .await
                .is_ok()
            {
                trace!("Acks of Cmd {:?} received", msg_id);
                return Ok(());
            }
        }

        // we expected at least one ack, but got 0
        Err(Error::InsufficientAcksReceived {
            msg_id,
            expected: 1,
            received: 0,
        })
    }

    #[cfg(feature = "cmd-happy-path")]
    async fn pick_elders(
        &self,
        target: XorName,
        elders: Vec<Peer>,
        skip: usize,
        take: usize,
    ) -> Vec<Peer> {
        use itertools::Itertools;
        elders
            .into_iter()
            .sorted_by(|lhs, rhs| target.cmp_distance(&lhs.name(), &rhs.name()))
            .skip(skip)
            .take(take)
            .collect()
    }

    /// Checks for acks for a given msg.
    /// Returns Ok if we've sufficient to call this cmd a success
    ///
    /// We send the cmd to each Elder, who then relays the cmd to the data holders.
    /// We will get back the cmd responses from each data holder, for each Elder.
    /// With sending to at least one correct Elder (3) who each sends to 4 data holders, that
    /// gives a total of 12 acks that we expect. With super majority (5) we get a total of 20 acks.
    async fn we_have_sufficient_acks_for_cmd(
        &self,
        msg_id: MsgId,
        elders: Vec<Peer>,
        mut send_cmd_tasks: JoinSet<(Peer, Vec<MsgResponse>)>,
    ) -> Result<()> {
        // We don't yet have a way to differentiate the data holder responses
        // we get from each Elder. We need to have the data holders sign their response (the serialized payload)
        // then the Elder receiving it needs to verify the sig before sending it on to the client. (The Elders
        // do not need to deserialize the payload.) The client can then see by the different sigs that they are from different
        // data holders. If a deserialized payload turns out to be something unexpected, such as a wrong type or data not found,
        // then the client can send that in, with the sig, to the Elders. Their incentive to do this is to ensure that the
        // data they paid for, is actually being stored in the network. This way, we have the clients doing the work of checking the adults,
        // while still shielding the adults from the clients.

        debug!("----> Init of check for acks for {msg_id:?}");

        let mut reports = vec![];

        // here we expect `elders.len()` x `data_copy_count()` acks
        while let Some(msg_resp) = send_cmd_tasks.join_next().await {
            debug!("Handling msg_resp sent to ack wait channel: {msg_resp:?}");
            match msg_resp {
                Ok(responses) => reports.push(categorize(msg_id, responses)),
                Err(task_join_err) => {
                    warn!("Task join failure occurred with msg {msg_id:?}: {task_join_err:?}");
                    warn!("An elder will be considered unresponsive due to this, it might or might not be warranted.");
                    continue;
                }
            };
        }

        // For now, we require `data_copy_count()` acks from _each_ Elder!
        // (Below line would instead allow us to deduplicate.)
        // let expected_unique_acks = data_copy_count();
        let data_copy_count = data_copy_count();
        let expected_acks = elders.len() * data_copy_count;

        let (succeeded, failed): (Vec<_>, Vec<_>) =
            reports.iter().partition(|r| r.has_enough_acks());

        if succeeded.len() >= elders.len() {
            trace!(
                "{msg_id:?} Good! We've got the expected_acks from {} elders, /
                ({} from each, for a total of {expected_acks}).",
                elders.len(),
                data_copy_count
            );
            return Ok(());
        }

        let received_errors: Vec<_> = failed
            .iter()
            .map(|r| (r.elder, r.received_errors.clone()))
            .collect();

        if received_errors.len() > data_copy_count / 2 {
            error!("Received majority of error response for cmd {msg_id:?}: {received_errors:?}");
            return Err(Error::CmdError {
                received_errors,
                msg_id,
            });
        }

        let received_acks = succeeded.len() * data_copy_count;

        let incomplete_acks: Vec<_> = failed.iter().map(|r| r.elder).collect();

        let unresponsive: Vec<_> = elders
            .iter()
            .filter(|peer| !reports.iter().any(|r| &r.elder == peer))
            .collect();

        debug!(
            "Insufficient CmdAcks returned for {msg_id:?}: {received_acks}/{expected_acks}. \
            Unresponsive elders: {unresponsive:?}  \
            Incomplete acks from: {incomplete_acks:?}",
        );

        Err(Error::InsufficientAcksReceived {
            msg_id,
            expected: expected_acks,
            received: received_acks,
        })
    }

    /// Returns either `supermajority=5` or `at_least_one_correct=3` Elders.
    async fn get_cmd_elders(
        &self,
        dst_address: XorName,
        needs_super_majority: bool,
    ) -> Result<(bls::PublicKey, Vec<Peer>)> {
        let a_close_sap = self
            .network
            .read()
            .await
            .closest(&dst_address, None)
            .cloned();

        // Get DataSection elders details.
        if let Some(sap) = a_close_sap {
            let sap_elders = sap.elders_vec();
            let section_pk = sap.section_key();
            trace!("SAP elders found {sap_elders:?}");

            let targets_count = if needs_super_majority {
                // Supermajority of elders is expected for payment-cmds.
                supermajority(sap_elders.len())
            } else {
                // Three out of seven is enough for data cmds.
                at_least_one_correct_elder()
            };

            // any SAP that does not hold elders_count() is indicative of a broken network (after genesis)
            if sap_elders.len() < targets_count {
                error!(
                    "Insufficient knowledge to send to address {dst_address:?}, \
                    elders for this section: {sap_elders:?} ({targets_count} needed), \
                    section PK is: {section_pk:?}"
                );
                return Err(Error::InsufficientElderKnowledge {
                    connections: sap_elders.len(),
                    required: targets_count,
                    section_pk,
                });
            }

            Ok((section_pk, sap_elders))
        } else {
            Err(Error::NoNetworkKnowledge(dst_address))
        }
    }

    #[instrument(skip_all, level = "trace")]
    async fn send_cmd_msg(&self, elders: Vec<Peer>, wire_msg: WireMsg) -> Result<()> {
        let msg_id = wire_msg.msg_id();
        debug!(
            "---> Send msg {msg_id:?} going out to {} elders.",
            elders.len()
        );
        let bytes = wire_msg.serialize()?;

        let mut tasks = JoinSet::new();

        for (peer_index, peer) in elders.iter().enumerate() {
            let session = self.clone();
            let bytes = bytes.clone();
            let _abort_handle = tasks.spawn(send_to_one(msg_id, *peer, peer_index, session, bytes));
        }

        trace!("Cmd msg {msg_id:?} sent");

        // We wait for ALL the expected acks get received.
        // The AE messages are handled by the tasks, hence no extra wait is required.
        match self
            .we_have_sufficient_acks_for_cmd(msg_id, elders, tasks)
            .await
        {
            Ok(()) => {
                trace!("Acks of Cmd {:?} received", msg_id);
                Ok(())
            }
            error => error,
        }
    }
}

async fn send_to_one(
    msg_id: MsgId,
    peer: Peer,
    peer_index: usize,
    session: Session,
    bytes: UsrMsgBytes,
) -> (Peer, Vec<MsgResponse>) {
    let mut connect_now = false;
    debug!("Trying to send msg {msg_id:?} to {peer:?}");
    loop {
        let link = session
            .peer_links
            .get_or_create_link(&peer, connect_now, Some(msg_id))
            .await;
        match link.send_bi(bytes.clone(), msg_id).await {
            Ok(recv_stream) => {
                debug!("That's {msg_id:?} sent to {peer:?}... starting receive listener");
                // Listen for responses on the bi-stream.
                let responses = session
                    .receive_cmd_responses(msg_id, peer, peer_index, data_copy_count(), recv_stream)
                    .await;
                break (peer, responses);
            }
            Err(error) if !connect_now => {
                // Retry (only once) to reconnect to this peer and send the msg.
                error!(
                    "Failed to send {msg_id:?} to {peer:?} on a new \
                    bi-stream: {error:?}. Creating a new connection to retry once ..."
                );
                session.peer_links.remove_link_from_peer_links(&peer).await;
                connect_now = true;
                continue;
            }
            Err(error) => {
                error!("Error sending {msg_id:?} bidi to {peer:?}: {error:?}");
                session.peer_links.remove_link_from_peer_links(&peer).await;
                break (
                    peer,
                    vec![MsgResponse::Failure(
                        peer.addr(),
                        Error::FailedToInitateBiDiStream { msg_id, error },
                    )],
                );
            }
        }
    }
}

/// This is the collection of
/// received responses and failures while
/// trying to read a response, from a specific Elder,
/// from which we expect `data_copy_count()` responses.
struct ResponseReport {
    elder: Peer,
    expected_acks: usize,
    received_acks: BTreeSet<SocketAddr>,
    received_errors: Vec<DataError>,
    invalid_responses: BTreeSet<SocketAddr>,
    failures: BTreeSet<SocketAddr>,
}

impl ResponseReport {
    fn new(elder: Peer) -> Self {
        Self {
            elder,
            expected_acks: data_copy_count(),
            received_acks: BTreeSet::new(),
            received_errors: vec![],
            invalid_responses: BTreeSet::new(),
            failures: BTreeSet::new(),
        }
    }

    /// Will require `data_copy_count()` acks.
    /// (Would normally count unique responses, and require a certain number of successes.)
    fn has_enough_acks(&self) -> bool {
        self.received_acks.len() > self.expected_acks
        // let bad_requests =
        //     self.received_errors.len() + self.invalid_responses.len() + self.failures.len();

        // self.received_acks.len() > bad_requests
    }
}

fn categorize(msg_id: MsgId, peer_responses: (Peer, Vec<MsgResponse>)) -> ResponseReport {
    let (peer, responses) = peer_responses;
    let mut report = ResponseReport::new(peer);

    for response in responses {
        match response {
            MsgResponse::CmdResponse(src, response) => {
                match response.result() {
                    Ok(()) => {
                        if src == report.elder.addr() {
                            let _ = report.received_acks.insert(src);
                        }
                    }
                    Err(error) => {
                        if src == report.elder.addr() {
                            report.received_errors.push(error.clone());
                            debug!("Cmd response error from {} in response to msg {msg_id:?}: {error:?}", report.elder);
                        }
                    }
                }
            }
            MsgResponse::Failure(src, error) => {
                if src == report.elder.addr() {
                    debug!(
                        "Failure returned from {} in response to msg {msg_id:?}: {error:?}",
                        report.elder
                    );
                    let _ = report.failures.insert(src);
                }
                continue;
            }
            MsgResponse::QueryResponse(src, resp) => {
                if src == report.elder.addr() {
                    debug!("Unexpected query response received from {} for {msg_id:?} when awaiting a CmdAck: {resp:?}", report.elder);
                    let _ = report.invalid_responses.insert(src);
                }
                continue;
            }
        };
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use sn_interface::{
        elder_count,
        network_knowledge::SectionTree,
        test_utils::{prefix, TestKeys, TestSapBuilder},
    };

    use eyre::Result;
    use std::net::{Ipv4Addr, SocketAddr};
    use xor_name::Prefix;

    fn new_network_network_contacts() -> (SectionTree, bls::SecretKey, bls::PublicKey) {
        let (genesis_sap, genesis_sk_set, ..) = TestSapBuilder::new(Prefix::default()).build();

        let genesis_sk = genesis_sk_set.secret_key();
        let genesis_pk = genesis_sk.public_key();
        let genesis_sap = TestKeys::get_section_signed(&genesis_sk, genesis_sap);
        let tree = SectionTree::new(genesis_sap).expect("SAP belongs to the genesis prefix");

        (tree, genesis_sk, genesis_pk)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cmd_sent_to_all_elders() -> Result<()> {
        let elders_len = supermajority(elder_count());
        let needs_super_majority = true;

        let prefix = prefix("0");
        let (sap, secret_key_set, ..) = TestSapBuilder::new(prefix).elder_count(elders_len).build();
        let sap0 = TestKeys::get_section_signed(&secret_key_set.secret_key(), sap);
        let (mut network_contacts, _genesis_sk, _) = new_network_network_contacts();
        assert!(network_contacts.insert_without_chain(sap0));

        let session = Session::new(
            SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)),
            network_contacts,
        )?;

        let mut rng = rand::thread_rng();
        let result = session
            .get_cmd_elders(XorName::random(&mut rng), needs_super_majority)
            .await?;
        assert_eq!(result.0, secret_key_set.public_keys().public_key());
        assert_eq!(result.1.len(), elders_len);

        Ok(())
    }
}
