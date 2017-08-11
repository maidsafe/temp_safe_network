// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use client::{Client, MDataInfo};
use errors::CoreError;
use futures::Future;
use nfs::{NfsError, NfsFuture};
use routing::{ClientError, MutableData, PermissionSet, User, Value};
use std::collections::BTreeMap;
use utils::FutureExt;

/// Create a new directory based on the provided `MDataInfo`
pub fn create_dir<T: 'static>(
    client: &Client<T>,
    dir: &MDataInfo,
    contents: BTreeMap<Vec<u8>, Value>,
    perms: BTreeMap<User, PermissionSet>,
) -> Box<NfsFuture<()>> {
    let pub_key = fry!(client.owner_key().map_err(NfsError::from));
    let owners = btree_set![pub_key];
    let dir_md = fry!(
        MutableData::new(dir.name, dir.type_tag, perms, contents, owners)
            .map_err(CoreError::from)
    );
    client
        .put_mdata(dir_md)
        .or_else(move |err| {
            match err {
                // This dir has been already created
                CoreError::RoutingClientError(ClientError::DataExists) => Ok(()),
                e => Err(e),
            }
        })
        .map_err(NfsError::from)
        .into_box()
}
