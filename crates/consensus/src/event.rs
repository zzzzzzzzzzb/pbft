use crate::message::message::Payload;
use crate::message::{Commit, PrePrepare, Prepare};
use crate::{client::broadcast, message::Message};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, info};

pub enum EventType {
    Broadcast = 0,
    Commit = 1,
    Commited = 2,
}

pub struct Event {
    pub msg: Message,
    pub event_type: EventType,
}

impl Event {
    pub fn new_broadcast(node_id: u64, m: Message) -> Self {
        let msg = match m.payload {
            Some(Payload::Request(request)) => Some(Payload::PrePrepare(PrePrepare {
                payload: request.payload,
                signature: vec![],
            })),
            Some(Payload::PrePrepare(pre_prepare)) => Some(Payload::Prepare(Prepare {
                payload: pre_prepare.payload,
                signature: vec![],
            })),
            Some(Payload::Prepare(prepare)) => Some(Payload::Commit(Commit {
                payload: prepare.payload,
                signature: vec![],
            })),
            _ => None,
        };

        Self {
            msg: Message {
                view: m.view,
                seq: m.seq,
                id: node_id,
                digest: String::new(),
                payload: msg,
            },
            event_type: EventType::Broadcast,
        }
    }

    pub fn new_commit(msg: Message) -> Self {
        Self {
            msg,
            event_type: EventType::Commit,
        }
    }

    pub fn new_commited(seq: u64) -> Self {
        let mut msg: Message = Default::default();
        msg.seq = seq;
        Self {
            msg,
            event_type: EventType::Commited,
        }
    }
}

pub struct EventHandler {
    id: usize,
    id_list: HashMap<usize, String>,
    commited_seq: AtomicUsize,
    receiver: Receiver<Event>,
}

impl EventHandler {
    pub fn new(id: usize, id_list: HashMap<usize, String>, receiver: Receiver<Event>) -> Self {
        Self {
            id,
            id_list,
            commited_seq: AtomicUsize::new(0),
            receiver,
        }
    }

    pub async fn start(&mut self) {
        while let Some(event) = self.receiver.recv().await {
            match event.event_type {
                EventType::Broadcast => {
                    broadcast(self.id, self.id_list.clone(), event.msg).await;
                }
                EventType::Commit => {
                    if event.msg.seq == (self.commited_seq.load(Ordering::SeqCst) + 1) as u64 {
                        info!("[COMMITED] view:{} seq:{}", event.msg.view, event.msg.seq);
                        self.commited_seq.fetch_add(1, Ordering::SeqCst);
                    } else {
                        debug!(
                            "[COMMIT]view:{} seq:{} > commited_seq + 1:{}. wait",
                            event.msg.view,
                            event.msg.seq,
                            self.commited_seq.load(Ordering::SeqCst) + 1
                        );
                    }
                }
                EventType::Commited => {}
            }
        }
    }
}
