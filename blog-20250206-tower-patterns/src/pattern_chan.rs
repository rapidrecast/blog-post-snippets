use crate::error::LayerError;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use tower::{Layer, Service};

/// ChannelPatternService is the service is similar to the IOPatternService
pub struct ChannelPatternService<InnerService, OutputType>
where
    InnerService: Service<(Receiver<OutputType>, Sender<OutputType>)> + Clone + Send + 'static,
{
    inner: InnerService,
    _phantom_output: PhantomData<OutputType>,
}

impl<InnerService, InputType, OutputType> Service<(Receiver<InputType>, Sender<InputType>)> for ChannelPatternService<InnerService, OutputType>
where
// We force the response to be (), but it could be generic and handled in some way
    InnerService: Service<(Receiver<OutputType>, Sender<OutputType>), Response=()> + Clone + Send + 'static,
// We need to declare that the inner service's future is also Send and 'static
// This is because we are sending it to a tokio::spawn call, which requires it to be Send
// and 'static is because the future may outlive this layers lifetime
    InnerService::Future: Future<Output=Result<InnerService::Response, InnerService::Error>> + Send + 'static,
// We also need to make the Error type Send and static, for the same reason
    InnerService::Error: Send + 'static,
    InputType: Into<OutputType> + Send + 'static,
    OutputType: Into<InputType> + Send + 'static,
{
    // Since we are given (Receiver, Sender), we are constantly giving the return values, so we
    // don't have a need for a response type
    type Response = ();
    type Error = LayerError<InnerService::Error>;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| LayerError::InnerError(e))
    }

    fn call(&mut self, (mut input_receiver, mut input_sender): (Receiver<InputType>, Sender<InputType>)) -> Self::Future {
        // The implementation is similar to the IOPatternService
        // We are also doing the same translation of types as we have in the basic service
        let mut inner = self.inner.clone();
        Box::pin(async move {
            // First we will create the channel pairs to communicate with the inner service
            let (sx_svc, mut rx_this) = channel::<OutputType>(1);
            let (sx_this, rx_svc) = channel::<OutputType>(1);

            // Now we will spawn the inner service so it can process in parallel without blocking us
            // We don't need to declare the type explicitly, but it helps the IDE figure things out :)
            let task: JoinHandle<Result<InnerService::Response, InnerService::Error>> = tokio::spawn(inner.call((rx_svc, sx_svc)));

            // Ideally, what follows would be in a loop, but we will simplify it for the example

            // We ready from the input
            let input_message = input_receiver.recv().await;
            let input_message = input_message.ok_or(LayerError::ServiceLayerError("Failed to receive message from input receiver"))?;

            // Send the translated message downstream to the inner service
            sx_this.send(input_message.into()).await.map_err(|_| LayerError::ServiceLayerError("Failed to send message to inner service"))?;

            // Now we wait for the response from the downstream service
            let response = rx_this.recv().await.ok_or(LayerError::ServiceLayerError("Failed to receive response from inner service"))?;

            // And finally send it back to the caller
            input_sender.send(response.into()).await.map_err(|_| LayerError::ServiceLayerError("Failed to send response to caller"))?;

            // Here we do some cleanup to make sure everything shuts down correctly
            drop(input_sender);
            drop(input_receiver);
            drop(rx_this);
            drop(sx_this);
            task.await
                .map_err(|_join_error| LayerError::ServiceLayerError("Failed to join inner service task"))?
                .map_err(|e| LayerError::InnerError(e))?;

            Ok(())
        })
    }
}

#[derive(Default)]
pub struct ChannelPatternLayer<OutputType>
where
    OutputType: Send + 'static,
{
    _phantom_output: PhantomData<OutputType>,
}

impl<OutputType> ChannelPatternLayer<OutputType>
where
    OutputType: Send + 'static,
{
    pub fn new() -> Self {
        ChannelPatternLayer {
            _phantom_output: PhantomData,
        }
    }
}

impl<InnerService, OutputType> Layer<InnerService> for ChannelPatternLayer<OutputType>
where
    InnerService: Service<(Receiver<OutputType>, Sender<OutputType>)> + Clone + Send + 'static,
    OutputType: Send + 'static,
{
    type Service = ChannelPatternService<InnerService, OutputType>;

    fn layer(&self, inner: InnerService) -> Self::Service {
        ChannelPatternService {
            inner,
            _phantom_output: PhantomData,
        }
    }
}