// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Utilities for integration tests

mod rng;

pub use self::rng::TestRng;

use bytes::Bytes;
use crossbeam_channel::{Receiver, Sender};
use routing::quic_p2p::{self, Builder, Event, Network, NodeInfo, OurType, Peer, QuicP2p};
use safe_nd::{
    AppFullId, AppPublicId, ClientFullId, ClientPublicId, Coins, Error, HandshakeRequest,
    HandshakeResponse, Message, MessageId, Notification, PublicId, PublicKey, Request, Response,
    Signature, Transaction, TransactionId,
};
use safe_vault::{
    routing::{ConsensusGroup, ConsensusGroupRef, Node},
    Command, Config, Vault,
};
use serde::Serialize;
use std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    slice,
};
use tempdir::TempDir;
use unwrap::unwrap;

/// Default number of vaults to run the tests with.
const DEFAULT_NUM_VAULTS: usize = 5;

macro_rules! unexpected {
    ($e:expr) => {
        panic!("Unexpected {:?}", $e)
    };
}

pub struct Environment {
    rng: TestRng,
    network: Network,
    vaults: Vec<TestVault>,
    _consensus_group: ConsensusGroupRef,
}

impl Environment {
    pub fn with_multiple_vaults(num_vaults: usize) -> Self {
        assert!(num_vaults > 0);

        let network = Network::new(Default::default());

        let consensus_group = ConsensusGroup::new();
        let vaults = if num_vaults > 1 {
            let mut vaults = Vec::with_capacity(num_vaults);
            for _ in 0..num_vaults {
                vaults.push(TestVault::new(
                    Some(consensus_group.clone()),
                    rng::new(network.new_rng()),
                ));
            }
            vaults
        } else {
            vec![TestVault::new(None, rng::new(network.new_rng()))]
        };

        Self {
            rng: rng::new(network.new_rng()),
            network,
            vaults,
            _consensus_group: consensus_group,
        }
    }

    pub fn new() -> Self {
        Self::with_multiple_vaults(DEFAULT_NUM_VAULTS)
    }

    pub fn rng(&mut self) -> &mut TestRng {
        &mut self.rng
    }

    // Poll the mock network and the environment's vault.
    pub fn poll(&mut self) {
        let mut progress = true;
        while progress {
            self.network.poll();
            progress = self.vaults.iter_mut().any(|vault| vault.inner.poll());
        }
    }

    pub fn new_connected_client(&mut self) -> TestClient {
        let mut client = TestClient::new_disconnected(&mut self.rng);
        self.establish_connection(&mut client);
        client
    }

    pub fn new_connected_app(&mut self, owner: ClientPublicId) -> TestApp {
        let mut app = TestApp::new_disconnected(&mut self.rng, owner);
        self.establish_connection(&mut app);
        app
    }

    pub fn new_disconnected_app(&mut self, owner: ClientPublicId) -> TestApp {
        TestApp::new_disconnected(&mut self.rng, owner)
    }

    /// Establish connection assuming we are already at the destination section.
    pub fn establish_connection<T: TestClientTrait>(&mut self, client: &mut T) {
        let conn_info = self.vaults[0].connection_info();
        client.quic_p2p().connect_to(conn_info.clone());
        self.poll();

        client.expect_connected_to(&conn_info);

        // Bootstrap handshake procedure where we assume that we don't have to rebootstrap
        let client_public_id = client.full_id().public_id();
        client.send_bootstrap_request(&client_public_id);
        self.poll();

        client.handle_handshake_response_join_from(&client_public_id, &conn_info);
        self.poll();

        client.handle_challenge_from(&conn_info);
        self.poll();
    }
}

trait AsMutSlice<T> {
    fn as_mut_slice(&mut self) -> &mut [T];
}

struct TestVault {
    inner: Vault,
    _root_dir: TempDir,
    _command_tx: Sender<Command>,
}

impl TestVault {
    /// Create a test Vault within a group.
    fn new(consensus_group: Option<ConsensusGroupRef>, mut rng: TestRng) -> Self {
        let root_dir = unwrap!(TempDir::new("safe_vault"));

        let mut config = Config::default();
        config.set_root_dir(root_dir.path());

        let (command_tx, command_rx) = crossbeam_channel::bounded(0);

        let (routing_node, routing_rx) = if let Some(group) = consensus_group {
            unwrap!(Node::builder().create_within_group(group))
        } else {
            unwrap!(Node::builder().create())
        };
        let inner = unwrap!(Vault::new(
            routing_node,
            routing_rx,
            config,
            command_rx,
            &mut rng
        ));

        Self {
            inner,
            _root_dir: root_dir,
            _command_tx: command_tx,
        }
    }

    fn connection_info(&mut self) -> NodeInfo {
        unwrap!(self.inner.our_connection_info())
    }
}

impl Deref for TestVault {
    type Target = Vault;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TestVault {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl AsMutSlice<TestVault> for TestVault {
    fn as_mut_slice(&mut self) -> &mut [TestVault] {
        slice::from_mut(self)
    }
}

impl AsMutSlice<TestVault> for [TestVault] {
    fn as_mut_slice(&mut self) -> &mut [TestVault] {
        self
    }
}

pub enum FullId {
    Client(ClientFullId),
    App(AppFullId),
}

impl FullId {
    fn sign<T: AsRef<[u8]>>(&self, data: T) -> Signature {
        match self {
            FullId::Client(full_id) => full_id.sign(data),
            FullId::App(full_id) => full_id.sign(data),
        }
    }

    fn public_id(&self) -> PublicId {
        match self {
            FullId::Client(full_id) => PublicId::Client(full_id.public_id().clone()),
            FullId::App(full_id) => PublicId::App(full_id.public_id().clone()),
        }
    }
}

pub trait TestClientTrait {
    fn quic_p2p(&mut self) -> &mut QuicP2p;
    fn rx(&self) -> &Receiver<Event>;
    fn full_id(&self) -> &FullId;
    fn set_connected_vault(&mut self, connected_vault: NodeInfo);
    fn connected_vault(&self) -> NodeInfo;

    fn sign<T: AsRef<[u8]>>(&self, data: T) -> Signature {
        self.full_id().sign(data)
    }

    fn expect_connected_to(&mut self, conn_info: &NodeInfo) {
        match self.rx().try_recv() {
            Ok(Event::ConnectedTo {
                peer: Peer::Node { ref node_info },
            }) if node_info == conn_info => (),
            x => unexpected!(x),
        };
        self.set_connected_vault(conn_info.clone());
    }

    fn send_bootstrap_request(&mut self, client_public_id: &PublicId) {
        let bootstrap_request = HandshakeRequest::Bootstrap(client_public_id.clone());
        self.send(&bootstrap_request);
    }

    fn handle_handshake_response_join_from(
        &mut self,
        client_public_id: &PublicId,
        conn_info: &NodeInfo,
    ) {
        let (sender, bytes) = self.expect_new_message();
        assert_eq!(sender, conn_info.peer_addr);

        let handshake_response: HandshakeResponse = unwrap!(bincode::deserialize(&bytes));
        let _payload = match handshake_response {
            HandshakeResponse::Join(payload) => payload,
            _ => panic!("Unexpected HandshakeResponse"),
        };

        // TODO: For Phase 2B and multiple sections we need handle the payload, disconnect and
        // connect to the new set of elders if necessary.

        let request = HandshakeRequest::Join(client_public_id.clone());
        self.send(&request);
    }

    fn handle_challenge_from(&mut self, conn_info: &NodeInfo) {
        let (sender, bytes) = self.expect_new_message();
        assert_eq!(sender, conn_info.peer_addr);

        let handshake_response: HandshakeResponse = unwrap!(bincode::deserialize(&bytes));
        let payload = match handshake_response {
            // TODO: handle the set of PublicIds sent as part of the challenge response.
            HandshakeResponse::Challenge(_, payload) => payload,
            _ => panic!("Unexpected"),
        };

        let signature = self.full_id().sign(payload);
        let response = HandshakeRequest::ChallengeResult(signature);
        self.send(&response);
    }

    fn expect_new_message(&self) -> (SocketAddr, Bytes) {
        loop {
            match self.rx().try_recv() {
                Ok(Event::SentUserMessage { .. }) => continue,
                Ok(Event::NewMessage { peer_addr, msg }) => return (peer_addr, msg),
                x => unexpected!(x),
            }
        }
    }

    fn expect_no_new_message(&self) {
        loop {
            match self.rx().try_recv() {
                Ok(Event::SentUserMessage { .. }) => continue,
                Err(error) => {
                    assert!(error.is_empty());
                    return;
                }
                x => unexpected!(x),
            }
        }
    }

    fn send<T: Serialize>(&mut self, msg: &T) {
        let msg = unwrap!(bincode::serialize(msg));
        let node_info = self.connected_vault();
        self.quic_p2p()
            .send(Peer::Node { node_info }, Bytes::from(msg), 0)
    }

    fn send_request(&mut self, request: Request) -> MessageId {
        let message_id = MessageId::new();

        let to_sign = (&request, &message_id);
        let to_sign = unwrap!(bincode::serialize(&to_sign));
        let signature = self.full_id().sign(&to_sign);

        let msg = Message::Request {
            request,
            message_id,
            signature: Some(signature),
        };

        self.send(&msg);

        message_id
    }

    fn expect_response(&mut self, expected_message_id: MessageId) -> Response {
        let bytes = self.expect_new_message().1;
        let message: Message = unwrap!(bincode::deserialize(&bytes));

        match message {
            Message::Response {
                message_id,
                response,
            } => {
                assert_eq!(
                    message_id, expected_message_id,
                    "Received Response with unexpected MessageId."
                );
                response
            }
            Message::Request { request, .. } => unexpected!(request),
            Message::Notification { notification } => unexpected!(notification),
        }
    }

    fn expect_notification(&mut self) -> Notification {
        let bytes = self.expect_new_message().1;
        let message: Message = unwrap!(bincode::deserialize(&bytes));

        match message {
            Message::Notification { notification } => notification,
            Message::Request { request, .. } => unexpected!(request),
            Message::Response { response, .. } => unexpected!(response),
        }
    }
}

pub struct TestClient {
    quic_p2p: QuicP2p,
    rx: Receiver<Event>,
    full_id: FullId,
    public_id: ClientPublicId,
    connected_vault: Option<NodeInfo>,
}

impl TestClient {
    fn new_disconnected(rng: &mut TestRng) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let config = quic_p2p::Config {
            our_type: OurType::Client,
            ..Default::default()
        };
        let client_full_id = ClientFullId::new_ed25519(rng);
        let public_id = client_full_id.public_id().clone();

        Self {
            quic_p2p: unwrap!(Builder::new(tx).with_config(config).build()),
            rx,
            full_id: FullId::Client(client_full_id),
            public_id,
            connected_vault: None,
        }
    }

    pub fn public_id(&self) -> &ClientPublicId {
        &self.public_id
    }
}

impl TestClientTrait for TestClient {
    fn quic_p2p(&mut self) -> &mut QuicP2p {
        &mut self.quic_p2p
    }

    fn rx(&self) -> &Receiver<Event> {
        &self.rx
    }

    fn full_id(&self) -> &FullId {
        &self.full_id
    }

    fn set_connected_vault(&mut self, connected_vault: NodeInfo) {
        self.connected_vault = Some(connected_vault);
    }

    fn connected_vault(&self) -> NodeInfo {
        unwrap!(self.connected_vault.clone())
    }
}

impl Deref for TestClient {
    type Target = QuicP2p;

    fn deref(&self) -> &Self::Target {
        &self.quic_p2p
    }
}

impl DerefMut for TestClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.quic_p2p
    }
}

pub struct TestApp {
    quic_p2p: QuicP2p,
    rx: Receiver<Event>,
    full_id: FullId,
    public_id: AppPublicId,
    connected_vault: Option<NodeInfo>,
}

impl TestApp {
    fn new_disconnected(rng: &mut TestRng, owner: ClientPublicId) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let config = quic_p2p::Config {
            our_type: OurType::Client,
            ..Default::default()
        };
        let app_full_id = AppFullId::new_ed25519(rng, owner);
        let public_id = app_full_id.public_id().clone();

        Self {
            quic_p2p: unwrap!(Builder::new(tx).with_config(config).build()),
            rx,
            full_id: FullId::App(app_full_id),
            public_id,
            connected_vault: None,
        }
    }

    pub fn public_id(&self) -> &AppPublicId {
        &self.public_id
    }
}

impl TestClientTrait for TestApp {
    fn quic_p2p(&mut self) -> &mut QuicP2p {
        &mut self.quic_p2p
    }

    fn rx(&self) -> &Receiver<Event> {
        &self.rx
    }

    fn full_id(&self) -> &FullId {
        &self.full_id
    }

    fn set_connected_vault(&mut self, connected_vault: NodeInfo) {
        self.connected_vault = Some(connected_vault);
    }

    fn connected_vault(&self) -> NodeInfo {
        unwrap!(self.connected_vault.clone())
    }
}

impl Deref for TestApp {
    type Target = QuicP2p;

    fn deref(&self) -> &Self::Target {
        &self.quic_p2p
    }
}

impl DerefMut for TestApp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.quic_p2p
    }
}

pub fn get_from_response<T, D>(env: &mut Environment, client: &mut T, request: Request) -> D
where
    T: TestClientTrait,
    D: TryFrom<Response>,
    <D as TryFrom<Response>>::Error: Debug,
{
    let message_id = client.send_request(request);
    env.poll();
    let response = client.expect_response(message_id);
    unwrap!(response.try_into())
}

pub fn perform_mutation<T: TestClientTrait>(
    env: &mut Environment,
    client: &mut T,
    request: Request,
) {
    get_from_response::<_, ()>(env, client, request);
}

pub fn send_request_expect_ok<T, D>(
    env: &mut Environment,
    client: &mut T,
    request: Request,
    expected_success: D,
) where
    T: TestClientTrait,
    D: TryFrom<Response> + Eq + Debug,
    <D as TryFrom<Response>>::Error: Debug,
{
    assert_eq!(expected_success, get_from_response(env, client, request));
}

pub fn send_request_expect_err<T: TestClientTrait>(
    env: &mut Environment,
    client: &mut T,
    request: Request,
    expected_error: Error,
) {
    let expected_response = request.error_response(expected_error);
    let message_id = client.send_request(request);
    env.poll();
    assert_eq!(expected_response, client.expect_response(message_id));
}

pub fn create_balance(
    env: &mut Environment,
    src_client: &mut TestClient,
    dst_client: Option<&mut TestClient>,
    amount: impl IntoCoins,
) {
    let new_balance_owner = match dst_client {
        Some(ref dst_client) => *dst_client.public_id().public_key(),
        None => *src_client.public_id().public_key(),
    };
    let amount = amount.into_coins();
    let transaction_id = 0;

    let message_id = src_client.send_request(Request::CreateBalance {
        new_balance_owner,
        amount,
        transaction_id,
    });
    env.poll();

    let expected = Transaction {
        id: transaction_id,
        amount,
    };

    let notification = dst_client.unwrap_or(src_client).expect_notification();
    assert_eq!(notification, Notification(expected));

    let response = src_client.expect_response(message_id);
    let actual = unwrap!(Transaction::try_from(response));
    assert_eq!(actual, expected);
}

pub fn transfer_coins(
    env: &mut Environment,
    src_client: &mut impl TestClientTrait,
    dst_client: &mut TestClient,
    amount: impl IntoCoins,
    transaction_id: TransactionId,
) {
    let amount = amount.into_coins();

    let message_id = src_client.send_request(Request::TransferCoins {
        destination: *dst_client.public_id().name(),
        amount,
        transaction_id,
    });
    env.poll();

    let expected = Transaction {
        id: transaction_id,
        amount,
    };

    let notification = dst_client.expect_notification();
    assert_eq!(notification, Notification(expected));

    let response = src_client.expect_response(message_id);
    let actual = unwrap!(Transaction::try_from(response));
    assert_eq!(actual, expected);
}

pub fn gen_public_key(rng: &mut TestRng) -> PublicKey {
    *ClientFullId::new_ed25519(rng).public_id().public_key()
}

pub trait IntoCoins {
    fn into_coins(self) -> Coins;
}

impl IntoCoins for Coins {
    fn into_coins(self) -> Coins {
        self
    }
}

impl IntoCoins for u64 {
    fn into_coins(self) -> Coins {
        unwrap!(Coins::from_nano(self))
    }
}

pub fn multiply_coins(coins: Coins, factor: u64) -> Coins {
    unwrap!(Coins::from_nano(coins.as_nano() * factor))
}
