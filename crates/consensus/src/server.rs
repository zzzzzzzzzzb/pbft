use std::collections::HashMap;
use tokio::sync::{mpsc};
use tokio::sync::mpsc::{Sender};
use tonic::Response;
use tonic::transport::Server as TransportServer;
use tracing::{debug, error, info};
use crate::error::ConsensusError;
use crate::event::EventHandler;
use crate::message::{Message, MessageResponse};
use crate::message::pbft_server::{Pbft, PbftServer};
use crate::pool::{RequestHandler};

pub struct Server {
    sender: Sender<Message>,
}

impl Server {
    async fn request(&self, msg: Message) -> Result<(), ConsensusError>{
        if let Err(e) = self.sender.send(msg).await {
            error!("send request to pool err: {}", e)
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl Pbft for Server {
    async fn send_message(
        &self,
        request: tonic::Request<Message>,
    ) -> std::result::Result<tonic::Response<MessageResponse>, tonic::Status> {
        match self.request(request.into_inner()).await{
            Ok(_) => {}
            Err(err) => {
                let reply = MessageResponse {
                    message: err.to_string(),
                };
                return Ok(Response::new(reply))
            }
        };

        Ok(Response::new(MessageResponse {
            message: String::from("success"),
        }))
    }
}

pub async fn run(id: usize, is_leader: bool, id_list: HashMap<usize, String>, address: String) -> Result<(), ConsensusError> {
    let addr = address.parse()?;

    let (tx_req, rv_req) = mpsc::channel(1024); // request

    let (tx_event, rv_event) = mpsc::channel(1024); // event

    let server = Server {
        sender: tx_req,
    };

    let mut request_handler = RequestHandler::new(id.clone(), is_leader, id_list.clone(), rv_req, 10, tx_event);

    let mut event_handler = EventHandler::new(id.clone(), id_list.clone(), rv_event);

    let task_req = tokio::spawn(async move {
        debug!("request handler starting...");
        request_handler.start().await;
    });

    let task_event = tokio::spawn(async move {
        debug!("event handler starting...");
        event_handler.start().await;
    });

    let task_server = tokio::spawn(async move {
        info!("PBFT server listening on {}...", addr);
        if let Err(e)= TransportServer::builder()
            .add_service(PbftServer::new(server))
            .serve(addr)
            .await {
            error!("start pbft server err: {}", e);
        }
    });

    let _ = tokio::join!(task_req, task_event, task_server);

    Ok(())
}