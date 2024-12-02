use std::fmt::{Display, Formatter};
use futures::future::BoxFuture;
use hyper::body::Incoming;
use hyper::{Method, Request, Response};

pub struct MyTowerService {

}

impl hyper::service::Service<Request<Incoming>> for MyTowerService {
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
                    Err(MyError{payload: "POST is not allowed".to_string()})
                }
                _ => {
                    Err(MyError { payload: "Method not allowed".to_string() })
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct MyError {
    payload: String,
}

impl Display for MyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Error: {:?}", self))
    }
}

impl std::error::Error for MyError {

}