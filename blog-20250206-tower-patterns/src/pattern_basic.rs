use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// This is the tower service that underpins the Basic Pattern Layer
/// When the Basic Pattern Layer is used, it will create this service, wrapping whatever services are below it
/// The basic pattern accepts an input and provides an output
pub struct BasicPatternService<InnerService, InputType, OutputType, ReturnType>
where
// Because of the future return type (dynamic dispatch, heap allocated), we need to include
// - Clone: we cannot move the inner service out of this service, so we must clone it
// - Send: we are moving the (cloned) service into an async block, so it cannot have internal pointers breaking
// - 'static: the async block may live beyond this service's lifetime, if the service is deallocated while the future is still running
    InnerService: Service<OutputType, Response=ReturnType> + Clone + Send + 'static,
    InputType: Into<OutputType> + Send + 'static,
    OutputType: Send + 'static,
    ReturnType: Send + 'static,
{
    inner: InnerService,
    _phantom_input: PhantomData<InputType>,
    _phantom_output: PhantomData<OutputType>,
    _phantom_return: PhantomData<ReturnType>,
}

impl<InnerService, InputType, OutputType, ReturnType> Service<InputType> for BasicPatternService<InnerService, InputType, OutputType, ReturnType>
where
    InnerService: Service<OutputType, Response=ReturnType> + Clone + Send + 'static,
    InnerService::Error: Send + 'static,
// We need to specify that the future of the service we are wrapping needs to also be send and 'static
    InnerService::Future: Future<Output=Result<ReturnType, InnerService::Error>> + Send + 'static,
    InputType: Into<OutputType> + Send + 'static,
    OutputType: Send + 'static,
    ReturnType: Send + 'static,
{
    type Response = ReturnType;
    type Error = InnerService::Error;
    type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, input: InputType) -> Self::Future {
        let mut inner = self.inner.clone();
        Box::pin(async move {
            inner.call(input.into()).await
        })
    }
}

#[derive(Default)]
pub struct BasicPatternLayer<InputType, OutputType, ReturnType>
where
    InputType: Into<OutputType> + Send + 'static,
    OutputType: Send + 'static,
    ReturnType: Send + 'static,
{
    _phantom_input: PhantomData<InputType>,
    _phantom_output: PhantomData<OutputType>,
    _phantom_return: PhantomData<ReturnType>,
}

impl<InnerService, InputType, OutputType, ReturnType> Layer<InnerService> for BasicPatternLayer<InputType, OutputType, ReturnType>
where
// Because of the future return type (dynamic dispatch, heap allocated), we need to include
// - Clone: we cannot move the inner service out of this service, so we must clone it
// - Send: we are moving the (cloned) service into an async block, so it cannot have internal pointers breaking
// - 'static: the async block may live beyond this service's lifetime, if the service is deallocated while the future is still running
    InnerService: Service<OutputType, Response=ReturnType> + Clone + Send + 'static,
    InputType: Into<OutputType> + Send + 'static,
    OutputType: Send + 'static,
    ReturnType: Send + 'static,
{
    type Service = BasicPatternService<InnerService, InputType, OutputType, ReturnType>;

    fn layer(&self, inner: InnerService) -> Self::Service {
        BasicPatternService { inner, _phantom_input: PhantomData::default(), _phantom_output: PhantomData::default(), _phantom_return: PhantomData::default() }
    }
}
