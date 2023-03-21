use super::NodeCtl;

use sn_node::node::NodeRef;

use color_eyre::eyre::{ErrReport, Result};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{
    mpsc::{self, Sender},
    RwLock,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Code, Request, Response, Status};
use tracing::{debug, info, trace};

use safenode::safe_node_server::{SafeNode, SafeNodeServer};
use safenode::{
    NodeEvent as RpcNodeEvent, NodeEventsRequest, NodeInfoRequest, NodeInfoResponse, Peer,
    RestartRequest, RestartResponse, SectionMembersRequest, SectionMembersResponse, StopRequest,
    StopResponse,
};

// this would include code generated from .proto file
#[allow(unused_qualifications, clippy::unwrap_used)]
pub mod safenode {
    tonic::include_proto!("safenode");
}

// Defining a struct to hold information used by our gRPC service backend
pub struct SafeNodeRpcService {
    addr: SocketAddr,
    log_dir: String,
    node_ref: Arc<RwLock<NodeRef>>,
    ctl_tx: Sender<NodeCtl>,
}

// Implementing RPC interface for service defined in .proto
#[tonic::async_trait]
impl SafeNode for SafeNodeRpcService {
    type NodeEventsStream = ReceiverStream<Result<RpcNodeEvent, Status>>;

    async fn node_info(
        &self,
        request: Request<NodeInfoRequest>,
    ) -> Result<Response<NodeInfoResponse>, Status> {
        trace!(
            "RPC request received at {}: {:?}",
            self.addr,
            request.get_ref()
        );
        let context = &self.node_ref.read().await.context;
        let resp = Response::new(NodeInfoResponse {
            node_name: context.name().0.to_vec(),
            is_elder: context.is_elder(),
            log_dir: self.log_dir.clone(),
        });

        Ok(resp)
    }

    async fn section_members(
        &self,
        request: Request<SectionMembersRequest>,
    ) -> Result<Response<SectionMembersResponse>, Status> {
        trace!(
            "RPC request received at {}: {:?}",
            self.addr,
            request.get_ref()
        );
        let network_knowledge = self
            .node_ref
            .read()
            .await
            .context
            .network_knowledge()
            .clone();
        let peers = network_knowledge
            .members()
            .into_iter()
            .map(|node_id| Peer {
                node_name: node_id.name().0.to_vec(),
                is_elder: network_knowledge.is_elder(&node_id.name()),
                addr: format!("{}", node_id.addr()),
            })
            .collect();

        let resp = Response::new(SectionMembersResponse { peers });

        Ok(resp)
    }

    async fn node_events(
        &self,
        request: Request<NodeEventsRequest>,
    ) -> Result<Response<Self::NodeEventsStream>, Status> {
        trace!(
            "RPC request received at {}: {:?}",
            self.addr,
            request.get_ref()
        );

        let (client_tx, client_rx) = mpsc::channel(4);

        let mut events_rx = self.node_ref.read().await.events_channel.subscribe();
        let _handle = tokio::spawn(async move {
            while let Ok(event) = events_rx.recv().await {
                let event = RpcNodeEvent {
                    event: format!("Event-{event}"),
                };

                if let Err(err) = client_tx.send(Ok(event)).await {
                    debug!(
                        "Dropping stream sender to RPC client due to failure in \
                        last attempt to notify an event: {err}"
                    );
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(client_rx)))
    }

    async fn stop(&self, request: Request<StopRequest>) -> Result<Response<StopResponse>, Status> {
        trace!(
            "RPC request received at {}: {:?}",
            self.addr,
            request.get_ref()
        );

        let err = ErrReport::msg("Node has been stopped by an RPC request");
        match self.ctl_tx.send(NodeCtl::Stop(err)).await {
            Ok(()) => Ok(Response::new(StopResponse {})),
            Err(err) => Err(Status::new(
                Code::Internal,
                format!("Failed to stop the node: {err}"),
            )),
        }
    }

    async fn restart(
        &self,
        request: Request<RestartRequest>,
    ) -> Result<Response<RestartResponse>, Status> {
        trace!(
            "RPC request received at {}: {:?}",
            self.addr,
            request.get_ref()
        );

        match self
            .ctl_tx
            .send(NodeCtl::Restart(Duration::from_secs(2)))
            .await
        {
            Ok(()) => Ok(Response::new(RestartResponse {})),
            Err(err) => Err(Status::new(
                Code::Internal,
                format!("Failed to restart the node: {err}"),
            )),
        }
    }
}

pub(super) fn start_rpc_service(
    addr: SocketAddr,
    log_dir: String,
    node_ref: Arc<RwLock<NodeRef>>,
    ctl_tx: Sender<NodeCtl>,
) {
    // creating a service
    let service = SafeNodeRpcService {
        addr,
        log_dir,
        node_ref,
        ctl_tx,
    };
    info!("RPC Server listening on {addr}");
    println!("RPC Server listening on {addr}");

    let _handle = tokio::spawn(async move {
        // adding our service to our server.
        Server::builder()
            .add_service(SafeNodeServer::new(service))
            .serve(addr)
            .await
    });
}
