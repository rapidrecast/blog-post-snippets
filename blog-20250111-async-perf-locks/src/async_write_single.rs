use crate::Engine;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct AsyncWriteSingleEngine {
    lock: Arc<RwLock<()>>,
}

impl Engine for AsyncWriteSingleEngine {
    async fn handle_request(&self, request: usize) -> usize {
        let lock = self.lock.write().await;
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
