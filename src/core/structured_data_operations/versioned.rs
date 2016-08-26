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


use core::client::Client;
use core::errors::CoreError;
use core::structured_data_operations::{DataFitResult, check_if_data_can_fit_in_structured_data};
use maidsafe_utilities::serialisation::{deserialise, serialise};
use routing::{Data, DataIdentifier, ImmutableData, StructuredData, XorName};
use rust_sodium::crypto::sign;
use std::sync::{Arc, Mutex};

/// Create the StructuredData to manage versioned data.
#[cfg_attr(feature="clippy", allow(too_many_arguments))]
pub fn create(client: Arc<Mutex<Client>>,
              version_name_to_store: XorName,
              tag_type: u64,
              name: XorName,
              version: u64,
              owner_keys: Vec<sign::PublicKey>,
              prev_owner_keys: Vec<sign::PublicKey>,
              private_signing_key: &sign::SecretKey)
              -> Result<StructuredData, CoreError> {
    trace!("Creating versioned StructuredData.");

    create_impl(client,
                &[version_name_to_store],
                tag_type,
                name,
                version,
                owner_keys,
                prev_owner_keys,
                private_signing_key)
}

/// Get the complete version list
pub fn get_all_versions(client: Arc<Mutex<Client>>,
                        struct_data: &StructuredData)
                        -> Result<Vec<XorName>, CoreError> {
    trace!("Getting all versions of versioned StructuredData.");

    let immut_data = try!(get_immutable_data(client, struct_data));
    Ok(try!(deserialise(&immut_data.value())))
}

/// Append a new version
pub fn append_version(client: Arc<Mutex<Client>>,
                      struct_data: StructuredData,
                      version_to_append: XorName,
                      private_signing_key: &sign::SecretKey)
                      -> Result<StructuredData, CoreError> {
    trace!("Appending version to versioned StructuredData.");

    // let immut_data = try!(get_immutable_data(mut client, struct_data));
    // client.delete(immut_data);
    let mut versions = try!(get_all_versions(client.clone(), &struct_data));
    versions.push(version_to_append);
    create_impl(client,
                &versions,
                struct_data.get_type_tag(),
                *struct_data.name(),
                struct_data.get_version() + 1,
                struct_data.get_owner_keys().clone(),
                struct_data.get_previous_owner_keys().clone(),
                private_signing_key)
}

#[cfg_attr(feature="clippy", allow(too_many_arguments))]
fn create_impl(client: Arc<Mutex<Client>>,
               version_names_to_store: &[XorName],
               tag_type: u64,
               name: XorName,
               version: u64,
               owner_keys: Vec<sign::PublicKey>,
               prev_owner_keys: Vec<sign::PublicKey>,
               private_signing_key: &sign::SecretKey)
               -> Result<StructuredData, CoreError> {
    let immutable_data = ImmutableData::new(try!(serialise(&version_names_to_store)));
    let name_of_immutable_data = *immutable_data.name();

    let encoded_name = try!(serialise(&name_of_immutable_data));

    match try!(check_if_data_can_fit_in_structured_data(&encoded_name,
                                                        owner_keys.clone(),
                                                        prev_owner_keys.clone())) {
        DataFitResult::DataFits => {
            trace!("Name of ImmutableData containing versions fits in StructuredData.");

            let data = Data::Immutable(immutable_data);
            try!(Client::put_recover(client, data, None));

            Ok(try!(StructuredData::new(tag_type,
                                        name,
                                        version,
                                        encoded_name,
                                        owner_keys,
                                        prev_owner_keys,
                                        Some(private_signing_key))))
        }
        _ => {
            trace!("Name of ImmutableData containing versions does not fit in StructuredData.");
            Err(CoreError::StructuredDataHeaderSizeProhibitive)
        }
    }
}

fn get_immutable_data(client: Arc<Mutex<Client>>,
                      struct_data: &StructuredData)
                      -> Result<ImmutableData, CoreError> {
    let name = try!(deserialise(&struct_data.get_data()));
    let resp_getter = try!(unwrap!(client.lock()).get(DataIdentifier::Immutable(name), None));
    let data = try!(resp_getter.get());
    match data {
        Data::Immutable(immutable_data) => Ok(immutable_data),
        _ => Err(CoreError::ReceivedUnexpectedData),
    }
}

#[cfg(test)]
mod test {

    use core::utility;
    use rand;
    use routing::XorName;

    use std::sync::{Arc, Mutex};
    use super::*;

    const TAG_ID: u64 = ::core::MAIDSAFE_TAG + 1001;

    #[test]
    fn save_and_retrieve_immutable_data() {
        let client = unwrap!(utility::test_utils::get_client());
        let client = Arc::new(Mutex::new(client));

        let id: XorName = rand::random();
        let owners = utility::test_utils::generate_public_keys(1);
        let prev_owners = Vec::new();
        let secret_key = &utility::test_utils::generate_secret_keys(1)[0];

        let version_0: XorName = rand::random();

        let mut structured_data_result = create(client.clone(),
                                                version_0,
                                                TAG_ID,
                                                id,
                                                0,
                                                owners,
                                                prev_owners,
                                                secret_key);

        let mut structured_data = unwrap!(structured_data_result);
        let mut versions_res = get_all_versions(client.clone(), &structured_data);
        let mut versions = unwrap!(versions_res);
        assert_eq!(versions.len(), 1);

        let version_1: XorName = rand::random();

        structured_data_result =
            append_version(client.clone(), structured_data, version_1, secret_key);
        structured_data = unwrap!(structured_data_result);
        versions_res = get_all_versions(client, &structured_data);
        versions = unwrap!(versions_res);
        assert_eq!(versions.len(), 2);

        assert_eq!(versions[0], version_0);
        assert_eq!(versions[1], version_1);
    }
}
