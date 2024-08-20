use crate::{client::broadcast, message::Message};
use std::collections::HashMap;
use tokio::sync::mpsc::Receiver;

pub struct EventHandler {
    id: usize,
    id_list: HashMap<usize, String>,
    receiver: Receiver<Message>,
}

impl EventHandler {
    pub fn new(id: usize, id_list: HashMap<usize, String>, receiver: Receiver<Message>) -> Self {
        Self {
            id,
            id_list,
            receiver,
        }
    }

    pub async fn start(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            broadcast(self.id, self.id_list.clone(), message).await;
        }
    }
}
