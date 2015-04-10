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

use std::sync::{Mutex, Arc, Condvar};
use std::io::Error as IoError;

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
  my_account : Account,
  my_facade : Arc<Mutex<ClientFacade>>,
  my_cvar : Arc<(Mutex<bool>, Condvar)>
}

impl<'a> Client<'a> {
  pub fn new(username : &String, password : &[u8], pin : u32) -> Client<'a> {
    let account = Account::create_account(username, password, pin).ok().unwrap();
    let cvar = Arc::new((Mutex::new(false), Condvar::new()));
    let facade = Arc::new(Mutex::new(ClientFacade::new(cvar.clone())));
    let mut client = Client { my_routing: RoutingClient::new(facade.clone(), account.get_maid().clone(), DhtId::generate_random()),
                              my_account: account, my_facade: facade, my_cvar: cvar };
    let encrypted = client.my_account.encrypt(&password, pin);
    let network_id = Account::generate_network_id(&username, pin);
    client.my_routing.put(DhtId::new(network_id.0), encrypted.ok().unwrap());
    client
  }

  pub fn log_in(username : &String, password : &[u8], pin : u32) -> Client<'a> {
    let mut fetched_encrypted : Vec<u8>;
    {
      let network_id = Account::generate_network_id(username, pin);
      let temp_account = Account::new();
      let temp_cvar = Arc::new((Mutex::new(false), Condvar::new()));
      let temp_facade = Arc::new(Mutex::new(ClientFacade::new(temp_cvar.clone())));
      let mut temp_routing = RoutingClient::new(temp_facade.clone(), temp_account.get_maid().clone(), DhtId::generate_random());
      let get_queue = temp_routing.get(0u64, DhtId::new(network_id.0));
      let &(ref lock, ref cvar) = &*temp_cvar;
      let mut fetched = lock.lock().unwrap();
      while !*fetched {
          fetched = cvar.wait(fetched).unwrap();
      }
      let &ref facade_lock = &*temp_facade;
      let mut facade = facade_lock.lock().unwrap();
      let result = facade.get_response(get_queue.ok().unwrap());
      fetched_encrypted = result.ok().unwrap();
    }
    let existing_account = Account::decrypt(&fetched_encrypted[..], &password, pin).ok().unwrap();
    let cvar = Arc::new((Mutex::new(false), Condvar::new()));
    let facade = Arc::new(Mutex::new(ClientFacade::new(cvar.clone())));
    Client { my_routing: RoutingClient::new(facade.clone(), existing_account.get_maid().clone(), DhtId::generate_random()),
             my_account: existing_account, my_facade: facade, my_cvar: cvar }
  }

  pub fn put(&mut self, data: Vec<u8>) {
    self.my_routing.put(DhtId::new(self.my_account.get_maid().get_name().0), data);
  }

  pub fn get(&mut self, data_name: DhtId) -> Result<Vec<u8>, IoError> {
    let get_queue = self.my_routing.get(0u64, data_name);
    let &(ref lock, ref cvar) = &*self.my_cvar;
    let mut fetched = lock.lock().unwrap();
    while !*fetched {
        fetched = cvar.wait(fetched).unwrap();
    }
    let &ref facade_lock = &*self.my_facade;
    let mut facade = facade_lock.lock().unwrap();
    let result = facade.get_response(get_queue.ok().unwrap());
    *fetched = false;
    Ok(result.ok().unwrap())
  }

}
