// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::safeurl::XorUrlBase;

// Default base encoding used for XOR URLs
pub const DEFAULT_XORURL_BASE: XorUrlBase = XorUrlBase::Base32z;

pub const PREDICATE_LINK: &str = "link";
pub const PREDICATE_TYPE: &str = "type";
pub const PREDICATE_SIZE: &str = "size";
pub const PREDICATE_MODIFIED: &str = "modified";
pub const PREDICATE_CREATED: &str = "created";
pub const PREDICATE_ORIGINAL_MODIFIED: &str = "o_modified";
pub const PREDICATE_ORIGINAL_CREATED: &str = "o_created";
pub const PREDICATE_READONLY: &str = "readonly";
pub const PREDICATE_MODE_BITS: &str = "mode_bits";

// see: https://stackoverflow.com/questions/18869772/mime-type-for-a-directory
// We will use the FreeDesktop standard for directories and symlinks.
//   https://specifications.freedesktop.org/shared-mime-info-spec/shared-mime-info-spec-latest.html#idm140625828597376
//
// TBD: is there a better location for these?
//      maybe files.rs or xorurl_media_types.rs?
pub const MIMETYPE_FILESYSTEM_DIR: &str = "inode/directory";
pub const MIMETYPE_FILESYSTEM_SYMLINK: &str = "inode/symlink";
