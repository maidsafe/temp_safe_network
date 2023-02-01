// Copyright 2023 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

#[cfg(any(test, feature = "test-utils"))]
/// Utility functions for testing clients
pub mod test_client;

#[cfg(test)]
pub use test_client::{
    create_test_client, create_test_client_with, get_dbc_owner_from_secret_key_hex,
    try_create_test_client,
};

#[cfg(any(test, feature = "test-utils"))]
pub use test_client::read_genesis_dbc_from_first_node;

#[cfg(test)]
pub use sn_interface::init_logger;

#[cfg(test)]
use eyre::{bail, Result};
#[cfg(test)]
use sn_interface::messaging::data::Error as ErrorMsg;

#[cfg(test)]
pub fn extract_error_string(
    received_errors: Vec<(sn_interface::types::Peer, Vec<ErrorMsg>)>,
) -> Result<String> {
    match received_errors
        .into_iter()
        .flat_map(|(_, errors)| errors)
        .filter_map(|err| {
            if let ErrorMsg::InvalidOperation(error_string) = err {
                Some(error_string)
            } else {
                None
            }
        })
        .last()
    {
        Some(error_string) => Ok(error_string),
        None => bail!("We expected an error to be returned"),
    }
}
