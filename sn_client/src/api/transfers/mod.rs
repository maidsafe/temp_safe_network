// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod errors;

use crate::Client;
use crate::Result;

use sn_dbc::{
    rng, Dbc, DbcTransaction, Hash, Owner, OwnerOnce, RevealedAmount, SpentProof, SpentProofShare,
    Token, TransactionBuilder,
};
use sn_interface::{
    dbcs::DbcReason,
    elder_count,
    network_knowledge::supermajority,
    types::fees::{FeeCiphers, RequiredFee, SpendPriority},
};

use bls::PublicKey;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use xor_name::XorName;

pub use errors::Error;

type ReissueCiphers = BTreeMap<PublicKey, BTreeMap<XorName, (RequiredFee, OwnerOnce)>>;

///
#[derive(Debug)]
pub struct ReissueInputs {
    /// The dbcs to spend, as to send tokens.
    pub input_dbcs: Vec<Dbc>,
    /// The dbcs that will be created, holding the tokens to send.
    pub outputs: Vec<(Token, OwnerOnce)>,
    /// Any surplus amount after spending the necessary input dbcs.
    pub change_amount: Token,
    /// Total fees to pay for spending the inputs.
    #[cfg(not(feature = "data-network"))]
    pub fees_paid: Token,
    /// This is the set of input dbc keys, each having a set of
    /// Elder names and their respective fee ciphers.
    /// Sent together with spends, so that Elders can verify their fee payments.
    #[cfg(not(feature = "data-network"))]
    pub all_fee_cipher_params: ReissueCiphers,
}

/// The results of reissuing dbcs.
#[derive(Debug)]
pub struct ReissueOutputs {
    /// The dbcs holding the tokens that were sent.
    pub outputs: Vec<(Dbc, OwnerOnce, RevealedAmount)>,
    /// The dbc holding surplus amount after spending the necessary input dbcs.
    pub change: Option<Dbc>,
    /// The dbcs we spent when reissuing.
    pub spent_dbcs: BTreeSet<PublicKey>,
    /// Total fees to paid for this tx.
    #[cfg(not(feature = "data-network"))]
    pub fees_paid: Token,
}

// Number of attempts to make trying to spend inputs when reissuing DBCs
// As the spend and query cmds are cascaded closely, there is high chance
// that the first two query attempts could both be failed.
// Hence the max number of attempts set to a higher value.
const NUM_OF_DBC_REISSUE_ATTEMPTS: u8 = 5;

/// Send the tokens to the specified destination keys, using the provided dbcs.
/// The new dbcs that are created, one per specified destination, will have the
/// unique id which is the public key of the `OwnerOnce` instances provided.
///
/// Transfer fees will be paid if not in data-network.
/// The input dbcs will be spent on the network, and the resulting
/// dbcs (and change dbc if any) are returned.
/// NB: We are skipping the DbcReason arg for now. It can be added later.
pub async fn send_tokens(
    client: &Client,
    dbcs: Vec<Dbc>,
    recipients: Vec<(Token, OwnerOnce)>,
    priority: SpendPriority,
) -> Result<ReissueOutputs> {
    // We need to select the necessary number of dbcs from those that we were passed.
    // This will also account for any fees.
    let reissue_inputs = select_inputs(client, dbcs, recipients, priority).await?;

    // then we can reissue
    reissue_dbcs(client, reissue_inputs, DbcReason::none()).await
}

/// Select the necessary number of dbcs out of those passed in,
/// to pay for the outputs specified.
/// This will also account for any fees.
pub async fn select_inputs(
    client: &Client,
    dbcs: Vec<Dbc>,
    mut outputs: Vec<(Token, OwnerOnce)>,
    priority: SpendPriority,
) -> Result<ReissueInputs> {
    // We'll combine one or more input DBCs and reissue:
    // - one output DBC per recipient,
    // - and a single DBC for the change - if any - which will be returned from this function.
    let mut input_dbcs = Vec::<Dbc>::new();
    let mut total_input_amount = Token::zero();
    let mut total_output_amount = outputs
        .iter()
        .fold(Some(Token::zero()), |total, (amount, _)| {
            total.and_then(|t| t.checked_add(*amount))
        })
        .ok_or_else(|| {
            Error::DbcReissueError(
                "Overflow occurred while summing the output amounts for the output DBCs."
                    .to_string(),
            )
        })?;

    let mut change_amount = total_output_amount;
    let mut rng = rng::thread_rng();
    #[cfg(not(feature = "data-network"))]
    let mut all_fee_cipher_params = BTreeMap::new();
    #[cfg(not(feature = "data-network"))]
    let mut fees_paid = Token::zero();

    for dbc in dbcs {
        let revealed_bearer = match dbc.as_revealed_input_bearer().map_err(Error::DbcError) {
            Ok(revealed) => revealed,
            Err(error) => {
                error!(
                    "Could not get the secret key from dbc {:?}, it is not a bearer! {error}",
                    dbc.public_key()
                );
                continue;
            }
        };
        let input_key = revealed_bearer.public_key();

        // ------------ fee part start ----------------
        #[cfg(not(feature = "data-network"))]
        let fee_per_input = {
            // Each section will have elder_count() instances to pay individually (for now, later they will be more).
            let elder_fees = match client.get_section_fees(input_key, priority).await {
                Ok(fees) => fees,
                Err(error) => {
                    error!("Could not get fees for input dbc: {input_key:?}: {error}");
                    continue;
                }
            };
            let num_responses = elder_fees.len();
            let required_responses = supermajority(elder_count());
            if required_responses > num_responses {
                warn!("Not enough elders contacted for the section to spend the input. Got: {num_responses}, needed: {required_responses}");
                continue;
            }

            // Fees that were not encrypted to us.
            let mut invalid_fees = BTreeSet::new();
            // As the Elders encrypt the amount to our public key, we need to decrypt it.
            let mut decrypted_elder_fees = vec![];

            for (elder, fee) in elder_fees {
                match fee.content.decrypt_amount(&revealed_bearer.secret_key) {
                    Ok(amount) => decrypted_elder_fees.push(((elder, fee), amount)),
                    Err(error) => {
                        error!("Decrypting the fee content from {elder} failed! {error}");
                        let _ = invalid_fees.insert(fee.content.elder_reward_key);
                    }
                }
            }

            let max_invalid_fees = elder_count() - required_responses;
            if invalid_fees.len() > max_invalid_fees {
                let valid_responses = num_responses - invalid_fees.len();
                warn!("Not enough valid fees received from the section to spend the input. Found: {valid_responses}, needed: {required_responses}", );
                continue;
            }

            // Total fee paid to all recipients in the section for this input.
            let fee_per_input = decrypted_elder_fees
                .iter()
                .fold(Some(Token::zero()), |total, (_, fee)| {
                    total.and_then(|t| t.checked_add(*fee))
                })
                .ok_or_else(|| Error::DbcReissueError(
                    "Overflow occurred while summing the individual Elder's fees in order to calculate the total amount for the output DBCs."
                        .to_string(),
                ))?;

            let mut fee_cipher_params = BTreeMap::new();

            // Add elders to outputs and generate their fee ciphers.
            decrypted_elder_fees
                .iter()
                .for_each(|((elder, required_fee), fee)| {
                    let owner = Owner::from(required_fee.content.elder_reward_key);
                    let owner_once = OwnerOnce::from_owner_base(owner, &mut rng);
                    outputs.push((*fee, owner_once.clone()));

                    let _ =
                        fee_cipher_params.insert(elder.name(), (required_fee.clone(), owner_once));
                });

            let _ = all_fee_cipher_params.insert(input_key, fee_cipher_params);

            fees_paid = fees_paid.checked_add(fee_per_input).ok_or_else(|| {
                Error::DbcReissueError(
                    "Overflow occurred while summing all the input fees.".to_string(),
                )
            })?;

            fee_per_input
        };
        // ---------------- fee part end ----------------

        let dbc_balance = match dbc.revealed_amount_bearer() {
            Ok(revealed_amount) => Token::from_nano(revealed_amount.value()),
            Err(err) => {
                warn!("Ignoring input Dbc (id: {input_key:?}) due to not being a bearer: {err:?}");
                continue;
            }
        };

        // Add this Dbc as input to be spent.
        input_dbcs.push(dbc.clone());

        // Input amount increases with the amount of the dbc.
        total_input_amount = total_input_amount.checked_add(dbc_balance)
            .ok_or_else(|| {
                Error::DbcReissueError(
                    "Overflow occurred while increasing total input amount while trying to cover the output DBCs."
                    .to_string(),
            )
            })?;

        #[cfg(not(feature = "data-network"))]
        {
            // Output amount now increases a bit, as we have to cover the fee as well..
            total_output_amount = total_output_amount.checked_add(fee_per_input)
            .ok_or_else(|| {
                Error::DbcReissueError(
                "Overflow occurred while adding mint fee in order to calculate the total amount for the output DBCs."
                    .to_string(),
            )
            })?;
            // ..and so does `change_amount` (that we subtract from to know if we've covered `total_output_amount`).
            change_amount = change_amount.checked_add(fee_per_input)
            .ok_or_else(|| {
                Error::DbcReissueError(
                "Overflow occurred while adding mint fee in order to calculate the total amount for the output DBCs."
                    .to_string(),
            )
            })?;
        }

        // If we've already combined input DBCs for the total output amount, then stop.
        match change_amount.checked_sub(dbc_balance) {
            Some(pending_output) => {
                change_amount = pending_output;
                if change_amount.as_nano() == 0 {
                    break;
                }
            }
            None => {
                change_amount = Token::from_nano(dbc_balance.as_nano() - change_amount.as_nano());
                break;
            }
        }
    }

    // If not enough spendable was found, this check will return an error.
    verify_amounts(total_input_amount, total_output_amount)?;

    Ok(ReissueInputs {
        input_dbcs,
        outputs,
        change_amount,
        #[cfg(not(feature = "data-network"))]
        fees_paid,
        #[cfg(not(feature = "data-network"))]
        all_fee_cipher_params,
    })
}

/// The input dbcs will be spent on the network, and the resulting
/// dbcs (and change dbc if any) are returned.
/// This will pay transfer fees if not in data-network.
async fn reissue_dbcs(
    client: &Client,
    reissue_inputs: ReissueInputs,
    reason: DbcReason,
) -> Result<ReissueOutputs> {
    let ReissueInputs {
        input_dbcs,
        outputs,
        change_amount,
        #[cfg(not(feature = "data-network"))]
        fees_paid,
        #[cfg(not(feature = "data-network"))]
        all_fee_cipher_params,
    } = reissue_inputs;

    let mut tx_builder = TransactionBuilder::default()
        .add_inputs_dbc_bearer(input_dbcs.iter())
        .map_err(Error::DbcError)?
        .add_outputs_by_amount(outputs.into_iter().map(|(token, owner)| (token, owner)));

    let change_owneronce =
        OwnerOnce::from_owner_base(client.dbc_owner().clone(), &mut rng::thread_rng());
    if change_amount.as_nano() > 0 {
        tx_builder = tx_builder.add_output_by_amount(change_amount, change_owneronce.clone());
    }

    let proof_key_verifier = SpentProofKeyVerifier { client };
    let inputs_spent_proofs: BTreeSet<SpentProof> = input_dbcs
        .iter()
        .flat_map(|dbc| dbc.inputs_spent_proofs.clone())
        .collect();
    let inputs_spent_transactions: BTreeSet<DbcTransaction> = input_dbcs
        .iter()
        .flat_map(|dbc| dbc.inputs_spent_transactions.clone())
        .collect();

    // Finalize the tx builder to get the dbc builder.
    let mut dbc_builder = tx_builder
        .build(rng::thread_rng())
        .map_err(Error::DbcError)?;

    // Get fee outputs to generate the fee ciphers.
    #[cfg(not(feature = "data-network"))]
    let outputs = dbc_builder
        .revealed_outputs
        .iter()
        .map(|output| (output.public_key, output.revealed_amount))
        .collect();

    let inputs = dbc_builder.inputs();
    let spent_dbcs = inputs.iter().map(|(key, _)| key).cloned().collect();

    // Spend all the input DBCs, collecting the spent proof shares for each of them
    for (public_key, tx) in inputs {
        // Generate the fee ciphers.
        #[cfg(not(feature = "data-network"))]
        let input_fee_ciphers = {
            let fee_cipher_params = all_fee_cipher_params
                .get(&public_key)
                .ok_or(Error::DbcReissueError("Missing fee!".to_string()))?;
            fee_ciphers(&outputs, fee_cipher_params)?
        };

        let tx_hash = Hash::from(tx.hash());
        // TODO: spend DBCs concurrently spawning tasks
        let mut attempts = 0;
        loop {
            attempts += 1;
            client
                .spend_dbc(
                    public_key,
                    tx.clone(),
                    reason,
                    inputs_spent_proofs.clone(),
                    inputs_spent_transactions.clone(),
                    #[cfg(not(feature = "data-network"))]
                    input_fee_ciphers.clone(),
                )
                .await?;

            let spent_proof_shares = client.spent_proof_shares(public_key).await?;

            // TODO: we temporarilly filter the spent proof shares which correspond to the TX we
            // are spending now. This is because current implementation of Spentbook allows
            // double spents, so we may be retrieving spent proof shares for others spent TXs.
            let shares_for_current_tx: HashSet<SpentProofShare> = spent_proof_shares
                .into_iter()
                .filter(|proof_share| proof_share.content.transaction_hash == tx_hash)
                .collect();

            match verify_spent_proof_shares_for_tx(
                public_key,
                tx_hash,
                &shares_for_current_tx,
                &proof_key_verifier,
            ) {
                Ok(()) => {
                    dbc_builder = dbc_builder
                        .add_spent_proof_shares(shares_for_current_tx.into_iter())
                        .add_spent_transaction(tx);
                    break;
                }
                Err(err) if attempts == NUM_OF_DBC_REISSUE_ATTEMPTS => {
                    return Err(Error::DbcReissueError(format!(
                        "Failed to spend input, {} proof shares obtained from spentbook: {}",
                        shares_for_current_tx.len(),
                        err
                    )))?;
                }
                Err(_) => {}
            }
        }
    }

    // Perform verifications of input TX and spentproofs,
    // as well as building the output DBCs.
    let mut output_dbcs = dbc_builder
        .build(&proof_key_verifier)
        .map_err(Error::DbcError)?;

    let mut change_dbc = None;
    output_dbcs.retain(|(dbc, owneronce, _)| {
        if owneronce == &change_owneronce && change_amount.as_nano() > 0 {
            change_dbc = Some(dbc.clone());
            false
        } else {
            true
        }
    });

    Ok(ReissueOutputs {
        outputs: output_dbcs,
        change: change_dbc,
        spent_dbcs,
        #[cfg(not(feature = "data-network"))]
        fees_paid,
    })
}

/// This will encrypt the necessary components of a fee payment,
/// so that an Elder can find and verify the fee paid to them, from
/// within a request to spend a dbc.
#[cfg(not(feature = "data-network"))]
fn fee_ciphers(
    outputs: &BTreeMap<PublicKey, RevealedAmount>,
    fee_cipher_params: &BTreeMap<XorName, (RequiredFee, OwnerOnce)>,
) -> errors::Result<BTreeMap<XorName, FeeCiphers>> {
    let mut input_fee_ciphers = BTreeMap::new();
    for (elder_name, (required_fee, owner_once)) in fee_cipher_params {
        // Encrypt the index to the _well-known reward key_.
        let derivation_index_cipher = required_fee
            .content
            .elder_reward_key
            .encrypt(owner_once.derivation_index);

        let output_owner_pk = owner_once.as_owner().public_key();
        let revealed_amount = outputs
            .get(&output_owner_pk)
            .ok_or(Error::DbcReissueError("Missing output!".to_string()))?;

        // Encrypt the amount to the _derived key_ (i.e. new dbc id).
        let amount_cipher = revealed_amount.encrypt(&output_owner_pk);
        let _ = input_fee_ciphers.insert(
            *elder_name,
            FeeCiphers::new(amount_cipher, derivation_index_cipher),
        );
    }
    Ok(input_fee_ciphers)
}

// Private helper to verify if a set of spent proof shares are valid for a given public_key and TX
fn verify_spent_proof_shares_for_tx(
    public_key: PublicKey,
    tx_hash: Hash,
    proof_shares: &HashSet<SpentProofShare>,
    proof_key_verifier: &SpentProofKeyVerifier,
) -> errors::Result<()> {
    SpentProof::try_from_proof_shares(public_key, tx_hash, proof_shares)
        .and_then(|spent_proof| spent_proof.verify(tx_hash, proof_key_verifier))?;

    Ok(())
}

// Make sure total input amount gathered with input DBCs are enough for the output amount
fn verify_amounts(total_input_amount: Token, total_output_amount: Token) -> errors::Result<()> {
    if total_output_amount > total_input_amount {
        return Err(Error::NotEnoughBalance(total_input_amount.to_string()));
    }
    Ok(())
}

/// Verifier required by sn_dbc API to check a SpentProof
/// is signed by known sections keys.
struct SpentProofKeyVerifier<'a> {
    client: &'a Client,
}

impl sn_dbc::SpentProofKeyVerifier for SpentProofKeyVerifier<'_> {
    type Error = crate::Error;

    // Called by sn_dbc API when it needs to verify a SpentProof is signed by a known key,
    // we check if the key is any of the network sections keys we are aware of
    fn verify_known_key(&self, key: &PublicKey) -> Result<()> {
        if !futures::executor::block_on(self.client.is_known_section_key(key)) {
            Err(Error::DbcVerificationFailed(format!(
                "SpentProof key is an unknown section key: {}",
                key.to_hex()
            )))?
        } else {
            Ok(())
        }
    }
}
