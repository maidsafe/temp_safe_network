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

#![allow(unused, dead_code, missing_docs)]

pub const MAX_STRUCTURED_DATA_SIZE_IN_BYTES: usize = 102400;

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub enum ImmutableDataType {
    Normal,
    Backup,
    Sacrificaial,
}

#[derive(Clone, RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub struct ImmutableData {
    type_tag: ImmutableDataType,
    value: Vec<u8>,
}

impl ImmutableData {
    /// Creates a new instance of ImmutableData
    pub fn new(type_tag: ImmutableDataType, value: Vec<u8>) -> ImmutableData {
        ImmutableData {
            type_tag: type_tag,
            value: value,
        }
    }

    /// Returns the value
    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }

    /// Returns the value
    pub fn get_tag_type(&self) -> &ImmutableDataType {
        &self.type_tag
    }

    /// Returns name ensuring invariant
    pub fn name(&self) -> ::routing::NameType {
        let digest = ::sodiumoxide::crypto::hash::sha512::hash(&self.value);
        match self.type_tag {
            ImmutableDataType::Normal       => ::routing::NameType(digest.0),
            ImmutableDataType::Backup       => ::routing::NameType(::sodiumoxide::crypto::hash::sha512::hash(&digest.0).0),
            ImmutableDataType::Sacrificaial => ::routing::NameType(::sodiumoxide::crypto::hash::sha512::hash(&::sodiumoxide::crypto::hash::sha512::hash(&digest.0).0).0)
        }
    }
}

#[derive(Clone)]
pub struct StructuredData {
    tag_type: u64,
    identifier: ::routing::NameType,
    version: u64,
    data: Vec<u8>,
    current_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
    previous_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
    previous_owner_signatures: Vec<::sodiumoxide::crypto::sign::Signature>,
}

impl StructuredData {
    pub fn new(tag_type: u64,
               identifier: ::routing::NameType,
               version: u64,
               data: Vec<u8>,
               current_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
               previous_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
               sign_key: &::sodiumoxide::crypto::sign::SecretKey) -> StructuredData {
        let mut structured_data = StructuredData {
            tag_type: tag_type,
            identifier: identifier,
            version: version,
            data: data,
            current_owner_keys: current_owner_keys,
            previous_owner_keys: previous_owner_keys,
            previous_owner_signatures: vec![],
        };

        structured_data.add_signature(sign_key);
        structured_data
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_identifier(&self) -> &::routing::NameType {
        &self.identifier
    }

    pub fn get_version(&self) -> u64 {
        self.version
    }

    pub fn get_signatures(&self) -> &Vec<::sodiumoxide::crypto::sign::Signature> {
        &self.previous_owner_signatures
    }

    pub fn get_owners(&self) -> &Vec<::sodiumoxide::crypto::sign::PublicKey> {
        &self.current_owner_keys
    }

    pub fn get_previous_owners(&self) -> &Vec<::sodiumoxide::crypto::sign::PublicKey> {
        &self.previous_owner_keys
    }

    pub fn add_signature(&mut self, sign_key: &::sodiumoxide::crypto::sign::SecretKey) {
        let signable = self.data_to_sign();
        self.previous_owner_signatures.push(::sodiumoxide::crypto::sign::sign_detached(&signable[..], sign_key));
    }

    pub fn data_to_sign(&self) -> Vec<u8> {
        self.data.iter().chain(self.version.to_string().as_bytes().iter().chain(
                self.current_owner_keys.iter().fold(
                    Vec::<u8>::new(), |mut key_vec, key| { key_vec.extend(key.0.iter().map(|a| *a).collect::<Vec<u8>>()); key_vec }
                    ).iter().chain(
                        self.previous_owner_keys.iter().fold(
                            Vec::<u8>::new(), |mut key_vec, key| { key_vec.extend(key.0.iter().map(|a| *a).collect::<Vec<u8>>()); key_vec }
                            ).iter()))).map(|a| *a).collect::<Vec<u8>>()
    }

    pub fn get_tag_type(&self) -> u64 {
        self.tag_type
    }

    pub fn name(&self) -> ::routing::NameType {
        StructuredData::compute_name(self.tag_type, &self.identifier)
    }

    pub fn compute_name(tag_type: u64, identifier: &::routing::NameType) -> ::routing::NameType {
        use ::sodiumoxide::crypto::hash::sha512::hash;
        ::routing::NameType(
            hash(&hash(&identifier.0).0.iter().chain(tag_type.to_string().as_bytes().iter()).map(|a| *a).collect::<Vec<u8>>()[..]).0)
    }

    pub fn replace_signatures(&mut self, new_signatures: Vec<::sodiumoxide::crypto::sign::Signature>) {
        self.previous_owner_signatures = new_signatures;
    }

}

impl ::rustc_serialize::Encodable for StructuredData {
    fn encode<E: ::rustc_serialize::Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        let mut curr_owner = Vec::<Vec<u8>>::new();
        for it in self.current_owner_keys.iter() {
            curr_owner.push(it.0.iter().map(|a| *a).collect());
        }

        let mut prev_owner = Vec::<Vec<u8>>::new();
        for it in self.previous_owner_keys.iter() {
            prev_owner.push(it.0.iter().map(|a| *a).collect());
        }

        let mut previous_owner_signatures = Vec::<Vec<u8>>::new();
        for it in self.previous_owner_signatures.iter() {
            previous_owner_signatures.push(it.0.iter().map(|a| *a).collect());
        }

        ::cbor::CborTagEncode::new(100_001, &(&self.identifier,
                                           self.tag_type,
                                           self.version,
                                           &self.data,
                                           curr_owner,
                                           prev_owner,
                                           previous_owner_signatures)).encode(e)
    }
}

impl ::rustc_serialize::Decodable for StructuredData {
    fn decode<D: ::rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        try!(d.read_u64());

        let (identifier,
             tag_type,
             version,
             data,
             curr_owner,
             previous_owner_keys,
             previous_owner_signatures):
            (::routing::NameType,
             u64,
             u64,
             Vec<u8>,
             Vec<Vec<u8>>,
             Vec<Vec<u8>>,
             Vec<Vec<u8>>) = try!(::rustc_serialize::Decodable::decode(d));

        let mut vec_current_owner = Vec::<::sodiumoxide::crypto::sign::PublicKey>::new();
        for it in curr_owner.iter() {
            let mut arr_current = [0u8; 32];
            for it_inner in it.iter().enumerate() {
                arr_current[it_inner.0] = *it_inner.1;
            }

            vec_current_owner.push(::sodiumoxide::crypto::sign::PublicKey(arr_current));
        }

        let mut vec_prev_owner = Vec::<::sodiumoxide::crypto::sign::PublicKey>::new();
        for it in previous_owner_keys.iter() {
            let mut arr_current = [0u8; 32];
            for it_inner in it.iter().enumerate() {
                arr_current[it_inner.0] = *it_inner.1;
            }

            vec_prev_owner.push(::sodiumoxide::crypto::sign::PublicKey(arr_current));
        }

        let mut signatures_decoded = Vec::<::sodiumoxide::crypto::sign::Signature>::new();
        for it in previous_owner_signatures.iter() {
            let mut arr_current = [0u8; 64];
            for it_inner in it.iter().enumerate() {
                arr_current[it_inner.0] = *it_inner.1;
            }

            signatures_decoded.push(::sodiumoxide::crypto::sign::Signature(arr_current));
        }

        Ok(StructuredData {
            tag_type: tag_type,
            identifier: identifier,
            version: version,
            data: data,
            current_owner_keys: vec_current_owner,
            previous_owner_keys: vec_prev_owner,
            previous_owner_signatures: signatures_decoded,
        })
    }
}

impl ::std::cmp::PartialEq for StructuredData {
    fn eq(&self, other: &StructuredData) -> bool {
        let lhs_signable = self.data_to_sign();
        let rhs_signable = other.data_to_sign();

        lhs_signable == rhs_signable &&
            self.tag_type == other.tag_type &&
            self.identifier == other.identifier
    }
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub enum DataRequest {
    StructuredData(u64),
    ImmutableData(ImmutableDataType),
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub enum Data {
    StructuredData(StructuredData),
    ImmutableData(ImmutableData),
    ShutDown,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialisation_structured_data() {
        let data = vec![99u8; 100];
        let (public_key, secret_key) = ::sodiumoxide::crypto::sign::gen_keypair();

        let sd0 = StructuredData::new(0,
                                      ::routing::NameType::new([0u8; 64]),
                                      0,
                                      data.clone(),
                                      vec![public_key.clone()],
                                      vec![public_key.clone()],
                                      &secret_key);

        let sd1 = StructuredData::new(0,
                                      ::routing::NameType::new([0u8; 64]),
                                      0,
                                      data,
                                      vec![public_key.clone()],
                                      Vec::new(),
                                      &secret_key);

        let mut encoder0 = ::cbor::Encoder::from_memory();
        encoder0.encode(&[&(sd0)]).unwrap();

        let mut decoder0 = ::cbor::Decoder::from_bytes(encoder0.into_bytes());
        let sd0_decoded: StructuredData = decoder0.decode().next().unwrap().unwrap();

        assert!(sd0_decoded == sd0);

        let mut encoder1 = ::cbor::Encoder::from_memory();
        encoder1.encode(&[&(sd1)]).unwrap();

        let mut decoder1 = ::cbor::Decoder::from_bytes(encoder1.into_bytes());
        let sd1_decoded: StructuredData = decoder1.decode().next().unwrap().unwrap();

        assert!(sd1_decoded == sd1);
        assert!(sd1_decoded != sd0_decoded && sd1 != sd0);
    }

    #[test]
    fn serialisation_immutable_data() {
        let data = vec![99u8; 100];

        let id0 = ImmutableData::new(ImmutableDataType::Normal, data.clone());

        let mut encoder0 = ::cbor::Encoder::from_memory();
        encoder0.encode(&[&(id0)]).unwrap();

        let mut decoder0 = ::cbor::Decoder::from_bytes(encoder0.into_bytes());
        let id0_decoded: ImmutableData = decoder0.decode().next().unwrap().unwrap();

        assert!(id0_decoded == id0);

        let id1 = ImmutableData::new(ImmutableDataType::Backup, data.clone());

        let mut encoder1 = ::cbor::Encoder::from_memory();
        encoder1.encode(&[&(id1)]).unwrap();

        let mut decoder1 = ::cbor::Decoder::from_bytes(encoder1.into_bytes());
        let id1_decoded: ImmutableData = decoder1.decode().next().unwrap().unwrap();

        assert!(id1_decoded == id1);
        assert!(id0 != id1 && id0_decoded != id1_decoded);
    }

    #[test]
    fn serialisation_data() {
        let data = vec![99u8; 100];
        let (public_key, secret_key) = ::sodiumoxide::crypto::sign::gen_keypair();

        let sd0 = StructuredData::new(0,
                                      ::routing::NameType::new([0u8; 64]),
                                      0,
                                      data.clone(),
                                      vec![public_key.clone()],
                                      vec![public_key.clone()],
                                      &secret_key);

        let data0 = Data::StructuredData(sd0.clone());

        let mut encoder0 = ::cbor::Encoder::from_memory();
        encoder0.encode(&[&(data0)]).unwrap();

        let mut decoder0 = ::cbor::Decoder::from_bytes(encoder0.into_bytes());
        let data0_decoded: Data = decoder0.decode().next().unwrap().unwrap();

        if let Data::StructuredData(sd0_decoded) = data0_decoded {
            assert!(sd0 == sd0_decoded);
        } else {
            panic!("Unexpected");
        }

        let id0 = ImmutableData::new(ImmutableDataType::Normal, data);

        let data1 = Data::ImmutableData(id0.clone());

        encoder0 = ::cbor::Encoder::from_memory();
        encoder0.encode(&[&(data1)]).unwrap();

        decoder0 = ::cbor::Decoder::from_bytes(encoder0.into_bytes());
        let data1_decoded: Data = decoder0.decode().next().unwrap().unwrap();

        if let Data::ImmutableData(id0_decoded) = data1_decoded {
            assert!(id0 == id0_decoded);
        } else {
            panic!("Unexpected");
        }
    }
}
