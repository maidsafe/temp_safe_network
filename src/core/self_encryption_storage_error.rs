// Copyright 2016 MaidSafe.net limited.
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


use core::errors::CoreError;
use self_encryption::StorageError;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Errors arising from storage object being used by self-encryptors.
#[derive(Debug)]
pub struct SelfEncryptionStorageError(pub Box<CoreError>);

impl Display for SelfEncryptionStorageError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Display::fmt(&*self.0, formatter)
    }
}

impl Error for SelfEncryptionStorageError {
    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&Error> {
        self.0.cause()
    }
}

impl From<CoreError> for SelfEncryptionStorageError {
    fn from(error: CoreError) -> SelfEncryptionStorageError {
        SelfEncryptionStorageError(Box::new(error))
    }
}

impl StorageError for SelfEncryptionStorageError {}
