use super::validator::Validator;
use crate::{node::node_ops::MessagingDuty, node::msg_wrapping::ElderMsgWrapping};
use safe_nd::{AccountId, Message, MessageId, Money, NodeCmd, NodeTransferCmd, TransferValidated};
use safe_transfers::{ActorEvent, TransferActor};
use ActorEvent::*;

pub(super) struct SectionFunds {
    actor: TransferActor<Validator>,
    wrapping: ElderMsgWrapping,
}

impl SectionFunds {
    pub fn new(actor: TransferActor<Validator>, wrapping: ElderMsgWrapping) -> Self {
        Self { actor, wrapping }
    }

    pub fn initiate_reward_payout(&mut self, amount: Money, to: AccountId) -> Option<MessagingDuty> {
        use NodeCmd::*;
        use NodeTransferCmd::*;
        match self.actor.transfer(amount, to) {
            Ok(Some(event)) => {
                let applied = self.actor.apply(TransferInitiated(event.clone()));
                if applied.is_err() {
                    // This would be a bug!
                    // send some error, log, crash .. or something
                    None
                } else {
                    self.wrapping.send(Message::NodeCmd {
                        cmd: Transfers(InitiateRewardPayout(event.signed_transfer)),
                        id: MessageId::new(),
                    })
                }
            }
            Ok(None) => None,
            Err(_error) => None, // for now, but should give NodeCmdError
        }
    }

    pub fn receive(&mut self, validation: TransferValidated) -> Option<MessagingDuty> {
        use NodeCmd::*;
        use NodeTransferCmd::*;
        match self.actor.receive(validation) {
            Ok(Some(event)) => {
                let applied = self.actor.apply(TransferValidationReceived(event.clone()));
                if applied.is_err() {
                    // This would be a bug!
                    // send some error, log, crash .. or something
                    None
                } else {
                    self.wrapping.send(Message::NodeCmd {
                        cmd: Transfers(FinaliseRewardPayout(event.proof?)),
                        id: MessageId::new(),
                    })
                }
            }
            Ok(None) => None,
            Err(_error) => None, // for now, but should give NodeCmdError
        }
    }
}
