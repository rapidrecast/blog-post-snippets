use crate::Engine;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct BlockingReadSingleEngine {
    lock: Arc<RwLock<()>>,
}

impl Engine for BlockingReadSingleEngine {
    async fn handle_request(&self, request: usize) -> usize {
        let read_lock = self.lock.read().unwrap();
        if read_lock.deref() == &() {
            request + 1
        } else {
            panic!("Lock is not available");
        }
    }

    async fn clone_start(&self) -> Self {
        self.clone()
    }
}
