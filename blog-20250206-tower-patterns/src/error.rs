/// An error type that captures errors from the layer but retains errors from the inner service
#[derive(Debug)]
pub enum LayerError<E> {
    #[allow(unused)]
    ServiceLayerError(&'static str),
    InnerError(E),
}

impl<E> From<E> for LayerError<E> {
    fn from(e: E) -> Self {
        LayerError::InnerError(e)
    }
}

