// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{utils, Error, PublicKey, XorName};
use bincode::serialized_size;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{self, Debug, Formatter},
    u64,
};

/// Maximum allowed size for a serialised Chunk to grow to.
pub const MAX_CHUNK_SIZE_IN_BYTES: u64 = 1024 * 1024 + 10 * 1024;

/// Private Chunk: an immutable chunk of data which can be deleted. Can only be fetched
/// by the listed owner.
#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Clone)]
pub struct PrivateChunk {
    /// Network address. Omitted when serialising and calculated from the `value` and `owner` when
    /// deserialising.
    address: Address,
    /// Contained chunk.
    value: Vec<u8>,
    /// Contains a set of owners of this chunk.
    owner: PublicKey,
}

impl PrivateChunk {
    /// Creates a new instance of `PrivateChunk`.
    pub fn new(value: Vec<u8>, owner: PublicKey) -> Self {
        let address = Address::Private(XorName::from_content(&[&value, &owner.to_bytes()]));

        Self {
            address,
            value,
            owner,
        }
    }

    /// Returns the value.
    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }

    /// Returns the set of owners.
    pub fn owner(&self) -> &PublicKey {
        &self.owner
    }

    /// Returns the address.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the name.
    pub fn name(&self) -> &XorName {
        self.address.name()
    }

    /// Returns size of contained value.
    pub fn payload_size(&self) -> usize {
        self.value.len()
    }

    /// Returns size of this chunk after serialisation.
    pub fn serialised_size(&self) -> u64 {
        serialized_size(&self.serialised_structure()).unwrap_or(u64::MAX)
    }

    /// Returns `true` if the size is valid.
    pub fn validate_size(&self) -> bool {
        self.serialised_size() <= MAX_CHUNK_SIZE_IN_BYTES
    }

    fn serialised_structure(&self) -> (&[u8], &PublicKey) {
        (&self.value, &self.owner)
    }
}

impl Serialize for PrivateChunk {
    fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        // Address is omitted since it's derived from value + owner
        self.serialised_structure().serialize(serialiser)
    }
}

impl<'de> Deserialize<'de> for PrivateChunk {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (value, owner) = Deserialize::deserialize(deserializer)?;
        Ok(Self::new(value, owner))
    }
}

impl Debug for PrivateChunk {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        // TODO: Output owners?
        write!(formatter, "PrivateChunk {:?}", self.name())
    }
}

/// Public Chunk: an immutable chunk of data which cannot be deleted.
#[derive(Hash, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PublicChunk {
    /// Network address. Omitted when serialising and calculated from the `value` when
    /// deserialising.
    address: Address,
    /// Contained chunk.
    value: Vec<u8>,
}

impl PublicChunk {
    /// Creates a new instance of `Chunk`.
    pub fn new(value: Vec<u8>) -> Self {
        Self {
            address: Address::Public(XorName::from_content(&[&value])),
            value,
        }
    }

    /// Returns the value.
    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }

    /// Returns the address.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Returns the name.
    pub fn name(&self) -> &XorName {
        self.address.name()
    }

    /// Returns size of contained value.
    pub fn payload_size(&self) -> usize {
        self.value.len()
    }

    /// Returns size of this chunk after serialisation.
    pub fn serialised_size(&self) -> u64 {
        serialized_size(self).unwrap_or(u64::MAX)
    }

    /// Returns true if the size is valid.
    pub fn validate_size(&self) -> bool {
        self.serialised_size() <= MAX_CHUNK_SIZE_IN_BYTES
    }
}

impl Serialize for PublicChunk {
    fn serialize<S: Serializer>(&self, serialiser: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(serialiser)
    }
}

impl<'de> Deserialize<'de> for PublicChunk {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value: Vec<u8> = Deserialize::deserialize(deserializer)?;
        Ok(PublicChunk::new(value))
    }
}

impl Debug for PublicChunk {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "PublicChunk {:?}", self.name())
    }
}

/// Kind of an Chunk.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum Kind {
    /// Private.
    Private,
    /// Public.
    Public,
}

impl Kind {
    /// Creates `Kind` from a `public` flag.
    pub fn from_flag(public: bool) -> Self {
        if public {
            Kind::Public
        } else {
            Kind::Private
        }
    }

    /// Returns true if public.
    pub fn is_public(self) -> bool {
        self == Kind::Public
    }

    /// Returns true if private.
    pub fn is_private(self) -> bool {
        !self.is_public()
    }
}

/// Address of an Chunk.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum Address {
    /// Private namespace.
    Private(XorName),
    /// Public namespace.
    Public(XorName),
}

impl Address {
    /// Constructs an `Address` given `kind` and `name`.
    pub fn from_kind(kind: Kind, name: XorName) -> Self {
        match kind {
            Kind::Public => Address::Public(name),
            Kind::Private => Address::Private(name),
        }
    }

    /// Returns the kind.
    pub fn kind(&self) -> Kind {
        match self {
            Address::Private(_) => Kind::Private,
            Address::Public(_) => Kind::Public,
        }
    }

    /// Returns the name.
    pub fn name(&self) -> &XorName {
        match self {
            Address::Private(ref name) | Address::Public(ref name) => name,
        }
    }

    /// Returns true if published.
    pub fn is_public(&self) -> bool {
        self.kind().is_public()
    }

    /// Returns true if unpublished.
    pub fn is_private(&self) -> bool {
        self.kind().is_private()
    }

    /// Returns the Address serialised and encoded in z-base-32.
    pub fn encode_to_zbase32(&self) -> Result<String, Error> {
        utils::encode(&self)
    }

    /// Creates from z-base-32 encoded string.
    pub fn decode_from_zbase32<T: AsRef<str>>(encoded: T) -> Result<Self, Error> {
        utils::decode(encoded)
    }
}

/// Object storing an Chunk variant.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Debug)]
pub enum Chunk {
    /// Private Chunk.
    Private(PrivateChunk),
    /// Public Chunk.
    Public(PublicChunk),
}

impl Chunk {
    /// Returns the address.
    pub fn address(&self) -> &Address {
        match self {
            Chunk::Private(chunk) => chunk.address(),
            Chunk::Public(chunk) => chunk.address(),
        }
    }

    /// Returns the name.
    pub fn name(&self) -> &XorName {
        self.address().name()
    }

    /// Returns the owner if private chunk.
    pub fn owner(&self) -> Option<&PublicKey> {
        match self {
            Chunk::Private(chunk) => Some(chunk.owner()),
            _ => None,
        }
    }

    /// Returns the kind.
    pub fn kind(&self) -> Kind {
        self.address().kind()
    }

    /// Returns true if published.
    pub fn is_public(&self) -> bool {
        self.kind().is_public()
    }

    /// Returns true if unpublished.
    pub fn is_private(&self) -> bool {
        self.kind().is_private()
    }

    /// Returns the value.
    pub fn value(&self) -> &Vec<u8> {
        match self {
            Chunk::Private(chunk) => chunk.value(),
            Chunk::Public(chunk) => chunk.value(),
        }
    }

    /// Returns `true` if the size is valid.
    pub fn validate_size(&self) -> bool {
        match self {
            Chunk::Private(chunk) => chunk.validate_size(),
            Chunk::Public(chunk) => chunk.validate_size(),
        }
    }

    /// Returns size of this chunk after serialisation.
    pub fn serialised_size(&self) -> u64 {
        match self {
            Chunk::Private(chunk) => chunk.serialised_size(),
            Chunk::Public(chunk) => chunk.serialised_size(),
        }
    }
}

impl From<PrivateChunk> for Chunk {
    fn from(chunk: PrivateChunk) -> Self {
        Chunk::Private(chunk)
    }
}

impl From<PublicChunk> for Chunk {
    fn from(chunk: PublicChunk) -> Self {
        Chunk::Public(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::Result, utils};
    use super::{Address, PrivateChunk, PublicChunk, PublicKey, XorName};
    use bls::SecretKey;
    use hex::encode;
    use rand::{self, Rng, SeedableRng};
    use rand_xorshift::XorShiftRng;
    use std::{env, iter, thread};

    #[test]
    fn deterministic_name() {
        let chunk1 = b"Hello".to_vec();
        let chunk2 = b"Goodbye".to_vec();

        let owner1 = PublicKey::Bls(SecretKey::random().public_key());
        let owner2 = PublicKey::Bls(SecretKey::random().public_key());

        let ichunk1 = PrivateChunk::new(chunk1.clone(), owner1);
        let ichunk2 = PrivateChunk::new(chunk1, owner2);
        let ichunk3 = PrivateChunk::new(chunk2.clone(), owner1);
        let ichunk3_clone = PrivateChunk::new(chunk2, owner1);

        assert_eq!(ichunk3, ichunk3_clone);

        assert_ne!(ichunk1.name(), ichunk2.name());
        assert_ne!(ichunk1.name(), ichunk3.name());
        assert_ne!(ichunk2.name(), ichunk3.name());
    }

    #[test]
    fn deterministic_test() {
        let value = "immutable chunk value".to_owned().into_bytes();
        let chunk = PublicChunk::new(value);
        let chunk_name = encode(chunk.name().0.as_ref());
        let expected_name = "920f9a03bc90af3a7bfaf50c03abd5ff5b1579bd4006ba28eebcf240d4922519";

        assert_eq!(&expected_name, &chunk_name);
    }

    #[test]
    fn serialisation() -> Result<()> {
        let mut rng = get_rng();
        let len = rng.gen_range(1, 10_000);
        let value = iter::repeat_with(|| rng.gen()).take(len).collect();
        let chunk = PublicChunk::new(value);
        let serialised = utils::serialise(&chunk)?;
        let parsed = utils::deserialise(&serialised)?;
        assert_eq!(chunk, parsed);
        Ok(())
    }

    fn get_rng() -> XorShiftRng {
        let env_var_name = "RANDOM_SEED";
        let seed = env::var(env_var_name)
            .map(|res| res.parse::<u64>().unwrap_or_else(|_| rand::random()))
            .unwrap_or_else(|_| rand::random());
        println!(
            "To replay this '{}', set env var {}={}",
            thread::current().name().unwrap_or(""),
            env_var_name,
            seed
        );
        XorShiftRng::seed_from_u64(seed)
    }

    #[test]
    fn zbase32_encode_decode_chunk_address() -> Result<()> {
        let name = XorName::random();
        let address = Address::Public(name);
        let encoded = address.encode_to_zbase32()?;
        let decoded = self::Address::decode_from_zbase32(&encoded)?;
        assert_eq!(address, decoded);
        Ok(())
    }
}
