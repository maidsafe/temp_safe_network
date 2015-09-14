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

/// Gnerates a random mock client for testing
pub fn get_client() -> Result<::client::Client, ::errors::ClientError> {
    let pin = try!(::utility::generate_random_string(10));
    let keyword = try!(::utility::generate_random_string(10));
    let password = try!(::utility::generate_random_string(10));

    ::client::Client::create_account(keyword, pin, password)
}

/// Gnerates Random Public Keys
pub fn generate_public_keys(len: usize) -> Vec<::sodiumoxide::crypto::sign::PublicKey> {
    (0..len).map(|_| ::sodiumoxide::crypto::sign::gen_keypair().0).collect()
}

/// Gnerates Random Secret Keys
pub fn generate_secret_keys(len: usize) -> Vec<::sodiumoxide::crypto::sign::SecretKey> {
    (0..len).map(|_| ::sodiumoxide::crypto::sign::gen_keypair().1).collect()
}

/// Gnerates public keys of maximun size
pub fn get_max_sized_public_keys(len: usize) -> Vec<::sodiumoxide::crypto::sign::PublicKey> {
    ::std::iter::repeat(::sodiumoxide::crypto::sign::PublicKey([::std::u8::MAX; ::sodiumoxide::crypto::sign::PUBLICKEYBYTES])).take(len).collect()
}

/// Gnerates secret keys of maximun size
pub fn get_max_sized_secret_keys(len: usize) -> Vec<::sodiumoxide::crypto::sign::SecretKey> {
    ::std::iter::repeat(::sodiumoxide::crypto::sign::SecretKey([::std::u8::MAX; ::sodiumoxide::crypto::sign::SECRETKEYBYTES])).take(len).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn random_client() {
        let client0 = eval_result!(get_client());
        let client1 = eval_result!(get_client());

        assert!(eval_result!(client0.get_public_signing_key()) != eval_result!(client1.get_public_signing_key()));
        assert!(eval_result!(client0.get_public_encryption_key()) != eval_result!(client1.get_public_encryption_key()));
    }
}
