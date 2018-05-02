// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use base64::{self, DecodeError, URL_SAFE_NO_PAD};

/// Encode the data using base64 encoding.
pub fn base64_encode(input: &[u8]) -> String {
    base64::encode_config(input, URL_SAFE_NO_PAD)
}

/// Decode base64 encoded data.
pub fn base64_decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    base64::decode_config(input, URL_SAFE_NO_PAD)
}
