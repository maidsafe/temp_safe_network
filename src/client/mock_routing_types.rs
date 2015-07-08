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

#![allow(unused, dead_code)]

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub enum Data {
    StructuredData(StructuredData),
    ImmutableData(ImmutableData),
}

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
    identifier: ::routing::NameType,//::sodiumoxide::crypto::hash::sha512::Digest,
    version: u64,
    data: Vec<u8>,
    current_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
    previous_owner_keys: Option<Vec<::sodiumoxide::crypto::sign::PublicKey>>,
    signatures: Vec<::sodiumoxide::crypto::sign::Signature>,
}

impl StructuredData {
    pub fn new(tag_type: u64,
               identifier: ::routing::NameType,//::sodiumoxide::crypto::hash::sha512::Digest,
               version: u64,
               data: Vec<u8>,
               current_owner_keys: Vec<::sodiumoxide::crypto::sign::PublicKey>,
               previous_owner_keys: Option<Vec<::sodiumoxide::crypto::sign::PublicKey>>,
               sign_key: &::sodiumoxide::crypto::sign::SecretKey) -> StructuredData {
        let mut structured_data = StructuredData {
            tag_type: tag_type,
            identifier: identifier,
            version: version,
            data: data,
            current_owner_keys: current_owner_keys,
            previous_owner_keys: previous_owner_keys,
            signatures: vec![],
        };

        structured_data.add_signature(sign_key);
        structured_data
    }

    pub fn add_signature(&mut self, sign_key: &::sodiumoxide::crypto::sign::SecretKey) {
        let signable = self.get_signable_data();
        self.signatures.push(::sodiumoxide::crypto::sign::sign_detached(&signable[..], sign_key));
    }

    pub fn get_signable_data(&self) -> Vec<u8> {
        let mut vec = self.data.iter().chain( self.version.to_string().as_bytes().iter().chain(
                self.current_owner_keys.iter().fold(
                    Vec::<u8>::new(), |mut key_vec, key| { key_vec.extend(key.0.iter().map(|a| *a).collect::<Vec<u8>>()); key_vec }
                    ).iter())).map(|a| *a).collect::<Vec<u8>>();

        if let Some(ref previous_owner_keys) = self.previous_owner_keys {
            vec.extend(previous_owner_keys.iter().fold(Vec::<u8>::new(), |mut key_vec, key| { key_vec.extend(key.0.iter().map(|a| *a).collect::<Vec<u8>>()); key_vec }));
        }

        vec
    }

    pub fn name(&self) -> ::routing::NameType {
        use ::sodiumoxide::crypto::hash::sha512::hash;
        ::routing::NameType(
            hash(&hash(&self.identifier.0).0.iter().chain(self.tag_type.to_string().as_bytes().iter()).map(|a| *a).collect::<Vec<u8>>()[..]).0)
    }
}

impl ::rustc_serialize::Encodable for StructuredData {
    fn encode<E: ::rustc_serialize::Encoder>(&self, e: &mut E) -> Result<(), E::Error> {
        let mut curr_owner = Vec::<Vec<u8>>::new();
        for it in self.current_owner_keys.iter() {
            curr_owner.push(it.0.iter().map(|a| *a).collect());
        }

        let opt_prev_owner_keys: Option<Vec<Vec<u8>>>;
        if let Some(ref previous_owner_keys) = self.previous_owner_keys {
            let mut prev_owner = Vec::<Vec<u8>>::new();
            for it in previous_owner_keys.iter() {
                prev_owner.push(it.0.iter().map(|a| *a).collect());
            }

            opt_prev_owner_keys = Some(prev_owner);
        } else {
            opt_prev_owner_keys = None;
        }

        let mut signatures = Vec::<Vec<u8>>::new();
        for it in self.signatures.iter() {
            signatures.push(it.0.iter().map(|a| *a).collect());
        }

        ::cbor::CborTagEncode::new(100_001, &(&self.identifier,
                                           self.tag_type,
                                           self.version,
                                           &self.data,
                                           curr_owner,
                                           opt_prev_owner_keys,
                                           signatures)).encode(e)
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
             opt_prev_owner_keys,
             signatures):
            (::routing::NameType,
             u64,
             u64,
             Vec<u8>,
             Vec<Vec<u8>>,
             Option<Vec<Vec<u8>>>,
             Vec<Vec<u8>>) = try!(::rustc_serialize::Decodable::decode(d));

        let mut vec_current_owner = Vec::<::sodiumoxide::crypto::sign::PublicKey>::new();
        for it in curr_owner.iter() {
            let mut arr_current = [0u8; 32];
            for it_inner in it.iter().enumerate() {
                arr_current[it_inner.0] = *it_inner.1;
            }

            vec_current_owner.push(::sodiumoxide::crypto::sign::PublicKey(arr_current));
        }

        let opt_prev_owner_keys_decoded: Option<Vec<::sodiumoxide::crypto::sign::PublicKey>>;
        if let Some(previous_owner_keys) = opt_prev_owner_keys {
            let mut vec_prev_owner = Vec::<::sodiumoxide::crypto::sign::PublicKey>::new();
            for it in previous_owner_keys.iter() {
                let mut arr_current = [0u8; 32];
                for it_inner in it.iter().enumerate() {
                    arr_current[it_inner.0] = *it_inner.1;
                }

                vec_prev_owner.push(::sodiumoxide::crypto::sign::PublicKey(arr_current));
            }

            opt_prev_owner_keys_decoded = Some(vec_prev_owner);
        } else {
            opt_prev_owner_keys_decoded = None;
        }

        let mut signatures_decoded = Vec::<::sodiumoxide::crypto::sign::Signature>::new();
        for it in signatures.iter() {
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
            previous_owner_keys: opt_prev_owner_keys_decoded,
            signatures: signatures_decoded,
        })
    }
}

impl ::std::cmp::PartialEq for StructuredData {
    fn eq(&self, other: &StructuredData) -> bool {
        let lhs_signable = self.get_signable_data();
        let rhs_signable = other.get_signable_data();

        lhs_signable == rhs_signable &&
            self.tag_type == other.tag_type &&
            self.identifier == other.identifier
    }
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
                                  Some(vec![public_key.clone()]),
                                  &secret_key);

        let sd1 = StructuredData::new(0,
                                  ::routing::NameType::new([0u8; 64]),
                                  0,
                                  data,
                                  vec![public_key.clone()],
                                  None,
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
                                  Some(vec![public_key.clone()]),
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
