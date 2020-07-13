// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{node::Init, Result};
use log::{error, trace};
use pickledb::{PickleDb, PickleDbDumpPolicy};
use rand::{distributions::Standard, CryptoRng, Rng};
use routing::{Node as Routing, SrcLocation};
use safe_nd::{
    BlsKeypairShare, ClientPublicId, Keypair, PublicId, PublicKey, Signature, SignatureShare,
    XorName,
};
use serde::Serialize;
use std::{
    cell::{Cell, Ref, RefCell},
    fs,
    path::Path,
    rc::Rc,
};
use threshold_crypto::{self, serde_impl::SerdeSecret};
use unwrap::unwrap;

pub(crate) fn new_db<D: AsRef<Path>, N: AsRef<Path>>(
    db_dir: D,
    db_name: N,
    init_mode: Init,
) -> Result<PickleDb> {
    let db_path = db_dir.as_ref().join(db_name);
    if init_mode == Init::New {
        trace!("Creating database at {}", db_path.display());
        fs::create_dir_all(db_dir)?;
        let mut db = PickleDb::new_bin(db_path, PickleDbDumpPolicy::AutoDump);
        // Write then delete a value to ensure DB file is actually written to disk.
        db.set("", &"")?;
        let _ = db.rem("")?;
        return Ok(db);
    }
    trace!("Loading database at {}", db_path.display());
    let result = PickleDb::load_bin(db_path.clone(), PickleDbDumpPolicy::AutoDump);
    if let Err(ref error) = &result {
        error!("Failed to load {}: {}", db_path.display(), error);
    }
    Ok(result?)
}

pub(crate) fn random_vec<R: CryptoRng + Rng>(rng: &mut R, size: usize) -> Vec<u8> {
    rng.sample_iter(&Standard).take(size).collect()
}

pub(crate) fn serialise<T: Serialize>(data: &T) -> Vec<u8> {
    unwrap!(bincode::serialize(data))
}

/// Returns the client's public ID, the owner's public ID, or None depending on whether `public_id`
/// represents a Client, App or Node respectively.
pub(crate) fn owner(public_id: &PublicId) -> Option<&ClientPublicId> {
    match public_id {
        PublicId::Node(_) => None,
        PublicId::Client(pub_id) => Some(pub_id),
        PublicId::App(pub_id) => Some(pub_id.owner()),
    }
}

/// Returns the client's ID if `public_id` represents a Client, or None if it represents an App or
/// Node.
pub(crate) fn client(public_id: &PublicId) -> Option<&ClientPublicId> {
    match public_id {
        PublicId::Node(_) | PublicId::App(_) => None,
        PublicId::Client(pub_id) => Some(pub_id),
    }
}

/// Returns the client's or app's public key if `public_id` represents a Client or App respectively,
/// or None if it represents a Node.
pub(crate) fn own_key(public_id: &PublicId) -> Option<&PublicKey> {
    match public_id {
        PublicId::Node(_) => None,
        PublicId::Client(ref client) => Some(client.public_key()),
        PublicId::App(ref app) => Some(app.public_key()),
    }
}

pub(crate) fn get_source_name(src: SrcLocation) -> XorName {
    if let SrcLocation::Node(xorname) = src {
        XorName(xorname.0)
    } else {
        XorName::default()
    }
}

pub(crate) fn bls_sign(routing: Ref<Routing>, data: &[u8]) -> Option<Signature> {
    let signature = routing
        .secret_key_share()
        .map_or(None, |key| Some(key.sign(data)));
    signature.map(|sig| {
        Signature::BlsShare(SignatureShare {
            index: routing.our_index().unwrap_or(0),
            share: sig,
        })
    })
}

pub(crate) fn key_pair(routing: Rc<RefCell<Routing>>) -> Result<Keypair> {
    let node = routing.borrow();
    let index = node.our_index()?;
    let bls_secret_key = node.secret_key_share()?;
    let secret = SerdeSecret(bls_secret_key.clone());
    let public = bls_secret_key.public_key_share();
    Ok(Keypair::BlsShare(BlsKeypairShare {
        index,
        secret,
        public,
    }))
}
