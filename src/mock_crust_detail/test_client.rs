// Copyright 2016 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use super::poll;
use super::test_node::TestNode;
use GROUP_SIZE;
use rand::{Rng, XorShiftRng};
use routing::{self, AccountInfo, Authority, ClientError, EntryAction, Event, EventStream, FullId,
              ImmutableData, MessageId, MutableData, PermissionSet, Response,
              TYPE_TAG_SESSION_PACKET, User, Value, XorName};
use routing::mock_crust::{self, Config, Network, ServiceHandle};
use rust_sodium::crypto::sign;
use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::sync::mpsc::TryRecvError;

macro_rules! assert_recv_response {
    ($client:expr, $resp:ident, $request_msg_id:expr) => {
        match $client.try_recv() {
            Ok(Event::Response { response: Response::$resp { res, msg_id }, .. }) => {
                assert_eq!($request_msg_id, msg_id);
                return res;
            }
            Ok(event) => panic!("Unexpected event: {:?}", event),
            Err(error) => panic!("Unexpected error: {:?}", error),
        }
    }
}

/// Client for use in tests only
pub struct TestClient {
    _handle: ServiceHandle,
    routing_client: routing::Client,
    full_id: FullId,
    client_manager: Authority<XorName>,
    rng: XorShiftRng,
}

// FIXME: there are inconsistencies in how the request methods are implemented,
// for no apparent reason:
//
// - some do `flush`, so don't.
// - some use `poll::nodes_and_client`, some `poll::poll_and_resend_unacknowledged`.
// - some panic when no response received, some return error.
//
// We should either make them consistent, or document clearly why the inconsistency
// is important.

impl TestClient {
    /// Create a test client for the mock network
    pub fn new(network: &Network, config: Option<Config>) -> Self {
        let full_id = FullId::new();
        let handle = network.new_service_handle(config.clone(), None);
        let client = mock_crust::make_current(&handle, || {
            unwrap!(routing::Client::new(Some(full_id.clone()), config, GROUP_SIZE))
        });

        let client_manager = Authority::ClientManager(*full_id.public_id().name());

        TestClient {
            _handle: handle,
            routing_client: client,
            full_id: full_id,
            client_manager: client_manager,
            rng: network.new_rng(),
        }
    }

    /// Set the `ClientManager` this client will send all mutation request to. By default,
    /// it is the `ClientManager` of this client, but this can be changed for clients that
    /// are apps.
    pub fn set_client_manager(&mut self, name: XorName) {
        self.client_manager = Authority::ClientManager(name);
    }

    /// Returns the next event received from routing, if any.
    pub fn try_recv(&mut self) -> Result<Event, TryRecvError> {
        self.routing_client.try_next_ev()
    }

    /// empty this client event loop
    pub fn poll(&mut self) -> usize {
        let mut result = 0;

        while self.routing_client.poll() {
            result += 1;
        }

        result
    }

    /// empty this client event loop
    pub fn poll_once(&mut self) -> bool {
        self.routing_client.poll()
    }

    /// Resend all unacknowledged messages.
    pub fn resend_unacknowledged(&mut self) -> bool {
        self.routing_client.resend_unacknowledged()
    }

    /// check client successfully connected to mock network
    pub fn ensure_connected(&mut self, nodes: &mut [TestNode]) {
        let _ = poll::nodes_and_client(nodes, self);

        match self.try_recv() {
            Ok(Event::Connected) => (),
            e => panic!("Expected Ok(Event::Connected), got {:?}", e),
        }
    }

    /// create an account and store it
    pub fn create_account(&mut self, nodes: &mut [TestNode]) {
        let owner = *self.full_id.public_id().signing_public_key();
        let owners = iter::once(owner).collect::<BTreeSet<_>>();

        let data = unwrap!(MutableData::new(self.rng.gen(),
                                            TYPE_TAG_SESSION_PACKET,
                                            Default::default(),
                                            Default::default(),
                                            owners));

        unwrap!(self.put_mdata_response(data, nodes));
    }

    /// Put immutable data
    pub fn put_idata(&mut self, data: ImmutableData) -> MessageId {
        let msg_id = MessageId::new();
        unwrap!(self.routing_client
                    .put_idata(self.client_manager, data, msg_id));
        msg_id
    }

    /// Put immutable data and read from mock network
    pub fn put_idata_response(&mut self,
                              data: ImmutableData,
                              nodes: &mut [TestNode])
                              -> Result<(), ClientError> {
        let request_msg_id = self.put_idata(data);
        let _ = poll::poll_and_resend_unacknowledged(nodes, self);

        match self.try_recv() {
            Ok(Event::Response { response: Response::PutIData { res, msg_id }, .. }) => {
                assert_eq!(request_msg_id, msg_id);
                res
            }
            Ok(response) => panic!("Unexpected response: {:?}", response),
            Err(error) => Err(ClientError::from(format!("{:?}", error))),
        }
    }

    /// Try and get immutable data from nodes provided.
    pub fn get_idata_response(&mut self,
                              name: XorName,
                              nodes: &mut [TestNode])
                              -> Result<ImmutableData, ClientError> {
        self.get_idata_response_with_src(name, nodes)
            .map(|(data, _)| data)
    }

    /// Try to get immutable data from the given nodes. Returns the retrieved data and
    /// the source authority the data was sent by.
    pub fn get_idata_response_with_src
        (&mut self,
         name: XorName,
         nodes: &mut [TestNode])
         -> Result<(ImmutableData, Authority<XorName>), ClientError> {
        let dst = Authority::NaeManager(name);
        let request_msg_id = MessageId::new();
        self.flush();

        unwrap!(self.routing_client.get_idata(dst, name, request_msg_id));
        let _ = poll::nodes_and_client(nodes, self);

        loop {
            match self.try_recv() {
                Ok(Event::Response {
                       response: Response::GetIData { res, msg_id },
                       src,
                       ..
                   }) => {
                    if request_msg_id != msg_id {
                        warn!("{:?}  --   {:?}", request_msg_id, msg_id);
                    } else {
                        return res.map(|data| (data, src));
                    }
                }
                Ok(event) => panic!("Unexpected event: {:?}", event),
                Err(error) => panic!("Expected error: {:?}", error),
            }
        }
    }

    /// Put mutable data
    pub fn put_mdata(&mut self, data: MutableData) -> MessageId {
        let msg_id = MessageId::new();
        let requester = *self.signing_public_key();
        unwrap!(self.routing_client
                    .put_mdata(self.client_manager, data, msg_id, requester));
        msg_id
    }

    /// Put mutable data and wait for the response.
    pub fn put_mdata_response(&mut self,
                              data: MutableData,
                              nodes: &mut [TestNode])
                              -> Result<(), ClientError> {
        let request_msg_id = self.put_mdata(data);
        let _ = poll::poll_and_resend_unacknowledged(nodes, self);

        assert_recv_response!(self, PutMData, request_msg_id)
    }

    /// Sends a `GetMDataVersion` request and wait for the response.
    pub fn get_mdata_version_response(&mut self,
                                      name: XorName,
                                      tag: u64,
                                      nodes: &mut [TestNode])
                                      -> Result<u64, ClientError> {
        self.flush();

        let dst = Authority::NaeManager(name);
        let msg_id = MessageId::new();

        unwrap!(self.routing_client
                    .get_mdata_version(dst, name, tag, msg_id));
        let _ = poll::nodes_and_client(nodes, self);

        assert_recv_response!(self, GetMDataVersion, msg_id)
    }

    /// Sends a `GetMDataShell` request and wait for the response.
    pub fn get_mdata_shell_response(&mut self,
                                    name: XorName,
                                    tag: u64,
                                    nodes: &mut [TestNode])
                                    -> Result<MutableData, ClientError> {
        self.flush();

        let dst = Authority::NaeManager(name);
        let msg_id = MessageId::new();

        unwrap!(self.routing_client
                    .get_mdata_shell(dst, name, tag, msg_id));
        let _ = poll::nodes_and_client(nodes, self);

        assert_recv_response!(self, GetMDataShell, msg_id)
    }

    /// Sends a `ListMDataEntries` request and wait for the response.
    pub fn list_mdata_entries_response(&mut self,
                                       name: XorName,
                                       tag: u64,
                                       nodes: &mut [TestNode])
                                       -> Result<BTreeMap<Vec<u8>, Value>, ClientError> {
        self.flush();

        let dst = Authority::NaeManager(name);
        let msg_id = MessageId::new();

        unwrap!(self.routing_client
                    .list_mdata_entries(dst, name, tag, msg_id));
        let _ = poll::nodes_and_client(nodes, self);

        assert_recv_response!(self, ListMDataEntries, msg_id)
    }

    /// Sends a `GetMDataValue` request and wait for the response.
    pub fn get_mdata_value_response(&mut self,
                                    name: XorName,
                                    tag: u64,
                                    key: Vec<u8>,
                                    nodes: &mut [TestNode])
                                    -> Result<Value, ClientError> {
        self.flush();

        let dst = Authority::NaeManager(name);
        let msg_id = MessageId::new();

        unwrap!(self.routing_client
                    .get_mdata_value(dst, name, tag, key, msg_id));
        let _ = poll::nodes_and_client(nodes, self);

        assert_recv_response!(self, GetMDataValue, msg_id)
    }

    /// Sends a `MutateMDataEntries` request.
    pub fn mutate_mdata_entries(&mut self,
                                name: XorName,
                                tag: u64,
                                actions: BTreeMap<Vec<u8>, EntryAction>)
                                -> MessageId {
        let msg_id = MessageId::new();
        let requester = *self.signing_public_key();
        unwrap!(self.routing_client
                    .mutate_mdata_entries(self.client_manager,
                                          name,
                                          tag,
                                          actions,
                                          msg_id,
                                          requester));
        msg_id
    }

    /// Sends a `MutateMDataEntries` request and wait for the response.
    pub fn mutate_mdata_entries_response(&mut self,
                                         name: XorName,
                                         tag: u64,
                                         actions: BTreeMap<Vec<u8>, EntryAction>,
                                         nodes: &mut [TestNode])
                                         -> Result<(), ClientError> {
        self.flush();
        let msg_id = self.mutate_mdata_entries(name, tag, actions);
        let _ = poll::poll_and_resend_unacknowledged(nodes, self);
        assert_recv_response!(self, MutateMDataEntries, msg_id)
    }

    /// Sends a `ListMDataPermissions` request and wait for the response.
    pub fn list_mdata_permissions_response
        (&mut self,
         name: XorName,
         tag: u64,
         nodes: &mut [TestNode])
         -> Result<BTreeMap<User, PermissionSet>, ClientError> {
        self.flush();

        let dst = Authority::NaeManager(name);
        let msg_id = MessageId::new();

        unwrap!(self.routing_client
                    .list_mdata_permissions(dst, name, tag, msg_id));
        let _ = poll::nodes_and_client(nodes, self);

        assert_recv_response!(self, ListMDataPermissions, msg_id)
    }

    /// Sends a `ListMDataUserPermissions` request and wait for the response.
    pub fn list_mdata_user_permissions_response(&mut self,
                                                name: XorName,
                                                tag: u64,
                                                user: User,
                                                nodes: &mut [TestNode])
                                                -> Result<PermissionSet, ClientError> {
        self.flush();

        let dst = Authority::NaeManager(name);
        let msg_id = MessageId::new();

        unwrap!(self.routing_client
                    .list_mdata_user_permissions(dst, name, tag, user, msg_id));
        let _ = poll::nodes_and_client(nodes, self);

        assert_recv_response!(self, ListMDataUserPermissions, msg_id)
    }

    /// Sends a `SetMDataUserPermissions` request and wait for the response.
    pub fn set_mdata_user_permissions_response(&mut self,
                                               name: XorName,
                                               tag: u64,
                                               user: User,
                                               permissions: PermissionSet,
                                               version: u64,
                                               nodes: &mut [TestNode])
                                               -> Result<(), ClientError> {
        self.flush();

        let msg_id = MessageId::new();
        let requester = *self.signing_public_key();

        unwrap!(self.routing_client
                    .set_mdata_user_permissions(self.client_manager,
                                                name,
                                                tag,
                                                user,
                                                permissions,
                                                version,
                                                msg_id,
                                                requester));
        let _ = poll::poll_and_resend_unacknowledged(nodes, self);

        assert_recv_response!(self, SetMDataUserPermissions, msg_id)
    }

    /// Sends a `DelMDataUserPermissions` request and wait for the response.
    pub fn del_mdata_user_permissions_response(&mut self,
                                               name: XorName,
                                               tag: u64,
                                               user: User,
                                               version: u64,
                                               nodes: &mut [TestNode])
                                               -> Result<(), ClientError> {
        self.flush();

        let msg_id = MessageId::new();
        let requester = *self.signing_public_key();
        unwrap!(self.routing_client
                    .del_mdata_user_permissions(self.client_manager,
                                                name,
                                                tag,
                                                user,
                                                version,
                                                msg_id,
                                                requester));
        let _ = poll::poll_and_resend_unacknowledged(nodes, self);

        assert_recv_response!(self, DelMDataUserPermissions, msg_id)
    }

    /// Sends a `ChangeMDataOwner` request and wait for the response.
    pub fn change_mdata_owner_response(&mut self,
                                       name: XorName,
                                       tag: u64,
                                       new_owners: BTreeSet<sign::PublicKey>,
                                       version: u64,
                                       nodes: &mut [TestNode])
                                       -> Result<(), ClientError> {
        self.flush();

        let msg_id = MessageId::new();
        unwrap!(self.routing_client
                    .change_mdata_owner(self.client_manager,
                                        name,
                                        tag,
                                        new_owners,
                                        version,
                                        msg_id));
        let _ = poll::poll_and_resend_unacknowledged(nodes, self);

        assert_recv_response!(self, ChangeMDataOwner, msg_id)
    }

    /// Sends a GetAccountInfo request, polls the mock network and expects a GetAccountInfo response
    pub fn get_account_info_response(&mut self,
                                     nodes: &mut [TestNode])
                                     -> Result<AccountInfo, ClientError> {
        self.flush();

        let msg_id = MessageId::new();
        unwrap!(self.routing_client
                    .get_account_info(self.client_manager, msg_id));
        let _ = poll::nodes_and_client(nodes, self);

        assert_recv_response!(self, GetAccountInfo, msg_id)
    }

    /// Sends a ListAuthKeysAndVersion request and wait for the response.
    pub fn list_auth_keys_and_version_response
        (&mut self,
         nodes: &mut [TestNode])
         -> Result<(BTreeSet<sign::PublicKey>, u64), ClientError> {
        self.flush();

        let msg_id = MessageId::new();
        unwrap!(self.routing_client
                    .list_auth_keys_and_version(self.client_manager, msg_id));
        let _ = poll::nodes_and_client(nodes, self);

        assert_recv_response!(self, ListAuthKeysAndVersion, msg_id)
    }

    /// Sends a InsAuthKey request and wait for the response.
    pub fn ins_auth_key_response(&mut self,
                                 key: sign::PublicKey,
                                 version: u64,
                                 nodes: &mut [TestNode])
                                 -> Result<(), ClientError> {
        self.flush();

        let msg_id = MessageId::new();
        unwrap!(self.routing_client
                    .ins_auth_key(self.client_manager, key, version, msg_id));
        let _ = poll::nodes_and_client(nodes, self);

        assert_recv_response!(self, InsAuthKey, msg_id)
    }

    /// Return a full id for this client
    pub fn full_id(&self) -> &FullId {
        &self.full_id
    }

    /// Return signing public key for this client
    pub fn signing_public_key(&self) -> &sign::PublicKey {
        self.full_id.public_id().signing_public_key()
    }

    /// Return client's network name
    pub fn name(&self) -> &XorName {
        &self.full_id.public_id().name()
    }

    fn flush(&mut self) {
        while let Ok(_) = self.try_recv() {}
    }
}
