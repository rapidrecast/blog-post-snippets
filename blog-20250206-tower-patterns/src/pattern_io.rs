use crate::error::LayerError;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{simplex, AsyncReadExt, AsyncWriteExt, ReadHalf, SimplexStream, WriteHalf};
use tokio::task::JoinHandle;
use tower::{Layer, Service};

const MAX_BUF_SIZE: usize = 1024;

pub struct IoTowerService<InnerService>
where
// We are explicit with the types, since we know the implementation we are providing downstream
// However, the downstream service (such as another instance of this layer) can be generic
    InnerService: Service<(ReadHalf<SimplexStream>, WriteHalf<SimplexStream>), Response=()>,
{
    inner: InnerService,
}

impl<InnerService, Reader, Writer> Service<(Reader, Writer)> for IoTowerService<InnerService>
where
    InnerService: Service<(ReadHalf<SimplexStream>, WriteHalf<SimplexStream>), Response=()> + Clone + Send + 'static,
    InnerService::Future: Future<Output=Result<InnerService::Response, InnerService::Error>> + Send + 'static,
    InnerService::Error: Send + 'static,
// We need Unpin because otherwise we cannot access the methods of these traits
    Reader: AsyncReadExt + Send + Unpin + 'static,
    Writer: AsyncWriteExt + Send + Unpin + 'static,
{
    // Since all communication is done via the readers and writers, there isn't really a need for a return type
    type Response = ();
    type Error = LayerError<InnerService::Error>;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map(|result| result.map_err(|err| err.into()))
    }

    fn call(&mut self, (mut input_reader, mut input_writer): (Reader, Writer)) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move {
            // We create the read/write pairs that we will use to communicate with the downstream service
            let (read_svc, mut write_this) = simplex(MAX_BUF_SIZE);
            let (mut read_this, write_svc) = simplex(MAX_BUF_SIZE);

            // Now we spawn the downstream inner service because otherwise we would need to poll it to make it progress
            // Calling await on it directly would block the current task, preventing us from relaying messages
            // Because we have so many generics, my IDE isn't prompting with types, so I declared them explicitly here.
            let task: JoinHandle<Result<InnerService::Response, InnerService::Error>> = tokio::spawn(inner.call((read_svc, write_svc)));

            // Ideally everything below would be a loop, but we won't bother with that
            // We would need to handle more conditions than we would like for the purpose of the example

            // Read from the layer input
            let mut input_read_buffer = [0u8; 1024];
            let result_sz = input_reader.read(&mut input_read_buffer).await;
            let sz = match result_sz {
                Ok(0) | Err(_) => {
                    // The other side has closed the connection
                    Err(LayerError::ServiceLayerError("Failed to read from input reader"))
                }
                Ok(sz) => {
                    Ok(sz)
                }
            }?;

            // Now we will reverse the input we received and send it down to the inner service
            let reversed: Vec<u8> = input_read_buffer[..sz].iter().rev().cloned().collect();
            write_this.write_all(&reversed).await.map_err(|_| LayerError::ServiceLayerError("Failed to write to inner service"))?;

            // Let's now read the response from the downstream service
            let sz = read_this.read(&mut input_read_buffer).await.map_err(|_| LayerError::ServiceLayerError("Failed to read from inner service"))?;
            if sz == 0 {
                // The other side has closed the connection
                return Err(LayerError::ServiceLayerError("Failed to read from inner service"));
            }
            // Let's reverse what the downstream service sent
            let reversed: Vec<u8> = input_read_buffer[..sz].iter().rev().cloned().collect();

            // Finally we will write this back to our input
            input_writer.write_all(&reversed).await.map_err(|_| LayerError::ServiceLayerError("Failed to write to input writer"))?;

            // Technically, we should properly handle the spawned task
            // Thankfully, because we aren't doing things in a loop, we can simplify.
            // By dropping all the handlers we own, the invoking function and the downstream service
            // will know to terminate
            drop(input_reader);
            drop(input_writer);
            drop(read_this);
            drop(write_this);

            // Let's politely wait for the task to complete in case it has errored
            let inner_service_result = task.await.map_err(|_| LayerError::ServiceLayerError("Task failed"))?;
            inner_service_result.map_err(|err| LayerError::InnerError(err))?;
            Ok(())
        })
    }
}

/// I/O Tower Layer takes a (read, write) (as it would for servers) and will also send down
/// a (read, write) pair (as you would do for clients)
#[derive(Default)]
pub struct IoTowerLayer {}

impl<InnerService> Layer<InnerService> for IoTowerLayer
where
    InnerService: Service<(ReadHalf<SimplexStream>, WriteHalf<SimplexStream>), Response=()> + Clone + Send + 'static,
    InnerService::Future: Future<Output=Result<InnerService::Response, InnerService::Error>> + Send + 'static,
    InnerService::Error: Send + 'static,
{
    type Service = IoTowerService<InnerService>;

    fn layer(&self, inner: InnerService) -> Self::Service {
        IoTowerService { inner }
    }
}