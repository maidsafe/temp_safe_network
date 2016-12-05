// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net
// Commercial License, version 1.0 or later, or (2) The General Public License
// (GPL), version 3, depending on which licence you accepted on initial access
// to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project
// generally, you agree to be bound by the terms of the MaidSafe Contributor
// Agreement, version 1.0.
// This, along with the Licenses can be found in the root directory of this
// project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network
// Software distributed under the GPL Licence is distributed on an "AS IS"
// BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
//
// Please review the Licences for the specific language governing permissions
// and limitations relating to use of the SAFE Network Software.

//! Structure representing application registered with the launcher + set of
//! FFI operations on it.

// TODO(Spandan) - Run through this and make interfaces efficient (return
// references instead of copies etc.) and uniform (i.e. not use get_ prefix for
// mem functions.

/*

use core::FutureExt;
use ffi::{FfiError, OpaqueCtx, Session, helper, launcher_config};
use futures::Future;
// use nfs::DirId;
use rust_sodium::crypto::{box_, secretbox};
use std::os::raw::c_void;

/// Represents an application connected to the launcher.
#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub enum App {
    /// Unregistered application
    Unregistered,
    /// Authorised application
    Registered {
        // /// Application directory
        // app_dir_id: DirId,
        /// Defines whether the application has access to SAFE Drive
        safe_drive_access: bool,
        /// Symmetric encryption key of the app
        sym_key: secretbox::Key,
    },
}

impl App {
    /// Get app root directory ID
    pub fn app_dir(&self) -> Result<DirId, FfiError> {
        if let App::Registered { ref app_dir_id, .. } = *self {
            Ok(app_dir_id.clone())
        } else {
            Err(FfiError::OperationForbiddenForApp)
        }
    }

    /// Get root directory: for shared paths, this is the SAFEdrive directory,
    /// otherwise it's the app directory.
    pub fn root_dir(&self, client: Client, is_shared: bool) -> Box<FfiFuture<DirId>> {
        if is_shared {
            if let App::Registered { ref safe_drive_access, .. } = *self {
                if !safe_drive_access {
                    return err!(FfiError::PermissionDenied);
                }
                helper::safe_drive_metadata(client.clone())
                    .map(move |dir_meta| dir_meta.id())
                    .into_box()
            } else {
                err!(FfiError::from("Safe Drive directory is not available for an unregistered \
                                     app"))
            }
        } else {
            futures::done(self.app_dir()).into_box()
        }
    }
}

*/
