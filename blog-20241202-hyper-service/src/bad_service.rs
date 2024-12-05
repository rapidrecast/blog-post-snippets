use crate::error::MyError;
use futures::future::BoxFuture;
use hyper::body::Incoming;
use hyper::{Method, Request, Response};
use std::fmt::Display;

/// An example of a bad tower-esque service that cannot be tested since it uses Incoming and that cannot be constructed directly
pub struct BadTowerService {}

impl hyper::service::Service<Request<Incoming>> for BadTowerService {
    type Response = Response<String>;
    type Error = MyError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Box::pin(async move {
            match req.method().clone() {
                Method::GET => {
                    println!("GET request");
                    Ok(Response::new("GET request".to_string()))
                }
                Method::POST => {
                    println!("POST request");
                    Err(MyError { payload: "POST is not allowed".to_string() })
                }
                _ => {
                    Err(MyError { payload: "Method not allowed".to_string() })
                }
            }
        })
    }
}
