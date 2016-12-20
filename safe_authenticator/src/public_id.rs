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

use AuthFuture;
use errors::AuthError;
use futures::{Future, IntoFuture};
use maidsafe_utilities::serialisation::serialise;
use routing::{ClientError, EntryActions, MutableData, XorName};
use rust_sodium::crypto::hash::sha256;
use safe_core::{Client, CoreError, FutureExt, MDataInfo, PUBLIC_ID_TAG};

const PUBLIC_ID_ENTRY_KEY: &'static [u8] = b"_publicId";

/// Create mutable data for Public ID.
pub fn create<T: AsRef<str>>(client: &Client, public_id: &T) -> Box<AuthFuture<()>> {
    let client2 = client.clone();
    let client3 = client.clone();
    let client4 = client.clone();

    let name = XorName(sha256::hash(public_id.as_ref().as_bytes()).0);

    client.user_root_dir()
        .and_then(|root_dir| {
            let owner_key = client.owner_key()?;
            let key = root_dir.enc_entry_key(PUBLIC_ID_ENTRY_KEY)?;

            Ok((owner_key, root_dir, key))
        })
        .map_err(AuthError::from)
        .into_future()
        .and_then(move |(owner_key, root_dir, key)| {
            client2.get_mdata_value(root_dir.name, root_dir.type_tag, key.clone())
                .then(move |res| {
                    match res {
                        Ok(_) => Err(CoreError::RoutingClientError(ClientError::EntryExists)),
                        Err(CoreError::RoutingClientError(ClientError::NoSuchEntry)) => {
                            Ok((owner_key, root_dir, key))
                        }
                        Err(err) => Err(err),
                    }
                })
                .map_err(AuthError::from)
        })
        .and_then(move |(owner_key, root_dir, key)| {
            let info = MDataInfo::new_public(name, PUBLIC_ID_TAG);
            let data = MutableData::new(info.name,
                                        info.type_tag,
                                        Default::default(),
                                        Default::default(),
                                        btree_set![owner_key]).map_err(CoreError::from)
                .map_err(AuthError::from)?;

            Ok((root_dir, key, info, data))
        })
        .and_then(move |(root_dir, key, info, data)| {
            client3.put_mdata(data)
                .map(move |_| (root_dir, key, info))
                .map_err(From::from)
        })
        .and_then(|(root_dir, key, info)| {
            let value = serialise(&info)?;
            let value = root_dir.enc_entry_value(&value)?;

            Ok((root_dir, key, value))
        })
        .and_then(move |(root_dir, key, value)| {
            let actions = EntryActions::new()
                .ins(key, value, 0)
                .into();
            client4.mutate_mdata_entries(root_dir.name, root_dir.type_tag, actions)
                .map_err(From::from)
        })
        .into_box()
}
