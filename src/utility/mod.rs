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

/// Common utility functions for writting test cases
pub mod test_utils;

use ::rand::Rng;

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

    Ok(try!(serialise(&(asym_cipher_text, sym_cipher_text))))
}

/// Reverse of hybrid_encrypt. Refer hybrid_encrypt.
pub fn hybrid_decrypt(cipher_text: &[u8],
                      asym_nonce: &::sodiumoxide::crypto::box_::Nonce,
                      asym_public_key: &::sodiumoxide::crypto::box_::PublicKey,
                      asym_secret_key: &::sodiumoxide::crypto::box_::SecretKey) -> Result<Vec<u8>, ::errors::ClientError> {
    let (asym_cipher_text, sym_cipher_text): (Vec<u8>, Vec<u8>) = try!(deserialise(cipher_text));

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

/// utility function to serialise an Encodable type
pub fn serialise<T>(data: &T) -> Result<Vec<u8>, ::errors::ClientError>
                                 where T: ::rustc_serialize::Encodable {
    let mut encoder = ::cbor::Encoder::from_memory();
    try!(encoder.encode(&[data]));
    Ok(encoder.into_bytes())
}

/// utility function to deserialise a Decodable type
pub fn deserialise<T>(data: &[u8]) -> Result<T, ::errors::ClientError>
                                      where T: ::rustc_serialize::Decodable {
    let mut d = ::cbor::Decoder::from_bytes(data);
    Ok(try!(try!(d.decode().next().ok_or(::errors::ClientError::UnsuccessfulEncodeDecode))))
}

/// Generates a random string for specified size
pub fn generate_random_string(length: usize) -> Result<String, ::errors::ClientError> {
    let mut os_rng = try!(::rand::OsRng::new());
    Ok((0..length).map(|_| os_rng.gen::<char>()).collect())
}

/// Generates a random PIN number
pub fn generate_random_pin() -> u32 {
    ::rand::random::<u32>() % 10000
}

/// Generate a random vector of given length
pub fn generate_random_vector<T>(length: usize) -> Result<Vec<T>, ::errors::ClientError>
                                                   where T: ::rand::Rand {
    let mut os_rng = try!(::rand::OsRng::new());
    Ok((0..length).map(|_| os_rng.gen()).collect())
}

/// Returns true if both slices are equal in length, and have equal contents
pub fn slice_equal<T: PartialEq>(lhs: &[T], rhs: &[T]) -> bool {
    lhs.len() == rhs.len() && lhs.iter().zip(rhs.iter()).all(|(a, b)| a == b)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn hybrid_encrypt_decrypt() {
        // Identical Plain Texts
        let plain_text_0 = vec![123u8; 1000];
        let plain_text_1 = plain_text_0.clone();

        let nonce = ::sodiumoxide::crypto::box_::gen_nonce();
        let (public_key, secret_key) = ::sodiumoxide::crypto::box_::gen_keypair();

        // Encrypt
        let hybrid_encrypt_0 = hybrid_encrypt(&plain_text_0[..], &nonce, &public_key, &secret_key);
        let hybrid_encrypt_1 = hybrid_encrypt(&plain_text_1[..], &nonce, &public_key, &secret_key);

        assert!(hybrid_encrypt_0.is_ok());
        assert!(hybrid_encrypt_1.is_ok());

        // Same Plain Texts
        assert_eq!(plain_text_0, plain_text_1);

        let result0 = hybrid_encrypt_0.ok().unwrap();
        let result1 = hybrid_encrypt_1.ok().unwrap();

        // Different Results because of random "iv"
        assert!(result0 != result1);

        // Decrypt
        let hybrid_decrypt_0 = hybrid_decrypt(&result0, &nonce, &public_key, &secret_key);
        let hybrid_decrypt_1 = hybrid_decrypt(&result1, &nonce, &public_key, &secret_key);

        assert!(hybrid_decrypt_0.is_ok());
        assert!(hybrid_decrypt_1.is_ok());

        // Should have decrypted to the same Plain Texts
        assert_eq!(plain_text_0, hybrid_decrypt_0.ok().unwrap());
        assert_eq!(plain_text_1, hybrid_decrypt_1.ok().unwrap());
    }

    #[test]
    fn serialise_deserialise() {
        let original_data = (generate_random_vector::<u8>(13).ok().unwrap(),
                             generate_random_vector::<i64>(19).ok().unwrap(),
                             generate_random_string(10).ok().unwrap());

        let serialised_data = serialise(&original_data).ok().unwrap();
        let deserialised_data: (Vec<u8>, Vec<i64>, String) = deserialise(&serialised_data).ok().unwrap();
        assert_eq!(original_data, deserialised_data);
    }

    #[test]
    fn random_string() {
        const SIZE: usize = 10;
        let str0 = generate_random_string(SIZE).ok().unwrap();
        let str1 = generate_random_string(SIZE).ok().unwrap();
        let str2 = generate_random_string(SIZE).ok().unwrap();

        assert!(str0 != str1);
        assert!(str0 != str2);
        assert!(str1 != str2);
    }

    #[test]
    fn random_vector() {
        const SIZE: usize = 10;
        let vec0 = generate_random_vector::<u8>(SIZE).ok().unwrap();
        let vec1 = generate_random_vector::<u8>(SIZE).ok().unwrap();
        let vec2 = generate_random_vector::<u8>(SIZE).ok().unwrap();

        assert!(vec0 != vec1);
        assert!(vec0 != vec2);
        assert!(vec1 != vec2);
    }
}
