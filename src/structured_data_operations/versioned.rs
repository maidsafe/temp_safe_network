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
pub fn create(client: &::client::Client,
              version_name_to_store: ::routing::NameType,
              tag_type: u64,
              identifier: ::routing::NameType,
              version: u64,
              owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
              prev_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
              private_signing_key: &::sodiumoxide::crypto::sign::SecretKey) -> Result<::routing::structured_data::StructuredData, ::errors::CoreError> {
    create_impl(client,
                &vec![version_name_to_store],
                tag_type,
                identifier,
                version,
                owner_keys,
                prev_owner_keys,
                private_signing_key)
}

/// Get the complete version list
pub fn get_all_versions(client: &mut ::client::Client, struct_data: &::routing::structured_data::StructuredData) -> Result<Vec<::routing::NameType>, ::errors::CoreError> {
    let immut_data = try!(get_immutable_data(client, struct_data));
    ::utility::deserialise(&immut_data.value())
}

/// Append a new version
pub fn append_version(client: &mut ::client::Client,
                      struct_data: ::routing::structured_data::StructuredData,
                      version_to_append: ::routing::NameType,
                      private_signing_key: &::sodiumoxide::crypto::sign::SecretKey) -> Result<::routing::structured_data::StructuredData, ::errors::CoreError> {
    // let immut_data = try!(get_immutable_data(mut client, struct_data));
    // client.delete(immut_data);
    let mut versions = try!(get_all_versions(client, &struct_data));
    versions.push(version_to_append);
    create_impl(client,
                &versions,
                struct_data.get_type_tag(),
                struct_data.get_identifier().clone(),
                struct_data.get_version() + 1,
                struct_data.get_owner_keys().clone(),
                struct_data.get_previous_owner_keys().clone(),
                private_signing_key)
}

fn create_impl(client: &::client::Client,
               version_names_to_store: &Vec<::routing::NameType>,
               tag_type: u64,
               identifier: ::routing::NameType,
               version: u64,
               owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
               prev_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
               private_signing_key: &::sodiumoxide::crypto::sign::SecretKey) -> Result<::routing::structured_data::StructuredData, ::errors::CoreError> {
    let immutable_data = ::routing::immutable_data::ImmutableData::new(::routing::immutable_data::ImmutableDataType::Normal,
                                                                       try!(::utility::serialise(version_names_to_store)));
    let name_of_immutable_data = immutable_data.name();

    let encoded_name = try!(::utility::serialise(&name_of_immutable_data));

    match try!(::structured_data_operations::check_if_data_can_fit_in_structured_data(&encoded_name, owner_keys.clone(), prev_owner_keys.clone())) {
        ::structured_data_operations::DataFitResult::DataFits => {
            let data = ::routing::data::Data::ImmutableData(immutable_data);
            try!(client.put(data, None));

            Ok(try!(::routing::structured_data::StructuredData::new(tag_type,
                                                                    identifier,
                                                                    version,
                                                                    encoded_name,
                                                                    owner_keys,
                                                                    prev_owner_keys,
                                                                    Some(private_signing_key))))
        },
        _ => Err(::errors::CoreError::StructuredDataHeaderSizeProhibitive),
    }
}

fn get_immutable_data(client: &mut ::client::Client,
                      struct_data: &::routing::structured_data::StructuredData) -> Result<::routing::immutable_data::ImmutableData, ::errors::CoreError> {
    let name = try!(::utility::deserialise(&struct_data.get_data()));
    let response_getter = client.get(::routing::data::DataRequest::ImmutableData(name, ::routing::immutable_data::ImmutableDataType::Normal), None);
    let data = try!(response_getter.get());
    match data {
        ::routing::data::Data::ImmutableData(immutable_data) => Ok(immutable_data),
        _ => Err(::errors::CoreError::ReceivedUnexpectedData),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TAG_ID: u64 = ::MAIDSAFE_TAG + 1001;

    #[test]
    fn save_and_retrieve_immutable_data() {
        let mut client = eval_result!(::utility::test_utils::get_client());

        let id = ::routing::NameType::new(eval_result!(::utility::generate_random_array_u8_64()));
        let owners = ::utility::test_utils::generate_public_keys(1);
        let prev_owners = Vec::new();
        let ref secret_key = ::utility::test_utils::generate_secret_keys(1)[0];

        let version_0 = ::routing::NameType::new(eval_result!(::utility::generate_random_array_u8_64()));

        let mut structured_data_result = create(&client, version_0.clone(), TAG_ID, id, 0, owners, prev_owners, secret_key);

        let mut structured_data = eval_result!(structured_data_result);
        let mut versions_res = get_all_versions(&mut client, &structured_data);
        let mut versions = eval_result!(versions_res);
        assert_eq!(versions.len(), 1);

        let version_1 = ::routing::NameType::new(eval_result!(::utility::generate_random_array_u8_64()));

        structured_data_result = append_version(&mut client, structured_data, version_1.clone(), secret_key);
        structured_data = eval_result!(structured_data_result);
        versions_res = get_all_versions(&mut client, &structured_data);
        versions = eval_result!(versions_res);
        assert_eq!(versions.len(), 2);

        assert_eq!(versions[0], version_0);
        assert_eq!(versions[1], version_1);
    }
}
