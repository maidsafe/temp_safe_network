// Copyright 2015 MaidSafe.net limited.
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

// TODO move to utility - this mod should be private
/// Gnerates a mock client
pub fn get_client() -> Result<::client::Client, ::errors::ClientError> {
    let pin = try!(::utility::generate_random_string(10));
    let keyword = try!(::utility::generate_random_string(10));
    let password = try!(::utility::generate_random_string(10));

    ::client::Client::create_account(keyword, pin, password)
}

/// Gnerates Random public keys
pub fn generate_public_keys(size: usize) -> Vec<::sodiumoxide::crypto::sign::PublicKey> {
    let mut public_keys = Vec::with_capacity(size);
    for _ in 0..size {
        public_keys.push(::sodiumoxide::crypto::sign::gen_keypair().0);
    }
    public_keys
}

/// Gnerates public keys of maximun size
pub fn get_max_sized_public_keys(size: usize) -> Vec<::sodiumoxide::crypto::sign::PublicKey> {
    let mut public_keys = Vec::with_capacity(size);
    for _ in 0..size {
        public_keys.push(::sodiumoxide::crypto::sign::PublicKey([::std::u8::MAX; ::sodiumoxide::crypto::sign::PUBLICKEYBYTES]));
    }
    public_keys
}

/// Gnerates secret keys of maximun size
pub fn get_max_sized_secret_keys(size: usize) -> Vec<::sodiumoxide::crypto::sign::SecretKey> {
    let mut secret_keys = Vec::with_capacity(size);
    for _ in 0..size {
        secret_keys.push(::sodiumoxide::crypto::sign::SecretKey([::std::u8::MAX; ::sodiumoxide::crypto::sign::SECRETKEYBYTES]));
    }
    secret_keys
}

/// Gnerates Random SecretKey
pub fn generate_secret_keys(size: usize) -> Vec<::sodiumoxide::crypto::sign::SecretKey> {
    let mut secret_keys = Vec::with_capacity(size);
    for _ in 0..size {
        secret_keys.push(::sodiumoxide::crypto::sign::gen_keypair().1);
    }
    secret_keys
}

/// Saves data as immutable data and returns the name of the immutable data
pub fn save_as_immutable_data(client: &mut ::client::Client, data: Vec<u8>) -> ::routing::NameType {
    let immutable_data = ::routing::immutable_data::ImmutableData::new(::routing::immutable_data::ImmutableDataType::Normal, data);
    let name_of_immutable_data = immutable_data.name();
    client.put(::routing::data::Data::ImmutableData(immutable_data), None);
    name_of_immutable_data
}
