

use safe_nd::{AccountId, Money, TransferValidated};
use safe_transfers::TransferActor;
use crate::cmd::{ElderCmd, RewardCmd};

pub(super) struct SectionFunds {
    actor: TransferActor,
}

impl SectionFunds {

    pub fn initiate_reward_payout(&mut self, amount: Money, to: AccountId) -> Option<NodeCmd> {
        match self.actor.transfer(amount, to) {
            Ok(Some(event)) => {
                self.actor.apply(event);
                let message = Message::NetworkCmd(NetworkCmd::InitiateRewardPayout {
                    signed_transfer: event.signed_transfer
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
            Ok(None) => None,
            Err(error) => {
                None // for now, but should give NetworkCmdError
            }
        }
    }

    pub fn receive(&mut self, validation: TransferValidated) -> Option<NodeCmd> {
        match self.actor.receive(validation) {
            Ok(Some(event)) => {
                self.actor.apply(event);
                let proof = event.proof?;
                let message = Message::NetworkCmd(NetworkCmd::FinaliseRewardPayout {
                    debit_agreement: proof
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
            Ok(None) => None,
            Err(error) => {
                None // for now, but should give NetworkCmdError
            }
        }
    }
    
    // fn sign_with_signature_share(&self, data: &[u8]) -> Option<(usize, SignatureShare)> {
    //     let signature = self
    //         .routing
    //         .borrow()
    //         .secret_key_share()
    //         .map_or(None, |key| Some(key.sign(data)));
    //     signature.map(|sig| (self.routing.borrow().our_index().unwrap_or(0), sig))
    // }
}

fn wrap(cmd: RewardCmd) -> Option<NodeCmd> {
    Some(ElderCmd::Reward(cmd))
}