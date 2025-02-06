use crate::pattern_handler::ServiceHandler;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub struct DataTypeA(pub u8);
pub struct DataTypeB(pub u32);

impl Into<DataTypeB> for DataTypeA {
    fn into(self) -> DataTypeB {
        DataTypeB(self.0 as u32)
    }
}

impl Into<DataTypeA> for DataTypeB {
    fn into(self) -> DataTypeA {
        DataTypeA(self.0 as u8)
    }
}

/// A handler that holds the latest value received and increments every time its requested
/// We implement Clone so that the invoker still has access to it.
/// Equally, we could return IncrementingHandler from the HandlerPatternLayer, or we could have the
/// ServiceHandler implementation modify data that the invoker owns
#[derive(Clone)]
pub struct IncrementingHandler {
    pub value: Arc<AtomicU32>,
}

impl IncrementingHandler {
    pub fn new(initial_value: u32) -> Self {
        IncrementingHandler {
            value: Arc::new(AtomicU32::new(initial_value)),
        }
    }
}

impl ServiceHandler<u32> for IncrementingHandler {
    async fn send_message(&mut self, msg: u32) -> () {
        self.value.store(msg, Ordering::SeqCst);
    }

    /// Every time a message is requested, we increment the stored value
    async fn receive_message(&mut self) -> u32 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }
}