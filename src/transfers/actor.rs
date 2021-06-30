// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    wallet::Wallet, ActorEvent, Error, Outcome, Result, StateSynched, TernaryResult,
    TransferInitiated, TransferRegistrationSent, TransferValidated, TransferValidationReceived,
    TransfersSynched,
};
use crate::types::{
    ActorHistory, Credit, CreditAgreementProof, CreditId, Debit, DebitId, OwnerType, PublicKey,
    SectionElders, SignatureShare, SignedCredit, SignedDebit, Signing, Token,
    TransferAgreementProof, WalletHistory,
};
use bls::PublicKeySet;
use crdts::Dot;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use tracing::debug;

/// The Actor is the part of an AT2 system
/// that initiates transfers, by requesting Replicas
/// to validate them, and then receive the proof of agreement.
/// It also syncs transfers from the Replicas.
#[derive(Clone)]
pub struct Actor<S: Signing> {
    ///
    id: OwnerType,
    ///
    signing: S,
    /// Set of all transfers impacting a given identity
    wallet: Wallet,
    /// Ensures that the actor's transfer
    /// initiations (ValidateTransfer cmd) are sequential.
    next_expected_debit: u64,
    /// When a transfer is initiated, validations are accumulated here.
    /// After quorum is reached and proof produced, the set is cleared.
    accumulating_validations: HashMap<DebitId, HashMap<usize, TransferValidated>>,
    /// The PK Set of the Replicas
    replicas: SectionElders,
    /// A log of applied events.
    history: ActorHistory,
}

impl<S: Signing> Actor<S> {
    /// Use this ctor for a new instance,
    /// or to rehydrate from events ([see the synch method](Actor::synch)).
    /// Pass in the key set of the replicas of this actor, i.e. our replicas.
    /// Credits to our wallet are most likely debited at other replicas than our own (the sender's replicas),
    pub fn new(signing: S, replicas: SectionElders) -> Actor<S> {
        let id = signing.id();
        let wallet = Wallet::new(id.clone());
        Actor {
            id,
            signing,
            replicas,
            wallet,
            next_expected_debit: 0,
            accumulating_validations: Default::default(),
            history: ActorHistory::empty(),
        }
    }

    ///
    pub fn from_info(signing: S, info: WalletHistory) -> Result<Actor<S>> {
        let mut actor = Self::new(signing, info.replicas);
        match actor.from_history(info.history) {
            Ok(Some(event)) => actor.apply(ActorEvent::TransfersSynched(event))?,
            Ok(None) => {}
            Err(Error::NoActorHistory) => {
                // do nothing
            }
            Err(error) => return Err(error),
        }

        Ok(actor)
    }

    /// Temp, for test purposes
    pub fn from_snapshot(wallet: Wallet, signing: S, replicas: SectionElders) -> Actor<S> {
        let id = wallet.id().clone();
        Actor {
            id,
            signing,
            replicas,
            wallet,
            next_expected_debit: 0,
            accumulating_validations: Default::default(),
            history: ActorHistory::empty(),
        }
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Queries ----------------------------------
    /// -----------------------------------------------------------------

    /// Query for the id of the Actor.
    pub fn id(&self) -> PublicKey {
        self.id.public_key()
    }

    /// Query for the id of the Actor.
    pub fn owner(&self) -> &OwnerType {
        &self.id
    }

    /// Query for the balance of the Actor.
    pub fn balance(&self) -> Token {
        self.wallet.balance()
    }

    ///
    pub fn replicas_public_key(&self) -> PublicKey {
        PublicKey::Bls(self.replicas.key_set.public_key())
    }

    ///
    pub fn replicas(&self) -> SectionElders {
        self.replicas.clone()
    }

    /// History of credits and debits
    pub fn history(&self) -> ActorHistory {
        self.history.clone()
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Cmds -------------------------------------
    /// -----------------------------------------------------------------

    /// Step 1. Build a valid cmd for validation of a debit.
    pub fn transfer(
        &self,
        amount: Token,
        recipient: PublicKey,
        msg: String,
    ) -> Outcome<TransferInitiated> {
        if recipient == self.id() {
            return Outcome::rejected(Error::SameSenderAndRecipient);
        }

        let id = Dot::new(self.id(), self.wallet.next_debit());

        // ensures one debit is completed at a time
        if self.next_expected_debit != self.wallet.next_debit() {
            return Outcome::rejected(Error::DebitPending);
        }
        if self.next_expected_debit != id.counter {
            return Outcome::rejected(Error::DebitProposed);
        }
        if amount > self.balance() {
            return Outcome::rejected(Error::InsufficientBalance);
        }

        if amount == Token::from_nano(0) {
            return Outcome::rejected(Error::ZeroValueTransfer);
        }

        let debit = Debit { id, amount };
        let credit = Credit {
            id: debit.credit_id()?,
            recipient,
            amount,
            msg,
        };

        let actor_signature = self.signing.sign(&debit)?;
        let signed_debit = SignedDebit {
            debit,
            actor_signature,
        };
        let actor_signature = self.signing.sign(&credit)?;
        let signed_credit = SignedCredit {
            credit,
            actor_signature,
        };

        Outcome::success(TransferInitiated {
            signed_debit,
            signed_credit,
        })
    }

    /// Step 2. Receive validations from Replicas, aggregate the signatures.
    pub fn receive(&self, validation: TransferValidated) -> Outcome<TransferValidationReceived> {
        // Always verify signature first! (as to not leak any information).
        if self.verify(&validation).is_err() {
            debug!("Invalid signature in transfer/actor receive step.");
            return Err(Error::InvalidSignature);
        }
        debug!(">>>>Actor: Verified validation.");

        let signed_debit = &validation.signed_debit;
        let signed_credit = &validation.signed_credit;

        // check if credit and debit correspond
        if signed_credit.id() != &signed_debit.credit_id()? {
            return Err(Error::CreditDebitIdMismatch);
        }
        // check if validation was initiated by this actor
        if self.id() != signed_debit.sender() {
            return Err(Error::WrongValidationActor);
        }
        // check if expected this validation
        if self.next_expected_debit != signed_debit.id().counter + 1 {
            return Err(Error::OperationOutOfOrder(
                signed_debit.id().counter,
                self.next_expected_debit,
            ));
        }
        // check if already received
        if let Some(map) = self.accumulating_validations.get(&validation.id()) {
            if map.contains_key(&validation.replica_debit_sig.index) {
                return Err(Error::ValidatedAlready);
            }
        } else {
            return Err(Error::NoSetForDebitId(validation.id()));
        }

        debug!("Actor receive stepped passed all checks");

        // TODO: Cover scenario where replica keys might have changed during an ongoing transfer.
        let map = self
            .accumulating_validations
            .get(&validation.id())
            .ok_or_else(|| Error::NoSetForTransferId(validation.id()))?;

        let mut proof = None;

        // If the previous count of accumulated + current validation coming in here,
        // is greater than the threshold, then we have reached the numbers needed
        // to build the proof ( = threshold + 1).
        let agreed = map.len() + 1 > self.replicas.key_set.threshold()
            && self.replicas.key_set == validation.replicas;
        if agreed {
            let debit_bytes = match bincode::serialize(&signed_debit) {
                Err(_) => return Err(Error::Serialisation("Serialization Error".to_string())),
                Ok(data) => data,
            };
            let credit_bytes = match bincode::serialize(&signed_credit) {
                Err(_) => return Err(Error::Serialisation("Serialization Error".to_string())),
                Ok(data) => data,
            };

            // collect sig shares
            let debit_sig_shares: BTreeMap<_, _> = map
                .values()
                .chain(vec![&validation])
                .map(|v| v.replica_debit_sig.clone())
                .map(|s| (s.index, s.share))
                .collect();
            // collect sig shares
            let credit_sig_shares: BTreeMap<_, _> = map
                .values()
                .chain(vec![&validation])
                .map(|v| v.replica_credit_sig.clone())
                .map(|s| (s.index, s.share))
                .collect();

            // Combine shares to produce the main signature.
            let debit_sig = self
                .replicas
                .key_set
                .combine_signatures(&debit_sig_shares)
                .map_err(|_| Error::CannotAggregate)?;
            // Combine shares to produce the main signature.
            let credit_sig = self
                .replicas
                .key_set
                .combine_signatures(&credit_sig_shares)
                .map_err(|_| Error::CannotAggregate)?;

            let valid_debit = self
                .replicas
                .key_set
                .public_key()
                .verify(&debit_sig, debit_bytes);
            let valid_credit = self
                .replicas
                .key_set
                .public_key()
                .verify(&credit_sig, credit_bytes);

            // Validate the combined signatures. If the shares were valid, this can't fail.
            if valid_debit && valid_credit {
                proof = Some(TransferAgreementProof {
                    signed_debit: signed_debit.clone(),
                    debit_sig: crate::types::Signature::Bls(debit_sig),
                    signed_credit: signed_credit.clone(),
                    credit_sig: crate::types::Signature::Bls(credit_sig),
                    debiting_replicas_keys: self.replicas.key_set.clone(),
                });
            } // else, we have some corrupt data. (todo: Do we need to act on that fact?)
        }

        Outcome::success(TransferValidationReceived { validation, proof })
    }

    /// Step 3. Registration of an agreed transfer.
    /// (The actual sending of the registration over the wire is done by upper layer,
    /// only after that, the event is applied to the actor instance.)
    pub fn register(
        &self,
        transfer_proof: TransferAgreementProof,
    ) -> Outcome<TransferRegistrationSent> {
        // Always verify signature first! (as to not leak any information).
        if self.verify_transfer_proof(&transfer_proof).is_err() {
            return Err(Error::InvalidSignature);
        }
        if self.wallet.next_debit() == transfer_proof.id().counter {
            Outcome::success(TransferRegistrationSent { transfer_proof })
        } else {
            Err(Error::OperationOutOfOrder(
                transfer_proof.id().counter,
                self.wallet.next_debit(),
            ))
        }
    }

    ///
    pub fn synch(
        &self,
        balance: Token,
        debit_version: u64,
        credit_ids: HashSet<CreditId>,
    ) -> Outcome<StateSynched> {
        // todo: use WalletSnapshot, aggregate sigs
        Outcome::success(StateSynched {
            id: self.id(),
            balance,
            debit_version,
            credit_ids,
        })
    }

    /// Step xx. Continuously receiving credits from Replicas via push or pull model, decided by upper layer.
    /// The credits are most likely originating at an Actor whose Replicas are not the same as our Replicas.
    /// That means that the signature on the DebitAgreementProof, is that of some Replicas we don't know.
    /// What we do here is to validate replicas in upper layers
    /// for determining if this remote group of Replicas is indeed valid.
    ///
    /// This also ensures that we receive transfers initiated at other Actor instances (same id or other,
    /// i.e. with multiple instances of same Actor we can also sync debits made on other isntances).
    /// Todo: This looks to be handling the case when there is a transfer in flight from this client
    /// (i.e. self.next_expected_debit has been incremented, but transfer not yet accumulated).
    /// Just make sure this is 100% the case as well.
    ///
    /// NB: If a non-complete* set of debits has been provided, this Actor instance
    /// will still apply any credits, and thus be out of synch with its Replicas,
    /// as it will have a balance that is higher than at the Replicas.
    /// (*Non-complete means non-contiguous set or not starting immediately
    /// after current debit version.)
    pub fn from_history(&self, history: ActorHistory) -> Outcome<TransfersSynched> {
        if history.is_empty() {
            return Outcome::no_change();
        }
        // filter out any credits and debits already existing in current wallet
        let credits = self.validate_credits(&history.credits);
        let debits = self.validate_debits(&history.debits);
        if !credits.is_empty() || !debits.is_empty() {
            Outcome::success(TransfersSynched(ActorHistory { credits, debits }))
        } else {
            Err(Error::NoActorHistory) // TODO: the error is actually that credits and/or debits failed validation..
        }
    }

    fn validate_credits(&self, credits: &[CreditAgreementProof]) -> Vec<CreditAgreementProof> {
        let valid_credits: Vec<_> = credits
            .iter()
            .cloned()
            .unique_by(|e| *e.id())
            .filter(|_credit_proof| {
                #[cfg(feature = "simulated-payouts")]
                return true;

                #[cfg(not(feature = "simulated-payouts"))]
                self.verify_credit_proof(_credit_proof).is_ok()
            })
            .filter(|credit| self.id() == credit.recipient())
            .filter(|credit| !self.wallet.contains(&credit.id()))
            .collect();

        valid_credits
    }

    /// Filters out any debits already applied,
    /// and makes sure the returned set is a contiguous
    /// set of debits beginning immediately after current debit version.
    #[allow(clippy::explicit_counter_loop)]
    fn validate_debits(&self, debits: &[TransferAgreementProof]) -> Vec<TransferAgreementProof> {
        let mut debits: Vec<_> = debits
            .iter()
            .unique_by(|e| e.id())
            .filter(|transfer| self.id() == transfer.sender())
            .filter(|transfer| transfer.id().counter >= self.wallet.next_debit())
            .filter(|transfer| self.verify_transfer_proof(transfer).is_ok())
            .collect();

        debits.sort_by_key(|t| t.id().counter);

        let mut iter = 0;
        let mut valid_debits = vec![];
        for out in debits {
            let version = out.id().counter;
            let expected_version = iter + self.wallet.next_debit();
            if version != expected_version {
                break; // since it's sorted, if first is not matching, then no point continuing
            }
            valid_debits.push(out.clone());
            iter += 1;
        }

        valid_debits
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Mutation ---------------------------------
    /// -----------------------------------------------------------------

    /// Mutation of state.
    /// There is no validation of an event, it is assumed to have
    /// been properly validated before raised, and thus anything that breaks is a bug.
    pub fn apply(&mut self, event: ActorEvent) -> Result<()> {
        debug!("Transfer Actor {}: applying event {:?}", self.id(), event);

        match event {
            ActorEvent::TransferInitiated(e) => {
                self.next_expected_debit = e.id().counter + 1;
                let _ = self.accumulating_validations.insert(e.id(), HashMap::new());
                Ok(())
            }
            ActorEvent::TransferValidationReceived(e) => {
                match self.accumulating_validations.get_mut(&e.validation.id()) {
                    Some(map) => {
                        let _ = map.insert(e.validation.replica_debit_sig.index, e.validation);
                    }
                    None => return Err(Error::PendingTransferNotFound),
                }
                Ok(())
            }
            ActorEvent::TransferRegistrationSent(e) => {
                self.wallet
                    .apply_debit(e.transfer_proof.signed_debit.debit.clone())?;
                self.accumulating_validations.clear();
                self.history.debits.push(e.transfer_proof);
                Ok(())
            }
            ActorEvent::TransfersSynched(e) => {
                for credit in e.0.credits {
                    // append credits _before_ debits
                    self.wallet
                        .apply_credit(credit.signed_credit.credit.clone())?;
                    self.history.credits.push(credit);
                }
                for debit in e.0.debits {
                    // append debits _after_ credits
                    self.wallet.apply_debit(debit.signed_debit.debit.clone())?;
                    self.history.debits.push(debit);
                }
                self.next_expected_debit = self.wallet.next_debit();
                Ok(())
            }
            ActorEvent::StateSynched(e) => {
                self.wallet = Wallet::from(
                    self.owner().clone(),
                    e.balance,
                    e.debit_version,
                    e.credit_ids,
                );
                self.next_expected_debit = self.wallet.next_debit();
                Ok(())
            }
        }
        // consider event log, to properly be able to reconstruct state from restart
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Private methods --------------------------
    /// -----------------------------------------------------------------

    /// We verify that we signed the underlying cmd,
    /// and the replica signature against the pk set included in the event.
    /// Note that we use the provided pk set to verify the event.
    /// This might not be the way we want to do it.
    fn verify(&self, event: &TransferValidated) -> Result<()> {
        let signed_debit = &event.signed_debit;
        let signed_credit = &event.signed_credit;

        // Check that we signed this.
        if let error @ Err(_) = self.verify_is_our_transfer(signed_debit, signed_credit) {
            return error;
        }

        let valid_debit = self
            .verify_share(signed_debit, &event.replica_debit_sig, &event.replicas)
            .is_ok();
        let valid_credit = self
            .verify_share(signed_credit, &event.replica_credit_sig, &event.replicas)
            .is_ok();

        if valid_debit && valid_credit {
            Ok(())
        } else {
            Err(Error::InvalidSignature)
        }
    }

    // Check that the replica signature is valid per the provided public key set.
    // (if we only use this in one place we can move the content to that method)
    fn verify_share<T: serde::Serialize>(
        &self,
        item: T,
        replica_signature: &SignatureShare,
        replicas: &PublicKeySet,
    ) -> Result<()> {
        let sig_share = &replica_signature.share;
        let share_index = replica_signature.index;
        match bincode::serialize(&item) {
            Err(_) => Err(Error::Serialisation("Could not serialise item".into())),
            Ok(data) => {
                let verified = replicas
                    .public_key_share(share_index)
                    .verify(sig_share, data);
                if verified {
                    Ok(())
                } else {
                    Err(Error::InvalidSignature)
                }
            }
        }
    }

    /// Verify that this is a valid TransferAgreementProof over our cmd.
    fn verify_transfer_proof(&self, proof: &TransferAgreementProof) -> Result<()> {
        let signed_debit = &proof.signed_debit;
        let signed_credit = &proof.signed_credit;
        // Check that we signed this.
        if let error @ Err(_) = self.verify_is_our_transfer(signed_debit, signed_credit) {
            return error;
        }

        // Check that the proof corresponds to a/the public key set of our Replicas.
        let valid_debit = match bincode::serialize(&proof.signed_debit) {
            Err(_) => return Err(Error::Serialisation("Could not serialise debit".into())),
            Ok(data) => {
                let public_key = crate::types::PublicKey::Bls(self.replicas.key_set.public_key());
                public_key.verify(&proof.debit_sig, &data).is_ok()
            }
        };

        let valid_credit = match bincode::serialize(&proof.signed_credit) {
            Err(_) => return Err(Error::Serialisation("Could not serialise credit".into())),
            Ok(data) => {
                let public_key = crate::types::PublicKey::Bls(self.replicas.key_set.public_key());
                public_key.verify(&proof.credit_sig, &data).is_ok()
            }
        };

        if valid_debit && valid_credit {
            Ok(())
        } else {
            Err(Error::InvalidSignature)
        }
    }

    /// Verify that this is a valid ReceivedCredit.
    #[cfg(not(feature = "simulated-payouts"))]
    fn verify_credit_proof(&self, proof: &CreditAgreementProof) -> Result<()> {
        let debiting_replicas_keys = PublicKey::Bls(proof.debiting_replicas_keys.public_key());

        debug!("Verfying debiting_replicas_sig..!");
        // Check that the proof corresponds to a/the public key set of our Replicas.
        match bincode::serialize(&proof.signed_credit) {
            Err(_) => Err(Error::Serialisation("Could not serialise credit".into())),
            Ok(data) => debiting_replicas_keys
                .verify(&proof.debiting_replicas_sig, &data)
                .map_err(Error::NetworkDataError),
        }
    }

    /// Check that we signed this.
    fn verify_is_our_transfer(
        &self,
        signed_debit: &SignedDebit,
        signed_credit: &SignedCredit,
    ) -> Result<()> {
        debug!("Actor: Verifying is this our transfer?!");
        let valid_debit = self
            .signing
            .verify(&signed_debit.actor_signature, &signed_debit.debit);
        let valid_credit = self
            .signing
            .verify(&signed_credit.actor_signature, &signed_credit.credit);

        if !(valid_debit && valid_credit) {
            debug!(
                "Actor: Valid debit sig? {}, Valid credit sig? {}",
                valid_debit, valid_credit
            );
            Err(Error::InvalidSignature)
        } else if signed_credit.id() != &signed_debit.credit_id()? {
            Err(Error::CreditDebitIdMismatch)
        } else {
            Ok(())
        }
    }
}

impl<S: Signing + fmt::Debug> fmt::Debug for Actor<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Actor {{ id: {:?}, signing: {:?}, wallet: {:?}, next_expected_debit: {:?}, accumulating_validations: {:?}, replicas: PkSet {{ public_key: {:?} }}}}",
            self.id,
            self.signing,
            self.wallet,
            self.next_expected_debit,
            self.accumulating_validations,
            self.replicas.key_set.public_key(),
        )
    }
}

#[cfg(test)]
mod test {
    use super::{
        Actor, ActorEvent, Error, OwnerType, Result, TransferInitiated, TransferRegistrationSent,
        Wallet,
    };
    use crate::types::{
        Credit, Debit, Keypair, PublicKey, SectionElders, Signature, SignatureShare, Token,
        TransferAgreementProof, TransferValidated,
    };
    use bls::{SecretKey, SecretKeySet};
    use crdts::Dot;
    use serde::Serialize;
    use std::collections::BTreeMap;
    use xor_name::Prefix;

    #[test]
    fn creates_actor() -> Result<()> {
        // Act
        let (_actor, _sk_set) = get_actor_and_replicas_sk_set(10)?;
        Ok(())
    }

    #[test]
    fn initial_state_is_applied() -> Result<()> {
        // Act
        let initial_amount = 10;
        let (actor, _sk_set) = get_actor_and_replicas_sk_set(initial_amount)?;
        assert_eq!(actor.balance(), Token::from_nano(initial_amount));
        Ok(())
    }

    #[test]
    fn initiates_transfers() -> Result<()> {
        // Act
        let (actor, _sk_set) = get_actor_and_replicas_sk_set(10)?;
        let debit = get_debit(&actor)?;
        let mut actor = actor;
        actor.apply(ActorEvent::TransferInitiated(debit))?;
        Ok(())
    }

    #[test]
    fn cannot_initiate_0_value_transfers() -> anyhow::Result<()> {
        let (actor, _sk_set) = get_actor_and_replicas_sk_set(10)?;

        match actor.transfer(Token::from_nano(0), get_random_pk(), "asfd".to_string()) {
            Ok(_) => Err(anyhow::anyhow!(
                "Should not be able to send 0 value transfers",
            )),
            Err(error) => {
                assert!(error
                    .to_string()
                    .contains("Transfer amount must be greater than zero"));
                Ok(())
            }
        }
    }

    #[test]
    fn can_apply_completed_transfer() -> Result<()> {
        // Act
        let (actor, sk_set) = get_actor_and_replicas_sk_set(15)?;
        let debit = get_debit(&actor)?;
        let mut actor = actor;
        actor.apply(ActorEvent::TransferInitiated(debit.clone()))?;
        let transfer_event = get_transfer_registration_sent(debit, &sk_set)?;
        actor.apply(ActorEvent::TransferRegistrationSent(transfer_event))?;
        assert_eq!(Token::from_nano(5), actor.balance());
        Ok(())
    }

    #[test]
    fn can_apply_completed_transfers_in_succession() -> Result<()> {
        // Act
        let (actor, sk_set) = get_actor_and_replicas_sk_set(22)?;
        let debit = get_debit(&actor)?;
        let mut actor = actor;
        actor.apply(ActorEvent::TransferInitiated(debit.clone()))?;
        let transfer_event = get_transfer_registration_sent(debit, &sk_set)?;
        actor.apply(ActorEvent::TransferRegistrationSent(transfer_event))?;

        assert_eq!(Token::from_nano(12), actor.balance()); // 22 - 10

        let debit2 = get_debit(&actor)?;
        actor.apply(ActorEvent::TransferInitiated(debit2.clone()))?;
        let transfer_event = get_transfer_registration_sent(debit2, &sk_set)?;
        actor.apply(ActorEvent::TransferRegistrationSent(transfer_event))?;

        assert_eq!(Token::from_nano(2), actor.balance()); // 22 - 10 - 10
        Ok(())
    }

    #[allow(clippy::needless_range_loop)]
    #[test]
    fn can_return_proof_for_validated_transfers() -> Result<()> {
        let (actor, sk_set) = get_actor_and_replicas_sk_set(22)?;
        let debit = get_debit(&actor)?;
        let mut actor = actor;
        actor.apply(ActorEvent::TransferInitiated(debit.clone()))?;
        let validations = get_transfer_validation_vec(debit, &sk_set)?;

        // 7 elders and validations
        for i in 0..7 {
            let transfer_validation = actor
                .receive(validations[i].clone())?
                .ok_or(Error::ReceiveValidationFailed)?;

            if i < 1
            // threshold is 1
            {
                assert_eq!(transfer_validation.clone().proof, None);
            } else {
                assert_ne!(transfer_validation.proof, None);
            }

            actor.apply(ActorEvent::TransferValidationReceived(
                transfer_validation.clone(),
            ))?;
        }
        Ok(())
    }

    fn get_debit(actor: &Actor<Keypair>) -> Result<TransferInitiated> {
        let event = actor
            .transfer(Token::from_nano(10), get_random_pk(), "asdf".to_string())?
            .ok_or(Error::TransferCreationFailed)?;
        Ok(event)
    }

    fn try_serialize<T: Serialize>(value: T) -> Result<Vec<u8>> {
        match bincode::serialize(&value) {
            Ok(res) => Ok(res),
            _ => Err(Error::Serialisation("Serialisation error".to_string())),
        }
    }

    /// returns a vec of validated transfers from the sk_set 'replicas'
    fn get_transfer_validation_vec(
        transfer: TransferInitiated,
        sk_set: &SecretKeySet,
    ) -> Result<Vec<TransferValidated>> {
        let signed_debit = transfer.signed_debit;
        let signed_credit = transfer.signed_credit;
        let serialized_signed_debit = try_serialize(&signed_debit)?;
        let serialized_signed_credit = try_serialize(&signed_credit)?;

        let sk_shares: Vec<_> = (0..7).map(|i| sk_set.secret_key_share(i)).collect();
        let pk_set = sk_set.public_keys();

        let debit_sig_shares: BTreeMap<_, _> = (0..7)
            .map(|i| (i, sk_shares[i].sign(serialized_signed_debit.clone())))
            .collect();
        let credit_sig_shares: BTreeMap<_, _> = (0..7)
            .map(|i| (i, sk_shares[i].sign(serialized_signed_credit.clone())))
            .collect();

        let mut validated_transfers = vec![];

        for i in 0..7 {
            let debit_sig_share = &debit_sig_shares[&i];
            let credit_sig_share = &credit_sig_shares[&i];
            assert!(pk_set
                .public_key_share(i)
                .verify(debit_sig_share, serialized_signed_debit.clone()));
            assert!(pk_set
                .public_key_share(i)
                .verify(credit_sig_share, serialized_signed_credit.clone()));
            validated_transfers.push(TransferValidated {
                signed_debit: signed_debit.clone(),
                signed_credit: signed_credit.clone(),
                replica_debit_sig: SignatureShare {
                    index: i,
                    share: debit_sig_share.clone(),
                },
                replica_credit_sig: SignatureShare {
                    index: i,
                    share: credit_sig_share.clone(),
                },
                replicas: pk_set.clone(),
            })
        }

        Ok(validated_transfers)
    }

    fn get_transfer_registration_sent(
        transfer: TransferInitiated,
        sk_set: &SecretKeySet,
    ) -> Result<TransferRegistrationSent> {
        let signed_debit = transfer.signed_debit;
        let signed_credit = transfer.signed_credit;
        let serialized_signed_debit = try_serialize(&signed_debit)?;
        let serialized_signed_credit = try_serialize(&signed_credit)?;

        let sk_shares: Vec<_> = (0..7).map(|i| sk_set.secret_key_share(i)).collect();
        let pk_set = sk_set.public_keys();

        let debit_sig_shares: BTreeMap<_, _> = (0..7)
            .map(|i| (i, sk_shares[i].sign(serialized_signed_debit.clone())))
            .collect();
        let credit_sig_shares: BTreeMap<_, _> = (0..7)
            .map(|i| (i, sk_shares[i].sign(serialized_signed_credit.clone())))
            .collect();

        // Combine them to produce the main signature.
        let debit_sig = match pk_set.combine_signatures(&debit_sig_shares) {
            Ok(s) => s,
            _ => return Err(Error::InvalidSignature),
        };
        let credit_sig = match pk_set.combine_signatures(&credit_sig_shares) {
            Ok(s) => s,
            _ => return Err(Error::InvalidSignature),
        };

        // Validate the main signature. If the shares were valid, this can't fail.
        assert!(pk_set
            .public_key()
            .verify(&debit_sig, serialized_signed_debit));
        assert!(pk_set
            .public_key()
            .verify(&credit_sig, serialized_signed_credit));

        let debit_sig = Signature::Bls(debit_sig);
        let credit_sig = Signature::Bls(credit_sig);
        let transfer_agreement_proof = TransferAgreementProof {
            signed_debit,
            signed_credit,
            debit_sig,
            credit_sig,
            debiting_replicas_keys: pk_set,
        };

        Ok(TransferRegistrationSent {
            transfer_proof: transfer_agreement_proof,
        })
    }

    fn get_actor_and_replicas_sk_set(amount: u64) -> Result<(Actor<Keypair>, SecretKeySet)> {
        let mut rng = rand::thread_rng();
        let keypair = Keypair::new_ed25519(&mut rng);
        let client_pubkey = keypair.public_key();
        let bls_secret_key = SecretKeySet::random(1, &mut rng);
        let replicas_id = bls_secret_key.public_keys();
        let balance = Token::from_nano(amount);
        let sender = Dot::new(get_random_pk(), 0);
        let credit = get_credit(sender, client_pubkey, balance)?;
        let mut wallet = Wallet::new(OwnerType::Single(credit.recipient()));
        wallet.apply_credit(credit)?;

        let replicas = SectionElders {
            prefix: Prefix::default(),
            names: Default::default(),
            key_set: replicas_id,
        };

        let actor = Actor::from_snapshot(wallet, keypair, replicas);
        Ok((actor, bls_secret_key))
    }

    fn get_credit(from: Dot<PublicKey>, recipient: PublicKey, amount: Token) -> Result<Credit> {
        let debit = Debit { id: from, amount };
        Ok(Credit {
            id: debit.credit_id()?,
            recipient,
            amount,
            msg: "asdf".to_string(),
        })
    }

    #[allow(unused)]
    fn get_random_dot() -> Dot<PublicKey> {
        Dot::new(get_random_pk(), 0)
    }

    fn get_random_pk() -> PublicKey {
        PublicKey::from(SecretKey::random().public_key())
    }
}
