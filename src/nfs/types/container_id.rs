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
const ID_LEN: usize = 512;
use std::cmp::*;
use std::fmt;
use cbor::CborTagEncode;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use rand::random;

#[derive(Eq)]
pub struct ContainerId([u8;ID_LEN]);

/// Convert a container of `u8`s to an array.  If the container is not the exact size specified,
/// `None` is returned.  Otherwise, all of the elements are moved into the array.
///
/// ## Examples
///
/// ```
/// # #[macro_use] extern crate routing;
/// # fn main() {
/// let mut data = Vec::<u8>::new();
/// data.push(1);
/// data.push(2);
/// let data_copy = data.clone();
/// assert!(container_of_u8_to_array!(data, 2).is_some());
/// assert!(container_of_u8_to_array!(data_copy, 3).is_none());
/// # }
/// ```
macro_rules! container_of_u8_to_array {
    ($container:ident, $size:expr) => {{
        if $container.len() != $size {
            None
        } else {
            let mut arr = [0u8; $size];
            for element in $container.into_iter().enumerate() {
                arr[element.0] = element.1;
            }
            Some(arr)
        }
    }};
}

impl ContainerId {
    pub fn new() -> ContainerId {
        let mut vec = Vec::with_capacity(ID_LEN);
        for _ in (0..ID_LEN) {
            vec.push(random::<u8>());
        }
        ContainerId(container_of_u8_to_array!(vec, ID_LEN).unwrap())
    }
}

fn slice_equal<T: PartialEq>(lhs: &[T], rhs: &[T]) -> bool {
    lhs.len() == rhs.len() && lhs.iter().zip(rhs.iter()).all(|(a, b)| a == b)
}

impl Clone for ContainerId {
    fn clone(&self) -> Self {
        let mut arr_cloned = [0u8; ID_LEN];
        let &ContainerId(arr_self) = self;

        for i in 0..arr_self.len() {
            arr_cloned[i] = arr_self[i];
        }

        ContainerId(arr_cloned)
    }
}

impl PartialEq for ContainerId {
    fn eq(&self, other: &ContainerId) -> bool {
        slice_equal(&self.0, &other.0)
    }
}

impl Ord for ContainerId {
    #[inline]
    fn cmp(&self, other : &ContainerId) -> Ordering {
        Ord::cmp(&&self.0[..], &&other.0[..])
    }
}

impl PartialOrd for ContainerId {
    #[inline]
    fn partial_cmp(&self, other : &ContainerId) -> Option<Ordering> {
        PartialOrd::partial_cmp(&&self.0[..], &&other.0[..])
    }
    #[inline]
    fn lt(&self, other : &ContainerId) -> bool {
        PartialOrd::lt(&&self.0[..], &&other.0[..])
    }
    #[inline]
    fn le(&self, other : &ContainerId) -> bool {
        PartialOrd::le(&&self.0[..], &&other.0[..])
    }
    #[inline]
    fn gt(&self, other : &ContainerId) -> bool {
        PartialOrd::gt(&&self.0[..], &&other.0[..])
    }
    #[inline]
    fn ge(&self, other : &ContainerId) -> bool {
        PartialOrd::ge(&&self.0[..], &&other.0[..])
    }
}

impl Encodable for ContainerId {
    fn encode<E: Encoder>(&self, e: &mut E)->Result<(), E::Error> {
        CborTagEncode::new(5483_000, &(self.0.as_ref())).encode(e)
    }
}

impl Decodable for ContainerId {
    fn decode<D: Decoder>(d: &mut D)->Result<ContainerId, D::Error> {
        try!(d.read_u64());
        let id : Vec<u8> = try!(Decodable::decode(d));
        match container_of_u8_to_array!(id, ID_LEN) {
            Some(id_arr) => Ok(ContainerId(id_arr)),
            None => Err(d.error("Bad NameType size"))
        }
    }
}

fn id_to_vec(id: &[u8;ID_LEN]) -> Vec<u8>{
    let mut vec = Vec::with_capacity(ID_LEN);
    for i in &id[..] {
        vec.push(*i);
    }
    vec
}

impl fmt::Debug for ContainerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", id_to_vec(&self.0))
    }
}

impl fmt::Display for ContainerId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", id_to_vec(&self.0))
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use cbor;
    #[test]
    fn equality() {
        let id = ContainerId::new();
        let id_cloned = id.clone();
        let id_sec = ContainerId::new();
        assert_eq!(id, id_cloned);
        assert!(id != id_sec);
        assert!(id == id_cloned);
    }

    #[test]
    fn serialise() {
        let obj_before = ContainerId::new();

        let mut e = cbor::Encoder::from_memory();
        e.encode(&[&obj_before]).unwrap();

        let mut d = cbor::Decoder::from_bytes(e.as_bytes());
        let obj_after: ContainerId = d.decode().next().unwrap().unwrap();

        assert_eq!(obj_before, obj_after);
    }
}
