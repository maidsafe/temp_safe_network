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

/// Client Errors
pub enum ClientError {
    /// StructuredData has no space available to fit in any user data inside it.
    StructuredDataHeaderSizeProhibitive,
    /// Could not Serialise or Deserialise
    UnsuccessfulEncodeDecode,
    /// Asymmetric Key Decryption Failed
    AsymmetricDecipherFailure,
    /// Symmetric Key Decryption Failed
    SymmetricDecipherFailure,
    /// GetFailure
    GetFailure,
    /// ReceivedUnexpectedData
    ReceivedUnexpectedData,
}

impl From<::cbor::CborError> for ClientError {
    fn from(_: ::cbor::CborError) -> ClientError {
        ClientError::UnsuccessfulEncodeDecode
    }
}
