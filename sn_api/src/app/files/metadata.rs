// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::files_map::FileInfo;
use crate::{
    app::{
        consts::*,
        helpers::{gen_timestamp_secs, systemtime_to_rfc3339},
    },
    Error, Result,
};
use log::debug;
use std::{fs, path::Path};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// Represents file metadata.  Simplifies passing it around.
// note: all values are String or Option<String>
// to facilitate use with FileInfo.
pub(crate) struct FileMeta {
    created: String,
    modified: String,
    pub(crate) file_size: String,
    pub(crate) file_type: String,
    readonly: Option<String>,
    mode_bits: Option<String>,
    original_created: Option<String>,
    original_modified: Option<String>,
}

impl FileMeta {
    // Instantiates FileMeta from a local filesystem path.
    pub(crate) fn from_path(path: &str, follow_links: bool) -> Result<Self> {
        let (metadata, file_type) = get_metadata(Path::new(path), follow_links)?;

        // created and modified may not be available on all platforms/filesystems.
        let original_created = if let Ok(time) = metadata.created() {
            Some(systemtime_to_rfc3339(time))
        } else {
            None
        };
        let original_modified = if let Ok(time) = metadata.modified() {
            Some(systemtime_to_rfc3339(time))
        } else {
            None
        };
        let readonly = Some(metadata.permissions().readonly().to_string());

        // We use 0 as file_size for metadata such as directories, symlinks.
        let file_size = if metadata.file_type().is_file() {
            metadata.len().to_string()
        } else {
            "0".to_string()
        };

        #[cfg(windows)]
        let mode_bits = None; // Todo:  what does git do for windows?

        #[cfg(not(windows))]
        let mode_bits = Some(metadata.permissions().mode().to_string());

        let s = Self {
            created: gen_timestamp_secs(),
            modified: gen_timestamp_secs(),
            file_size,
            file_type,
            readonly,
            mode_bits,
            original_created,
            original_modified,
        };
        Ok(s)
    }

    // Instantiates FileMeta from a FileInfo
    pub(crate) fn from_file_item(file_item: &FileInfo) -> Self {
        // The first 4 must be present, else a crash.
        // lots of other code relies on this, so big refactor
        // would be needed to change it.
        let created = file_item[PREDICATE_CREATED].to_string();
        let modified = file_item[PREDICATE_MODIFIED].to_string();
        let file_size = file_item[PREDICATE_SIZE].to_string();
        let file_type = file_item[PREDICATE_TYPE].to_string();

        // These are all Option<String>
        let original_created = file_item
            .get(PREDICATE_ORIGINAL_CREATED)
            .map(ToOwned::to_owned);
        let original_modified = file_item
            .get(PREDICATE_ORIGINAL_MODIFIED)
            .map(ToOwned::to_owned);
        let readonly = file_item.get(PREDICATE_READONLY).map(ToOwned::to_owned);
        let mode_bits = file_item.get(PREDICATE_MODE_BITS).map(ToOwned::to_owned);

        Self {
            created,
            modified,
            file_size,
            file_type,
            readonly,
            mode_bits,
            original_created,
            original_modified,
        }
    }

    // Instantiates FileMeta from just type and size properties.
    pub(crate) fn from_type_and_size(file_type: &str, file_size: &str) -> Self {
        Self {
            created: gen_timestamp_secs(),
            modified: gen_timestamp_secs(),
            file_size: file_size.to_string(),
            file_type: file_type.to_string(),
            readonly: None,
            mode_bits: None,
            original_created: None,
            original_modified: None,
        }
    }

    // converts Self to FileInfo
    pub(crate) fn to_file_item(&self) -> FileInfo {
        let mut file_item = FileInfo::new();
        Self::add_to_fileitem(
            &mut file_item,
            PREDICATE_CREATED,
            Some(self.created.clone()),
        );
        Self::add_to_fileitem(
            &mut file_item,
            PREDICATE_MODIFIED,
            Some(self.modified.clone()),
        );
        Self::add_to_fileitem(&mut file_item, PREDICATE_SIZE, Some(self.file_size.clone()));
        Self::add_to_fileitem(&mut file_item, PREDICATE_TYPE, Some(self.file_type.clone()));
        Self::add_to_fileitem(&mut file_item, PREDICATE_READONLY, self.readonly.clone());
        Self::add_to_fileitem(&mut file_item, PREDICATE_MODE_BITS, self.mode_bits.clone());
        Self::add_to_fileitem(
            &mut file_item,
            PREDICATE_ORIGINAL_CREATED,
            self.original_created.clone(),
        );
        Self::add_to_fileitem(
            &mut file_item,
            PREDICATE_ORIGINAL_MODIFIED,
            self.original_modified.clone(),
        );

        file_item
    }

    // returns false if a directory or symlink, true if anything else (a file).
    pub(crate) fn filetype_is_file(file_type: &str) -> bool {
        !matches!(
            file_type,
            MIMETYPE_FILESYSTEM_DIR | MIMETYPE_FILESYSTEM_SYMLINK
        )
    }

    // returns false if a directory or symlink, true if anything else (a file).
    pub(crate) fn filetype_is_symlink(file_type: &str) -> bool {
        file_type == MIMETYPE_FILESYSTEM_SYMLINK
    }

    // returns false if a directory or symlink, true if anything else (a file).
    pub(crate) fn filetype_is_dir(file_type: &str) -> bool {
        file_type == MIMETYPE_FILESYSTEM_DIR
    }

    // returns false if a directory or symlink, true if anything else (a file).
    pub(crate) fn is_file(&self) -> bool {
        Self::filetype_is_file(&self.file_type)
    }

    pub(crate) fn is_symlink(&self) -> bool {
        Self::filetype_is_symlink(&self.file_type)
    }

    pub(crate) fn is_dir(&self) -> bool {
        Self::filetype_is_dir(&self.file_type)
    }

    // helper: adds property to FileInfo if val.is_some()
    fn add_to_fileitem(file_item: &mut FileInfo, key: &str, val: Option<String>) {
        if let Some(v) = val {
            file_item.insert(key.to_string(), v);
        }
    }
}

// Get file metadata from local filesystem
pub(crate) fn get_metadata(path: &Path, follow_links: bool) -> Result<(fs::Metadata, String)> {
    let result = if follow_links {
        fs::metadata(path)
    } else {
        fs::symlink_metadata(path)
    };
    let metadata = result.map_err(|err| {
        Error::FileSystemError(format!(
            "Couldn't read metadata from source path ('{}'): {}",
            path.display(),
            err
        ))
    })?;
    debug!("Metadata for location: {:?}", metadata);

    let media_type = get_media_type(path, &metadata);
    Ok((metadata, media_type))
}

fn get_media_type(path: &Path, meta: &fs::Metadata) -> String {
    // see: https://stackoverflow.com/questions/18869772/mime-type-for-a-directory
    // We will use the FreeDesktop standard for directories and symlinks.
    //   https://specifications.freedesktop.org/shared-mime-info-spec/shared-mime-info-spec-latest.html#idm140625828597376
    if meta.file_type().is_dir() {
        return MIMETYPE_FILESYSTEM_DIR.to_string();
    } else if meta.file_type().is_symlink() {
        return MIMETYPE_FILESYSTEM_SYMLINK.to_string();
    }
    let mime_type = mime_guess::from_path(&path);
    let media_type = mime_type.first_raw().unwrap_or("Raw");
    media_type.to_string()
}
