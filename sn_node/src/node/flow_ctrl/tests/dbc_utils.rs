use crate::node::api::gen_genesis_dbc;
use eyre::{eyre, Result};
use sn_dbc::{
    Dbc, KeyImage, Owner, OwnerOnce, RingCtTransaction, SpentProof, SpentProofShare, Token,
    TransactionBuilder,
};
use sn_interface::{messaging::data::RegisterCmd, types::ReplicatedData};
use std::collections::BTreeSet;

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
    let genesis_dbc = gen_genesis_dbc(sk_set, &sk_set.secret_key())?;
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