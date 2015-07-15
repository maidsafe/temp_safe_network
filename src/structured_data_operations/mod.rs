// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

/// NetworkStorage implements the Storage trait from the Self_Encryption
pub mod self_encryption_storage;
/// Unversioned-Structured Data
pub mod unversioned;
/// Versioned-Structured Data
pub mod versioned;

pub use self::self_encryption_storage::SelfEncryptionStorage;

const PADDING_SIZE_IN_BYTES: usize = 1024;
const MIN_RESIDUAL_SPACE_FOR_VALID_STRUCTURED_DATA_IN_BYTES: usize = 64;

/// Inform about data fitting or not into given StructuredData
#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum DataFitResult {
    /// Invalid StrucuturedData.
    NoDataCanFit,
    /// Given data is too large to fit into the given StructuredData
    DataDoesNotFit,
    /// Given data fits into the given StructuredData
    DataFits,
}

/// Calculates approximate space available for data. Calculates the worst case scenario in which
/// all owners must sign this StructuredData.
pub fn get_approximate_space_for_data(owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
                                      prev_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>) -> usize {
    let max_signatures_possible =  if prev_owner_keys.is_empty() {
        owner_keys.len()
   } else {
       prev_owner_keys.len()
   };

    let (_, fake_signer) = ::sodiumoxide::crypto::sign::gen_keypair();
    let mut structured_data = ::client::StructuredData::new(::std::u64::MAX,
                                                            ::routing::NameType::new([0; 64]),
                                                            ::std::u64::MAX,
                                                            Vec::new(),
                                                            owner_keys,
                                                            prev_owner_keys,
                                                            &fake_signer);

    // Fill it with rest of signatures
    for _ in 1..max_signatures_possible {
        structured_data.add_signature(&fake_signer);
    }

    let serialised_structured_data_len = ::utility::serialise(structured_data).len() + PADDING_SIZE_IN_BYTES;

    if ::client::MAX_STRUCTURED_DATA_SIZE_IN_BYTES <= serialised_structured_data_len {
        0
    } else {
        ::client::MAX_STRUCTURED_DATA_SIZE_IN_BYTES - serialised_structured_data_len
    }
}

/// Check if it is possible to fit the given data into the given StructuredData
pub fn check_if_data_can_fit_in_structured_data(data: Vec<u8>,
                                                owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
                                                prev_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>) -> DataFitResult {
    if data.len() > ::client::MAX_STRUCTURED_DATA_SIZE_IN_BYTES - PADDING_SIZE_IN_BYTES {
        DataFitResult::DataDoesNotFit
    } else {
        let available_size = get_approximate_space_for_data(owner_keys, prev_owner_keys);
        if available_size <= MIN_RESIDUAL_SPACE_FOR_VALID_STRUCTURED_DATA_IN_BYTES {
            DataFitResult::NoDataCanFit
        } else if available_size < data.len() {
            DataFitResult::DataDoesNotFit
        } else {
            DataFitResult::DataFits
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn genearte_public_keys(size: usize) -> Vec<::sodiumoxide::crypto::sign::PublicKey> {
        let mut public_keys = Vec::with_capacity(size);
        for _ in 0..size {
            public_keys.push(::sodiumoxide::crypto::sign::gen_keypair().0);
        }
        public_keys
    }

    #[test]
    fn test_get_approximate_space_for_data() {
        let mut keys = genearte_public_keys(10);
        assert!(get_approximate_space_for_data(keys.clone(), Vec::new()) > 5000);
        assert!(get_approximate_space_for_data(Vec::new(), keys.clone()) > 5000);
        keys.extend(genearte_public_keys(40)); // 50 keys
        assert!(get_approximate_space_for_data(keys.clone(), Vec::new()) > 5000);
        assert!(get_approximate_space_for_data(genearte_public_keys(1), keys.clone()) > 5000);
        keys.extend(genearte_public_keys(480)); // 530 keys
        assert!(get_approximate_space_for_data(keys.clone(), Vec::new()) > 100);
        assert!(get_approximate_space_for_data(genearte_public_keys(1), keys.clone()) > 100);
    }

    #[test]
    fn test_check_if_data_can_fit_in_structured_data() {
        // Empty data
        {
            let mut keys = genearte_public_keys(250);
            assert_eq!(DataFitResult::DataFits, check_if_data_can_fit_in_structured_data(Vec::new(), keys.clone(), Vec::new()));
            assert_eq!(DataFitResult::DataFits, check_if_data_can_fit_in_structured_data(Vec::new(), genearte_public_keys(1), keys.clone()));
            keys.extend(genearte_public_keys(350));
            assert_eq!(DataFitResult::NoDataCanFit, check_if_data_can_fit_in_structured_data(Vec::new(), keys, Vec::new()));
        }
        // Data of size 80kb
        {
            let data = vec![1u8; 1024 * 80];
            let mut keys = genearte_public_keys(1);
            assert_eq!(DataFitResult::DataFits, check_if_data_can_fit_in_structured_data(data.clone(), keys.clone(), Vec::new()));
            keys.extend(genearte_public_keys(289));
            assert_eq!(DataFitResult::DataDoesNotFit, check_if_data_can_fit_in_structured_data(data.clone(), keys.clone(), Vec::new()));
            keys.extend(genearte_public_keys(248));
            assert_eq!(DataFitResult::DataDoesNotFit, check_if_data_can_fit_in_structured_data(data.clone(), keys.clone(), Vec::new()));
            keys.extend(genearte_public_keys(10));
            assert_eq!(DataFitResult::NoDataCanFit, check_if_data_can_fit_in_structured_data(data.clone(), keys, Vec::new()));
        }
        // Data size of 100 kb
        {
            let data = vec![1u8; 102400];
            assert_eq!(DataFitResult::DataDoesNotFit, check_if_data_can_fit_in_structured_data(data.clone(), genearte_public_keys(1), Vec::new()));
        }
        // Data size of 101 kb
        {
            let data = vec![1u8; 103424];
            assert_eq!(DataFitResult::DataDoesNotFit, check_if_data_can_fit_in_structured_data(data.clone(), genearte_public_keys(1), Vec::new()));
        }
    }

}
