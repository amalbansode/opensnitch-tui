use std::net::SocketAddr;

use tonic::Streaming;
use tonic::{Request, Response, Status, transport::Server};

use crate::event::{AppEvent, Event};
use crate::opensnitch_proto::pb::ui_server::Ui;
use crate::opensnitch_proto::pb::ui_server::UiServer;
use crate::opensnitch_proto::pb::{
    Alert, ClientConfig, MsgResponse, Notification, NotificationReply, NotificationReplyCode,
    PingReply, PingRequest, Statistics,
};

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;

#[derive(Debug)]
pub struct OpenSnitchUIGrpcServer {
    pub event_sender: mpsc::UnboundedSender<Event>,
    pub app_to_server_notification_sender: Arc<Mutex<mpsc::Sender<Result<Notification, Status>>>>,
}

#[tonic::async_trait]
impl Ui for OpenSnitchUIGrpcServer {
    type NotificationsStream = ReceiverStream<Result<Notification, Status>>;

    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingReply>, Status> {
        let stats: Statistics = request.get_ref().stats.as_ref().unwrap().clone();
        let _ = self.event_sender.send(Event::App(AppEvent::Update(stats)));

        let reply = PingReply {
            id: request.get_ref().id,
        };

        Ok(Response::new(reply))
    }

    async fn post_alert(&self, request: Request<Alert>) -> Result<Response<MsgResponse>, Status> {
        let alert = request.get_ref().clone();
        let _ = self.event_sender.send(Event::App(AppEvent::Alert(alert)));

        let reply = MsgResponse {
            id: request.get_ref().id,
        };

        Ok(Response::new(reply))
    }

    async fn subscribe(
        &self,
        request: Request<ClientConfig>,
    ) -> Result<Response<ClientConfig>, Status> {
        // Be super dumb and reflect back the rx'ed config.
        // In the future, we could use serde_json and fill in the real value for
        // ClientConfig::Config::json(DefaultAction) per some type of tui config.
        let reply = request.get_ref().clone();
        Ok(Response::new(reply))
    }

    async fn notifications(
        &self,
        request: Request<Streaming<NotificationReply>>,
    ) -> Result<Response<Self::NotificationsStream>, Status> {
        let mut in_stream = request.into_inner();
        let (app_to_server_notification_tx, app_to_server_notification_rx) = mpsc::channel(128);
        let tx = self.event_sender.clone();

        tokio::spawn(async move {
            loop {
                let stream_grpc_event = in_stream.message().await;
                match stream_grpc_event {
                    Ok(nominal_grpc_event) => {
                        match nominal_grpc_event {
                            Some(notification) => {
                                if notification.code() == NotificationReplyCode::Error {
                                    let _ = tx.send(Event::App(
                                        AppEvent::NotificationReplyTypeError(notification.data),
                                    ));
                                } else {
                                    // println!("notif OK reply");
                                }
                            }
                            None => {
                                // Stream closed by peer
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        // gRPC error from peer on stream
                        break;
                        // if let Some(io_err) = match_for_io_error(&err) {
                        //     if io_err.kind() == ErrorKind::BrokenPipe {
                        //         // here you can handle special case when client
                        //         // disconnected in unexpected way
                        //         eprintln!("\tclient disconnected: broken pipe");
                        //         break;
                        //     }
                        // }

                        // match tx.send(Err(err)).await {
                        //     Ok(_) => (),
                        //     Err(_err) => break, // response was dropped
                        // }
                    }
                }
            }
            // println!("\tstream ended");
        });

        // Grab a lock on the app to server notification sender, and swaparoo the new sender in.
        // A pre-existing receiver on the old sender should also eventually close since its sender will have closed.
        // abtodo: how does the app know to flush any state related to "prior" peer or should we just live with
        // the desync? maybe send a "flush notifications event" in above async task?
        let mut sender_chan = self.app_to_server_notification_sender.lock().await;
        *sender_chan = app_to_server_notification_tx;

        // Return a stream wrapper over the app to server notifications Receiver.
        let out_stream = Self::NotificationsStream::new(app_to_server_notification_rx);
        Ok(Response::new(out_stream))
    }
}

#[derive(Debug)]
pub struct OpenSnitchUIServer {
    address: SocketAddr,
    event_sender: mpsc::UnboundedSender<Event>,
}

impl OpenSnitchUIServer {
    pub fn new(event_sender: mpsc::UnboundedSender<Event>) -> Self {
        // Unix domain sockets unsupported due to upstream "authority" handling bug
        Self {
            address: "127.0.0.1:50051".parse().unwrap(),
            event_sender: event_sender,
        }
    }

    pub fn spawn_and_run(
        &self,
        app_to_server_notification_sender: &Arc<Mutex<mpsc::Sender<Result<Notification, Status>>>>,
    ) {
        let event_sender_handle = self.event_sender.clone();
        let address = self.address.clone();
        let notification_sender = Arc::clone(&app_to_server_notification_sender);
        tokio::spawn(async move {
            let grpc_server = OpenSnitchUIGrpcServer {
                event_sender: event_sender_handle,
                app_to_server_notification_sender: notification_sender,
            };
            let _ = Server::builder()
                .add_service(UiServer::new(grpc_server))
                .serve(address)
                .await;
        });
    }
}
