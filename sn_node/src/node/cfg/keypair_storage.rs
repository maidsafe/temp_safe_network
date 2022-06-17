// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::node::{Error, Result};
use ed25519_dalek::{Keypair, PublicKey, KEYPAIR_LENGTH};
use hex::{decode, encode};
use std::path::Path;
use tokio::fs;

// Filename for storing the node's reward (Ed25519 hex-encoded) public key
const REWARD_PUBLIC_KEY_FILENAME: &str = "reward_public_key";
// Filename for storing the node's reward (Ed25519 hex-encoded) secret key
const REWARD_SECRET_KEY_FILENAME: &str = "reward_secret_key";

const NETWORK_KEYPAIR_FILENAME: &str = "network_keypair";

/// Writes the network keypair to disk.
pub(crate) async fn store_network_keypair(
    root_dir: &Path,
    keypair_as_bytes: [u8; KEYPAIR_LENGTH],
) -> Result<()> {
    let keypair_path = root_dir.join(NETWORK_KEYPAIR_FILENAME);
    fs::write(keypair_path, encode(keypair_as_bytes)).await?;

    Ok(())
}

/// Returns Some(KeyPair) or None if file doesn't exist.
#[allow(dead_code)]
pub(crate) async fn get_network_keypair(root_dir: &Path) -> Result<Option<Keypair>> {
    let path = root_dir.join(NETWORK_KEYPAIR_FILENAME);
    if !path.is_file() {
        return Ok(None);
    }

    let keypair_hex_bytes = fs::read(&path).await?;
    let keypair_bytes = decode(keypair_hex_bytes).map_err(|err| {
        Error::Configuration(format!(
            "couldn't hex-decode network keypair bytes read from {}: {}",
            path.display(),
            err
        ))
    })?;

    let keypair = Keypair::from_bytes(&keypair_bytes).map_err(|err| {
        Error::Configuration(format!(
            "invalid network keypair bytes read from {}: {}",
            path.display(),
            err
        ))
    })?;

    Ok(Some(keypair))
}

/// Writes the public and secret key (hex-encoded) to different locations at disk.
pub(crate) async fn store_new_reward_keypair(root_dir: &Path, keypair: &Keypair) -> Result<()> {
    let secret_key_path = root_dir.join(REWARD_SECRET_KEY_FILENAME);
    let public_key_path = root_dir.join(REWARD_PUBLIC_KEY_FILENAME);
    fs::write(secret_key_path, encode(keypair.secret.to_bytes())).await?;
    fs::write(public_key_path, encode(keypair.public.to_bytes())).await?;

    Ok(())
}

/// Returns Some(PublicKey) or None if file doesn't exist. It assumes it's hex-encoded.
pub(crate) async fn get_reward_pk(root_dir: &Path) -> Result<Option<PublicKey>> {
    let path = root_dir.join(REWARD_PUBLIC_KEY_FILENAME);
    if !path.is_file() {
        return Ok(None);
    }

    let pk_hex_bytes = fs::read(&path).await?;
    let pk_bytes = decode(pk_hex_bytes).map_err(|err| {
        Error::Configuration(format!(
            "couldn't hex-decode rewards Ed25519 public key bytes from {}: {}",
            path.display(),
            err
        ))
    })?;

    let pk = PublicKey::from_bytes(&pk_bytes).map_err(|err| {
        Error::Configuration(format!(
            "invalid rewards Ed25519 public key bytes read from {}: {}",
            path.display(),
            err
        ))
    })?;

    Ok(Some(pk))
}

#[cfg(test)]
mod test {
    use super::{
        get_network_keypair, get_reward_pk, store_network_keypair, store_new_reward_keypair,
    };
    use eyre::{eyre, Result};
    use rand_07::rngs::OsRng;
    use tempfile::{tempdir, TempDir};

    #[tokio::test]
    async fn pubkey_to_and_from_file() -> Result<()> {
        let mut rng = OsRng;
        let keypair = ed25519_dalek::Keypair::generate(&mut rng);

        let root = create_temp_root()?;
        let root_dir = root.path();
        store_new_reward_keypair(root_dir, &keypair).await?;
        let pk_result = get_reward_pk(root_dir).await?;

        assert_eq!(pk_result, Some(keypair.public));
        Ok(())
    }

    #[tokio::test]
    async fn keypair_to_and_from_file() -> Result<()> {
        let mut rng = OsRng;
        let keypair = ed25519_dalek::Keypair::generate(&mut rng);

        let root = create_temp_root()?;
        let root_dir = root.path();

        let keypair_result = get_network_keypair(root_dir).await?;
        assert!(keypair_result.is_none());

        store_network_keypair(root_dir, keypair.to_bytes()).await?;
        let keypair_result = get_network_keypair(root_dir).await?;
        if let Some(kp) = keypair_result {
            assert_eq!(kp.public, keypair.public);
            Ok(())
        } else {
            Err(eyre!("Network keypair was not read from file"))
        }
    }

    // creates a temp dir
    fn create_temp_root() -> Result<TempDir> {
        tempdir().map_err(|e| eyre!("Failed to create temp dir: {}", e))
    }
}
