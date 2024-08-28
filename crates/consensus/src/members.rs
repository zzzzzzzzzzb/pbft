use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

pub trait Membership {
    fn is_leader(&self) -> bool;
    fn become_leader(&self);
    fn local_id(&self) -> usize;
    fn members(&self) -> HashMap<usize, String>;
    fn add_node(&self, id: usize, addr: String);
    fn delete_node(&self, id: usize);
}

#[derive(Clone)]
pub struct Members {
    id: usize,
    is_leader: Arc<AtomicBool>,
    list: Arc<Mutex<HashMap<usize, String>>>,
}

impl Members {
    pub fn new(id: usize, is_leader: bool, list: &HashMap<usize, String>) -> Self {
        Self {
            id,
            is_leader: Arc::new(AtomicBool::new(is_leader)),
            list: Arc::new(Mutex::new(list.clone())),
        }
    }
}

impl Membership for Members {
    fn is_leader(&self) -> bool {
        self.is_leader.clone().load(Ordering::SeqCst)
    }

    fn become_leader(&self) {
        let _ = self.is_leader.clone().compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        );
    }

    fn local_id(&self) -> usize {
        self.id
    }

    fn members(&self) -> HashMap<usize, String> {
        if let Ok(lock) = self.list.clone().try_lock() {
            return lock.clone();
        }
        HashMap::new()
    }

    fn add_node(&self, id: usize, addr: String) {
        if let Ok(mut lock) = self.list.clone().try_lock() {
            if !lock.contains_key(&id) {
                lock.insert(id, addr);
            }
        }
    }

    fn delete_node(&self, id: usize) {
        if let Ok(mut lock) = self.list.clone().try_lock() {
            if lock.contains_key(&id) {
                lock.remove(&id);
            }
        }
    }
}
