// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

// Default host of the authenticator endpoint to send the requests to
pub const SN_AUTHD_ENDPOINT_HOST: &str = "https://localhost";
// Default authenticator port number where to send requests to
pub const SN_AUTHD_ENDPOINT_PORT: u16 = 33000;

// Number of milliseconds to allow an idle connection with authd before closing it
pub const SN_AUTHD_CONNECTION_IDLE_TIMEOUT: u64 = 120_000;
