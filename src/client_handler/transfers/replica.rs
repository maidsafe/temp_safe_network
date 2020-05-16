use super::history::History;
use super::Identity;
use crdts::{CmRDT, VClock};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use safe_nd::{
    Error, Money, ProofOfAgreement, RegisterTransfer, Result, Signature, Transfer, TransferIndices,
    TransferRegistered, TransferValidated, ValidateTransfer,
};
use threshold_crypto::SignatureShare;

/// The Replica is the part of an AT2 system
/// that forms validating groups, and signs individual
/// Actors' transfers.
/// They validate incoming requests for transfer, and
/// apply operations that has a valid proof of agreement from the group.
/// Replicas don't initiate transfers or drive the algo - only Actors do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replica {
    id: Identity,
    // /// The PK Set of the section
    // pk_set: threshold_crypto::PublicKeySet, // temporary exclude
    /// Set of all transfers impacting a given identity
    histories: HashMap<Identity, History>,
    /// Ensures that invidual actors' transfer
    /// initiations (ValidateTransfer cmd) are sequential.
    pending_transfers: VClock<Identity>,
}

/// Events raised by the Replica.
#[derive(Clone, Hash, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub enum TransferEvent {
    /// The event raised when
    /// ValidateTransfer cmd has been successful.
    TransferValidated(TransferValidated),
    /// The event raised when
    /// RegisterTransfer cmd has been successful.
    TransferRegistered(TransferRegistered),
    /// The event raised when
    /// PropagateTransfer cmd has been successful.
    TransferPropagated(TransferPropagated),
}

/// The Elder event raised when
/// PropagateTransfer cmd has been successful.
/// Not part of the public contract, hence only used in this module.
#[derive(Clone, Hash, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct TransferPropagated {
    /// The transfer proof.
    pub proof: ProofOfAgreement,
}

impl Replica {
    pub fn new(id: Identity) -> Self {
        // , pk_set: threshold_crypto::PublicKeySet // temporary exclude
        Replica {
            id,
            // pk_set, // temporary exclude
            histories: Default::default(),
            pending_transfers: VClock::new(),
        }
    }

    /// This is the one and only infusion of money to the system. Ever.
    /// It is carried out by the first node in the network.
    /// WIP
    pub fn genesis(&self, cmd: RegisterTransfer) -> Result<TransferPropagated> {
        let proof = cmd.proof;
        // Always verify signature first! (as to not leak any information).
        if !self.verify_proof(&proof) {
            return Err(Error::InvalidSignature);
        }
        // genesis must be the first
        if self.histories.len() > 0 {
            return Err(Error::InvalidOperation);
        }
        Ok(TransferPropagated { proof })
    }

    pub fn history(
        &self,
        identity: &Identity,
        since_indices: TransferIndices,
    ) -> Option<(Vec<Transfer>, Vec<Transfer>)> {
        match self.histories.get(&identity).cloned() {
            None => None,
            Some(history) => Some(history.new_since(since_indices)),
        }
    }

    pub fn balance(&self, identity: &Identity) -> Option<Money> {
        let result = self.histories.get(identity);
        match result {
            None => None,
            Some(history) => Some(history.balance()),
        }
    }

    /// For now, with test money there is no from account.., money is created from thin air.
    pub fn test_validate_transfer(
        &self,
        transfer_cmd: ValidateTransfer,
    ) -> Result<TransferValidated> {
        let id = transfer_cmd.transfer.id;
        if id.actor == transfer_cmd.transfer.to {
            Err(Error::InvalidOperation)
        } else {
            let elder_signature = self.sign(&transfer_cmd);
            Ok(TransferValidated {
                transfer_cmd,
                elder_signature,
                //pk_set: self.pk_set, // temporary exclude
            })
        }
    }

    /// Main business logic validation of a debit.
    pub fn validate_transfer(&self, transfer_cmd: ValidateTransfer) -> Result<TransferValidated> {
        let transfer = &transfer_cmd.transfer;
        // Always verify signature first! (as to not leak any information).
        if !self.verify_cmd_signature(&transfer_cmd) {
            return Err(Error::InvalidSignature);
        }
        if transfer.id.actor == transfer.to {
            return Err(Error::InvalidOperation); // "Sender and recipient are the same"
        }
        if !self.histories.contains_key(&transfer.id.actor) {
            // println!(
            //     "{} sender does not exist (trying to transfer {} to {}).",
            return Err(Error::NoSuchSender);
        }
        if transfer.id != self.pending_transfers.inc(transfer.id.actor) {
            return Err(Error::InvalidOperation); // "either already proposed or out of order msg"
        }
        match self.balance(&transfer.id.actor) {
            Some(balance) => {
                if transfer.amount > balance {
                    // println!("{} does not have enough money to transfer {} to {}. (balance: {})"
                    return Err(Error::InsufficientBalance);
                }
            }
            None => return Err(Error::NoSuchSender), //"From account doesn't exist"
        }

        let elder_signature = self.sign(&transfer_cmd);
        Ok(TransferValidated {
            transfer_cmd,
            elder_signature,
            //pk_set: self.pk_set, // temporary exclude
        })
    }

    /// Validation of agreement, and order.
    pub fn register_transfer(&self, cmd: RegisterTransfer) -> Result<TransferRegistered> {
        let proof = cmd.proof;
        // Always verify signature first! (as to not leak any information).
        if !self.verify_proof(&proof) {
            return Err(Error::InvalidSignature);
        }
        let transfer = &proof.transfer_cmd.transfer;
        let id = transfer.id;
        let sender = self.histories.get(&id.actor);
        match sender {
            None => Err(Error::NoSuchSender),
            Some(history) => match history.is_sequential(transfer) {
                Ok(is_seq) => {
                    if is_seq {
                        Ok(TransferRegistered { proof })
                    } else {
                        Err(Error::InvalidOperation) // "Non-sequential operation"
                    }
                }
                Err(_) => Err(Error::InvalidOperation), // from this place this code won't happen, but history validates the transfer is actually outgoing from it's owner.
            },
        }
    }

    /// Validation of agreement.
    /// Since this leads to a credit, there is no requirement on order.
    pub fn propagate_transfer(&self, cmd: RegisterTransfer) -> Result<TransferPropagated> {
        let proof = cmd.proof;
        // Always verify signature first! (as to not leak any information).
        if !self.verify_proof(&proof) {
            return Err(Error::InvalidSignature);
        }
        Ok(TransferPropagated { proof })
    }

    pub fn apply(&mut self, event: TransferEvent) {
        match event {
            TransferEvent::TransferValidated(e) => {
                let transfer = e.transfer_cmd.transfer;
                self.pending_transfers.apply(transfer.id);
            }
            TransferEvent::TransferRegistered(e) => {
                let transfer = e.proof.transfer_cmd.transfer;
                self.append(transfer.id.actor, transfer);
            }
            TransferEvent::TransferPropagated(e) => {
                let transfer = e.proof.transfer_cmd.transfer;
                self.append(transfer.to, transfer);
            }
        };
    }

    // Extend the history for the key.
    fn append(&mut self, key: Identity, transfer: Transfer) {
        // Creates if not exists.
        match self.histories.get_mut(&key) {
            Some(history) => history.append(transfer),
            None => {
                let _ = self
                    .histories
                    .insert(key, History::new(key, transfer.clone()));
            }
        }
    }

    fn sign(&self, _cmd: &ValidateTransfer) -> SignatureShare {
        unimplemented!()
    }

    fn verify_cmd_signature(&self, _cmd: &ValidateTransfer) -> bool {
        unimplemented!()
    }

    fn verify_proof(&self, _proof: &ProofOfAgreement) -> bool {
        unimplemented!()
    }
}
