// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use std::sync::Arc;

use crate::{
    routing::{
        dkg::SectionDkgOutcome,
        error::{Error, Result},
    },
    types::CFSet,
};
use dashmap::DashMap;

/// All the key material needed to sign or combine signature for our section key.
#[derive(custom_debug::Debug)]
pub(crate) struct SectionKeyShare<S: Signer> {
    /// Public key set to verify threshold signatures and combine shares.
    pub(crate) public_key_set: bls::PublicKeySet,
    /// Index of the owner of this key share within the set of all section elders.
    pub(crate) index: usize,
    /// Signing func, as to never pass around the secret key share.
    pub(crate) signer: S,
}

pub trait Signer {
    fn sign<M: AsRef<[u8]>>(self, msg: M) -> bls::SignatureShare;
}

/// Struct that holds the current section keys and helps with new key generation.
#[derive(Debug)]
pub(crate) struct SectionKeysProvider {
    /// A cache for current and previous section BLS keys.
    cache: MiniKeyCache,
    /// The new keys to use when section update completes.
    // TODO: evict outdated keys.
    // TODO: alternatively, store the pending keys in DkgVoter instead. That way the outdated ones
    //       would get dropped when the DKG session itself gets dropped which we already have
    //       implemented.
    pending: DashMap<bls::PublicKey, SectionDkgOutcome>,
}

impl SectionKeysProvider {
    pub(crate) fn new(cache_size: u8, current: Option<SectionDkgOutcome>) -> Self {
        let provider = Self {
            pending: DashMap::new(),
            cache: MiniKeyCache::with_capacity(cache_size as usize),
        };
        if let Some(share) = current {
            let public_key = share.public_key();
            provider.insert_dkg_outcome(share);
            provider.finalise_dkg(&public_key);
        }
        provider
    }

    pub(crate) fn sign_with(
        &self,
        data: &[u8],
        public_key: &bls::PublicKey,
    ) -> Result<(usize, bls::SignatureShare)> {
        self.cache.sign_with(data, public_key)
    }

    pub(crate) fn key_share(&self) -> Result<SectionKeyShare<impl Signer>> {
        self.cache.get_most_recent()
    }

    pub(crate) fn has_key_share(&self) -> bool {
        self.cache.has_key_share()
    }

    pub(crate) fn insert_dkg_outcome(&self, share: SectionDkgOutcome) {
        let public_key = share.public_key();
        let _ = self.pending.insert(public_key, share);
    }

    pub(crate) fn finalise_dkg(&self, public_key: &bls::PublicKey) {
        if let Some((_, share)) = self.pending.remove(public_key) {
            if let Some(evicted) = self.cache.add(share) {
                trace!("evicted old key from cache: {:?}", evicted);
            }
            trace!("finalised DKG: {:?}", public_key);
        }
    }
}

#[derive(custom_debug::Debug)]
struct KeyHolder {
    #[debug(skip)]
    secret_key_share: Arc<bls::SecretKeyShare>,
}

impl Signer for KeyHolder {
    /// One-off sign, then dropping self.
    fn sign<M: AsRef<[u8]>>(self, msg: M) -> bls::SignatureShare {
        let sig = self.secret_key_share.sign(msg);
        sig
    }
}

#[derive(custom_debug::Debug)]
struct SecretKeyShare {
    /// Secret Key share.
    #[debug(skip)]
    secret_key_share: Arc<bls::SecretKeyShare>,
    /// Index of the owner of this key share within the set of all section elders.
    index: usize,
    /// Public key set to verify threshold signatures and combine shares.
    public_key_set: bls::PublicKeySet,
}

/// Implementation of super simple cache, for no more than a handfull of items.
#[derive(Debug)]
struct MiniKeyCache {
    capacity: usize,
    list: CFSet<(bls::PublicKey, SecretKeyShare)>,
}

impl MiniKeyCache {
    /// Constructor for capacity based `KeyCache`.
    fn with_capacity(capacity: usize) -> MiniKeyCache {
        MiniKeyCache {
            capacity,
            list: CFSet::new(),
        }
    }

    /// Returns true if a key share exists.
    fn has_key_share(&self) -> bool {
        self.list.last().is_some()
    }

    /// Returns the most recently added key.
    fn get_most_recent(&self) -> Result<SectionKeyShare<impl Signer>> {
        if let Some(item) = self.list.last() {
            let (_, share) = item.as_ref();
            return Ok(SectionKeyShare {
                public_key_set: share.public_key_set.clone(),
                index: share.index,
                signer: KeyHolder {
                    secret_key_share: share.secret_key_share.clone(),
                },
            });
        }
        Err(Error::MissingSectionKeyShare)
    }

    /// Uses the secret key from cache, corresponding to
    /// the provided public key.
    fn sign_with(
        &self,
        data: &[u8],
        public_key: &bls::PublicKey,
    ) -> Result<(usize, bls::SignatureShare)> {
        for item in self.list.values() {
            let (cached_public, section_key_share) = item.as_ref();
            if public_key == cached_public {
                let sig = section_key_share.secret_key_share.sign(data);
                return Ok((section_key_share.index, sig));
            }
        }
        Err(Error::MissingSectionKeyShare)
    }

    /// Adds a new key to the cache, and removes + returns the oldest
    /// key if cache size is exceeded.
    fn add(&self, key_share: SectionDkgOutcome) -> Option<bls::PublicKey> {
        let public_key = &key_share.public_key();
        for item in self.list.values() {
            let (cached_public, _) = item.as_ref();
            if public_key == cached_public {
                return None;
            }
        }

        let mut evicted = None;
        if self.capacity == self.list.len() {
            if let Some(item) = self.list.pop_front() {
                let (cached_public, _) = item.as_ref();
                evicted = Some(*cached_public);
            }
        }

        let (index, public_key_set, secret_key_share) = key_share.consume();

        let key_share = SecretKeyShare {
            index,
            public_key_set,
            secret_key_share: Arc::new(secret_key_share),
        };

        self.list.push((*public_key, key_share));

        evicted
    }
}
