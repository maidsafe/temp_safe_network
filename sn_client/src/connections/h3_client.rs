// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::Result;

use super::Session;
use sn_interface::types::Peer;

use bytes::Bytes;
use h3_quinn::{self, quinn, quinn::crypto::rustls::Error};
use rustls::{self, client::ServerCertVerified, Certificate, ServerName};
use std::{sync::Arc, time::SystemTime};

//static ALPN: &[u8] = b"h3";
static ALPN: &[&[u8]] = &[b"h3", b"h3-29", b"h3-28", b"h3-27"];

pub(super) async fn send_on_h3(
    session: Session,
    peer: Peer,
    wire_msg_bytes: Bytes,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut peer_addr = peer.addr();
    peer_addr.set_port(peer_addr.port() + 1);
    let dest = format!("https://{}", peer_addr).parse::<http::Uri>()?;

    let auth = dest
        .authority()
        .ok_or("destination must have a host")?
        .clone();

    // dns me!
    let addr = tokio::net::lookup_host((auth.host(), peer_addr.port()))
        .await?
        .next()
        .ok_or("dns found no addresses")?;

    info!(">>>>> H3: DNS Lookup for {:?}: {:?}", dest, addr);

    // quinn setup
    let tls_config_builder = rustls::ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&rustls::version::TLS13])?;
    let mut tls_config = tls_config_builder
        .with_custom_certificate_verifier(Arc::new(YesVerifier))
        .with_no_client_auth();
    tls_config.enable_early_data = true;
    //tls_config.alpn_protocols = vec![ALPN.into()];
    tls_config.alpn_protocols = ALPN.iter().map(|&x| x.into()).collect();
    let client_config = quinn::ClientConfig::new(Arc::new(tls_config));

    let mut client_endpoint = h3_quinn::quinn::Endpoint::client("[::]:0".parse().unwrap())?;
    client_endpoint.set_default_client_config(client_config);
    let quinn_conn = h3_quinn::Connection::new(client_endpoint.connect(addr, auth.host())?.await?);

    info!(
        ">>>>> H3: QUIC connected from {}...",
        client_endpoint.local_addr()?
    );

    // generic h3
    let (mut _driver, mut send_request) = h3::client::new(quinn_conn).await?;

    /*
    let mut tasks = Vec::default();
    let drive = tokio::spawn(async move {
        futures::future::poll_fn(|cx| driver.poll_close(cx))
            .await
            .unwrap();
    });
    tasks.push(drive);
    */

    info!(
        ">>>>> H3: Sending request {} bytes ...",
        wire_msg_bytes.len()
    );

    let req = http::Request::builder().uri(dest).body(()).unwrap();

    let mut stream = send_request.send_request(req).await.unwrap();

    stream.send_data(wire_msg_bytes.clone()).await.unwrap();

    stream.finish().await.unwrap();
    info!(
        ">>>>> H3: Finished sending request {} bytes ...",
        wire_msg_bytes.len()
    );

    Session::spawn_h3_msg_listener_thread(
        session, peer, stream, /*send_request, client_endpoint.clone()*/
    );

    //let _ = futures::future::join_all(tasks).await;
    client_endpoint.wait_idle().await;
    info!(">>>>> H3: Sending request Finished!",);

    Ok(())
}

struct YesVerifier;

impl rustls::client::ServerCertVerifier for YesVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}
