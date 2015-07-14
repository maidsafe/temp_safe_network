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
            let _ = client.put_new(name_of_immutable_data, data);
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

    match client.get_new(location, ::client::DataRequest::ImmutableData(::client::ImmutableDataType::Normal)).unwrap().get() {
        Ok(raw_data) => {
            let mut decoder = ::cbor::Decoder::from_bytes(raw_data);
            match try!(try!(decoder.decode().next().ok_or(::errors::ClientError::UnsuccessfulEncodeDecode))) {
                ::client::Data::ImmutableData(immut_data) => Ok(immut_data),
                _ => Err(::errors::ClientError::ReceivedUnexpectedData),
            }
        },
        Err(_) => Err(::errors::ClientError::GetFailure),
    }
}
