// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Implementation of Transfers in the SAFE Network.

mod actor;
mod error;
mod test_utils;
mod wallet;
mod wallet_replica;

pub use self::{
    actor::Actor as TransferActor, error::Error, wallet::Wallet, wallet_replica::WalletReplica,
};

use serde::{Deserialize, Serialize};
use sn_data_types::{
    ActorHistory, CreditId, DebitId, PublicKey, SignedCredit, SignedDebit, Token,
    TransferAgreementProof, TransferValidated,
};
use std::collections::HashSet;

type Result<T> = std::result::Result<T, Error>;
type Outcome<T> = Result<Option<T>>;

trait TernaryResult<T> {
    fn success(item: T) -> Self;
    fn no_change() -> Self;
    fn rejected(error: Error) -> Self;
}

impl<T> TernaryResult<T> for Outcome<T> {
    fn success(item: T) -> Self {
        Ok(Some(item))
    }
    fn no_change() -> Self {
        Ok(None)
    }
    fn rejected(error: Error) -> Self {
        Err(error)
    }
}

// ------------------------------------------------------------
//                      Actor
// ------------------------------------------------------------

/// Events raised by the Actor.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum ActorEvent {
    /// Raised when a request to create
    /// a transfer validation cmd for Replicas,
    /// has been successful (valid on local state).
    TransferInitiated(TransferInitiated),
    /// Raised when an Actor receives a Replica transfer validation.
    TransferValidationReceived(TransferValidationReceived),
    /// Raised when the Actor has accumulated a
    /// quorum of validations, and produced a RegisterTransfer cmd
    /// for sending to Replicas.
    TransferRegistrationSent(TransferRegistrationSent),
    /// Raised when the Actor has received
    /// unknown credits on querying Replicas.
    TransfersSynched(TransfersSynched),
    /// Raised when the Actor has received
    /// unknown credits on querying Replicas.
    StateSynched(StateSynched),
}

/// Raised when the Actor has received
/// f.ex. credits that its Replicas were holding upon
/// the propagation of them from a remote group of Replicas,
/// or unknown debits that its Replicas were holding
/// upon the registration of them from another
/// instance of the same Actor.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct StateSynched {
    id: PublicKey,
    balance: Token,
    debit_version: u64,
    credit_ids: HashSet<CreditId>,
}

/// Raised when the Actor has received
/// f.ex. credits that its Replicas were holding upon
/// the propagation of them from a remote group of Replicas,
/// or unknown debits that its Replicas were holding
/// upon the registration of them from another
/// instance of the same Actor.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct TransfersSynched(ActorHistory);

/// This event is raised by the Actor after having
/// successfully created a transfer cmd to send to the
/// Replicas for validation.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct TransferInitiated {
    /// The debit signed by the initiating Actor.
    pub signed_debit: SignedDebit,
    /// The credit signed by the initiating Actor.
    pub signed_credit: SignedCredit,
}

impl TransferInitiated {
    /// Get the debit id
    pub fn id(&self) -> DebitId {
        self.signed_debit.id()
    }
}

/// Raised when a Replica responds with
/// a successful validation of a transfer.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct TransferValidationReceived {
    /// The event raised by a Replica.
    validation: TransferValidated,
    /// Added when quorum of validations
    /// have been received from Replicas.
    pub proof: Option<TransferAgreementProof>,
}

/// Raised when the Actor has accumulated a
/// quorum of validations, and produced a RegisterTransfer cmd
/// for sending to Replicas.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct TransferRegistrationSent {
    transfer_proof: TransferAgreementProof,
}

#[allow(unused)]
mod test {
    use super::{
        actor::Actor, test_utils, test_utils::*, wallet, wallet_replica::WalletReplica, ActorEvent,
        Error, Result, TransferInitiated, Wallet,
    };
    use bls::{PublicKeySet, PublicKeyShare, SecretKey, SecretKeySet, SecretKeyShare};
    use crdts::{
        quickcheck::{quickcheck, TestResult},
        Dot,
    };
    use sn_data_types::{
        ActorHistory, Credit, CreditAgreementProof, CreditId, Debit, Keypair, OwnerType, PublicKey,
        ReplicaEvent, SectionElders, SignatureShare, SignedCredit, SignedDebit, SignedTransfer,
        Token, Transfer, TransferAgreementProof,
    };
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    macro_rules! hashmap {
        ($( $key: expr => $val: expr ),*) => {{
             let mut map = ::std::collections::HashMap::new();
             $( let _ = map.insert($key, $val); )*
             map
        }}
    }

    // ------------------------------------------------------------------------
    // ------------------------ Basic Transfer --------------------------------
    // ------------------------------------------------------------------------

    #[test]
    fn basic_transfer() {
        let _ = transfer_between_actors(100, 10, 3);
    }

    // #[allow(trivial_casts)]
    // #[test]
    // fn quickcheck_basic_transfer() {
    //     quickcheck(transfer_between_actors as fn(u64, u64, u8, u8, u8, u8) -> TestResult);
    // }

    #[test]
    fn synching() -> Result<()> {
        let section_count = 1;
        let replicas_per_section = 1;
        let section_configs = vec![vec![]];
        let Network {
            genesis_credit,
            mut sections,
            mut actors,
        } = setup_new_network(section_count, replicas_per_section, section_configs)?;

        let genesis_key = genesis_credit.recipient();
        let genesis_elder = &mut sections.remove(0).elders.remove(0);
        let wallet_replica = match genesis_elder.replicas.get_mut(&genesis_key) {
            Some(w) => w,
            None => panic!("Failed the test; no such wallet."),
        };
        let _ = wallet_replica
            .genesis(&genesis_credit)?
            .ok_or(Error::GenesisFailed)?;

        let event = ReplicaEvent::TransferPropagated(sn_data_types::TransferPropagated {
            credit_proof: genesis_credit.clone(),
        });
        wallet_replica.apply(event)?;

        println!("Finding genesis actor by id: {}", genesis_key);
        let mut actor_balance = None;
        for actor in actors.iter_mut() {
            println!("Actor id: {}", actor.actor.id());
            if actor.actor.id() == genesis_key {
                println!("Found actor!");
                if let Some(synched_event) = actor.actor.from_history(ActorHistory {
                    credits: vec![genesis_credit.clone()],
                    debits: vec![],
                })? {
                    actor
                        .actor
                        .apply(ActorEvent::TransfersSynched(synched_event))?;
                    actor_balance = Some(actor.actor.balance());
                }
                break;
            }
        }
        let balance = wallet_replica.balance();
        assert_eq!(genesis_credit.amount(), balance);
        assert_eq!(Some(balance), actor_balance);
        Ok(())
    }

    // ------------------------------------------------------------------------
    // ------------------------ Genesis --------------------------------
    // ------------------------------------------------------------------------

    #[test]
    fn can_start_with_genesis() -> Result<()> {
        let section_count = 1;
        let replicas_per_section = 1;
        let section_configs = vec![vec![u32::MAX as u64]];
        let Network {
            genesis_credit,
            mut sections,
            ..
        } = setup_new_network(section_count, replicas_per_section, section_configs)?;

        let genesis_key = genesis_credit.recipient();
        let genesis_elder = &mut sections.remove(0).elders.remove(0);
        let wallet_replica = match genesis_elder.replicas.get_mut(&genesis_key) {
            Some(w) => w,
            None => panic!("Failed the test; no such wallet."),
        };
        let _ = wallet_replica
            .genesis(&genesis_credit)?
            .ok_or(Error::GenesisFailed)?;

        wallet_replica.apply(ReplicaEvent::TransferPropagated(
            sn_data_types::TransferPropagated {
                credit_proof: genesis_credit.clone(),
            },
        ))?;
        let balance = wallet_replica.balance();
        assert_eq!(genesis_credit.amount(), balance);
        Ok(())
    }

    #[test]
    fn genesis_can_only_be_the_first() -> Result<()> {
        let section_count = 1;
        let replica_count = 1;
        let section_configs = vec![vec![0]];

        let Network {
            genesis_credit,
            mut sections,
            ..
        } = setup_new_network(section_count, replica_count, section_configs)?;
        let genesis_elder = &mut sections.remove(0).elders.remove(0);
        let wallet_replica = match genesis_elder.replicas.get_mut(&genesis_credit.recipient()) {
            Some(w) => w,
            None => panic!("Failed the test; no such wallet."),
        };
        let _ = wallet_replica
            .genesis(&genesis_credit)?
            .ok_or(Error::GenesisFailed)?;

        wallet_replica.apply(ReplicaEvent::TransferPropagated(
            sn_data_types::TransferPropagated {
                credit_proof: genesis_credit.clone(),
            },
        ))?;

        // try genesis again..
        let result = wallet_replica.genesis(&genesis_credit);
        match result {
            Ok(_) => panic!("Should not be able to genesis again."),
            Err(e) => assert_eq!(e, Error::InvalidOperation),
        }
        Ok(())
    }

    // ------------------------------------------------------------------------
    // ------------------------ Basic Transfer Body ---------------------------
    // ------------------------------------------------------------------------

    fn transfer_between_actors(
        sender_balance: u64,
        recipient_balance: u64,
        replica_count: u8,
    ) -> TestResult {
        match basic_transfer_between_actors(sender_balance, recipient_balance, replica_count) {
            Ok(Some(_)) => TestResult::passed(),
            Ok(None) => TestResult::discard(),
            Err(_) => TestResult::failed(),
        }
    }

    fn basic_transfer_between_actors(
        sender_balance: u64,
        recipient_balance: u64,
        replica_count: u8,
    ) -> Result<Option<()>> {
        // --- Filter ---
        if 0 == sender_balance || 2 >= replica_count {
            return Ok(None);
        }

        // --- Arrange ---
        let section_count = 2;
        let sender_index = 0;
        let recipient_index = 1;
        let recipient_final = sender_balance + recipient_balance;
        let section_configs = vec![vec![sender_balance], vec![recipient_balance]];
        let Network {
            mut actors,
            mut sections,
            ..
        } = setup_new_network(section_count, replica_count, section_configs)?;
        let mut sender_section = sections.remove(0);
        let mut recipient_section = sections.remove(0);
        let mut sender = actors.remove(0);
        let mut recipient = actors.remove(0);

        // --- Act ---
        // 1. Init transfer at Sender Actor.
        let transfer = init_transfer(&mut sender, recipient.actor.id())?;
        // 2. Validate at Sender Replicas.
        let debit_proof = validate_at_sender_replicas(transfer, &mut sender)?
            .ok_or(Error::SenderValidationFailed)?;
        // 3. Register at Sender Replicas.
        register_at_debiting_replicas(&debit_proof, &mut sender_section)?;
        // 4. Propagate to Recipient Replicas.
        let events =
            propagate_to_crediting_replicas(debit_proof.credit_proof(), &mut recipient_section);
        // 5. Synch at Recipient Actor.
        synch(&mut recipient)?;

        // --- Assert ---
        // Actor and Replicas have the correct balance.
        assert_balance(sender, Token::zero());
        assert_balance(recipient, Token::from_nano(recipient_final));
        Ok(Some(()))
    }

    fn assert_balance(actor: TestActor, amount: Token) {
        assert!(actor.actor.balance() == amount);
        actor.section.elders.iter().for_each(|elder| {
            let wallet = match elder.replicas.get(&actor.actor.id()) {
                Some(w) => w,
                None => panic!("Failed the test; no such wallet."),
            };
            assert_eq!(wallet.balance(), amount)
        });
    }

    // ------------------------------------------------------------------------
    // ------------------------ AT2 Steps -------------------------------------
    // ------------------------------------------------------------------------

    // 1. Init debit at Sender Actor.
    fn init_transfer(sender: &mut TestActor, to: PublicKey) -> Result<TransferInitiated> {
        let transfer = sender
            .actor
            .transfer(sender.actor.balance(), to, "asdf".to_string())?
            .ok_or(Error::TransferCreationFailed)?;

        sender
            .actor
            .apply(ActorEvent::TransferInitiated(transfer.clone()))?;

        Ok(transfer)
    }

    // 2. Validate debit at Sender Replicas.
    fn validate_at_sender_replicas(
        transfer: TransferInitiated,
        sender: &mut TestActor,
    ) -> Result<Option<TransferAgreementProof>> {
        for elder in &mut sender.section.elders {
            let wallet_replica = match elder.replicas.get_mut(&sender.actor.id()) {
                Some(w) => w,
                None => panic!("Failed the test; no such wallet."),
            };
            let _ = wallet_replica
                .validate(&transfer.signed_debit, &transfer.signed_credit)?
                .ok_or(Error::ValidationFailed)?;

            let signed_transfer = SignedTransfer {
                debit: transfer.signed_debit.clone(),
                credit: transfer.signed_credit.clone(),
            };
            let (replica_debit_sig, replica_credit_sig) =
                elder.signing.sign_transfer(&signed_transfer)?;
            let validation = sn_data_types::TransferValidated {
                signed_credit: signed_transfer.credit,
                signed_debit: signed_transfer.debit,
                replica_debit_sig,
                replica_credit_sig,
                replicas: sender.section.id.clone(),
            };
            // then apply to inmem state
            wallet_replica.apply(ReplicaEvent::TransferValidated(validation.clone()))?;

            let validation_received = sender
                .actor
                .receive(validation)?
                .ok_or(Error::ReceiveValidationFailed)?;
            sender.actor.apply(ActorEvent::TransferValidationReceived(
                validation_received.clone(),
            ))?;
            if let Some(proof) = validation_received.proof {
                let registered = sender
                    .actor
                    .register(proof.clone())?
                    .ok_or(Error::RegisterProofFailed)?;
                sender
                    .actor
                    .apply(ActorEvent::TransferRegistrationSent(registered))?;
                return Ok(Some(proof));
            }
        }
        Ok(None)
    }

    // 3. Register debit at Sender Replicas.
    fn register_at_debiting_replicas(
        debit_proof: &TransferAgreementProof,
        section: &mut Section,
    ) -> Result<()> {
        for elder in &mut section.elders {
            let wallet_replica = match elder.replicas.get_mut(&debit_proof.sender()) {
                Some(w) => w,
                None => panic!("Failed the test; no such wallet."),
            };
            let registered = wallet_replica
                .register(debit_proof)?
                .ok_or(Error::RegisterProofFailed)?;
            wallet_replica.apply(ReplicaEvent::TransferRegistered(registered))?;
        }
        Ok(())
    }

    // 4. Propagate credit to Recipient Replicas.
    fn propagate_to_crediting_replicas(
        credit_proof: CreditAgreementProof,
        section: &mut Section,
    ) -> Vec<ReplicaEvent> {
        section
            .elders
            .iter_mut()
            .map(|replica| {
                let wallet_replica = match replica.replicas.get_mut(&credit_proof.recipient()) {
                    Some(w) => w,
                    None => panic!("Failed the test; no such wallet."),
                };
                let _ = wallet_replica
                    .receive_propagated(&credit_proof)?
                    .ok_or(Error::ReceivePropagationFailed)?;

                let propagated = sn_data_types::TransferPropagated {
                    credit_proof: credit_proof.clone(),
                };
                // then apply to inmem state
                wallet_replica.apply(ReplicaEvent::TransferPropagated(propagated.clone()))?;
                Ok(ReplicaEvent::TransferPropagated(propagated))
            })
            .filter_map(|c: Result<ReplicaEvent>| match c {
                Ok(c) => Some(c),
                _ => None,
            })
            .collect()
    }

    // 5. Synch at Recipient Actor.
    fn synch(recipient: &mut TestActor) -> Result<()> {
        let section = &recipient.section;
        let wallet = section.elders[0]
            .replicas
            .get(&recipient.actor.id())
            .ok_or_else(|| Error::WalletNotFound(recipient.actor.id()))?;
        let snapshot = wallet.wallet().ok_or(Error::CouldNotGetWalletForReplica)?;
        let state = recipient
            .actor
            .synch(
                snapshot.balance,
                snapshot.debit_version,
                snapshot.credit_ids,
            )?
            .ok_or(Error::SyncFailed)?;
        recipient.actor.apply(ActorEvent::StateSynched(state))
    }

    // ------------------------------------------------------------------------
    // ------------------------ Setup Helpers ---------------------------------
    // ------------------------------------------------------------------------

    fn get_random_pk() -> PublicKey {
        PublicKey::from(SecretKey::random().public_key())
    }

    fn find_group(index: u8, sections: &[Section]) -> Option<Section> {
        for section in sections {
            if section.index == index {
                return Some(section.clone());
            }
        }
        None
    }

    fn setup_random_wallet(balance: u64, section: u8) -> Result<TestWallet> {
        let mut rng = rand::thread_rng();
        let keypair = Keypair::new_ed25519(&mut rng);
        let recipient = keypair.public_key();
        let owner = OwnerType::Single(recipient);
        let mut wallet = Wallet::new(owner);
        setup_wallet(balance, section, keypair, wallet)
    }

    fn setup_wallet(
        balance: u64,
        section: u8,
        keypair: Keypair,
        wallet: Wallet,
    ) -> Result<TestWallet> {
        let mut wallet = wallet;
        if balance > 0 {
            let amount = Token::from_nano(balance);
            let sender = Dot::new(get_random_pk(), 0);
            let debit = Debit { id: sender, amount };
            let credit = Credit {
                id: debit.credit_id()?,
                recipient: wallet.id().public_key(),
                amount,
                msg: "".to_string(),
            };
            let _ = wallet.apply_credit(credit)?;
        }

        Ok(TestWallet {
            wallet,
            keypair,
            section,
        })
    }

    fn setup_actor(wallet: TestWallet, sections: &[Section]) -> Result<TestActor> {
        let section = find_group(wallet.section, sections).ok_or(Error::CouldNotFindGroup)?;
        let replicas = SectionElders {
            prefix: xor_name::Prefix::default(),
            key_set: section.id.clone(),
            names: Default::default(),
        };
        let actor = Actor::from_snapshot(wallet.wallet, wallet.keypair, replicas);

        Ok(TestActor { actor, section })
    }

    // Create n replica groups, with k replicas in each
    fn setup_section_keys(group_count: u8, replica_count: u8) -> HashMap<u8, SectionKeys> {
        let mut rng = rand::thread_rng();
        let mut groups = HashMap::new();
        for i in 0..group_count {
            let threshold = std::cmp::max(1, 2 * replica_count / 3) - 1;
            let bls_secret_key = SecretKeySet::random(threshold as usize, &mut rng);
            let peers = bls_secret_key.public_keys();
            let mut shares = vec![];
            for j in 0..replica_count {
                let share = bls_secret_key.secret_key_share(j as usize);
                shares.push((share, j as usize));
            }
            let _ = groups.insert(
                i,
                SectionKeys {
                    index: i,
                    id: peers,
                    keys: shares,
                },
            );
        }
        groups
    }

    fn setup_new_network(
        section_count: u8,
        replicas_per_section: u8,
        section_configs: Vec<Vec<u64>>,
    ) -> Result<Network> {
        // setup genesis section
        let mut wallets_in_genesis_section = vec![];
        let genesis_index = 0;
        let mut section_configs = section_configs;
        let mut genesis_section_configs = section_configs.remove(genesis_index);
        for balance in genesis_section_configs {
            let wallet = setup_random_wallet(balance, genesis_index as u8)?;
            let _ = wallets_in_genesis_section.push(wallet);
        }

        let ((genesis_credit, genesis_wallet), genesis_section) =
            setup_genesis_section(replicas_per_section, wallets_in_genesis_section.clone())?;
        let _ = wallets_in_genesis_section.insert(0, genesis_wallet);

        let mut all_sections = vec![genesis_section];
        let mut other_section_wallets = vec![];
        if section_count > 1 {
            // setup rest of the sections
            for (section_index, wallet_configs) in section_configs.iter().enumerate() {
                let mut next_section_wallets = vec![];
                for balance in wallet_configs {
                    let wallet = setup_random_wallet(*balance, (section_index + 1) as u8)?;
                    let _ = next_section_wallets.push(wallet.clone());
                    let _ = other_section_wallets.push(wallet);
                }
                let section_keys = generate_section_keys(section_index as u8, replicas_per_section);
                let section =
                    setup_section(section_index as u8, section_keys, next_section_wallets);
                all_sections.push(section);
            }
        }

        let wallets = wallets_in_genesis_section
            .into_iter()
            .chain(other_section_wallets.into_iter())
            .collect();
        let actors = get_test_actors(wallets, all_sections.clone())?;

        Ok(Network {
            genesis_credit,
            sections: all_sections,
            actors,
        })
    }

    fn get_test_actors(wallets: Vec<TestWallet>, sections: Vec<Section>) -> Result<Vec<TestActor>> {
        let mut actors = vec![];
        for wallet in wallets {
            actors.push(setup_actor(wallet.clone(), &sections)?);
        }
        Ok(actors)
    }

    fn generate_section_keys(section_index: u8, replica_count: u8) -> SectionKeys {
        //
        let mut rng = rand::thread_rng();
        let threshold = std::cmp::max(1, 2 * replica_count / 3) - 1;
        let bls_secret_key = SecretKeySet::random(threshold as usize, &mut rng);
        let peers = bls_secret_key.public_keys();
        let mut shares = vec![];
        for j in 0..replica_count {
            let share = bls_secret_key.secret_key_share(j as usize);
            shares.push((share, j as usize));
        }
        SectionKeys {
            index: section_index,
            id: peers,
            keys: shares,
        }
    }

    fn setup_section(
        section_index: u8,
        section_keys: SectionKeys,
        wallets: Vec<TestWallet>,
    ) -> Section {
        /// ----
        let mut elders = vec![];
        let peer_replicas = section_keys.id.clone();
        for (secret_key, key_index) in &section_keys.keys {
            let wallets = wallets.clone();
            let mut wallet_replicas = hashmap![];
            for wallet in wallets.into_iter() {
                let wallet_id = wallet.wallet.id();
                let pending_proposals = Default::default();
                let wallet_replica = WalletReplica::from_snapshot(
                    wallet_id.clone(),
                    secret_key.public_key_share(),
                    *key_index,
                    peer_replicas.clone(),
                    wallet.wallet.clone(),
                    pending_proposals,
                    None,
                );
                let _ = wallet_replicas.insert(wallet_id.public_key(), wallet_replica);
            }
            elders.push(Elder {
                id: secret_key.public_key_share(),
                replicas: wallet_replicas,
                signing: ReplicaSigning::new(secret_key.clone(), *key_index, peer_replicas.clone()),
            });
        }
        Section {
            index: section_index,
            id: peer_replicas,
            elders,
        }
    }

    fn setup_genesis_section(
        replica_count: u8,
        wallets: Vec<TestWallet>,
    ) -> Result<((CreditAgreementProof, TestWallet), Section)> {
        // setup genesis wallet
        let threshold = (replica_count - 1) as usize;
        let balance = u32::MAX as u64 * 1_000_000_000;
        let mut rng = rand::thread_rng();
        let bls_secret_key = SecretKeySet::random(threshold, &mut rng);
        let peer_replicas = bls_secret_key.public_keys();
        let id = PublicKey::Bls(peer_replicas.public_key());
        let keypair = sn_data_types::Keypair::new_bls_share(
            0,
            bls_secret_key.secret_key_share(0),
            peer_replicas.clone(),
        );
        let owner = OwnerType::Multi(peer_replicas.clone());
        let empty_genesis_wallet = setup_wallet(0, 0, keypair, Wallet::new(owner))?;
        let genesis_credit = get_multi_genesis(balance, id, bls_secret_key.clone())?;

        let mut wallets = wallets;
        wallets.insert(0, empty_genesis_wallet.clone());

        // setup genesis replicas
        let mut elders = vec![];
        for key_index in 0..replica_count as usize {
            // copy over all wallets to the replicas
            for wallet in &wallets {
                let secret_key = bls_secret_key.secret_key_share(key_index);
                let peer_replicas = bls_secret_key.public_keys();
                let mut wallet_replicas = hashmap![];
                let wallet_id = wallet.wallet.id();
                let pending_proposals = Default::default();
                let wallet_replica = WalletReplica::from_snapshot(
                    wallet_id.clone(),
                    secret_key.public_key_share(),
                    key_index,
                    peer_replicas.clone(),
                    wallet.wallet.clone(),
                    pending_proposals,
                    None,
                );
                let _ = wallet_replicas.insert(wallet_id.public_key(), wallet_replica);
                elders.push(Elder {
                    id: secret_key.public_key_share(),
                    replicas: wallet_replicas,
                    signing: ReplicaSigning::new(
                        secret_key.clone(),
                        key_index,
                        peer_replicas.clone(),
                    ),
                });
            }
        }
        Ok((
            (genesis_credit, empty_genesis_wallet),
            Section {
                index: 0,
                id: peer_replicas,
                elders,
            },
        ))
    }

    // ------------------------------------------------------------------------
    // ------------------------ Structs ---------------------------------------
    // ------------------------------------------------------------------------
}
