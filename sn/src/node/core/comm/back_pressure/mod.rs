// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod load_monitoring;

use self::load_monitoring::{LoadMonitoring, INITIAL_MSGS_PER_S};

use futures::future::join;
use itertools::Itertools;
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::Instant};

const MIN_REPORT_INTERVAL: Duration = Duration::from_secs(60);
const REPORT_TTL: Duration = Duration::from_secs(300); // 5 minutes

const DEFAULT_MSGS_PER_S_AND_PEER: usize = INITIAL_MSGS_PER_S / 10;

type OutgoingReports = BTreeMap<SocketAddr, (Instant, usize)>;
type IncomingReports = BTreeMap<SocketAddr, (Instant, usize)>;

#[derive(Clone)]
pub(crate) struct BackPressure {
    monitoring: LoadMonitoring,
    our_reports: Arc<RwLock<OutgoingReports>>,
    reports: Arc<RwLock<IncomingReports>>,
    last_eviction: Arc<RwLock<Instant>>,
}

impl BackPressure {
    pub(crate) fn new() -> Self {
        Self {
            monitoring: LoadMonitoring::new(),
            our_reports: Arc::new(RwLock::new(OutgoingReports::new())),
            reports: Arc::new(RwLock::new(IncomingReports::new())),
            last_eviction: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub(crate) fn count_msg(&self) {
        self.monitoring.count_msg();
    }

    /// Gets msgs / s or default for a peer
    pub(crate) async fn get_peer_tolerance(&self, addr: &SocketAddr) -> usize {
        self.reports
            .read()
            .await
            .get(addr)
            .copied()
            .map(|(_, cfg)| cfg)
            .unwrap_or(DEFAULT_MSGS_PER_S_AND_PEER)
    }

    /// Remove regulation for specific nodes when we don't need them anymore (e.g. they left).
    pub(crate) async fn remove(&self, addr: SocketAddr) {
        let _prev = self.reports.write().await.remove(&addr);
    }

    /// Sets requested msgs_per_s for a node, when it errored back saying it was strained.
    pub(crate) async fn set(&self, addr: SocketAddr, msgs_per_s: usize) {
        let _prev = self
            .reports
            .write()
            .await
            .insert(addr, (Instant::now(), msgs_per_s));
    }

    /// Sent to nodes calling us, if we are strained
    pub(crate) async fn tolerated_msgs_per_s(&self, caller: &SocketAddr) -> Option<usize> {
        let now = Instant::now();
        let sent = { self.our_reports.read().await.get(caller).copied() };
        let tolerated_msgs_per_s = match sent {
            Some((then, _)) => {
                // do not refresh too often
                if now > then && now - then > MIN_REPORT_INTERVAL {
                    self.get_load(caller, now).await
                } else {
                    return None; // send None if too short time has elapsed
                }
            }
            None => self.get_load(caller, now).await,
        };

        tolerated_msgs_per_s
    }

    async fn get_load(&self, caller: &SocketAddr, now: Instant) -> Option<usize> {
        let msgs_per_s = self.monitoring.msgs_per_s().await;
        let num_callers = { self.our_reports.read().await.len() };

        // avoid divide by 0 errors
        let msgs_per_s_and_peer = usize::max(
            DEFAULT_MSGS_PER_S_AND_PEER,
            msgs_per_s / usize::max(1, num_callers),
        );

        let prev = self
            .our_reports
            .write()
            .await
            .insert(*caller, (now, msgs_per_s_and_peer));

        // placed in this block, we reduce the frequency of this check
        let last_eviction = { *self.last_eviction.read().await };
        // only try evict when there's any likelihood of there being any expired..
        if now > last_eviction && now - last_eviction > REPORT_TTL {
            self.evict_expired(now).await;
        }

        if let Some((_, previous)) = prev {
            // bound update rates by require some minimum level of change:
            // if current val is 5 % worse, or 10 % better, then update our peer with it
            let change_ratio = msgs_per_s_and_peer as f64 / previous as f64;
            if 0.95 >= change_ratio || change_ratio >= 1.1 || change_ratio == 0.0 {
                return None;
            }
        }

        Some(msgs_per_s_and_peer)
    }

    async fn evict_expired(&self, now: Instant) {
        let _res = join(self.evict_in_expired(now), self.evict_out_expired(now)).await;
        *self.last_eviction.write().await = now;
    }

    async fn evict_in_expired(&self, now: Instant) {
        let expired = {
            self.reports
                .read()
                .await
                .iter()
                .filter_map(|(key, (last_seen, _))| {
                    let last_seen = *last_seen;
                    if now > last_seen && now - last_seen > REPORT_TTL {
                        Some(*key)
                    } else {
                        None
                    }
                })
                .collect_vec()
        };

        for addr in expired {
            self.remove(addr).await
        }
    }

    async fn evict_out_expired(&self, now: Instant) {
        let expired = {
            self.our_reports
                .read()
                .await
                .iter()
                .filter_map(|(key, (last_seen, _))| {
                    let last_seen = *last_seen;
                    if now > last_seen && now - last_seen > REPORT_TTL {
                        Some(*key)
                    } else {
                        None
                    }
                })
                .collect_vec()
        };

        for addr in expired {
            let _prev = self.our_reports.write().await.remove(&addr);
        }
    }
}
