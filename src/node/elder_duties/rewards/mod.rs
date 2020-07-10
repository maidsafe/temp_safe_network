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

use crate::node::messaging::Messaging;
use crate::cmd::NodeCmd;
use super::{section_funds::SectionFunds, system::FarmingSystem, validator::Validator};
use safe_transfers::TransferActor;
use safe_farming::{Accumulation, AccumulationEvent, RewardCounter, StorageRewards};
use safe_nd::{XorName, AccountId, Result, MsgEnvelope, MsgSender, Message, NetworkCmd, Duty, ElderDuty, Money};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter},
    rc::Rc,
    collections::HashMap,
};

pub(super) struct Rewards {
    farming: FarmingSystem<StorageRewards>,
    node_accounts: HashMap<XorName, AccountId>,
    section_funds: SectionFunds,
}

impl Rewards {
    pub fn new(actor: TransferActor<Validator>) -> Self {
        let acc = Accumulation::new(Default::default(), Default::default());
        let base_cost = Money::from_nano(1);
        let algo = StorageRewards::new(base_cost);
        let farming = FarmingSystem::new(algo, acc);
        let section_funds = SectionFunds::new(actor);
        Self {
            farming,
            node_accounts: Default::default(),
            section_funds,
        }
    }

    /// Work is the total work associated with this account id.
    /// It is a strictly incrementing value during the lifetime of
    /// the owner on the network.
    pub fn add_account(&mut self, id: AccountId, counter: RewardCounter) -> Option<NodeCmd> {
        let work = counter.work;
        match self.system.add_account(id, work) {
            Ok() => (),
            Err(err) => {
                // todo: NetworkCmdError
                return None;
            }
        };
        if counter.reward > Money::zero() {
            self.section_funds.payout_rewards(counter.reward, id)
        }
        None
    }

    pub fn reward(
        &mut self,
        data: Vec<u8>,
    ) -> Option<NodeCmd> {
        let num_bytes = data.len() as u64;
        let data_hash = data;
        match self.system.reward(data_hash, num_bytes, factor) {
            Ok() => None,
            Err(err) => {
                // todo: NetworkCmdError
                return None;
            }
        }
    }

    /// On node relocation, claim the node farming account
    /// rewards, and send it to the new section.
    pub fn relocate(&mut self, old_node_id: XorName, new_node_id: XorName) -> Option<NodeCmd> {
        let account_id = self.node_accounts.get(&old_node_id)?;
        let counter = match self.claim(account_id) {
            Ok(counter) => counter,
            Err(err) => {
                // todo: NetworkCmdError
                return None;
            }
        };
        let message = Message::NetworkCmd(NetworkCmd::ReceiveWorker {
            new_node_id,
            account_id,
            counter,
        });
        let signature = &utils::sign(self.routing.borrow(), &utils::serialise(&message));
        let msg = MsgEnvelope {
            message,
            origin: MsgSender::Node {
                id,
                duty: Duty::Elder(ElderDuty::Rewards),
                signature,
            },
            proxies: Default::default(),
        };
        wrap(RewardCmd::SendToSection(msg))
    }
}

fn wrap(cmd: RewardCmd) -> Option<NodeCmd> {
    Some(NodeCmd::Reward(cmd))
}