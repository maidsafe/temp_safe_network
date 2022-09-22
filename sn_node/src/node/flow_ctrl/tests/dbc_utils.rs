use crate::node::{api::gen_genesis_dbc, flow_ctrl::dispatcher::Dispatcher};
use eyre::{eyre, Result};
use sn_dbc::{
    get_public_commitments_from_transaction, Commitment, Dbc, Hash, IndexedSignatureShare,
    KeyImage, Owner, OwnerOnce, PublicKey, RingCtTransaction, Signature, SpentProof,
    SpentProofContent, SpentProofShare, Token, TransactionBuilder,
};
use sn_interface::network_knowledge::section_keys::build_spent_proof_share;
use sn_interface::{
    messaging::data::{DataCmd, RegisterCmd, ServiceMsg, SpentbookCmd},
    network_knowledge::{SectionAuthorityProvider, SectionKeysProvider},
    types::{Peer, ReplicatedData},
};
use std::collections::BTreeSet;
use std::str::FromStr;

struct FakeProofKeyVerifier {}
impl sn_dbc::SpentProofKeyVerifier for FakeProofKeyVerifier {
    type Error = crate::node::Error;

    fn verify_known_key(&self, _key: &PublicKey) -> std::result::Result<(), crate::node::Error> {
        Ok(())
    }
}

/// Get the spent proof share that's packaged inside the data that's to be replicated to the adults
/// in the section.
pub(crate) fn get_spent_proof_share_from_replicated_data(
    replicated_data: ReplicatedData,
) -> Result<SpentProofShare> {
    match replicated_data {
        ReplicatedData::SpentbookWrite(reg_cmd) => match reg_cmd {
            RegisterCmd::Edit(signed_edit) => {
                let entry = signed_edit.op.edit.crdt_op.value;
                let spent_proof_share: SpentProofShare = rmp_serde::from_slice(&entry)?;
                Ok(spent_proof_share)
            }
            _ => Err(eyre!("A RegisterCmd::Edit variant was expected")),
        },
        _ => Err(eyre!(
            "A ReplicatedData::SpentbookWrite variant was expected"
        )),
    }
}

/// Returns the info necessary to populate the `SpentbookCmd::Spend` message to be handled.
///
/// The genesis DBC is used, but that doesn't really matter; for testing the code in the message
/// handler we could use any DBC.
///
/// The `gen_genesis_dbc` function returns the DBC itself. To put it through the spending message
/// handler, it needs to have a transaction, which is what we provide here before we return it
/// back for use in tests.
pub(crate) fn get_genesis_dbc_spend_info(
    sk_set: &bls::SecretKeySet,
) -> Result<(
    KeyImage,
    RingCtTransaction,
    BTreeSet<SpentProof>,
    BTreeSet<RingCtTransaction>,
)> {
    let genesis_dbc = gen_genesis_dbc(sk_set)?;
    let dbc_owner = genesis_dbc.owner_base().clone();
    let output_owner = OwnerOnce::from_owner_base(dbc_owner, &mut rand::thread_rng());
    let tx_builder = TransactionBuilder::default()
        .set_decoys_per_input(0)
        .set_require_all_decoys(false)
        .add_input_dbc_bearer(&genesis_dbc)?;
    let inputs_amount_sum = tx_builder.inputs_amount_sum();
    let dbc_builder = tx_builder
        .add_output_by_amount(inputs_amount_sum, output_owner)
        .build(&mut rand::thread_rng())?;
    let (key_image, tx) = &dbc_builder.inputs()[0];
    Ok((
        *key_image,
        tx.clone(),
        genesis_dbc.spent_proofs.clone(),
        genesis_dbc.spent_transactions,
    ))
}

/// Reissue a new DBC (at a particular amount) from a given input DBC.
///
/// The change DBC will be discarded.
///
/// A spent proof share is generated for the input DBC, but it doesn't go through the complete
/// spending validation process. This should be OK for the testing process.
///
/// This function was originally setup to use the dispatcher to send a `SpentbookCmd::Spend`
/// message to avoid duplication of code, but unfortunately this causes a stack overflow during the
/// test run. Thus, some functions in the `Node` type were scoped at `pub(crate)` and these are
/// called here.
pub(crate) fn reissue_dbc(
    input: &Dbc,
    amount: u64,
    output_owner_sk: &bls::SecretKey,
    sap: &SectionAuthorityProvider,
    section_keys_provider: &SectionKeysProvider,
) -> Result<Dbc> {
    let output_amount = Token::from_nano(amount);
    let input_amount = input.amount_secrets_bearer()?.amount();
    let change_amount = input_amount
        .checked_sub(output_amount)
        .ok_or_else(|| eyre!("The input amount minus the amount must evaluate to a valid value"))?;

    let mut rng = rand::thread_rng();
    let output_owner = Owner::from(output_owner_sk.clone());
    let mut dbc_builder = TransactionBuilder::default()
        .set_decoys_per_input(0)
        .set_require_all_decoys(false)
        .add_input_dbc_bearer(input)?
        .add_output_by_amount(
            output_amount,
            OwnerOnce::from_owner_base(output_owner, &mut rng),
        )
        .add_output_by_amount(
            change_amount,
            OwnerOnce::from_owner_base(input.owner_base().clone(), &mut rng),
        )
        .build(rng)?;
    for (key_image, tx) in dbc_builder.inputs() {
        let public_commitments = get_public_commitments_from_transaction(
            &tx,
            &input.spent_proofs,
            &input.spent_transactions,
        )?;
        let public_commitments: Vec<Commitment> = public_commitments
            .into_iter()
            .flat_map(|(k, v)| if k == key_image { v } else { vec![] })
            .collect();
        let spent_proof_share = build_spent_proof_share(
            &key_image,
            &tx,
            sap,
            section_keys_provider,
            public_commitments,
        )?;
        dbc_builder = dbc_builder
            .add_spent_proof_share(spent_proof_share)
            .add_spent_transaction(tx);
    }
    let verifier = FakeProofKeyVerifier {};
    let output_dbcs = dbc_builder.build(&verifier)?;
    let (output_dbc, ..) = output_dbcs
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("At least one output DBC should have been generated"))?;
    Ok(output_dbc)
}

pub(crate) fn get_input_dbc_spend_info(
    input: &Dbc,
    amount: u64,
    output_owner_sk: &bls::SecretKey,
) -> Result<(PublicKey, RingCtTransaction)> {
    let output_amount = Token::from_nano(amount);
    let input_amount = input.amount_secrets_bearer()?.amount();
    let change_amount = input_amount
        .checked_sub(output_amount)
        .ok_or_else(|| eyre!("The input amount minus the amount must evaluate to a valid value"))?;

    let mut rng = rand::thread_rng();
    let output_owner = Owner::from(output_owner_sk.clone());
    let dbc_builder = TransactionBuilder::default()
        .set_decoys_per_input(0)
        .set_require_all_decoys(false)
        .add_input_dbc_bearer(input)?
        .add_output_by_amount(
            output_amount,
            OwnerOnce::from_owner_base(output_owner, &mut rng),
        )
        .add_output_by_amount(
            change_amount,
            OwnerOnce::from_owner_base(input.owner_base().clone(), &mut rng),
        )
        .build(rng)?;
    let inputs = dbc_builder.inputs();
    let first = inputs
        .first()
        .ok_or_else(|| eyre!("There must be at least one input on the transaction"))?;
    Ok(first.clone())
}

pub(crate) fn reissue_invalid_dbc_with_no_inputs(
    input: &Dbc,
    amount: u64,
    output_owner_sk: &bls::SecretKey,
) -> Result<Dbc> {
    let output_amount = Token::from_nano(amount);
    let input_amount = input.amount_secrets_bearer()?.amount();
    let change_amount = input_amount
        .checked_sub(output_amount)
        .ok_or_else(|| eyre!("The input amount minus the amount must evaluate to a valid value"))?;

    let mut rng = rand::thread_rng();
    let output_owner = Owner::from(output_owner_sk.clone());
    let dbc_builder = TransactionBuilder::default()
        .set_decoys_per_input(0)
        .set_require_all_decoys(false)
        .add_output_by_amount(
            output_amount,
            OwnerOnce::from_owner_base(output_owner, &mut rng),
        )
        .add_output_by_amount(
            change_amount,
            OwnerOnce::from_owner_base(input.owner_base().clone(), &mut rng),
        )
        .build(rng)?;
    let output_dbcs = dbc_builder.build_without_verifying()?;
    let (output_dbc, ..) = output_dbcs
        .into_iter()
        .next()
        .ok_or_else(|| eyre!("At least one output DBC should have been generated"))?;
    Ok(output_dbc)
}
