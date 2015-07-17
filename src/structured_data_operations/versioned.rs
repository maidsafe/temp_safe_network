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

/// Create the StructuredData to manage versioned data.
pub fn create(client: &mut ::client::Client,
              version_name_to_store: ::routing::NameType,
              tag_type: u64,
              identifier: ::routing::NameType,
              version: u64,
              owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
              prev_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
              private_signing_key: &::sodiumoxide::crypto::sign::SecretKey) -> Result<::client::StructuredData, ::errors::ClientError> {
    create_impl(client,
                vec![version_name_to_store],
                tag_type,
                identifier,
                version,
                owner_keys,
                prev_owner_keys,
                private_signing_key)
}

/// Get the complete version list
pub fn get_all_versions(client: &mut ::client::Client, struct_data: &::client::StructuredData) -> Result<Vec<::routing::NameType>, ::errors::ClientError> {
    let immut_data = try!(get_immutable_data(client, struct_data));
    let mut decoder = ::cbor::Decoder::from_bytes(&immut_data.value()[..]);
    Ok(try!(try!(decoder.decode().next().ok_or(::errors::ClientError::UnsuccessfulEncodeDecode))))
}

/// Append a new version
pub fn append_version(client: &mut ::client::Client,
                      struct_data: ::client::StructuredData,
                      version_to_append: ::routing::NameType,
                      private_signing_key: &::sodiumoxide::crypto::sign::SecretKey) -> Result<::client::StructuredData, ::errors::ClientError> {
    // let immut_data = try!(get_immutable_data(mut client, struct_data));
    // client.delete(immut_data);
    let mut versions = try!(get_all_versions(client, &struct_data));
    versions.push(version_to_append);
    create_impl(client,
                versions,
                struct_data.get_tag_type(),
                struct_data.get_identifier().clone(),
                struct_data.get_version(),
                struct_data.get_owners().clone(),
                struct_data.get_previous_owners().clone(),
                private_signing_key)
}

fn create_impl(client: &mut ::client::Client,
               version_names_to_store: Vec<::routing::NameType>,
               tag_type: u64,
               identifier: ::routing::NameType,
               version: u64,
               owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
               prev_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
               private_signing_key: &::sodiumoxide::crypto::sign::SecretKey) -> Result<::client::StructuredData, ::errors::ClientError> {
    let mut version_encoder = ::cbor::Encoder::from_memory();
    try!(version_encoder.encode(version_names_to_store));

    let immutable_data = ::client::ImmutableData::new(::client::ImmutableDataType::Normal, version_encoder.into_bytes());
    let name_of_immutable_data = immutable_data.name();

    let mut name_encoder = ::cbor::Encoder::from_memory();
    try!(name_encoder.encode(&[name_of_immutable_data.clone()]));
    let encoded_name = name_encoder.into_bytes();

    let data = ::client::Data::ImmutableData(immutable_data);

    match ::structured_data_operations::check_if_data_can_fit_in_structured_data(encoded_name.clone(), owner_keys.clone(), prev_owner_keys.clone()) {
        ::structured_data_operations::DataFitResult::DataFits => {
            let _ = client.put(name_of_immutable_data, data);
            Ok(::client::StructuredData::new(tag_type,
                                             identifier,
                                             version,
                                             encoded_name,
                                             owner_keys,
                                             prev_owner_keys,
                                             private_signing_key))
        },
        _ => Err(::errors::ClientError::StructuredDataHeaderSizeProhibitive),
    }
}

fn get_immutable_data(client: &mut ::client::Client,
                      struct_data: &::client::StructuredData) -> Result<::client::ImmutableData, ::errors::ClientError> {
    let mut decoder = ::cbor::Decoder::from_bytes(&struct_data.get_data()[..]);
    let location = try!(try!(decoder.decode().next().ok_or(::errors::ClientError::UnsuccessfulEncodeDecode)));

    let mut response_getter = try!(client.get(location, ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal)));
    let data = try!(response_getter.get());
    match data {
        ::client::Data::ImmutableData(immutable_data) => {
            let mut decoder = ::cbor::Decoder::from_bytes(immutable_data.value().clone());
            match try!(try!(decoder.decode().next().ok_or(::errors::ClientError::UnsuccessfulEncodeDecode))) {
                ::client::Data::ImmutableData(immut_data) => Ok(immut_data),
                _ => Err(::errors::ClientError::ReceivedUnexpectedData),
            }
        },
        _ => Err(::errors::ClientError::ReceivedUnexpectedData),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TAG_ID : u64 = ::MAIDSAFE_TAG + 1001;

    #[test]
    fn save_and_retrieve_immtable_data() {
        let mut client = ::utility::test_utils::get_client();
        let id : ::routing::NameType = ::routing::test_utils::Random::generate_random();
        let first_data = vec![1u8; 10];
        let second_data = vec![2u8; 20];
        let owners = ::utility::test_utils::generate_public_keys(1);
        let prev_owners = Vec::new();
        let ref secret_key = ::utility::test_utils::generate_secret_keys(1)[0];
        let mut version = ::utility::test_utils::save_as_immutable_data(&mut client, first_data.clone());

        let structured_data_result = create(&mut client, version, TAG_ID, id, 0, owners, prev_owners, secret_key);
        assert!(structured_data_result.is_ok());
        let mut structured_data = structured_data_result.ok().unwrap();
        let mut versions = get_all_versions(&mut client, &structured_data).ok().unwrap();
        assert_eq!(versions.len(), 1);
        version = ::utility::test_utils::save_as_immutable_data(&mut client, second_data.clone());
        assert!(append_version(&mut client, structured_data.clone(), version.clone(), secret_key).is_err());
        structured_data.set_version(1u64);
        assert!(append_version(&mut client, structured_data.clone(), version, secret_key).is_ok());
        versions = get_all_versions(&mut client, &structured_data).ok().unwrap();
        assert_eq!(versions.len(), 2);
    }
}
