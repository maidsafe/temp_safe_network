/*  Copyright 2015 MaidSafe.net limited

    This MaidSafe Software is licensed to you under (1) the MaidSafe.net Commercial License,
    version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
    licence you accepted on initial access to the Software (the "Licences").

    By contributing code to the MaidSafe Software, or to this project generally, you agree to be
    bound by the terms of the MaidSafe Contributor Agreement, version 1.0, found in the root
    directory of this project at LICENSE, COPYING and CONTRIBUTOR respectively and also
    available at: http://www.maidsafe.net/licenses

    Unless required by applicable law or agreed to in writing, the MaidSafe Software distributed
    under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS
    OF ANY KIND, either express or implied.

    See the Licences for the specific language governing permissions and limitations relating to
    use of the MaidSafe Software.                                                                 */
use cbor;
use routing;
use std;
use sodiumoxide::crypto;

use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use std::mem;
use routing::routing_client::ClientIdPacket;

static ACCOUNT_TAG : u64 = 5483_4000;
static MAIDSAFE_VERSION_LABEL : &'static str = "MaidSafe Version 1 Key Derivation";

pub struct Account {
    account_id : ClientIdPacket,
    // Add Mpids etc
}

///
/// Represents an Account on the SAFE network
///
impl Account {
    pub fn new() -> Account {
        let sign_keys = crypto::sign::gen_keypair();
        let asym_keys = crypto::asymmetricbox::gen_keypair();
        return Account{ account_id : ClientIdPacket::new((sign_keys.0, asym_keys.0), (sign_keys.1, asym_keys.1)) };
    }

    ///
    /// Create a new `Account`
    ///
    pub fn create_account(username : &String, password : &[u8], pin : u32) -> Result<Account, ::MaidsafeError> {
        let new_account = Account::new();
        return Ok(new_account);
    }

    pub fn get_account(&self) -> &ClientIdPacket {
        return &self.account_id;
    }

    fn hash_pin(hasher : &mut ::crypto::sha2::Sha512, pin : u32) {
        use crypto::digest::Digest;
        use std::slice;
        unsafe {
            let address : *const u8 = mem::transmute(&pin);
            hasher.input(&slice::from_raw_parts(address, mem::size_of_val(&pin)));
        }
    }

    pub fn generate_network_id(username : &String, pin : u32) -> routing::name_type::NameType {
        use crypto::digest::Digest;

        let mut hasher = ::crypto::sha2::Sha512::new();
        let digest_size = hasher.output_bytes();

        let mut output1 = vec![0u8; digest_size];
        let mut output2 = vec![0u8; digest_size];

        hasher.input_str(&username);
        hasher.result(&mut output1);

        hasher.reset();
        Account::hash_pin(&mut hasher, pin);
        hasher.result(&mut output2);

        let mut name = [0u8; 64];
        hasher.reset();
        hasher.input(&output1);
        hasher.input(&output2);
        hasher.result(&mut name);
        return routing::name_type::NameType::new(name);
    }

    fn generate_crypto_keys(password : &[u8], pin : u32) -> (Vec<u8>, Vec<u8>) {
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

    pub fn encrypt(&self, password : &[u8], pin : u32) -> Result<Vec<u8>, ::MaidsafeError> {
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

    pub fn decrypt(encrypted : &[u8], password : &[u8], pin : u32) -> Result<Account, ::MaidsafeError> {
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

        return Ok(try!(Account::deserialise(&decrypted)))
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

impl Encodable for Account {
    fn encode<E: Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        return cbor::CborTagEncode::new(ACCOUNT_TAG, &self.account_id).encode(e);
    }
}

impl Decodable for Account {
    fn decode<D: Decoder>(d: &mut D) -> Result<Account, D::Error> {
        let tag : u64 = try!(Decodable::decode(d));

        // TODO : I'd like to check the tag value, and return an error ... but how?
        // The Error class is dependent on the generic decoder type, and the
        // functionality for returning an error that works with all types
        // doesn't seem possible. Slightly less fault tolerant now ...

        let account_id : ClientIdPacket = try!(Decodable::decode(d));
        return Ok(Account{ account_id : account_id });
    }
}

#[allow(dead_code)]
fn slice_eq(left : &[u8], right : &[u8]) -> bool {
    return left.iter().zip(right.iter()).all(|(a, b)| a == b);
}

// somewhat of a hack, but the crypto types didn't implement the eq trait
#[allow(dead_code)]
fn account_eq(left : &Account, right : &Account) -> bool {
    let data = "blah blah blah".to_string().into_bytes();
    let signed_result = slice_eq(&left.get_account().sign(&data), &right.get_account().sign(&data));

    let other_account = Account::new();
    let mut sealed1_result : bool;
    let mut sealed2_result : bool;
    {
        let sealed = other_account.get_account().encrypt(&data, &left.get_account().get_public_keys().1);
        let opened1 = left.get_account().decrypt(&sealed.0, &sealed.1, &other_account.get_account().get_public_keys().1);
        let opened2 = right.get_account().decrypt(&sealed.0, &sealed.1, &other_account.get_account().get_public_keys().1);
        sealed1_result = match opened1 {
            Ok(contents1) => match opened2 {
                Ok(contents2) => slice_eq(&contents1, &contents2),
                Err(_) => false
            },
            Err(_) => false
        }
    }
    {
        let sealed = other_account.get_account().encrypt(&data, &right.get_account().get_public_keys().1);
        let opened1 = left.get_account().decrypt(&sealed.0, &sealed.1, &other_account.get_account().get_public_keys().1);
        let opened2 = right.get_account().decrypt(&sealed.0, &sealed.1, &other_account.get_account().get_public_keys().1);
        sealed2_result = match opened1 {
            Ok(contents1) => match opened2 {
                Ok(contents2) => slice_eq(&contents1, &contents2),
                Err(_) => false
            },
            Err(_) => false
        }
    }
    return sealed2_result && sealed1_result && signed_result;
}

#[test]
fn generating_new_account() {
    let username = "James".to_string();
    let password = "Bond".as_bytes();
    let pin = 500u32;
    let account1 = Account::create_account(&username, &password, pin);
    let account2 = Account::create_account(&username, &password, pin);
    assert!(!account_eq(&account1.ok().unwrap(), &account2.ok().unwrap()));
}

fn generating_network_id() {
    let username1 = "user1".to_string();
    {
        let user1_id1 = Account::generate_network_id(&username1, 0);
        let user1_id2 = Account::generate_network_id(&username1, 1234);
        let user1_id3 = Account::generate_network_id(&username1, std::u32::MAX);

        assert!(!slice_eq(&user1_id1.get_id(), &user1_id2.get_id()));
        assert!(!slice_eq(&user1_id1.get_id(), &user1_id3.get_id()));
        assert!(!slice_eq(&user1_id2.get_id(), &user1_id3.get_id()));
        assert!(slice_eq(&user1_id1.get_id(), &Account::generate_network_id(&username1, 0).get_id()));
        assert!(slice_eq(&user1_id2.get_id(), &Account::generate_network_id(&username1, 1234).get_id()));
        assert!(slice_eq(&user1_id3.get_id(), &Account::generate_network_id(&username1, std::u32::MAX).get_id()));
    }
    {
        let username2 = "user2".to_string();
        assert!(
            !slice_eq(
                &Account::generate_network_id(&username1, 248).get_id(),
                &Account::generate_network_id(&username2, 248).get_id()));
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
    let account = Account::new();
    match account.serialise() {
        Ok(serialised) => match Account::deserialise(&serialised) {
            Ok(account_again) => assert!(account_eq(&account, &account_again)),
            Err(_) => assert!(false)
        },
        Err(_) => assert!(false)
    }
}

#[test]
fn encryption() {
    let account = Account::new();
    let password = "impossible to guess".to_string().into_bytes();
    let pin = 10000u32;
    match account.encrypt(&password, pin) {
        Ok(encrypted) => {
            assert!(encrypted.len() > 0);
            assert!(account.serialise().map(|serialised| assert!(encrypted != serialised)).is_ok());
            match Account::decrypt(&encrypted, &password, pin) {
                Ok(account_again) => assert!(account_eq(&account, &account_again)),
                Err(_) => assert!(false)
            }
        }
        Err(_) => assert!(false)
    }
}
