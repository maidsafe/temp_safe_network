// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::xorurl::XorUrlBase;

pub const CONTENT_ADDED_SIGN: &str = "+";
pub const CONTENT_UPDATED_SIGN: &str = "*";
pub const CONTENT_DELETED_SIGN: &str = "-";
pub const CONTENT_ERROR_SIGN: &str = "E";

pub const FAKE_RDF_PREDICATE_LINK: &str = "link";
pub const FAKE_RDF_PREDICATE_TYPE: &str = "type";
pub const FAKE_RDF_PREDICATE_SIZE: &str = "size";
pub const FAKE_RDF_PREDICATE_MODIFIED: &str = "modified";
pub const FAKE_RDF_PREDICATE_CREATED: &str = "created";

// Default host of the authenticator endpoint to send the requests to
pub const SAFE_AUTHD_ENDPOINT_HOST: &str = "https://localhost";
// Default authenticator port number where to send requests to
pub const SAFE_AUTHD_ENDPOINT_PORT: u16 = 33000;

// Default base encoding used for XOR URLs
pub const DEFAULT_XORURL_BASE: XorUrlBase = XorUrlBase::Base32z;

// Number of milliseconds to allow an idle connection with authd before closing it
pub const SAFE_AUTHD_CONNECTION_IDLE_TIMEOUT: u64 = 60_000;
