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
use cbor::CborTagEncode;
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

#[derive(Eq)]
pub struct ContainerId([u8;ID_LEN]);

impl ContainerId {
    pub fn new() -> ContainerId {
        ContainerId([1u8;ID_LEN])
    }
}
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
