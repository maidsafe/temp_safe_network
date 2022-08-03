// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#![allow(dead_code)]

use super::errors::{Error, Result};
use crdts::{
    merkle_reg::{Hash, MerkleReg, Node, Sha3Hash},
    CmRDT,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Debug, Formatter},
    iter, mem,
};
use tiny_keccak::{Hasher, Sha3};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct SectionInfo {
    key: bls::PublicKey,
    sig: bls::Signature,
}

impl Debug for SectionInfo {
    fn fmt(&self, formatter: &mut Formatter) -> std::fmt::Result {
        let bytes: Vec<u8> = self
            .key
            .to_bytes()
            .into_iter()
            .chain(self.sig.to_bytes().into_iter())
            .collect();
        let hex = hex::encode(bytes);
        let hex: String = hex.chars().into_iter().take(10).collect();
        write!(formatter, "SectionInfo({})", hex)
    }
}

impl Sha3Hash for SectionInfo {
    fn hash(&self, hasher: &mut Sha3) {
        hasher.update(&self.key.to_bytes());
        hasher.update(&self.sig.to_bytes());
    }
}

/// A Merkle DAG of BLS keys where every key is signed by its parent key, except the genesis one.
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SectionsDAG {
    genesis_key: bls::PublicKey,
    dag: MerkleReg<SectionInfo>,
    dag_root: BTreeSet<bls::PublicKey>,
    hashes: BTreeMap<bls::PublicKey, Hash>,
}

impl Debug for SectionsDAG {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.keys().format(", "))
    }
}

impl SectionsDAG {
    /// Creates a new DAG containing only the `genesis_key`
    pub fn new(genesis_key: bls::PublicKey) -> Self {
        Self {
            genesis_key,
            dag: MerkleReg::new(),
            dag_root: BTreeSet::new(),
            hashes: BTreeMap::new(),
        }
    }

    /// Insert new key into the DAG. `parent_key` must exist in the DAG and must validate
    /// `signature`, otherwise error is returned.
    pub fn insert(
        &mut self,
        parent_key: &bls::PublicKey,
        key: bls::PublicKey,
        signature: bls::Signature,
    ) -> Result<()> {
        if !self.verify_sig(parent_key, &key, &signature) {
            return Err(Error::InvalidSignature);
        }

        let parent = if *parent_key == self.genesis_key {
            self.dag_root.insert(key);
            BTreeSet::new()
        } else {
            let hash = self
                .hashes
                .get(parent_key)
                .ok_or(Error::KeyNotFound(*parent_key))?;
            BTreeSet::from([*hash])
        };

        let info = SectionInfo {
            key,
            sig: signature,
        };
        // dag's children arg = parent
        let child = self.dag.write(info, parent);
        self.hashes.insert(key, child.hash());
        self.dag.apply(child);

        Ok(())
    }

    /// Insert new key into the DAG. `parent_key` must exist in the DAG and must validate
    /// `signature`, otherwise error is returned.
    fn insert_node(&mut self, parent_key: &bls::PublicKey, node: Node<SectionInfo>) -> Result<()> {
        self.insert(parent_key, node.value.key, node.value.sig)
    }

    /// Get a partial `SectionsDAG` from the `from` key to the `to` key
    /// Returns `Error::KeyNotFound` if the `to` key is not present in the DAG;
    /// Returns `Error::InvalidBranch` if the `from` key is not found or is not a direct ancestor of the `to` key
    pub fn partial_dag(&self, from: &bls::PublicKey, to: &bls::PublicKey) -> Result<Self> {
        // start from the "to" key (bottom of the tree) and traverse to the root
        let mut crdt_ops: Vec<Node<SectionInfo>> = Vec::new();
        // if "to" key is genesis, returns error
        let mut node = self.get_node(to)?;
        loop {
            if node.value.key == *from {
                // we have reached the end
                break;
            }
            // we don't insert the `from` node
            crdt_ops.push(node.clone());
            match self.parent_node(node.hash()) {
                Some(parent) => node = parent,
                // reached root of the dag
                None => {
                    // after reaching dag_root, return the partial_dag if from == genesis
                    if *from == self.genesis_key {
                        break;
                    }
                    // from key was apparently not a direct ancestor
                    return Err(Error::InvalidBranch);
                }
            }
        }
        // create the SectionsDAG and apply the CRDT ops
        let mut dag = SectionsDAG::new(*from);
        let mut parent = *from;
        for node in crdt_ops.into_iter().rev() {
            let key = node.value.key;
            dag.insert_node(&parent, node)?;
            parent = key;
        }
        Ok(dag)
    }

    /// Update our current `SectionsDAG` with the keys from another `SectionsDAG`
    /// Returns `Error::InvalidSignature` if the provided DAG fails signature verification
    /// Returns `Error::KeyNotFound` if the genesis_key of either of the DAGs is not present in the
    /// other
    pub fn merge(&mut self, mut other: Self) -> Result<()> {
        if !other.self_verify() {
            return Err(Error::InvalidSignature);
        }
        // find which DAG is the parent
        if !self.has_key(other.genesis_key()) {
            if other.has_key(self.genesis_key()) {
                mem::swap(self, &mut other);
            } else {
                return Err(Error::KeyNotFound(self.genesis_key));
            }
        }
        // insert all the nodes from `other`
        let mut need_to_visit: Vec<(bls::PublicKey, Node<SectionInfo>)> = Vec::new();
        for key in other.dag_root.iter() {
            need_to_visit.push((*other.genesis_key(), other.get_node(key)?));
        }
        while let Some((parent_key, current_node)) = need_to_visit.pop() {
            need_to_visit.extend(
                other
                    .child_nodes(current_node.hash())
                    .into_iter()
                    .map(|child_node| (current_node.value.key, child_node)),
            );
            self.insert_node(&parent_key, current_node)?;
        }

        Ok(())
    }

    /// Returns the genesis key
    pub fn genesis_key(&self) -> &bls::PublicKey {
        &self.genesis_key
    }

    /// Returns the list of the leaf keys
    pub fn leaf_keys(&self) -> BTreeSet<bls::PublicKey> {
        // if leaf_sections is empty, genesis is the only leaf
        let leaf_sections = self.non_genesis_leaf_sections();
        if leaf_sections.is_empty() {
            BTreeSet::from([self.genesis_key])
        } else {
            leaf_sections.iter().map(|info| info.key).collect()
        }
    }

    /// Iterator over all the keys in the `SectionsDAG`
    pub fn keys(&self) -> impl Iterator<Item = &bls::PublicKey> {
        iter::once(&self.genesis_key).chain(self.dag.all_nodes().map(|node| &node.value.key))
    }

    /// Returns whether `key` is present in this `SectionsDAG`.
    pub fn has_key(&self, key: &bls::PublicKey) -> bool {
        key == &self.genesis_key || self.hashes.contains_key(key)
    }

    /// Verify every BLS key in the DAG is proven (signed) by its parent key,
    /// except the genesis key
    pub fn self_verify(&self) -> bool {
        let mut verified_keys = BTreeSet::from([self.genesis_key]);
        // if we contain only genesis, non_genesis_leaf_sections returns empty list; hence return
        // true since no verification is required
        self.non_genesis_leaf_sections().iter().all(|section| {
            let mut key = section.key;
            loop {
                let parent = match self.get_parent_key(&key) {
                    // cannot be None as section.key can never be genesis
                    Ok(Some(parent)) => parent,
                    _ => break false,
                };

                let sig = match self.get_node(&key) {
                    Ok(node) => node.value.sig,
                    Err(_) => {
                        error!(
                            "SectionsDAG::self_verify() unreachable. Can be error only if hashes go out of sync"
                        );
                        break false;
                    }
                };

                if !self.verify_sig(&parent, &key, &sig) {
                    break false;
                }
                verified_keys.insert(key);
                // we stop when parent is verified; meaning all ancestors are verified.
                if verified_keys.contains(&parent) {
                    break true;
                }
                // go up the chain and keep verifying its parent
                key = parent;
            }
        })
    }

    /// Returns `true` if the `genesis_key` is present in the list of `trusted_keys`
    pub fn check_trust<'a, I>(&self, trusted_keys: I) -> bool
    where
        I: IntoIterator<Item = &'a bls::PublicKey>,
    {
        let trusted_keys: BTreeSet<_> = trusted_keys.into_iter().collect();
        trusted_keys.contains(&self.genesis_key)
    }

    /// Returns the parent of the provided key. None is returned if we're provided the `genesis_key`
    /// Can return `Error::KeyNotFound`
    pub fn get_parent_key(&self, key: &bls::PublicKey) -> Result<Option<bls::PublicKey>> {
        if *key == self.genesis_key {
            Ok(None)
        } else {
            self.get_hash(key).map(|hash| {
                self.parent_node(hash)
                    .map(|node| node.value.key)
                    // parent_node is None if we're provided the dag_root; hence parent = genesis key
                    .or(Some(self.genesis_key))
            })
        }
    }

    /// Returns the ancestors of the provided key. List is empty if we're provided the `genesis_key`
    /// Can return `Error::KeyNotFound`
    pub fn get_ancestors(&self, key: &bls::PublicKey) -> Result<Vec<bls::PublicKey>> {
        let mut ancestors = Vec::new();
        let mut key = *key;
        // loop until we reach key = genesis_key, then get_parent_key = None
        while let Some(parent) = self.get_parent_key(&key)? {
            ancestors.push(parent);
            key = parent;
        }
        Ok(ancestors)
    }

    /// Returns the immediate children of a given section key. List is empty if the section
    /// is a leaf. Can return `Error::KeyNotFound`
    pub fn get_child_keys(&self, key: &bls::PublicKey) -> Result<Vec<bls::PublicKey>> {
        if *key == self.genesis_key {
            Ok(self.dag_root.iter().cloned().collect())
        } else {
            self.get_hash(key).map(|hash| {
                self.child_nodes(hash)
                    .iter()
                    .map(|node| node.value.key)
                    .collect()
            })
        }
    }

    /// Returns the list of the leaf sections. A section is considered a leaf if it has not
    /// gone through any churn or split. Empty list if we only hold the genesis.
    fn non_genesis_leaf_sections(&self) -> Vec<SectionInfo> {
        self.dag.read().values().cloned().collect()
    }

    /// Returns the parent node of the given hash; There can be only one parent;
    /// If hash of dag_root is provided, parent = genesis, hence parent_node is absent
    fn parent_node(&self, hash: Hash) -> Option<Node<SectionInfo>> {
        // .children() means the actual parent; max 1 parent
        self.dag.children(hash).nodes().next().cloned()
    }

    // Returns the child node of the given hash; Since a section can only split into 2 section,
    // children can be max 2. List is empty if the provided Hash belongs to a leaf section.
    fn child_nodes(&self, hash: Hash) -> Vec<Node<SectionInfo>> {
        // .parent() means the actual children
        self.dag.parents(hash).nodes().cloned().collect()
    }

    fn get_node(&self, key: &bls::PublicKey) -> Result<Node<SectionInfo>> {
        let hash = self.get_hash(key)?;
        match self.dag.node(hash) {
            Some(node) => Ok(node.clone()),
            None => {
                error!("SectionsDAG::get_node() unreachable. Can be error only if hashes go out of sync");
                Err(Error::KeyNotFound(*key))
            }
        }
    }

    fn get_hash(&self, key: &bls::PublicKey) -> Result<Hash> {
        let hash = *self.hashes.get(key).ok_or(Error::KeyNotFound(*key))?;
        Ok(hash)
    }

    fn verify_sig(
        &self,
        parent_key: &bls::PublicKey,
        section_key: &bls::PublicKey,
        sig: &bls::Signature,
    ) -> bool {
        bincode::serialize(section_key)
            .map(|bytes| parent_key.verify(sig, bytes))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::{SectionInfo, SectionsDAG};
    use crdts::CmRDT;
    use eyre::Result;
    use proptest::prelude::{any, proptest, ProptestConfig, Strategy};
    use rand::rngs::SmallRng;
    use rand::{distributions::Standard, Rng, SeedableRng};
    use std::collections::{BTreeMap, BTreeSet};
    use std::fmt;

    #[test]
    fn insert_last() -> Result<()> {
        let (mut last_sk, pk) = gen_keypair(None);
        let mut dag = SectionsDAG::new(pk);
        let mut expected_keys = vec![pk];

        for _ in 0..10 {
            let last_pk = &expected_keys[expected_keys.len() - 1];
            let (sk, info) = gen_signed_keypair(None, &last_sk);

            dag.insert(last_pk, info.key, info.sig)?;

            expected_keys.push(info.key);
            last_sk = sk;
        }
        assert_lists(dag.keys(), &expected_keys);
        Ok(())
    }

    #[test]
    fn insert_fork() -> Result<()> {
        // We use a DAG with two branches, a and b:
        //  gen -> pk_a1 -> pk_a2
        //     |
        //     +-> pk_b
        //
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (sk_a1, info_a1) = gen_signed_keypair(None, &sk_gen);
        let (_, info_a2) = gen_signed_keypair(None, &sk_a1);
        let (_, info_b) = gen_signed_keypair(None, &sk_gen);

        let mut dag = SectionsDAG::new(pk_gen);
        dag.insert(&pk_gen, info_a1.key, info_a1.sig)?;
        dag.insert(&info_a1.key, info_a2.key, info_a2.sig)?;
        dag.insert(&pk_gen, info_b.key, info_b.sig)?;

        assert_lists(
            dag.keys(),
            &vec![pk_gen, info_a1.key, info_a2.key, info_b.key],
        );

        // cannot get partial dag till genesis
        assert!(dag.partial_dag(&pk_gen, &pk_gen).is_err());
        assert_lists(
            dag.partial_dag(&pk_gen, &info_a2.key)?.keys(),
            &vec![pk_gen, info_a1.key, info_a2.key],
        );
        assert_lists(
            dag.partial_dag(&pk_gen, &info_b.key)?.keys(),
            &vec![pk_gen, info_b.key],
        );

        assert!(dag.partial_dag(&info_a2.key, &pk_gen).is_err());
        assert!(dag.partial_dag(&info_a1.key, &info_b.key).is_err());
        assert!(dag.self_verify());
        Ok(())
    }

    #[test]
    fn insert_duplicate_key() -> Result<()> {
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (_, info_a) = gen_signed_keypair(None, &sk_gen);

        let mut dag = SectionsDAG::new(pk_gen);
        assert!(dag.insert(&pk_gen, info_a.key, info_a.sig.clone()).is_ok());
        assert!(dag.insert(&pk_gen, info_a.key, info_a.sig).is_ok());
        assert_lists(dag.keys(), &vec![info_a.key, pk_gen]);

        Ok(())
    }

    #[test]
    fn invalid_signature() {
        let (_, pk_gen) = gen_keypair(None);
        let bad_sk_gen = bls::SecretKey::random();
        let (_, info_a) = gen_signed_keypair(None, &bad_sk_gen);

        let mut dag = SectionsDAG::new(pk_gen);
        assert!(dag.insert(&pk_gen, info_a.key, info_a.sig).is_err());
    }

    #[test]
    fn wrong_parent_child_order() -> Result<()> {
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (sk_a1, info_a1) = gen_signed_keypair(None, &sk_gen);
        let (_, info_a2) = gen_signed_keypair(None, &sk_a1);

        let mut dag = SectionsDAG::new(pk_gen);
        // inserting child before parent
        assert!(dag
            .insert(&info_a1.key, info_a2.key, info_a2.sig.clone())
            .is_err());
        assert_lists(dag.keys(), &vec![pk_gen]);

        dag.insert(&pk_gen, info_a1.key, info_a1.sig)?;
        dag.insert(&info_a1.key, info_a2.key, info_a2.sig)?;
        assert_lists(dag.keys(), &vec![pk_gen, info_a1.key, info_a2.key]);

        Ok(())
    }

    #[test]
    fn merge() -> Result<()> {
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (sk_a1, info_a1) = gen_signed_keypair(None, &sk_gen);
        let (sk_a2, info_a2) = gen_signed_keypair(None, &sk_a1);
        let (_, info_a3) = gen_signed_keypair(None, &sk_a2);

        // main_dag: 0->1->2->3
        let mut main_dag = SectionsDAG::new(pk_gen);
        main_dag.insert(&pk_gen, info_a1.key, info_a1.sig)?;
        let mut dag_01 = main_dag.clone();
        let mut dag_01_err = main_dag.clone();
        main_dag.insert(&info_a1.key, info_a2.key, info_a2.sig)?;
        let mut dag_012 = main_dag.clone();
        main_dag.insert(&info_a2.key, info_a3.key, info_a3.sig)?;
        let mut dag_0123 = main_dag.clone();

        // main_dag: 0->1->2->3
        // partial_dag: 0
        // out: 0->1->2->3
        let partial_dag = SectionsDAG::new(pk_gen);
        main_dag.merge(partial_dag)?;
        assert!(main_dag.self_verify());
        assert_eq!(main_dag, dag_0123);

        // dag: 0->1->2
        // partial_dag: 2->3
        // out: 0->1->2->3
        let partial_dag = main_dag.partial_dag(&info_a2.key, &info_a3.key)?;
        dag_012.merge(partial_dag)?;
        assert!(dag_012.self_verify());
        assert_eq!(dag_012, main_dag);

        // dag: 0->1
        // partial_dag: 1->2->3
        // out: 0->1->2->3
        let partial_dag = main_dag.partial_dag(&info_a1.key, &info_a3.key)?;
        dag_01.merge(partial_dag)?;
        assert!(dag_01.self_verify());
        assert_eq!(dag_01, main_dag);

        // dag: 0->1->2->3
        // partial_dag: 0->1->2->3
        // out: 0->1->2->3
        let partial_dag = main_dag.partial_dag(&pk_gen, &info_a3.key)?;
        dag_0123.merge(partial_dag)?;
        assert!(dag_0123.self_verify());
        assert_eq!(dag_0123, main_dag);

        // dag: 0->1
        // partial_dag: 2->3
        // out: Error
        let partial_dag = main_dag.partial_dag(&info_a2.key, &info_a3.key)?;
        assert!(dag_01_err.merge(partial_dag).is_err());
        assert_lists(dag_01_err.keys(), &vec![pk_gen, info_a1.key]);

        Ok(())
    }

    #[test]
    fn merge_with_branches() -> Result<()> {
        // create a main DAG
        //  gen -> pk_a1 -> pk_a2
        //     |
        //     +-> pk_b1 -> pk_b2
        //              |
        //              +-> pk_c
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (sk_a1, info_a1) = gen_signed_keypair(None, &sk_gen);
        let (_, info_a2) = gen_signed_keypair(None, &sk_a1);
        let (sk_b1, info_b1) = gen_signed_keypair(None, &sk_gen);
        // pk_b1->pk_b2
        let (_, info_b2) = gen_signed_keypair(None, &sk_b1);
        // pk_b1->pk_c
        let (_, info_c) = gen_signed_keypair(None, &sk_b1);

        let mut main_dag = SectionsDAG::new(pk_gen);
        // gen->pk_a1->pk_a2
        main_dag.insert(&pk_gen, info_a1.key, info_a1.sig)?;
        main_dag.insert(&info_a1.key, info_a2.key, info_a2.sig)?;
        // gen->pk_b1->pk_b2
        main_dag.insert(&pk_gen, info_b1.key, info_b1.sig)?;
        main_dag.insert(&info_b1.key, info_b2.key, info_b2.sig)?;
        // pk_b1->pk_c
        main_dag.insert(&info_b1.key, info_c.key, info_c.sig)?;

        let mut dag = SectionsDAG::new(pk_gen);
        // merge from gen till pk_c
        let partial_dag = main_dag.partial_dag(&pk_gen, &info_c.key)?;
        dag.merge(partial_dag)?;
        // merge from gen till pk_a2
        let partial_dag = main_dag.partial_dag(&pk_gen, &info_a2.key)?;
        dag.merge(partial_dag)?;
        // merge from gen till pk_b2
        let partial_dag = main_dag.partial_dag(&pk_gen, &info_b2.key)?;
        dag.merge(partial_dag)?;

        assert!(main_dag.self_verify());
        assert!(dag.self_verify());
        assert_eq!(main_dag, dag);
        Ok(())
    }

    #[test]
    fn merge_fork() -> Result<()> {
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (_, info_a) = gen_signed_keypair(None, &sk_gen);
        let (_, info_b) = gen_signed_keypair(None, &sk_gen);

        // dag_a: gen->pk_a
        // dag_b: gen->pk_b
        // dag_a + dag_b: gen -> pk_a
        //                   |
        //                   +-> pk_b
        let mut dag_a = SectionsDAG::new(pk_gen);
        dag_a.insert(&pk_gen, info_a.key, info_a.sig)?;
        let mut dag_b = SectionsDAG::new(pk_gen);
        dag_b.insert(&pk_gen, info_b.key, info_b.sig)?;

        let dag_from_a = dag_a.partial_dag(&pk_gen, &info_a.key)?;
        let dag_from_b = dag_b.partial_dag(&pk_gen, &info_b.key)?;

        dag_a.merge(dag_from_b)?;
        dag_b.merge(dag_from_a)?;

        assert!(dag_a.self_verify());
        assert!(dag_b.self_verify());
        assert_eq!(dag_a, dag_b);
        Ok(())
    }

    #[test]
    fn self_verify_invalid_sigs() -> Result<()> {
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (sk1, info1) = gen_signed_keypair(None, &sk_gen);
        let (_, info2) = gen_signed_keypair(None, &sk1);
        let (sk3, mut info3) = gen_signed_keypair(None, &sk1);
        let (_, info4) = gen_signed_keypair(None, &sk3);

        // make a DAG with a fork but an invalid signature in section 3
        // gen->1->2
        //    |
        //    +->3->4
        let mut dag = SectionsDAG::new(pk_gen);
        dag.insert(&pk_gen, info1.key, info1.sig.clone())?;
        dag.insert(&info1.key, info2.key, info2.sig)?;
        // insert section3 manually with corrupted sig
        info3.sig = info1.sig; // use a random section's sig
        let hash1 = dag.get_hash(&info1.key)?;
        let node3_parent = BTreeSet::from([hash1]);
        let node3 = dag.dag.write(info3.clone(), node3_parent);
        dag.hashes.insert(info3.key, node3.hash());
        dag.dag.apply(node3);
        // continue inserting section 4
        dag.insert(&info3.key, info4.key, info4.sig)?;

        assert!(!dag.self_verify());
        Ok(())
    }

    #[test]
    fn verify_parent_during_split() -> Result<()> {
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (_, info_a) = gen_signed_keypair(None, &sk_gen);
        let (_, info_b) = gen_signed_keypair(None, &sk_gen);

        let mut dag = SectionsDAG::new(pk_gen);
        dag.insert(&pk_gen, info_a.key, info_a.sig)?;
        dag.insert(&pk_gen, info_b.key, info_b.sig)?;

        assert_eq!(
            dag.get_parent_key(&info_a.key)?,
            dag.get_parent_key(&info_b.key)?
        );
        Ok(())
    }

    #[test]
    fn verify_parents_during_churn() -> Result<()> {
        // gen -> pk_a1 -> pk_a2
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (sk_a1, info_a1) = gen_signed_keypair(None, &sk_gen);
        let (_, info_a2) = gen_signed_keypair(None, &sk_a1);

        let mut dag = SectionsDAG::new(pk_gen);
        dag.insert(&pk_gen, info_a1.key, info_a1.sig)?;
        dag.insert(&info_a1.key, info_a2.key, info_a2.sig)?;

        assert!(matches!(dag.get_parent_key(&info_a2.key)?, Some(key) if key == info_a1.key));
        assert!(matches!(dag.get_parent_key(&info_a1.key)?, Some(key) if key == pk_gen));
        assert!(matches!(dag.get_parent_key(&pk_gen)?, None));
        Ok(())
    }

    #[test]
    fn verify_children() -> Result<()> {
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (_, info_a) = gen_signed_keypair(None, &sk_gen);
        let (sk_b1, info_b1) = gen_signed_keypair(None, &sk_gen);
        let (_, info_b2) = gen_signed_keypair(None, &sk_b1);

        let mut dag = SectionsDAG::new(pk_gen);
        dag.insert(&pk_gen, info_a.key, info_a.sig)?;
        dag.insert(&pk_gen, info_b1.key, info_b1.sig)?;
        dag.insert(&info_b1.key, info_b2.key, info_b2.sig)?;

        assert_lists(dag.get_child_keys(&pk_gen)?, vec![info_a.key, info_b1.key]);
        assert_lists(dag.get_child_keys(&info_b1.key)?, vec![info_b2.key]);
        assert!(dag.get_child_keys(&info_a.key)?.is_empty());
        Ok(())
    }

    #[test]
    fn verify_leaves() -> Result<()> {
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (_, info_a) = gen_signed_keypair(None, &sk_gen);
        let (sk_b1, info_b1) = gen_signed_keypair(None, &sk_gen);
        let (_, info_b2) = gen_signed_keypair(None, &sk_b1);

        let mut sections_dag = SectionsDAG::new(pk_gen);
        sections_dag.insert(&pk_gen, info_a.key, info_a.sig)?;
        sections_dag.insert(&pk_gen, info_b1.key, info_b1.sig)?;
        sections_dag.insert(&info_b1.key, info_b2.key, info_b2.sig)?;

        assert_lists(sections_dag.leaf_keys(), vec![info_a.key, info_b2.key]);
        Ok(())
    }

    #[test]
    fn verify_ancestors() -> Result<()> {
        // gen -> pk_a1 -> pk_a2 -> pk_a3
        let (sk_gen, pk_gen) = gen_keypair(None);
        let (sk_a1, info_a1) = gen_signed_keypair(None, &sk_gen);
        let (sk_a2, info_a2) = gen_signed_keypair(None, &sk_a1);
        let (_, info_a3) = gen_signed_keypair(None, &sk_a2);

        let mut dag = SectionsDAG::new(pk_gen);
        dag.insert(&pk_gen, info_a1.key, info_a1.sig)?;
        dag.insert(&info_a1.key, info_a2.key, info_a2.sig)?;
        dag.insert(&info_a2.key, info_a3.key, info_a3.sig)?;

        assert_lists(
            dag.get_ancestors(&info_a3.key)?,
            vec![pk_gen, info_a1.key, info_a2.key],
        );
        assert_lists(dag.get_ancestors(&info_a2.key)?, vec![pk_gen, info_a1.key]);
        assert_lists(dag.get_ancestors(&info_a1.key)?, vec![pk_gen]);
        let empty: Vec<bls::PublicKey> = Vec::new();
        assert_lists(dag.get_ancestors(&pk_gen)?, empty);

        Ok(())
    }

    // Proptest to make sure that the using `merge` with various combinations of partial dags will give
    // back the original `SectionsDAG`
    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 20, .. ProptestConfig::default()
        })]
        #[test]
        fn proptest_merge_sections_dag((main_dag, update_variations_list) in arb_sections_dag_and_partial_dags(100, 5)) {
            for variation in update_variations_list {
                let mut dag = SectionsDAG::new(main_dag.genesis_key);
                for partial_dag in variation {
                    dag.merge(partial_dag).unwrap();
                }
                assert_eq!(dag, main_dag);
            }
        }
    }

    // Test helpers
    fn assert_lists<I, J>(a: I, b: J)
    where
        I: IntoIterator,
        J: IntoIterator,
        I::Item: fmt::Debug + PartialEq<J::Item> + Eq,
        J::Item: fmt::Debug + PartialEq<I::Item> + Eq,
    {
        let vec1: Vec<_> = a.into_iter().collect();
        let mut vec2: Vec<_> = b.into_iter().collect();
        assert_eq!(vec1.len(), vec2.len());
        for item1 in &vec1 {
            let idx2 = vec2.iter().position(|item2| *item2 == *item1);
            assert!(idx2.is_some());
            vec2.remove(idx2.unwrap());
        }
        assert_eq!(vec2.len(), 0);
    }

    fn gen_keypair(rng: Option<&mut SmallRng>) -> (bls::SecretKey, bls::PublicKey) {
        let sk: bls::SecretKey = match rng {
            Some(rng) => rng.sample(Standard),
            None => bls::SecretKey::random(),
        };

        (sk.clone(), sk.public_key())
    }

    fn gen_signed_keypair(
        rng: Option<&mut SmallRng>,
        parent_sk: &bls::SecretKey,
    ) -> (bls::SecretKey, SectionInfo) {
        let (sk, pk) = gen_keypair(rng);
        let sig = sign(parent_sk, &pk);
        let info = SectionInfo { key: pk, sig };
        (sk, info)
    }

    fn sign(signing_sk: &bls::SecretKey, pk_to_sign: &bls::PublicKey) -> bls::Signature {
        bincode::serialize(pk_to_sign)
            .map(|bytes| signing_sk.sign(&bytes))
            .expect("failed to serialize public key")
    }

    // Generate an arbitrary sized `SectionsDAG` and a List<list of `SectionsDAG` which inserted in
    // that order gives back the main_dag>; i.e., multiple variations of <list of SectionsDAG>
    fn arb_sections_dag_and_partial_dags(
        max_sections: usize,
        update_variations: usize,
    ) -> impl Strategy<Value = (SectionsDAG, Vec<Vec<SectionsDAG>>)> {
        (any::<u64>(), 2..=max_sections).prop_map(move |(seed, size)| {
            // size is [2, max_sections] size of 0,1 will give us back only the genesis_key and
            // hence we cannot obtain the partial dag
            let main_dag = gen_random_sections_dag(Some(seed as u64), size).unwrap();
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut update_variations_list = Vec::new();

            for _ in 0..update_variations {
                let mut leaves: Vec<_> = main_dag.leaf_keys().into_iter().collect();
                let mut inserted_keys = BTreeSet::from([main_dag.genesis_key]);

                let mut list_of_part_dags = Vec::new();
                while !leaves.is_empty() {
                    // get partial dag to a random leaf
                    let random_leaf = *leaves
                        .get(rng.gen::<usize>() % leaves.len())
                        .expect("leaves cannot be empty");

                    // a simple chain of keys containing (`random_leaf`, `genesis_key`]
                    let mut ancestors = main_dag.get_ancestors(&random_leaf).unwrap();
                    ancestors.reverse();
                    ancestors.push(random_leaf);

                    // find the position of the first node which we have not inserted
                    let first_unique_node_idx = ancestors
                        .iter()
                        .position(|key| !inserted_keys.contains(key))
                        .expect("The leaf should not have been inserted, hence cannot be None");

                    let rand_from_idx = if first_unique_node_idx != 0 {
                        rng.gen_range(0..first_unique_node_idx)
                    } else {
                        0
                    };
                    let rand_to_idx = rng.gen_range(rand_from_idx + 1..ancestors.len());

                    let rand_from = ancestors[rand_from_idx];
                    let rand_to = ancestors[rand_to_idx];
                    let partial_dag = main_dag.partial_dag(&rand_from, &rand_to).unwrap();

                    // if the "to" key is a leaf, remove it from leaves
                    if let Some(index) = leaves.iter().position(|key| *key == rand_to) {
                        leaves.swap_remove(index);
                    }
                    // update the inserted list
                    partial_dag.keys().for_each(|key| {
                        inserted_keys.insert(*key);
                    });

                    list_of_part_dags.push(partial_dag);
                }
                update_variations_list.push(list_of_part_dags);
            }
            (main_dag, update_variations_list)
        })
    }

    fn gen_random_sections_dag(seed: Option<u64>, n_sections: usize) -> Result<SectionsDAG> {
        let mut rng = match seed {
            Some(seed) => SmallRng::seed_from_u64(seed),
            None => SmallRng::from_entropy(),
        };

        let (sk_gen, pk_gen) = gen_keypair(Some(&mut rng));
        let mut dag = SectionsDAG::new(pk_gen);
        let mut sk_map: BTreeMap<bls::PublicKey, bls::SecretKey> = BTreeMap::new();
        sk_map.insert(pk_gen, sk_gen);
        let mut count = 1;

        if n_sections <= 1 {
            return Ok(dag);
        };
        while count < n_sections {
            let leaves: Vec<_> = dag.leaf_keys().into_iter().collect();
            let pk_random_leaf = leaves.get(rng.gen::<usize>() % leaves.len()).unwrap();
            let sk_random_leaf = sk_map.get(pk_random_leaf).unwrap().clone();

            let (sk1, info1) = gen_signed_keypair(Some(&mut rng), &sk_random_leaf);
            dag.insert(pk_random_leaf, info1.key, info1.sig)?;
            sk_map.insert(info1.key, sk1);
            if rng.gen_range(0..2) % 2 == 0 {
                // Split, insert extra one
                let (sk2, info2) = gen_signed_keypair(Some(&mut rng), &sk_random_leaf);
                dag.insert(pk_random_leaf, info2.key, info2.sig)?;
                sk_map.insert(info2.key, sk2);
                count += 2;
            } else {
                // Churn
                count += 1;
            }
        }
        Ok(dag)
    }
}
