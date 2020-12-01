use super::store::TransferStore;
use crate::{utils::Init, Error, Outcome, ReplicaInfo, Result, TernaryResult};
use bls::PublicKeySet;
use futures::lock::Mutex;
use sn_data_types::{
    CreditAgreementProof, Error as NdError, Money, PublicKey, ReplicaEvent, SignedTransfer,
    TransferAgreementProof, TransferPropagated, TransferRegistered, TransferValidated,
};
use sn_transfers::{get_genesis, WalletReplica};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

type WalletLocks = HashMap<PublicKey, Arc<Mutex<TransferStore>>>;

#[derive(Clone)]
pub struct Replicas {
    root_dir: PathBuf,
    info: ReplicaInfo,
    locks: WalletLocks,
}

impl Replicas {
    pub(crate) fn new(root_dir: PathBuf, info: ReplicaInfo) -> Result<Self> {
        Ok(Self {
            root_dir,
            info,
            locks: Default::default(),
        })
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Queries ----------------------------------
    /// -----------------------------------------------------------------

    /// History of events
    pub async fn history(&self, id: PublicKey) -> Outcome<Vec<ReplicaEvent>> {
        let store = match TransferStore::new(id.into(), &self.root_dir, Init::Load) {
            Ok(store) => store,
            Err(_e) => TransferStore::new(id.into(), &self.root_dir, Init::New)?,
        };
        Outcome::oki(store.get_all())
    }

    ///
    pub async fn balance(&self, id: PublicKey) -> Outcome<Money> {
        let store = match TransferStore::new(id.into(), &self.root_dir, Init::Load) {
            Ok(store) => store,
            Err(_e) => TransferStore::new(id.into(), &self.root_dir, Init::New)?,
        };
        let wallet = self.load_wallet(&store, id).await?;
        Outcome::oki(wallet.balance())
    }

    /// Get the replica's PK set
    pub fn replicas_pk_set(&self) -> PublicKeySet {
        self.info.peer_replicas.clone()
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Cmds -------------------------------------
    /// -----------------------------------------------------------------

    /// This is the one and only infusion of money to the system. Ever.
    /// It is carried out by the first node in the network.
    async fn genesis<F: FnOnce() -> Result<PublicKey, NdError>>(
        &self,
        credit_proof: &CreditAgreementProof,
        past_key: F,
    ) -> Outcome<()> {
        let id = credit_proof.recipient();
        // Acquire lock of the wallet.
        let key_lock = self.load_key_lock(id).await?;
        let mut store = key_lock.lock().await;

        // Access to the specific wallet is now serialised!
        let wallet = self.load_wallet(&store, id).await?;
        if wallet.genesis(credit_proof, past_key).is_ok() {
            // sign + update state
            if let Some(crediting_replica_sig) = self
                .info
                .signing
                .lock()
                .await
                .sign_credit_proof(credit_proof)?
            {
                // Q: are we locked on `info.signing` here? (we don't want to be)
                store.try_insert(ReplicaEvent::TransferPropagated(TransferPropagated {
                    credit_proof: credit_proof.clone(),
                    crediting_replica_sig,
                    crediting_replica_keys: PublicKey::Bls(self.info.peer_replicas.public_key()),
                }))?;
                return Ok(None);
            }
            return Ok(None);
        }
        Err(Error::InvalidMessage)
    }

    pub async fn initiate(&self, events: &[ReplicaEvent]) -> Outcome<()> {
        use ReplicaEvent::*;
        if events.is_empty() {
            //info!("Events are empty. Initiating Genesis replica.");
            // This means we are the first node in the network.
            let balance = u32::MAX as u64 * 1_000_000_000;
            let credit_proof = get_genesis(
                balance,
                PublicKey::Bls(self.info.peer_replicas.public_key()),
            )?;
            let genesis_source = Ok(PublicKey::Bls(credit_proof.replica_keys().public_key()));
            return self.genesis(&credit_proof, || genesis_source).await;
        }
        for e in events {
            let id = match e {
                TransferValidated(e) => e.sender(),
                TransferRegistered(e) => e.sender(),
                TransferPropagated(e) => e.recipient(),
                KnownGroupAdded(_) => unimplemented!(),
            };

            // Acquire lock of the wallet.
            let key_lock = self.load_key_lock(id).await?;
            let mut store = key_lock.lock().await;

            // Access to the specific wallet is now serialised!
            store.try_insert(e.to_owned())?;
        }
        Outcome::oki_no_value()
    }

    ///
    pub fn update_replica_keys(&mut self, info: ReplicaInfo) {
        self.info = info;
    }

    /// For now, with test money there is no from wallet.., money is created from thin air.
    pub async fn test_validate_transfer(&self, signed_transfer: SignedTransfer) -> Outcome<()> {
        let id = signed_transfer.sender();
        // Acquire lock of the wallet.
        let key_lock = self.load_key_lock(id).await?;
        let mut store = key_lock.lock().await;

        // Access to the specific wallet is now serialised!
        let wallet = self.load_wallet(&store, id).await?;
        match wallet.test_validate_transfer(&signed_transfer.debit, &signed_transfer.credit) {
            Ok(None) => (),
            Err(e) => return Err(Error::NetworkData(e)),
            Ok(Some(())) => {
                // sign + update state
                if let Some((replica_debit_sig, replica_credit_sig)) = self
                    .info
                    .signing
                    .lock()
                    .await
                    .sign_transfer(&signed_transfer)?
                {
                    store.try_insert(ReplicaEvent::TransferValidated(TransferValidated {
                        signed_credit: signed_transfer.credit,
                        signed_debit: signed_transfer.debit,
                        replica_debit_sig,
                        replica_credit_sig,
                        replicas: self.info.peer_replicas.clone(),
                    }))?;
                    return Ok(None);
                }
            }
        };
        Ok(None)
    }

    /// Step 1. Main business logic validation of a debit.
    pub async fn validate(&self, signed_transfer: SignedTransfer) -> Outcome<TransferValidated> {
        let id = signed_transfer.sender();
        // Acquire lock of the wallet.
        let key_lock = self.load_key_lock(id).await?;
        let mut store = key_lock.lock().await;

        // Access to the specific wallet is now serialised!
        let wallet = self.load_wallet(&store, id).await?;
        match wallet.validate(&signed_transfer.debit, &signed_transfer.credit) {
            Ok(None) => (),
            Err(e) => return Err(Error::NetworkData(e)),
            Ok(Some(())) => {
                // signing will be serialised
                if let Some((replica_debit_sig, replica_credit_sig)) = self
                    .info
                    .signing
                    .lock()
                    .await
                    .sign_transfer(&signed_transfer)?
                {
                    // release lock and update state
                    let event = TransferValidated {
                        signed_credit: signed_transfer.credit,
                        signed_debit: signed_transfer.debit,
                        replica_debit_sig,
                        replica_credit_sig,
                        replicas: self.info.peer_replicas.clone(),
                    };
                    store.try_insert(ReplicaEvent::TransferValidated(event.clone()))?;
                    return Outcome::oki(event);
                }
            }
        };
        Ok(None)
    }

    /// Step 2. Validation of agreement, and order at debit source.
    pub async fn register(
        &self,
        transfer_proof: &TransferAgreementProof,
    ) -> Outcome<TransferRegistered> {
        let id = transfer_proof.sender();
        // Acquire lock of the wallet.
        let key_lock = self.load_key_lock(id).await?;
        let mut store = key_lock.lock().await;

        // Access to the specific wallet is now serialised!
        let wallet = self.load_wallet(&store, id).await?;
        match wallet.register(transfer_proof, || {
            self.find_past_key(&transfer_proof.replica_keys())
        }) {
            Ok(None) => (),
            Err(e) => return Err(Error::NetworkData(e)),
            Ok(Some(event)) => {
                store.try_insert(ReplicaEvent::TransferRegistered(event.clone()))?;
                return Outcome::oki(event);
            }
        };
        Ok(None)
    }

    /// Step 3. Validation of DebitAgreementProof, and credit idempotency at credit destination.
    /// (Since this leads to a credit, there is no requirement on order.)
    pub async fn receive_propagated(
        &self,
        credit_proof: &CreditAgreementProof,
    ) -> Outcome<TransferPropagated> {
        // Acquire lock of the wallet.
        let id = credit_proof.recipient();
        let key_lock = self.load_key_lock(id).await?;
        let mut store = key_lock.lock().await;

        // Access to the specific wallet is now serialised!
        let wallet = self.load_wallet(&store, id).await?;
        if wallet
            .receive_propagated(credit_proof, || {
                self.find_past_key(&credit_proof.replica_keys())
            })
            .is_ok()
        {
            // sign + update state
            if let Some(crediting_replica_sig) = self
                .info
                .signing
                .lock()
                .await
                .sign_credit_proof(credit_proof)?
            {
                let event = TransferPropagated {
                    credit_proof: credit_proof.clone(),
                    crediting_replica_keys: PublicKey::Bls(self.info.peer_replicas.public_key()),
                    crediting_replica_sig,
                };
                store.try_insert(ReplicaEvent::TransferPropagated(event.clone()))?;
                return Outcome::oki(event);
            }
        }
        Err(Error::InvalidMessage)
    }

    async fn load_key_lock(&self, id: PublicKey) -> Result<Arc<Mutex<TransferStore>>> {
        match self.locks.get(&id) {
            Some(val) => Ok(val.clone()),
            None => Err(Error::Logic),
        }
    }

    async fn load_wallet(&self, store: &TransferStore, id: PublicKey) -> Result<WalletReplica> {
        // id: PublicKey
        // let store = match TransferStore::new(id.into(), &self.root_dir, Init::Load) {
        //     Ok(store) => store,
        //     Err(_e) => TransferStore::new(id.into(), &self.root_dir, Init::New)?,
        // };
        let events = store.get_all();
        let wallet = WalletReplica::from_history(
            id,
            self.info.id,
            self.info.key_index,
            self.info.peer_replicas.clone(),
            events,
        )
        .map_err(|e| Error::NetworkData(e))?;
        Ok(wallet)
    }

    fn find_past_key(&self, keyset: &PublicKeySet) -> Result<PublicKey, NdError> {
        let section_keys = self.info.section_proof_chain.clone();
        let key = section_keys
            .keys()
            .find(|&key_in_chain| key_in_chain == &keyset.public_key());
        if let Some(key_in_chain) = key {
            Ok(PublicKey::Bls(*key_in_chain))
        } else {
            Err(NdError::NetworkOther("PublicKey provided by the transfer was never a part of the Section retrospectively".to_string()))
        }
    }
}
