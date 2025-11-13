use tonic::{transport::Server, Request, Response, Status};

use crate::opensnitch_tui::ui_server::Ui;
use crate::opensnitch_tui::ui_server::UiServer;
use crate::opensnitch_tui::{PingRequest, PingReply, Statistics};

use std::sync::mpsc;
use std::thread;

pub mod opensnitch_tui {
    tonic::include_proto!("protocol");
}

#[derive(Debug)]
pub struct MyUI {
    pub tx: mpsc::Sender<Statistics>,
}

#[tonic::async_trait]
impl Ui for MyUI {
    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<PingReply>, Status> {
        let stats: Statistics = request.get_ref().stats.as_ref().unwrap().clone();
        let _ = self.tx.send(stats);

        let reply = PingReply {
            id: request.get_ref().id,
        };

        Ok(Response::new(reply))
    }
}

fn recv_stats(rx: mpsc::Receiver<Statistics>) {
    thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            println!("Got a request: {:?}", msg);
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Unix domain sockets unsupported due to upstream "authority" handling bug
    let addr = "127.0.0.1:50051".parse()?;
    let (tx, rx) = mpsc::channel();

    let myui = MyUI{
        tx: tx,
    };
    recv_stats(rx);

    Server::builder()
        .add_service(UiServer::new(myui))
        .serve(addr)
        .await?;

    Ok(())
}