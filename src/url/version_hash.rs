// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::types::register::EntryHash;
use multibase::Base;
use std::str::FromStr;
use std::fmt::{self, Display};
use std::convert::TryInto;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VersionHashError {
    #[error("Decoding error")]
    DecodingError(#[from] multibase::Error),
    #[error("Invalid hash length")]
    InvalidHashLength,
    #[error("Invalid encoding (must be Base32Z)")]
    InvalidEncoding,
}

/// Version Hash corresponding to the register entry hash where the content is stored
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Default, Copy)]
pub struct VersionHash {
    register_entry_hash: EntryHash,
}

impl Display for VersionHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let base32z = multibase::encode(Base::Base32Z, self.register_entry_hash);
        write!(f, "{}", base32z)
    }
}

impl FromStr for VersionHash {
    type Err = VersionHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (base, data) = multibase::decode(s)?;
        if base != Base::Base32Z {
            return Err(VersionHashError::InvalidEncoding)
        }
        let entry_hash = data.try_into().map_err(|_| VersionHashError::InvalidHashLength)?;
        Ok(VersionHash{register_entry_hash: entry_hash})
    }
}

impl From<&EntryHash> for VersionHash {
    fn from(register_entry_hash: &EntryHash) -> Self {
        VersionHash{register_entry_hash: register_entry_hash.to_owned()}
    }
}

impl VersionHash {
    /// Getter for register the entry hash corresponding to that version
    pub fn register_entry_hash(&self) -> EntryHash {
        self.register_entry_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Result, bail};

    #[test]
    fn test_version_hash_encode_decode() -> Result<()> {
        let string_hash_32bits = "hqt1zg7dwci3ze7dfqp48e3muqt4gkh5wqt1zg7dwci3ze7dfqp4y";
        let vh = VersionHash::from_str(string_hash_32bits)?;
        let str_vh = vh.to_string();
        assert_eq!(&str_vh, string_hash_32bits);
        Ok(())
    }

    #[test]
    fn test_version_hash_decoding_error() -> Result<()> {
        let string_hash = "hxf1zgedpcfzg1ebbhxf1zgedpcfzg1ebbhxf1zgedpcfzg1ebb";
        match VersionHash::from_str(string_hash) {
            Err(VersionHashError::DecodingError(_)) => Ok(()),
            _ => bail!("Should have triggered a DecodingError"),
        }
    }

    #[test]
    fn test_version_hash_invalid_encoding() -> Result<()> {
        let string_hash = "900573277761329450583662625";
        let vh = VersionHash::from_str(string_hash);
        assert_eq!(vh, Err(VersionHashError::InvalidEncoding));
        Ok(())
    }

    #[test]
    fn test_version_hash_invalid_len() -> Result<()> {
        let string_hash = "hxf1zgedpcfzg1ebb";
        let vh = VersionHash::from_str(string_hash);
        assert_eq!(vh, Err(VersionHashError::InvalidHashLength));
        Ok(())
    }
}
