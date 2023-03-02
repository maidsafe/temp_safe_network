// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use sn_dbc::{
    rng, Dbc, Error as DbcError, Hash, IndexedSignatureShare, Owner, OwnerOnce, PedersenGens,
    RevealedCommitment, RevealedInput, SpentProofContent, SpentProofShare, Token,
    TransactionBuilder,
};

use std::{fmt::Debug, result};
use thiserror::Error;

mod reasons;

pub use reasons::DbcReason;

/// Amount of tokens to be owned by the Genesis DBC.
/// At the inception of the Network a total supply of 4,525,524,120 whole tokens will be created.
/// Each whole token can be subdivided 10^9 times,
/// thus creating a total of 4,525,524,120,000,000,000 available units.
pub const GENESIS_DBC_AMOUNT: u64 = 4_525_524_120 * u64::pow(10, 9);

/// A specialised `Result` type for types crate.
pub type Result<T> = result::Result<T, Error>;

/// Main error type for the crate.
#[derive(Error, Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    /// Error occurred when minting the Genesis DBC.
    #[error("Genesis DBC error:: {0}")]
    GenesisDbcError(String),
    /// FailedToParseReason
    #[error("Failed to parse reason: {0}")]
    FailedToParseReason(#[from] DbcError),
}

/// Generate the genesis DBC.
///
/// Requires the initial section key to sign the share and a different secret key for the DBC.
///
/// The genesis DBC will be created using a different key from the initial section key. This is
/// because the genesis DBC, along with its secret key, will be publicly available for auditing
/// purposes. It needs to be a set rather than just a key because a spent proof share gets
/// generated, which requires a key set. We can't use the same key for the genesis DBC and section
/// because if the genesis DBC is publicly available, the secret key could be used to create a bad
/// section that would be accepted by the network.
pub fn gen_genesis_dbc(
    genesis_section_sk_set: &bls::SecretKeySet,
    genesis_dbc_sk: &bls::SecretKey,
) -> Result<Dbc> {
    // Use the same key as the input and output of Genesis Tx.
    let output_owner =
        OwnerOnce::from_owner_base(Owner::from(genesis_dbc_sk.clone()), &mut rng::thread_rng());

    let revealed_commitment = RevealedCommitment::from_value(GENESIS_DBC_AMOUNT, rng::thread_rng());

    // Use the same key as the input and output of Genesis Tx.
    let genesis_input = RevealedInput::new(genesis_dbc_sk.clone(), revealed_commitment);

    let mut dbc_builder = TransactionBuilder::default()
        .add_input(genesis_input)
        .add_output_by_amount(Token::from_nano(GENESIS_DBC_AMOUNT), output_owner)
        .build(rng::thread_rng())
        .map_err(|err| {
            Error::GenesisDbcError(format!(
                "Failed to build the DBC transaction for genesis DBC: {err}",
            ))
        })?;

    let (public_key, tx) = dbc_builder.inputs().into_iter().next().ok_or_else(|| {
        Error::GenesisDbcError(
            "DBC builder (unexpectedly) contains an empty set of inputs.".to_string(),
        )
    })?;

    // let's build the spent proof and add it to the DBC builder
    let content = SpentProofContent {
        public_key,
        transaction_hash: Hash::from(tx.hash()),
        reason: Hash::default(),
        public_commitment: revealed_commitment.commit(&PedersenGens::default()),
    };

    let sk_share_index = 0;
    let sig_share = genesis_section_sk_set
        .secret_key_share(sk_share_index)
        .sign(content.hash().as_ref());
    let spentbook_sig_share = IndexedSignatureShare::new(sk_share_index, sig_share);

    let spent_proof_share = SpentProofShare {
        content,
        spentbook_pks: genesis_section_sk_set.public_keys(),
        spentbook_sig_share,
    };

    dbc_builder = dbc_builder
        .add_spent_proof_share(spent_proof_share)
        .add_spent_transaction(tx);

    // build the output DBCs
    let outputs = dbc_builder.build_without_verifying().map_err(|err| {
        Error::GenesisDbcError(format!(
            "DBC builder failed to create output genesis DBC: {err}",
        ))
    })?;

    // just one output DBC is expected which is the genesis DBC
    let (genesis_dbc, _, _) = outputs.into_iter().next().ok_or_else(|| {
        Error::GenesisDbcError(
            "DBC builder (unexpectedly) contains an empty set of outputs.".to_string(),
        )
    })?;

    Ok(genesis_dbc)
}
