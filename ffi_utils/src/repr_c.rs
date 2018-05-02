// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! FFI tools

/// Trait to convert between FFI and Rust representations of types
pub trait ReprC {
    /// C representation of the type
    type C;
    /// Error type
    type Error;

    /// Converts from a raw type into an owned type by cloning data
    unsafe fn clone_from_repr_c(c_repr: Self::C) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl ReprC for u64 {
    type C = u64;
    type Error = ();

    unsafe fn clone_from_repr_c(c_repr: Self::C) -> Result<Self, Self::Error> {
        Ok(c_repr)
    }
}

impl ReprC for u32 {
    type C = u32;
    type Error = ();

    unsafe fn clone_from_repr_c(c_repr: Self::C) -> Result<Self, Self::Error> {
        Ok(c_repr)
    }
}

impl ReprC for usize {
    type C = usize;
    type Error = ();

    unsafe fn clone_from_repr_c(c_repr: Self::C) -> Result<Self, Self::Error> {
        Ok(c_repr)
    }
}

impl<T> ReprC for *const T {
    type C = *const T;
    type Error = ();

    unsafe fn clone_from_repr_c(c_repr: Self::C) -> Result<Self, Self::Error> {
        Ok(c_repr)
    }
}

impl<T> ReprC for *mut T {
    type C = *mut T;
    type Error = ();

    unsafe fn clone_from_repr_c(c_repr: Self::C) -> Result<Self, Self::Error> {
        Ok(c_repr)
    }
}

impl ReprC for [u8; 24] {
    type C = *const [u8; 24];
    type Error = ();

    unsafe fn clone_from_repr_c(c_repr: Self::C) -> Result<Self, Self::Error> {
        Ok(*c_repr)
    }
}

impl ReprC for [u8; 32] {
    type C = *const [u8; 32];
    type Error = ();

    unsafe fn clone_from_repr_c(c_repr: Self::C) -> Result<Self, Self::Error> {
        Ok(*c_repr)
    }
}

impl ReprC for [u8; 64] {
    type C = *const [u8; 64];
    type Error = ();

    unsafe fn clone_from_repr_c(c_repr: Self::C) -> Result<Self, Self::Error> {
        Ok(*c_repr)
    }
}
