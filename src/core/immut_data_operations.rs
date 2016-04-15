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

use std::sync::{Arc, Mutex};

use core::utility;
use xor_name::XorName;
use core::client::Client;
use core::errors::CoreError;
use core::SelfEncryptionStorage;
use self_encryption::{DataMap, SelfEncryptor};
use sodiumoxide::crypto::box_::{PublicKey, SecretKey, Nonce};
use maidsafe_utilities::serialisation::{serialise, deserialise};
use routing::{Data, DataRequest, ImmutableData, ImmutableDataType};

// TODO(Spandan) Ask Routing to define this constant and use it from there
const MAX_IMMUT_DATA_SIZE_IN_BYTES: usize = 1024 * 1024;

#[derive(RustcEncodable, RustcDecodable)]
enum DataTypeEncoding {
    SerialisedDataMap(Vec<u8>),
    DataMap(DataMap),
}

/// Create and obtain immutable data out of the given raw data. The API will encrypt the right
/// content if the keys are provided and will ensure the max immutable data chunk size is
/// respected.
pub fn create(client: Arc<Mutex<Client>>,
              data: Vec<u8>,
              encryption_keys: Option<(&PublicKey, &SecretKey, &Nonce)>)
              -> Result<ImmutableData, CoreError> {
    let mut se = SelfEncryptor::new(SelfEncryptionStorage::new(client.clone()), DataMap::None);
    se.write(&data, 0);
    let mut data_map = se.close();

    let serialised_dm = try!(serialise(&data_map));
    let mut immut_data = if let Some((pk, sk, nonce)) = encryption_keys {
        let cipher_text = try!(utility::hybrid_encrypt(&serialised_dm, nonce, pk, sk));
        let encoded_cipher_text = try!(serialise(&DataTypeEncoding::SerialisedDataMap(cipher_text)));
        ImmutableData::new(ImmutableDataType::Normal, encoded_cipher_text)
    } else {
        let encoded_plain_text = try!(serialise(&DataTypeEncoding::SerialisedDataMap(serialised_dm)));
        ImmutableData::new(ImmutableDataType::Normal, encoded_plain_text)
    };

    let mut serialised_id = try!(serialise(&immut_data));
    while serialised_id.len() > MAX_IMMUT_DATA_SIZE_IN_BYTES {
        let mut se = SelfEncryptor::new(SelfEncryptionStorage::new(client.clone()), DataMap::None);
        se.write(&serialised_id, 0);
        data_map = se.close();
        let encoded_dm = try!(serialise(&DataTypeEncoding::DataMap(data_map)));
        immut_data = ImmutableData::new(ImmutableDataType::Normal, encoded_dm);
        serialised_id = try!(serialise(&immut_data));
    }

    Ok(immut_data)
}

/// Get actual data from ImmutableData created via create() function in this module.
pub fn get_data(client: Arc<Mutex<Client>>,
                immut_data_name: XorName,
                decryption_keys: Option<(&PublicKey, &SecretKey, &Nonce)>)
                -> Result<Vec<u8>, CoreError> {
    let data_req = DataRequest::Immutable(immut_data_name, ImmutableDataType::Normal);
    let resp_getter = try!(unwrap_result!(client.lock()).get(data_req, None));

    match try!(resp_getter.get()) {
        Data::Immutable(mut id) => {
            while let Ok(DataTypeEncoding::DataMap(dm)) = deserialise(&id.value()) {
                let mut se = SelfEncryptor::new(SelfEncryptionStorage::new(client.clone()), dm);
                let length = se.len();
                id = try!(deserialise(&se.read(0, length)));
            }

            match try!(deserialise(&id.value())) {
                DataTypeEncoding::SerialisedDataMap(encoded_dm) => {
                    let dm = if let Some((pk, sk, nonce)) = decryption_keys {
                        let plain_text = try!(utility::hybrid_decrypt(&encoded_dm, nonce, pk, sk));
                        try!(deserialise(&plain_text))
                    } else {
                        try!(deserialise(&encoded_dm))
                    };

                    let mut se = SelfEncryptor::new(SelfEncryptionStorage::new(client.clone()), dm);
                    let length = se.len();
                    Ok(se.read(0, length))
                }
                _ => Err(CoreError::ReceivedUnexpectedData),
            }
        }
        _ => Err(CoreError::ReceivedUnexpectedData),
    }
}
