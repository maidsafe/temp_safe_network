// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Utilities

use crate::node::{Error, Result};
use bytes::Bytes;
use rand::{distributions::Standard, CryptoRng, Rng};
use serde::{de::DeserializeOwned, Serialize};

/// Easily create a `BTreeSet`.
#[macro_export]
macro_rules! btree_set {
    ($($item:expr),*) => {{
        let mut _set = ::std::collections::BTreeSet::new();
        $(
            let _ = _set.insert($item);
        )*
        _set
    }};

    ($($item:expr),*,) => {
        btree_set![$($item),*]
    };
}

/// Easily create a `BTreeMap` with the key => value syntax.
#[macro_export]
macro_rules! btree_map {
    () => ({
        ::std::collections::BTreeMap::new()
    });

    ($($key:expr => $value:expr),*) => {{
        let mut _map = ::std::collections::BTreeMap::new();
        $(
            let _ = _map.insert($key, $value);
        )*
        _map
    }};

    ($($key:expr => $value:expr),*,) => {
        btree_map![$($key => $value),*]
    };
}

#[allow(dead_code)]
pub(crate) fn random_vec<R: CryptoRng + Rng>(rng: &mut R, size: usize) -> Vec<u8> {
    rng.sample_iter(&Standard).take(size).collect()
}

pub(crate) fn serialise<T: Serialize>(data: &T) -> Result<Bytes> {
    let serialised_data = bincode::serialize(data).map_err(Error::Bincode)?;
    Ok(Bytes::copy_from_slice(serialised_data.as_slice()))
}

#[allow(unused)]
pub(crate) fn deserialise<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    bincode::deserialize(bytes).map_err(Error::Bincode)
}

/// Command that the user can send to a running node to control its execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Shutdown the node
    Shutdown,
}
