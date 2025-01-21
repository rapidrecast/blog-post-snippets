pub mod blocking_write_single;
pub mod blocking_read_single;
pub mod async_write_single;
pub mod async_read_single;
pub mod std_channel_single;

use std::ops::Deref;

/// Resembles an API
pub struct RequestHandler<const R: usize, E: Engine> {
    pub engine: E,
}

pub trait Engine: Clone + Send + Default {
    /// Handle the request by adding 1 to the request value
    async fn handle_request(&self, request: usize) -> usize;
    /// Clones and optionally starts a new handler thread
    async fn clone_start(&self) -> Self;
}

impl<const R: usize, E: Engine> RequestHandler<R, E> {
    pub async fn simulate_workload(&self) {
        for i in 0..R {
            assert_eq!(self.engine.handle_request(i).await, i + 1);
            tokio::task::yield_now().await;
        }
    }
}

