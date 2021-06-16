// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    wallet::{Wallet, WalletSnapshot},
    Error, Outcome, Result, TernaryResult,
};
use bls::{PublicKeySet, PublicKeyShare};
use log::{debug, error};
#[cfg(feature = "simulated-payouts")]
use sn_data_types::Credit;
use sn_data_types::{
    CreditAgreementProof, Debit, OwnerType, ReplicaEvent, Signature, SignedCredit, SignedDebit,
    SignedTransfer, SignedTransferShare, Token, TransferAgreementProof, TransferRegistered,
    TransferValidationProposed,
};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
            let mut map = ::std::collections::HashMap::new();
            $( let _ = map.insert($key, $val); )*
            map
    }}
}

/// The Replica is the part of an AT2 system
/// that forms validating groups, and signs
/// individual transfers between wallets.
/// Replicas validate requests to debit an wallet, and
/// apply operations that has a valid "debit agreement proof"
/// from the group, i.e. signatures from a quorum of its peers.
/// Replicas don't initiate transfers or drive the algo - only Actors do.
#[derive(Clone, PartialEq, Eq)]
pub struct WalletReplica {
    /// The owner of the Wallet.
    id: OwnerType,
    /// The public key share of this Replica.
    replica_id: PublicKeyShare,
    /// The index of this Replica key share, in the group set.
    key_index: usize,
    /// The PK set of our peer Replicas.
    peer_replicas: PublicKeySet,
    /// All wallets that this Replica validates transfers for.
    wallet: Wallet,
    /// For multisig validations.
    pending_proposals: HashMap<u64, HashMap<usize, TransferValidationProposed>>,
    /// Ensures that invidual wallet's debit
    /// initiations (ValidateTransfer cmd) are sequential.
    pending_debit: Option<u64>,
}

impl WalletReplica {
    /// A new Replica instance from a history of events.
    pub fn from_history(
        id: OwnerType,
        replica_id: PublicKeyShare,
        key_index: usize,
        peer_replicas: PublicKeySet,
        events: Vec<ReplicaEvent>,
    ) -> Result<Self> {
        let mut instance = Self::from_snapshot(
            id.clone(),
            replica_id,
            key_index,
            peer_replicas,
            Wallet::new(id),
            Default::default(),
            None,
        );

        for e in events {
            instance.apply(e)?;
        }

        Ok(instance)
    }

    /// A new Replica instance from current state.
    pub fn from_snapshot(
        id: OwnerType,
        replica_id: PublicKeyShare,
        key_index: usize,
        peer_replicas: PublicKeySet,
        wallet: Wallet,
        pending_proposals: HashMap<u64, HashMap<usize, TransferValidationProposed>>,
        pending_debit: Option<u64>,
    ) -> Self {
        Self {
            id,
            replica_id,
            key_index,
            peer_replicas,
            //other_groups,
            wallet,
            pending_proposals,
            pending_debit,
        }
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Queries ----------------------------------
    /// -----------------------------------------------------------------

    ///
    pub fn balance(&self) -> Token {
        self.wallet.balance()
    }

    ///
    pub fn wallet(&self) -> Option<WalletSnapshot> {
        let wallet = self.wallet.to_owned();
        Some(wallet.into())
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Cmds -------------------------------------
    /// -----------------------------------------------------------------

    /// This is the one and only infusion of money to the system. Ever.
    /// It is carried out by the first node in the network.
    pub fn genesis(&self, credit_proof: &CreditAgreementProof) -> Outcome<()> {
        // Genesis must be the first credit.
        if self.balance() != Token::zero() || self.pending_debit.is_some() {
            return Err(Error::InvalidOperation);
        }
        self.receive_propagated(credit_proof)
    }

    /// For now, with test token there is no from wallet.., token is created from thin air.
    pub fn test_validate_transfer(
        &self,
        signed_debit: &SignedDebit,
        signed_credit: &SignedCredit,
    ) -> Outcome<()> {
        if signed_debit.sender() == signed_credit.recipient() {
            Err(Error::SameSenderAndRecipient)
        } else if signed_credit.id() != &signed_debit.credit_id()? {
            Err(Error::CreditDebitIdMismatch)
        } else if signed_credit.amount() != signed_debit.amount() {
            Err(Error::CreditDebitValueMismatch)
        } else {
            Outcome::success(())
        }
    }

    /// Step 1. Main business logic validation of a debit.
    pub fn validate(
        &self,
        signed_debit: &SignedDebit,
        signed_credit: &SignedCredit,
    ) -> Outcome<()> {
        let debit = &signed_debit.debit;
        let credit = &signed_credit.credit;

        // Always verify signature first! (as to not leak any information).
        if self
            .verify_actor_signature(&signed_debit, &signed_credit)
            .is_err()
        {
            return Outcome::rejected(Error::InvalidSignature);
        } else if debit.sender() == credit.recipient() {
            return Outcome::rejected(Error::SameSenderAndRecipient);
        } else if credit.id() != &debit.credit_id()? {
            return Outcome::rejected(Error::CreditDebitIdMismatch);
        } else if credit.amount() != debit.amount() {
            return Outcome::rejected(Error::CreditDebitValueMismatch);
        } else if debit.amount() == Token::zero() {
            return Outcome::rejected(Error::ZeroValueTransfer);
        } else if self.wallet.id().public_key() != debit.sender() {
            return Outcome::rejected(Error::NoSuchSender);
        } else if self.pending_debit.is_none() && debit.id.counter != 0 {
            return Outcome::rejected(Error::ShouldBeInitialOperation);
        } else if let Some(counter) = self.pending_debit {
            if debit.id.counter != (counter + 1) {
                return Outcome::rejected(Error::OperationOutOfOrder(
                    debit.id.counter,
                    counter + 1,
                ));
            }
        } else if debit.amount() > self.balance() {
            return Outcome::rejected(Error::InsufficientBalance);
        }

        Outcome::success(())
    }

    /// Step 2. Validation of agreement, and order at debit source.
    pub fn register(&self, transfer_proof: &TransferAgreementProof) -> Outcome<TransferRegistered> {
        debug!("Checking registered transfer");

        // Always verify signature first! (as to not leak any information).
        if self.verify_registered_proof(transfer_proof).is_err() {
            return Err(Error::InvalidSignature);
        }

        let debit = &transfer_proof.signed_debit.debit;
        if self.wallet.next_debit() == debit.id().counter {
            Outcome::success(TransferRegistered {
                transfer_proof: transfer_proof.clone(),
            })
        } else {
            Outcome::rejected(Error::OperationOutOfOrder(
                debit.id().counter,
                self.wallet.next_debit(),
            ))
            // from this place this code won't happen, but history validates the transfer is actually debits from it's owner).
        }
    }

    /// Step 3. Validation of TransferAgreementProof, and credit idempotency at credit destination.
    /// (Since this leads to a credit, there is no requirement on order.)
    pub fn receive_propagated(&self, credit_proof: &CreditAgreementProof) -> Outcome<()> {
        // Always verify signature first! (as to not leak any information).
        self.verify_propagated_proof(credit_proof)?;
        if self.wallet.contains(&credit_proof.id()) {
            Outcome::no_change()
        } else {
            Outcome::success(())
        }
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Mutation ---------------------------------
    /// -----------------------------------------------------------------

    /// Mutation of state.
    /// There is no validation of an event, it (the cmd) is assumed to have
    /// been properly validated before the fact is established (event raised),
    /// and thus anything that breaks here, is a bug in the validation..
    pub fn apply(&mut self, event: ReplicaEvent) -> Result<()> {
        match event {
            ReplicaEvent::TransferValidationProposed(e) => {
                let debit = &e.signed_debit.debit;
                let index = e.signed_debit.actor_signature.index;
                if let Some(pending) = self.pending_proposals.get_mut(&debit.id.counter) {
                    let _ = pending.insert(index, e);
                } else {
                    let _ = self
                        .pending_proposals
                        .insert(debit.id.counter, hashmap!(index => e));
                };
                Ok(())
            }
            ReplicaEvent::TransferValidated(e) => {
                let debit = e.signed_debit.debit;
                self.pending_debit = Some(debit.id.counter);
                Ok(())
            }
            ReplicaEvent::TransferRegistered(e) => {
                let debit = e.transfer_proof.signed_debit.debit;
                self.wallet.apply_debit(Debit {
                    id: debit.id(),
                    amount: debit.amount(),
                })
            }
            ReplicaEvent::TransferPropagated(e) => {
                let credit = e.credit_proof.signed_credit.credit;
                self.wallet.apply_credit(credit)
            }
        }
    }

    /// Test-helper API to simulate Client CREDIT Transfers.
    #[cfg(feature = "simulated-payouts")]
    pub fn credit_without_proof(&mut self, credit: Credit) -> Result<()> {
        self.wallet.simulated_credit(credit)
    }

    /// Test-helper API to simulate Client DEBIT Transfers.
    #[cfg(feature = "simulated-payouts")]
    pub fn debit_without_proof(&mut self, debit: Debit) -> Result<()> {
        self.wallet.simulated_debit(debit)
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Private methods --------------------------
    /// -----------------------------------------------------------------

    ///
    fn verify_actor_signature(
        &self,
        signed_debit: &SignedDebit,
        signed_credit: &SignedCredit,
    ) -> Result<()> {
        println!("Actor signature verification");
        let debit = &signed_debit.debit;
        let credit = &signed_credit.credit;
        let debit_bytes = match bincode::serialize(&debit) {
            Err(_) => return Err(Error::Serialisation("Could not serialise debit".into())),
            Ok(bytes) => bytes,
        };
        let credit_bytes = match bincode::serialize(&credit) {
            Err(_) => return Err(Error::Serialisation("Could not serialise credit".into())),
            Ok(bytes) => bytes,
        };

        let valid_debit = signed_debit
            .sender()
            .verify(&signed_debit.actor_signature, debit_bytes)
            .is_ok();

        println!("Debit is valid?: {:?}", valid_debit);
        let valid_credit = signed_debit
            .sender()
            .verify(&signed_credit.actor_signature, credit_bytes)
            .is_ok();
        println!("Credit is valid?: {:?}", valid_debit);

        if valid_debit && valid_credit && credit.id() == &debit.credit_id()? {
            Ok(())
        } else {
            Err(Error::InvalidSignature)
        }
    }

    /// Verify that this is a valid _registered_
    /// TransferAgreementProof, i.e. signed by our peers.
    fn verify_registered_proof(&self, proof: &TransferAgreementProof) -> Result<()> {
        if proof.signed_credit.id() != &proof.signed_debit.credit_id()? {
            return Err(Error::CreditDebitValueMismatch);
        }
        // Check that the proof corresponds to a public key set of our peers.
        let debit_bytes = match bincode::serialize(&proof.signed_debit) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::Serialisation("Could not serialise transfer".into())),
        };
        let credit_bytes = match bincode::serialize(&proof.signed_credit) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::Serialisation("Could not serialise transfer".into())),
        };
        // Check if proof is signed by our peers.
        let public_key = sn_data_types::PublicKey::Bls(self.peer_replicas.public_key());
        let valid_debit = public_key.verify(&proof.debit_sig, &debit_bytes).is_ok();
        let valid_credit = public_key.verify(&proof.credit_sig, &credit_bytes).is_ok();
        if valid_debit && valid_credit {
            return Ok(());
        }

        // If it's not signed with our peers' public key, we won't consider it valid.
        Err(Error::InvalidSignature)
    }

    /// Verify the sig over the CreditAgreementProof.
    fn verify_propagated_proof(&self, proof: &CreditAgreementProof) -> Result<()> {
        match bincode::serialize(&proof.signed_credit) {
            Err(_) => Err(Error::Serialisation("Could not serialise transfer".into())),
            Ok(credit_bytes) => {
                let key = sn_data_types::PublicKey::Bls(proof.debiting_replicas_keys.public_key());
                key.verify(&proof.debiting_replicas_sig, &credit_bytes)
                    .map_err(|_| Error::InvalidSignature)
            }
        }
    }
}

impl WalletReplica {
    /// Step 1. Main business logic validation of a debit.
    pub fn propose_validation(
        &self,
        signed_transfer: &SignedTransferShare,
    ) -> Outcome<TransferValidationProposed> {
        let signed_debit = signed_transfer.debit();
        let signed_credit = signed_transfer.credit();
        let debit = &signed_debit.debit;

        debug!(
            "debit counter for this validation is: {:?}, the transfer is for: {:?}",
            debit.id.counter,
            signed_transfer.credit().amount()
        );

        let credit = &signed_credit.credit;

        // Always verify signature first! (as to not leak any information).
        if let Err(e) = self.verify_actor_signature_share(signed_transfer) {
            println!("Failed verification of actor sig!");
            return Outcome::rejected(e);
        } else if debit.sender() == credit.recipient() {
            return Outcome::rejected(Error::SameSenderAndRecipient);
        } else if credit.id() != &debit.credit_id()? {
            return Outcome::rejected(Error::CreditDebitIdMismatch);
        } else if credit.amount() != debit.amount() {
            return Outcome::rejected(Error::CreditDebitValueMismatch);
        } else if debit.amount() == Token::zero() {
            return Outcome::rejected(Error::ZeroValueTransfer);
        } else if self.id.public_key() != debit.sender() {
            return Outcome::rejected(Error::NoSuchSender);
        } else if self.pending_debit.is_none() && debit.id.counter != 0 {
            return Outcome::rejected(Error::ShouldBeInitialOperation);
        } else if let Some(counter) = self.pending_debit {
            if debit.id.counter != (counter + 1) {
                return Outcome::rejected(Error::OperationOutOfOrder(debit.id.counter, counter));
            }
        }

        debug!("Correct proposal.");
        debug!("Accumulating transfer validation proposal..");

        self.accumulate(TransferValidationProposed {
            signed_credit: signed_credit.to_owned(),
            signed_debit: signed_debit.to_owned(),
            agreed_transfer: None,
        })
    }

    /// Step 2. Receive validations from Replicas, aggregate the signatures.
    fn accumulate(
        &self,
        proposal: TransferValidationProposed,
    ) -> Outcome<TransferValidationProposed> {
        let actors = match &self.id {
            OwnerType::Multi(actors) => actors,
            OwnerType::Single(_) => return Outcome::rejected(Error::InvalidOwner),
        };
        let signed_debit = &proposal.signed_debit;
        let signed_credit = &proposal.signed_credit;
        let share_index = signed_debit.share_index();
        let id = signed_debit.id();
        let debit_counter = id.counter;

        // check if already received
        if let Some(map) = self.pending_proposals.get(&debit_counter) {
            if map.contains_key(&share_index) {
                return Outcome::no_change();
            }
        }

        let map = HashMap::new();
        let map = self.pending_proposals.get(&debit_counter).unwrap_or(&map);

        // If the previous count of accumulated + current proposal coming in here,
        // is greater than the threshold, then we have reached the numbers needed
        // to build the agreed_transfer (= threshold + 1).
        let agreed = map.len() + 1 > actors.threshold() && self.id.public_key() == id.actor;

        if !agreed {
            debug!("No agreement reached yet for proposal.");
            // No agreement reached yet,
            // so the proposal does not have a populated agreement field.
            return Outcome::success(proposal);
        }

        let debit_bytes = match bincode::serialize(&signed_debit.debit) {
            Err(_) => {
                return Err(Error::Serialisation(
                    "Could not serialise debit".to_string(),
                ))
            }
            Ok(data) => data,
        };
        let credit_bytes = match bincode::serialize(&signed_credit.credit) {
            Err(_) => {
                return Err(Error::Serialisation(
                    "Could not serialise credit".to_string(),
                ))
            }
            Ok(data) => data,
        };

        // collect debit sig shares
        let debit_sig_shares: BTreeMap<_, _> = map
            .values()
            .chain(vec![&proposal])
            .map(|v| v.signed_debit.actor_signature.clone())
            .map(|s| (s.index, s.share))
            .collect();
        // collect credit sig shares
        let credit_sig_shares: BTreeMap<_, _> = map
            .values()
            .chain(vec![&proposal])
            .map(|v| v.signed_credit.actor_signature.clone())
            .map(|s| (s.index, s.share))
            .collect();

        // Combine shares to produce the main signature.
        let debit_sig = actors
            .combine_signatures(&debit_sig_shares)
            .map_err(|_| Error::CannotAggregate)?;
        // Combine shares to produce the main signature.
        let credit_sig = actors
            .combine_signatures(&credit_sig_shares)
            .map_err(|_| Error::CannotAggregate)?;

        let valid_debit = actors.public_key().verify(&debit_sig, debit_bytes);
        let valid_credit = actors.public_key().verify(&credit_sig, credit_bytes);

        // Validate the combined signatures. If the shares were valid, this can't fail.
        if valid_debit && valid_credit {
            let mut proposal = proposal.clone();
            proposal.agreed_transfer = Some(SignedTransfer {
                debit: SignedDebit {
                    debit: signed_debit.debit.clone(),
                    actor_signature: Signature::Bls(debit_sig),
                },
                credit: SignedCredit {
                    credit: signed_credit.credit.clone(),
                    actor_signature: Signature::Bls(credit_sig),
                },
            });
            Outcome::success(proposal)
        } else {
            error!(
                "valid debit? {:?} : sender: {:?} recipient {:?}",
                valid_debit,
                proposal.sender(),
                proposal.recipient()
            );
            error!("valid credit? {:?}", valid_credit);
            // else, we have some corrupt data. (todo: Do we need to act on that fact?)
            Err(Error::InvalidCreditOrDebit)
        }
    }

    fn verify_actor_signature_share(
        &self,
        signed_transfer_share: &SignedTransferShare,
    ) -> Result<()> {
        debug!("Actor signature share verification");
        let signed_debit = signed_transfer_share.debit();
        let signed_credit = signed_transfer_share.credit();
        let debit = &signed_debit.debit;
        let credit = &signed_credit.credit;
        let debit_bytes = match bincode::serialize(debit) {
            Err(_) => return Err(Error::Serialisation("Could not serialise debit".into())),
            Ok(bytes) => bytes,
        };
        let credit_bytes = match bincode::serialize(credit) {
            Err(_) => return Err(Error::Serialisation("Could not serialise credit".into())),
            Ok(bytes) => bytes,
        };

        let key_share = signed_transfer_share
            .actors()
            .public_key_share(signed_debit.actor_signature.index);

        let valid_debit = key_share.verify(&signed_debit.actor_signature.share, &debit_bytes);
        let valid_credit = key_share.verify(&signed_credit.actor_signature.share, &credit_bytes);

        debug!("Debit is valid?: {:?}", valid_debit);
        debug!("Credit is valid?: {:?}", valid_debit);

        if credit.id() != &debit.credit_id()? {
            return Err(Error::CreditDebitIdMismatch);
        }
        if valid_debit && valid_credit {
            Ok(())
        } else {
            Err(Error::Unknown(format!(
                "InvalidSignature! valid_debit: {}, valid_credit: {}",
                valid_debit, valid_credit
            )))
        }
    }
}

impl fmt::Debug for WalletReplica {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WalletReplica {{ id: {:?}, replica_id: {:?}, key_index: {:?}, peer_replicas: PkSet {{ public_key: {:?} }}, wallet: {:?}, pending_proposals: {:?}, pending_debit: {:?} }}",
            self.id,
            self.replica_id,
            self.key_index,
            self.peer_replicas.public_key(),
            self.wallet,
            self.pending_proposals,
            self.pending_debit
        )
    }
}
