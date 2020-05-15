use super::Identity;
use crdts::{CmRDT, VClock};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use safe_nd::{
    Error, Money, ProofOfAgreement, RegisterTransfer, Result, Signature, Transfer,
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
    history: HashMap<Identity, Vec<Transfer>>,
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
            history: Default::default(),
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
        if self.history.len() > 0 {
            return Err(Error::InvalidOperation);
        }
        Ok(TransferPropagated { proof })
    }

    pub fn history(&self, identity: &Identity) -> Vec<Transfer> {
        self.history.get(&identity).cloned().unwrap_or_default()
    }

    pub fn balance(&self, identity: &Identity) -> Option<Money> {
        // todo: cache
        let h = self.history(identity);

        let outgoing = h
            .iter()
            .filter(|t| &t.id.actor == identity)
            .map(|t| t.amount.as_nano())
            .sum();
        let incoming = h
            .iter()
            .filter(|t| &t.to == identity)
            .map(|t| t.amount.as_nano())
            .sum();

        // We compute differences in a larger space since we need to move to signed numbers
        // and hence we lose a bit.
        let balance = Money::from_nano(incoming).checked_sub(Money::from_nano(outgoing));

        balance
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
        if !self.history.contains_key(&transfer.id.actor) {
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
        if !self.history.contains_key(&transfer.id.actor) {
            // this check could be redundant..
            return Err(Error::NoSuchSender); // ..also, if we actually reach here, there's probably some problem with the logic, that needs to be solved
        }
        if !self.is_sequential(transfer) {
            return Err(Error::InvalidOperation); // "Non-sequential operation"
        }
        Ok(TransferRegistered { proof })
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
        let _ = self.history.entry(key).or_default().push(transfer.clone());
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

    fn is_sequential(&self, transfer: &Transfer) -> bool {
        let id = transfer.id;
        let result = self.history.get(&id.actor);
        match result {
            None => return id.counter == 0, // zero based indexing, first transfer will be nr 0
            Some(sequence) => match sequence.last() {
                Some(previous) => previous.id.counter + 1 == id.counter,
                None => panic!("This would be a bug, we don't add empty collections here!"),
            },
        }
    }
}
