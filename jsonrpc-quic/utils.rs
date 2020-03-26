// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::errors::Error;
use super::Result;
use std::time::Duration;

pub fn new_transport_cfg(idle_timeout_msec: u64) -> Result<quinn::TransportConfig> {
    let mut transport_config = quinn::TransportConfig::default();
    let _ = transport_config
        .max_idle_timeout(Some(Duration::from_millis(idle_timeout_msec)))
        .map_err(|e| Error::GeneralError(e.to_string()))?;
    Ok(transport_config)
}
