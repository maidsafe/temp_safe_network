use super::store::TransferStore;
use crate::{utils::Init, Error, Outcome, ReplicaInfo, Result, TernaryResult};
use bls::PublicKeySet;
use futures::lock::Mutex;
use sn_data_types::{
    DebitAgreementProof, Error as NdError, Money, PublicKey, ReplicaEvent, SignedTransfer,
    Transfer, TransferPropagated, TransferRegistered, TransferValidated,
};
use sn_transfers::WalletReplica;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

type WalletLocks = HashMap<PublicKey, Arc<Mutex<PublicKey>>>;

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

    /// Query for new credits since specified index.
    /// NB: This is not guaranteed to give you all unknown to you,
    /// since there is no absolute order on the credits!
    /// Includes the credit at specified index (which may,
    /// or may not, be the same as the one that the Actor has at the same index).
    pub async fn credits_since(&self, id: PublicKey, index: usize) -> Outcome<Vec<Transfer>> {
        let (wallet, _) = self.load_wallet(id).await?;
        Outcome::oki(wallet.credits_since(index))
    }

    /// Query for new debits transfers since specified index.
    /// Includes the debit at specified index.
    pub async fn debits_since(&self, id: PublicKey, index: usize) -> Outcome<Vec<Transfer>> {
        let (wallet, _) = self.load_wallet(id).await?;
        Outcome::oki(wallet.debits_since(index))
    }

    ///
    pub async fn balance(&self, id: PublicKey) -> Outcome<Money> {
        let (wallet, _) = self.load_wallet(id).await?;
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
    pub async fn genesis<F: FnOnce() -> Result<bool, NdError>>(
        &mut self,
        debit_proof: &DebitAgreementProof,
        past_key: F,
    ) -> Outcome<()> {
        let id = debit_proof.from();
        // Acquire lock of the wallet.
        let key_lock = self.load_key_lock(id).await?;
        let lock = key_lock.lock().await;
        // Access to the specific wallet is now serialised!
        let (wallet, mut store) = self.load_wallet(id).await?;

        if wallet.genesis(debit_proof, past_key).is_ok() {
            // sign + update state
            if let Some(crediting_replica_sig) =
                self.info.signing.lock().await.sign_proof(debit_proof)?
            {
                store.try_append(ReplicaEvent::TransferPropagated(TransferPropagated {
                    debit_proof: debit_proof.clone(),
                    debiting_replicas: PublicKey::Bls(self.info.peer_replicas.public_key()),
                    crediting_replica_sig,
                }));
                return Ok(None);
            }
            return Ok(None);
        }
        Err(Error::InvalidMessage)
    }

    pub async fn initiate(&mut self, events: &[ReplicaEvent]) -> Outcome<()> {
        use ReplicaEvent::*;
        for e in events {
            let id = match e {
                TransferValidated(e) => e.from(),
                TransferRegistered(e) => e.from(),
                TransferPropagated(e) => e.to(),
            };

            // Acquire lock of the wallet.
            let key_lock = self.load_key_lock(id).await?;
            let lock = key_lock.lock().await;

            // Access to the specific wallet is now serialised!
            let (_, mut store) = self.load_wallet(id).await?;
            store.try_append(e.to_owned());
        }
        Outcome::oki_no_value()
    }

    pub fn update_replica_keys(&mut self, info: ReplicaInfo) {
        self.info = info;
    }

    /// For now, with test money there is no from wallet.., money is created from thin air.
    pub async fn test_validate_transfer(&mut self, signed_transfer: SignedTransfer) -> Outcome<()> {
        let id = signed_transfer.from();
        // Acquire lock of the wallet.
        let key_lock = self.load_key_lock(id).await?;
        let lock = key_lock.lock().await;
        // Access to the specific wallet is now serialised!
        let (wallet, mut store) = self.load_wallet(id).await?;

        match wallet.test_validate_transfer(&signed_transfer) {
            Ok(None) => (),
            Err(e) => return Err(Error::NetworkData(e)),
            Ok(Some(event)) => {
                // sign + update state
                if let Some(replica_signature) = self
                    .info
                    .signing
                    .lock()
                    .await
                    .sign_validated_transfer(&signed_transfer)?
                {
                    store.try_append(ReplicaEvent::TransferValidated(TransferValidated {
                        signed_transfer,
                        replica_signature,
                        replicas: self.info.peer_replicas.clone(),
                    }));
                    return Ok(None);
                }
            }
        };
        Ok(None)
    }

    /// Step 1. Main business logic validation of a debit.
    pub async fn validate(
        &mut self,
        signed_transfer: SignedTransfer,
    ) -> Outcome<TransferValidated> {
        let id = signed_transfer.from();
        // Acquire lock of the wallet.
        let key_lock = self.load_key_lock(id).await?;
        let lock = key_lock.lock().await;
        // Access to the specific wallet is now serialised!
        let (wallet, mut store) = self.load_wallet(id).await?;

        match wallet.validate(&signed_transfer) {
            Ok(None) => (),
            Err(e) => return Err(Error::NetworkData(e)),
            Ok(Some(event)) => {
                // signing will be serialised
                if let Some(replica_signature) = self
                    .info
                    .signing
                    .lock()
                    .await
                    .sign_validated_transfer(&signed_transfer)?
                {
                    // release lock and update state
                    let event = TransferValidated {
                        signed_transfer,
                        replica_signature,
                        replicas: self.info.peer_replicas.clone(),
                    };
                    store.try_append(ReplicaEvent::TransferValidated(event.clone()));
                    return Outcome::oki(event);
                }
            }
        };
        Ok(None)
    }

    /// Step 2. Validation of agreement, and order at debit source.
    pub async fn register<F: FnOnce() -> bool>(
        &mut self,
        debit_proof: &DebitAgreementProof,
    ) -> Outcome<TransferRegistered> {
        let id = debit_proof.from();
        // Acquire lock of the wallet.
        let key_lock = self.load_key_lock(id).await?;
        let lock = key_lock.lock().await;

        // Access to the specific wallet is now serialised!
        let (wallet, mut store) = self.load_wallet(id).await?;
        match wallet.register(debit_proof, || self.find_past_key(&debit_proof)) {
            Ok(None) => (),
            Err(e) => return Err(Error::NetworkData(e)),
            Ok(Some(event)) => {
                store.try_append(ReplicaEvent::TransferRegistered(event.clone()));
                return Outcome::oki(event);
            }
        };
        Ok(None)
    }

    /// Step 3. Validation of DebitAgreementProof, and credit idempotency at credit destination.
    /// (Since this leads to a credit, there is no requirement on order.)
    pub async fn receive_propagated(
        &mut self,
        debit_proof: &DebitAgreementProof,
    ) -> Outcome<TransferPropagated> {
        // Acquire lock of the wallet.
        let id = debit_proof.to();
        let key_lock = self.load_key_lock(id).await?;
        let lock = key_lock.lock().await;

        // Access to the specific wallet is now serialised!
        let (wallet, mut store) = self.load_wallet(id).await?;
        if wallet
            .receive_propagated(debit_proof, || self.find_past_key(&debit_proof))
            .is_ok()
        {
            // sign + update state
            if let Some(crediting_replica_sig) =
                self.info.signing.lock().await.sign_proof(debit_proof)?
            {
                let event = TransferPropagated {
                    debit_proof: debit_proof.clone(),
                    debiting_replicas: PublicKey::Bls(self.info.peer_replicas.public_key()),
                    crediting_replica_sig,
                };
                store.try_append(ReplicaEvent::TransferPropagated(event.clone()));
                return Outcome::oki(event);
            }
        }
        Err(Error::InvalidMessage)
    }

    async fn load_key_lock(&mut self, id: PublicKey) -> Result<Arc<Mutex<PublicKey>>> {
        match self.locks.get(&id) {
            Some(val) => Ok(val.clone()),
            None => {
                self.locks.insert(id, Arc::new(Mutex::new(id)));
                match self.locks.get(&id) {
                    Some(val) => Ok(val.clone()),
                    None => Err(Error::Logic),
                }
            }
        }
    }

    async fn load_wallet(&self, id: PublicKey) -> Result<(WalletReplica, TransferStore)> {
        let store = match TransferStore::new(id.into(), &self.root_dir, Init::Load) {
            Ok(store) => store,
            Err(e) => TransferStore::new(id.into(), &self.root_dir, Init::New)?,
        };
        let events = store.try_load()?;
        let wallet = WalletReplica::from_history(
            id,
            self.info.id,
            self.info.key_index,
            self.info.peer_replicas.clone(),
            events,
        )
        .map_err(|e| Error::NetworkData(e))?;
        Ok((wallet, store))
    }

    fn find_past_key(&self, proof: &DebitAgreementProof) -> Result<bool, NdError> {
        let section_keys = self.info.section_proof_chain.clone();
        let serialized = bincode::serialize(&proof.signed_transfer)
            .map_err(|e| NdError::NetworkOther(e.to_string()))?;
        let sig = proof
            .debiting_replicas_sig
            .clone()
            .into_bls()
            .ok_or_else(|| {
                NdError::NetworkOther("Error retrieving threshold::Signature from DAP ".to_string())
            })?;
        let key = section_keys
            .keys()
            .find(|&key_in_chain| key_in_chain == &proof.replica_key.public_key());
        if let Some(key_in_chain) = key {
            Ok(key_in_chain.verify(&sig, serialized))
        } else {
            Err(NdError::NetworkOther("PublicKey provided by the transfer was never a part of the Section retrospectively".to_string()))
        }
    }
}
