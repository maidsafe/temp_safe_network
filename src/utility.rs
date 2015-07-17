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

/// Combined Asymmetric and Symmetric encryption. The data is encrypted using random Key and
/// IV with Xsalsa-symmetric encryption. Random IV ensures that same plain text produces different
/// cipher-texts for each fresh symmetric encryption. The Key and IV are then asymmetrically
/// enrypted using Public-MAID and the whole thing is then serialised into a single Vec<u8>.
pub fn hybrid_encrypt(plain_text: &[u8],
                      asym_nonce: &::sodiumoxide::crypto::box_::Nonce,
                      asym_public_key: &::sodiumoxide::crypto::box_::PublicKey,
                      asym_secret_key: &::sodiumoxide::crypto::box_::SecretKey) -> Result<Vec<u8>, ::errors::ClientError> {
    let sym_key = ::sodiumoxide::crypto::secretbox::gen_key();
    let sym_nonce = ::sodiumoxide::crypto::secretbox::gen_nonce();

    let mut asym_plain_text = [0u8; ::sodiumoxide::crypto::secretbox::KEYBYTES + ::sodiumoxide::crypto::secretbox::NONCEBYTES];
    for it in sym_key.0.iter().chain(sym_nonce.0.iter()).enumerate() {
        asym_plain_text[it.0] = *it.1;
    }

    let sym_cipher_text = ::sodiumoxide::crypto::secretbox::seal(plain_text, &sym_nonce, &sym_key);
    let asym_cipher_text = ::sodiumoxide::crypto::box_::seal(&asym_plain_text, asym_nonce, asym_public_key, asym_secret_key);

    let mut encoder = ::cbor::Encoder::from_memory();
    try!(encoder.encode(&[(asym_cipher_text, sym_cipher_text)]));

    Ok(encoder.into_bytes())
}

/// Reverse of hybrid_encrypt. Refer hybrid_encrypt.
pub fn hybrid_decrypt(cipher_text: &[u8],
                      asym_nonce: &::sodiumoxide::crypto::box_::Nonce,
                      asym_public_key: &::sodiumoxide::crypto::box_::PublicKey,
                      asym_secret_key: &::sodiumoxide::crypto::box_::SecretKey) -> Result<Vec<u8>, ::errors::ClientError> {
    let mut decoder = ::cbor::Decoder::from_bytes(cipher_text);
    let (asym_cipher_text, sym_cipher_text): (Vec<u8>, Vec<u8>) = try!(try!(decoder.decode().next().ok_or(::errors::ClientError::UnsuccessfulEncodeDecode)));

    if let Some(asym_plain_text) = ::sodiumoxide::crypto::box_::open(&asym_cipher_text, asym_nonce, asym_public_key, asym_secret_key) {
        if asym_plain_text.len() != ::sodiumoxide::crypto::secretbox::KEYBYTES + ::sodiumoxide::crypto::secretbox::NONCEBYTES {
            Err(::errors::ClientError::AsymmetricDecipherFailure)
        } else {
            let mut sym_key = ::sodiumoxide::crypto::secretbox::Key([0u8; ::sodiumoxide::crypto::secretbox::KEYBYTES]);
            let mut sym_nonce = ::sodiumoxide::crypto::secretbox::Nonce([0u8; ::sodiumoxide::crypto::secretbox::NONCEBYTES]);

            for it in asym_plain_text.iter().take(::sodiumoxide::crypto::secretbox::KEYBYTES).enumerate() {
                sym_key.0[it.0] = *it.1;
            }
            for it in asym_plain_text.iter().skip(::sodiumoxide::crypto::secretbox::KEYBYTES).enumerate() {
                sym_nonce.0[it.0] = *it.1;
            }

            if let Some(sym_plain_text) = ::sodiumoxide::crypto::secretbox::open(&sym_cipher_text, &sym_nonce, &sym_key) {
                Ok(sym_plain_text)
            } else {
                Err(::errors::ClientError::SymmetricDecipherFailure)
            }
        }
    } else {
        Err(::errors::ClientError::AsymmetricDecipherFailure)
    }
}

#[allow(dead_code)]
/// utility function to serialise an Encodable type
pub fn serialise<T>(data: T) -> Vec<u8> where T : ::rustc_serialize::Encodable {
    let mut e = ::cbor::Encoder::from_memory();
    let _ = e.encode(&[data]);
    e.into_bytes()
}

#[allow(dead_code)]
/// utility function to deserialise a ::data_parser::Parser
pub fn deserialise_parser(data: Vec<u8>) -> ::data_parser::Parser {
    let mut d = ::cbor::Decoder::from_bytes(data);
    d.decode().next().unwrap().unwrap()
}

#[allow(dead_code)]
/// utility function to deserialise a Decodable type
pub fn deserialise<T>(data: Vec<u8>) -> T where T: ::rustc_serialize::Decodable {
    let mut d = ::cbor::Decoder::from_bytes(data);
    d.decode().next().unwrap().unwrap()
}

#[allow(dead_code)]
/// Generates a random string for specified size
pub fn generate_random_string(length: usize) -> String {
    (0..length).map(|_| ::rand::random::<char>()).collect()
}

#[allow(dead_code)]
/// Generates a random PIN number
pub fn generate_random_pin() -> u32 {
    ::rand::random::<u32>() % 10000
}

/// Returns true if both slices are equal in length, and have equal contents
pub fn slice_equal<T: PartialEq>(lhs: &[T], rhs: &[T]) -> bool {
    lhs.len() == rhs.len() && lhs.iter().zip(rhs.iter()).all(|(a, b)| a == b)
}

/// Common utility functions for writting test cases
pub mod test_utils {

    /// Gnerates a mock client
    pub fn get_client() -> ::client::Client {
        let keyword = ::utility::generate_random_string(10);
        let password = ::utility::generate_random_string(10);
        let pin = ::utility::generate_random_pin();
        ::client::Client::create_account(&keyword, pin, &password).unwrap()
    }
    /// Gnerates Random public keys
    pub fn genearte_public_keys(size: usize) -> Vec<::sodiumoxide::crypto::sign::PublicKey> {
        let mut public_keys = Vec::with_capacity(size);
        for _ in 0..size {
            public_keys.push(::sodiumoxide::crypto::sign::gen_keypair().0);
        }
        public_keys
    }
    /// Gnerates Random SecretKey
    pub fn genearte_secret_keys(size: usize) -> Vec<::sodiumoxide::crypto::sign::SecretKey> {
        let mut secret_keys = Vec::with_capacity(size);
        for _ in 0..size {
            secret_keys.push(::sodiumoxide::crypto::sign::gen_keypair().1);
        }
        secret_keys
    }
    /// Saves data as immutable data and returns the name of the immutable data
    pub fn save_as_immutable_data(client: &mut ::client::Client, data: Vec<u8>) -> ::routing::NameType {
        let immutable_data = ::client::ImmutableData::new(::client::ImmutableDataType::Normal, data);
        let name_of_immutable_data = immutable_data.name();
        let _ = client.put_new(name_of_immutable_data.clone(), ::client::Data::ImmutableData(immutable_data));
        name_of_immutable_data
    }
}
