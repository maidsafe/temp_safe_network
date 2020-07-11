use super::validator::Validator;
use crate::{cmd::NodeCmd, utils};
use routing::Node as Routing;
use safe_nd::{
    AccountId, Duty, ElderDuty, Message, MessageId, Money, MsgEnvelope, MsgSender, NetworkCmd,
    PublicKey, TransferValidated,
};
use safe_transfers::{ActorEvent, TransferActor};
use std::{cell::RefCell, rc::Rc};

pub(super) struct SectionFunds {
    id: PublicKey,
    actor: TransferActor<Validator>,
    routing: Rc<RefCell<Routing>>,
}

impl SectionFunds {
    pub fn new(
        id: PublicKey,
        actor: TransferActor<Validator>,
        routing: Rc<RefCell<Routing>>,
    ) -> Self {
        Self { id, actor, routing }
    }

    pub fn initiate_reward_payout(&mut self, amount: Money, to: AccountId) -> Option<NodeCmd> {
        match self.actor.transfer(amount, to) {
            Ok(Some(event)) => {
                self.actor.apply(ActorEvent::TransferInitiated(event));
                let message = Message::NetworkCmd {
                    cmd: NetworkCmd::InitiateRewardPayout(event.signed_transfer),
                    id: MessageId::new(),
                };
                let signature = utils::sign(self.routing.borrow(), &utils::serialise(&message))?;
                let msg = MsgEnvelope {
                    message,
                    origin: MsgSender::Node {
                        id: self.id,
                        duty: Duty::Elder(ElderDuty::Rewards),
                        signature,
                    },
                    proxies: Default::default(),
                };
                Some(NodeCmd::SendToSection(msg))
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
                self.actor
                    .apply(ActorEvent::TransferValidationReceived(event));
                let proof = event.proof?;
                let message = Message::NetworkCmd {
                    cmd: NetworkCmd::FinaliseRewardPayout(proof),
                    id: MessageId::new(),
                };
                let signature = utils::sign(self.routing.borrow(), &utils::serialise(&message))?;
                let msg = MsgEnvelope {
                    message,
                    origin: MsgSender::Node {
                        id: self.id,
                        duty: Duty::Elder(ElderDuty::Rewards),
                        signature,
                    },
                    proxies: Default::default(),
                };
                Some(NodeCmd::SendToSection(msg))
            }
            Ok(None) => None,
            Err(error) => {
                None // for now, but should give NetworkCmdError
            }
        }
    }
}
