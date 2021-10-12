// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use super::{
    files_map::{FilesMap, GetAttr},
    metadata::FileMeta,
};
use crate::{Error, Result};

pub(crate) trait RealPath {
    fn realpath(&self, fpath: &str) -> Result<String>;
}

impl RealPath for FilesMap {
    // Realpath returns the real path of a given file in the filecontainer
    // after resolving instances of "../", "./", and any relative symlinks.
    //
    // Note: fpath must be an absolute path within the FileContainer.
    fn realpath(&self, fpath: &str) -> Result<String> {
        if fpath.is_empty() {
            let msg = "Path cannot be empty".to_string();
            return Err(Error::InvalidInput(msg));
        }

        // Ensure input is an absolute path
        if &fpath[0..1] != "/" {
            let msg = format!("Relative path not supported. {}", fpath);
            return Err(Error::InvalidInput(msg));
        }

        let mut path: Vec<&str> = fpath.split('/').collect();
        let mut nlinks = 0;
        let mut newpath = Vec::<&str>::new();
        let mut ended = false;

        while !ended {
            let mut iter = path.iter().peekable();

            while let Some(cur) = iter.next() {
                if *cur == "." {
                    continue;
                } else if *cur == ".." {
                    if !newpath.is_empty() {
                        newpath.pop();
                    }
                    if newpath.is_empty() {
                        let msg = "Cannot ascend beyond root directory".to_string();
                        return Err(Error::ContentNotFound(msg));
                    }
                } else {
                    newpath.push(cur);
                    let tmppath = newpath.join("/");

                    match &self.get(&tmppath) {
                        Some(fileitem) => {
                            let meta = FileMeta::from_file_item(fileitem);

                            if meta.is_symlink() {
                                nlinks += 1;
                                if nlinks > 16 {
                                    let msg = "Too many levels of symbolic links".to_string();
                                    return Err(Error::ContentNotFound(msg));
                                }

                                let target_str = &fileitem.getattr("symlink_target")?;
                                if target_str.is_empty() {
                                    let msg = format!(
                                        "Invalid/corrupted symlink '{}'. missing target.",
                                        tmppath
                                    );
                                    return Err(Error::ContentNotFound(msg));
                                }

                                let mut target_parts = target_str.split('/').collect();

                                // if target is an absolute path, we use it as-is.
                                // else if relative path, we append it to new newpath
                                //    after removing the current path component.
                                let mut target: Vec<&str> = if &target_str[0..1] == "/" {
                                    target_parts
                                } else {
                                    newpath.pop();
                                    newpath.append(&mut target_parts);
                                    newpath
                                };

                                for n in &mut iter {
                                    target.push(n)
                                }

                                path = target;
                                newpath = Vec::<&str>::new();
                                break;
                            } else if meta.is_dir() {
                                if iter.peek() == None {
                                    ended = true;
                                }
                            } else {
                                // must be file.
                                ended = true;
                            }
                        }
                        // None case applies for "/" root dir.  It also
                        // occurs for really old FileContainers created before
                        // empty directories existed, in which case there is
                        // always a further path component.  And of course it
                        // applies for an invalid path.
                        None => {
                            if iter.peek() == None {
                                ended = true;
                            }
                        }
                    }
                }
            }
        }
        Ok(newpath.join("/"))
    }
}
