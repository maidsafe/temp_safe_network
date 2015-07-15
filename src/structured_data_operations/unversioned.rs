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

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq)]
enum DataTypeEncoding {
    ContainsData(Vec<u8>),
    ContainsDataMap(::self_encryption::datamap::DataMap),
    ContainsDataMapName(::routing::NameType),
}

/// Create StructuredData in accordance with data-encoding rules abstracted from user. For
/// StructuredData created with create, data must be obtained using the complementary function
/// defined in this module to get_data()
pub fn create(client: ::std::sync::Arc<::std::sync::Mutex<::client::Client>>,
              tag_type: u64,
              id: ::routing::NameType,
              version: u64,
              data: Vec<u8>,
              owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
              prev_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
              private_signing_key: &::sodiumoxide::crypto::sign::SecretKey,
              data_encryption_keys: Option<(&::sodiumoxide::crypto::box_::PublicKey,
                                            &::sodiumoxide::crypto::box_::SecretKey,
                                            &::sodiumoxide::crypto::box_::Nonce)>) -> Result<::client::StructuredData, ::errors::ClientError> {
    let data_to_store = try!(get_encoded_data_to_store(DataTypeEncoding::ContainsData(data.clone()), data_encryption_keys));

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
            let mut se = ::self_encryption::SelfEncryptor::new(::structured_data_operations::SelfEncryptionStorage::new(client.clone()), ::self_encryption::datamap::DataMap::None);
            se.write(&data, 0);
            let data_map = se.close();

            let data_to_store = try!(get_encoded_data_to_store(DataTypeEncoding::ContainsDataMap(data_map.clone()), data_encryption_keys));
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

                    let immutable_data = ::client::ImmutableData::new(::client::ImmutableDataType::Normal, data_to_store);
                    let name = immutable_data.name();
                    let data = ::client::Data::ImmutableData(immutable_data);
                    let _ = client.lock().unwrap().put_new(name.clone(), data);

                    let data_to_store = try!(get_encoded_data_to_store(DataTypeEncoding::ContainsDataMapName(name), data_encryption_keys));

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

/// Get Actual Data From StructuredData created via create() function in this module.
pub fn get_data(client: ::std::sync::Arc<::std::sync::Mutex<::client::Client>>,
                   struct_data: &::client::StructuredData,
                   data_decryption_keys: Option<(&::sodiumoxide::crypto::box_::PublicKey,
                                                 &::sodiumoxide::crypto::box_::SecretKey,
                                                 &::sodiumoxide::crypto::box_::Nonce)>) -> Result<Vec<u8>, ::errors::ClientError> {
    match try!(get_decoded_stored_data(struct_data.get_data().clone(), data_decryption_keys)) {
        DataTypeEncoding::ContainsData(data) => Ok(data),
        DataTypeEncoding::ContainsDataMap(data_map) => {
            let mut se = ::self_encryption::SelfEncryptor::new(::structured_data_operations::SelfEncryptionStorage::new(client), data_map);
            let length = se.len();
            Ok(se.read(0, length))
        },
        DataTypeEncoding::ContainsDataMapName(data_map_name) => {
            match client.lock().unwrap().get_new(data_map_name, ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal)).unwrap().get() {
                // TODO This is wrong as feedback is to be Data not raw data. Wait for routing to
                // build and correct everywhere
                Ok(raw_data_map) => {
                    match try!(get_decoded_stored_data(raw_data_map, data_decryption_keys)) {
                        DataTypeEncoding::ContainsDataMap(data_map) => {
                            let mut se = ::self_encryption::SelfEncryptor::new(::structured_data_operations::SelfEncryptionStorage::new(client.clone()), data_map);
                            let length = se.len();
                            Ok(se.read(0, length))
                        },
                        _ => Err(::errors::ClientError::ReceivedUnexpectedData),
                    }
                },
                Err(_) => Err(::errors::ClientError::GetFailure),
            }
        }
    }
}

fn get_encoded_data_to_store(data: DataTypeEncoding,
                             data_encryption_keys: Option<(&::sodiumoxide::crypto::box_::PublicKey,
                                                           &::sodiumoxide::crypto::box_::SecretKey,
                                                           &::sodiumoxide::crypto::box_::Nonce)>) -> Result<Vec<u8>, ::errors::ClientError> {
    let mut encoder = ::cbor::Encoder::from_memory();
    try!(encoder.encode(&[data])); // TODO utilise ::utility::serialise() once return type is corrected there

    if let Some((ref public_encryp_key, ref secret_encryp_key, ref nonce)) = data_encryption_keys {
        Ok(try!(::utility::hybrid_encrypt(&encoder.into_bytes()[..], nonce, public_encryp_key, secret_encryp_key)))
    } else {
        Ok(encoder.into_bytes())
    }
}

fn get_decoded_stored_data(raw_data: Vec<u8>,
                           data_decryption_keys: Option<(&::sodiumoxide::crypto::box_::PublicKey,
                                                         &::sodiumoxide::crypto::box_::SecretKey,
                                                         &::sodiumoxide::crypto::box_::Nonce)>) -> Result<DataTypeEncoding, ::errors::ClientError> {
    let data = if let Some((ref public_encryp_key, ref secret_encryp_key, ref nonce)) = data_decryption_keys {
        try!(::utility::hybrid_decrypt(&raw_data, nonce, public_encryp_key, secret_encryp_key))
    } else {
        raw_data
    };

    let mut decoder = ::cbor::Decoder::from_bytes(data);
    Ok(try!(try!(decoder.decode().next().ok_or(::errors::ClientError::UnsuccessfulEncodeDecode))))
}

#[cfg(test)]
mod test {
    // extern crate rand;

    use super::*;
    // use self::rand::Rng;

    const TAG_ID : u64 = ::MAIDSAFE_TAG + 1000;

    fn get_client() -> ::std::sync::Arc<::std::sync::Mutex<::client::Client>> {
        let keyword = ::utility::generate_random_string(10);
        let password = ::utility::generate_random_string(10);
        let pin = ::utility::generate_random_pin();
        ::std::sync::Arc::new(::std::sync::Mutex::new(::client::Client::create_account(&keyword, pin, &password).unwrap()))
    }

    fn genearte_public_keys(size: usize) -> Vec<::sodiumoxide::crypto::sign::PublicKey> {
        let mut public_keys = Vec::with_capacity(size);
        for _ in 0..size {
            public_keys.push(::sodiumoxide::crypto::sign::gen_keypair().0);
        }
        public_keys
    }

    fn genearte_secret_keys(size: usize) -> Vec<::sodiumoxide::crypto::sign::SecretKey> {
        let mut secret_keys = Vec::with_capacity(size);
        for _ in 0..size {
            secret_keys.push(::sodiumoxide::crypto::sign::gen_keypair().1);
        }
        secret_keys
    }

    // fn generate_nonce() -> [u8; 24] {
    //     let v = rand::thread_rng().gen_iter::<u8>().take(24).collect::<Vec<u8>>();
    //     convert_to_array!(v, 24).unwrap()
    // }

    #[test]
    fn test_create() {
        let client = get_client();
        // Empty Data
        {
            let id : ::routing::NameType = ::routing::test_utils::Random::generate_random();
            let data = Vec::new();
            let owners = genearte_public_keys(1);
            let prev_owners = Vec::new();
            let ref secret_key = genearte_secret_keys(1)[0];
            let result = create(client.clone(),
                                TAG_ID,
                                id,
                                0,
                                data.clone(),
                                owners.clone(),
                                prev_owners.clone(),
                                secret_key,
                                None);
            assert!(result.is_ok());
        }
        // Data of size 80 KB
        {
            let id : ::routing::NameType = ::routing::test_utils::Random::generate_random();
            let data = vec![99u8; 1024 * 80];
            let owners = genearte_public_keys(1);
            let prev_owners = Vec::new();
            let ref secret_key = genearte_secret_keys(1)[0];
            let result = create(client.clone(),
                                TAG_ID,
                                id,
                                0,
                                data.clone(),
                                owners.clone(),
                                prev_owners.clone(),
                                secret_key,
                                None);
            assert!(result.is_ok());
        }
        // Data of size 80 KB with 10 owners
        {
            let id : ::routing::NameType = ::routing::test_utils::Random::generate_random();
            let data = vec![99u8; 1024 * 80];
            let owners = genearte_public_keys(10);
            let prev_owners = Vec::new();
            let ref secret_key = genearte_secret_keys(1)[0];
            let result = create(client.clone(),
                                TAG_ID,
                                id,
                                0,
                                data.clone(),
                                owners.clone(),
                                prev_owners.clone(),
                                secret_key,
                                None);
            assert!(result.is_ok());
        }
        // Data of size 80 KB with 100 owners
        {
            let id : ::routing::NameType = ::routing::test_utils::Random::generate_random();
            let data = vec![99u8; 1024 * 80];
            let owners = genearte_public_keys(100);
            let prev_owners = Vec::new();
            let ref secret_key = genearte_secret_keys(1)[0];
            let result = create(client.clone(),
                                TAG_ID,
                                id,
                                0,
                                data.clone(),
                                owners.clone(),
                                prev_owners.clone(),
                                secret_key,
                                None);
            assert!(result.is_ok());
        }
        // Data of size 80 KB with MAX owners
        {
            let id : ::routing::NameType = ::routing::test_utils::Random::generate_random();
            let data = vec![99u8; 1024 * 80];
            let owners = genearte_public_keys(543);
            let prev_owners = Vec::new();
            let ref secret_key = genearte_secret_keys(1)[0];
            let result = create(client.clone(),
                                TAG_ID,
                                id,
                                0,
                                data.clone(),
                                owners.clone(),
                                prev_owners.clone(),
                                secret_key,
                                None);
            assert!(result.is_ok());
        }
        // Data of size 80 KB with MAX + 5 owners
        {
            let id : ::routing::NameType = ::routing::test_utils::Random::generate_random();
            let data = vec![99u8; 1024 * 80];
            let owners = genearte_public_keys(548);
            let prev_owners = Vec::new();
            let ref secret_key = genearte_secret_keys(1)[0];
            let result = create(client.clone(),
                                TAG_ID,
                                id,
                                0,
                                data.clone(),
                                owners.clone(),
                                prev_owners.clone(),
                                secret_key,
                                None);
            assert!(result.is_err());
        }
    }

}
