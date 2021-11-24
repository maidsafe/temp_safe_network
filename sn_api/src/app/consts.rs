// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use crate::url::XorUrlBase;

// Default base encoding used for XOR URLs
pub const DEFAULT_XORURL_BASE: XorUrlBase = XorUrlBase::Base32z;

pub const CONTENT_ADDED_SIGN: &str = "+";
pub const CONTENT_UPDATED_SIGN: &str = "*";
pub const CONTENT_DELETED_SIGN: &str = "-";
pub const CONTENT_ERROR_SIGN: &str = "E";

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
