// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Utilities for binding generators.

use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;

/// Recursively copy all files with the given extension from the source to the
/// targt directories.
pub fn copy_files<S: AsRef<Path>, T: AsRef<Path>>(
    source: S,
    target: T,
    extension: &str,
) -> io::Result<()> {
    let source = source.as_ref();
    let target = target.as_ref();

    for entry in WalkDir::new(source) {
        let entry = entry?;

        if entry.path().is_file() &&
            entry
                .path()
                .to_str()
                .map(|s| s.ends_with(extension))
                .unwrap_or(false)
        {
            let source_path = entry.path();
            let target_path = target.join(source_path.strip_prefix(source).unwrap_or(source_path));

            let _ = fs::copy(source_path, target_path)?;
        }
    }

    Ok(())
}
