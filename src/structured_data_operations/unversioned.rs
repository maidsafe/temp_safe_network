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
pub fn create<T>(storage: ::std::sync::Arc<T>,
                 tag_type: u64,
                 id: ::routing::NameType,
                 version: u64,
                 data: Vec<u8>,
                 owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
                 prev_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
                 private_signing_key: &::sodiumoxide::crypto::sign::SecretKey,
                 data_encryption_keys: Option<(&::sodiumoxide::crypto::box_::PublicKey,
                                               &::sodiumoxide::crypto::box_::SecretKey,
                                               &::sodiumoxide::crypto::box_::Nonce)>) -> Result<::client::StructuredData, ::errors::ClientError>
                                                                                         where T: ::self_encryption::Storage + Sync + Send + 'static {
    let data_to_store = try!(get_data_to_store_in_structured_data(CONTAINS_DATA, &data, data_encryption_keys));

    match ::structured_data_operations::check_if_data_can_fit_in_structured_data(data_to_store.clone(), owner_keys.clone(), prev_owner_keys.clone()) {
        ::structured_data_operations::DataFitResult::DataFits => {
            Ok(::client::StructuredData::new(tag_type,
                                             id,
                                             version,
                                             data_to_store,
                                             owner_keys,
                                             prev_owner_keys,
                                             private_signing_key))

        },
        ::structured_data_operations::DataFitResult::DataDoesNotFit => {
            let mut se = ::self_encryption::SelfEncryptor::new(storage.clone(), ::self_encryption::datamap::DataMap::None);
            se.write(&data, 0);
            let data_map = se.close();

            let data_to_store = try!(get_data_to_store_in_structured_data(CONTAINS_DATA_MAP, &data_map, data_encryption_keys));
            match ::structured_data_operations::check_if_data_can_fit_in_structured_data(data_to_store.clone(), owner_keys.clone(), prev_owner_keys.clone()) {
                ::structured_data_operations::DataFitResult::DataFits => {
                    Ok(::client::StructuredData::new(tag_type,
                                                     id,
                                                     version,
                                                     data_to_store,
                                                     owner_keys,
                                                     prev_owner_keys,
                                                     private_signing_key))

                },
                ::structured_data_operations::DataFitResult::DataDoesNotFit => {
                    // TODO Improve this - will require changes elsewhere - eg., implement storage
                    // trait in client itself

                    let raw_data_map = if let Some(encrypted_vec) = try!(encrypt_data_if_needed(&data_map, data_encryption_keys)) {
                        encrypted_vec
                    } else {
                        ::utility::serialise(data_map)
                    };

                    let immutable_data = ::client::ImmutableData::new(::client::ImmutableDataType::Normal, raw_data_map.clone());
                    storage.put(Vec::new(), raw_data_map); // TODO improve 1st parameter - use new function to take Data as parameter

                    let data_to_store = try!(get_data_to_store_in_structured_data(CONTAINS_DATA_MAP_NAME, &immutable_data.name(), None));

                    Ok(::client::StructuredData::new(tag_type,
                                                     id,
                                                     version,
                                                     data_to_store,
                                                     owner_keys,
                                                     prev_owner_keys,
                                                     private_signing_key))
                },
                ::structured_data_operations::DataFitResult::NoDataCanFit => Err(::errors::ClientError::StructuredDataHeaderSizeProhibitive),
            }
        },
        ::structured_data_operations::DataFitResult::NoDataCanFit => Err(::errors::ClientError::StructuredDataHeaderSizeProhibitive),
    }
}

fn get_data_to_store_in_structured_data<T>(data_encoding: u8,
                                           data: &T,
                                           data_encryption_keys: Option<(&::sodiumoxide::crypto::box_::PublicKey,
                                                                         &::sodiumoxide::crypto::box_::SecretKey,
                                                                         &::sodiumoxide::crypto::box_::Nonce)>) -> Result<Vec<u8>, ::errors::ClientError>
                                                                                                                   where T: ::rustc_serialize::Encodable {
    let mut encoder = ::cbor::Encoder::from_memory();
    if let Some(encrypted_vec) = try!(encrypt_data_if_needed(data, data_encryption_keys)) {
        try!(encoder.encode(&[(data_encoding, encrypted_vec)]));
    } else {
        try!(encoder.encode(&[(data_encoding, data)]));
    }
    Ok(encoder.into_bytes())
}

fn encrypt_data_if_needed<T>(data: &T,
                             data_encryption_keys: Option<(&::sodiumoxide::crypto::box_::PublicKey,
                                                           &::sodiumoxide::crypto::box_::SecretKey,
                                                           &::sodiumoxide::crypto::box_::Nonce)>) -> Result<Option<Vec<u8>>, ::errors::ClientError>
                                                                                                     where T: ::rustc_serialize::Encodable {
    if let Some((ref public_encryp_key, ref secret_encryp_key, ref nonce)) = data_encryption_keys {
        let mut encoder = ::cbor::Encoder::from_memory();
        try!(encoder.encode(&[data])); // TODO utilise ::utility::serialise() once return type is corrected there
        Ok(Some(try!(::utility::hybrid_encrypt(&encoder.into_bytes()[..], nonce, public_encryp_key, secret_encryp_key))))
    } else {
        Ok(None)
    }
}
