use crate::utils;
use log::{error, info};
use routing::Node;
use safe_nd::{
    IDataAddress, Message, MessageId, MsgEnvelope, MsgSender, Signature, SignatureShare, XorName,
};
use std::cell::RefCell;
use std::collections::{hash_map::Entry, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
//use threshold_crypto::{Signature, SignatureShare};

type RequestInfo = (MsgEnvelope, MsgSender, Vec<SignatureShare>);
type DuplicationInfo = (IDataAddress, BTreeSet<XorName>, Vec<SignatureShare>);

pub struct Accumulator {
    routing_node: Rc<RefCell<Node>>,
    messages: HashMap<MessageId, RequestInfo>,
    duplications: HashMap<MessageId, DuplicationInfo>,
    completed: HashSet<MessageId>,
}

impl Accumulator {
    pub fn new(routing_node: Rc<RefCell<Node>>) -> Self {
        Self {
            routing_node,
            messages: Default::default(),
            duplications: Default::default(),
            completed: Default::default(),
        }
    }

    pub(crate) fn accumulate_cmd(&mut self, msg: &MsgEnvelope) -> Option<(MsgEnvelope, Signature)> {
        let msg_id = msg.id();
        if self.completed.contains(&msg_id) {
            info!("Message already processed.");
            return None;
        }
        match msg.message {
            Message::Cmd { .. } => {
                let signature = match msg.most_recent_sender() {
                    MsgSender::Client { .. } => return None, // this is just bad, we shouldn't be here, so this is not a good return value
                    MsgSender::Node {
                        signature: Signature::BlsShare(share),
                        ..
                    }
                    | MsgSender::Section {
                        signature: Signature::BlsShare(share),
                        ..
                    } => share,
                    _ => return None, // no other variation is valid here (yet)
                };
                // If the request comes from a node, the signatures have already
                // been accumulated at the Adult before the data is requested for duplication.
                // if let MsgSender::Node { duty: Duty::Adult(_), .. } = &msg.most_recent_sender() {
                //     Some((msg, (signature.1).0))
                // } else {
                match self.messages.entry(msg_id) {
                    Entry::Vacant(entry) => {
                        let _ = entry.insert((msg.clone(), msg.origin.clone(), vec![signature.clone()]));
                    }
                    Entry::Occupied(mut entry) => {
                        let (_, _, signatures) = entry.get_mut();
                        signatures.push(signature.clone());
                    }
                }

                let (_, _, signatures) = self.messages.get(&msg_id)?;
                // NB: This is wrong! pk set should come with the sig share.
                // use routing::ProofShare etc.
                let public_key_set = self.routing_node.borrow().public_key_set().ok()?.clone();
                info!(
                    "Got {} signatures. We need {}",
                    signatures.len(),
                    public_key_set.threshold() + 1
                );
                if signatures.len() > public_key_set.threshold() {
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
                        //let id = safe_nd::PublicKey::Bls(public_key_set.public_key());
                        let signature = safe_nd::Signature::Bls(signature);
                        return Some((msg.clone(), signature));
                    } else {
                        error!("Accumulated signature is invalid");
                    }
                }
                None
            }
            _ => None, // only client originating cmds are accumulated here
        }

        // match msg {
        //     Message::Request {
        //         request,
        //         requester,
        //         message_id,
        //         signature,
        //     } => {
        //         if let Some(signature) = signature {
        //             // If the request comes from a node, the signatures have already
        //             // been accumulated at the Adult before the data is requested for duplication.
        //             if let Request::Node(NodeRequest::Read(Read::Blob(_))) = &request {
        //                 Some((
        //                     Message::Request {
        //                         request,
        //                         requester,
        //                         message_id,
        //                         signature: None,
        //                     },
        //                     (signature.1).0,
        //                 ))
        //             } else {
        //                 match self.messages.entry(message_id) {
        //                     Entry::Vacant(entry) => {
        //                         let _ = entry.insert((request, requester, vec![signature]));
        //                     }
        //                     Entry::Occupied(mut entry) => {
        //                         let (_, _, signatures) = entry.get_mut();
        //                         signatures.push(signature);
        //                     }
        //                 }

        //                 let (_, _, signatures) = self.messages.get(&message_id)?;
        //                 let public_key_set =
        //                     self.routing_node.borrow().public_key_set().ok()?.clone();
        //                 info!(
        //                     "Got {} signatures. We need {}",
        //                     signatures.len(),
        //                     public_key_set.threshold() + 1
        //                 );
        //                 if signatures.len() > public_key_set.threshold() {
        //                     let (request, requester, signatures) =
        //                         self.messages.remove(&message_id)?;
        //                     let signed_data = utils::serialise(&request);
        //                     for (idx, sig) in &signatures {
        //                         if !public_key_set
        //                             .public_key_share(idx)
        //                             .verify(sig, &signed_data)
        //                         {
        //                             error!("Invalid signature share");
        //                             return None;
        //                         }
        //                     }
        //                     let signature = public_key_set
        //                         .combine_signatures(signatures.iter().map(|(i, sig)| (*i, sig)))
        //                         .ok()?;
        //                     if public_key_set.public_key().verify(&signature, &signed_data) {
        //                         let _ = self.completed.insert(message_id);
        //                         return Some((
        //                             Message::Request {
        //                                 request,
        //                                 requester,
        //                                 message_id,
        //                                 signature: None,
        //                             },
        //                             signature,
        //                         ));
        //                     } else {
        //                         error!("Accumulated signature is invalid");
        //                     }
        //                 }
        //                 None
        //             }
        //         } else {
        //             None
        //         }
        //     }
        //     Message::Duplicate {
        //         address,
        //         holders,
        //         message_id,
        //         signature,
        //     } => {
        //         if let Some(signature) = signature {
        //             match self.duplications.entry(message_id) {
        //                 Entry::Vacant(entry) => {
        //                     let _ = entry.insert((address, holders, vec![signature]));
        //                 }
        //                 Entry::Occupied(mut entry) => {
        //                     let (_, _, signatures) = entry.get_mut();
        //                     signatures.push(signature);
        //                 }
        //             }
        //         }

        //         let (_, _, signatures) = self.duplications.get(&message_id)?;
        //         let public_key_set = self.routing_node.borrow().public_key_set().ok()?.clone();
        //         info!(
        //             "Got {} signatures. We need {}",
        //             signatures.len(),
        //             public_key_set.threshold() + 1
        //         );
        //         if signatures.len() > public_key_set.threshold() {
        //             let (address, holders, signatures) = self.duplications.remove(&message_id)?;
        //             log::debug!("{:?}", public_key_set);
        //             let signed_data = utils::serialise(&address);
        //             for (idx, sig) in &signatures {
        //                 if !public_key_set
        //                     .public_key_share(idx)
        //                     .verify(sig, &signed_data)
        //                 {
        //                     error!("Invalid signature share");
        //                     return None;
        //                 }
        //             }
        //             let signature = public_key_set
        //                 .combine_signatures(signatures.iter().map(|(i, sig)| (*i, sig)))
        //                 .ok()?;
        //             if public_key_set.public_key().verify(&signature, &signed_data) {
        //                 let _ = self.completed.insert(message_id);
        //                 return Some((
        //                     Message::Duplicate {
        //                         address,
        //                         holders,
        //                         message_id,
        //                         signature: None,
        //                     },
        //                     signature,
        //                 ));
        //             } else {
        //                 error!("Accumulated signature is invalid");
        //             }
        //         }
        //         None
        //     }
        //     _ => {
        //         error!("Should not accumulate");
        //         None
        //     }
        // }
    }
}
