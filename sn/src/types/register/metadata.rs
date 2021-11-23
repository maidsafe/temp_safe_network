// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

/// An action on Register data type.
#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum Action {
    /// Read from the data.
    Read,
    /// Write to the data.
    Write,
}

/// An entry in a Register (note that the vec<u8> is size limited: MAX_REG_ENTRY_SIZE)
pub type Entry = Vec<u8>;
