use crate::error::LayerError;
use crate::pattern_handler::ServiceHandler;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// The injected pattern service contains a handler that can be used to interact with the protocol
/// You would use this pattern in situations where you might have an optional protocol upgrade, that
/// you don't necessarily want to send to the downstream service.
/// So for example you could have an http service that sends synchronous requests downstream, but if
/// it upgrades to websocket or some other protocol, then it gets delegated to the internal handler
pub struct InjectedPatternService<InnerService, Handler, Message>
where
    InnerService: Service<Message, Response=Message> + Clone + Send,
    Handler: ServiceHandler<Message> + Clone + Send,
{
    inner: InnerService,
    handler: Handler,
    _phantom_message: PhantomData<Message>,
}

impl<InnerService, Handler, Message> Service<Message> for InjectedPatternService<InnerService, Handler, Message>
where
    InnerService: Service<Message, Response=Message> + Clone + Send + 'static,
    InnerService::Error: Send + 'static,
    InnerService::Future: Send + 'static,
    Handler: ServiceHandler<Message> + Clone + Send + 'static,
    Message: Send + 'static,
{
    type Response = ();
    type Error = LayerError<InnerService::Error>;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|e| LayerError::InnerError(e))
    }

    fn call(&mut self, req: Message) -> Self::Future {
        let mut inner = self.inner.clone();
        let mut handler = self.handler.clone();
        Box::pin(async move {
            // We send whatever input we have to the internal handler
            handler.send_message(req).await;
            // We receive the response from the internal handler
            let response = handler.receive_message().await;
            // We send the response to the inner service
            let response = inner.call(response).await.map_err(|e| LayerError::InnerError(e))?;
            // We send the final result back to the internal handler
            handler.send_message(response).await;
            Ok(())
        })
    }
}

pub struct InjectedPatternLayer<Handler, Message>
where
    Handler: ServiceHandler<Message> + Clone + Send + 'static,
    Message: Send + 'static,
{
    handler: Handler,
    _phantom_message: PhantomData<Message>,
}

impl<Handler, Message> InjectedPatternLayer<Handler, Message>
where
    Handler: ServiceHandler<Message> + Clone + Send + 'static,
    Message: Send + 'static,
{
    pub fn new(handler: Handler) -> Self {
        InjectedPatternLayer {
            handler,
            _phantom_message: PhantomData,
        }
    }
}

impl<InnerService, Handler, Message> Layer<InnerService> for InjectedPatternLayer<Handler, Message>
where
    InnerService: Service<Message, Response=Message> + Clone + Send + 'static,
    InnerService::Error: Send + 'static,
    InnerService::Future: Send + 'static,
    Handler: ServiceHandler<Message> + Clone + Send + 'static,
    Message: Send + 'static,
{
    type Service = InjectedPatternService<InnerService, Handler, Message>;

    fn layer(&self, inner: InnerService) -> Self::Service {
        InjectedPatternService {
            inner,
            handler: self.handler.clone(),
            _phantom_message: PhantomData,
        }
    }
}