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

const CONTAINS_DATA: u8          = 0;
const CONTAINS_DATA_MAP: u8      = 1;
const CONTAINS_DATA_MAP_NAME: u8 = 2;

/// Create StructuredData
pub fn create(tag_type: u64,
              id: ::routing::NameType,
              version: u64,
              data: Vec<u8>,
              owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
              prev_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
              private_signing_key: &::sodiumoxide::crypto::sign::SecretKey,
              data_encryption_keys: Option<(&::sodiumoxide::crypto::box_::PublicKey,
                                            &::sodiumoxide::crypto::box_::SecretKey,
                                            &::sodiumoxide::crypto::box_::Nonce)>) -> Result<::client::StructuredData, ::errors::ClientError> {
    let data_to_test = if let Some((ref public_encryp_key, ref secret_encryp_key, ref nonce)) = data_encryption_keys {
        let mut encoder = ::cbor::Encoder::from_memory();
        try!(encoder.encode(CONTAINS_DATA));
        try!(encoder.encode(try!(::utility::hybrid_encrypt(&data[..], nonce, public_encryp_key, secret_encryp_key))));
        encoder.into_bytes()
    } else {
        data.clone()
    };

    match ::structured_data_operations::check_if_data_can_fit_in_structured_data(data_to_test.clone(), owner_keys.clone(), prev_owner_keys.clone()) {
        ::structured_data_operations::DataFitResult::DataFits => {
            Ok(::client::StructuredData::new(tag_type,
                                             id,
                                             version,
                                             data_to_test,
                                             owner_keys,
                                             prev_owner_keys,
                                             private_signing_key))

        },
        ::structured_data_operations::DataFitResult::DataDoesNotFit => {
            unimplemented!();
        },
        ::structured_data_operations::DataFitResult::NoDataCanFit => Err(::errors::ClientError::StructuredDataHeaderSizeProhibitive),
    }
}
