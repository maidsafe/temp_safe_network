// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod client;
mod client_msg_analysis;
mod transfers;

use self::{
    client::ClientGateway,
    client_msg_analysis::ClientMsgAnalysis,
    transfers::{replicas::Replicas, Transfers},
};
use crate::{
    capacity::RateLimit,
    node::node_ops::{KeySectionDuty, NodeOperation},
    node::state_db::NodeInfo,
    Network, ReplicaInfo, Result,
};
use crate::{Outcome, TernaryResult};
use futures::lock::Mutex;
use log::trace;
use rand::{CryptoRng, Rng};
use sn_routing::Prefix;
use std::path::PathBuf;
use std::sync::Arc;

/// A Key Section interfaces with clients,
/// who are essentially a public key,
/// (hence the name Key Section), used by
/// a specific socket address.
/// The Gateway deals with onboarding (handshakes etc)
/// and routing messages back and forth to clients.
/// Payments deals with the payment for data writes,
/// while transfers deals with sending money between keys.
pub struct KeySection<R: CryptoRng + Rng> {
    gateway: ClientGateway<R>,
    transfers: Transfers,
    msg_analysis: ClientMsgAnalysis,
    replicas: Replicas,
    routing: Network,
}

impl<R: CryptoRng + Rng> KeySection<R> {
    pub async fn new(
        info: &NodeInfo,
        rate_limit: RateLimit,
        routing: Network,
        rng: R,
    ) -> Result<Self> {
        let gateway = ClientGateway::new(info, routing.clone(), rng).await?;
        let replicas = Self::new_replica_manager(info.root_dir.clone(), routing.clone()).await?;
        let transfers = Transfers::new(info.keys.clone(), replicas.clone(), rate_limit);
        let msg_analysis = ClientMsgAnalysis::new(routing.clone());

        Ok(Self {
            gateway,
            transfers,
            msg_analysis,
            replicas,
            routing,
        })
    }

    /// Initiates as first node in a network.
    pub async fn init_first(&mut self) -> Outcome<NodeOperation> {
        self.transfers.init_first().await
    }

    /// Issues queries to Elders of the section
    /// as to catch up with shares state and
    /// start working properly in the group.
    pub async fn catchup_with_section(&mut self) -> Outcome<NodeOperation> {
        // currently only at2 replicas need to catch up
        self.transfers.catchup_with_replicas().await
    }

    // Update our replica with the latest keys
    pub async fn elders_changed(&mut self) -> Outcome<NodeOperation> {
        let secret_key_share = self.routing.secret_key_share().await?;
        let id = secret_key_share.public_key_share();
        let key_index = self.routing.our_index().await?;
        let peer_replicas = self.routing.public_key_set().await?;
        let signing =
            sn_transfers::ReplicaSigning::new(secret_key_share, key_index, peer_replicas.clone());
        let info = ReplicaInfo {
            id,
            key_index,
            peer_replicas,
            section_proof_chain: self.routing.our_history().await,
            signing: Arc::new(Mutex::new(signing)),
            initiating: false,
        };
        self.replicas.update_replica_keys(info);
        Outcome::oki_no_value()
    }

    /// When section splits, the Replicas in either resulting section
    /// also split the responsibility of the accounts.
    /// Thus, both Replica groups need to drop the accounts that
    /// the other group is now responsible for.
    pub async fn section_split(&mut self, prefix: Prefix) -> Outcome<NodeOperation> {
        // Removes accounts that are no longer our section responsibility.
        let not_matching = |key: PublicKey| {
            let xorname: XorName = key.into();
            !prefix.matches(&XorName(xorname.0))
        };
        if let Some(all_keys) = self.replica_manager.lock().await.all_keys() {
            let accounts = all_keys
                .iter()
                .filter(|key| not_matching(**key))
                .copied()
                .collect::<BTreeSet<PublicKey>>();
            self.replica_manager.lock().await.drop_accounts(&accounts)?;
            Outcome::oki_no_change()
        } else {
            Outcome::error(Error::Logic("Could not fetch all replica keys".to_string()))
        }
    }

    pub async fn process_key_section_duty(
        &mut self,
        duty: KeySectionDuty,
    ) -> Outcome<NodeOperation> {
        trace!("Processing as Elder KeySection");
        use KeySectionDuty::*;
        match duty {
            EvaluateClientMsg(msg) => self.msg_analysis.evaluate(&msg).await,
            RunAsGateway(duty) => self.gateway.process_as_gateway(duty).await,
            RunAsTransfers(duty) => self.transfers.process_transfer_duty(&duty).await,
        }
    }

    async fn new_replica_manager(root_dir: PathBuf, routing: Network) -> Result<Replicas> {
        let secret_key_share = routing.secret_key_share().await?;
        let key_index = routing.our_index().await?;
        let peer_replicas = routing.public_key_set().await?;
        let id = secret_key_share.public_key_share();
        let signing =
            sn_transfers::ReplicaSigning::new(secret_key_share, key_index, peer_replicas.clone());
        let info = ReplicaInfo {
            id,
            key_index,
            peer_replicas,
            section_proof_chain: routing.our_history().await,
            signing: Arc::new(Mutex::new(signing)),
            initiating: true,
        };
        let replica_manager = Replicas::new(root_dir, info)?;
        Ok(replica_manager)
    }

    // async fn new_replica_manager(
    //     info: &NodeInfo,
    //     routing: Network,
    //     rate_limit: RateLimit,
    // ) -> Result<Arc<Mutex<ReplicaManager>>> {
    //     let public_key_set = routing.public_key_set().await?;
    //     let secret_key_share = routing.secret_key_share().await?;
    //     let key_index = routing.our_index().await?;
    //     let proof_chain = routing.our_history().await;
    //     let store = TransferStore::new(info.root_dir.clone(), info.init_mode)?;
    //     let replica_manager = ReplicaManager::new(
    //         store,
    //         &secret_key_share,
    //         key_index,
    //         rate_limit,
    //         &public_key_set,
    //         proof_chain,
    //     )?;
    //     Ok(Arc::new(Mutex::new(replica_manager)))
    // }
}
