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

/// Unversioned-Structured Data
pub mod unversioned;
/// Versioned-Structured Data
pub mod versioned;

use xor_name::XorName;
use core::errors::CoreError;
use routing::StructuredData;
use sodiumoxide::crypto::sign;
use maidsafe_utilities::serialisation::serialise;

const PADDING_SIZE_IN_BYTES: usize = 1024;
const MIN_RESIDUAL_SPACE_FOR_VALID_STRUCTURED_DATA_IN_BYTES: usize = 70;

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
pub fn get_approximate_space_for_data(owner_keys: Vec<sign::PublicKey>,
                                      prev_owner_keys: Vec<sign::PublicKey>)
                                      -> Result<usize, CoreError> {
    let max_signatures_possible = if prev_owner_keys.is_empty() {
        owner_keys.len()
    } else {
        prev_owner_keys.len()
    };

    let mut structured_data = try!(StructuredData::new(::std::u64::MAX,
                                                       XorName::new([::std::u8::MAX; 64]),
                                                       ::std::u64::MAX,
                                                       Vec::new(),
                                                       owner_keys,
                                                       prev_owner_keys,
                                                       None));

    // Fill it with rest of signatures
    structured_data.replace_signatures(vec![sign::Signature([::std::u8::MAX; sign::SIGNATUREBYTES]);
                                            max_signatures_possible]);

    let serialised_structured_data_len = try!(serialise(&structured_data)).len() +
                                         PADDING_SIZE_IN_BYTES;
    if ::routing::MAX_STRUCTURED_DATA_SIZE_IN_BYTES <= serialised_structured_data_len {
        Ok(0)
    } else {
        Ok(::routing::MAX_STRUCTURED_DATA_SIZE_IN_BYTES - serialised_structured_data_len)
    }
}

/// Check if it is possible to fit the given data into the given StructuredData
pub fn check_if_data_can_fit_in_structured_data(data: &Vec<u8>,
                                                owner_keys: Vec<sign::PublicKey>,
                                                prev_owner_keys: Vec<sign::PublicKey>)
                                                -> Result<DataFitResult, CoreError> {
    if data.len() > ::routing::MAX_STRUCTURED_DATA_SIZE_IN_BYTES - PADDING_SIZE_IN_BYTES {
        Ok(DataFitResult::DataDoesNotFit)
    } else {
        let available_size = try!(get_approximate_space_for_data(owner_keys, prev_owner_keys));
        if available_size <= MIN_RESIDUAL_SPACE_FOR_VALID_STRUCTURED_DATA_IN_BYTES {
            Ok(DataFitResult::NoDataCanFit)
        } else if available_size < data.len() {
            Ok(DataFitResult::DataDoesNotFit)
        } else {
            Ok(DataFitResult::DataFits)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::utility::test_utils;

    // Refers the fixed size of the test_get_approximate_space_for_data fn without signatures
    const DEFAULT_FIXED_SIZE: usize = ::routing::MAX_STRUCTURED_DATA_SIZE_IN_BYTES - 1276;
    // 196 is approximate size (close enough) of a Fixed Key after serialisation.
    const FIXED_SIZE_OF_KEY: usize = 196;

    #[test]
    #[ignore]
    fn approximate_space_for_data() {
        // Assertion based on Fixed Key sizes
        {
            let mut keys = test_utils::get_max_sized_public_keys(1);
            assert_eq!(unwrap_result!(get_approximate_space_for_data(keys.clone(), Vec::new())),
                       DEFAULT_FIXED_SIZE - FIXED_SIZE_OF_KEY);
            keys.extend(test_utils::get_max_sized_public_keys(1));
            assert_eq!(unwrap_result!(get_approximate_space_for_data(keys.clone(), Vec::new())),
                       DEFAULT_FIXED_SIZE - (FIXED_SIZE_OF_KEY * keys.len()));
            keys.extend(test_utils::get_max_sized_public_keys(513)); // 515 keys Max
            assert!(unwrap_result!(get_approximate_space_for_data(keys.clone(), Vec::new())) <
                    FIXED_SIZE_OF_KEY);
            keys.extend(test_utils::get_max_sized_public_keys(1));
            assert!(unwrap_result!(get_approximate_space_for_data(keys.clone(), Vec::new())) == 0);
        }
        // Random key assertions
        {
            let mut keys = test_utils::generate_public_keys(10);
            assert!(unwrap_result!(get_approximate_space_for_data(keys.clone(), Vec::new())) >
                    5000);
            assert!(unwrap_result!(get_approximate_space_for_data(test_utils::generate_public_keys(1), keys.clone())) > 5000);
            keys.extend(test_utils::generate_public_keys(40)); // 50 keys
            assert!(unwrap_result!(get_approximate_space_for_data(keys.clone(), Vec::new())) >
                    5000);
            assert!(unwrap_result!(get_approximate_space_for_data(test_utils::generate_public_keys(1), keys.clone())) > 5000);
            keys.extend(test_utils::generate_public_keys(470)); // 520 keys
            assert!(unwrap_result!(get_approximate_space_for_data(keys.clone(), Vec::new())) > 100);
            assert!(unwrap_result!(get_approximate_space_for_data(test_utils::generate_public_keys(1), keys.clone())) > 100);
        }
    }

    #[test]
    #[ignore]
    fn data_can_fit_in_structured_data() {
        // Assertion based on Fixed Key sizes
        // Maximum of 516 keys can be accomodated after serialisation. Thus the fixed key tests work
        // on that calculation
        {
            let mut keys = test_utils::get_max_sized_public_keys(1);
            assert_eq!(DataFitResult::DataFits,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&Vec::with_capacity(0),
                                                                               keys.clone(),
                                                                               Vec::new())));
            assert_eq!(DataFitResult::DataDoesNotFit,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&vec![1u8; 102400],
                                                                               keys.clone(),
                                                                               Vec::new())));
            assert_eq!(DataFitResult::DataDoesNotFit,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&vec![1u8; 103424],
                                                                               keys.clone(),
                                                                               Vec::new())));
            keys.extend(test_utils::get_max_sized_public_keys(514));
            assert_eq!(DataFitResult::DataFits,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&Vec::with_capacity(0),
                                                                               keys.clone(),
                                                                               Vec::new())));
            assert_eq!(DataFitResult::DataDoesNotFit,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&vec![0u8; 102400],
                                                                               keys.clone(),
                                                                               Vec::new())));
            keys.extend(test_utils::get_max_sized_public_keys(1));
            assert_eq!(DataFitResult::NoDataCanFit,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&Vec::with_capacity(0),
                                                                               keys.clone(),
                                                                               Vec::new())));
        }
        // Empty data
        {
            let mut keys = test_utils::generate_public_keys(250);
            assert_eq!(DataFitResult::DataFits,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&Vec::new(),
                                                                               keys.clone(),
                                                                               Vec::new())));
            assert_eq!(DataFitResult::DataFits,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&Vec::new(),
                                                                               test_utils::generate_public_keys(1),
                                                                               keys.clone())));
            keys.extend(test_utils::generate_public_keys(350));
            assert_eq!(DataFitResult::NoDataCanFit,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&Vec::new(),
                                                                               keys,
                                                                               Vec::new())));
        }
        // Data of size 80kb
        {
            let data = vec![99u8; 1024 * 80];
            let mut keys = test_utils::generate_public_keys(1);
            assert_eq!(DataFitResult::DataFits,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&data,
                                                                               keys.clone(),
                                                                               Vec::new())));
            keys.extend(test_utils::generate_public_keys(98));
            assert_eq!(DataFitResult::DataFits,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&data,
                                                                               keys.clone(),
                                                                               Vec::new())));
            keys.extend(test_utils::generate_public_keys(190));
            assert_eq!(DataFitResult::DataDoesNotFit,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&data,
                                                                               keys.clone(),
                                                                               Vec::new())));
            keys.extend(test_utils::generate_public_keys(225));
            assert_eq!(DataFitResult::DataDoesNotFit,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&data,
                                                                               keys.clone(),
                                                                               Vec::new())));
            keys.extend(test_utils::generate_public_keys(15));
            assert_eq!(DataFitResult::NoDataCanFit,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&data,
                                                                               keys,
                                                                               Vec::new())));
        }
        // Data size of 100 kb
        {
            let data = vec![1u8; 102400];
            assert_eq!(DataFitResult::DataDoesNotFit,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&data,
                                                                               test_utils::generate_public_keys(1),
                                                                               Vec::new())));
        }
        // Data size of 101 kb
        {
            let data = vec![1u8; 103424];
            assert_eq!(DataFitResult::DataDoesNotFit,
                       unwrap_result!(check_if_data_can_fit_in_structured_data(&data,
                                                                               test_utils::generate_public_keys(1),
                                                                               Vec::new())));
        }
    }
}
