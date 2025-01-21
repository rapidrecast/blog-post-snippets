use crate::Engine;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct BlockingWriteSingleEngine {
    lock: Arc<RwLock<()>>,
}

impl Engine for BlockingWriteSingleEngine {
    async fn handle_request(&self, request: usize) -> usize {
        let lock = self.lock.write().unwrap();
        if lock.deref() == &() {
            request + 1
        } else {
            panic!("Lock is not available");
        }
    }

    async fn clone_start(&self) -> Self {
        self.clone()
    }
}
