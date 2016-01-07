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

use routing::InterfaceError;

// TODO(Spandan) This is a mess with new routing - get it sorted once sprint begins
/// Reception of reqested Data
pub enum DataReceivedEvent {
    /// Received Data
    DataReceived,
}

/// Netowork Events that Client Modules need to deal with
pub enum NetworkEvent {
    /// The client engine is connected to atleast one peer
    Connected,
    /// Graceful Exit Condition
    Terminated,
}

/// Failures in operations that Client Modules need to deal with
pub enum OperationFailureEvent {
    /// PUT request failed
    PutFailure(InterfaceError),
    /// POST request failed
    PostFailure(InterfaceError),
    /// DELETE request failed
    DeleteFailure(InterfaceError),
    /// Graceful Exit Condition
    Terminated,
}
