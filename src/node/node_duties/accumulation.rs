use crate::utils;
use log::{error, info};
use routing::Node;
use safe_nd::{
    BlobAddress, Message, MessageId, MsgEnvelope, MsgSender, Signature, SignatureShare, XorName,
};
use std::cell::RefCell;
use std::collections::{hash_map::Entry, BTreeSet, HashMap, HashSet};
use std::rc::Rc;

type RequestInfo = (MsgEnvelope, MsgSender, Vec<SignatureShare>);
type DuplicationInfo = (BlobAddress, BTreeSet<XorName>, Vec<SignatureShare>);

pub struct Accumulation {
    routing_node: Rc<RefCell<Node>>,
    messages: HashMap<MessageId, RequestInfo>,
    duplications: HashMap<MessageId, DuplicationInfo>,
    completed: HashSet<MessageId>,
}

impl Accumulation {
    pub fn new(routing_node: Rc<RefCell<Node>>) -> Self {
        Self {
            routing_node,
            messages: Default::default(),
            duplications: Default::default(),
            completed: Default::default(),
        }
    }

    pub(crate) fn process(&mut self, msg: &MsgEnvelope) -> Option<MsgEnvelope> {
        if self.completed.contains(&msg.id()) {
            info!("Message already processed.");
            return None;
        }
        let signature = match msg.most_recent_sender() {
            MsgSender::Node {
                signature: Signature::BlsShare(share),
                ..
            } => share,
            MsgSender::Section {
                signature: Signature::Bls(_),
                ..
            } => return Some(*msg), // already group signed, no need to accumulate (check sig though?, or somewhere else, earlier on?)
            _ => return None, // no other variation is valid
        };
        info!(
            "{}: Accumulating signatures for {:?}",
            "should be id here",
            //&id,
            msg.id()
        );
        match self.messages.entry(msg.id()) {
            Entry::Vacant(entry) => {
                let _ = entry.insert((msg.clone(), msg.origin.clone(), vec![signature.clone()]));
            }
            Entry::Occupied(mut entry) => {
                let (_, _, signatures) = entry.get_mut();
                signatures.push(signature.clone());
            }
        }
        self.try_aggregate(msg)
    }

    fn try_aggregate(&mut self, msg: &MsgEnvelope) -> Option<MsgEnvelope> {
        let msg_id = msg.id();
        let (_, _, signatures) = self.messages.get(&msg_id)?;

        // NB: This is wrong! pk set should come with the sig share.
        // use routing::ProofShare etc.

        // THIS IS WRONG v
        let public_key_set = self.routing_node.borrow().public_key_set().ok()?.clone();
        // THIS IS WRONG ^

        info!(
            "Got {} signatures. We need {}",
            signatures.len(),
            public_key_set.threshold() + 1
        );
        if public_key_set.threshold() >= signatures.len() {
            return None;
        }

        let (msg, _sender, signatures) = self.messages.remove(&msg_id)?;
        let signed_data = utils::serialise(&msg);
        for sig in &signatures {
            if !public_key_set
                .public_key_share(sig.index)
                .verify(&sig.share, &signed_data)
            {
                error!("Invalid signature share");
                return None;
            }
        }
        let signature = public_key_set
            .combine_signatures(signatures.iter().map(|sig| (sig.index, &sig.share)))
            .ok()?;
        if public_key_set.public_key().verify(&signature, &signed_data) {
            let _ = self.completed.insert(msg_id);

            // THIS IS WRONG v
            let id = safe_nd::PublicKey::Bls(public_key_set.public_key());
            // THIS IS WRONG ^

            let signature = safe_nd::Signature::Bls(signature);
            // upgrade sender to Section, since it accumulated
            let sender = match msg.most_recent_sender() {
                MsgSender::Node { duty, .. } => MsgSender::Section {
                    id,
                    duty: *duty,
                    signature,
                },
                _ => return None, // invalid use case, we only accumulate from Nodes
            };
            // Replace the Node with the Section.
            let _ = msg.proxies.pop();
            return Some(msg.with_proxy(sender));
        // beware that we might have to forgo the proxies vector
        // and instead just have a most recent proxy, if we are seeing
        // different order on the proxies on the msgs to be accumulated
        // (otherwise, the signature won't aggregate, since it is not over the same data)
        // perhaps it can be solved by ordering the vec, but maybe that defeats
        // part of the purpose; to see the path.
        } else {
            error!("Accumulated signature is invalid");
            None
        }
    }
}
