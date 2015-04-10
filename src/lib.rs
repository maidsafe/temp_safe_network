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
#![crate_name = "maidsafe_client"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://maidsafe.net/img/Resources/branding/maidsafe_logo.fab2.png",
       html_favicon_url = "http://maidsafe.net/img/favicon.ico",
              html_root_url = "http://dirvine.github.io/dirvine/maidsafe_client/")]
#![allow(dead_code)]

extern crate cbor;
extern crate crypto;
extern crate maidsafe_types;
extern crate routing;

pub mod account;
mod client;

use std::sync::{Mutex, Arc};

use routing::routing_client::RoutingClient;
use routing::types::DhtId;
use client::ClientFacade;
use account::Account;


pub enum CryptoError {
    SymmetricCryptoError(crypto::symmetriccipher::SymmetricCipherError),
    BadBuffer
}

impl From<crypto::symmetriccipher::SymmetricCipherError> for CryptoError {
    fn from(error : crypto::symmetriccipher::SymmetricCipherError) -> CryptoError {
        return CryptoError::SymmetricCryptoError(error);
    }
}


pub enum MaidsafeError {
    CryptoError(CryptoError),
    EncodingError(cbor::CborError),
}

impl From<CryptoError> for MaidsafeError {
    fn from(error : CryptoError) -> MaidsafeError {
        return MaidsafeError::CryptoError(error);
    }
}

impl From<cbor::CborError> for MaidsafeError {
    fn from(error : cbor::CborError) -> MaidsafeError {
        return MaidsafeError::EncodingError(error);
    }
}

impl From<crypto::symmetriccipher::SymmetricCipherError> for MaidsafeError {
    fn from(error : crypto::symmetriccipher::SymmetricCipherError) -> MaidsafeError {
        return MaidsafeError::CryptoError(CryptoError::SymmetricCryptoError(error));
    }
}



pub struct Client<'a> {
  my_routing : RoutingClient<'a ClientFacade>,
  my_account : Account
}

impl<'a> Client<'a> {
  pub fn new(username : &String, password : &[u8], pin : u32) -> Client<'a> {
    let account = Account::create_account(username, password, pin).ok().unwrap();
    let mut client = Client { my_routing: RoutingClient::new(Arc::new(Mutex::new(ClientFacade::new())),
                                                             account.get_maid().clone(), DhtId::generate_random()),
                              my_account: account };
    let encrypted = client.my_account.encrypt(&password, pin);
    let network_id = Account::generate_network_id(&username, pin);
    client.my_routing.put(DhtId::new(network_id.0), encrypted.ok().unwrap());
    client
  }

  pub fn log_in(username : &String, password : &[u8], pin : u32) -> Client<'a> {
    let network_id = Account::generate_network_id(username, pin);
    let temp_account = Account::new();
    let mut temp_routing = RoutingClient::new(Arc::new(Mutex::new(ClientFacade::new())),
                                              temp_account.get_maid().clone(), DhtId::generate_random());
    temp_routing.get(0u64, DhtId::new(network_id.0));
    // TODO here we have to wait for a get_response, but how the notification come in ?
    let encrypted = [5u8, 1024];
    let existing_account = Account::decrypt(&encrypted, &password, pin).ok().unwrap();
    Client { my_routing: RoutingClient::new(Arc::new(Mutex::new(ClientFacade::new())),
                                            existing_account.get_maid().clone(), DhtId::generate_random()),
             my_account: existing_account }
  }

}
