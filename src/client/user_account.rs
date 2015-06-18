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
// relating to use of the SAFE Network Software.                                                                */

#![allow(unsafe_code)]

use cbor;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

use routing;
use maidsafe_types;

static MAIDSAFE_VERSION_LABEL : &'static str = "MaidSafe Version 1 Key Derivation";

#[derive(Clone, PartialEq, Debug, RustcEncodable, RustcDecodable)]
/// Represents a Session Packet for the user. It is necessary to fetch and decode this via user
/// supplied credentials to retrieve all the Maid/Mpid etc keys of the user and also his Root
/// Directory ID if he has put data onto the network.
pub struct Account {
    an_maid: maidsafe_types::RevocationIdType,
    maid: maidsafe_types::IdType,
    public_maid: maidsafe_types::PublicIdType,

    an_mpid: maidsafe_types::RevocationIdType,
    mpid: maidsafe_types::IdType,
    public_mpid: maidsafe_types::PublicIdType,

    root_dir_id: Option<routing::NameType>,
}

#[allow(dead_code)]
impl Account {
    /// Create a new Session Packet with Randomly generated Maid keys for the user
    pub fn new(root_dir_id: Option<routing::NameType>) -> Account {
        let an_maid = maidsafe_types::RevocationIdType::new::<maidsafe_types::MaidTypeTags>();
        let maid = maidsafe_types::IdType::new(&an_maid);
        let public_maid = maidsafe_types::PublicIdType::new(&maid, &an_maid);

        let an_mpid = maidsafe_types::RevocationIdType::new::<maidsafe_types::MpidTypeTags>();
        let mpid = maidsafe_types::IdType::new(&an_mpid);
        let public_mpid = maidsafe_types::PublicIdType::new(&mpid, &an_mpid);

        Account {
            an_maid: an_maid,
            maid: maid,
            public_maid: public_maid,
            an_mpid: an_mpid,
            mpid: mpid,
            public_mpid: public_mpid,
            root_dir_id: root_dir_id,
        }
    }

    /// Get user's AnMAID
    pub fn get_an_maid(&self) -> &maidsafe_types::RevocationIdType {
        &self.an_maid
    }

    /// Get user's MAID
    pub fn get_maid(&self) -> &maidsafe_types::IdType {
        &self.maid
    }

    /// Get user's Public-MAID
    pub fn get_public_maid(&self) -> &maidsafe_types::PublicIdType {
        &self.public_maid
    }

    /// Get user's AnMPID
    pub fn get_an_mpid(&self) -> &maidsafe_types::RevocationIdType {
        &self.an_mpid
    }

    /// Get user's MPID
    pub fn get_mpid(&self) -> &maidsafe_types::IdType {
        &self.mpid
    }

    /// Get user's Public-MPID
    pub fn get_public_mpid(&self) -> &maidsafe_types::PublicIdType {
        &self.public_mpid
    }

    /// Get user's root directory ID
    pub fn get_root_dir_id(&self) -> Option<&routing::NameType> {
        match self.root_dir_id {
            Some(ref dir_id) => Some(dir_id),
            None => None,
        }
    }

    /// Set user's root directory ID
    pub fn set_root_dir_id(&mut self, root_dir_id: routing::NameType) -> bool {
        match self.root_dir_id {
            Some(_) => false,
            None => {
                self.root_dir_id = Some(root_dir_id);
                true
            },
        }
    }

    /// Generate User's Identity for the network using supplied credentials in a deterministic way.
    /// This is similary to the username in various places.
    pub fn generate_network_id(keyword: &String, pin: u32) -> routing::NameType {
        use crypto::digest::Digest;

        let mut hasher = ::crypto::sha2::Sha512::new();
        let digest_size = hasher.output_bytes();

        let mut output1 = vec![0u8; digest_size];
        let mut output2 = vec![0u8; digest_size];

        hasher.input_str(&keyword);
        hasher.result(&mut output1);

        hasher.reset();
        Account::hash_pin(&mut hasher, pin);
        hasher.result(&mut output2);

        let mut name = [0u8; 64];
        hasher.reset();
        hasher.input(&output1);
        hasher.input(&output2);
        hasher.result(&mut name);

        routing::NameType::new(name)
    }

    /// Symmetric encryption of Session Packet using User's credentials. Credentials are passed
    /// through PBKDF2 first
    pub fn encrypt(&self, password: &[u8], pin: u32) -> Result<Vec<u8>, ::MaidsafeError> {
        let serialised = try!(self.serialise());

        let mut encrypted : Vec<u8> = Vec::new();
        {
            use crypto::symmetriccipher::Encryptor;
            use crypto::buffer::WriteBuffer;
            use crypto::buffer::ReadBuffer;

            // when/if multiple versions are supported, the key/iv should be
            // unpredictable. Consider the new key gen algorithm for EncryptDataMap.
            let keys = Account::generate_crypto_keys(password, pin);

            // there SHOULD be an HMAC here. But crypto-experts recommend a second
            // key for this, and there isn't much to go on. Maybe the pin shouldn't
            // be used in the password? it'd be a weak hmac though, kinda screwed
            // really, authenticated encryption is necessary here probably.

            let mut buffer = [0u8; 4096];
            let mut read_buffer = ::crypto::buffer::RefReadBuffer::new(&serialised);
            let mut write_buffer = ::crypto::buffer::RefWriteBuffer::new(&mut buffer);
            let mut encryptor = ::crypto::aes::cbc_encryptor(
                ::crypto::aes::KeySize::KeySize256, &keys.0, &keys.1, ::crypto::blockmodes::PkcsPadding);

            loop {
                let result = try!(encryptor.encrypt(&mut read_buffer, &mut write_buffer, true));
                encrypted.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&a| a.clone()));
                match result {
                    ::crypto::buffer::BufferResult::BufferUnderflow => break,
                    ::crypto::buffer::BufferResult::BufferOverflow => {}
                }
            }
        }

        return Ok(encrypted);
    }

    /// Symmetric decryption of Session Packet using User's credentials. Credentials are passed
    /// through PBKDF2 first
    pub fn decrypt(encrypted: &[u8], password: &[u8], pin: u32) -> Result<Account, ::MaidsafeError> {
        let mut decrypted : Vec<u8> = Vec::new();
        {
            use crypto::symmetriccipher::Decryptor;
            use crypto::buffer::WriteBuffer;
            use crypto::buffer::ReadBuffer;

            // when/if multiple versions are supported, the key/iv should be
            // unpredictable. Consider the new key gen algorithm for EncryptDataMap.
            let keys = Account::generate_crypto_keys(password, pin);

            // there SHOULD be an HMAC here. But crypto-experts recommend a second
            // key for this, and there isn't much to go on. Maybe the pin shouldn't
            // be used in the password? it'd be a weak hmac though, kinda screwed
            // really, authenticated encryption is necessary here probably.

            let mut buffer = [0u8; 4096];
            let mut read_buffer = ::crypto::buffer::RefReadBuffer::new(&encrypted);
            let mut write_buffer = ::crypto::buffer::RefWriteBuffer::new(&mut buffer);
            let mut decryptor = ::crypto::aes::cbc_decryptor(
                ::crypto::aes::KeySize::KeySize256, &keys.0, &keys.1, ::crypto::blockmodes::PkcsPadding);

            loop {
                let result = try!(decryptor.decrypt(&mut read_buffer, &mut write_buffer, true));
                decrypted.extend(write_buffer.take_read_buffer().take_remaining().iter().map(|&a| a.clone()));
                match result {
                    ::crypto::buffer::BufferResult::BufferUnderflow => break,
                    ::crypto::buffer::BufferResult::BufferOverflow => {}
                }
            }
        }

        Ok(try!(Account::deserialise(&decrypted)))
    }

    fn hash_pin(hasher : &mut ::crypto::sha2::Sha512, pin : u32) {
        use crypto::digest::Digest;
        use std::slice;
        unsafe {
            let address : *const u8 = ::std::mem::transmute(&pin);
            hasher.input(&slice::from_raw_parts(address, ::std::mem::size_of_val(&pin)));
        }
    }

    fn generate_crypto_keys(password: &[u8], pin: u32) -> (Vec<u8>, Vec<u8>) {
        use crypto::digest::Digest;

        let mut hasher = ::crypto::sha2::Sha512::new();
        let digest_size = hasher.output_bytes();
        let key_size = digest_size / 2;
        let iv_size = key_size / 2;

        let iterations : u16 = ((pin % 10000) + 10000) as u16;
        let mut salt : Vec<u8>;
        {
            let mut salt_partial = vec![0u8; digest_size];

            Account::hash_pin(&mut hasher, pin);
            hasher.result(&mut salt_partial);

            // Original version uses a Secure Byte Block,
            // which we have no available resource for
            salt = salt_partial.iter().chain(password.iter())
                                      .chain(MAIDSAFE_VERSION_LABEL.as_bytes().iter())
                                      .map(|&a| a.clone())
                                      .collect();
        }

        hasher.reset();
        let mut mac = ::crypto::hmac::Hmac::new(hasher, &password);

        let mut output = vec![0u8 ; digest_size];
        ::crypto::pbkdf2::pbkdf2(&mut mac, &salt, iterations as u32, &mut output);

        let key = output.iter().take(key_size).map(|&a| a.clone()).collect();
        let iv = output.into_iter().skip(key_size).take(iv_size).collect();

        return (key, iv);
    }

    fn serialise(&self) -> cbor::CborResult<Vec<u8>> {
        let mut encoder = cbor::Encoder::from_memory();
        return encoder.encode(&[&self]).map(|()| encoder.into_bytes());
    }

    fn deserialise(source : &[u8]) -> cbor::CborResult<Account> {
        match cbor::Decoder::from_bytes(source).decode::<Account>().next() {
            Some(result) => return result,
            None => return Err(cbor::CborError::UnexpectedEOF)
        }
    }
}

#[cfg(test)]
mod test {
    use std;

    use super::Account;

    fn slice_eq(left : &[u8], right : &[u8]) -> bool {
        return left.iter().zip(right.iter()).all(|(a, b)| a == b);
    }

    #[test]
    fn generating_new_account() {
        let account1 = Account::new(None);
        let account2 = Account::new(None);
        assert!(account1 != account2);
    }

    #[test]
    fn generating_network_id() {
        let keyword1 = "user1".to_string();
        {
            let user1_id1 = Account::generate_network_id(&keyword1, 0);
            let user1_id2 = Account::generate_network_id(&keyword1, 1234);
            let user1_id3 = Account::generate_network_id(&keyword1, std::u32::MAX);

            assert!(!slice_eq(&user1_id1.get_id(), &user1_id2.get_id()));
            assert!(!slice_eq(&user1_id1.get_id(), &user1_id3.get_id()));
            assert!(!slice_eq(&user1_id2.get_id(), &user1_id3.get_id()));
            assert!(slice_eq(&user1_id1.get_id(), &Account::generate_network_id(&keyword1, 0).get_id()));
            assert!(slice_eq(&user1_id2.get_id(), &Account::generate_network_id(&keyword1, 1234).get_id()));
            assert!(slice_eq(&user1_id3.get_id(), &Account::generate_network_id(&keyword1, std::u32::MAX).get_id()));
        }
        {
            let keyword2 = "user2".to_string();
            assert!(
                !slice_eq(
                    &Account::generate_network_id(&keyword1, 248).get_id(),
                    &Account::generate_network_id(&keyword2, 248).get_id()));
        }
    }

    #[test]
    fn generating_crypto_keys() {
        let password1 = "super great password".as_bytes();
        let password2 = "even better password".as_bytes();
        {
            let keys1 = Account::generate_crypto_keys(&password1, 0);
            let keys2 = Account::generate_crypto_keys(&password1, 1234);
            let keys3 = Account::generate_crypto_keys(&password1, std::u32::MAX);

            assert!(!slice_eq(&keys1.0, &keys2.0));
            assert!(!slice_eq(&keys1.0, &keys3.0));
            assert!(!slice_eq(&keys2.0, &keys3.0));

            assert!(!slice_eq(&keys1.1, &keys2.1));
            assert!(!slice_eq(&keys1.1, &keys3.1));
            assert!(!slice_eq(&keys2.1, &keys3.1));
        }
        {
            let keys1 = Account::generate_crypto_keys(&password1, 0);
            let keys2 = Account::generate_crypto_keys(&password2, 0);

            assert!(!slice_eq(&keys1.0, &keys2.0));
            assert!(!slice_eq(&keys1.1, &keys2.1));
        }
        {
            let keys = Account::generate_crypto_keys(&password1, 0);
            let again = Account::generate_crypto_keys(&password1, 0);
            assert!(slice_eq(&keys.0, &again.0));
            assert!(slice_eq(&keys.1, &again.1));
        }
    }

    #[test]
    fn serialisation() {
        let account = Account::new(None);

        match account.serialise() {
            Ok(serialised) => match Account::deserialise(&serialised) {
                Ok(account_again) => assert_eq!(account, account_again),
                Err(_) => panic!("Should have been equal !!"),
            },
            Err(_) => panic!("Serialisation Failed !!"),
        }
    }

    #[test]
    fn encryption() {
        let account = Account::new(None);

        let password = "impossible to guess".to_string().into_bytes();
        let pin = 10000u32;
        match account.encrypt(&password, pin) {
            Ok(encrypted) => {
                assert!(encrypted.len() > 0);
                assert!(account.serialise().map(|serialised| assert!(encrypted != serialised)).is_ok());
                match Account::decrypt(&encrypted, &password, pin) {
                    Ok(account_again) => assert_eq!(account, account_again),
                    Err(_) => panic!("Should have been equal !!"),
                }
            }
            Err(_) => panic!("Account Encryption Failed !!"),
        }
    }
}
