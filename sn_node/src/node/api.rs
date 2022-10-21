// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::comm::Comm;
use crate::{
    node::{
        flow_ctrl::{cmds::Cmd, event_channel::EventSender},
        Error, Event, MyNode, Result, GENESIS_DBC_AMOUNT,
    },
    UsedSpace,
};

use sn_interface::{
    network_knowledge::{
        MyNodeInfo, NetworkKnowledge, SectionAuthorityProvider, SectionKeyShare, SectionsDAG,
        GENESIS_DBC_SK,
    },
    types::log_markers::LogMarker,
};

use ed25519_dalek::Keypair;
use sn_dbc::{
    bls_ringct::{bls_bulletproofs::PedersenGens, group::Curve},
    rng, Dbc, Hash, IndexedSignatureShare, MlsagMaterial, Owner, OwnerOnce, RevealedCommitment,
    SpentProofContent, SpentProofShare, Token, TransactionBuilder, TrueInput,
};
use std::{path::PathBuf, sync::Arc};
use xor_name::XorName;

impl MyNode {
    pub(crate) async fn first_node(
        comm: Comm,
        keypair: Arc<Keypair>,
        event_sender: EventSender,
        used_space: UsedSpace,
        root_storage_dir: PathBuf,
        genesis_sk_set: bls::SecretKeySet,
    ) -> Result<(Self, Dbc)> {
        let our_addr = comm.socket_addr();
        let info = MyNodeInfo {
            keypair: keypair.clone(),
            addr: our_addr,
        };

        let genesis_dbc =
            gen_genesis_dbc(&genesis_sk_set, &bls::SecretKey::from_hex(GENESIS_DBC_SK)?)?;

        let (network_knowledge, section_key_share) =
            NetworkKnowledge::first_node(info.peer(), genesis_sk_set)?;

        let node = Self::new(
            comm,
            keypair.clone(),
            network_knowledge,
            Some(section_key_share),
            event_sender,
            used_space,
            root_storage_dir,
        )
        .await?;

        Ok((node, genesis_dbc))
    }

    pub(crate) fn relocate(
        &mut self,
        new_keypair: Arc<Keypair>,
        new_section: NetworkKnowledge,
    ) -> Result<()> {
        // we first try to relocate section info.
        self.network_knowledge.relocated_to(new_section)?;

        self.keypair = new_keypair;

        Ok(())
    }

    pub(crate) fn network_knowledge(&self) -> &NetworkKnowledge {
        &self.network_knowledge
    }

    pub(crate) fn section_chain(&self) -> SectionsDAG {
        self.network_knowledge.section_chain()
    }

    /// Is this node an elder?
    pub(crate) fn is_elder(&self) -> bool {
        self.network_knowledge.is_elder(&self.info().name())
    }

    pub(crate) fn is_not_elder(&self) -> bool {
        !self.is_elder()
    }

    /// Returns the current BLS public key set
    pub(crate) fn public_key_set(&self) -> Result<bls::PublicKeySet> {
        Ok(self.key_share()?.public_key_set)
    }

    /// Returns the SAP of the section matching the name.
    pub(crate) fn matching_section(&self, name: &XorName) -> Result<SectionAuthorityProvider> {
        self.network_knowledge
            .section_auth_by_name(name)
            .map_err(Error::from)
    }

    /// Returns our key share in the current BLS group if this node is a member of one, or
    /// `Error::MissingSecretKeyShare` otherwise.
    pub(crate) fn key_share(&self) -> Result<SectionKeyShare> {
        let section_key = self.network_knowledge.section_key();
        self.section_keys_provider
            .key_share(&section_key)
            .map_err(Error::from)
    }

    pub(crate) async fn send_event(&self, event: Event) {
        self.event_sender.send(event).await
    }

    // ----------------------------------------------------------------------------------------
    //   ---------------------------------- Mut ------------------------------------------
    // ----------------------------------------------------------------------------------------

    // Generate a new section info based on the current set of members, but
    // excluding the ones in the provided list. And if the outcome list of candidates
    // differs from the current elders, trigger a DKG.
    pub(crate) fn trigger_dkg(&mut self) -> Result<Vec<Cmd>> {
        debug!("{}", LogMarker::TriggeringPromotionAndDemotion);
        let mut cmds = vec![];
        for session_id in self.best_elder_candidates() {
            cmds.extend(self.send_dkg_start(session_id)?);
        }

        Ok(cmds)
    }
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
pub(crate) fn gen_genesis_dbc(
    genesis_section_sk_set: &bls::SecretKeySet,
    genesis_dbc_sk: &bls::SecretKey,
) -> Result<Dbc> {
    // Use the same key as the input and output of Genesis Tx.
    let output_owner =
        OwnerOnce::from_owner_base(Owner::from(genesis_dbc_sk.clone()), &mut rng::thread_rng());

    let revealed_commitment =
        RevealedCommitment::from_value(GENESIS_DBC_AMOUNT, &mut rng::thread_rng());

    // Use the same key as the input and output of Genesis Tx.
    let true_input = TrueInput::new(genesis_dbc_sk.clone(), revealed_commitment);

    // build our MlsagMaterial manually without randomness.
    // note: no decoy inputs because no other DBCs exist prior to genesis DBC.
    let mlsag_material = MlsagMaterial {
        true_input,
        decoy_inputs: vec![],
        pi_base: 0,
        alpha: (1234.into(), 5678.into()),
        r: vec![(9123.into(), 4567.into())],
    };

    let mut dbc_builder = TransactionBuilder::default()
        .add_input(mlsag_material)
        .add_output_by_amount(Token::from_nano(GENESIS_DBC_AMOUNT), output_owner)
        .build(&mut rng::thread_rng())
        .map_err(|err| {
            Error::GenesisDbcError(format!(
                "Failed to build the ringct transaction for genesis DBC: {}",
                err
            ))
        })?;

    let (key_image, tx) = dbc_builder.inputs().into_iter().next().ok_or_else(|| {
        Error::GenesisDbcError(
            "DBC builder (unexpectedly) contains an empty set of inputs.".to_string(),
        )
    })?;

    // let's build the spent proof and add it to the DBC builder
    let content = SpentProofContent {
        key_image,
        transaction_hash: Hash::from(tx.hash()),
        public_commitments: vec![revealed_commitment
            .commit(&PedersenGens::default())
            .to_affine()],
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
            "DBC builder failed to create output genesis DBC: {}",
            err
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
