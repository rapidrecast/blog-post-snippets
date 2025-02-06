use crate::error::LayerError;
use std::future::Future;
use std::marker::PhantomData;
use std::ops::AddAssign;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// The ServiceHandler works similarly to how a tower Service would work
/// The difference is that using this pattern you can combine it with other layers in between
/// instead of relying just on the tower Service handling the message
pub trait ServiceHandler<Message> {
    fn send_message(&mut self, msg: Message) -> impl Future<Output=()> + Send;
    fn receive_message(&mut self) -> impl Future<Output=Message> + Send;
}

pub struct HandlerPatternService<InnerService, OutputHandler, Message>
where
    InnerService: Service<(), Response=OutputHandler>,
    OutputHandler: ServiceHandler<Message>,
{
    inner: InnerService,
    _phantom_output_handler: PhantomData<OutputHandler>,
    _phantom_message: PhantomData<Message>,
}

impl<InnerService, InputHandler, OutputHandler, Message> Service<InputHandler> for HandlerPatternService<InnerService, OutputHandler, Message>
where
    InnerService: Service<(), Response=OutputHandler> + Clone + Send + 'static,
    InnerService::Future: Future<Output=Result<OutputHandler, InnerService::Error>> + Send,
    InputHandler: ServiceHandler<Message> + Send + 'static,
// We do not need to declare 'static because we are no longer spawning
    OutputHandler: ServiceHandler<Message> + Send,
    Message: AddAssign + Send,
{
    type Response = ();
    type Error = LayerError<InnerService::Error>;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| LayerError::InnerError(e))
    }

    fn call(&mut self, mut input_handler: InputHandler) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move {
            // We get the handler from the downstream service
            // Normally, you wouldn't do this, as it could be implemented by a tower Service internally
            // but by having it as a layer, you can do transformations to the message before it is
            // received by the mediating layer
            let mut inner_handler = inner.call(()).await.map_err(|e| LayerError::InnerError(e))?;

            // Lets receive some input and send it to the downstream service
            let input = input_handler.receive_message().await;
            inner_handler.send_message(input).await;

            // Now lets get some output from the downstream service and modify it
            let mut first = inner_handler.receive_message().await;
            let second = inner_handler.receive_message().await;
            first += second;

            // Now we will send it back to the input handler
            input_handler.send_message(first).await;
            Ok(())
        })
    }
}

pub struct HandlerPatternLayer<Message>
where
    Message: AddAssign + Send,
{
    _phantom_message: PhantomData<Message>,
}

impl<Message> HandlerPatternLayer<Message>
where
    Message: AddAssign + Send,
{
    pub fn new() -> Self {
        HandlerPatternLayer {
            _phantom_message: PhantomData,
        }
    }
}

impl<InnerService, OutputHandler, Message> Layer<InnerService> for HandlerPatternLayer<Message>
where
    InnerService: Service<(), Response=OutputHandler> + Clone + Send + 'static,
    InnerService::Future: Future<Output=Result<OutputHandler, InnerService::Error>> + Send,
    OutputHandler: ServiceHandler<Message> + Send,
    Message: AddAssign + Send,
{
    type Service = HandlerPatternService<InnerService, OutputHandler, Message>;

    fn layer(&self, inner: InnerService) -> Self::Service {
        HandlerPatternService {
            inner,
            _phantom_output_handler: PhantomData,
            _phantom_message: PhantomData,
        }
    }
}