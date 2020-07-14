// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod section_funds;
mod system;
mod validator;

use self::section_funds::SectionFunds;
pub use self::{system::FarmingSystem, validator::Validator};
use crate::{cmd::OutboundMsg, node::keys::NodeKeys, node::msg_decisions::ElderMsgDecisions};
use safe_farming::{Accumulation, StorageRewards};
use safe_nd::{
    AccountId, ElderDuty, Error, Message, MessageId, Money, MsgSender, NetworkCmdError,
    NetworkEvent, NetworkRewardError, RewardCounter, XorName,
};
use safe_transfers::TransferActor;
use std::collections::HashMap;

pub struct Rewards {
    farming: FarmingSystem<StorageRewards>,
    node_accounts: HashMap<XorName, RewardAccount>,
    section_funds: SectionFunds,
    decisions: ElderMsgDecisions,
}

struct RewardAccount {
    pub id: AccountId,
    pub stage: RewardStage,
}

#[derive(PartialEq)]
enum RewardStage {
    /// From the point of being added, the
    /// stage of the RewardAccount is `Active`.
    Active,
    /// After a node leaves the section
    /// the RewardAccount transitions into
    /// stage `AwaitingClaim`.
    AwaitingClaim,
}

impl Rewards {
    pub fn new(keys: NodeKeys, actor: TransferActor<Validator>) -> Self {
        let decisions = ElderMsgDecisions::new(keys, ElderDuty::Rewards);
        let acc = Accumulation::new(Default::default(), Default::default());
        let base_cost = Money::from_nano(1);
        let algo = StorageRewards::new(base_cost);
        let farming = FarmingSystem::new(algo, acc);
        let section_funds = SectionFunds::new(actor, decisions.clone());
        Self {
            farming,
            node_accounts: Default::default(),
            section_funds,
            decisions,
        }
    }

    /// Work is the total work associated with this account id.
    /// It is a strictly incrementing value during the lifetime of
    /// the owner on the network.
    pub fn add_account(&mut self, id: AccountId, counter: RewardCounter) -> Option<OutboundMsg> {
        let work = counter.work;
        match self.farming.add_account(id, work) {
            Ok(_) => (),
            Err(_err) => {
                // todo: NetworkCmdError
                return None;
            }
        };
        if counter.reward > Money::zero() {
            return self
                .section_funds
                .initiate_reward_payout(counter.reward, id);
        }
        None
    }

    pub fn reward(&mut self, data: Vec<u8>) -> Option<OutboundMsg> {
        let num_bytes = data.len() as u64;
        let data_hash = data;
        let factor = 2.0;
        match self.farming.reward(data_hash, num_bytes, factor) {
            Ok(_) => None,
            Err(_err) => None, // todo: NetworkCmdError. Or not? This is an internal thing..
        }
    }

    /// When the section becomes aware that a node has left,
    /// it is flagged for being claimed.
    pub fn node_left(&mut self, node_id: XorName) -> Option<OutboundMsg> {
        let account = self.node_accounts.get_mut(&node_id)?;
        account.stage = RewardStage::AwaitingClaim;
        None
    }

    /// On node relocation, receiving section queries for the
    /// node counter, and asks to claim the node farming account
    /// rewards, and for the old section to send it to the new section.
    pub fn claim_rewards(
        &mut self,
        old_node_id: XorName,
        new_node_id: XorName,
        msg_id: MessageId,
        origin: &MsgSender,
    ) -> Option<OutboundMsg> {
        use NetworkCmdError::*;
        use NetworkRewardError::*;
        let account = self.node_accounts.get(&old_node_id)?;
        if account.stage != RewardStage::AwaitingClaim {
            // ..means the node has not left, and was not
            // marked as awaiting claim..
            let error = Error::NetworkOther("InvalidClaim: Account is still active.".to_string());
            let error = Rewards(RewardClaiming {
                error,
                account_id: account.id,
            });
            return self.decisions.network_error(error, msg_id, origin);
        }

        // Remove the counter.
        let counter = match self.farming.claim(account.id) {
            Ok(counter) => counter,
            Err(error) => {
                let error = Rewards(RewardClaiming {
                    error,
                    account_id: account.id,
                });
                return self.decisions.network_error(error, msg_id, origin);
            }
        };

        // Send it to the new section.
        let message = Message::NetworkEvent {
            event: NetworkEvent::RewardCounterClaimed {
                new_node_id,
                account_id: account.id,
                counter,
            },
            id: MessageId::new(),
            correlation_id: msg_id,
        };

        self.decisions.send(message)
    }
}
