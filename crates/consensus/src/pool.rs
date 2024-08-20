use crate::message::{message::Payload, Commit, Message, PrePrepare, Prepare};
use std::{
    collections::{hash_map, HashMap},
    sync::Arc,
};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};
use tracing::{debug, error, info, warn};

struct SeqMessage {
    pre_prepare: HashMap<usize, PrePrepare>,
    prepare: HashMap<usize, Prepare>,
    commit: HashMap<usize, Commit>,
}

pub struct Pool {
    id: usize,
    is_leader: bool,
    id_list: HashMap<usize, String>,

    view: usize,
    stable_checkpoint: usize,

    capacity: usize,
    queue: Vec<SeqMessage>,
    start: usize,

    event_sender: Sender<Message>,
}

pub struct RequestHandler {
    message_pool: Arc<Mutex<Pool>>,
    receiver: Receiver<Message>,
}

impl RequestHandler {
    pub fn new(
        id: usize,
        is_leader: bool,
        id_list: HashMap<usize, String>,
        receiver: Receiver<Message>,
        capacity: usize,
        sender: Sender<Message>,
    ) -> Self {
        let mut b: Vec<SeqMessage> = Vec::new();
        for _ in 0..capacity {
            b.push(SeqMessage {
                pre_prepare: HashMap::new(),
                prepare: HashMap::new(),
                commit: HashMap::new(),
            })
        }
        Self {
            receiver,
            message_pool: Arc::new(Mutex::new(Pool {
                id,
                is_leader,
                id_list,
                view: 1,
                stable_checkpoint: 1,
                capacity,
                queue: b,
                start: 0,
                event_sender: sender,
            })),
        }
    }

    pub async fn start(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            if let Ok(mut lock) = self.message_pool.clone().try_lock() {
                lock.add(message).await;
            }
        }
    }
}

impl Pool {
    async fn add(&mut self, m: Message) {
        let m_view = m.view as usize;
        let m_seq = m.seq as usize;

        if !self.view_seq_check(m_view, m_seq) {
            return;
        }

        let index = self.index_in_queue(m_seq);

        match m.payload {
            Some(Payload::Request(ref request)) => {
                debug!(
                    "[REQUEST] received request. view:{}, sequence:{}",
                    m_view, m_seq
                );
                if self.is_leader {
                    info!(
                        "[REQUEST]is leader, broadcast pre-prepare message. view:{}, sequence:{}",
                        m_view, m_seq
                    );
                    let pre_prepare = PrePrepare {
                        payload: request.clone().payload,
                        signature: vec![],
                    };
                    let _ = self.queue[index]
                        .pre_prepare
                        .insert(m.id as usize, pre_prepare);

                    self.event(self.transfer(m.clone())).await;
                } else {
                    debug!("not leader, do nothing");
                }
            }
            Some(Payload::PrePrepare(ref pre_prepare)) => {
                debug!(
                    "[PRE-PREPARE] received pre-prepare message from node{}. view:{}, sequence:{}",
                    m.id, m_view, m_seq
                );
                if !self.is_pre_prepared(index) {
                    let _ = self.queue[index]
                        .pre_prepare
                        .insert(m.id as usize, pre_prepare.clone());
                    info!(
                        "[PRE-PREPARE] view:{}, sequence:{} pre-prepared",
                        m_view, m_seq
                    );
                    self.event(self.transfer(m.clone())).await;
                }
            }
            Some(Payload::Prepare(ref prepare)) => {
                debug!(
                    "[PREPARE] received prepare message from node{}. view:{}, sequence:{}",
                    m.id, m_view, m_seq
                );
                if let hash_map::Entry::Vacant(e) = self.queue[index].prepare.entry(m.id as usize) {
                    e.insert(prepare.clone());
                }
                if self.is_prepared(index) {
                    info!("[PREPARE] view:{}, sequence:{} prepared", m_view, m_seq);
                    self.event(self.transfer(m.clone())).await;
                }
            }
            Some(Payload::Commit(ref commit)) => {
                debug!(
                    "[COMMIT] received commit message from node{}. view:{}, sequence:{}",
                    m.id, m_view, m_seq
                );
                if let hash_map::Entry::Vacant(e) = self.queue[index].commit.entry(m.id as usize) {
                    e.insert(commit.clone());
                }
                if self.is_commited(index) {
                    // TODO
                    info!("[COMMIT] view:{}, sequence:{} commited", m_view, m_seq);
                }
            }
            _ => {
                error!("no such message type");
            }
        }
    }

    async fn event(&self, m: Message) {
        if let Err(e) = self.event_sender.send(m).await {
            error!("event sender error:{}", e);
        }
    }

    fn is_pre_prepared(&self, index: usize) -> bool {
        !self.queue[index].pre_prepare.is_empty()
    }

    fn is_prepared(&self, index: usize) -> bool {
        self.is_pre_prepared(index) && self.counts_prepare(index) >= self.bft_node_num()
    }

    fn is_commited(&self, index: usize) -> bool {
        self.is_prepared(index) && self.counts_commit(index) >= self.bft_node_num()
    }

    fn counts_prepare(&self, index: usize) -> usize {
        self.queue[index].prepare.len()
    }
    fn counts_commit(&self, index: usize) -> usize {
        self.queue[index].commit.len()
    }
    fn bft_node_num(&self) -> usize {
        // 2f
        (self.id_list.len() * 2) / 3
    }
    fn index_in_queue(&self, seq: usize) -> usize {
        seq - self.stable_checkpoint + self.start
    }
    fn view_seq_check(&self, view: usize, seq: usize) -> bool {
        if seq <= self.stable_checkpoint || seq >= self.stable_checkpoint + self.capacity {
            warn!("seq <= stable_checkpoint || seq >= self.stable_checkpoint + self.capacity");
            return false;
        }
        if view != self.view {
            warn!("view != self.view");
            return false;
        }
        true
    }

    fn transfer(&self, m: Message) -> Message {
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

        Message {
            view: m.view,
            seq: m.seq,
            id: self.id as u64,
            digest: String::new(),
            payload: msg.clone(),
        }
    }
}
