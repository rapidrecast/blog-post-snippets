use crate::Engine;
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// StdChannelSingleEngine handles batches of 1 but does not use locks
#[derive(Clone)]
pub struct StdChannelSingleEngine {
    /// The channel where requests are received by a background loop and responses are sent
    request_receiver: Arc<RwLock<Receiver<Box<(usize, SyncSender<usize>)>>>>,
    /// The channel to which requests are sent
    request_sender: Arc<SyncSender<Box<(usize, SyncSender<usize>)>>>,
    handlers: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl Default for StdChannelSingleEngine {
    fn default() -> Self {
        let (request_sender, request_receiver) = std::sync::mpsc::sync_channel(100);
        StdChannelSingleEngine {
            // We use a RwLock so that futures using it are send. There is only 1 write acquired ever.
            request_receiver: Arc::new(RwLock::new(request_receiver)),
            request_sender: Arc::new(request_sender),
            handlers: Arc::new(RwLock::new(None)),
        }
    }
}

impl Engine for StdChannelSingleEngine {
    async fn handle_request(&self, request: usize) -> usize {
        // Create this request channel callback pair
        let (response_sender, response_receiver) = std::sync::mpsc::sync_channel(1);
        self.request_sender.send(Box::from((request, response_sender))).unwrap();
        loop {
            if let Ok(response) = response_receiver.try_recv() {
                return response;
            }
            tokio::task::yield_now().await;
        }
    }

    async fn clone_start(&self) -> Self {
        let lock = self.handlers.try_read().unwrap();
        let lock = lock.is_none();
        if lock {
            let lock = self.handlers.write().await;
            let task = tokio::spawn(handle_forever(self.request_receiver.clone()));
            *lock = Some(task);
        }
        StdChannelSingleEngine {
            request_receiver: self.request_receiver.clone(),
            request_sender: self.request_sender.clone(),
            handlers: self.handlers.clone(),
        }
    }
}

pub async fn handle_forever(request_receiver: Arc<RwLock<Receiver<Box<(usize, SyncSender<usize>)>>>>) {
    loop {
        let recv = request_receiver.try_write().unwrap();
        let res = recv.try_recv();
        let poll_failed = match res {
            Ok(the_box) => {
                let (req, request_sender) = *the_box;
                request_sender.send(req).unwrap();
                None
            }
            Err(e) => {
                Some(e)
            }
        };
        match poll_failed {
            None => {
                // Poll again, we have data
            }
            Some(TryRecvError::Empty) => {
                drop(recv);
                tokio::task::yield_now().await;
            }
            Some(TryRecvError::Disconnected) => {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::std_channel_single::handle_forever;

    #[test]
    pub fn handle_forever_is_send() {
        fn is_send<T: Send>(_: T) {}
        let fut = handle_forever(Default::default());
        is_send(fut);
    }
}