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
use crate::{
    cmd::OutboundMsg, messaging::Messaging, node::keys::NodeKeys,
    node::msg_decisions::ElderMsgDecisions,
};
use safe_farming::{Accumulation, RewardCounter, StorageRewards};
use safe_nd::{AccountId, ElderDuty, Money, XorName};
use safe_transfers::TransferActor;
use std::collections::HashMap;

pub(super) struct Rewards {
    farming: FarmingSystem<StorageRewards>,
    node_accounts: HashMap<XorName, AccountId>,
    section_funds: SectionFunds,
    decisions: ElderMsgDecisions,
}

impl Rewards {
    pub fn new(keys: NodeKeys, actor: TransferActor<Validator>) -> Self {
        let decisions = ElderMsgDecisions::new(keys.clone(), ElderDuty::Rewards);
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
            Err(err) => {
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
            Err(err) => {
                // todo: NetworkCmdError
                return None;
            }
        }
    }

    /// On node relocation, receiving section queries for the
    /// node counter, and asks to claim the node farming account
    /// rewards, and for the old section to send it to the new section.
    pub fn relocate(&mut self, old_node_id: XorName, new_node_id: XorName) -> Option<OutboundMsg> {
        unimplemented!()
        // let account_id = self.node_accounts.get(&old_node_id)?;
        // let counter = match self.claim(account_id) {
        //     Ok(counter) => counter,
        //     Err(err) => {
        //         // todo: NetworkCmdError
        //         return None;
        //     }
        // };
        // let message = Message::NetworkCmd { cmd: NetworkCmd::ReceiveWorker {
        //     new_node_id,
        //     account_id,
        //     counter,
        // });
        // let signature = &utils::sign(self.routing.borrow(), &utils::serialise(&message));
        // let msg = MsgEnvelope {
        //     message,
        //     origin: MsgSender::Node {
        //         id,
        //         duty: Duty::Elder(ElderDuty::Rewards),
        //         signature,
        //     },
        //     proxies: Default::default(),
        // };
        // Some(OutboundMsg::SendToSection(msg))
    }
}
