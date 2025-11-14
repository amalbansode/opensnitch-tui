use std::net::SocketAddr;

use tonic::{Request, Response, Status, transport::Server};

use crate::event::{AppEvent, Event};
use crate::opensnitch_proto::pb::ui_server::Ui;
use crate::opensnitch_proto::pb::ui_server::UiServer;
use crate::opensnitch_proto::pb::{Alert, MsgResponse, PingReply, PingRequest, Statistics};

use tokio::sync::mpsc;

#[derive(Debug)]
pub struct OpenSnitchUIGrpcServer {
    pub sender: mpsc::UnboundedSender<Event>,
}

#[tonic::async_trait]
impl Ui for OpenSnitchUIGrpcServer {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingReply>, Status> {
        let stats: Statistics = request.get_ref().stats.as_ref().unwrap().clone();
        let _ = self.sender.send(Event::App(AppEvent::Update(stats)));

        let reply = PingReply {
            id: request.get_ref().id,
        };

        Ok(Response::new(reply))
    }

    async fn post_alert(&self, request: Request<Alert>) -> Result<Response<MsgResponse>, Status> {
        let alert = request.get_ref().clone();
        let _ = self.sender.send(Event::App(AppEvent::Alert(alert)));

        let reply = MsgResponse {
            id: request.get_ref().id,
        };

        Ok(Response::new(reply))
    }
}

#[derive(Debug)]
pub struct OpenSnitchUIServer {
    address: SocketAddr,
    sender: mpsc::UnboundedSender<Event>,
}

impl OpenSnitchUIServer {
    pub fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        // Unix domain sockets unsupported due to upstream "authority" handling bug
        Self {
            address: "127.0.0.1:50051".parse().unwrap(),
            sender: sender,
        }
    }

    pub fn spawn_and_run(&self) {
        let sender_handle = self.sender.clone();
        let address = self.address.clone();
        tokio::spawn(async move {
            let grpc_server = OpenSnitchUIGrpcServer {
                sender: sender_handle,
            };
            let _ = Server::builder()
                .add_service(UiServer::new(grpc_server))
                .serve(address)
                .await;
        });
    }
}
