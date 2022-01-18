// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

mod authd_client_api;
mod notifs_endpoint;

use super::common;
use super::constants;

// Expose what's meant to be the public API
pub use authd_client_api::{AuthAllowPrompt, AuthdStatus, PendingAuthReqs, SafeAuthdClient};
