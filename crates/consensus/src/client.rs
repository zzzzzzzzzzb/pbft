use std::collections::HashMap;
use tracing::{debug, warn};
use tonic::transport::Endpoint;
use crate::error::ConsensusError;
use crate::message::Message;
use crate::message::pbft_client::PbftClient;

pub async fn send(addr: &str, msg: Message) -> Result<(), ConsensusError> {
    let address: Endpoint = addr.parse()?;

    let mut client = PbftClient::connect(address).await?;

    let request = tonic::Request::new(msg);

    let _resp = client.send_message(request).await?;

    Ok(())
}

pub async fn broadcast(local: usize, list: HashMap<usize, String>, msg: Message) {
    for (id, addr) in list {
        if id != local {
            match send(&addr, msg.clone()).await {
                Ok(_) => {
                    debug!("send msg to addr {} success", addr);
                }
                Err(err) => {
                    warn!("send msg to addr {} err: {}", addr, err.to_string());
                }
            }
        }
    }
}