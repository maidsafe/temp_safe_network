
pub mod client_sender;
pub mod network_sender;
pub mod receiver;

use crate::node::node_ops::{NodeOperation, NodeDuty, MessagingDuty};
use client_sender::ClientSender;
use network_sender::NetworkSender;
use routing::Node as Routing;
use std::{
    cell::RefCell,
    rc::Rc,
};

pub struct Messaging {
    client_sender: ClientSender,
    network_sender: NetworkSender,
}

impl Messaging {

    pub fn new(routing: Rc<RefCell<Routing>>) -> Self {
        let client_sender = ClientSender::new(routing.clone());
        let network_sender = NetworkSender::new(routing.clone());
        Self { 
            client_sender, 
            network_sender 
        }
    }

    pub fn process(&mut self, duty: MessagingDuty) -> Option<NodeOperation> {
        use MessagingDuty::*;

        let result = match duty {
            SendToClient { address, msg } => self.client_sender.send(address, &msg),
            SendToNode(msg) => self.network_sender.send_to_node(msg),
            SendToSection(msg) => self.network_sender.send_to_network(msg),
            SendToAdults {
                targets,
                msg,
            } => self.network_sender.send_to_nodes(targets, &msg),
            VoteFor(decision) => self.network_sender.vote_for(decision),
            SendHandshake { address, response } => self.client_sender.handshake(address, &response),
            DisconnectClient(address) => self.client_sender.disconnect(address),
        };
        use NodeDuty::*;
        use NodeOperation::*;

        result.map(|c| RunAsNode(ProcessMessaging(c)))
    }
}