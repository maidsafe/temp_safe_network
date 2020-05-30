// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::{BlsKeyPair, Error, Result, Safe};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

// Instantiate a Safe instance
pub async fn new_safe_instance() -> Result<Safe> {
    let mut safe = Safe::default();
    safe.connect("", Some("fake-credentials")).await?;
    Ok(safe)
}

// Create a random NRS name
pub fn random_nrs_name() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(15).collect()
}

// Try to unwrap an Option<BlsKeyPair> and throw error if it's None
pub fn unwrap_key_pair(kp: Option<BlsKeyPair>) -> Result<BlsKeyPair> {
    let key_pair = kp.ok_or_else(|| {
        Error::Unexpected("Unexpectedly there is no BlsKeyPair to unwrap".to_string())
    })?;
    Ok(key_pair)
}
